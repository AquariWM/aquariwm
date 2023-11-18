// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::{
	borrow::{Borrow, BorrowMut},
	ops::{Deref, DerefMut, Index, IndexMut},
};

use super::*;

mod iter;
mod node_changes;

impl<Window> CurrentLayout<Window> {
	/// Creates a new [tiled layout] using the given layout `Manager` type parameter.
	///
	/// [tiled layout]: Self::Tiled
	#[inline]
	pub(crate) fn new_tiled<Manager, Windows>(windows: Windows, width: u32, height: u32) -> Self
	where
		Manager: TilingLayoutManager<Window>,
		Windows: IntoIterator<Item = Window>,
		Windows::IntoIter: ExactSizeIterator,
	{
		let layout = TilingLayout::new(Manager::orientation(), width, height);

		Self::Tiled(Box::new(Manager::init(layout, windows)))
	}
}

impl<Window> TilingLayout<Window> {
	/// Creates an empty layout of the given `orientation`.
	#[inline]
	pub(crate) const fn new(orientation: Orientation, width: u32, height: u32) -> Self {
		Self {
			root: GroupNode::with_dimensions(orientation, width, height),
		}
	}
}

impl<Window> Borrow<GroupNode<Window>> for TilingLayout<Window> {
	#[inline]
	fn borrow(&self) -> &GroupNode<Window> {
		self
	}
}

impl<Window> BorrowMut<GroupNode<Window>> for TilingLayout<Window> {
	#[inline]
	fn borrow_mut(&mut self) -> &mut GroupNode<Window> {
		self
	}
}

impl<Window> Deref for TilingLayout<Window> {
	type Target = GroupNode<Window>;

	#[inline]
	fn deref(&self) -> &Self::Target {
		&self.root
	}
}

impl<Window> DerefMut for TilingLayout<Window> {
	#[inline]
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.root
	}
}

impl<Window> Node<Window> {
	#[inline]
	pub(crate) const fn new_window(window: Window) -> Self {
		Self::Window(WindowNode::new(window))
	}

	#[inline]
	pub(crate) const fn window_with_dimensions(window: Window, width: u32, height: u32) -> Self {
		Self::Window(WindowNode::with_dimensions(window, width, height))
	}

	#[inline]
	pub(crate) const fn new_group(orientation: Orientation) -> Self {
		Self::Group(GroupNode::new(orientation))
	}

	#[inline]
	pub(crate) const fn group_with_dimensions(orientation: Orientation, width: u32, height: u32) -> Self {
		Self::Group(GroupNode::with_dimensions(orientation, width, height))
	}

	/// Returns the width of the node.
	#[inline]
	pub(crate) const fn width(&self) -> u32 {
		match self {
			Self::Window(node) => node.width,
			Self::Group(node) => node.width,
		}
	}

	/// Returns the height of the node.
	#[inline]
	pub(crate) const fn height(&self) -> u32 {
		match self {
			Self::Window(node) => node.height,
			Self::Group(node) => node.height,
		}
	}

	/// Sets the `width` of the node.
	#[inline]
	pub(crate) fn set_width(&mut self, width: u32) {
		match self {
			Self::Window(node) => node.width = width,
			Self::Group(node) => node.width = width,
		}
	}

	/// Sets the `height` of the node.
	#[inline]
	pub(crate) fn set_height(&mut self, height: u32) {
		match self {
			Self::Window(node) => node.height = height,
			Self::Group(node) => node.height = height,
		}
	}

	/// Sets the primary axis of the node.
	///
	/// The primary axis is the one that affects the node's size within its group.
	#[inline]
	pub(crate) const fn primary(&self, axis: Axis) -> u32 {
		match axis {
			Axis::Horizontal => self.width(),
			Axis::Vertical => self.height(),
		}
	}

	/// Sets the secondary axis of the node.
	///
	/// The secondary axis is the one that is only affected by the size of the node's group.
	#[inline]
	pub(crate) const fn secondary(&self, axis: Axis) -> u32 {
		match axis {
			Axis::Horizontal => self.height(),
			Axis::Vertical => self.width(),
		}
	}

	/// Sets the [`primary`] axis of the node.
	///
	/// [`primary`]: Self::primary
	#[inline]
	pub(crate) fn set_primary(&mut self, primary: u32, axis: Axis) {
		match axis {
			Axis::Horizontal => self.set_width(primary),
			Axis::Vertical => self.set_height(primary),
		}
	}

	/// Sets the [`secondary`] axis of the node.
	///
	/// [`secondary`]: Self::secondary
	#[inline]
	pub(crate) fn set_secondary(&mut self, secondary: u32, axis: Axis) {
		match axis {
			Axis::Horizontal => self.set_height(secondary),
			Axis::Vertical => self.set_width(secondary),
		}
	}
}

