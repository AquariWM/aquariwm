// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::{
	cell::RefCell,
	collections::VecDeque,
	fmt::Debug,
	rc::{Rc, Weak},
};

use derive_extras::builder;

/// Contains `impl` blocks for types defined in [layout].
///
/// This is a separate module to keep the [layout] module file more readable.
///
/// [layout]: self
mod implementations;

/// Default [layout managers] that come with AquariWM.
///
/// [layout managers]: TilingLayoutManager
pub mod managers;

// This is a false positive: `derive_extras::Default` is not the same as `Default`.
#[allow(unused_qualifications)]
/// Controls settings used when [applying] a [tiling layout].
///
/// [applying]: GroupNode::apply_changes
/// [tiling layout]: TilingLayout
#[derive(Debug, PartialEq, Eq, Hash, Clone, derive_extras::Default, builder)]
#[new]
pub struct LayoutSettings {
	/// The gap between [nodes] in the [tiling layout].
	///
	/// [nodes]: Node
	/// [tiling layout]: TilingLayout
	#[default = 15]
	pub window_gap: u32,
}

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
	root: Branch<Window>,

	x: i32,
	y: i32,

	width: u32,
	height: u32,
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

#[derive(Debug, Clone)]
pub enum Node<Window> {
	Branch(Branch<Window>),
	Leaf(Leaf<Window>),
}

#[derive(Debug, Clone)]
pub struct Branch<Window>(Rc<RefCell<BranchData<Window>>>);

#[derive(Debug, Clone)]
struct BranchData<Window> {
	orientation: Orientation,

	parent: Option<Weak<RefCell<BranchData<Window>>>>,
	children: VecDeque<Node<Window>>,

	x: i32,
	y: i32,

	width: u32,
	height: u32,

	/// The total [primary dimensions] of the `children` in the branch.
	///
	/// This is used when rescaling nodes.
	///
	/// [primary dimensions]: Node::primary_dimension
	total_children_primary_dimensions: u32,

	/// A record of the changes made to the branch node that are yet to be applied.
	///
	/// Changes are applied all at once, because certain changes can be expensive and might only
	/// have to be applied once, even for multiple operations.
	///
	/// For example, resizing a window is very expensive (not _necessarily_ for the window manager
	/// itself, but most certainly for the window, which would have to re-render itself in
	/// responsive to the change in dimensions).
	changes_made: Option<BranchChanges>,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
struct BranchChanges {
	/// The branch's orientation has been changed to this.
	///
	/// Orientation changes are not applied immediately because it is useful to compare against the
	/// old orientation to know if the axis has changed (if so, every child needs to be repositioned
	/// and resized), or otherwise if whether the orientation is [reversed] has changed (children
	/// only need to be repositioned).
	///
	/// [reversed]: Orientation::reversed
	new_orientation: Option<Orientation>,
	/// A sorted list of the indexes of the children added to the branch node.
	additions: VecDeque<usize>,

	/// Whether changes were made to the branch node's coordinates, dimensions, or both.
	other_changes_made: Option<NodeChanges>,
}

#[derive(Debug, Clone)]
pub struct Leaf<Window>(Rc<RefCell<LeafData<Window>>>);

#[derive(Debug, Clone)]
struct LeafData<Window> {
	window: Window,

	parent: Option<Weak<RefCell<BranchData<Window>>>>,

	x: i32,
	y: i32,

	width: u32,
	height: u32,

	/// A record of changes made to the leaf node that have not yet been applied to the window.
	changes_made: Option<NodeChanges>,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
enum NodeChanges {
	/// The node's coordinates have changed; window(s) need to be repositioned.
	Coordinates,
	/// The node's dimensions have changed; window(s) need to be resized.
	Dimensions,

	/// Both the node's coordinates and dimensions have changed; window(s) need to be repositioned
	/// and resized.
	Both,
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
/// # struct MyManager<Window: Send + Sync + 'static>;
/// #
/// unsafe impl<Window: Send + Sync + 'static>
///     TilingLayoutManager<Window> for MyManager<Window> {
///     // ...
/// #     fn orientation() -> Orientation
/// #     where
/// #         Self: Sized,
/// #     { Orientation::LeftToRight }
/// #
/// #     fn init<WindowsIter>(layout: TilingLayout<Window>, windows: WindowsIter) -> Self
/// #     where
/// #         Self: Sized,
/// #         WindowsIter: IntoIterator<Item = Window>,
/// #         WindowsIter::IntoIter: ExactSizeIterator,
/// #     { Self }
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
pub unsafe trait TilingLayoutManager<Window>: 'static {
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
