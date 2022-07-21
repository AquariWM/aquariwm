// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use x::Window;
use xcb::{x, Connection};

/// Sets up the window manager and registers for various events with the X server.
///
/// This function registers for the
/// [xcb::x::EventMask::SUBSTRUCTURE_REDIRECT](SUBSTRUCTURE_REDIRECT) and
/// [xcb::x::EventMask::SUBSTRUCTURE_NOTIFY](SUBSTRUCTURE_NOTIFY) events on the root window, which
/// is what allows a window manager to manage windows.
pub fn init(conn: &Connection, screen_num: usize) -> xcb::Result<()> {
	// Get the relevant screen and root window from the connection object using the `screen_num`
	// provided by [xcb::Connection::connect].
	let screen = conn.get_setup().roots().nth(screen_num).unwrap();
	let root = screen.root();

	// Request substructure redirection on the root window and send a descriptive error message if
	// the request fails.
	conn.check_request(conn.send_request_checked(&x::ChangeWindowAttributes {
		window: root,
		value_list: &[x::Cw::EventMask(
			x::EventMask::SUBSTRUCTURE_REDIRECT | x::EventMask::SUBSTRUCTURE_NOTIFY,
		)],
	}))
	.expect("Uh oh! Failed to start AquariWM because there is already a window manager running");

	// Query the X server for the existing window tree so that we can perform set up on any
	// existing windows.
	let query = conn.send_request(&x::QueryTree { window: root });

	// Receive the results of the query and get the top level windows.
	let results = conn.wait_for_reply(query)?;
	let children = results.children();

	// Request window attributes for every direct child of the root window.
	let cookies = children
		.iter()
		.map(|child| conn.send_request(&x::GetWindowAttributes { window: *child }));

	// Receive responses for the [x::GetWindowAttributes](GetWindowAttributes) requests and
	// register for events on each [x::MapState::Viewable](Viewable) window.
	cookies
		.map(|cookie| conn.wait_for_reply(cookie))
		.zip(children)
		.filter(|(response, _)| response.is_ok())
		.for_each(|(response, window)| {
			if response.unwrap().map_state() == x::MapState::Viewable {
				register_for_events(conn, *window)
					.expect("Failed to register additional events on a pre-existing window");
			}
		});

	conn.flush()?;
	Ok(())
}

/// Registers to receive useful events for the given window.
///
/// This function is used for setting up existing mapped windows when the window manager is
/// first launched, as well as for windows when they are mapped by the window manager at any other
/// time. The function sends a [xcb::x::ChangeWindowAttributes](ChangeWindowAttributes) request to
/// the X server, adding event masks for the following events:
/// - [xcb::x::EventMask::ENTER_WINDOW](EventMask::ENTER_WINDOW)
/// - [xcb::x::EventMask::FOCUS_CHANGE](EventMask::FOCUS_CHANGE)
pub fn register_for_events(conn: &Connection, window: Window) -> xcb::Result<()> {
	conn.send_request(&x::ChangeWindowAttributes {
		window,
		value_list: &[x::Cw::EventMask(
			x::EventMask::ENTER_WINDOW | x::EventMask::FOCUS_CHANGE,
		)],
	});

	Ok(())
}
