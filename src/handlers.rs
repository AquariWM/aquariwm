// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use xcb::x::{self, ConfigWindow, ConfigWindowMask};
use xcb::Connection;

use crate::setup;

/// Grants all client requests to configure their windows.
///
/// The window manager need not make any modifications to client
/// [ConfigureWindow](xcb::x::ConfigureWindow) requests, as any such modifications can be made by
/// AquariWM once the window is mapped.
pub fn on_configure(conn: &Connection, req: x::ConfigureRequestEvent) -> xcb::Result<()> {
	// We create an array of the values from the request and their corresponding masks to make it
	// easy to filter out the values that aren't actually contained in the request. Sending values
	// that don't actually exist obviously breaks a lot of things.
	let fields = [
		(ConfigWindow::X(req.x().into()), ConfigWindowMask::X),
		(ConfigWindow::Y(req.y().into()), ConfigWindowMask::Y),
		(
			ConfigWindow::Width(req.width().into()),
			ConfigWindowMask::WIDTH,
		),
		(
			ConfigWindow::Height(req.height().into()),
			ConfigWindowMask::HEIGHT,
		),
		(
			ConfigWindow::BorderWidth(req.border_width().into()),
			ConfigWindowMask::BORDER_WIDTH,
		),
		(
			ConfigWindow::Sibling(req.sibling()),
			ConfigWindowMask::SIBLING,
		),
		(
			ConfigWindow::StackMode(req.stack_mode()),
			ConfigWindowMask::STACK_MODE,
		),
	];

	// The `value_mask` sent with the request is a bitmask that tells us which fields were sent in
	// the request, and which fields were not. To get the correct values, we filter the fields by
	// which fields are indicated in the `value_mask`, then map the fields to just their individual
	// values. We can then collect the iterator into a list that we can easily send to the X
	// server.
	let values: Vec<ConfigWindow> = fields
		.into_iter()
		.filter_map(|(value, mask)| req.value_mask().contains(mask).then(|| value))
		.collect();

	conn.send_request(&x::ConfigureWindow {
		window: req.window(),
		value_list: &values,
	});

	conn.flush()?;
	Ok(())
}

/// Grants client requests to map windows and registers extra events afterwards.
///
/// Extra events are registered on newly mapped windows with the [setup::register_for_events]
/// function.
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

	conn.send_request(&x::MapWindow {
		window: req.window(),
	});

	// Register for events useful to the window manager on the newly mapped window. See
	// documentation on [setup::register_for_events] for more information.
	setup::register_for_events(conn, req.window())?;

	conn.flush()?;
	Ok(())
}

/// Focuses the window entered by the pointer when the pointer enters a window.
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

/// Brings newly focused windows to the top of the stack.
pub fn on_window_focused(conn: &Connection, notif: x::FocusInEvent) -> xcb::Result<()> {
	conn.send_request(&x::ConfigureWindow {
		window: notif.event(),
		value_list: &[x::ConfigWindow::StackMode(x::StackMode::Above)],
	});

	conn.flush()?;
	Ok(())
}

/// Saves the pointer position when window position/size manipulation starts.
pub fn on_button_press(_conn: &Connection, _notif: x::ButtonPressEvent) -> xcb::Result<()> {
	// The window manager will have to the save the pointer position somewhere when a mouse button
	// is pressed in combination with the `Super` key on a window. This is so that when
	// manipulating a window's position or size, AquariWM knows where to move the window or how to
	// change its size.

	// WindowManipulation {
	//     window: x::Window,
	//     start_x: i16,
	//     start_y: i16,
	// }

	// Only one window can be manipulated at one time, and it can only be manipulated in one way
	// at a time. The current window manipulation will only cease once the relevant mouse button
	// has been released.

	Ok(())
}

/// Ends the current window manipulation if the appropriate button is released.
///
/// If there is currently a window manipulation occurring, and the mouse button released is the
/// button associated with the current type of window manipulation, then the current window
/// manipulation will finish and the information stored about the current window manipulation will
/// be removed, thus opening up the window manager to be able to begin another window manipulation
/// in the future, if wanted.
pub fn on_button_release(_conn: &Connection, _notif: x::ButtonReleaseEvent) -> xcb::Result<()> {
	Ok(())
}

/// Manipulates a window's position or size if `Super + Mouse Button` is dragged on a window.
pub fn on_drag(_conn: &Connection, _notif: x::MotionNotifyEvent) -> xcb::Result<()> {
	// If window manipulation (of position or size) is currently occuring, which AquariWM can keep
	// track of with the button press events, then this function can send requests to change the
	// position or size of the window currently being manipulated by comparing the pointer's
	// current position to its original position when the window manipulation commenced.

	Ok(())
}
