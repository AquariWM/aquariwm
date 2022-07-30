// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use tracing::{debug, info, trace};

use xcb::x::{self, Window};
use xcb::{Connection, Xid};

use crate::extensions::ConfigureRequestEventExtensions;
use crate::window_manipulation::WindowManipulation;

/// The central object of the entire AquariWM window manager. Contains state and the event loop.
#[allow(dead_code)]
pub struct AquariWm {
	conn: Connection,
	/// The root window of the screen.
	root: Window,
	/// Represents the ongoing manipulation of a window, if one is occurring.
	///
	/// [`None`] here means that there is no window being manipulated.
	manipulation: Option<WindowManipulation>,
}

#[allow(dead_code)]
impl AquariWm {
	/// Starts the window manager by instantiating `Self` and running the event loop.
	pub fn start(conn: Connection, root: Window) -> xcb::Result<()> {
		let wm = Self {
			conn,
			root,
			manipulation: None,
		};
		wm.run()
	}

	/// The event loop blocks until event(s) are received. Upon receiving an event, the type of
	/// event is matched against and the window manager will react accordingly. Usually this will
	/// involve sending one or more new requests to the X server in response.
	fn run(mut self) -> xcb::Result<()> {
		info!("Running the window manager");
		loop {
			match self.conn.wait_for_event()? {
				// Accept client requests to configure their windows in full.
				xcb::Event::X(x::Event::ConfigureRequest(req)) => {
					trace!(window = req.window().resource_id(), "Configuring window");
					self.conn.send_request(&x::ConfigureWindow {
						window: req.window(),
						value_list: &req.values(),
					});

					self.conn.flush()?;
				}
				// Allow clients to map their windows, but initialise those windows with AquariWM
				// afterwards.
				xcb::Event::X(x::Event::MapRequest(req)) => {
					let window = req.window();
					trace!(window = window.resource_id(), "Mapping window");

					self.conn.send_request(&x::MapWindow { window });
					// TODO: Check if this is necessary. Maybe events can be registered fine
					//       before a window is mapped, and that it is only that they are cleared
					//       when it is unmapped?
					self.conn.flush()?;

					debug!(
						window = window.resource_id(),
						"Setting up newly mapped window"
					);
					crate::init_window(&self.conn, &window)?;
					self.conn.flush()?;
				}
				// Start window manipulation if no other window manipulation is already happening.
				// This event is only fired when the Super key (mod mask 4) is pressed.
				xcb::Event::X(x::Event::ButtonPress(notif)) => {
					if self.manipulation.is_none() {
						let window = notif.event();
						let cursor_pos = (notif.root_x(), notif.root_y());

						// If the primary mouse button is pressed, and there is no ongoing window
						// manipulation, start moving the window.
						if notif.detail() == x::ButtonIndex::N1 as u8 {
							self.manipulation =
								Some(WindowManipulation::moving(&self.conn, window, cursor_pos)?);
						}

						// If the secondary mouse button is pressed, and there is no ongoing
						// window manipulation, start resizing the window.
						if notif.detail() == x::ButtonIndex::N3 as u8 {
							self.manipulation = Some(WindowManipulation::resizing(
								&self.conn, window, cursor_pos,
							)?);
						}
					}
				}
				// End window manipulation if one is occurring and its button is released. This
				// is only fired when the Super key (mod mask 4) is pressed.
				xcb::Event::X(x::Event::ButtonRelease(notif)) => {
					// If there is an ongoing window manipulation and the appropriate key for the
					// type of window manipulation is pressed, end the window manipulation by
					// setting the window manipulation to `None`.
					if self.manipulation.is_some() {
						let manipulation = self.manipulation.unwrap();

						if manipulation.is_moving() && notif.detail() == x::ButtonIndex::N1 as u8 {
							self.manipulation = None;
						}

						if manipulation.is_resizing() && notif.detail() == x::ButtonIndex::N3 as u8
						{
							self.manipulation = None;
						}
					}
				}
				// If a window manipulation is occurring, update the window's position or size
				// accordingly.
				//
				// Motion events are only received when the Super key (mod mask 4) is pressed, and
				// a mouse button is held down.
				xcb::Event::X(x::Event::MotionNotify(notif)) => {
					if self.manipulation.is_some() {
						self.manipulation
							.unwrap()
							.apply(&self.conn, (notif.root_x(), notif.root_y()))?;
					}
				}
				// Focus whatever window the cursor enters.
				xcb::Event::X(x::Event::EnterNotify(notif)) => {
					trace!(
						window = notif.event().resource_id(),
						"Focusing window entered by cursor"
					);
					self.conn.send_request(&x::SetInputFocus {
						revert_to: x::InputFocus::Parent,
						focus: notif.event(),
						time: x::CURRENT_TIME,
					});

					self.conn.flush()?;
				}
				// Bring windows to the top of the stack as they are focused.
				xcb::Event::X(x::Event::FocusIn(notif)) => {
					trace!(
						window = notif.event().resource_id(),
						"Bringing the newly focused window to the top of the stack"
					);
					self.conn.send_request(&x::ConfigureWindow {
						window: notif.event(),
						value_list: &[x::ConfigWindow::StackMode(x::StackMode::Above)],
					});

					self.conn.flush()?;
				}
				_ => {}
			}
		}
	}
}
