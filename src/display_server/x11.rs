// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::{env, fmt::Debug, future::Future, io, thread};

use futures::future;
use thiserror::Error;
use tracing::{event, span, Level};
use x11rb_async::{
	self as x11rb,
	connection::Connection,
	protocol::{
		xproto::{
			self as x11,
			ChangeWindowAttributesAux as Attributes,
			ConnectionExt,
			CreateNotifyEvent as CreateNotify,
			DestroyNotifyEvent as DestroyNotify,
			EventMask,
			MapRequestEvent as MapRequest,
			UnmapNotifyEvent as UnmapNotify,
		},
		Event,
	},
	rust_connection::RustConnection,
};

use crate::{
	display_server::{AsyncDisplayServer, DisplayServer},
	layout,
	state,
};

#[cfg(feature = "testing")]
mod testing;
mod util;

#[derive(Debug, thiserror::Error)]
pub enum Error {
	/// An error attempting to connect to the X server.
	#[error(transparent)]
	Connect(#[from] x11rb::errors::ConnectError),
	/// An error occurring from an active connection to an X server.
	#[error(transparent)]
	Connection(#[from] x11rb::errors::ConnectionError),
	/// An error in a request's reply.
	#[error(transparent)]
	Reply(#[from] x11rb::errors::ReplyError),

	/// There was an error parsing a [`x11::MapState`].
	#[error("There was an error attempting to parse a MapState: {0}")]
	MapStateParseError(#[from] ParseError<u8>),

	#[error(transparent)]
	Io(#[from] io::Error),
}

pub type Result<T, Err = Error> = std::result::Result<T, Err>;
pub type ConnResult<T> = std::result::Result<T, x11rb::errors::ConnectionError>;

pub struct X11 {
	/// The connection to the X server.
	pub conn: RustConnection,
	/// The root window for the screen.
	pub root: x11::Window,
}

impl AsyncDisplayServer for X11 {
	type Error = Error;
}

impl DisplayServer for X11 {
	type Output = impl Future<Output = Result<(), Error>>;
	const NAME: &'static str = "X11";

	fn run(testing: bool) -> Self::Output {
		async move {
			let init_span = span!(Level::INFO, "Initialisation").entered();

			// Spawn Xephyr - a nested X server - if `testing` is enabled so AquariWM runs in a testing
			// window. Keep it in scope so it can be killed when it is dropped.
			#[cfg(feature = "testing")]
			let _process = testing.then_some(testing::Xephyr::spawn()?);

			// Connect to the X server on the display specified by the `DISPLAY` env variable.
			let (connection, screen_num, drive) = RustConnection::connect(None).await?;

			// Spawn a task that reads from the connection.
			tokio::spawn(async move {
				if let Err(error) = drive.await {
					event!(Level::ERROR, "Error while driving the X11 connection: {}", error);
				}
			});

			// Get the setup and, with it, the screen.
			let setup = connection.setup();
			let screen = &setup.roots[screen_num];
			// Get the root window of the screen.
			let root = screen.root;

			// Wrap the connection to provide easy access to utility methods.
			let wm = Self { conn: connection, root };

			// Attempt to register as a window manager.
			match wm.register_window_manager().await {
				Ok(_) => event!(Level::INFO, "Successfully registered window manager"),

				// If we failed to register the window manager, exit AquariWM.
				Err(error) => {
					event!(
						Level::ERROR,
						"Failed to register AquariWM as a window manager; another window manager is probably already \
						 running:\n{}",
						error
					);

					return Err(error);
				},
			}

			let mut state = state::AquariWm::with_windows(wm.query_windows().await?);

			if testing {
				event!(Level::INFO, "Testing mode enabled");

				// Attempt to launch a terminal.
				match crate::launch_terminal() {
					Ok((name, _)) => event!(Level::INFO, "Launched {name:?}"),
					Err(error) => event!(Level::WARN, "Failed to launch terminal: {error}"),
				}
			}

			init_span.exit();
			let event_loop_span = span!(Level::DEBUG, "Event loop");

			loop {
				let _span = event_loop_span.enter();

				// Flush the requests of the previous iteration, if there are any to flush.
				wm.conn.flush().await?;

				// Wait for the next event.
				let event = wm.conn.wait_for_event().await?;
				event!(Level::TRACE, "{:?}", event);

				match event {
					// Track the state of newly created windows.
					Event::CreateNotify(CreateNotify { window, .. }) => {
						state.add_window(window, state::MapState::Unmapped);
					},
					// Stop tracking the state of destroyed windows.
					Event::DestroyNotify(DestroyNotify { window, .. }) => {
						state.remove_window(&window);
					},

					// If a client requests to map its window, map it.
					Event::MapRequest(MapRequest { window, .. }) => {
						wm.conn.map_window(window);
						state.map_window(&window);
					},

					// If a client requests to configure its window, honor it. For a tiling layout, this
					// should modify the configure request to place it in the tiling layout.
					Event::ConfigureRequest(request) => {
						wm.honor_configure_window(&request).await?;
					},

					// If a client requests to raise or lower its window, honor it. For a tiling layout,
					// this should be rejected for tiled windows, as they should always be at the bottom
					// of the stack.
					Event::CirculateRequest(request) => {
						wm.circulate_window(&state, request.window, request.place).await?;
					},
					Event::UnmapNotify(UnmapNotify { window, .. }) => {
						state.unmap_window(&window);
					},

					_ => (),
				}
			}
		}
	}
}

impl X11 {
	/// Circulates the given [floating] `window` in the given `direction`.
	///
	/// This function has no effect if the given `window` is not [floating].
	///
	/// [floating]: layout::Mode::Floating
	pub async fn circulate_window<Direction>(
		&self,
		state: &state::AquariWm<x11::Window>,
		window: x11::Window,
		direction: Direction,
	) -> Result<()>
	where
		Direction: TryInto<CirculateDirection>,
		Direction::Error: Into<Error>,
	{
		// If it is a floating window...
		if let Some(state::WindowState {
			mode: layout::Mode::Floating,
			..
		}) = state.windows.get(&window)
		{
			let direction: CirculateDirection = direction.try_into().map_err(|error| error.into())?;

			self.conn.circulate_window(direction.into(), window).await?;

			// If we're moving the window to the bottom, then we have to move all the tiling windows
			// below it.
			if direction == CirculateDirection::MoveToBottom {
				let tiled_windows = state
					.windows
					.iter()
					.filter_map(|(window, state::WindowState { mode, .. })| match mode {
						layout::Mode::Tiled => Some(window),

						layout::Mode::Floating => None,
					});

				// Move each tiled window to the bottom of the window stack.
				future::try_join_all(tiled_windows.map(|&window| {
					self.conn
						.circulate_window(CirculateDirection::MoveToBottom.into(), window)
				}))
				.await?;
			}
		}

		Ok(())
	}

	/// Honors a [configure window request] without modifying it.
	///
	/// [configure window request]: x11::ConfigureRequestEvent
	pub async fn honor_configure_window(&self, request: &x11::ConfigureRequestEvent) -> Result<()> {
		self.conn
			.configure_window(request.window, &util::ConfigureValues::from(request).into())
			.await?;

		Ok(())
	}

	/// Registers for the `SUBSTRUCTURE_NOTIFY` and `SUBSTRUCTURE_REDIRECT` event masks on the root
	/// window; that is, register as a window manager.
	async fn register_window_manager(&self) -> Result<()> {
		let register_event_masks = self
			.conn
			.change_window_attributes(
				self.root,
				&Attributes::new().event_mask(EventMask::SUBSTRUCTURE_NOTIFY | EventMask::SUBSTRUCTURE_REDIRECT),
			)
			.await?;

		register_event_masks.check().await?;
		Ok(())
	}

	/// Queries the children of the `root` window and their [map states].
	///
	/// [map states]: state::MapState
	async fn query_windows(&self) -> Result<Vec<(x11::Window, state::MapState)>> {
		let windows = self.conn.query_tree(self.root).await?.reply().await?.children;

		// Send GetWindowAttributes requests for each window.
		let cookies =
			future::try_join_all(windows.iter().map(|&window| self.conn.get_window_attributes(window))).await?;
		let replies = future::try_join_all(cookies.into_iter().map(|cookie| cookie.reply())).await?;

		let map_states = replies.into_iter().map(|reply| reply.map_state.try_into());

		// Zip windows up with their map states.
		Ok(windows
			.into_iter()
			.zip(map_states)
			.map(|(window, map_state)| map_state.map(|map_state| (window, map_state)))
			.try_collect()?)
	}
}

#[derive(Debug, Error)]
#[error("Failed to parse {value}")]
pub struct ParseError<T: Debug> {
	pub value: T,
}

impl TryFrom<x11::MapState> for state::MapState {
	type Error = ParseError<u8>;

	fn try_from(state: x11::MapState) -> std::result::Result<Self, Self::Error> {
		match state {
			x11::MapState::VIEWABLE | x11::MapState::UNVIEWABLE => Ok(state::MapState::Mapped),
			x11::MapState::UNMAPPED => Ok(state::MapState::Unmapped),

			other => Err(ParseError { value: u8::from(other) }),
		}
	}
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum CirculateDirection {
	MoveToBottom,
	MoveToTop,
}

impl TryFrom<x11::Circulate> for CirculateDirection {
	type Error = ParseError<u8>;

	fn try_from(direction: x11::Circulate) -> Result<Self, Self::Error> {
		match direction {
			x11::Circulate::LOWER_HIGHEST => Ok(Self::MoveToBottom),
			x11::Circulate::RAISE_LOWEST => Ok(Self::MoveToTop),

			other => Err(ParseError { value: other.into() }),
		}
	}
}

impl TryFrom<x11::Place> for CirculateDirection {
	type Error = ParseError<u8>;

	fn try_from(direction: x11::Place) -> Result<Self, Self::Error> {
		match direction {
			x11::Place::ON_BOTTOM => Ok(Self::MoveToBottom),
			x11::Place::ON_TOP => Ok(Self::MoveToTop),

			other => Err(ParseError { value: other.into() }),
		}
	}
}

impl From<CirculateDirection> for x11::Circulate {
	fn from(direction: CirculateDirection) -> Self {
		match direction {
			CirculateDirection::MoveToBottom => Self::LOWER_HIGHEST,
			CirculateDirection::MoveToTop => Self::RAISE_LOWEST,
		}
	}
}
