// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! The layout of windows.
//!
//! There are two kinds of window layouts: _floating_ and _tiled_.
//!
//! ## Floating layouts
//! A _floating_ layout is the traditional layout of windows used in Windows, MacOS, GNOME, KDE
//! Plasma, and most other desktop environments: windows are manually repositioned and resized by
//! the user, they may overlap each other, and they have a particular stacking order (from front to
//! back).
//!
//! ## Tiling layouts
//! A _tiled_ layout is a layout where windows are tiled next to each other in a [tiling layout]
//! without overlapping each other. Because they don't overlap, their stacking order is irrelevant.
//!
//! # How do layouts work in AquariWM?
//! AquariWM can operate using either a [floating layout] layout or a [tiled layout]. In a
//! [floating layout], all windows are floating, like in a traditional floating window manager. In a
//! [tiled layout], however, AquariWM has two layers: [tiled] windows at the bottom, and [floating]
//! windows above them.
//!
//! Most windows default to being [tiled], other than certain exceptions (e.g. popup windows and
//! docks). Windows may be switched between [tiled] and [floating] modes by the user. In a
//! [floating layout], the window's [mode] is simply ignored; in a [tiled layout], it determines
//! whether the window is actually tiled.
//!
//! ## How do tiling layouts work in AquariWM?
//! Tiling layouts are represented by a [tree] of [nodes]. These nodes can either be [branches] or
//! [leaves]. Branches represent a group of nodes somewhere in the tree with a particular
//! [orientation]. Leaves represent the actual windows in the layout. Every tree has a single root
//! branch, within which all the other nodes in the tree are contained.
//!
//! The way in which windows are placed into the [tree] is determined by a [layout manager].
//! Whenever a window is put into the [tiled layout], the layout manager decides where to add a
//! [leaf node] to the tree, and whether it needs to move other parts of the tree around. The layout
//! manager may also move parts of the tree around when a window and its leaf node is removed from
//! the tiled layout.
//!
//! The only thing the [layout manager] has control over is the _structure_ of the [tree]. The user
//! is freely allowed to swap windows between [leaf nodes] without the involvement of the layout
//! manager, as well as to resize the proportions of [nodes] within their [branch]. While each node
//! has its own coordinates and dimensions, the layout manager is not made aware of them.
//!
//! When a [node] is first inserted into the [tree] by a [layout manager], it is given an equal
//! proportion of its [parent branch]; that is, if it is the fifth node in its branch, it is given
//! ⅕ of the branch. The user may resize that proportion afterwards, e.g. by dragging the node's
//! boundary.
//!
//! When a [node] is added to or removed from a [branch], all the other nodes are uniformly
//! rescaled. For example, if there are 3 nodes before a fourth is added, the existing 3 nodes will
//! each be scaled down to ¾ of their former proportions. If one of those nodes were then to be
//! removed, the rest would be scaled back up to once again fill the branch.
//!
//! ### Example - main and stack
//! A popular tiling layout is the _main and stack_ layout. The main and stack layout consists of a
//! _main_ window on the left, and a vertical _stack_ on the right with all the other windows. If
//! there is only one window, then there will be no stack; if there are no windows, there will be
//! neither stack nor main.
//!
//! A main and stack layout with 4 windows would be represented in AquariWM like so:
//!
//! <svg xmlns="http://www.w3.org/2000/svg" version = "1.1" viewBox="0 0 475 200" style = "width:475px; height:200px;">
//!     <g>
//!         <line x1="175px" y1="50px" x2="50px" y2="75px" style="stroke:white;stroke-width:2" />
//!         <line x1="175px" y1="50px" x2="300px" y2="75px" style="stroke:white;stroke-width:2" />
//!         <line x1="300px" y1="125px" x2="175px" y2="150px" style="stroke:white;stroke-width:2" />
//!         <line x1="300px" y1="125px" x2="300px" y2="150px" style="stroke:white;stroke-width:2" />
//!         <line x1="300px" y1="125px" x2="425px" y2="150px" style="stroke:white;stroke-width:2" />
//!         <rect fill="red" x="125px" y="0px" width="100px" height="50px"></rect>
//!         <rect fill="dodgerblue" x="0px" y="75px" width="100px" height="50px"></rect>
//!         <rect fill="red" x="250px" y="75px" width="100px" height="50px"></rect>
//!         <rect fill="dodgerblue" x="125px" y="150px" width="100px" height="50px"></rect>
//!         <rect fill="dodgerblue" x="250px" y="150px" width="100px" height="50px"></rect>
//!         <rect fill="dodgerblue" x="375px" y="150px" width="100px" height="50px"></rect>
//!         <text
//!             x="175px"
//!             y="20px"
//!             style="
//!                 text-anchor: middle;
//!                 dominant-baseline: middle;
//!                 fill: white;
//!                 font-size: 10px;
//!                 font-family: sans-serif;
//!                 font-weight: bold;
//!             "
//!         >
//!             Root
//!             <tspan x="175px" y="30px" style="font-weight: normal">
//!                 Branch - Horizontal
//!             </tspan>
//!         </text>
//!         <text
//!             x="50px"
//!             y="95px"
//!             style="
//!                 text-anchor: middle;
//!                 dominant-baseline: middle;
//!                 fill: white;
//!                 font-size: 10px;
//!                 font-family: sans-serif;
//!                 font-weight: bold;
//!             "
//!         >
//!             Main
//!             <tspan x="50px" y="105px" style="font-weight: normal">
//!                 Leaf
//!             </tspan>
//!         </text>
//!         <text
//!             x="300px"
//!             y="95px"
//!             style="
//!                 text-anchor: middle;
//!                 dominant-baseline: middle;
//!                 fill: white;
//!                 font-size: 10px;
//!                 font-family: sans-serif;
//!                 font-weight: bold;
//!             "
//!         >
//!             Stack
//!             <tspan x="300px" y="105px" style="font-weight: normal">
//!                 Branch - Vertical
//!             </tspan>
//!         </text>
//!         <text
//!             x="175px"
//!             y="170px"
//!             style="
//!                 text-anchor: middle;
//!                 dominant-baseline: middle;
//!                 fill: white;
//!                 font-size: 10px;
//!                 font-family: sans-serif;
//!                 font-weight: bold;
//!             "
//!         >
//!             A
//!             <tspan x="175px" y="180px" style="font-weight: normal">
//!                 Leaf
//!             </tspan>
//!         </text>
//!         <text
//!             x="300px"
//!             y="170px"
//!             style="
//!                 text-anchor: middle;
//!                 dominant-baseline: middle;
//!                 fill: white;
//!                 font-size: 10px;
//!                 font-family: sans-serif;
//!                 font-weight: bold;
//!             "
//!         >
//!             B
//!             <tspan x="300px" y="180px" style="font-weight: normal">
//!                 Leaf
//!             </tspan>
//!         </text>
//!         <text
//!             x="425px"
//!             y="170px"
//!             style="
//!                 text-anchor: middle;
//!                 dominant-baseline: middle;
//!                 fill: white;
//!                 font-size: 10px;
//!                 font-family: sans-serif;
//!                 font-weight: bold;
//!             "
//!         >
//!             C
//!             <tspan x="425px" y="180px" style="font-weight: normal">
//!                 Leaf
//!             </tspan>
//!         </text>
//!     </g>
//! </svg>
//!
//! [tree]: TilingLayout
//! [layout manager]: TilingLayoutManager
//!
//! [node]: Node
//! [nodes]: Node
//!
//! [parent branch]: Node::parent
//!
//! [branch]: Branch
//! [branches]: Branch
//!
//! [leaves]: Leaf
//! [leaf node]: Leaf
//! [leaf nodes]: Leaf
//!
//! [orientation]: Orientation
//!
//! # Implementing [`TilingLayoutManager`]
//! Layout managers, described in the previous section, can be defined by implementing the
//! [`TilingLayoutManager`] trait.
//!
//! There are certain guarantees that a [`TilingLayoutManager`] implementation must uphold; these
//! are detailed in the [`TilingLayoutManager`] documentation, which you should read before
//! continuing.
//!
//! In this guide we will be implementing a very simple layout manager, where every window is simply
//! put in a single horizontal row. You can get a lot more creative with your layout manager; I
//! encourage you to look through the [tiling layout] and [branch] documentation to see what is
//! possible (but make sure to uphold the [`TilingLayoutManager`] guarantees!).
//!
//! Before implementing [`TilingLayoutManager`], you must first begin with a struct which contains
//! a [`TilingLayout`] - this is the layout tree which your layout manager will be managing.
//! ```
//! # use aquariwm::layout::TilingLayout;
//! #
//! pub struct Row<Window: PartialEq + 'static> {
//!     layout: TilingLayout<Window>,
//! }
//! ```
//! Notice that the layout manager is generic over the `Window` type that is used: this is because
//! AquariWM is available on both X11 and Wayland, so the window type used will differ depending on
//! the display server AquariWM is running on.
//!
//! The first thing we can do implementing [`TilingLayoutManager`] is to hook up the [`layout`] and
//! [`layout_mut`] methods, as well as the initial [`orientation`] of the root [branch]:
//! ```
//! # use aquariwm::layout::{TilingLayoutManager, TilingLayout, Orientation};
//! #
//! # pub struct Row<Window: PartialEq + 'static> {
//! #     layout: TilingLayout<Window>,
//! # }
//! #
//! unsafe impl<Window> TilingLayoutManager<Window> for Row<Window>
//! where
//!     Window: PartialEq + 'static,
//! {
//!     #[inline(always)]
//!     fn orientation() -> Orientation
//!     where
//!         Self: Sized,
//!     {
//!         Orientation::LeftToRight
//!     }
//! #
//! # fn init<WindowsIter>(layout: TilingLayout<Window>, windows: WindowsIter) -> Self
//! # where
//! #     Self: Sized,
//! #     WindowsIter: IntoIterator<Item = Window>,
//! #     WindowsIter::IntoIter: ExactSizeIterator,
//! # {
//! #     unimplemented!();
//! # }
//!
//!     #[inline(always)]
//!     fn layout(&self) -> &TilingLayout<Window> {
//!         &self.layout
//!     }
//!
//!     #[inline(always)]
//!     fn layout_mut(&mut self) -> &mut TilingLayout<Window> {
//!         &mut self.layout
//!     }
//!
//!     // ... TODO ...
//! #
//! # fn add_window(&mut self, window: Window) { unimplemented!() }
//! # fn remove_window(&mut self, window: &Window) { unimplemented!() }
//! }
//! ```
//! > TODO: finish guide: implementation of `init`, `add_window`, and `remove_window`.
//!
//! [`layout`]: TilingLayoutManager::layout
//! [`layout_mut`]: TilingLayoutManager::layout_mut
//! [`orientation`]: TilingLayoutManager::orientation
//!
//! [tiling layout]: TilingLayout
//!
//! [floating layout]: CurrentLayout::Floating
//! [tiled layout]: CurrentLayout::Tiled
//!
//! [mode]: Mode
//! [floating]: Mode::Floating
//! [tiled]: Mode::Tiled

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

