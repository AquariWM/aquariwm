// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

// Some potentially helpful reference resources:
//
// how to write a window manager----
//              https://jich4n.com/posts/how-x-window-managers-work-and-how-to-write-one-part-i/
// ICCCM                            https://tronche.com/gui/x/icccm/
// EWMH                             https://specifications.freedesktop.org/wm-spec/latest/
// tinywm, a helpful reference      http://incise.org/tinywm.html
// XCB tutorial                     https://xcb.freedesktop.org/tutorial/
// XCB window manipulation          https://xcb.freedesktop.org/windowcontextandmanipulation/

// the code below isn't representative of the features of AquariWM... this is simply a test
// implementation so I can make sure the basics all work and to get some experience with them.

use xcb::x;

fn main() -> xcb::Result<()> {
	// connect to the X server
	let (conn, screen_num) = xcb::Connection::connect(None)?;

	// Get the `x::Screen` object from the connection's `x::Setup` with the `screen_num`.
	let screen = conn.get_setup().roots().nth(screen_num as usize).unwrap();
	// Get the screen's root window.
	let root = screen.root();

	// Send a request asking to receive events relating to the cursor motion when the cursor
	// enters a new window.
	let _enter_window_cookie = conn.send_request(&x::GrabPointer {
		// we still want pointer events to be processed as usual
		owner_events: true,
		// we want to hear about pointer events on the root window (and all its children)
		grab_window: root,
		// we want to hear when the pointer enters a new window, so we can change focus
		event_mask: x::EventMask::ENTER_WINDOW,
		// async grab mode means that the events being grabbed are not frozen when we grab them
		pointer_mode: x::GrabMode::Async,
		keyboard_mode: x::GrabMode::Async,
		// we don't want to confine the cursor to be only within a particular window
		confine_to: x::WINDOW_NONE,
		// we don't want to overwrite the appearance of the cursor
		cursor: x::CURSOR_NONE,
		time: x::CURRENT_TIME,
	});

	// This is the main event loop. In here, we wait until an event is sent to AquariWM by the X
	// server, and then, based on the event received, we choose to react accordingly, or not at
	// all.
	loop {
		match conn.wait_for_event()? {
			xcb::Event::X(x::Event::MotionNotify(event)) => {
				// print the coordinates of the pointer when pointer motion is received
				println!(event.root_x() + ", " + event.root_y());
			}
			_ => {}
		}
	}
}
