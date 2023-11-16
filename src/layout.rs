// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::{collections::VecDeque, fmt::Debug};

/// Contains `impl` blocks for types defined in [layout].
///
/// This is a separate module to keep the [layout] module file more readable.
///
/// [layout]: self
mod implementations;

/// Default [layout managers] that come with AquariWM.
///
/// [layout managers]: TilingLayoutManager
mod managers;

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
	Tiled(Box<dyn TilingLayoutManager<Window>>),

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
pub struct TilingLayout<Window> {
	root: GroupNode<Window>,
}

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
	Window(WindowNode<Window>),
}

/// Represents a group of [nodes] in a [layout] tree.
///
/// [nodes]: Node
/// [layout]: TilingLayout
pub struct GroupNode<Window> {
	orientation: Orientation,

	nodes: VecDeque<Node<Window>>,
	/// The total size of all nodes along the [axis] of the group.
	total_node_primary: u32,

	/// Additions to `nodes` made by the [layout manager] in the latest [`add_window`] or
	/// [`remove_window`] call.
	///
	/// This is a sorted list of indexes.
	///
	/// Additions are tracked so that nodes can be resized afterwards. This prevents multiple
	/// resizings per node, which is particularly important when it comes to the resized windows.
	///
	/// [layout manager]: TilingLayoutManager
	///
	/// [`add_window`]: TilingLayoutManager::add_window
	/// [`remove_window`]: TilingLayoutManager::remove_window
	additions: VecDeque<usize>,
	total_removed_primary: u32,

	/// The new [`orientation`] for the group set by the [layout manager] in the latest
	/// [`add_window`] or [`remove_window`] call.
	///
	/// [`orientation`]: Self::orientation()
	/// [layout manager]: TilingLayoutManager
	///
	/// [`add_window`]: TilingLayoutManager::add_window
	/// [`remove_window`]: TilingLayoutManager::remove_window
	new_orientation: Option<Orientation>,
	new_width: Option<u32>,
	new_height: Option<u32>,

	width: u32,
	height: u32,
}

/// Represents a [node] containing a window.
///
/// [node]: Node
pub struct WindowNode<Window> {
	window: Window,

	width: u32,
	height: u32,
}

/// Manages a [tiling layout], restructuring the layout when a window needs to be [added] or
/// [removed].
///
/// `Window` is the type used to represent windows. It is a generic type parameter because different
/// display server implementations use different window types.
///
/// # Safety
/// This is an unsafe trait because implementors must uphold guarantees regarding windows being
/// added to and removed from the layout:
/// - [`layout`] *must* return a shared reference to the [`TilingLayout`] managed by the layout
///   manager.
/// - [`layout_mut`] *must* return a mutable reference to the [`TilingLayout`] managed by the layout
///   manager.
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
/// # Implementation notes
/// This trait should be implemented for all possible window types so that it works on all AquariWM
/// display server implementations.
/// ```
/// # struct MyLayoutManager<Window>;
/// #
/// unsafe impl<Window> TilingLayoutManager<Window>
///     for MyLayoutManager<Window> {
///     // ...
/// #     const ORIENTATION: Orientation = Orientation::LeftToRight;
/// #
/// #     fn init(
/// #         layout: TilingLayout<Window>,
/// #         windows: impl IntoIterator<Item = Window>,
/// #     ) -> Self {
/// #         Self
/// #     }
/// #
/// #     fn layout(&self) -> &TilingLayout<Window> { unimplemented!() }
/// #     fn layout_mut(&self) -> &mut TilingLayout<Window> { unimplemented!() }
/// #
/// #     fn add_window(&mut self, window: Window) {}
/// #
/// #     fn remove_window(&mut self, window: &Window) {}
/// }
/// ```
///
/// [tiling layout]: TilingLayout
///
/// [added]: Self::add_window
/// [`add_window`]: Self::add_window
///
/// [removed]: Self::remove_window
/// [`remove_window`]: Self::remove_window
pub unsafe trait TilingLayoutManager<Window>: Send + Sync + 'static {
	/// The default [orientation] for layouts created with this layout manager.
	///
	/// [orientation]: Orientation
	fn orientation() -> Orientation
	where
		Self: Sized;

	fn init<WindowsIter>(layout: TilingLayout<Window>, windows: WindowsIter) -> Self
	where
		Self: Sized,
		WindowsIter: IntoIterator<Item = Window>,
		WindowsIter::IntoIter: ExactSizeIterator;

	/// Returns a shared reference to the [layout] managed by the layout manager.
	///
	/// [layout]: TilingLayout
	fn layout(&self) -> &TilingLayout<Window>;

	/// Returns a mutable reference to the [layout] managed by the layout manager.
	///
	/// [layout]: TilingLayout
	fn layout_mut(&mut self) -> &mut TilingLayout<Window>;

	/// Add the given `window` to the layout.
	///
	/// # Implementation notes
	/// The `window` *must* be added to the layout somewhere. The layout *may* be restructured in
	/// response to the new node being added to the layout.
	///
	/// See the [trait documentation] for more information.
	///
	/// [trait documentation]: Self
	fn add_window(&mut self, window: Window);

	/// Remove the given `window` from the layout.
	///
	/// # Implementation notes
	/// The `window` *must* be removed from the layout. The layout *may* be restructured in response
	/// to that node being removed from the layout.
	///
	/// See the [trait documentation] for more information.
	///
	/// [trait documentation]: Self
	fn remove_window(&mut self, window: &Window);
}
