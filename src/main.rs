// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

// See:
// ============================
// rust-xcb docs - https://rust-x-bindings.github.io/rust-xcb/xcb/index.html
// rust-xcb repo - https://github.com/rust-x-bindings/rust-xcb
//
// how X window managers work and how to write one:
//      https://jichu4n.com/posts/how-x-window-managers-work-and-how-to-write-one-part-i/
//
// Inter-Client Communication Conventions Manual - https://tronche.com/gui/x/icccm/
// Extended Window Manager Hints - https://specifications.freedesktop.org/wm-spec/latest/

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

fn main() {
    // stub
}

// Window managers in X are simply clients that have permission to perform substructure redirection
// on the root window. Only one such client can be active at once. Start by informing X that we
// would like to select input on the root window for the substrcuture redirect mask and the
// substructure notify mask.

// Start by telling X what we want to receive events for. We want to receive input events for
// cursor movement (motion events), Super + Left Mouse Button, Super + Right Mouse Button, and
// Super + f.

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
