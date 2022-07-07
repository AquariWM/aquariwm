// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

///////////////////////////////////////////////////////////////////////////////////////////////////

// There are a number of resources that you may find helpful when writing this first stub.
//
// rust-xcb documentation           https://rust-x-bindings.github.io/rust-xcb/xcb/index.html
// rust-xcb github                  https://github.com/rust-x-bindings/rust-xcb
// how to write a window manager----
//              https://jich4n.com/posts/how-x-window-managers-work-and-how-to-write-one-part-i/
// ICCCM                            https://tronche.com/gui/x/icccm/
// EWMH                             https://specifications.freedesktop.org/wm-spec/latest/
// tinywm, a helpful reference      http://incise.org/tinywm.html
// the rust programming book        https://doc.rust-lang.org/stable/book/
// XCB tutorial                     https://xcb.freedesktop.org/tutorial/
// XCB window manipulation          https://xcb.freedesktop.org/windowcontextandmanipulation/

///////////////////////////////////////////////////////////////////////////////////////////////////

// The following code and features are in no way representative of what AquariWM is designed to be,
// nor what it will be for much longer. It is simply a test, to see what the implementation of
// basic window manager functions will involve...
//
// If you are reading this message, and you would like to contribute to the window manager: do feel
// free to remove all of the contents of this file and start over (but make sure to include the
// license header for the MPL-2.0). The first tasks for the real window manager design are to get
// the fundamentals of module communication set up: try to have a custom message sent through an
// IPC socket to AquariWM with information on where to put the focused window, and then have the
// AquariWM core move the focused window based on those coordinates received over IPC.

fn main() -> xcb::Result<()> {
	// connect to the X server
	let (conn, screen_num) = xcb::Connection::connect(None)?;

	// Get the `x::Screen` object from the connection's `x::Setup` with the `screen_num`.
	let screen = conn.get_setup().roots().nth(screen_num as usize).unwrap();
	// Get the screen's root window.
	let _root = screen.root();

	// The concept of a 'window manager' in X is simply a client that has permission to perform
	// substructure redirection on the root window. Only one X client is allowed to do this at
	// once. We therefore register for substructure redirection on the root window:

	// register for substructure redirection on the root window...

	// Potentially helpful example code (Rust, XCB):
	// https://github.com/mjkillough/lanta/blob/4c31f087514502f243eb15ac0f1a57072aa8779c/src/x.rs#L173
	//
	// xcb::EVENT_MASK_SUBSTRUCTURE_NOTIFY
	// xcb::EVENT_MASK_SUBSTRUCTURE_REDIRECT
	// (substructure redirect on root window with xcb::change_window_attributes_checked)

	// Next, we want to tell X what events we want to know about. We need to receive input events
	// relating to cursor movement (motion events), Super + Left Mouse Button,
	// Super + Right Mouse Button, and Super + f.

	// select mouse motion input
	// grab Super + Left Mouse Button
	// grab Super + Right Mouse Button
	// grab Super + f

	// event loop {
	// //
	// // mouse motion {
	// // // if (hovered window != focused window) focus(hovered window)
	// // }
	// //
	// // Super + Left Mouse Button -> move window
	// // Super + Right Mouse Button -> resize window
	// // Super + F -> toggle fullscreen on the window
	// //
	// }

	Ok(())
}
