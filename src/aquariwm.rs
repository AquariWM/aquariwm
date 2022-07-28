// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use xcb::x::Window;
use xcb::Connection;

/// The central object of the entire AquariWM window manager. Contains state and the event loop.
pub struct AquariWm {
	_conn: Connection,
	_root: Window,
}

impl AquariWm {
	pub fn new(conn: Connection, root: Window) -> Self {
		Self { _conn: conn, _root: root }
	}
}
