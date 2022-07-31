// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! AquariWM is an ICCCM- and EWMH-compliant window manager for the X Window System. It focuses on
//! providing a modular approach to window management, allowing different functionality to be
//! 'mixed and matched' according to a user's wishes. It supports this with a large effort to
//! provide clear and extensive documentation and guides for users, contributors and clientmakers
//! alike, and by maintaining a friendly and welcoming community that encourages participation.
//!
//! # TODO
//! To-do list follows. This section is simply a convenient place to keep track of upcoming changes
//! that need to be made to AquariWM. It is only temporary and must be removed when the project
//! nears any kind of stability.
//!
//! ## [ICCCM](https://x.org/releases/X11R7.6/doc/xorg-docs/specs/ICCCM/icccm.html) compliance
//! AquariWM needs to be compliant with the ICCCM. This means placing certain atoms on various
//! windows and reading certain atoms from various windows to communicate some basic window
//! manager information with clients. For example, `WM_STATE` should be set on all windows
//! managed by the window manager.
//!
//! ## [EWMH](https://specifications.freedesktop.org/wm-spec/latest) compliance
//! AquariWM also needs to be compliant with the Extended Window Manager Hints specification. This
//! is a newer specification for window managers introduced in the early 2000s to build upon the
//! ICCCM, but does not replace it (both specifications must be adhered to). It is _slightly_ more
//! involved; for example, the window manager must make its own window, child of root, and set
//! certain atoms on that window to communicate information about the window manager and to
//! demonstrate active compliance. The EWMH contains a number of 'higher-level' concepts used in
//! window managers, such as the concept of 'virtual desktops', also known as 'workspaces', among
//! other things.
//!
//! # Modularity
//! AquariWM's core feature is its modularity. To understand how it achieves this, you must first
//! understand the way in which the X Window System works.
//!
//! ### X Window System
//! The X window system is centered around a client-server model, as you may be used to using for
//! the internet. Each graphical application that runs on your computer under X is known as an _X
//! client_. When your computer is running an _X server_, these clients can communicate with the
//! server, sending various messages and requests, and receiving various notifications and replies
//! back.
//!
//! A _window manager_ fills a role somewhere in between these two sides: it requests that
//! certain requests made by clients are redirected to itself, rather than to the X server. A
//! window manager can send and receive messages to and from X clients, and it can decide which
//! messages to allow, modify, or deny, before sending its own messages to the X server.
//!
//! ### AquariWM clients
//! In very much a similar way to how X clients communicate with the X server (or any window
//! manager that may be intercepting these messages), AquariWM allows its modularity by defining
//! a protocol for X clients to communicate directly with AquariWM. AquariWM's functionality can be
//! split across multiple 'AquariWM clients', allowing an extreme level of control over how the
//! window manager operates.
//!
//! ## What does this allow AquariWM to do differently?
//! With a typical window manager, there is a limit to how much any particular functionality can be
//! customized without making direct changes to the window manager itself. In contrast, features
//! can be added or replaced in AquariWM without affecting any other part of the window manager.
//! Changing the way part of AquariWM works is as simple as launching a different AquariWM clients.

/// The central module of `aquariwm-server`. Responsible for overseeing AquariWM's state and
/// operation.
///
/// The `aquariwm` module can be thought of as the brain behind AquariWM. It keeps track of the
/// window manager's state, runs the event loop, and delegates tasks to other modules and
/// AquariWM clients.
mod aquariwm;

/// The `features` module contains various other modules for functionality of AquariWM.
///
/// These features are notably _not_ AquariWM clients in their own right, though they may act as
/// bridges to clients that relate to their functionality.
///
/// Features within the `features` module may be re-exported from private modules with `pub use` to
/// simplify other `use` statements, or, if a feature contains many items that are intended to be
/// used in other modules, that feature may be exported as its entire module with `pub mod`.
mod features;

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
				util::init_window(&conn, *window);
			}
		});

	// After [`util::init_window`] initializes all the windows (which involves sending requests), we
	// flush the connection to send all of those queued requests at once.
	conn.flush()?;

	info!("Initialization complete");

	// It is now time to finalize the initialization of AquariWM by instantiating the main window
	// manager.
	AquariWm::start(conn)?;
	Ok(())
}
