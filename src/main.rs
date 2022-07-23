// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

// Specifications relevant to AquariWM that it should follow where possible and reasonable:
// ICCCM    https://tronche.com/gui/x/icccm/
// EWMH     https://specifications.freedesktop.org/wm-spec/latest/

/// Useful trait extensions for [xcb] to provide easier access to certain utilities.
pub mod utility;

/// Handles events from the event loop.
///
/// This module contains functions to handle certain events from the event loop. It helps to split
/// the handling for different events into different functions for the sake of readability.
mod handlers;

/// Provides functions for the setup/initialization of both the window manager and new windows.
mod setup;

use xcb::x;

/// A primitive base window manager implementation for AquariWM to build upon.
///
/// This is not the proper implementation of AquariWM and its module system, but rather a very
/// basic floating window manager that can be built upon in time. It supports the basic
/// functions of moving windows, resizing windows, focusing a particular window, and toggling
/// fullscreen for the focused window.
pub fn main() -> xcb::Result<()> {
	// Connect to the X server.
	let (conn, screen_num) = xcb::Connection::connect(None)?;

	// Set up the window manager, i.e. register for substructure redirection on the root window
	// and grab other relevant events.
	setup::init(&conn, screen_num as usize)?;

	// Run the event loop and return its value (that's why the semicolon is missing).
	run(conn)
}

/// Receives events from the X server and calls the appropriate event handlers from the
/// [handlers] module.
///
/// The event loop waits until the program receives a new event from the X server, and then, based
/// on the event type received, it reacts accordingly (sending new requests to the X server when
/// necessary).
fn run(conn: xcb::Connection) -> xcb::Result<()> {
	loop {
		// Receive the next event from the X server, when available, and match against its type.
		match conn.wait_for_event()? {
			xcb::Event::X(x::Event::ConfigureRequest(req)) => {
				handlers::on_configure(&conn, req)?;
			}
			xcb::Event::X(x::Event::MapRequest(req)) => {
				handlers::on_map(&conn, req)?;
			}
			xcb::Event::X(x::Event::EnterNotify(notif)) => {
				handlers::on_window_enter(&conn, notif)?;
			}
			xcb::Event::X(x::Event::FocusIn(notif)) => {
				handlers::on_window_focused(&conn, notif)?;
			}
			xcb::Event::X(x::Event::ButtonPress(notif)) => {
				handlers::on_button_press(&conn, notif)?;
			}
			xcb::Event::X(x::Event::ButtonRelease(notif)) => {
				handlers::on_button_release(&conn, notif)?;
			}
			xcb::Event::X(x::Event::MotionNotify(notif)) => {
				handlers::on_drag(&conn, notif)?;
			}
			// Ignore any other events.
			_ => {}
		}
	}
}
