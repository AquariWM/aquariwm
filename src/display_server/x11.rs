// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::{env, future::Future, io, thread};

use tracing::{event, span, Level};
use xcb::x::{self as x11, Circulate, Cw as Attribute, EventMask, Place};

use crate::{
	display_server::{AsyncDisplayServer, DisplayServer},
	state,
};

mod util;

#[derive(Debug, thiserror::Error)]
pub enum Error {
	#[error(transparent)]
	Xcb(#[from] xcb::Error),

	#[error(transparent)]
	Io(#[from] io::Error),
}

impl From<xcb::ProtocolError> for Error {
	fn from(error: xcb::ProtocolError) -> Self {
		Self::Xcb(error.into())
	}
}

impl From<xcb::ConnError> for Error {
	fn from(error: xcb::ConnError) -> Self {
		Self::Xcb(error.into())
	}
}

pub type Result<T, Err = Error> = std::result::Result<T, Err>;

#[cfg(feature = "testing")]
mod testing {
	use std::{process, sync::mpsc, time::Duration};

	use winit::{
		event::{Event as WinitEvent, WindowEvent as WinitWindowEvent},
		event_loop::EventLoopBuilder as WinitEventLoopBuilder,
		platform::x11::EventLoopBuilderExtX11,
		window::WindowBuilder as WinitWindowBuilder,
	};

	use super::*;

	pub struct Xephyr(pub process::Child);

	impl Drop for Xephyr {
		fn drop(&mut self) {
			let Self(child) = self;

			child.kill().expect("Failed to kill Xephyr");
		}
	}

	impl Xephyr {
		pub fn spawn() -> io::Result<Self> {
			const TESTING_DISPLAY: &str = ":1";

			let (transmitter, receiver) = mpsc::channel();

			// Create and run a `winit` window for `Xephyr` to use in another thread so it doesn't block the
			// main thread.
			thread::spawn(move || {
				event!(Level::DEBUG, "Initialising winit window");

				let event_loop = WinitEventLoopBuilder::new().with_any_thread(true).build().unwrap();
				let window = WinitWindowBuilder::new()
					.with_title(X11::title())
					.build(&event_loop)
					.unwrap();

				// Send the window's window ID back to the main thread so it can be supplied to `Xephyr`.
				transmitter.send(u64::from(window.id())).unwrap();

				event_loop
					.run(move |event, target| match event {
						// Close the window if requested.
						WinitEvent::WindowEvent {
							event: WinitWindowEvent::CloseRequested,
							..
						} => target.exit(),

						_ => (),
					})
					.unwrap();
			});
			let window_id = receiver.recv().unwrap();

			event!(Level::DEBUG, "Initialising Xephyr");
			match process::Command::new("Xephyr")
				.arg("-resizeable")
				// Run `Xephyr` in the `winit` window.
				.args(["-parent", &window_id.to_string()])
				.arg(TESTING_DISPLAY)
				.spawn()
			{
				Ok(process) => {
					// Set the `DISPLAY` env variable to `TESTING_DISPLAY`.
					env::set_var("DISPLAY", TESTING_DISPLAY);

					// Sleep for 1s to wait for Xephyr to launch. Not ideal.
					thread::sleep(Duration::from_secs(1));

					Ok(Self(process))
				},

				Err(error) => {
					event!(Level::ERROR, "Error while attempting to initialise Xephyr: {error}");

					Err(error)
				},
			}
		}
	}
}

#[cfg(not(feature = "testing"))]
mod testing {
	pub struct Xephyr;

	impl Xephyr {
		#[inline(always)]
		pub fn spawn() -> io::Result<()> {
			Ok(())
		}
	}
}

pub struct X11 {
	pub connection: util::WindowManager,
	pub state: state::AquariWm<x11::Window>,
}

impl From<x11::MapState> for state::MapState {
	fn from(state: x11::MapState) -> Self {
		match state {
			x11::MapState::Viewable | x11::MapState::Unviewable => state::MapState::Mapped,

			x11::MapState::Unmapped => state::MapState::Unmapped,
		}
	}
}

impl AsyncDisplayServer for X11 {
	type Error = Error;
}

impl DisplayServer for X11 {
	type Output = impl Future<Output = Result<(), Error>>;
	const NAME: &'static str = "X11";

	fn run(testing: bool) -> Self::Output {
		async move {
			// Spawn Xephyr - a nested X server - if `testing` is enabled so AquariWM runs in a testing
			// window. Keep it in scope so it can be killed when it is dropped.
			let _process = testing.then_some(testing::Xephyr::spawn()?);

			// Connect to the X server and initialise AquariWM.
			let mut connection = util::WindowManager::new(None).await?;

			if testing {
				event!(Level::INFO, "Testing mode enabled");

				// Attempt to launch a terminal.
				match crate::launch_terminal() {
					Ok((name, _)) => event!(Level::INFO, "Launched {name:?}"),
					Err(error) => event!(Level::WARN, "Failed to launch terminal: {error}"),
				}
			}

			Ok(connection
				.run(|connection, state, event| {
					async {
						match event {
							// X11 core protocol events.
							xcb::Event::X(event) => match event {
								// Track the state of newly created windows.
								x11::Event::CreateNotify(notification) => {
									state.add_window(notification.window(), state::MapState::Unmapped);
								},
								// Stop tracking the state of destroyed windows.
								x11::Event::DestroyNotify(notification) => {
									state.remove_window(&notification.window());
								},

								// If a client requests to map its window, map it.
								x11::Event::MapRequest(request) => {
									let window = request.window();

									// TODO: place tiled windows in tiling layout

									connection.send_request(&x11::MapWindow { window });
									state.map_window(&window);
								},

								// If a client requests to configure its window, honor it. For a tiling layout, this
								// should modify the configure request to place it in the tiling layout.
								x11::Event::ConfigureRequest(request) => {
									connection.send_request(&x11::ConfigureWindow {
										window: request.window(),
										value_list: &util::value_list(&request),
									});
								},

								// If a client requests to raise or lower its window, honor it. For a tiling layout,
								// this should be rejected for tiled windows, as they should always be at the bottom
								// of the stack.
								x11::Event::CirculateRequest(request) => {
									// x.circulate_window(
									// 	request.window(),
									// 	match request.place() {
									// 		Place::OnTop => Circulate::RaiseLowest,
									// 		Place::OnBottom => Circulate::LowerHighest,
									// 	},
									// );
								},

								x11::Event::UnmapNotify(notification) => {
									state.unmap_window(&notification.window());
								},

								_ => (),
							},

							_ => (),
						}

						Ok(())
					}
				})
				.await?)
		}
	}
}
