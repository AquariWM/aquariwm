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

	// Request substructure redirection on the root window.
	conn.send_request(&x::ChangeWindowAttributes {
		window: root,
		value_list: &[x::Cw::EventMask(
			x::EventMask::SUBSTRUCTURE_REDIRECT | x::EventMask::SUBSTRUCTURE_NOTIFY,
		)],
	});

	// As the substructure redirection request is not checked, we must flush the connection.
	conn.flush()?;

	// Send a request asking to receive events relating to the cursor motion when the cursor
	// enters a new window.
	let enter_window_cookie = conn.send_request(&x::GrabPointer {
		// we still want pointer events to be processed as usual
		owner_events: true,
		// we want to hear about pointer events on the root window (and all its children)
		grab_window: root,
		// we want to hear when the pointer enters a new window, so we can change focus
		// ON SECOND THOUGHT: I don't think ENTER_WINDOW is applicable to the pointer? Not quite
		//                    sure, replaced it with POINTER_MOTION for now.
		event_mask: x::EventMask::POINTER_MOTION,
		// async grab mode means that the events being grabbed are not frozen when we grab them
		pointer_mode: x::GrabMode::Async,
		keyboard_mode: x::GrabMode::Async,
		// we don't want to confine the cursor to be only within a particular window
		confine_to: x::WINDOW_NONE,
		// we don't want to overwrite the appearance of the cursor
		cursor: x::CURSOR_NONE,
		time: x::CURRENT_TIME,
	});

	// We wait for all the replies to be received at once, so that there is no need to be waiting
	// when we can be sending the other requests. As there is no reply from substructure
	// redirection, there is only one such reply for the moment.
	conn.wait_for_reply(enter_window_cookie)?;

	// This is the main event loop. In here, we wait until an event is sent to AquariWM by the X
	// server, and then, based on the event received, we choose to react accordingly, or not at
	// all.
	loop {
		// Receive the next event from the X server, when available.
		let event = conn.wait_for_event()?;

		// trunk-ignore(clippy/single_match)
		match event {
			// Now, at the moment, the window manager is not functional because we have asked the
			// X server to redirect events for the root window to us right? Well, that means it's
			// now our job to, well, manage the windows. We must react to various requests made by
			// client windows. For now, we'll simply honor the requests (listen for the required
			// types, and just send them straight over to the X server with the same parameters).
			xcb::Event::X(x::Event::MotionNotify(motion)) => {
				// Print the coordinates of the cursor when it enters a new window.
				println!(
					"The cursor position is {}, {}.",
					motion.root_x(),
					motion.root_y()
				);
			}
			_ => {}
		}
	}
}
