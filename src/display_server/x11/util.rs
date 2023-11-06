// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::{
	future::Future,
	marker::PhantomData,
	pin::Pin,
	sync::{Arc, Mutex},
	task::{Context, Poll, Waker},
};

use futures::future::BoxFuture;
use tracing::{event, span, Level};
use x11::Cw as WindowAttribute;
use xcb::{x as x11, x::EventMask};

use super::X11;
use crate::{layout, state};

struct SharedState {
	pub waker: Option<Waker>,
	pub completed: bool,
}

pub struct WindowManager {
	state: state::AquariWm<x11::Window>,
	connection: Connection,
}

pub struct Connection {
	screen_num: i32,
	root: x11::Window,

	xcb_conn: xcb::Connection,
	cookies: Mutex<Vec<Arc<Mutex<SharedState>>>>,
}

impl WindowManager {
	pub fn state(&mut self) -> &mut state::AquariWm<x11::Window> {
		&mut self.state
	}
}

impl Connection {
	/// Sends a `request` that has a reply.
	///
	/// This function returns a [future]. [Futures](future) are lazy: calling this function won't do
	/// anything unless it is `await`ed.
	///
	/// When sending multiple requests and `await`ing their replies, consider using
	/// [`future::try_join_all`]:
	/// ```no_run
	/// # use aquariwm::x11::util;
	/// # use futures::future;
	/// # use xcb::x as x11;
	/// #
	/// # #[tokio::main]
	/// # async fn main() -> xcb::Result<()> {
	/// #     let connection = util::Connection::connect(None)?;
	/// #
	/// #     let setup = connection.get_setup();
	/// #     let screen = setup.roots().nth(connection.screen_num as usize).unwrap();
	/// #     let root = screen.root();
	/// #
	/// // Query the window tree.
	/// let tree = connection
	///     .send_request_with_reply(&x11:QueryTree { window: root })
	///     .await?;
	/// // Get the top-level windows.
	/// let windows: Vec<_> = tree.children.iter().copied().collect();
	///
	/// // Use `try_join_all` to await all the window attributes in no particular order.
	/// let replies = future::try_join_all(
	///     windows
	///         .into_iter()
	///         .map(|window| connection.send_request_with_reply(
	///             &x11::GetWindowAttributes { window },
	///         ))
	/// )
	/// .await?;
	/// # }
	/// ```
	///
	/// [future]: Future
	pub fn send_request_with_reply<'conn, Req: xcb::RequestWithReply<Cookie = <Req as xcb::Request>::Cookie>>(
		&'conn mut self,
		request: &'conn Req,
	) -> impl Future<Output = xcb::Result<<<Req as xcb::RequestWithReply>::Cookie as xcb::CookieWithReplyChecked>::Reply>>
	       + 'conn
	where
		<Req as xcb::Request>::Cookie: xcb::CookieWithReplyChecked,
	{
		CookieCheckedFuture::new(self, self.xcb_conn.send_request(request))
	}

	/// Sends a `request` that has a reply, without checking for errors.
	///
	/// Errors are instead sent to the event loop.
	///
	/// This function returns a [future]. [Futures](future) are lazy: calling this function won't do
	/// anything unless it is `await`ed.
	///
	/// For more information, see [`send_request_with_reply`].
	///
	/// [future]: Future
	/// [`send_request_with_reply`]: Self::send_request_with_reply
	pub fn send_request_with_reply_unchecked<
		'conn,
		Req: xcb::RequestWithReply<CookieUnchecked = <Req as xcb::Request>::Cookie>,
	>(
		&'conn mut self,
		request: &'conn Req,
	) -> impl Future<
		Output = xcb::ConnResult<
			Option<<<Req as xcb::RequestWithReply>::CookieUnchecked as xcb::CookieWithReplyUnchecked>::Reply>,
		>,
	> + 'conn
	where
		<Req as xcb::Request>::Cookie: xcb::CookieWithReplyUnchecked + 'conn,
	{
		CookieUncheckedFuture::new(self, self.xcb_conn.send_request(request))
	}

	/// Gets setup information from the X server.
	///
	/// See [`xcb::Connection::get_setup`] for more information.
	pub fn get_setup(&self) -> &x11::Setup {
		self.xcb_conn.get_setup()
	}

	/// Sends a `request` that has no reply, without checking for errors.
	pub fn send_request<Req: xcb::RequestWithoutReply>(&mut self, request: &Req) {
		self.xcb_conn.send_request(request);
	}

	/// Sends a `request` that has no reply, checking for errors.
	///
	/// Unfortunately, checking for errors for a request with no reply is a blocking operation, so
	/// this function cannot return a [future].
	///
	/// Errors can be checked using [`check_request`] with the cookie returned from this function.
	///
	/// [future]: Future
	/// [`check_request`]: Self::check_request
	pub fn send_request_checked<Req: xcb::RequestWithoutReply>(&mut self, request: &Req) -> xcb::VoidCookieChecked {
		self.xcb_conn.send_request_checked(request)
	}

	/// Checks for errors returned by a request that has no reply.
	///
	/// The `cookie` is the one returned by [`send_request_checked`].
	///
	/// This function is blocking; it does not return until the errors have been received, or XCB
	/// can be sure that no errors are going to be received. Unfortunately, this is no non-blocking
	/// equivalent.
	///
	/// [`send_request_checked`]: Self::send_request_checked
	pub fn check_request(&mut self, cookie: xcb::VoidCookieChecked) -> xcb::ProtocolResult<()> {
		self.xcb_conn.check_request(cookie)
	}

	/// Moves the given `window` to the [bottom] or [top] of the window stack if it is [floating].
	///
	/// [Tiled] windows will not be circulated.
	///
	/// If the window is moved to the [bottom] of the stack, then all [tiled] windows will then be
	/// moved to the [bottom] so that [tiled] windows are always at the [bottom] of the stack.
	///
	/// [bottom]: x11::Circulate::LowerHighest
	/// [top]: x11::Circulate::RaiseLowest
	///
	/// [floating]: layout::Mode::Floating
	/// [tiled]: layout::Mode::Tiled
	/// [Tiled]: layout::Mode::Tiled
	pub fn circulate_window(
		&mut self,
		state: &state::AquariWm<x11::Window>,
		window: x11::Window,
		direction: x11::Circulate,
	) {
		// Only circulate the window if it is floating.
		if let Some(state::WindowState {
			mode: layout::Mode::Floating,
			..
		}) = state.windows.get(&window)
		{
			// Circulate the window.
			self.xcb_conn.send_request(&x11::CirculateWindow { window, direction });

			// If the window was lowered to the bottom, then lower all the tiled windows below it again.
			if direction == x11::Circulate::LowerHighest {
				let tiled_windows = state.windows.iter().filter_map(|(window, state)| match state.mode {
					layout::Mode::Tiled => Some(window),

					layout::Mode::Floating => None,
				});

				// Move each tiled window to the bottom.
				for &window in tiled_windows {
					self.xcb_conn.send_request(&x11::CirculateWindow {
						window,
						direction: x11::Circulate::LowerHighest,
					});
				}
			}
		}
	}
}

