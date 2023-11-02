// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::{ffi::OsString, sync::Arc, time};

use smithay::{
	backend::renderer::utils::on_commit_buffer_handler,
	delegate_compositor,
	delegate_data_device,
	delegate_output,
	delegate_seat,
	delegate_shm,
	delegate_xdg_shell,
	desktop::{PopupKind, PopupManager, Space, Window, WindowSurfaceType},
	input::{keyboard::XkbConfig, pointer, pointer::CursorImageStatus, Seat, SeatHandler, SeatState},
	reexports::{
		calloop,
		calloop::{generic::Generic, EventLoop, Interest, LoopSignal},
		wayland_protocols::xdg::shell::server::xdg_toplevel,
		wayland_server::{
			backend::{ClientData, ClientId, DisconnectReason},
			protocol::{wl_buffer::WlBuffer, wl_seat::WlSeat, wl_surface::WlSurface},
			Client,
			Display,
			DisplayHandle,
			Resource,
		},
	},
	utils::{Logical as LogicalSpace, Rectangle, Serial},
	wayland::{
		buffer::BufferHandler,
		compositor::{get_parent, is_sync_subsurface, CompositorClientState, CompositorHandler, CompositorState},
		output::OutputManagerState,
		selection::{
			data_device::{
				set_data_device_focus,
				ClientDndGrabHandler,
				DataDeviceHandler,
				DataDeviceState,
				ServerDndGrabHandler,
			},
			SelectionHandler,
		},
		shell::xdg::{PopupSurface, PositionerState, ToplevelSurface, XdgShellHandler, XdgShellState},
		shm::{ShmHandler, ShmState},
		socket::ListeningSocketSource,
	},
};

use super::grabs::{move_grab::MoveSurfaceGrab, resize_grab::ResizeSurfaceGrab};

type Point<N = i32, Space = LogicalSpace> = smithay::utils::Point<N, Space>;

pub struct AquariWm {
	pub start_time: time::Instant,
	pub socket_name: OsString,
	pub display_handle: DisplayHandle,

	pub space: Space<Window>,
	pub loop_signal: LoopSignal,

	// Smithay state.
	pub compositor_state: CompositorState,
	pub xdg_shell_state: XdgShellState,
	pub shm_state: ShmState,
	pub output_manager_state: OutputManagerState,
	pub seat_state: SeatState<Self>,
	pub data_device_state: DataDeviceState,

	pub popup_manager: PopupManager,

	pub seat: Seat<Self>,
}

// Seat impls {{{
impl SeatHandler for AquariWm {
	type KeyboardFocus = WlSurface;
	type PointerFocus = WlSurface;

	fn seat_state(&mut self) -> &mut SeatState<Self> {
		&mut self.seat_state
	}

	fn focus_changed(&mut self, seat: &Seat<Self>, focused: Option<&Self::KeyboardFocus>) {
		let display_handle = &self.display_handle;

		let client = focused.and_then(|surface| display_handle.get_client(surface.id()).ok());
		set_data_device_focus(display_handle, seat, client);
	}

	fn cursor_image(&mut self, _seat: &Seat<Self>, _image: CursorImageStatus) {}
}

delegate_seat!(AquariWm);
// }}}

// Data device impls {{{
impl SelectionHandler for AquariWm {
	type SelectionUserData = ();
}

impl DataDeviceHandler for AquariWm {
	fn data_device_state(&self) -> &DataDeviceState {
		&self.data_device_state
	}
}

impl ClientDndGrabHandler for AquariWm {}
impl ServerDndGrabHandler for AquariWm {}

delegate_data_device!(AquariWm);
// }}}

// Output impl
delegate_output!(AquariWm);

// XDG shell impls {{{
impl XdgShellHandler for AquariWm {
	fn xdg_shell_state(&mut self) -> &mut XdgShellState {
		&mut self.xdg_shell_state
	}

