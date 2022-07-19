use x::Window;
use xcb::{x, Connection};

/// Set up the window manager and register for various events with the X server.
pub fn setup(conn: &Connection, screen_num: usize) -> xcb::Result<()> {
	// Get the relevant screen and root window from the connection object using the `screen_num`
	// provided by `xcb::Connection::connect`.
	let screen = conn.get_setup().roots().nth(screen_num).unwrap();
	let root = screen.root();

	// Request substructure redirection on the root window.
	// TODO: error handling for when a window manager is already running
	conn.send_request(&x::ChangeWindowAttributes {
		window: root,
		value_list: &[x::Cw::EventMask(
			x::EventMask::SUBSTRUCTURE_REDIRECT | x::EventMask::SUBSTRUCTURE_NOTIFY,
		)],
	});

	// Query the X server for the existing window tree so that we can perform set up on any
	// existing windows.
	let cookie = conn.send_request(&x::QueryTree { window: root });

	// Flush the queued requests to the X server.
	conn.flush()?;

	// Receive the results of the query and get the top level windows.
	let query = conn.wait_for_reply(cookie)?;
	let top_level_windows = query.children();

	// Register each window.
	for window in top_level_windows {
		register_window(conn, *window)?;
	}

	Ok(())
}

/// Set up a window with the window manager.
///
/// Registers to receive events for the given window from the X server. Used to set up windows,
/// whether that be when the window manager is first started or when a new window is created.
pub fn register_window(conn: &Connection, window: Window) -> xcb::Result<()> {
	// Register for the window manager to receive events relating to the mouse pointer entering
	// the client window.
	conn.send_request(&x::GrabPointer {
		// Continue to let the window process events as usual.
		owner_events: true,
		// Grab pointer 'enter window' events for the client window.
		grab_window: window,
		event_mask: x::EventMask::ENTER_WINDOW,
		// Don't freeze pointer and keyboard input events.
		pointer_mode: x::GrabMode::Async,
		keyboard_mode: x::GrabMode::Async,
		// Don't restrict the cursor to any particular window.
		confine_to: x::WINDOW_NONE,
		// Don't modify the appearance of the cursor.
		cursor: x::CURSOR_NONE,
		time: x::CURRENT_TIME,
	});

	conn.flush()?;
	Ok(())
}