// TODO: restructure for having `Connection` be separate to `WindowManager`.
impl WindowManager {
	/// Initiates a connection to the X server and initialises AquariWM state.
	///
	/// If `display_name` is specified, the connection is made to that display. Otherwise, the
	/// display specified by the `DISPLAY` environment variable is used.
	pub fn new(display_name: Option<&str>) -> xcb::Result<Self> {
		let _init_span = span!(Level::INFO, "Initialisation").entered();

		// Connect to the X server.
		let (xcb_conn, screen_num) = xcb::Connection::connect(display_name)?;

		// Get the setup and, with it, the screen.
		let setup = xcb_conn.get_setup();
		let screen = setup.roots().nth(screen_num as usize).unwrap();

		// Get the root window of the screen.
		let root = screen.root();

		// Register for SUBSTRUCTURE_NOTIFY and SUBSTRUCTURE_REDIRECT events on the root window; this
		// means registering as a window manager.
		let result = xcb_conn.send_and_check_request(&x11::ChangeWindowAttributes {
			window: root,
			value_list: &[WindowAttribute::EventMask(
				EventMask::SUBSTRUCTURE_NOTIFY | EventMask::SUBSTRUCTURE_REDIRECT,
			)],
		});
		match result {
			Ok(_) => event!(Level::INFO, "Successfully registered window manager"),

			// If we failed to register the window manager, exit AquariWM.
			Err(error) => {
				event!(
					Level::ERROR,
					"Failed to register a window manager; a window manager is probably already running!",
				);

				return Err(error.into());
			},
		}

		// Unfortunately, we can't join these replies using async because until the event loop,
		// there is nothing to poll them.
		let cookie = xcb_conn.send_request(&x11::QueryTree { window: root });
		let windows: Vec<_> = xcb_conn.wait_for_reply(cookie)?.children().iter().copied().collect();

		let cookies: Vec<_> = windows
			.iter()
			.map(|&window| xcb_conn.send_request(&x11::GetWindowAttributes { window }))
			.collect();
		let replies: Vec<_> = cookies
			.into_iter()
			.map(|cookie| xcb_conn.wait_for_reply(cookie))
			.try_collect()?;

		let windows = windows
			.into_iter()
			.zip(replies.into_iter().map(|reply| reply.map_state().into()));

		Ok(Self {
			state: state::AquariWm::with_windows(windows),
			connection: Connection {
				screen_num,
				root,

				xcb_conn,
				cookies: Mutex::new(Vec::new()),
			},
		})
	}

