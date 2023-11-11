// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

mod node_changes;

use std::{
	borrow::{Borrow, BorrowMut},
	ops::{Deref, DerefMut},
};

use super::*;

impl<Window> CurrentLayout<Window> {
	/// Creates a new [tiled layout] using the given layout `Manager` type parameter.
	///
	/// [tiled layout]: Self::Tiled
	pub(crate) fn new_tiled<Manager>(windows: impl IntoIterator<Item = Window>, width: u32, height: u32) -> Self
	where
		Manager: TilingLayoutManager<Window>,
	{
		let layout = TilingLayout::new(Manager::ORIENTATION, width, height);

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

impl<Window> Borrow<[Window]> for TilingLayout<Window> {
	fn borrow(&self) -> &[Window] {
		&self.root
	}
}

impl<Window> BorrowMut<[Window]> for TilingLayout<Window> {
	fn borrow_mut(&mut self) -> &mut [Window] {
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

	pub(crate) fn width(&self) -> u32 {
		match self {
			Self::Window(node) => node.width,
			Self::Group(node) => node.width,
		}
	}

	pub(crate) fn height(&self) -> u32 {
		match self {
			Self::Window(node) => node.height,
			Self::Group(node) => node.height,
		}
	}

	pub(crate) fn set_width(&mut self, width: u32) {
		match self {
			Self::Window(node) => node.width = width,
			Self::Group(node) => node.width = width,
		}
	}

	pub(crate) fn set_height(&mut self, height: u32) {
		match self {
			Self::Window(node) => node.height = height,
			Self::Group(node) => node.height = height,
		}
	}

	pub(crate) fn size_fn(axis: Axis) -> fn(&Self) -> u32 {
		match axis {
			Axis::Horizontal => Self::width,
			Axis::Vertical => Self::height,
		}
	}

	pub(crate) fn set_size_fn(axis: Axis) -> fn(&mut Self, u32) {
		match axis {
			Axis::Horizontal => Self::set_width,
			Axis::Vertical => Self::set_height,
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
			total_node_size: 0,

			additions: Vec::new(),
			total_removed_size: 0,
			axis_changed: false,

			width,
			height,
		}
	}

	pub(crate) fn size(&self) -> u32 {
		match self.orientation.axis() {
			Axis::Horizontal => self.width,
			Axis::Vertical => self.height,
		}
	}

	/// Returns the dimension that is not the [size()].
	///
	/// [size()]: Self::size()
	pub(crate) fn other(&self) -> u32 {
		match self.orientation.axis() {
			Axis::Horizontal => self.height,
			Axis::Vertical => self.width,
		}
	}
}

impl<Window> Borrow<[Window]> for GroupNode<Window> {
	fn borrow(&self) -> &[Window] {
		&self.nodes
	}
}

impl<Window> BorrowMut<[Window]> for GroupNode<Window> {
	fn borrow_mut(&mut self) -> &mut [Window] {
		&mut self.nodes
	}
}

impl<Window> Deref for GroupNode<Window> {
	type Target = [Window];

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
