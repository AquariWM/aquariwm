// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use tracing::{info, trace};

use xcb::x::{self, Window};
use xcb::{Connection, Xid};

/// The central object of the entire AquariWM window manager. Contains state and the event loop.
pub struct AquariWm {
	conn: Connection,
	_root: Window,
}

impl AquariWm {
    /// Instantiates the window manager object with the given [Connection] and root
    /// [window](Window).
	pub fn new(conn: Connection, root: Window) -> Self {
		Self { conn, _root: root }
	}

    /// Runs the event loop - the very core of the window manager which receives events.
    pub fn run(&self) -> xcb::Result<()> {
        info!("Running the window manager");
        loop {
            match self.conn.wait_for_event()? {
                xcb::Event::X(x::Event::ConfigureRequest(req)) => {
                    trace!("Processing request to configure window ({})", req.window().resource_id());
                }
                xcb::Event::X(x::Event::MapRequest(req)) => {
                    trace!("Processing request to map window ({})", req.window().resource_id());
                }
                xcb::Event::X(x::Event::ButtonPress(notif)) => {
                    trace!("Processing button press ({})", notif.detail());
                }
                xcb::Event::X(x::Event::ButtonRelease(notif)) => {
                    trace!("Processing button release ({})", notif.detail());
                }
                xcb::Event::X(x::Event::MotionNotify(_)) => {
                    trace!("Processing cursor drag");
                }
                xcb::Event::X(x::Event::EnterNotify(notif)) => {
                    trace!("Processing cursor-entered-window event ({})", notif.event().resource_id());
                }
                xcb::Event::X(x::Event::FocusIn(notif)) => {
                    trace!("Processing newly focused window ({})", notif.event().resource_id());
                }
                _ => {}
            }
        }
    }
}