	/// Runs the connection's event loop.
	///
	/// This function polls all yet-to-be-received replies and calls the given `event_loop` when new
	/// [events] are received.
	///
	/// [`Hack`] must be used surrounding an `async` block in the body of the `event_loop` closure
	/// due to Rust limitations.
	///
	/// # Examples
	/// ```no_run
	/// # use aquariwm::display_server::x11::util;
	/// # use xcb::x as x11;
	/// #
	/// # #[tokio::main]
	/// # async fn main() -> xcb::Result<()> {
	/// #     let connection = util::Connection::new(None)?;
	/// #
	/// connection.run(|connection, state, event| {
	///     util::Hack::new(async {
	///         match event {
	///             xcb::Event(x11::Event::CreateNotify(notification)) => {
	///
	///         }
	///     })
	/// })
	/// .await?;
	/// # }
	/// ```
	///
	/// [events]: xcb::Event
	pub async fn run<'event_loop, EventLoop: 'event_loop>(&mut self, event_loop: EventLoop) -> xcb::Result<()>
	where
		EventLoop: Unpin,
		EventLoop: for<'conn> FnMut(
			&'conn mut Connection,
			&'conn mut state::AquariWm<x11::Window>,
			xcb::Event,
		) -> Hack<'conn, 'event_loop, xcb::Result<()>>,
	{
		RunFuture::new(self, event_loop).await
	}
}

/// A wrapper around a [`BoxFuture`] required for an async closure with `&mut` parameters.
///
/// See [Using async closures with mut ref][hack] for more information.
///
/// [hack]: https://users.rust-lang.org/t/using-async-closures-with-mut-ref/47240/2
pub struct Hack<'short, 'long: 'short, Ret = ()>(BoxFuture<'short, Ret>, PhantomData<&'long ()>);

impl<'short, 'long: 'short, Ret> Hack<'short, 'long, Ret> {
	/// Creates a new `Hack` wrapper around the given `future`.
	pub fn new(future: impl Future<Output = Ret> + Send + 'short) -> Self {
		Self(Box::pin(future), PhantomData)
	}

	/// Returns the wrapped [`BoxFuture`].
	pub fn unwrap(self) -> BoxFuture<'short, Ret> {
		self.0
	}
}

pub struct RunFuture<'window_manager, 'event_loop, EventLoop: 'event_loop>
where
	EventLoop: Unpin,
	EventLoop: for<'wm> FnMut(
		&'wm mut Connection,
		&'wm mut state::AquariWm<x11::Window>,
		xcb::Event,
	) -> Hack<'wm, 'event_loop, xcb::Result<()>>,
{
	window_manager: &'window_manager mut WindowManager,

	event_loop: EventLoop,
	event_loop_future: Option<Hack<'window_manager, 'event_loop, xcb::Result<()>>>,
}

impl<'window_manager, 'event_loop, EventLoop: 'event_loop> RunFuture<'window_manager, 'event_loop, EventLoop>
where
	EventLoop: Unpin,
	EventLoop: for<'wm> FnMut(
		&'wm mut Connection,
		&'wm mut state::AquariWm<x11::Window>,
		xcb::Event,
	) -> Hack<'wm, 'event_loop, xcb::Result<()>>,
{
	pub(self) fn new(window_manager: &'window_manager mut WindowManager, event_loop: EventLoop) -> Self {
		Self {
			window_manager,

			event_loop,
			event_loop_future: None,
		}
	}
}

