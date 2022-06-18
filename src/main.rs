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

fn main() -> xcb::Result<()> {
    let _conn = xcb::Connection::connect(None)?;

    print!("Woohoo! The crowd goes wild! This should be an X connection that disconnects ");
    print!("when <_conn> goes out of sync...");

    Ok(())
}
