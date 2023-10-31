// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::cell::RefCell;

use bitflags::bitflags;
use smithay::{
	desktop::{Space, Window},
	input::{
		pointer::{
			self,
			GestureHoldBeginEvent,
			GestureHoldEndEvent,
			GesturePinchBeginEvent,
			GesturePinchEndEvent,
			GesturePinchUpdateEvent,
			GestureSwipeBeginEvent,
			GestureSwipeEndEvent,
			GestureSwipeUpdateEvent,
			PointerGrab,
		},
		SeatHandler,
	},
	reexports::{wayland_protocols::xdg::shell::server::xdg_toplevel, wayland_server::protocol::wl_surface::WlSurface},
	utils::Logical as LogicalSpace,
	wayland::{compositor, shell::xdg::SurfaceCachedState},
};
use tracing::{event, span, Level};

use super::PRIMARY_BUTTON;
use crate::wayland::state;

type Rectangle<N = i32, Space = LogicalSpace> = smithay::utils::Rectangle<N, Space>;
type Size<N = i32, Space = LogicalSpace> = smithay::utils::Size<N, Space>;
type Point<N = i32, Space = LogicalSpace> = smithay::utils::Point<N, Space>;

type PointerInnerHandle<'handle, State = state::AquariWm> = pointer::PointerInnerHandle<'handle, State>;
type Focus<State = state::AquariWm, N = i32, Space = LogicalSpace> =
	(<State as SeatHandler>::PointerFocus, Point<N, Space>);

bitflags! {
	/// The edge of a window that is to be resized.
	#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
	// RustRover currently has a false negative here.
	pub struct ResizeEdge: u32 {
		/// The top side of the window is to be resized.
		const TOP = 0b0001;
		/// The bottom side of the window is to be resized.
		const BOTTOM = 0b0010;
		/// The left side of the window is to be resized.
		const LEFT = 0b0100;
		/// The right side of the window is to be resized.
		const RIGHT = 0b1000;

		/// The top-left corner of the window is to be resized.
		const TOP_LEFT = Self::TOP.bits() | Self::LEFT.bits();
		/// The bottom-left corner of the window is to be resized.
		const BOTTOM_LEFT = Self::BOTTOM.bits() | Self::LEFT.bits();
		/// The top-right corner of the window is to be resized.
		const TOP_RIGHT = Self::TOP.bits() | Self::RIGHT.bits();
		/// The bottom-right corner of the window is to be resized.
		const BOTTOM_RIGHT = Self::BOTTOM.bits() | Self::RIGHT.bits();
	}
}

impl ResizeEdge {
	/// Whether the window's [`LEFT`] edge is being resized.
	///
	/// [`LEFT`]: Self::LEFT
	pub fn left(&self) -> bool {
		self.intersects(Self::LEFT)
	}

	/// Whether the window's [`TOP`] edge is being resized.
	///
	/// [`TOP`]: Self::TOP
	pub fn top(&self) -> bool {
		self.intersects(Self::TOP)
	}

	/// Whether the window's [`RIGHT`] edge is being resized.
	///
	/// [`RIGHT`]: Self::RIGHT
	pub fn right(&self) -> bool {
		self.intersects(Self::RIGHT)
	}

	/// Whether the window's [`BOTTOM`] edge is being resized.
	///
	/// [`BOTTOM`]: Self::BOTTOM
	pub fn bottom(&self) -> bool {
		self.intersects(Self::BOTTOM)
	}
}

impl From<xdg_toplevel::ResizeEdge> for ResizeEdge {
	#[inline]
	fn from(resize_edge: xdg_toplevel::ResizeEdge) -> Self {
		Self::from_bits(resize_edge as u32).unwrap()
	}
}

pub struct ResizeSurfaceGrab {
	start_data: pointer::GrabStartData<state::AquariWm>,
	window: Window,

	edges: ResizeEdge,

	initial_rectangle: Rectangle,
	target_size: Size,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Default)]
enum ResizeSurfaceState {
	#[default]
	Idle,

	/// Currently in the process of resizing.
	Resizing {
		edges: ResizeEdge,
		initial_rectangle: Rectangle,
	},
	/// Resizing is complete, currently waiting for the final commit before we can do the final
	/// move.
	WaitingForFinalCommit {
		edges: ResizeEdge,
		initial_rectangle: Rectangle,
	},
}

impl ResizeSurfaceState {
	/// Calls the given `callback` with the resize state for the given `surface`.
	fn with<T>(surface: &WlSurface, callback: impl FnOnce(&mut Self) -> T) -> T {
		compositor::with_states(surface, |states| {
			states.data_map.insert_if_missing(RefCell::<Self>::default);
			let state = states.data_map.get::<RefCell<Self>>().unwrap();

			callback(&mut state.borrow_mut())
		})
	}

