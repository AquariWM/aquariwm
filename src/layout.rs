// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::fmt::Debug;

/// Whether a window is [`Tiled`] or [`Floating`].
///
/// [`Tiled`]: Mode::Tiled
/// [`Floating`]: Mode::Floating
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, Default)]
pub enum Mode {
	/// When a tiling layout is active, the window is tiled.
	///
	/// This has no effect while no tiling layout is active, but will take effect when a tiling
	/// layout is activated, so it is still worth keeping track of.
	#[default]
	Tiled,

	/// The window is not tiled in a tiling layout, even if one is active.
	Floating,
}

/// AquariWM's current window layout manager.
#[derive(Debug, Default)]
pub enum Layout {
	/// AquariWM is currently using a tiling layout.
	Tiled(Box<dyn TilingLayout + Send + Sync>),

	/// AquariWM is not currently using a tiling layout.
	#[default]
	Floating,
}

pub trait TilingLayout: Debug {}
