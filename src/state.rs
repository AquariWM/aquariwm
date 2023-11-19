// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::{collections::HashMap, hash::Hash};

#[cfg(feature = "async")]
use {futures::future, std::future::Future};

use crate::layout::{self, CurrentLayout};

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum MapState {
	Mapped,
	Unmapped,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub struct WindowState {
	pub mode: layout::Mode,
	pub mapped: MapState,
}

impl WindowState {
	#[inline]
	pub fn new(mapped: MapState) -> Self {
		Self {
			mode: layout::Mode::default(),
			mapped,
		}
	}

	#[inline]
	pub const fn with_layout_mode(mode: layout::Mode, mapped: MapState) -> Self {
		Self { mode, mapped }
	}

	#[inline]
	pub fn set_floating(&mut self) {
		self.mode = layout::Mode::Floating;
	}

	#[inline]
	pub fn set_tiled(&mut self) {
		self.mode = layout::Mode::Tiled;
	}

	#[inline]
	pub fn set_unmapped(&mut self) {
		self.mapped = MapState::Unmapped;
	}

	#[inline]
	pub fn set_mapped(&mut self) {
		self.mapped = MapState::Mapped;
	}
}

pub struct AquariWm<Window: Eq + Hash + Clone + 'static> {
	/// The current window layout.
	pub layout: CurrentLayout<Window>,

	/// A [`HashMap`] of windows and their current [`WindowState`s].
	///
	/// [`WindowState`s]: WindowState
	pub windows: HashMap<Window, WindowState>,
}

impl<Window: Eq + Hash + Clone> Default for AquariWm<Window> {
	#[inline]
	fn default() -> Self {
		Self::new()
	}
}

impl<Window: Eq + Hash + Clone> AquariWm<Window> {
	/// Creates a new AquariWM state struct with the default [`CurrentLayout`] and no windows.
	#[inline]
	pub fn new() -> Self {
		Self { ..Default::default() }
	}

	/// Creates a new AquariWM state struct with the given `layout` and no windows.
	#[inline]
	pub fn with_tiling_layout<Manager>(width: u32, height: u32) -> Self
	where
		Manager: layout::TilingLayoutManager<Window>,
	{
		Self {
			layout: CurrentLayout::new_tiled::<Manager>(width, height),
			windows: HashMap::new(),
		}
	}

	/// Creates a new AquariWM state struct with the default [`CurrentLayout`] and the given
	/// `windows`.
	#[inline]
	pub fn with_windows(windows: impl IntoIterator<Item = (Window, MapState)>) -> Self {
		Self::with_layout_and_windows(CurrentLayout::default(), windows)
	}

	/// Creates a new AquariWM state struct with the given `layout` and `windows`.
	pub fn with_layout_and_windows(
		layout: CurrentLayout<Window>,
		windows: impl IntoIterator<Item = (Window, MapState)>,
	) -> Self {
		let mut aquariwm = Self {
			layout,
			windows: HashMap::new(),
		};

		aquariwm.add_windows(windows);

		aquariwm
	}

	pub fn add_window(&mut self, window: Window, mapped: MapState) {
		let state = WindowState::new(mapped);

		if state.mode == layout::Mode::Tiled && state.mapped == MapState::Mapped {
			if let CurrentLayout::Tiled(manager) = &mut self.layout {
				manager.add_window(window.clone());
			}
		}

		self.windows.insert(window, state);
	}

	#[inline]
	pub fn add_windows(&mut self, windows: impl IntoIterator<Item = (Window, MapState)>) {
		for (window, mapped) in windows {
			self.add_window(window, mapped);
		}
	}

	/// Updates AquariWM's state to reflect the given `window` being destroyed.
	///
	/// In order to apply any changes that may have been made to the tiling layout,
	/// [`apply_changes`]
	#[cfg_attr(feature = "async", doc = "or [`apply_changes_async`](Self::apply_changes_async)")]
	/// must be called.
	///
	/// [`apply_changes`]: Self::apply_changes
	pub fn remove_window(&mut self, window: &Window) {
		let state = self.windows.remove(window);

		// Remove the window from the tiling layout if needed.
		if let Some(state) = state {
			if state.mode == layout::Mode::Tiled && state.mapped == MapState::Mapped {
				if let CurrentLayout::Tiled(manager) = &mut self.layout {
					manager.remove_window(window);
				}
			}
		}
	}