	fn commit(&mut self) -> Option<(ResizeEdge, Rectangle)> {
		match *self {
			Self::Idle => None,

			Self::Resizing {
				edges,
				initial_rectangle,
			} => Some((edges, initial_rectangle)),
			// Commit changes.
			Self::WaitingForFinalCommit {
				edges,
				initial_rectangle,
			} => {
				// The resize is complete.
				*self = Self::Idle;

				Some((edges, initial_rectangle))
			},
		}
	}
}

impl ResizeSurfaceGrab {
	pub fn start(
		start_data: pointer::GrabStartData<state::AquariWm>,
		window: Window,
		edges: ResizeEdge,
		initial_rectangle: Rectangle,
	) -> Self {
		ResizeSurfaceState::with(window.toplevel().wl_surface(), |state| {
			*state = ResizeSurfaceState::Resizing {
				edges,
				initial_rectangle,
			};
		});

		Self {
			start_data,
			window,
			edges,
			initial_rectangle,
			target_size: initial_rectangle.size,
		}
	}
}

impl PointerGrab<state::AquariWm> for ResizeSurfaceGrab {
	fn motion(
		&mut self,
		state: &mut state::AquariWm,
		handle: &mut PointerInnerHandle<'_>,
		_focus: Option<Focus>,
		event: &pointer::MotionEvent,
	) {
		let _span = span!(Level::DEBUG, "Resizing window").entered();

		// While the grab is active, no client has pointer focus.
		handle.motion(state, None, event);

		// Determine the change in the window's dimensions.
		let (delta_width, delta_height) = {
			// Difference between the current pointer position and the original pointer position.
			let delta = event.location - self.start_data.location;

			let delta_width = if self.edges.left() {
				// If the left edge is being resized, then moving right actually shrinks the window.
				-delta.x
			} else if self.edges.right() {
				delta.x
			} else {
				// If neither the left nor right edges are resized, then the change in width is 0.
				0.0
			};
			let delta_height = if self.edges.top() {
				// If the top edge is being resized, then moving down actually shrinks the window.
				-delta.y
			} else if self.edges.bottom() {
				delta.y
			} else {
				// If neither the top nor bottom edges are resized, then the change in height is 0.
				0.0
			};

			(delta_width as i32, delta_height as i32)
		};

		// Determine the new dimensions of the window.
		let (new_width, new_height) = {
			let (initial_width, initial_height) = (self.initial_rectangle.size.w, self.initial_rectangle.size.h);

			(initial_width + delta_width, initial_height + delta_height)
		};

		let top_level_surface = self.window.toplevel();
		let wl_surface = top_level_surface.wl_surface();

		// Determine the minimum and maximum sizes of the window.
		let ((min_width, min_height), (max_width, max_height)) = compositor::with_states(wl_surface, |states| {
			let data = states.cached_state.current::<SurfaceCachedState>();

			let max = |dimension| if dimension == 0 { i32::MAX } else { dimension };

			let min_size = (data.min_size.w.max(1), data.min_size.h.max(1));
			let max_size = (max(data.max_size.w), max(data.max_size.h));

			(min_size, max_size)
		});

		// Set the target size for resizing.
		self.target_size = Size::from((
			new_width.max(min_width).min(max_width),
			new_height.max(min_height).min(max_height),
		));

		// 'Register' the resize with the XDG top level surface.
		top_level_surface.with_pending_state(|state| {
			state.states.set(xdg_toplevel::State::Resizing);
			state.size = Some(self.target_size);
		});
		event!(Level::TRACE, delta_width, delta_height, "Updated window size");

		top_level_surface.send_pending_configure();
	}

	fn relative_motion(
		&mut self,
		state: &mut state::AquariWm,
		handle: &mut PointerInnerHandle<'_>,
		focus: Option<Focus>,
		event: &pointer::RelativeMotionEvent,
	) {
		handle.relative_motion(state, focus, event);
	}

	fn button(
		&mut self,
		state: &mut state::AquariWm,
		handle: &mut PointerInnerHandle<'_>,
		event: &pointer::ButtonEvent,
	) {
		handle.button(state, event);

		if !handle.current_pressed().contains(&PRIMARY_BUTTON) {
			// No more buttons are pressed: release the grab.
			handle.unset_grab(state, event.serial, event.time, true);

			let top_level_surface = self.window.toplevel();
			let wl_surface = top_level_surface.wl_surface();

			// 'Register' the end of the resize with the XDG top level surface.
			top_level_surface.with_pending_state(|state| {
				state.states.unset(xdg_toplevel::State::Resizing);
				state.size = Some(self.target_size);
			});

			top_level_surface.send_pending_configure();

			// Update the resize surface state.
			ResizeSurfaceState::with(wl_surface, |state| {
				*state = ResizeSurfaceState::WaitingForFinalCommit {
					edges: self.edges,
					initial_rectangle: self.initial_rectangle,
				}
			});
		}
	}

