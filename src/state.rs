// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::{collections::HashMap, hash::Hash};

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

pub struct AquariWm<Window>
where
	Window: Eq + Hash,
{
	/// The current window layout.
	pub layout: CurrentLayout<Window>,

	/// A [`HashMap`] of windows and their current [`WindowState`s].
	///
	/// [`WindowState`s]: WindowState
	pub windows: HashMap<Window, WindowState>,
}

impl<Ordinary> Default for AquariWm<Ordinary>
where
	Ordinary: Eq + Hash,
{
	#[inline]
	fn default() -> Self {
		Self::new()
	}
}

impl<Window> AquariWm<Window>
where
	Window: Eq + Hash,
{
	/// Creates a new AquariWM state struct with the default [`CurrentLayout`] and no windows.
	#[inline]
	pub fn new() -> Self {
		Self::with_layout(CurrentLayout::default())
	}

	/// Creates a new AquariWM state struct with the given `layout` and no windows.
	#[inline]
	pub fn with_layout(layout: CurrentLayout<Window>) -> Self {
		Self {
			layout,
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

	#[inline]
	pub fn add_window(&mut self, window: Window, mapped: MapState) {
		self.insert_window(window, mapped);
	}

	#[inline]
	pub fn add_windows(&mut self, windows: impl IntoIterator<Item = (Window, MapState)>) {
		for (window, mapped) in windows {
			self.insert_window(window, mapped);
		}
	}

	#[inline]
	fn insert_window(&mut self, window: Window, mapped: MapState) {
		self.windows.insert(window, WindowState::new(mapped));
	}

	#[inline]
	pub fn remove_window(&mut self, window: &Window) {
		self.windows.remove(window);
	}

	#[inline]
	pub fn map_window(&mut self, window: &Window) {
		self.windows
			.get_mut(window)
			.expect("the window we are attempting to map is not tracked")
			.set_mapped();
	}

	#[inline]
	pub fn unmap_window(&mut self, window: &Window) {
		self.windows
			.get_mut(window)
			.expect("the window we are attempting to unmap is not tracked")
			.set_unmapped();
	}
}
