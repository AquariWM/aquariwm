// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use tracing::{info, trace};

use xcb::x::{self, Window};
use xcb::{Connection, Xid};

use crate::extensions::*;

/// The central object of the entire AquariWM window manager. Contains state and the event loop.
pub struct AquariWm {
	conn: Connection,
	_root: Window,
}

impl AquariWm {
	pub fn new(conn: Connection, root: Window) -> Self {
		Self { conn, _root: root }
	}

	/// Starts AquariWM's event loop to listen and respond to new events.
    ///
    /// The event loop blocks until event(s) are received. Upon receiving an event, the type of
    /// event is matched against and the window manager will react accordingly. Usually this will
    /// involve sending one or more new requests to the X server in response.
	pub fn run(&self) -> xcb::Result<()> {
		info!("Running the window manager");
		let conn = &self.conn;

		loop {
			match self.conn.wait_for_event()? {
				// Accept client requests to configure their windows in full.
				xcb::Event::X(x::Event::ConfigureRequest(req)) => {
					trace!("Configuring window ({})", req.window().resource_id());
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
					trace!("Mapping window ({})", window.resource_id());

					conn.send_request(&x::MapWindow { window });
					// TODO: Is it guaranteed that the `MapWindow` request will be processed
					//       before the window initialization requests are? Might need to wait
					//       until the window is mapped first...
					crate::init_window(conn, &window)?;

					conn.flush()?;
				}
				// TODO: Unimplemented. For window manipulation.
				xcb::Event::X(x::Event::ButtonPress(notif)) => {
					trace!("Processing button press ({})", notif.detail());
				}
				// TODO: Unimplemented. For window manipulation.
				xcb::Event::X(x::Event::ButtonRelease(notif)) => {
					trace!("Processing button release ({})", notif.detail());
				}
				// TODO: Unimplemented. For window manipulation.
				xcb::Event::X(x::Event::MotionNotify(_)) => {
					trace!("Processing cursor drag");
				}
				// Focus whatever window the cursor enters.
				xcb::Event::X(x::Event::EnterNotify(notif)) => {
					trace!(
						"Focusing window entered by cursor ({})",
						notif.event().resource_id()
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
						"Bringing the newly focused window to the top of the stack ({})",
						notif.event().resource_id()
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
