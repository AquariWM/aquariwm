// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use xcb::x::{self, ConfigWindow, ConfigWindowMask, StackMode, Window};

/// Allows wrapping an object of type `T` to instantiate [`Self`] and unwrapping to get it back.
///
/// The intention of this trait is to wrap objects to provide extra utilities around that object.
pub trait Wrapping<T> {
	/// Constructs itself by wrapping an object of its [`Wrapping`] type `T`.
	fn wrap(contents: T) -> Self;

	/// Returns the wrapped object.
	fn unwrap(self) -> T;
}

/// Wraps values for [ConfigWindowReqEvent]s. Equivalent to [ConfigWindow].
///
/// The purpose of this enum is to allow easy access to a particular field's associated
/// [ConfigWindowMask]. It can be converted [`into()`](Into::into) a [ConfigWindow] enum, and a
/// [ConfigWindow] enum can, in turn, be converted [`into()`](Into::into) a [ConfigWindowField].
pub enum ConfigWindowField {
	X(i16),
	Y(i16),
	Width(u16),
	Height(u16),
	BorderWidth(u16),
	Sibling(Window),
	StackMode(StackMode),
}

impl ConfigWindowField {
	/// Get the corresponding [ConfigWindowMask] for a [ConfigWindowField].
	///
	/// The mask can be easily checked in a [ConfigWindowReqEvent] with
	/// [ConfigWindowReqEvent::check_mask].
	pub fn mask(&self) -> ConfigWindowMask {
		match self {
			Self::X(_) => ConfigWindowMask::X,
			Self::Y(_) => ConfigWindowMask::Y,
			Self::Width(_) => ConfigWindowMask::WIDTH,
			Self::Height(_) => ConfigWindowMask::HEIGHT,
			Self::BorderWidth(_) => ConfigWindowMask::BORDER_WIDTH,
			Self::Sibling(_) => ConfigWindowMask::SIBLING,
			Self::StackMode(_) => ConfigWindowMask::STACK_MODE,
		}
	}
}

// Convert from [ConfigWindow] to [ConfigWindowField].
impl From<ConfigWindow> for ConfigWindowField {
	fn from(config_window: ConfigWindow) -> Self {
		match config_window {
			ConfigWindow::X(x) => Self::X(x as i16),
			ConfigWindow::Y(y) => Self::Y(y as i16),
			ConfigWindow::Width(width) => Self::Width(width as u16),
			ConfigWindow::Height(height) => Self::Height(height as u16),
			ConfigWindow::BorderWidth(border_width) => Self::BorderWidth(border_width as u16),
			ConfigWindow::Sibling(sibling) => Self::Sibling(sibling),
			ConfigWindow::StackMode(stack_mode) => Self::StackMode(stack_mode),
		}
	}
}

// Convert from [ConfigWindowField] to [ConfigWindow].
impl From<ConfigWindowField> for ConfigWindow {
	fn from(config_window_field: ConfigWindowField) -> ConfigWindow {
		match config_window_field {
			ConfigWindowField::X(x) => ConfigWindow::X(x.into()),
			ConfigWindowField::Y(y) => ConfigWindow::Y(y.into()),
			ConfigWindowField::Width(width) => ConfigWindow::Width(width.into()),
			ConfigWindowField::Height(height) => ConfigWindow::Height(height.into()),
			ConfigWindowField::BorderWidth(border_width) => {
				ConfigWindow::BorderWidth(border_width.into())
			}
			ConfigWindowField::Sibling(sibling) => ConfigWindow::Sibling(sibling),
			ConfigWindowField::StackMode(stack_mode) => ConfigWindow::StackMode(stack_mode),
		}
	}
}

/// A wrapper around [xcb::x::ConfigureRequestEvent]s that checks for missing fields.
///
/// This wrapper returns [`Option`](core::option)s for values that may be missing, and provides a
/// [`values()`](WrappedConfigureRequestEvent::values) method to simplify sending a
/// [ConfigureWindow](xcb::x::ConfigureWindow) request with matching parameters.
///
/// An existing [ConfigureRequestEvent](xcb::x::ConfigureRequestEvent) can be wrapped like so:
/// ```
/// xcb::Event::X(x::Event::ConfigureRequest(req)) => {
///     let req = ConfigWindowReqEvent::wrap(req);
/// }
/// ```
pub struct ConfigWindowReqEvent {
	xcb_event: x::ConfigureRequestEvent,
}