impl<Window> WindowNode<Window> {
	/// Creates a window node of the given `window` and dimensions of 0 by 0.
	///
	/// It is useful to create a window node with no size if that size is intended to be filled in
	/// later.
	#[inline]
	pub(crate) const fn new(window: Window) -> Self {
		Self::with_dimensions(window, 0, 0)
	}

	/// Creates a window node of the given `window` and dimensions.
	#[inline]
	pub(crate) const fn with_dimensions(window: Window, width: u32, height: u32) -> Self {
		Self { window, width, height }
	}

	/// Returns a reference to the window node's window.
	#[inline(always)]
	pub const fn window(&self) -> &Window {
		&self.window
	}

	/// Returns a mutable reference to the window node's window.
	#[inline(always)]
	pub fn window_mut(&mut self) -> &mut Window {
		&mut self.window
	}

	/// Sets the window node's window to the given `window`.
	#[inline(always)]
	pub fn set_window(&mut self, window: Window) {
		self.window = window;
	}

	/// Returns the window node's window.
	#[inline(always)]
	pub fn unwrap(self) -> Window {
		self.window
	}
}

impl<Window> Deref for WindowNode<Window> {
	type Target = Window;

	#[inline(always)]
	fn deref(&self) -> &Self::Target {
		&self.window
	}
}

impl<Window> DerefMut for WindowNode<Window> {
	#[inline(always)]
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.window
	}
}

impl<Window> Borrow<Window> for WindowNode<Window> {
	#[inline(always)]
	fn borrow(&self) -> &Window {
		self
	}
}

impl<Window> BorrowMut<Window> for WindowNode<Window> {
	#[inline(always)]
	fn borrow_mut(&mut self) -> &mut Window {
		self
	}
}

impl<Window> GroupNode<Window> {
	/// Creates an empty group of the given `orientation` and dimensions of 0 by 0.
	///
	/// It is useful to create a group with no size if that size is intended to be filled in later.
	#[inline]
	pub(crate) const fn new(orientation: Orientation) -> Self {
		Self::with_dimensions(orientation, 0, 0)
	}

	/// Creates an empty group of the given `orientation` and dimensions.
	pub(crate) const fn with_dimensions(orientation: Orientation, width: u32, height: u32) -> Self {
		Self {
			orientation,

			children: VecDeque::new(),
			total_node_primary: 0,

			additions: VecDeque::new(),
			total_removed_primary: 0,

			new_orientation: None,
			new_width: None,
			new_height: None,

			width,
			height,
		}
	}

	/// Returns the number of child [nodes] in the group.
	///
	/// This does not include further descendents of the group; a group with a single child group
	/// that itself has children is still going to have a `len` of 1.
	///
	/// [nodes]: Node
	#[inline(always)]
	pub fn len(&self) -> usize {
		self.children.len()
	}

	/// Returns [`true`] if there are no [nodes] in the group.
	///
	/// [nodes]: Node
	#[inline(always)]
	pub fn is_empty(&self) -> bool {
		self.children.is_empty()
	}

	/// Returns the first [node], or [`None`] if the group is empty.
	///
	/// [node]: Node
	#[inline]
	pub fn first(&self) -> Option<&Node<Window>> {
		match self.len() {
			0 => None,
			_ => Some(&self[0]),
		}
	}

	/// Returns the last [node], or [`None`] if the group is empty.
	///
	/// [node]: Node
	#[inline]
	pub fn last(&self) -> Option<&Node<Window>> {
		match self.len() {
			0 => None,
			len => Some(&self[len - 1]),
		}
	}

	/// Returns a mutable reference to the first [node], or [`None`] if the group is empty.
	///
	/// [node]: Node
	#[inline]
	pub fn first_mut(&mut self) -> Option<&mut Node<Window>> {
		match self.len() {
			0 => None,
			_ => Some(&mut self[0]),
		}
	}

	/// Returns a mutable reference to the last [node], or [`None`] if the group is empty.
	///
	/// [node]: Node
	#[inline]
	pub fn last_mut(&mut self) -> Option<&mut Node<Window>> {
		match self.len() {
			0 => None,
			len => Some(&mut self[len - 1]),
		}
	}

	/// Returns the [node] at the given `index`, or [`None`] if the `index` is out of bounds.
	///
	/// [node]: Node
	pub fn get(&self, index: usize) -> Option<&Node<Window>> {
		if index < self.children.len() {
			let index = if !self.orientation().reversed() {
				index
			} else {
				let last = self.children.len() - 1;
				last - index
			};

			Some(&self.children[index])
		} else {
			None
		}
	}

	/// Returns a mutable reference to the [node] at the given `index`, or [`None`] if the `index`
	/// is out of bounds.
	///
	/// [node]: Node
	pub fn get_mut(&mut self, index: usize) -> Option<&mut Node<Window>> {
		if index < self.children.len() {
			let index = if !self.orientation().reversed() {
				index
			} else {
				let last = self.children.len() - 1;
				last - index
			};

			Some(&mut self.children[index])
		} else {
			None
		}
	}

