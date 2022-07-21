// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use x::Window;
use xcb::{x, Connection};

/// Sets up the window manager and registers for various events with the X server.
pub fn init(conn: &Connection, screen_num: usize) -> xcb::Result<()> {
	// Get the relevant screen and root window from the connection object using the `screen_num`
	// provided by `xcb::Connection::connect`.
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
	.expect("Uh oh! Failed to start AquariWM because there is already a window manager running.");

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

	// Receive the responses to the requests. This unnecessarily waits for every reply, but it
	// should avoid most of the overhead of the X server... has potential to be optimized if
	// needed. You'd want to asynchronously receive the replies for all of the cookies,
	// immediately moving on to registering events on windows as they get their respective
	// replies.
	let responses = cookies.map(|cookie| conn.wait_for_reply(cookie));

	// Pair the responses with their corresponding windows to make it easier to associate each
	// reply with its window.
	let pairs = children.iter().zip(responses);

	// Register to receive events on every viewable window by comparing the window attributes.
	pairs
		.filter(|(_, response)| response.is_ok())
		.for_each(|(child, response)| {
			if response.unwrap().map_state() == x::MapState::Viewable {
				register_for_events(conn, *child)
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
/// time. The function sends a `xcb::x::ChangeWindowAttributes` request to the X server, adding
/// event masks for the following events:
/// - `xcb::x::EventMask::ENTER_WINDOW`
pub fn register_for_events(conn: &Connection, window: Window) -> xcb::Result<()> {
	conn.send_request(&x::ChangeWindowAttributes {
		window,
		value_list: &[x::Cw::EventMask(
			x::EventMask::ENTER_WINDOW | x::EventMask::FOCUS_CHANGE,
		)],
	});

	Ok(())
}
