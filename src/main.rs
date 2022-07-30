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

/// Keeps track of the manipulation of a window. See
/// [WindowManipulation](window_manipulation::WindowManipulation) for more information.
mod window_manipulation;

/// This module provides an assortment of utility traits to ease interaction with [xcb].
pub mod extensions;

/// Contains utilities for sending X requests, especially for queries or initialization on windows.
pub mod util;

use tracing::{debug, info, trace};

use xcb::x;
use xcb::{Connection, Xid};

use crate::aquariwm::AquariWm;

/// The main entrypoint for AquariWM; `main` is responsible for initalization/setup.
fn main() -> xcb::Result<()> {
	// Initialize the default [`tracing`] subscriber so that all logged messages are printed to
	// the console. AquariWM values the heavy use of log messages throughout the codebase.
	tracing_subscriber::fmt()
		.pretty()
		.with_max_level(tracing::Level::TRACE)
		.init();

	// TODO: Learn more about the [tracing] library and how to use its features more usefully.

	info!("Starting AquariWM");

	let (conn, screen_id) = Connection::connect(None)?;
	debug!(screen = screen_id, "Established connection to the X server",);

	let root = conn
		.get_setup()
		.roots()
		.nth(screen_id as usize)
		.unwrap()
		.root();

	// Send a request for substructure redirection on the root window (required for the window
	// manager to function, only one client can have substructure redirection at once).
	debug!(
		window = root.resource_id(),
		"Registering for events on root window"
	);
	conn.send_and_check_request(&x::ChangeWindowAttributes {
		window: root,
		value_list: &[x::Cw::EventMask(
			x::EventMask::SUBSTRUCTURE_REDIRECT | x::EventMask::SUBSTRUCTURE_NOTIFY,
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
	trace!(
		window = root.resource_id(),
		"Sending a query for the current window tree"
	);
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
				util::init_window(&conn, window);
			}
		});

	// After [`init_window`] initializes all the windows (which involves sending requests), we
	// flush the connection to send all of those queued requests at once.
	conn.flush()?;

	info!("Initialization complete");

	// It is now time to finalize the initialization of AquariWM by instantiating the main window
	// manager.
	AquariWm::start(conn)?;
	Ok(())
}