	fn new_toplevel(&mut self, surface: ToplevelSurface) {
		let window = Window::new(surface);
		self.space.map_element(window, (0, 0), false);
	}

	fn new_popup(&mut self, surface: PopupSurface, _positioner: PositionerState) {
		let _ = self.popup_manager.track_popup(PopupKind::Xdg(surface));
	}

	fn move_request(&mut self, surface: ToplevelSurface, seat: WlSeat, serial: Serial) {
		let seat = <Seat<Self>>::from_resource(&seat).unwrap();

		let wl_surface = surface.wl_surface();

		if let Some(start_data) = check_grab(&seat, wl_surface, serial) {
			let pointer = seat.get_pointer().unwrap();

			let window = self
				.space
				.elements()
				.find(|window| window.toplevel().wl_surface() == wl_surface)
				.unwrap()
				.clone();
			let initial_window_location = self.space.element_location(&window).unwrap();

			let grab = MoveSurfaceGrab {
				start_data,
				window,
				initial_window_location,
			};

			// False negative in RustRover/IntelliJ Rust plugin here.
			pointer.set_grab(self, grab, serial, pointer::Focus::Clear);
		}
	}

	fn resize_request(
		&mut self,
		surface: ToplevelSurface,
		seat: WlSeat,
		serial: Serial,
		edges: xdg_toplevel::ResizeEdge,
	) {
		let seat = Seat::from_resource(&seat).unwrap();

		let wl_surface = surface.wl_surface();

		if let Some(start_data) = check_grab(&seat, wl_surface, serial) {
			let pointer = seat.get_pointer().unwrap();

			let window = self
				.space
				.elements()
				.find(|window| window.toplevel().wl_surface() == wl_surface)
				.unwrap()
				.clone();
			let initial_window_location = self.space.element_location(&window).unwrap();
			let initial_window_size = window.geometry().size;

			surface.with_pending_state(|state| state.states.set(xdg_toplevel::State::Resizing));
			surface.send_pending_configure();

			let grab = ResizeSurfaceGrab::start(
				start_data,
				window,
				edges.into(),
				Rectangle::from_loc_and_size(initial_window_location, initial_window_size),
			);

			// False negative in RustRover/IntelliJ Rust plugin here.
			pointer.set_grab(self, grab, serial, pointer::Focus::Clear);
		}
	}

	fn grab(&mut self, _surface: PopupSurface, _seat: WlSeat, _serial: Serial) {
		// TODO: popup grabs
	}

	fn reposition_request(&mut self, surface: PopupSurface, positioner: PositionerState, token: u32) {
		surface.with_pending_state(|state| {
			// We should be calculating the geometry here, not using the default implementation, as it does not
			// take the window position and output constraints into account.
			let geometry = positioner.get_geometry();
			state.geometry = geometry;
			state.positioner = positioner;
		});

		surface.send_repositioned(token);
	}
}

delegate_xdg_shell!(AquariWm);

fn check_grab(seat: &Seat<AquariWm>, surface: &WlSurface, serial: Serial) -> Option<pointer::GrabStartData<AquariWm>> {
	let pointer = seat.get_pointer()?;

	// If this surface has a pointer grab...
	if pointer.has_grab(serial) {
		let start_data = pointer.grab_start_data()?;
		let (focus_surface, _) = start_data.focus.as_ref()?;

		// If the focus was for the same surface, return the grab start data.
		if focus_surface.id().same_client_as(&surface.id()) {
			return Some(start_data);
		}
	}

	None
}
// }}}

// Compositor impls {{{
impl CompositorHandler for AquariWm {
	fn compositor_state(&mut self) -> &mut CompositorState {
		&mut self.compositor_state
	}

