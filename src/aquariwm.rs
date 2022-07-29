// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use tracing::{debug, info, trace};

use xcb::x::{self, Window};
use xcb::{Connection, Xid};

use crate::extensions::ConfigureRequestEventExtensions;
use crate::window_manipulation::{WindowManipulation, Type};

/// The central object of the entire AquariWM window manager. Contains state and the event loop.
#[allow(dead_code)]
pub struct AquariWm {
	conn: Connection,
	/// Represents the ongoing manipulation of a window, if one is occurring.
	///
	/// [`None`] here means that there is no window being manipulated.
	manipulation: Option<WindowManipulation>,
}

#[allow(dead_code)]
impl AquariWm {
	/// Starts the window manager by instantiating `Self` and running the event loop.
	pub fn start(conn: Connection) -> xcb::Result<()> {
		let wm = Self {
			conn,
			manipulation: None,
		};
		wm.run()
	}

	/// Gets the currently occuring window manipulation, if any.
	///
	/// Returns `Some(WindowManipulation)` if there is a window being manipulated, or `None` if
	/// there is no window manipulation happening.
	pub fn manipulation(&self) -> Option<WindowManipulation> {
		self.manipulation
	}

	/// Begins window manipulation for the given window if no other window is being manipulated.
	///
	/// Returns [`Ok(())`] if the window manipulation was able to begin, or [`Err(())`] if there was
	/// already a window being manipulated.
	pub fn manipulate_window(&mut self, window: Window, cursor_pos: (i16, i16), mode: Type) -> Result<(), ()> {
		if self.manipulation.is_none() {
			self.manipulation = Some(WindowManipulation {
				window,
				cursor_pos,
				mode,
			});

			// We were able to begin the window manipulation, as no other window was already being
			// manipulated.
			return Ok(());
		}

		// There was already a window being manipulated, so we were not able to begin the window
		// manipulation.
		Err(())
	}

	/// Cancels the current [`WindowManipulation`], if any. Should only be used if user-initiated.
	///
	/// This function [takes](Option::take) the current [`WindowManipulation`] and returns it. The
	/// window manager's manipulation state is set to [`None`].
	pub fn cancel_manipulation(&mut self) -> Option<WindowManipulation> {
		self.manipulation.take()
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
