// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use tracing::{debug, info, trace};

use xcb::{x, Connection, Xid};

use crate::extensions::ConfigureRequestEventExtensions;
use crate::features::WindowManipulation;
use crate::util;

/// The central object of the entire AquariWM window manager. Contains state and the event loop.
pub struct AquariWm {
	conn: Connection,
	/// Represents the ongoing manipulation of a window, if one is occurring.
	///
	/// [`None`] here means that there is no window being manipulated.
	manipulation: Option<WindowManipulation>,
}

impl AquariWm {
	/// Starts the window manager by instantiating `Self` and running the event loop.
	pub fn start(conn: Connection) -> xcb::Result<()> {
		let wm = Self {
			conn,
			manipulation: None,
		};
		wm.run()
	}

	/// Loops through queued events, blocking if none are available until there are.
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
					util::init_window(&self.conn, window);
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

							util::grab_manip_buttons(&self.conn, window);
							self.conn.flush()?;
						}

						// If the secondary mouse button is pressed, and there is no ongoing
						// window manipulation, start resizing the window.
						if notif.detail() == x::ButtonIndex::N3 as u8 {
							self.manipulation = Some(WindowManipulation::resizing(
								&self.conn, window, cursor_pos,
							)?);

							util::grab_manip_buttons(&self.conn, window);
							self.conn.flush()?;
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

						if (manipulation.is_moving() && notif.detail() == x::ButtonIndex::N1 as u8)
							|| (manipulation.is_resizing()
								&& notif.detail() == x::ButtonIndex::N3 as u8)
						{
							debug!(
								window = manipulation.window().resource_id(),
								"Ending window manipulation"
							);

							trace!(
								window = manipulation.window().resource_id(),
								"Ungrabbing buttons on window"
							);
							self.conn.send_request(&x::UngrabButton {
								grab_window: manipulation.window(),
								button: x::ButtonIndex::Any,
								modifiers: x::ModMask::ANY,
							});
							util::init_grabs(&self.conn, manipulation.window());

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
					// TODO: Don't focus the window if the `EnterNotify` event was generated in
					//       response to the window being brought to the top of the stack because
					//       it was focused. Currently, this can easily lead to an infinite loop.
					//       Tip: ICCCM or EWMH may contain information regarding placing a flag
					//            somewhere that can help with this? I haven't looked into it.
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
