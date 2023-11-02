// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::{env, io, process, process::Command, thread, time::Duration};

use tracing::{event, span, Level};
use xcb::x::{self as x11, Circulate, Cw as Attribute, EventMask, Place};

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

			match Command::new("Xephyr").arg("-resizeable").arg(TESTING_DISPLAY).spawn() {
				Ok(process) => {
					event!(Level::DEBUG, "Initialised Xephyr");

					// Set the `DISPLAY` env variable to `TESTING_DISPLAY`.
					env::set_var("DISPLAY", TESTING_DISPLAY);

					// Sleep for 3s to wait for Xephyr to launch. Not ideal.
					thread::sleep(Duration::from_secs(3));

					Ok(Self(process))
				},

				Err(error) => {
					event!(Level::ERROR, "Failed to initialise Xephyr: {error}");

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

pub fn run(testing: bool) -> Result<()> {
	let init_span = span!(Level::INFO, "Initialising AquariWM (X11)").entered();

	if testing {
		event!(Level::INFO, "Testing mode enabled");
	} else {
		event!(Level::TRACE, "Testing mode disabled");
	}

	// Spawn Xephyr - a nested X server - if `testing` is enabled so AquariWM runs in a testing
	// window. Keep it in scope so it can be killed when it is dropped.
	let _process = testing.then_some(testing::Xephyr::spawn()?);

	// Connect to the X server.
	let (connection, screen_num) = xcb::Connection::connect(None)?;

	// Get the setup and screen.
	let setup = connection.get_setup();
	let screen = setup.roots().nth(screen_num as usize).unwrap();

	// The root window of the screen.
	let root = screen.root();

	// Register for SUBSTRUCTURE_NOTIFY and SUBSTRUCTURE_REDIRECT events on the root window; this
	// means registering as a window manager.
	let cookie = connection.send_request_checked(&x11::ChangeWindowAttributes {
		window: root,
		value_list: &[Attribute::EventMask(
			EventMask::SUBSTRUCTURE_NOTIFY | EventMask::SUBSTRUCTURE_REDIRECT,
		)],
	});

	match connection.check_request(cookie) {
		Ok(_) => event!(Level::INFO, "Successfully registered window manager"),

		// If we failed to register the window manager, exit the program.
		Err(error) => {
			event!(
				Level::ERROR,
				"Failed to register window manager; a window manager is already running!",
			);

			return Err(error.into());
		},
	}

	// Attempt to launch a terminal.
	crate::launch_terminal();

	init_span.exit();

	let event_loop_span = span!(Level::DEBUG, "Event loop");

	// The window manager's event loop.
	loop {
		let _span = event_loop_span.enter();

		// Flush requests sent in the previous iteration.
		connection.flush()?;

		match connection.wait_for_event()? {
			// X11 core protocol events.
			xcb::Event::X(event) => match event {
				// If a client requests to map its window, map it. For a tiling layout, this should
				// place it in the tiling layout when mapping it.
				x11::Event::MapRequest(request) => {
					connection.send_request(&x11::MapWindow {
						window: request.window(),
					});
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
					util::circulate_window(
						&connection,
						request.window(),
						match request.place() {
							Place::OnTop => Circulate::RaiseLowest,
							Place::OnBottom => Circulate::LowerHighest,
						},
					);
				},

				// TODO: for tiling layouts, remove the window from the layout.
				x11::Event::UnmapNotify(_notification) => {},

				_ => (),
			},

			_ => (),
		}
	}
}
