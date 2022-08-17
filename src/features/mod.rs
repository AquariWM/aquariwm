// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

/// Keeps track of the manipulation of a window. See the
/// [WindowManipulation](window_manipulation::WindowManipulation) enum for more information.
mod window_manipulation;

/// TODO: Stub
pub mod desktops;

// Re-export the [window_manipulation] module, so as to avoid repeating the name of the feature
// twice.
pub use window_manipulation::WindowManipulation;

use xcb::{x, x::Atom, Connection};

/// A structure to store interned atoms for use with X, especially for the ICCCM.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Atoms {
    pub wm_state: Atom,
}

impl Atoms {
    /// Initialise the atoms by sending `InternAtom` requests for each.
    pub fn init(conn: &Connection) -> xcb::Result<Self> {
        let wm_state_req = get_atom(conn, b"WM_STATE");

        let wm_state_reply = conn.wait_for_reply(wm_state_req)?;

        Ok(Self {
            wm_state: wm_state_reply.atom(),
        })
    }
}

fn get_atom(conn: &Connection, name: &[u8]) -> x::InternAtomCookie {
    conn.send_request(&x::InternAtom {
        only_if_exists: false,
        name,
    })
}
