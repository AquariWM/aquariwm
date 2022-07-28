// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use xcb::x::{ConfigWindow, ConfigWindowMask, ConfigureRequestEvent};

/// Extends [ConfigWindow], allowing easy access to a field's [ConfigWindowMask].
pub trait ConfigWindowExtensions {
	/// Gets the [ConfigWindowMask] that corresponds with the specific [ConfigWindow] field.
	///
	/// For example:
	/// ```
	/// let x = x::ConfigWindow::X(req.x());
	///
	/// if req.value_mask().contains(x.mask()) {
	///     return Some(x);
	/// }
	///
	/// None
	/// ```
	fn mask(&self) -> ConfigWindowMask;
}

impl ConfigWindowExtensions for ConfigWindow {
	fn mask(&self) -> ConfigWindowMask {
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

/// Extends [ConfigureRequestEvent] for easy access to a list of the request's values.
pub trait ConfigureRequestEventExtensions {
	/// Creates a list of the values contained in the request that are present in the `value_mask`.
	///
	/// This is useful for relaying values from the request when sending a new request, for
	/// example:
	/// ```
	/// xcb::Event::X(x::Event::ConfigureRequestEvent(req)) => {
	///     conn.send_request(&x::ConfigureWindow {
	///         window: req.window(),
	///         value_list: &req.values(),
	///     });
	///
	///     conn.flush()?;
	/// }
	/// ```
	fn values(&self) -> Vec<ConfigWindow>;
}

impl ConfigureRequestEventExtensions for ConfigureRequestEvent {
	fn values(&self) -> Vec<ConfigWindow> {
		let fields = [
			ConfigWindow::X(self.x().into()),
			ConfigWindow::Y(self.y().into()),
			ConfigWindow::Width(self.width().into()),
			ConfigWindow::Height(self.height().into()),
			ConfigWindow::BorderWidth(self.border_width().into()),
			ConfigWindow::Sibling(self.sibling()),
			ConfigWindow::StackMode(self.stack_mode()),
		];

		// Filter the list of fields to only contain those which are present in the `value_mask`,
		// then return it.
		fields
			.into_iter()
			.filter(|field| self.value_mask().contains(field.mask()))
			.collect()
	}
}
