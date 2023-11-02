// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use xcb::x as x11;

pub fn circulate_window(connection: &xcb::Connection, window: x11::Window, direction: x11::Circulate) {
	connection.send_request(&x11::CirculateWindow { window, direction });

	if direction == x11::Circulate::LowerHighest {
		// TODO: lower all tiled windows below it.
	}
}

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
			.filter_map(|config_window| config_window)
			.collect()
	}
}