impl<'window_manager, 'event_loop, EventLoop: 'event_loop> Future for RunFuture<'window_manager, 'event_loop, EventLoop>
where
	EventLoop: Unpin,
	EventLoop: for<'wm> FnMut(
		&'wm mut Connection,
		&'wm mut state::AquariWm<x11::Window>,
		xcb::Event,
	) -> Hack<'wm, 'event_loop, xcb::Result<()>>,
{
	type Output = xcb::Result<()>;

	fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
		// Flush requests from the previous iteration.
		self.window_manager.connection.xcb_conn.flush()?;

		// Introduce a new scope for to wake the cookies so that the mutex guard's shared reference
		// is dropped so that mutable references can be used later.
		{
			let cookie_states = &mut self
				.window_manager
				.connection
				.cookies
				.lock()
				.expect("failed to lock mutex");

			// Remove completed cookies.
			cookie_states.retain(|state| !state.lock().expect("failed to lock mutex").completed);

			// Wake every cookie.
			for state in cookie_states.iter() {
				if let Some(waker) = state.lock().expect("failed to lock mutex").waker.take() {
					waker.wake();
				}
			}
		}

		// If there is no ongoing `event_loop` future, call `event_loop` again.
		if self.event_loop_future.is_none() {
			if let Some(event) = self.window_manager.connection.xcb_conn.poll_for_event()? {
				let WindowManager { connection, state } = &mut self.window_manager;

				let hack = (self.event_loop)(connection, state, event);

				self.event_loop_future = Some(hack);
			}
		}
		// If there is an ongoing `event_loop` future, poll it.
		else if let Some(Hack(future, _)) = &mut self.event_loop_future {
			// If `event_loop` has returned, reset the future.
			if let Poll::Ready(result) = future.as_mut().poll(cx) {
				// Return an error if there was any.
				if let Err(error) = result {
					return Poll::Ready(Err(error));
				}

				// Reset the future so `event_loop` is called on the next iteration.
				self.event_loop_future = None;
			}
		}

		// Loop this function until an error occurs.
		Poll::Pending
	}
}

pub struct CookieCheckedFuture<'conn, Cookie>
where
	Cookie: xcb::CookieWithReplyChecked,
{
	cookie: Cookie,
	connection: &'conn xcb::Connection,
	shared_state: Arc<Mutex<SharedState>>,
}

pub struct CookieUncheckedFuture<'conn, Cookie>
where
	Cookie: xcb::CookieWithReplyUnchecked,
{
	cookie: Cookie,
	connection: &'conn xcb::Connection,
	shared_state: Arc<Mutex<SharedState>>,
}

impl<'conn, Cookie> Future for CookieCheckedFuture<'conn, Cookie>
where
	Cookie: xcb::CookieWithReplyChecked,
{
	type Output = xcb::Result<Cookie::Reply>;

	fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
		let poll = match self.connection.poll_for_reply(&self.cookie) {
			Ok(Some(reply)) => Poll::Ready(Ok(reply)),
			Ok(None) => Poll::Pending,

			Err(error) => Poll::Ready(Err(error)),
		};

		let shared_state = &mut self.shared_state.lock().expect("failed to lock mutex");
		match &poll {
			Poll::Pending => {
				shared_state.waker = Some(cx.waker().clone());
			},
			Poll::Ready(_) => {
				shared_state.completed = true;
			},
		}

		poll
	}
}

impl<'conn, Cookie> Future for CookieUncheckedFuture<'conn, Cookie>
where
	Cookie: xcb::CookieWithReplyUnchecked,
{
	type Output = xcb::ConnResult<Option<Cookie::Reply>>;

	fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
		let poll = match self.connection.poll_for_reply_unchecked(&self.cookie) {
			Ok(Some(Some(reply))) => Poll::Ready(Ok(Some(reply))),
			Ok(Some(None)) => Poll::Pending,

			Ok(None) => Poll::Ready(Ok(None)),
			Err(error) => Poll::Ready(Err(error)),
		};

		let shared_state = &mut self.shared_state.lock().expect("failed to lock mutex");
		match &poll {
			Poll::Pending => {
				shared_state.waker = Some(cx.waker().clone());
			},
			Poll::Ready(_) => {
				shared_state.completed = true;
			},
		}

		poll
	}
}

impl<'conn, Cookie> CookieCheckedFuture<'conn, Cookie>
where
	Cookie: xcb::CookieWithReplyChecked,
{
	pub(self) fn new(connection: &'conn Connection, cookie: Cookie) -> Self {
		let shared_state = Arc::new(Mutex::new(SharedState {
			waker: None,
			completed: false,
		}));
		connection.cookies.lock().unwrap().push(Arc::clone(&shared_state));

		Self {
			cookie,
			connection: &connection.xcb_conn,
			shared_state,
		}
	}
}

