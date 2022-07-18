// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

// Specifications relevant to AquariWM that it should follow where possible and reasonable:
// ICCCM    https://tronche.com/gui/x/icccm/
// EWMH     https://specifications.freedesktop.org/wm-spec/latest/

pub mod handlers;

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
	setup(&conn, screen_num as usize)?;

	// Run the event loop and return its value (that's why the semicolon is missing).
	run(conn)
}

/// Set up the window manager and register for various events with the X server.
fn setup(conn: &xcb::Connection, screen_num: usize) -> xcb::Result<()> {
	// Get the relevant screen and root window from the connection object using the `screen_num`
	// provided by `xcb::Connection::connect`.
	let screen = conn.get_setup().roots().nth(screen_num).unwrap();
	let root = screen.root();

	// Request substructure redirection on the root window.
	// TODO: error handling for when a window manager is already running
	conn.send_request(&x::ChangeWindowAttributes {
		window: root,
		value_list: &[x::Cw::EventMask(
			x::EventMask::SUBSTRUCTURE_REDIRECT | x::EventMask::SUBSTRUCTURE_NOTIFY,
		)],
	});

	// Flush the queued request to the X server.
	conn.flush()?;
	Ok(())
}

/// The main event loop of the window manager, where it handles received events.
///
/// The event loop waits until the program receives a new event from the X server, and then, based
/// on the event type received, it reacts accordingly (sending new requests to the X server when
/// necessary).
fn run(conn: xcb::Connection) -> xcb::Result<()> {
	loop {
		// Receive the next event from the X server, when available, and match against its type.
		match conn.wait_for_event()? {
			xcb::Event::X(x::Event::CreateNotify(notif)) => {
				handlers::on_create(&conn, notif)?;
			}
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
			xcb::Event::X(x::Event::FocusOut(notif)) => {
				handlers::on_window_unfocused(&conn, notif)?;
			}
			// Ignore any other events.
			_ => {}
		}
	}
}