	/// Updates AquariWM's state to reflect the given `window` being [mapped].
	///
	/// In order to apply any changes that may have been made to the tiling layout,
	/// [`apply_changes`]
	#[cfg_attr(feature = "async", doc = "or [`apply_changes_async`](Self::apply_changes_async)")]
	/// must be called.
	///
	/// [mapped]: MapState::Mapped
	/// [`apply_changes`]: Self::apply_changes
	pub fn map_window(&mut self, window: &Window) {
		let state = self
			.windows
			.get_mut(window)
			.expect("the window we are attempting to map is not tracked");

		if state.mode == layout::Mode::Tiled && state.mapped == MapState::Unmapped {
			if let CurrentLayout::Tiled(manager) = &mut self.layout {
				manager.add_window(window.clone());
			}
		}

		state.set_mapped();
	}

	/// Updates AquariWM's state to reflect the given `window` being [unmapped].
	///
	/// In order to apply any changes that may have been made to the tiling layout,
	/// [`apply_changes`]
	#[cfg_attr(feature = "async", doc = "or [`apply_changes_async`](Self::apply_changes_async)")]
	/// must be called.
	///
	/// [unmapped]: MapState::Unmapped
	/// [`apply_changes`]: Self::apply_changes
	pub fn unmap_window(&mut self, window: &Window) {
		let state = self
			.windows
			.get_mut(window)
			.expect("the window we are attempting to unmap is not tracked");

		if state.mode == layout::Mode::Tiled && state.mapped == MapState::Mapped {
			if let CurrentLayout::Tiled(manager) = &mut self.layout {
				manager.remove_window(window);
			}
		}

		state.set_unmapped();
	}

	/// Applies changes made by the [layout manager] by calling [`apply_resizes`] with the given
	/// `resize_window` function.
	///
	/// [layout manager]: layout::TilingLayoutManager
	/// [`apply_resizes`]: layout::GroupNode::apply_resizes
	#[cfg_attr(
		feature = "async",
		doc = "",
		doc = " # See also",
		doc = "[`apply_changes_async`] allows using a `resize_window` function that returns a",
		doc = "[future].",
		doc = "",
		doc = "[`apply_changes_async`]: Self::apply_changes_async",
		doc = "[future]: Future"
	)]
	pub fn apply_changes<Error>(
		&mut self,
		mut resize_window: impl FnMut(&Window, u32, u32) -> Result<(), Error>,
	) -> Result<(), Error> {
		if let CurrentLayout::Tiled(manager) = &mut self.layout {
			manager.layout_mut().apply_resizes(&mut resize_window)?;
		}

		Ok(())
	}

	#[doc(cfg(feature = "async"))]
	/// Applies changes made by the [layout manager] by calling `apply_changes` with the given
	/// `resize_window` function.
	///
	/// # See also
	/// [`apply_changes`] allows using a `resize_window` function that doesn't return a [future].
	///
	/// [layout manager]: layout::TilingLayoutManager
	/// [future]: Future
	///
	/// [`apply_changes`]: Self::apply_changes
	#[cfg(feature = "async")]
	pub async fn apply_changes_async<ResizeWindowFuture, Error>(
		&mut self,
		mut resize_window: impl FnMut(&Window, u32, u32) -> ResizeWindowFuture,
	) -> Result<(), Error>
	where
		ResizeWindowFuture: Future<Output = Result<(), Error>>,
	{
		if let CurrentLayout::Tiled(manager) = &mut self.layout {
			// Add all the `resize_window` futures to this list...
			let mut futures = Vec::new();

			manager
				.layout_mut()
				.apply_resizes(&mut |window, width, height| -> Result<(), Error> {
					futures.push(resize_window(window, width, height));

					Ok(())
				})?;

			// Await all the `resize_window` futures.
			future::try_join_all(futures).await?;
		}

		Ok(())
	}
}
