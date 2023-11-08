// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::{env, fmt::Debug, future::Future, io, thread};

use futures::future;
use thiserror::Error;
use tracing::{event, Level};
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
	state,
};

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

#[cfg(feature = "testing")]
mod testing {
	use std::{process, sync::mpsc, time::Duration};

	use winit::{
		event::{Event as WinitEvent, WindowEvent as WinitWindowEvent},
		event_loop::EventLoopBuilder as WinitEventLoopBuilder,
		platform::x11::EventLoopBuilderExtX11,
		window::WindowBuilder as WinitWindowBuilder,
	};

	use super::*;

	pub struct Xephyr(pub process::Child);

	impl Drop for Xephyr {
		fn drop(&mut self) {
			let Self(child) = self;

			child.kill().expect("Failed to kill Xephyr");
		}
	}

	impl Xephyr {
		pub fn spawn() -> io::Result<Self> {
			const TESTING_DISPLAY: &str = ":1";

			let (transmitter, receiver) = mpsc::channel();

			// Create and run a `winit` window for `Xephyr` to use in another thread so it doesn't block the
			// main thread.
			// TODO: use tokio for this instead!
			thread::spawn(move || {
				event!(Level::DEBUG, "Initialising winit window");

				let event_loop = WinitEventLoopBuilder::new().with_any_thread(true).build().unwrap();
				let window = WinitWindowBuilder::new()
					.with_title(X11::title())
					.build(&event_loop)
					.unwrap();

				// Send the window's window ID back to the main thread so it can be supplied to `Xephyr`.
				transmitter.send(u64::from(window.id())).unwrap();

				event_loop
					.run(move |event, target| {
						if let WinitEvent::WindowEvent {
							event: WinitWindowEvent::CloseRequested,
							..
						} = event
						{
							target.exit()
						}
					})
					.unwrap();
			});
			let window_id = receiver.recv().unwrap();

			event!(Level::DEBUG, "Initialising Xephyr");
			match process::Command::new("Xephyr")
				.arg("-resizeable")
				// Run `Xephyr` in the `winit` window.
				.args(["-parent", &window_id.to_string()])
				.arg(TESTING_DISPLAY)
				.spawn()
			{
				Ok(process) => {
					// Set the `DISPLAY` env variable to `TESTING_DISPLAY`.
					env::set_var("DISPLAY", TESTING_DISPLAY);

					// Sleep for 1s to wait for Xephyr to launch. Not ideal.
					thread::sleep(Duration::from_secs(1));

					Ok(Self(process))
				},

				Err(error) => {
					event!(Level::ERROR, "Error while attempting to initialise Xephyr: {error}");

					Err(error)
				},
			}
		}
	}
}

// TODO: remove
pub struct X11 {
	// pub connection: util::WindowManager,
	pub state: state::AquariWm<x11::Window>,
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

impl AsyncDisplayServer for X11 {
	type Error = Error;
}

impl DisplayServer for X11 {
	type Output = impl Future<Output = Result<(), Error>>;
	const NAME: &'static str = "X11";

	fn run(testing: bool) -> Self::Output {
		async move {
			// Spawn Xephyr - a nested X server - if `testing` is enabled so AquariWM runs in a testing
			// window. Keep it in scope so it can be killed when it is dropped.
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

			// Register for SUBSTRUCTURE_NOTIFY and SUBSTRUCTURE_REDIRECT events on the root window; this means
			// registering as a window manager.
			let register_event_masks = connection
				.change_window_attributes(
					root,
					&Attributes::new().event_mask(EventMask::SUBSTRUCTURE_NOTIFY | EventMask::SUBSTRUCTURE_REDIRECT),
				)
				.await?;
			// Check if we managed to register as a window manager.
			match register_event_masks.check().await {
				Ok(_) => event!(Level::INFO, "Successfully registered window manager"),

				// If we failed to register the window manager, exit AquariWM.
				Err(error) => {
					event!(
						Level::ERROR,
						"Failed to register AquariWM as a window manager; another window manager is probably already \
						 running:\n{}",
						error
					);

					return Err(error.into());
				},
			}

			let mut state = state::AquariWm::with_windows(Self::query_windows(&connection, root).await?);

			if testing {
				event!(Level::INFO, "Testing mode enabled");

				// Attempt to launch a terminal.
				match crate::launch_terminal() {
					Ok((name, _)) => event!(Level::INFO, "Launched {name:?}"),
					Err(error) => event!(Level::WARN, "Failed to launch terminal: {error}"),
				}
			}

			loop {
				// Wait for the next event.
				let event = connection.wait_for_event().await?;

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
						// TODO: place tiled windows in tiling layout

						connection.map_window(window);
						state.map_window(&window);
					},

					// If a client requests to configure its window, honor it. For a tiling layout, this
					// should modify the configure request to place it in the tiling layout.
					Event::ConfigureRequest(_request) => {
						// TODO
					},

					// If a client requests to raise or lower its window, honor it. For a tiling layout,
					// this should be rejected for tiled windows, as they should always be at the bottom
					// of the stack.
					Event::CirculateRequest(_request) => {
						// TODO
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
	/// Queries the children of the `root` window and their [map states].
	///
	/// [map states]: state::MapState
	async fn query_windows(
		connection: &RustConnection,
		root: x11::Window,
	) -> Result<Vec<(x11::Window, state::MapState)>> {
		let windows = connection.query_tree(root).await?.reply().await?.children;

		// Send GetWindowAttributes requests for each window.
		let cookies =
			future::try_join_all(windows.iter().map(|&window| connection.get_window_attributes(window))).await?;
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
