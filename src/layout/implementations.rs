// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::{
	borrow::{Borrow, BorrowMut},
	ops::{Deref, DerefMut},
};

use super::*;

mod node_changes;

impl<Window> CurrentLayout<Window> {
	/// Creates a new [tiled layout] using the given layout `Manager` type parameter.
	///
	/// [tiled layout]: Self::Tiled
	pub(crate) fn new_tiled<Manager, Windows>(windows: Windows, width: u32, height: u32) -> Self
	where
		Manager: TilingLayoutManager<Window> + 'static,

		Windows: IntoIterator<Item = Window>,
		Windows::IntoIter: ExactSizeIterator,
	{
		let layout = TilingLayout::new(Manager::orientation(), width, height);

		Self::Tiled(Box::new(Manager::init(layout, windows)))
	}
}

impl<Window> TilingLayout<Window> {
	/// Creates an empty layout of the given `orientation`.
	pub(crate) fn new(orientation: Orientation, width: u32, height: u32) -> Self {
		Self {
			root: GroupNode::new(orientation, width, height),
		}
	}
}

impl<Window> Borrow<[Node<Window>]> for TilingLayout<Window> {
	fn borrow(&self) -> &[Node<Window>] {
		&self.root
	}
}

impl<Window> BorrowMut<[Node<Window>]> for TilingLayout<Window> {
	fn borrow_mut(&mut self) -> &mut [Node<Window>] {
		&mut self.root
	}
}

impl<Window> Deref for TilingLayout<Window> {
	type Target = [Window];

	fn deref(&self) -> &Self::Target {
		self
	}
}

impl<Window> DerefMut for TilingLayout<Window> {
	fn deref_mut(&mut self) -> &mut Self::Target {
		self
	}
}

impl<Window> Node<Window> {
	pub(crate) fn new_window(window: Window, width: u32, height: u32) -> Self {
		Self::Window(WindowNode::new(window, width, height))
	}

	pub(crate) fn new_group(orientation: Orientation, width: u32, height: u32) -> Self {
		Self::Group(GroupNode::new(orientation, width, height))
	}

	/// Returns the width of the node.
	pub(crate) fn width(&self) -> u32 {
		match self {
			Self::Window(node) => node.width,
			Self::Group(node) => node.width,
		}
	}

	/// Returns the height of the node.
	pub(crate) fn height(&self) -> u32 {
		match self {
			Self::Window(node) => node.height,
			Self::Group(node) => node.height,
		}
	}

	/// Sets the `width` of the node.
	pub(crate) fn set_width(&mut self, width: u32) {
		match self {
			Self::Window(node) => node.width = width,
			Self::Group(node) => node.width = width,
		}
	}

	/// Sets the `height` of the node.
	pub(crate) fn set_height(&mut self, height: u32) {
		match self {
			Self::Window(node) => node.height = height,
			Self::Group(node) => node.height = height,
		}
	}

	/// Sets the primary axis of the node.
	///
	/// The primary axis is the one that affects the node's size within its group.
	pub(crate) fn primary(&self, axis: Axis) -> u32 {
		match axis {
			Axis::Horizontal => self.width(),
			Axis::Vertical => self.height(),
		}
	}

	/// Sets the secondary axis of the node.
	///
	/// The secondary axis is the one that is only affected by the size of the node's group.
	pub(crate) fn secondary(&self, axis: Axis) -> u32 {
		match axis {
			Axis::Horizontal => self.height(),
			Axis::Vertical => self.width(),
		}
	}

	/// Sets the [`primary`] axis of the node.
	///
	/// [`primary`]: Self::primary
	pub(crate) fn set_primary(&mut self, primary: u32, axis: Axis) {
		match axis {
			Axis::Horizontal => self.set_width(primary),
			Axis::Vertical => self.set_height(primary),
		}
	}

	/// Sets the [`secondary`] axis of the node.
	///
	/// [`secondary`]: Self::secondary
	pub(crate) fn set_secondary(&mut self, secondary: u32, axis: Axis) {
		match axis {
			Axis::Horizontal => self.set_height(secondary),
			Axis::Vertical => self.set_width(secondary),
		}
	}
}

impl<Window> WindowNode<Window> {
	pub(crate) fn new(window: Window, width: u32, height: u32) -> Self {
		Self { window, width, height }
	}
}

impl<Window> GroupNode<Window> {
	/// Creates an empty group of the given `orientation`.
	pub(crate) fn new(orientation: Orientation, width: u32, height: u32) -> Self {
		Self {
			orientation,

			nodes: Vec::new(),
			total_node_primary: 0,

			additions: Vec::new(),
			total_removed_primary: 0,

			new_orientation: None,
			new_width: None,
			new_height: None,

			width,
			height,
		}
	}

	pub(crate) fn primary(&self) -> u32 {
		match self.orientation().axis() {
			Axis::Horizontal => self.width,
			Axis::Vertical => self.height,
		}
	}

	pub(crate) fn secondary(&self) -> u32 {
		match self.orientation().axis() {
			Axis::Horizontal => self.height,
			Axis::Vertical => self.width,
		}
	}

	pub(crate) fn set_width(&mut self, width: u32) {
		self.new_width = Some(width);
	}

	pub(crate) fn set_height(&mut self, height: u32) {
		self.new_height = Some(height);
	}

	pub(crate) fn set_primary(&mut self, primary: u32) {
		match self.orientation().axis() {
			Axis::Horizontal => self.set_width(primary),
			Axis::Vertical => self.set_height(primary),
		}
	}

	pub(crate) fn set_secondary(&mut self, secondary: u32) {
		match self.orientation().axis() {
			Axis::Horizontal => self.set_height(secondary),
			Axis::Vertical => self.set_width(secondary),
		}
	}
}

impl<Window> Borrow<[Node<Window>]> for GroupNode<Window> {
	fn borrow(&self) -> &[Node<Window>] {
		&self.nodes
	}
}

impl<Window> BorrowMut<[Node<Window>]> for GroupNode<Window> {
	fn borrow_mut(&mut self) -> &mut [Node<Window>] {
		&mut self.nodes
	}
}

impl<Window> Deref for GroupNode<Window> {
	type Target = [Node<Window>];

	fn deref(&self) -> &Self::Target {
		self
	}
}

impl<Window> DerefMut for GroupNode<Window> {
	fn deref_mut(&mut self) -> &mut Self::Target {
		self
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
	pub fn reversed(&self) -> bool {
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
	pub fn axis(&self) -> Axis {
		match self {
			Self::LeftToRight | Self::RightToLeft => Axis::Horizontal,
			Self::TopToBottom | Self::BottomToTop => Axis::Vertical,
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
	pub fn flipped(&self) -> Axis {
		match self {
			Self::Horizontal => Self::Vertical,
			Self::Vertical => Self::Horizontal,
		}
	}
}