	fn axis(&mut self, state: &mut state::AquariWm, handle: &mut PointerInnerHandle<'_>, details: pointer::AxisFrame) {
		handle.axis(state, details);
	}

	fn frame(&mut self, state: &mut state::AquariWm, handle: &mut PointerInnerHandle<'_>) {
		handle.frame(state);
	}

	fn gesture_swipe_begin(
		&mut self,
		state: &mut state::AquariWm,
		handle: &mut PointerInnerHandle<'_>,
		event: &GestureSwipeBeginEvent,
	) {
		handle.gesture_swipe_begin(state, event);
	}

	fn gesture_swipe_update(
		&mut self,
		state: &mut state::AquariWm,
		handle: &mut PointerInnerHandle<'_>,
		event: &GestureSwipeUpdateEvent,
	) {
		handle.gesture_swipe_update(state, event);
	}

	fn gesture_swipe_end(
		&mut self,
		state: &mut state::AquariWm,
		handle: &mut PointerInnerHandle<'_>,
		event: &GestureSwipeEndEvent,
	) {
		handle.gesture_swipe_end(state, event);
	}

	fn gesture_pinch_begin(
		&mut self,
		state: &mut state::AquariWm,
		handle: &mut PointerInnerHandle<'_>,
		event: &GesturePinchBeginEvent,
	) {
		handle.gesture_pinch_begin(state, event);
	}

	fn gesture_pinch_update(
		&mut self,
		state: &mut state::AquariWm,
		handle: &mut PointerInnerHandle<'_>,
		event: &GesturePinchUpdateEvent,
	) {
		handle.gesture_pinch_update(state, event);
	}

	fn gesture_pinch_end(
		&mut self,
		state: &mut state::AquariWm,
		handle: &mut PointerInnerHandle<'_>,
		event: &GesturePinchEndEvent,
	) {
		handle.gesture_pinch_end(state, event);
	}

	fn gesture_hold_begin(
		&mut self,
		state: &mut state::AquariWm,
		handle: &mut PointerInnerHandle<'_>,
		event: &GestureHoldBeginEvent,
	) {
		handle.gesture_hold_begin(state, event);
	}

	fn gesture_hold_end(
		&mut self,
		state: &mut state::AquariWm,
		handle: &mut PointerInnerHandle<'_>,
		event: &GestureHoldEndEvent,
	) {
		handle.gesture_hold_end(state, event);
	}

	fn start_data(&self) -> &pointer::GrabStartData<state::AquariWm> {
		&self.start_data
	}
}

pub fn handle_commit(space: &mut Space<Window>, surface: &WlSurface) -> Option<()> {
	let window = space
		.elements()
		.find(|window| window.toplevel().wl_surface() == surface)
		.cloned()?;

	let mut window_location = space.element_location(&window)?;
	let geometry = window.geometry();

	let (new_x, new_y) = ResizeSurfaceState::with(surface, |state| {
		state
			.commit()
			.map(|(edges, initial_rectangle)| {
				let (initial_x, initial_y) = (initial_rectangle.loc.x, initial_rectangle.loc.y);
				let (initial_width, initial_height) = (initial_rectangle.size.w, initial_rectangle.size.h);

				let (geometry_width, geometry_height) = (geometry.size.w, geometry.size.h);

				// If the window is being resized by the left edge, its x must be adjusted
				// accordingly.
				let new_x = edges
					.intersects(ResizeEdge::LEFT)
					.then_some(initial_x + (initial_width - geometry_width));

				// If the window is being resized by the top edge, its y must be adjusted
				// accordingly.
				let new_y = edges
					.intersects(ResizeEdge::TOP)
					.then_some(initial_y + (initial_height - geometry_height));

				(new_x, new_y)
			})
			.unwrap_or((None, None))
	});

	if let Some(new_x) = new_x {
		window_location.x = new_x;
	}
	if let Some(new_y) = new_y {
		window_location.y = new_y;
	}

	// If either of the top or left edges of the window were resized, then its location must be
	// updated accordingly.
	if new_x.is_some() || new_y.is_some() {
		space.map_element(window, window_location, false);
	}

	Some(())
}
