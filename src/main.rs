// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

// Specifications relevant to AquariWM that it should follow where possible and reasonable:
// ICCCM    https://tronche.com/gui/x/icccm/
// EWMH     https://specifications.freedesktop.org/wm-spec/latest/

// The code below isn't representative of the features of AquariWM, this is simply a test
// implementation so I can make sure the basics all work and to get some experience with them.

use xcb::x;

/// A primitive base window manager implementation for AquariWM to build upon.
///
/// This is not the proper implementation of AquariWM and its module system, but rather a very
/// basic floating window manager that can be built upon in time. It supports the basic
/// functions of moving windows, resizing windows, focusing a particular window, and toggling
/// fullscreen for the focused window.
fn main() -> xcb::Result<()> {
	// Connect to the X server.
	let (conn, screen_num) = xcb::Connection::connect(None)?;

	// Get the relevant screen and root window from the connection object using the `screen_num`
	// provided by `xcb::Connection::connect`.
	let screen = conn.get_setup().roots().nth(screen_num as usize).unwrap();
	let root = screen.root();

	// Request substructure redirection on the root window.
	// TODO: error handling for when a window manager is already running
	conn.send_request(&x::ChangeWindowAttributes {
		window: root,
		value_list: &[x::Cw::EventMask(
			x::EventMask::SUBSTRUCTURE_REDIRECT | x::EventMask::SUBSTRUCTURE_NOTIFY,
		)],
	});

	// Flush the queued request to the X server.
	conn.flush()?;

	// Send a request asking to receive events relating to the cursor motion.
	let cookie = conn.send_request(&x::GrabPointer {
		// We still want pointer events to be processed as usual.
		owner_events: true,
		// We want to hear about pointer events on the root window (and all its children).
		grab_window: root,
		// We want to hear about the movement of the pointer.
		event_mask: x::EventMask::POINTER_MOTION,
		// Async grab mode means that the events being grabbed are not frozen when we grab them.
		pointer_mode: x::GrabMode::Async,
		keyboard_mode: x::GrabMode::Async,
		// We don't want to confine the cursor to be only within a particular window.
		confine_to: x::WINDOW_NONE,
		// We don't want to overwrite the appearance of the cursor.
		cursor: x::CURSOR_NONE,
		time: x::CURRENT_TIME,
	});

	// We wait for all the replies to be received at once, so that there is no need to be waiting
	// when we can be sending the other requests. As there is no reply from substructure
	// redirection, there is only one such reply for the moment.
	// TODO: do we have to wait for the reply? perhaps we can flush the connection just like when
	//       we aren't expecting any reply? since we do nothing with the reply, it might be better
	//       to flush if it is possible.
	conn.wait_for_reply(cookie)?;

	// Run the event loop and return its value (that's why the semicolon is missing).
	run(conn)
}

/// The main event loop of the window manager, where it handles received events.
///
/// The event loop waits until the program receives a new event from the X server, and then, based
/// on the event type received, it reacts accordingly (sending new requests to the X server when
/// necessary).
fn run(conn: xcb::Connection) -> xcb::Result<()> {
	loop {
		// Receive the next event from the X server, when available, and match against its type.
		match conn.wait_for_event()? {
			// Honor window configure requests completely, for now.
			xcb::Event::X(x::Event::ConfigureRequest(req)) => {
				on_configure(&conn, req)?;
			}
			// Map windows after creation.
			xcb::Event::X(x::Event::MapRequest(req)) => {
				on_map(&conn, req)?;
			}
			// Ignore any other events.
			_ => {}
		}
	}
}

/// Handle a client's request to configure a window.
///
/// When the window manager receives a request to configure a window, this function is called.
/// Currently, we simply send the exact same request back to the X server with no changes, but we
/// may wish to modify this request in the future. X clients must accept any modification we make
/// to their requests.
fn on_configure(conn: &xcb::Connection, req: x::ConfigureRequestEvent) -> xcb::Result<()> {
	// We simply send an identical request back to the X server. The numerical units received here
	// must be converted with `.into()` as the size of integer received in the `ConfigureRequest`
	// is smaller than that which the X server expects.
	conn.send_request(&x::ConfigureWindow {
		window: req.window(),
		value_list: &[
			x::ConfigWindow::X(req.x().into()),
			x::ConfigWindow::Y(req.y().into()),
			x::ConfigWindow::Width(req.width().into()),
			x::ConfigWindow::Height(req.height().into()),
			x::ConfigWindow::BorderWidth(req.border_width().into()),
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
fn on_map(conn: &xcb::Connection, req: x::MapRequestEvent) -> xcb::Result<()> {
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

	conn.flush()?;
	Ok(())
}
