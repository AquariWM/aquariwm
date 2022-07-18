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

	// send out requests to grab the pointer for the ENTER_WINDOW, BUTTON1_MOTION, BUTTON2_MOTION
	// event masks... I think this might supposed to be combined into one request, possibly even
	// needs to be to function? not sure, this is pretty much pseudocode right now I guess...

	let enter_window_cookie = conn.send_request(&x::GrabPointer {
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

	// for these button1 and button3 pointer grabs, we only actually need to receive the
	// information when the super key (also known as the meta key, windows key, command key, GUI
	// key, etc.) is pressed. perhaps that can be specified as part of the request? or maybe we
	// only grab the pointer when the super key is pressed, and ungrab it when it is released... or
	// maybe it doesn't matter, and we can be perfectly fine to just receive all of the
	// click-and-drags, but only react when the super key is pressed. I'm not sure which is best
	// right now.

	let button1_cookie = conn.send_request(&x::GrabPointer {
		// when moving a window, we don't actually want anything else to receive the drag inputs
		owner_events: false,
		grab_window: root,
		// button1 is the primary mouse button, typically known as the left mouse button
		event_mask: x::EventMask::BUTTON1_MOTION,
		pointer_mode: x::GrabMode::Async,
		keyboard_mode: x::GrabMode::Async,
		confine_to: x::WINDOW_NONE,
		cursor: x::CURSOR_NONE,
		time: x::CURRENT_TIME,
	});

	let button3_cookie = conn.send_request(&x::GrabPointer {
		owner_events: false,
		grab_window: root,
		// button3 is actually the secondary mouse button, a.k.a. right mouse button, as the
		// middle mouse button is the scroll wheel
		event_mask: x::EventMask::BUTTON3_MOTION,
		pointer_mode: x::GrabMode::Async,
		keyboard_mode: x::GrabMode::Async,
		confine_to: x::WINDOW_NONE,
		cursor: x::CURSOR_NONE,
		time: x::CURRENT_TIME,
	});

	// only after sending out the requests do we wait for their replies. we don't want to waste
	// time waiting for one reply when we could be sending another request!

	let _enter_window_reply = conn.wait_for_reply(enter_window_cookie)?;
	let _button1_reply = conn.wait_for_reply(button1_cookie)?;
	let _button3_reply = conn.wait_for_reply(button3_cookie)?;

	// main event loop
	loop {
		match conn.wait_for_event()? {
			// this is the main event loop. in here, we receive the latest event (wait_for_event
			// is an iterator) and match against different event types to determine how, or if, to
			// react.
			_ => {}
		}
	}
}
