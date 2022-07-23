use xcb::x::{self, ConfigWindow, ConfigWindowMask, Window};

/// Wraps an [xcb] event to simplify or ease its use.
pub trait WrapEvent<T>
where
	T: xcb::BaseEvent,
{
	/// Creates the wrapped event from its [xcb] counterpart.
	fn wrap(xcb_event: T) -> Self;

	/// Gets a reference to the wrapped [xcb] event.
	fn as_xcb_event(&self) -> &T;
}

/// A wrapper around [xcb::x::ConfigureRequestEvent]s that checks for missing fields.
///
/// This wrapper returns [Option]s for values that may be missing, and provides a
/// [values](WrappedConfigureRequestEvent::values) function to simplify sending a
/// [ConfigureWindow](xcb::x::ConfigureWindow) request with matching parameters.
///
/// An existing [ConfigureRequestEvent](xcb::x::ConfigureRequestEvent) can be wrapped like so:
/// ```
/// xcb::Event::X(x::Event::ConfigureRequest(req)) => {
///     let req = WrappedConfigureRequestEvent::wrap(req);
/// }
/// ```
pub struct WrappedConfigureRequestEvent {
	xcb_event: x::ConfigureRequestEvent,
}

impl WrapEvent<x::ConfigureRequestEvent> for WrappedConfigureRequestEvent {
	fn wrap(xcb_event: x::ConfigureRequestEvent) -> Self {
		Self { xcb_event }
	}

	fn as_xcb_event(&self) -> &x::ConfigureRequestEvent {
		&self.xcb_event
	}
}

impl WrappedConfigureRequestEvent {
	/// Gets the [StackMode](xcb::x::StackMode) of the window, if provided in the request.
	pub fn stack_mode(&self) -> Option<x::StackMode> {
		self.present(
			self.as_xcb_event().stack_mode(),
			ConfigWindowMask::STACK_MODE,
		)
	}

	/// Gets the parent of the window that the request is for.
	pub fn parent(&self) -> Window {
		self.as_xcb_event().parent()
	}

	/// Gets the window that the request asks to be configured.
	pub fn window(&self) -> Window {
		self.as_xcb_event().window()
	}

	/// Gets the window's sibling, if provided in the request.
	pub fn sibling(&self) -> Option<Window> {
		self.present(self.as_xcb_event().sibling(), ConfigWindowMask::SIBLING)
	}

	/// Gets the x-coordinate of the window, if provided in the request.
	///
	/// Though the underlying [ConfigRequestEvent](xcb::x::ConfigureRequestEvent) contains an [i16],
	/// the x-coordinate value is converted to an [i32] so that it can be easily provided in a
	/// [ConfigureWindow](xcb::x::ConfigureWindow) request.
	pub fn x(&self) -> Option<i32> {
		self.present(self.as_xcb_event().x() as i32, ConfigWindowMask::X)
	}

	/// Gets the y-coordinate of the window, if provided in the request.
	///
	/// Though the underlying [ConfigRequestEvent](xcb::x::ConfigureRequestEvent) contains an [i16],
	/// the y-coordinate value is converted to an [i32] so that it can be easily provided in a
	/// [ConfigureWindow](xcb::x::ConfigureWindow) request.
	pub fn y(&self) -> Option<i32> {
		self.present(self.as_xcb_event().y() as i32, ConfigWindowMask::Y)
	}

	/// Gets the width of the window, if provided in the request.
	///
	/// Though the underlying [ConfigRequestEvent](xcb::x::ConfigureRequestEvent) contains an [i16],
	/// the width is converted to an [i32] so that it can be easily provided in a
	/// [ConfigureWindow](xcb::x::ConfigureWindow) request.
	pub fn width(&self) -> Option<u32> {
		self.present(self.as_xcb_event().width() as u32, ConfigWindowMask::WIDTH)
	}

	/// Gets the height of the window, if provided in the request.
	///
	/// Though the underlying [ConfigRequestEvent](xcb::x::ConfigureRequestEvent) contains an [i16],
	/// the height is converted to an [i32] so that it can be easily provided in a
	/// [ConfigureWindow](xcb::x::ConfigureWindow) request.
	pub fn height(&self) -> Option<u32> {
		self.present(
			self.as_xcb_event().height() as u32,
			ConfigWindowMask::HEIGHT,
		)
	}

	/// Gets the border width of the window, if provided in the request.
	///
	/// Though the underlying [ConfigRequestEvent](xcb::x::ConfigureRequestEvent) contains an [i16],
	/// the border width is converted to an [i32] so that it can be easily provided in a
	/// [ConfigureWindow](xcb::x::ConfigureWindow) request.
	pub fn border_width(&self) -> Option<u32> {
		self.present(
			self.as_xcb_event().border_width() as u32,
			ConfigWindowMask::BORDER_WIDTH,
		)
	}

	/// Puts all of the values that were present in the request into a list.
	///
	/// This makes it easier to send in a [ConfigureWindow](xcb::x::ConfigureWindow) request:
	/// ```
	/// xcb::Event::X(x::Event::ConfigureRequest(req)) => {
	///     let req = WrappedConfigureRequestEvent::wrap(req);
	///
	///     conn.send_request(&x::ConfigureWindow {
	///         window: req.window(),
	///         value_list: &req.values(),
	///     });
	///
	///     conn.flush()?;
	/// }
	/// ```
	pub fn values(&self) -> Vec<ConfigWindow> {
		let fields = [
			(
				ConfigWindow::X(self.as_xcb_event().x().into()),
				ConfigWindowMask::X,
			),
			(
				ConfigWindow::Y(self.as_xcb_event().y().into()),
				ConfigWindowMask::Y,
			),
			(
				ConfigWindow::Width(self.as_xcb_event().width().into()),
				ConfigWindowMask::WIDTH,
			),
			(
				ConfigWindow::Height(self.as_xcb_event().height().into()),
				ConfigWindowMask::HEIGHT,
			),
			(
				ConfigWindow::BorderWidth(self.as_xcb_event().border_width().into()),
				ConfigWindowMask::BORDER_WIDTH,
			),
			(
				ConfigWindow::Sibling(self.as_xcb_event().sibling()),
				ConfigWindowMask::SIBLING,
			),
			(
				ConfigWindow::StackMode(self.as_xcb_event().stack_mode()),
				ConfigWindowMask::STACK_MODE,
			),
		];

		let values: Vec<ConfigWindow> = fields
			.into_iter()
			.filter_map(|(value, mask)| {
				self.as_xcb_event()
					.value_mask()
					.contains(mask)
					.then(|| value)
			})
			.collect();

		values
	}

	/// Wraps the field with an [Option] based on its presence in the `value_mask`.
	fn present<T>(&self, field: T, mask: ConfigWindowMask) -> Option<T> {
		if self.as_xcb_event().value_mask().contains(mask) {
			return Some(field);
		}

		None
	}
}