	fn client_compositor_state<'client>(&self, client: &'client Client) -> &'client CompositorClientState {
		&client.get_data::<ClientState>().unwrap().compositor_state
	}

	fn commit(&mut self, surface: &WlSurface) {
		on_commit_buffer_handler::<Self>(surface);

		if !is_sync_subsurface(surface) {
			// Find the root node.
			let mut root = surface.clone();
			while let Some(parent) = get_parent(&root) {
				root = parent;
			}

			if let Some(window) = self
				.space
				.elements()
				.find(|window| window.toplevel().wl_surface() == &root)
			{
				window.on_commit();
			}
		}
	}
}

impl BufferHandler for AquariWm {
	fn buffer_destroyed(&mut self, _buffer: &WlBuffer) {}
}

impl ShmHandler for AquariWm {
	fn shm_state(&self) -> &ShmState {
		&self.shm_state
	}
}

delegate_compositor!(AquariWm);
delegate_shm!(AquariWm);
// }}}

impl AquariWm {
	pub fn new(display: Display<Self>, event_loop: &mut EventLoop<Self>) -> Self {
		let start_time = time::Instant::now();
		let display_handle = display.handle();

		let mut seat_state = SeatState::new();

		// A seat is a group of input devices. It typically has a pointer (mouse) and maintains a keyboard
		// focus and a pointer focus.
		let mut seat = seat_state.new_wl_seat(&display_handle, "winit");

		// Add a keyboard.
		// FIXME: This is taken from the smallvil Smithay example - it assumes a keyboard is always
		//        connected. We should actually track connected keyboards.
		seat.add_keyboard(XkbConfig::default(), 200, 25).unwrap();
		// Add a mouse.
		// FIXME: This is taken from the smallvil Smithay example - it assumes a mouse is always
		//        connected. We should actually track connected pointer devices.
		seat.add_pointer();

		Self {
			start_time,
			socket_name: Self::init_wayland_listener(display, event_loop),

			// A two-dimensional plane on which outputs and windows can be mapped.
			space: Space::default(),
			loop_signal: event_loop.get_signal(),

			// A whole bunch of Smithay-related state.
			compositor_state: CompositorState::new::<Self>(&display_handle),
			xdg_shell_state: XdgShellState::new::<Self>(&display_handle),
			shm_state: ShmState::new::<Self>(&display_handle, Vec::new()),
			output_manager_state: OutputManagerState::new_with_xdg_output::<Self>(&display_handle),
			seat_state,
			data_device_state: DataDeviceState::new::<Self>(&display_handle),

			popup_manager: PopupManager::default(),

			display_handle,
			seat,
		}
	}

	fn init_wayland_listener(display: Display<Self>, event_loop: &mut EventLoop<Self>) -> OsString {
		let listening_socket = ListeningSocketSource::new_auto().unwrap();

		let socket_name = listening_socket.socket_name().to_os_string();

		let handle = event_loop.handle();

		event_loop
			.handle()
			.insert_source(listening_socket, move |client_stream, _, state| {
				state
					.display_handle
					.insert_client(client_stream, Arc::new(ClientState::default()))
					.unwrap();
			})
			.expect("Failed to initialise the wayland event source.");

		handle
			.insert_source(
				Generic::new(display, Interest::READ, calloop::Mode::Level),
				|_, display, state| {
					// Safety: we don't drop the display.
					unsafe { display.get_mut().dispatch_clients(state).unwrap() };

					Ok(calloop::PostAction::Continue)
				},
			)
			.unwrap();

		socket_name
	}

	pub fn surface_under(&self, pos: Point<f64>) -> Option<(WlSurface, Point<i32>)> {
		self.space.element_under(pos).and_then(|(window, location)| {
			window
				.surface_under(pos - location.to_f64(), WindowSurfaceType::ALL)
				.map(|(surface, pos)| (surface, pos + location))
		})
	}
}

#[derive(Default)]
pub struct ClientState {
	pub compositor_state: CompositorClientState,
}

impl ClientData for ClientState {
	fn initialized(&self, _client_id: ClientId) {}
	fn disconnected(&self, _client_id: ClientId, _reason: DisconnectReason) {}
}