impl Wrapping<x::ConfigureRequestEvent> for ConfigWindowReqEvent {
	fn wrap(xcb_event: x::ConfigureRequestEvent) -> Self {
		Self { xcb_event }
	}

	fn unwrap(self) -> x::ConfigureRequestEvent {
		self.xcb_event
	}
}

impl ConfigWindowReqEvent {
	/// Gets the [StackMode](x::StackMode) from the request, or [None] if not provided.
	pub fn stack_mode(&self) -> Option<x::StackMode> {
		self.wrap_value(self.xcb_event.stack_mode(), ConfigWindowMask::STACK_MODE)
	}

	/// Gets the parent [Window] from the request, or [None] if not provided.
	pub fn parent(&self) -> Window {
		self.xcb_event.parent()
	}

	/// Gets the [Window] from the request, or [None] if not provided.
	pub fn window(&self) -> Window {
		self.xcb_event.window()
	}

	/// Gets the sibling [Window] from the request, or [None] if not provided.
	pub fn sibling(&self) -> Option<Window> {
		self.wrap_value(self.xcb_event.sibling(), ConfigWindowMask::SIBLING)
	}

	/// Gets the x-coordinate from the request, or [None] if not provided.
	pub fn x(&self) -> Option<i32> {
		self.wrap_value(self.xcb_event.x() as i32, ConfigWindowMask::X)
	}

	/// Gets the y-coordinate from the request, or [None] if not provided.
	pub fn y(&self) -> Option<i32> {
		self.wrap_value(self.xcb_event.y() as i32, ConfigWindowMask::Y)
	}

	/// Gets the width from the request, or [None] if not provided.
	pub fn width(&self) -> Option<u32> {
		self.wrap_value(self.xcb_event.width() as u32, ConfigWindowMask::WIDTH)
	}

	/// Gets the height from the request, or [None] if not provided.
	pub fn height(&self) -> Option<u32> {
		self.wrap_value(self.xcb_event.height() as u32, ConfigWindowMask::HEIGHT)
	}

	/// Gets the border width from the request, or [None] if not provided.
	pub fn border_width(&self) -> Option<u32> {
		self.wrap_value(
			self.xcb_event.border_width() as u32,
			ConfigWindowMask::BORDER_WIDTH,
		)
	}

	/// Gets all the values provided in the request.
	///
	/// This makes it easier to send in a [ConfigureWindow](xcb::x::ConfigureWindow) request:
	/// ```
	/// xcb::Event::X(x::Event::ConfigureRequest(req)) => {
	///     let req = ConfigWindowReqEvent::wrap(req);
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
			ConfigWindowField::X(self.xcb_event.x()),
			ConfigWindowField::Y(self.xcb_event.y()),
			ConfigWindowField::Width(self.xcb_event.width()),
			ConfigWindowField::Height(self.xcb_event.height()),
			ConfigWindowField::BorderWidth(self.xcb_event.border_width()),
			ConfigWindowField::Sibling(self.xcb_event.sibling()),
			ConfigWindowField::StackMode(self.xcb_event.stack_mode()),
		];

		// We filter the fields by checking their masks with [`Self::check_mask`], then convert the
		// fields from [`ConfigWindowField`] enums into [`ConfigWindow`] enums (hence the explit
		// type annotation of `<ConfigWindow, _>` on [`Iterator::filter_map`]).
		//
		// If that's a little confusing, it's basically iterator magic for filtering out the fields
		// that aren't present in the `value_mask` and converting them to [`ConfigWindow`] enums.
		fields
			.into_iter()
			.filter_map::<ConfigWindow, _>(|field| {
				self.check_mask(field.mask()).then(|| field.into())
			})
			.collect()
	}

	/// Wraps the field value with an [Option] based on its presence in the `value_mask`.
	fn wrap_value<T>(&self, field: T, mask: ConfigWindowMask) -> Option<T> {
		if self.xcb_event.value_mask().contains(mask) {
			return Some(field);
		}

		None
	}

	/// Checks the value mask to see if the given [`mask`] is present.
	pub fn check_mask(&self, mask: ConfigWindowMask) -> bool {
		self.xcb_event.value_mask().contains(mask)
	}
}
