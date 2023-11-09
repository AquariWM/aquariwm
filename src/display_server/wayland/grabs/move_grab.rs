// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use smithay::{
	desktop::Window,
	input::pointer::{
		self,
		AxisFrame,
		ButtonEvent,
		GestureHoldBeginEvent,
		GestureHoldEndEvent,
		GesturePinchBeginEvent,
		GesturePinchEndEvent,
		GesturePinchUpdateEvent,
		GestureSwipeBeginEvent,
		GestureSwipeEndEvent,
		GestureSwipeUpdateEvent,
		MotionEvent,
		PointerGrab,
		PointerInnerHandle,
		RelativeMotionEvent,
	},
	reexports::wayland_server::protocol::wl_surface::WlSurface,
	utils::Logical as LogicalSpace,
};
use tracing::{event, span, Level};

use super::{super::state, PRIMARY_BUTTON};

type Point<N = i32, Space = LogicalSpace> = smithay::utils::Point<N, Space>;

pub struct MoveSurfaceGrab {
	pub start_data: pointer::GrabStartData<state::WaylandState>,
	pub window: Window,
	pub initial_window_location: Point,
}

impl PointerGrab<state::WaylandState> for MoveSurfaceGrab {
	fn motion(
		&mut self,
		state: &mut state::WaylandState,
		handle: &mut PointerInnerHandle<'_, state::WaylandState>,
		_focus: Option<(WlSurface, Point)>,
		event: &MotionEvent,
	) {
		let _span = span!(Level::DEBUG, "Moving window");

		// While the grab is active, no client has pointer focus.
		handle.motion(state, None, event);

		let delta = event.location - self.start_data.location;
		let new_location = self.initial_window_location.to_f64() + delta;
		state
			.space
			.map_element(self.window.clone(), new_location.to_i32_round(), true);

		let (delta_x, delta_y) = (delta.x, delta.y);
		event!(Level::TRACE, delta_x, delta_y, "Updated window position");
	}

	fn relative_motion(
		&mut self,
		state: &mut state::WaylandState,
		handle: &mut PointerInnerHandle<'_, state::WaylandState>,
		focus: Option<(WlSurface, Point)>,
		event: &RelativeMotionEvent,
	) {
		handle.relative_motion(state, focus, event);
	}

	fn button(
		&mut self,
		state: &mut state::WaylandState,
		handle: &mut PointerInnerHandle<'_, state::WaylandState>,
		event: &ButtonEvent,
	) {
		handle.button(state, event);

		if !handle.current_pressed().contains(&PRIMARY_BUTTON) {
			// No more buttons are pressed: release the grab.
			handle.unset_grab(state, event.serial, event.time, true);
		}
	}

	fn axis(
		&mut self,
		state: &mut state::WaylandState,
		handle: &mut PointerInnerHandle<'_, state::WaylandState>,
		details: AxisFrame,
	) {
		handle.axis(state, details);
	}

	fn frame(&mut self, state: &mut state::WaylandState, handle: &mut PointerInnerHandle<'_, state::WaylandState>) {
		handle.frame(state);
	}

	fn gesture_swipe_begin(
		&mut self,
		state: &mut state::WaylandState,
		handle: &mut PointerInnerHandle<'_, state::WaylandState>,
		event: &GestureSwipeBeginEvent,
	) {
		handle.gesture_swipe_begin(state, event);
	}

	fn gesture_swipe_update(
		&mut self,
		state: &mut state::WaylandState,
		handle: &mut PointerInnerHandle<'_, state::WaylandState>,
		event: &GestureSwipeUpdateEvent,
	) {
		handle.gesture_swipe_update(state, event);
	}

	fn gesture_swipe_end(
		&mut self,
		state: &mut state::WaylandState,
		handle: &mut PointerInnerHandle<'_, state::WaylandState>,
		event: &GestureSwipeEndEvent,
	) {
		handle.gesture_swipe_end(state, event);
	}

	fn gesture_pinch_begin(
		&mut self,
		state: &mut state::WaylandState,
		handle: &mut PointerInnerHandle<'_, state::WaylandState>,
		event: &GesturePinchBeginEvent,
	) {
		handle.gesture_pinch_begin(state, event);
	}

	fn gesture_pinch_update(
		&mut self,
		state: &mut state::WaylandState,
		handle: &mut PointerInnerHandle<'_, state::WaylandState>,
		event: &GesturePinchUpdateEvent,
	) {
		handle.gesture_pinch_update(state, event);
	}

	fn gesture_pinch_end(
		&mut self,
		state: &mut state::WaylandState,
		handle: &mut PointerInnerHandle<'_, state::WaylandState>,
		event: &GesturePinchEndEvent,
	) {
		handle.gesture_pinch_end(state, event);
	}

	fn gesture_hold_begin(
		&mut self,
		state: &mut state::WaylandState,
		handle: &mut PointerInnerHandle<'_, state::WaylandState>,
		event: &GestureHoldBeginEvent,
	) {
		handle.gesture_hold_begin(state, event);
	}

	fn gesture_hold_end(
		&mut self,
		state: &mut state::WaylandState,
		handle: &mut PointerInnerHandle<'_, state::WaylandState>,
		event: &GestureHoldEndEvent,
	) {
		handle.gesture_hold_end(state, event);
	}

	fn start_data(&self) -> &pointer::GrabStartData<state::WaylandState> {
		&self.start_data
	}
}
