// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use x11rb_async::protocol::xproto as x11;

/// Represents the values of a [`x11::ConfigureRequestEvent`] or [`x11::configure_window`] request
/// as optional fields.
///
/// Why this is not how they are represented in neither `xcb` nor `x11rb`, I cannot fathom.
pub struct ConfigureValues {
	/// Configures the x-coordinate of the window.
	pub x: Option<i16>,
	/// Configures the y-coordinate of the window.
	pub y: Option<i16>,

	/// Configures the width of a window.
	pub width: Option<u16>,
	/// Configures the height of a window.
	pub height: Option<u16>,

	/// Configures the width of the window's border.
	pub border_width: Option<u16>,
	/// Configures the window's sibling.
	pub sibling: Option<x11::Window>,
	/// Configures the window's [`StackMode`].
	pub stack_mode: Option<x11::StackMode>,
}

impl<'request> From<&'request x11::ConfigureRequestEvent> for ConfigureValues {
	fn from(request: &'request x11::ConfigureRequestEvent) -> Self {
		use x11::ConfigWindow as Mask;

		let mask = &request.value_mask;

		Self {
			x: mask.contains(Mask::X).then_some(request.x),
			y: mask.contains(Mask::Y).then_some(request.y),

			width: mask.contains(Mask::WIDTH).then_some(request.width),
			height: mask.contains(Mask::HEIGHT).then_some(request.height),

			border_width: mask.contains(Mask::BORDER_WIDTH).then_some(request.border_width),
			sibling: mask.contains(Mask::SIBLING).then_some(request.sibling),
			stack_mode: mask.contains(Mask::STACK_MODE).then_some(request.stack_mode),
		}
	}
}

impl<'values> From<&'values ConfigureValues> for x11::ConfigureWindowAux {
	fn from(values: &'values ConfigureValues) -> Self {
		Self {
			x: values.x.map(Into::into),
			y: values.y.map(Into::into),

			width: values.width.map(Into::into),
			height: values.height.map(Into::into),

			border_width: values.border_width.map(Into::into),
			sibling: values.sibling,
			stack_mode: values.stack_mode,
		}
	}
}

impl From<ConfigureValues> for x11::ConfigureWindowAux {
	fn from(values: ConfigureValues) -> Self {
		Self {
			x: values.x.map(Into::into),
			y: values.y.map(Into::into),

			width: values.width.map(Into::into),
			height: values.height.map(Into::into),

			border_width: values.border_width.map(Into::into),
			sibling: values.sibling,
			stack_mode: values.stack_mode,
		}
	}
}