	#[inline]
	pub(crate) const fn primary(&self) -> u32 {
		match self.orientation().axis() {
			Axis::Horizontal => self.width,
			Axis::Vertical => self.height,
		}
	}

	#[inline]
	pub(crate) const fn secondary(&self) -> u32 {
		match self.orientation().axis() {
			Axis::Horizontal => self.height,
			Axis::Vertical => self.width,
		}
	}

	#[inline]
	pub(crate) fn set_width(&mut self, width: u32) {
		self.new_width = Some(width);
	}

	#[inline]
	pub(crate) fn set_height(&mut self, height: u32) {
		self.new_height = Some(height);
	}

	#[inline]
	pub(crate) fn set_primary(&mut self, primary: u32) {
		match self.orientation().axis() {
			Axis::Horizontal => self.set_width(primary),
			Axis::Vertical => self.set_height(primary),
		}
	}

	#[inline]
	pub(crate) fn set_secondary(&mut self, secondary: u32) {
		match self.orientation().axis() {
			Axis::Horizontal => self.set_height(secondary),
			Axis::Vertical => self.set_width(secondary),
		}
	}
}

impl<Window> Index<usize> for GroupNode<Window> {
	type Output = Node<Window>;

	fn index(&self, index: usize) -> &Self::Output {
		if !self.orientation().reversed() {
			&self.children[index]
		} else {
			let last = self.children.len() - 1;
			let index = last - index;

			&self.children[index]
		}
	}
}

impl<Window> IndexMut<usize> for GroupNode<Window> {
	fn index_mut(&mut self, index: usize) -> &mut Self::Output {
		if !self.orientation().reversed() {
			&mut self.children[index]
		} else {
			let last = self.children.len() - 1;
			let index = last - index;

			&mut self.children[index]
		}
	}
}

impl Orientation {
	/// Returns whether this orientation is *reversed*.
	///
	/// A reversed orientation has the effect of flipping a [group] of [nodes] without having to
	/// reverse the actual list of nodes. In a reversed orientation, for example, swapping a [node]
	/// with the next node will actually swap that node with the previous [node in the list - this
	/// mimics the effect as if the list of [nodes] had been reversed and the node was swapped with
	/// the next node in the list.
	///
	/// The reversed orientations are [right-to-left] and [bottom-to-top].
	///
	/// [nodes]: Node
	/// [node]: Node
	///
	/// [right-to-left]: Orientation::RightToLeft
	/// [bottom-to-top]: Orientation::BottomToTop
	#[inline]
	pub const fn reversed(&self) -> bool {
		match self {
			Self::LeftToRight | Self::TopToBottom => false,
			Self::RightToLeft | Self::BottomToTop => true,
		}
	}

	/// Returns the [axis] of this orientation.
	///
	/// [Left-to-right] and [right-to-left] orientations have a [`Horizontal` axis].
	/// [Top-to-bottom] and [bottom-to-top] orientations have a [`Vertical` axis].
	///
	/// [axis]: Axis
	///
	/// [Left-to-right]: Orientation::LeftToRight
	/// [right-to-left]: Orientation::RightToLeft
	///
	/// [Top-to-bottom]: Orientation::TopToBottom
	/// [bottom-to-top]: Orientation::BottomToTop
	///
	/// [`Horizontal` axis]: Axis::Horizontal
	/// [`Vertical` axis]: Axis::Vertical
	#[inline]
	pub const fn axis(&self) -> Axis {
		match self {
			Self::LeftToRight | Self::RightToLeft => Axis::Horizontal,
			Self::TopToBottom | Self::BottomToTop => Axis::Vertical,
		}
	}

	/// Returns this orientation rotated by the given number of `rotations`.
	///
	/// A positive number of rotations will rotate the orientation clockwise, while a negative
	/// number of rotations will rotate the orientation counter-clockwise.
	pub fn rotated_by(&self, rotations: i32) -> Self {
		let current = match self {
			Orientation::LeftToRight => 0,
			Orientation::TopToBottom => 1,
			Orientation::RightToLeft => 2,
			Orientation::BottomToTop => 3,
		};

		match (current + rotations).rem_euclid(4) {
			0 => Orientation::LeftToRight,
			1 => Orientation::TopToBottom,
			2 => Orientation::RightToLeft,
			3 => Orientation::BottomToTop,

			_ => unreachable!(".rem_euclid(4) returns a value within 0..4"),
		}
	}
}

impl Axis {
	/// Returns the other axis.
	///
	/// For [`Horizontal`], [`Vertical`] is returned. For [`Vertical`], [`Horizontal`] is returned.
	///
	/// [`Horizontal`]: Self::Horizontal
	/// [`Vertical`]: Self::Vertical
	#[inline]
	pub const fn flipped(&self) -> Axis {
		match self {
			Self::Horizontal => Self::Vertical,
			Self::Vertical => Self::Horizontal,
		}
	}
}
