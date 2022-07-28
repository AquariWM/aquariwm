// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use tracing::{debug, info, trace};

use xcb::x::{self, Window};
use xcb::{Connection, Xid};

use crate::extensions::ConfigureRequestEventExtensions;

/// The central object of the entire AquariWM window manager. Contains state and the event loop.
pub struct AquariWm {
	conn: Connection,
	_root: Window,
}

impl AquariWm {
	/// Starts the window manager by instantiating `Self` and running the event loop.
	pub fn start(conn: Connection, root: Window) -> xcb::Result<()> {
		let wm = Self { conn, _root: root };
		wm.run()
	}

	/// Starts AquariWM's event loop to listen and respond to new events.
	///
	/// The event loop blocks until event(s) are received. Upon receiving an event, the type of
	/// event is matched against and the window manager will react accordingly. Usually this will
	/// involve sending one or more new requests to the X server in response.
	fn run(&self) -> xcb::Result<()> {
		info!("Running the window manager");
		let conn = &self.conn;

		loop {
			match self.conn.wait_for_event()? {
				// Accept client requests to configure their windows in full.
				xcb::Event::X(x::Event::ConfigureRequest(req)) => {
					trace!(window = req.window().resource_id(), "Configuring window");
					conn.send_request(&x::ConfigureWindow {
						window: req.window(),
						value_list: &req.values(),
					});

					conn.flush()?;
				}
				// Allow clients to map their windows, but initialise those windows with AquariWM
				// afterwards.
				xcb::Event::X(x::Event::MapRequest(req)) => {
					let window = req.window();
					trace!(window = window.resource_id(), "Mapping window");

					conn.send_request(&x::MapWindow { window });
					// TODO: Check if this is necessary. Maybe events can be registered fine
					//       before a window is mapped, and that it is only that they are cleared
					//       when it is unmapped?
					conn.flush()?;

					debug!(
						window = window.resource_id(),
						"Setting up newly mapped window"
					);
					crate::init_window(conn, &window)?;
					conn.flush()?;
				}
				// TODO: Unimplemented. For window manipulation.
				xcb::Event::X(x::Event::ButtonPress(notif)) => {
					trace!(keycode = notif.detail(), "Processing button press");
				}
				// TODO: Unimplemented. For window manipulation.
				xcb::Event::X(x::Event::ButtonRelease(notif)) => {
					trace!(keycode = notif.detail(), "Processing button release");
				}
				// TODO: Unimplemented. For window manipulation.
				xcb::Event::X(x::Event::MotionNotify(_)) => {
					trace!("Processing cursor drag");
				}
				// Focus whatever window the cursor enters.
				xcb::Event::X(x::Event::EnterNotify(notif)) => {
					trace!(
						window = notif.event().resource_id(),
						"Focusing window entered by cursor"
					);
					conn.send_request(&x::SetInputFocus {
						revert_to: x::InputFocus::Parent,
						focus: notif.event(),
						time: x::CURRENT_TIME,
					});

					conn.flush()?;
				}
				// Bring windows to the top of the stack as they are focused.
				xcb::Event::X(x::Event::FocusIn(notif)) => {
					trace!(
						window = notif.event().resource_id(),
						"Bringing the newly focused window to the top of the stack"
					);
					conn.send_request(&x::ConfigureWindow {
						window: notif.event(),
						value_list: &[x::ConfigWindow::StackMode(x::StackMode::Above)],
					});

					conn.flush()?;
				}
				_ => {}
			}
		}
	}
}
