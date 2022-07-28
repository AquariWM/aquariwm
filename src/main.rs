// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! AquariWM is an extensible, modular window manager for X11.
//!
//! Through its modular approach, AquariWM hopes to allow for in-depth customization and
//! configuration of the window manager and to make this accessible to as many people as possible.
//!
//! This modularity is achieved through a system of _components_. Each component is a separate
//! program, an isolated group of functionality that is responsible for a specific part of the
//! window manager. For example, a configuration component might be the program responsible for
//! configuring AquariWM to the user's preference. Doing this as a component allows the user to
//! choose whichever configuration method they prefer, meaning you can use what works best for
//! you.

/// The central module of AquariWM. Responsible for overseeing AquariWM's state and operation.
///
/// The `aquariwm` module can be thought of as the brain behind AquariWM. It keeps track of the
/// window manager's state, runs the event loop, and delegates tasks to other modules and
/// components.
mod aquariwm;

/// This module provides an assortment of utility traits to ease interaction with [xcb].
mod extensions;

use tracing::{debug, info, trace};

use xcb::x::{self, Window};
use xcb::{Connection, Xid};

use crate::aquariwm::AquariWm;

/// The main entrypoint for AquariWM; `main` is responsible for initalization/setup.
fn main() -> xcb::Result<()> {
	// Initialize the default [`tracing`] subscriber so that all logged messages are printed to
	// the console. AquariWM values the heavy use of log messages throughout the codebase. You
	// should use the different log levels as follows:
	//
	// - `Trace`
	//   You should use the `Trace` level often - it is the most verbose log level, and should
	//   be used for any small operation which could affect the running of the window manager.
	//   For example, you should generally make a `Trace` level message for every request sent to
	//   the X server.
	//
	// - `Debug`
	//   Use the `Debug` level for a more easily understood grouping of operations. For example,
	//   a `Debug` level message might say:
	//   > Registering for events on existing windows
	//
	// - `Info`
	//   The `Info` log level should be used relatively sparingly. It describes bigger steps in
	//   the operation of AquariWM, for example:
	//   > Initializing AquariWM
	//   or:
	//   > Loading configuration
	//
	// - `Warn`
	//   The `Warn` level should be used when actual functionality or features of the window
	//   manager are hindered or restricted because of a problem encountered, but that problem was
	//   not critical to the continued running of the window manager. It can also be used for
	//   alerting users of unwise or not-recommended configuration setups, even if they may wish
	//   to proceed with them nonetheless. That might particularly include the use of experimental
	//   features, for example.
	//
	// - `Error`
	//   Use the `Error` log level only for critical errors that can cause entire components or
	//   the window manager itself to no longer be able to continue to run properly.
	tracing_subscriber::fmt()
		.pretty()
		.with_max_level(tracing::Level::TRACE)
		.init();

	// TODO: Learn more about the [tracing] library and how to use its features more usefully.

	info!("Starting AquariWM");

	let (conn, screen_id) = Connection::connect(None)?;
	debug!(
		"Established connection to the X server on screen {}",
		screen_id
	);

	let root = conn
		.get_setup()
		.roots()
		.nth(screen_id as usize)
		.unwrap()
		.root();

	// Send a request for substructure redirection on the root window (required for the window
	// manager to function, only one client can have substructure redirection at once), among
	// other events.
	debug!("Registering for events on root window");
	conn.send_and_check_request(&x::ChangeWindowAttributes {
		window: root,
		value_list: &[x::Cw::EventMask(
			x::EventMask::SUBSTRUCTURE_REDIRECT
				| x::EventMask::SUBSTRUCTURE_NOTIFY
				| x::EventMask::BUTTON_PRESS
				| x::EventMask::BUTTON_RELEASE
				| x::EventMask::BUTTON_MOTION,
		)],
	})
	.expect("Uh oh! Couldn't start AquariWM because there was already a window manager running");

	// We send a query for the current window tree, so that we can register for events on all
	// existing windows. We wait until after the events are registered on the root window; we know
	// that once those events are registered, all window updates will be sent to the window
	// manager. Since the event loop hasn't started yet, we know that the window tree cannot have
	// changed between sending the query and receiving a reply.
	//
	// Well, it could have, but the relevant events that will trigger the setup for those windows
	// won't have been processed yet, and that's what actually matters.
	trace!("Sending a query for the current window tree");
	let query = conn.send_request(&x::QueryTree { window: root });

	// We receive a reply to the query. It isn't actually necessary for us to receive this reply
	// before starting the event loop, as far as I can tell, so it would be better to poll for the
	// reply in the event loop, but that's a little complicated and the overhead of one request
	// should be very little.
	let reply = conn.wait_for_reply(query)?;
	let windows = reply.children();

	// We send a new query, for each existing window, asking for their window attributes. Since we
	// want to send all the queries at one time and then receive all their results later, we want
	// to store the cookies returned - therefore, it is easy to do this with the `map` function.
	trace!("Sending queries for the window attributes of all existing windows");
	let cookies = windows
		.iter()
		.map(|window| conn.send_request(&x::GetWindowAttributes { window: *window }));

	// No need to flush the connection, as [Connection::wait_for_reply] will do that for us.

	// Similarly, we use `map` to wait for all the replies. Since we know that the replies will be
	// in order, as we are using `map` both times, we can simply use `zip` to match the replies
	// with their windows. We then initialize those windows that are mapped.
	debug!("Initializing existing windows");
	cookies
		.map(|cookie| conn.wait_for_reply(cookie))
		.zip(windows)
		.for_each(|(reply, window)| {
			if reply.is_ok() && reply.unwrap().map_state() == x::MapState::Viewable {
				init_window(&conn, window).ok();
			}
		});

	// After [`init_window`] initializes all the windows (which involves sending requests), we
	// flush the connection to send all of those queued requests at once.
	conn.flush()?;

	info!("Initialization complete");

	// It is now time to finalize the initialization of AquariWM by instantiating the main window
	// manager.
	let wm = AquariWm::new(conn, root);
	// TODO: Explore possible options for making this cleaner/more idiomatic. Ideally,
	//       instantiating the window manager and running the event loop would be the same
	//       function call.
	wm.run()
}

/// Initializes the given [window](x::Window) by requesting to receive certain events on it.
///
/// Requests to receive the following events:
/// - [`ENTER_WINDOW`](x::EventMask::ENTER_WINDOW)
/// - [`FOCUS_CHANGE`](x::EventMask::FOCUS_CHANGE)
fn init_window(conn: &Connection, window: &Window) -> xcb::Result<()> {
	trace!("Initializing window ({})", window.resource_id());
	conn.send_request(&x::ChangeWindowAttributes {
		window: *window,
		value_list: &[x::Cw::EventMask(
			x::EventMask::ENTER_WINDOW | x::EventMask::FOCUS_CHANGE,
		)],
	});

	Ok(())
}
