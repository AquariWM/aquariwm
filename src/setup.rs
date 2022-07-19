use x::Window;
use xcb::{x, Connection};

/// Set up the window manager and register for various events with the X server.
pub fn init(conn: &Connection, screen_num: usize) -> xcb::Result<()> {
	// Get the relevant screen and root window from the connection object using the `screen_num`
	// provided by `xcb::Connection::connect`.
	let screen = conn.get_setup().roots().nth(screen_num).unwrap();
	let root = screen.root();

	// Request substructure redirection on the root window and send a descriptive error message if
	// the request fails.
	conn.check_request(conn.send_request_checked(&x::ChangeWindowAttributes {
		window: root,
		value_list: &[x::Cw::EventMask(
			x::EventMask::SUBSTRUCTURE_REDIRECT | x::EventMask::SUBSTRUCTURE_NOTIFY,
		)],
	}))
	.expect("Uh oh! Failed to start AquariWM because there is already a window manager running.");

	// Query the X server for the existing window tree so that we can perform set up on any
	// existing windows.
	let query = conn.send_request(&x::QueryTree { window: root });

	// Receive the results of the query and get the top level windows.
	let results = conn.wait_for_reply(query)?;
	let children = results.children();

	// Request window attributes for every direct child of the root window.
	let cookies = children
		.iter()
		.map(|child| conn.send_request(&x::GetWindowAttributes { window: *child }));

	// Receive the responses to the requests. This unnecessarily waits for every reply, but it
	// should avoid most of the overhead of the X server... has potential to be optimized if
	// needed. You'd want to asynchronously receive the replies for all of the cookies,
	// immediately moving on to registering events on windows as they get their respective
	// replies.
	let responses = cookies.map(|cookie| conn.wait_for_reply(cookie));

	// Pair the responses with their corresponding windows to make it easier to associate each
	// reply with its window.
	let pairs = children.iter().zip(responses);

	// Register to receive events on every viewable window by comparing the window attributes.
	pairs
		.filter(|(_, response)| response.is_ok())
		.for_each(|(child, response)| {
			if response.unwrap().map_state() == x::MapState::Viewable {
				register_mapped_window(conn, child)
					.expect("Failed to register events on existing visible window.");
			}
		});

	conn.flush()?;
	Ok(())
}

/// Set up a window with the window manager.
///
/// Registers to receive events for the given window from the X server. Used to set up windows,
/// whether that be when the window manager is first started or when a new window is created.
pub fn register_mapped_window(conn: &Connection, window: &Window) -> xcb::Result<()> {
	// Register for the window manager to receive events relating to the mouse pointer entering
	// the client window.
	conn.send_request(&x::GrabPointer {
		// Continue to let the window process events as usual.
		owner_events: true,
		// Grab pointer 'enter window' events for the client window.
		grab_window: *window,
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

	Ok(())
}
