// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use xcb::{Connection, x};
use xcb::x::Window;

use xcb_wm::ewmh;
use ewmh::proto as e;

/// Initializes the window manager for EWMH compliance.
///
/// Creates a child window of the root window so that it can set the `_NET_SUPPORTING_WM_CHECK`
/// atom on the root window and that child window, as well as the `_NET_WM_NAME` atom on the child
/// window so that AquariWM can be recognised as an EWMH-compliant window manager.
///
/// This function also initialises an 'EWMH connection' by wrapping [xcb::Connection]. This is
/// returned by the function.
///
/// > # [Extended Window Manager Hints](https://freedesktop.org/wiki/Specifications/wm-spec/latest)
/// > ### `_NET_SUPPORTING_WM_CHECK`, `WINDOW`/`32`
/// > The Window Manager __MUST__ set this property on the root window to be the ID of a child
/// > window created by himself, to indicate that a compliant window manager is active. The child
/// > window __MUST__ also have the `_NET_SUPPORTING_WM_CHECK` property set to the ID of the child
/// > window. The child window __MUST__ also have the `_NET_WM_NAME` property set to the name of
/// > the Window Manager.  
///
/// Does not flush the connection.
pub fn init(conn: &Connection, root: Window) -> ewmh::Connection {
	// Create a wrapper around `conn` for use with EWMH. This creates all the relevant `ewmh`
	// atoms and lets us send certain `ewmh` messages (but apparently that excludes setting the
	// `_NET_SUPPORTING_WM_CHECK` atom).
	let ewmh_conn = ewmh::Connection::connect(conn);

	// Generate a `resource_id` for the child window.
	let window: Window = conn.generate_id();

	// Create the child window.
	conn.send_request(&x::CreateWindow {
		// The 'depth' of the window is 0; will be created as a direct child.
		depth: 0,
		// The `resource_id` that the window will use.
		wid: window,
		// The window is a child of the root window.
		parent: root,
		// Coordinates don't matter for us; we set these to 0.
		x: 0,
		y: 0,
		// Dimensions don't matter for us; we set these to 1, their minimums.
		width: 1,
		height: 1,
		// The border width doesn't matter for us; we set it to 0.
		border_width: 0,
		// The window's class doesn't matter to us; we set it to `InputOnly`.
		class: x::WindowClass::InputOnly,
		// The window's visual (something to do with colormaps) doesn't matter to us; we set it to
		// 0.
		visual: 0,
		// We don't wish to configure the window in any other way, so we provide an empty
		// `value_list`.
		value_list: &[],
	});

	// Set the name of the window manager to [`crate::NAME`]. This is done by setting the title of
	// AquariWM's child window that we are about to 'register' next.
	ewmh_conn.send_request(&e::SetWmName::new(window, crate::NAME));

	// Register for the `_NET_SUPPORTING_WM_CHECK` atom on the root window.
	conn.send_request(&x::ChangeProperty {
		mode: x::PropMode::Replace,
		window: root,
		property: ewmh_conn.atoms._NET_SUPPORTING_WM_CHECK,
		r#type: x::ATOM_WINDOW,
		data: &[window],
	});

	// Register for the `_NET_SUPPORTING_WM_CHECK` atom on the child window.
	conn.send_request(&x::ChangeProperty {
		mode: x::PropMode::Replace,
		window,
		property: ewmh_conn.atoms._NET_SUPPORTING_WM_CHECK,
		r#type: x::ATOM_WINDOW,
		data: &[window],
	});

	ewmh_conn
}
