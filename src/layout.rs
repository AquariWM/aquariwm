// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::fmt::Debug;

/// Contains `impl` blocks for types defined in [layout].
///
/// This is a separate module to keep the [layout] module file more readable.
///
/// [layout]: self
mod implementations;

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
#[derive(Default)]
pub enum CurrentLayout<Window> {
	/// AquariWM is currently using a tiling layout.
	Tiled {
		layout: TilingLayout<Window>,
		layout_manager: Box<dyn TilingLayoutManager<Window>>,
	},

	/// AquariWM is not currently using a tiling layout.
	#[default]
	Floating,
}

/// Represents the layout of [tiled] windows.
///
/// The tiling layout is managed by a [layout manager].
///
/// [tiled]: Mode::Tiled
/// [layout manager]: TilingLayoutManager
pub struct TilingLayout<Window>(GroupNode<Window>);

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum Orientation {
	/// [Nodes] are ordered [horizontally] from left to right.
	///
	/// [Nodes]: Node
	/// [horizontally]: Axis::Horizontal
	LeftToRight,
	/// [Nodes] are ordered [vertically] from top to bottom.
	///
	/// [Nodes]: Node
	/// [vertically]: Axis::Vertical
	TopToBottom,

	/// [Nodes][nodes] are ordered [horizontally] from right to left.
	///
	/// This is a cheap way of reversing [nodes] without having to reverse the list of [nodes]. For
	/// all intents and purposes, a right-to-left orientation is equivalent to reversing a list of
	/// [nodes] that uses a [left-to-right orientation].
	///
	/// [nodes]: Node
	/// [horizontally]: Axis::Horizontal
	/// [left-to-right orientation]: Orientation::LeftToRight
	RightToLeft,
	/// [Nodes][nodes] are ordered [vertically] from bottom to top.
	///
	/// This is a cheap way of reversing [nodes] without having to reverse the list of [nodes]. For
	/// all intents and purposes, a bottom-to-top orientation is equivalent to reversing a list of
	/// [nodes] that uses a [top-to-bottom orientation].
	///
	/// [nodes]: Node
	/// [vertically]: Axis::Vertical
	/// [top-to-bottom orientation]: Orientation::TopToBottom
	BottomToTop,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum Axis {
	Horizontal,
	Vertical,
}

/// Represents a node in a [layout] tree.
///
/// This can either be a [group] or a [window].
///
/// [layout]: TilingLayout
///
/// [group]: GroupNode
/// [window]: Window
pub enum Node<Window> {
	Group(GroupNode<Window>),
	Window(Window),
}

/// Represent a group of [nodes] in a [layout] tree.
///
/// [nodes]: Node
/// [layout]: TilingLayout
pub struct GroupNode<Window> {
	orientation: Orientation,
	nodes: Vec<Node<Window>>,
}

/// Manages a [tiling layout], restructuring the layout when a window needs to be [added] or
/// [removed].
///
/// `Window` is the type used to represent windows. It is a generic type parameter because different
/// display server implementations use different window types.
///
/// # Implementors notes
/// This trait should be implemented for all possible window types so that it works on all AquariWM
/// display server implementations.
/// ```
/// # struct MyLayoutManager<Window>;
/// #
/// unsafe impl<Window> TilingLayoutManager<Window>
///     for MyLayoutManager<Window> {
///     // ...
/// #     fn init(
/// #         layout: &mut TilingLayout<Window>,
/// #         windows: impl IntoIterator<Item = Window>,
/// #     ) -> Self {
/// #         Self
/// #     }
/// #
/// #     fn add_window(window: Window) {}
/// #
/// #     fn remove_window(window: &Window) {}
/// }
/// ```
///
/// ## Safety
/// This is an unsafe trait because implementors must uphold guarantees regarding windows being
/// added to and removed from the layout:
/// - If [`add_window`] is called, that `window` *must* be added to the layout.
/// - If [`remove_window`] is called, that `window` *must* be removed from the layout.
/// - Windows *must only* be added to the layout as a result of [`add_window`] being called with
///   that `window` as an argument.
/// - Windows *must only* be removed from the layout as a result of [`remove_window`] being called
///   with the `window` as an argument.
///
/// Windows *may* be removed and then added back to the layout in the implementations of
/// [`add_window`] and [`remove_window`] to restructure the layout.
///
/// [tiling layout]: TilingLayout
///
/// [added]: Self::add_window
/// [`add_window`]: Self::add_window
///
/// [removed]: Self::remove_window
/// [`remove_window`]: Self::remove_window
pub unsafe trait TilingLayoutManager<Window>: Send + Sync {
	/// The default [orientation] for layouts created with this layout manager.
	///
	/// [orientation]: Orientation
	const ORIENTATION: Orientation;

	fn init(layout: &mut TilingLayout<Window>, windows: impl IntoIterator<Item = Window>) -> Self;

	/// Add the given `window` to the layout.
	///
	/// # Implementor notes
	/// The `window` *must* be added to the layout somewhere. The layout *may* be restructured in
	/// response to the new node being added to the layout.
	fn add_window(window: Window);

	/// Remove the given `window` from the layout.
	///
	/// # Implementor notes
	/// The `window` *must* be removed from the layout. The layout *may* be restructured in response
	/// to that node being removed from the layout.
	fn remove_window(window: &Window);
}