impl<'conn, Cookie> CookieUncheckedFuture<'conn, Cookie>
where
	Cookie: xcb::CookieWithReplyUnchecked,
{
	pub(self) fn new(connection: &'conn Connection, cookie: Cookie) -> Self {
		let shared_state = Arc::new(Mutex::new(SharedState {
			waker: None,
			completed: false,
		}));
		connection
			.cookies
			.lock()
			.expect("failed to lock mutex")
			.push(Arc::clone(&shared_state));

		Self {
			cookie,
			connection: &connection.xcb_conn,
			shared_state,
		}
	}
}

impl X11 {}

/// Represents the values of a [`x11::ConfigureRequestEvent`] or [`x11::ConfigureWindow`] request
/// as optional fields.
///
/// Why this is not how they are represented in rust-xcb, I cannot fathom.
pub struct ConfigureValues {
	/// Configures the x-coordinate of the window.
	pub x: Option<i16>,
	/// Configures the y-coordinate of the window.
	pub y: Option<i16>,

	/// Configures the width of the window.
	pub width: Option<u16>,
	/// Configures the height of the window.
	pub height: Option<u16>,

	/// Configures the width of the window's border.
	pub border_width: Option<u16>,
	/// Configures the window's sibling.
	pub sibling: Option<x11::Window>,
	/// Configures the window's [`StackMode`].
	pub stack_mode: Option<x11::StackMode>,
}

/// Creates a value list that can be provided to a [`x11::ConfigureWindow`] request from a
/// [`x11::ConfigureRequestEvent`].
pub fn value_list(request: &x11::ConfigureRequestEvent) -> Vec<x11::ConfigWindow> {
	Vec::from(&ConfigureValues::from(request))
}

impl<'request> From<&'request x11::ConfigureRequestEvent> for ConfigureValues {
	fn from(request: &'request x11::ConfigureRequestEvent) -> Self {
		use x11::ConfigWindowMask as Mask;

		let mask = request.value_mask();

		Self {
			x: mask.contains(Mask::X).then(|| request.x()),
			y: mask.contains(Mask::Y).then(|| request.y()),

			width: mask.contains(Mask::WIDTH).then(|| request.width()),
			height: mask.contains(Mask::HEIGHT).then(|| request.height()),

			border_width: mask.contains(Mask::BORDER_WIDTH).then(|| request.border_width()),
			sibling: mask.contains(Mask::SIBLING).then(|| request.sibling()),
			stack_mode: mask.contains(Mask::STACK_MODE).then(|| request.stack_mode()),
		}
	}
}

impl<'request, 'values> From<&'request x11::ConfigureWindow<'values>> for ConfigureValues {
	fn from(request: &'request x11::ConfigureWindow<'values>) -> Self {
		let (mut x, mut y) = (None, None);
		let (mut width, mut height) = (None, None);
		let (mut border_width, mut sibling, mut stack_mode) = (None, None, None);

		for value in request.value_list {
			match value {
				x11::ConfigWindow::X(value) => x = Some(*value as i16),
				x11::ConfigWindow::Y(value) => y = Some(*value as i16),

				x11::ConfigWindow::Width(value) => width = Some(*value as u16),
				x11::ConfigWindow::Height(value) => height = Some(*value as u16),

				x11::ConfigWindow::BorderWidth(value) => border_width = Some(*value as u16),
				x11::ConfigWindow::Sibling(value) => sibling = Some(*value),
				x11::ConfigWindow::StackMode(value) => stack_mode = Some(*value),
			}
		}

		Self {
			x,
			y,

			width,
			height,

			border_width,
			sibling,
			stack_mode,
		}
	}
}

impl<'values> From<&'values ConfigureValues> for Vec<x11::ConfigWindow> {
	fn from(values: &'values ConfigureValues) -> Self {
		let x = values.x.map(|x| x11::ConfigWindow::X(x as i32));
		let y = values.y.map(|y| x11::ConfigWindow::Y(y as i32));

		let width = values.width.map(|width| x11::ConfigWindow::Width(width as u32));
		let height = values.height.map(|height| x11::ConfigWindow::Height(height as u32));

		let border_width = values
			.border_width
			.map(|border_width| x11::ConfigWindow::BorderWidth(border_width as u32));
		let sibling = values.sibling.map(x11::ConfigWindow::Sibling);
		let stack_mode = values.stack_mode.map(x11::ConfigWindow::StackMode);

		// Put all the config values into a vector and filter out the `None` values.
		vec![x, y, width, height, border_width, sibling, stack_mode]
			.into_iter()
			.flatten()
			.collect()
	}
}
