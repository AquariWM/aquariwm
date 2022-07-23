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

pub trait ConfigureRequestEventExtensions {
	/// Creates a list of the values contained in the request that are present in the value mask.
	///
	/// This is useful for relaying values from the request when sending a new request,
	/// particularly in the creation of a window manager.
	///
	/// For example:
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

		// We filter the fields by checking their masks with [`Self::check_mask`], then convert the
		// fields from [`ConfigWindowField`] enums into [`ConfigWindow`] enums (hence the explit
		// type annotation of `<ConfigWindow, _>` on [`Iterator::filter_map`]).
		//
		// If that's a little confusing, it's basically iterator magic for filtering out the fields
		// that aren't present in the `value_mask` and converting the remaining fields to
		// [`ConfigWindow`] enums.
		fields
			.into_iter()
			.filter_map(|field| {
				self.value_mask().contains(field.mask()).then(|| field)
			})
			.collect()
	}
}
