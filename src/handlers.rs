// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use xcb::{x, Connection};

use crate::setup;

/// Handle a client's request to configure a window.
///
/// When the window manager receives a request to configure a window, this function is called.
/// Currently, we simply send the exact same request back to the X server with no changes, but we
/// may wish to modify this request in the future. X clients must accept any modification we make
/// to their requests.
pub fn on_configure(conn: &Connection, req: x::ConfigureRequestEvent) -> xcb::Result<()> {
	// We simply send an identical request back to the X server. The numerical units received here
	// must be converted with `.into()` as the size of integer received in the `ConfigureRequest`
	// is smaller than that which the X server expects.
	conn.send_request(&x::ConfigureWindow {
		window: req.window(),
		value_list: &[
			x::ConfigWindow::X(req.x() as i32),
			x::ConfigWindow::Y(req.y() as i32),
			x::ConfigWindow::Width(req.width() as u32),
			x::ConfigWindow::Height(req.height() as u32),
			x::ConfigWindow::BorderWidth(req.border_width() as u32),
			x::ConfigWindow::Sibling(req.sibling()),
			x::ConfigWindow::StackMode(req.stack_mode()),
		],
	});

	conn.flush()?;
	Ok(())
}

/// Handle a client's request to map a window, reparenting if necessary.
///
/// Here we simply 'bounce back' a MapRequest to the X server, but in the future we can create a
/// frame window here and reparent the client window to it so that window decorations can exist.
pub fn on_map(conn: &Connection, req: x::MapRequestEvent) -> xcb::Result<()> {
	// In the real window manager, this is where the decorator module would come in. The decorator
	// module's job would be to populate a frame around the window with window decorations, such
	// as a title bar, a close button, etc. We would first ask the decorator module if it even
	// wants to decorate the window in particular, as there's no point in creating a frame for a
	// window that doesn't need one. If the decorator module wants to decorate the window, we can
	// create a new frame window with the appropriate position and size given by the layout
	// module, and then ask the decorator module to do its thing. The decorator module would send
	// us a reply indicating the area left, free from decoration, to place the real window within.
	// We would register for substructure redirection on the frame window (as substructure
	// redirection only applies to direct children, and this would make the real window a direct
	// child of the frame window, instead of the root window), and then reparent the real window
	// to this frame window. Finally, after all that, we could map the frame window and the real
	// window on top.
	//
	// We're not actually doing any of that right now though. Now we're just mapping the 'real
	// window' directly with no window decorations.

	let cookie = conn.send_and_check_request(&x::MapWindow {
		window: req.window(),
	})?;





	setup::register_mapped_window(conn, &req.window())?;

	conn.flush()?;
	Ok(())
}

/// Focus a window when the pointer enters it.
pub fn on_window_enter(conn: &Connection, notif: x::EnterNotifyEvent) -> xcb::Result<()> {
	// Focus the window and revert the focus to the parent window if the window is hidden or
	// destroyed.
	conn.send_request(&x::SetInputFocus {
		revert_to: x::InputFocus::Parent,
		focus: notif.event(),
		time: x::CURRENT_TIME,
	});

	conn.flush()?;
	Ok(())
}

/// When a window is focused, bring it to the front.
pub fn on_window_focused(conn: &Connection, notif: x::FocusInEvent) -> xcb::Result<()> {
	conn.send_request(&x::ConfigureWindow {
		window: notif.event(),
		value_list: &[x::ConfigWindow::StackMode(x::StackMode::Above)],
	});

	conn.flush()?;
	Ok(())
}
