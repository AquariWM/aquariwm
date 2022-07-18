// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

// Specifications relevant to AquariWM that it should follow where possible and reasonable:
// ICCCM    https://tronche.com/gui/x/icccm/
// EWMH     https://specifications.freedesktop.org/wm-spec/latest/

// The code below isn't representative of the features of AquariWM, this is simply a test
// implementation so I can make sure the basics all work and to get some experience with them.

mod handlers;

use xcb::x;

/// A primitive base window manager implementation for AquariWM to build upon.
///
/// This is not the proper implementation of AquariWM and its module system, but rather a very
/// basic floating window manager that can be built upon in time. It supports the basic
/// functions of moving windows, resizing windows, focusing a particular window, and toggling
/// fullscreen for the focused window.
fn main() -> xcb::Result<()> {
	// Connect to the X server.
	let (conn, screen_num) = xcb::Connection::connect(None)?;

	// Get the relevant screen and root window from the connection object using the `screen_num`
	// provided by `xcb::Connection::connect`.
	let screen = conn.get_setup().roots().nth(screen_num as usize).unwrap();
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

	// Send a request asking to receive events relating to the cursor motion.
	let cookie = conn.send_request(&x::GrabPointer {
		// We still want pointer events to be processed as usual.
		owner_events: true,
		// We want to hear about pointer events on the root window (and all its children).
		grab_window: root,
		// We want to hear about the movement of the pointer.
		event_mask: x::EventMask::POINTER_MOTION,
		// Async grab mode means that the events being grabbed are not frozen when we grab them.
		pointer_mode: x::GrabMode::Async,
		keyboard_mode: x::GrabMode::Async,
		// We don't want to confine the cursor to be only within a particular window.
		confine_to: x::WINDOW_NONE,
		// We don't want to overwrite the appearance of the cursor.
		cursor: x::CURSOR_NONE,
		time: x::CURRENT_TIME,
	});

	// We wait for all the replies to be received at once, so that there is no need to be waiting
	// when we can be sending the other requests. As there is no reply from substructure
	// redirection, there is only one such reply for the moment.
	// TODO: do we have to wait for the reply? perhaps we can flush the connection just like when
	//       we aren't expecting any reply? since we do nothing with the reply, it might be better
	//       to flush if it is possible.
	conn.wait_for_reply(cookie)?;

	// Run the event loop and return its value (that's why the semicolon is missing).
	run(conn)
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
			xcb::Event::X(x::Event::ConfigureRequest(req)) => {
				handlers::on_configure(&conn, req)?;
			}
			xcb::Event::X(x::Event::MapRequest(req)) => {
				handlers::on_map(&conn, req)?;
			}
			// Ignore any other events.
			_ => {}
		}
	}
}