/// A node in a [tiling layout tree]; either a [branch] or a [leaf].
///
/// Note: nodes are internally represented with <code>[Rc]<[RefCell]<_>></code>. Cloning a node will
/// only create a new reference to it.
///
/// [tiling layout tree]: TilingLayout
/// [branch]: Branch
/// [leaf]: Leaf
#[derive(Debug, Clone)]
pub enum Node<Window> {
	/// A [branch node].
	///
	/// [branch node]: Branch
	Branch(Branch<Window>),
	/// A [leaf node].
	///
	/// [leaf node]: Leaf
	Leaf(Leaf<Window>),
}

/// A branch in a [tiling layout tree] with a particular [orientation] and zero or more child
/// [nodes].
///
/// Note: this is internally represented with <code>[Rc]<[RefCell]<_>></code>. Cloning a branch node
/// will only create a new reference to the branch.
///
/// [tiling layout tree]: TilingLayout
/// [orientation]: Orientation
/// [nodes]: Node
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

/// A leaf in a [tiling layout tree] representing a particular window.
///
/// Note: this is internally represented with <code>[Rc]<[RefCell]<_>></code>. Cloning a leaf node
/// will only create a new reference to the leaf.
///
/// [tiling layout tree]: TilingLayout
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
/// # use aquariwm::layout::{TilingLayoutManager, Orientation, TilingLayout};
/// #
/// # struct MyManager<Window: 'static>;
/// #
/// unsafe impl<Window: 'static>
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
/// For a guide on implementing `TilingLayoutManager`, see [Implementing `TilingLayoutManager`].
///
/// [Implementing `TilingLayoutManager`]: self#implementing-tilinglayoutmanager
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
