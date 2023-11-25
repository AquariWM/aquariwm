// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::{
	borrow::{Borrow, BorrowMut},
	mem,
	ops::{Deref, DerefMut, Index, IndexMut},
};

use thiserror::Error;

use super::*;

mod iter;

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

impl<Window> Leaf<Window> {
	/// Creates a new leaf node with the given `window`.
	pub fn new(window: Window) -> Self {
		Self(Rc::new(RefCell::new(LeafData {
			window,

			parent: None,

			// Coordinates and dimensions are placeholder values until changes are applied to the
			// parent branch node.
			x: 0,
			y: 0,

			width: 0,
			height: 0,

			changes_made: None,
		})))
	}
}

impl<Window> Branch<Window> {
	/// Creates a new branch node with the given `window`.
	pub fn new(orientation: Orientation) -> Self {
		Self(Rc::new(RefCell::new(BranchData {
			orientation,

			parent: None,
			children: VecDeque::new(),

			// Coordinates and dimensions are placeholder values until changes are applied to the
			// parent branch node.
			x: 0,
			y: 0,

			width: 0,
			height: 0,

			changes_made: None,
		})))
	}
}

impl<Window> NewNode<Window> {
	/// Creates a new leaf node with the given `window`.
	///
	/// This is equivalent to <code>Self::Leaf([Leaf::new]\(window))</code>.
	#[inline(always)]
	pub fn new_leaf(window: Window) -> Self {
		Self::Leaf(Leaf::new(window))
	}

	/// Creates a new branch node with the given `orientation`.
	///
	/// This is equivalent to <code>Self::Branch([BranchData::new]\(window))</code>.
	#[inline(always)]
	pub fn new_branch(orientation: Orientation) -> Self {
		Self::Branch(Branch::new(orientation))
	}
}

impl<Window> NewNode<Window> {
	#[inline]
	fn set_parent(&mut self, parent: Weak<RefCell<BranchData<Window>>>) {
		match self {
			Self::Leaf(leaf) => RefCell::borrow_mut(&leaf.0).parent = Some(parent),
			Self::Branch(branch) => RefCell::borrow_mut(&branch.0).parent = Some(parent),
		}
	}

	#[inline]
	fn remove_parent(&mut self) {
		match self {
			Self::Leaf(leaf) => RefCell::borrow_mut(&leaf.0).parent = None,
			Self::Branch(branch) => RefCell::borrow_mut(&branch.0).parent = None,
		}
	}

	/// Returns the node's parent, if any.
	#[inline(always)]
	pub fn parent(&self) -> Option<Branch<Window>> {
		match self {
			Self::Leaf(leaf) => leaf.parent(),
			Self::Branch(branch) => branch.parent(),
		}
	}
}

/// An error returned if there are still references to a node when it is attempted to be unwrapped.
#[derive(Debug, Error, PartialEq, Eq, Hash, Clone, Copy)]
#[error("unable to unwrap the leaf node because there are still references to it")]
pub struct NodeUnwrapError;

impl<Window> Leaf<Window> {
	/// Returns the leaf node's parent, if any.
	#[inline]
	pub fn parent(&self) -> Option<Branch<Window>> {
		RefCell::borrow(&self.0)
			.parent
			.as_ref()
			.map(Weak::upgrade)
			.flatten()
			.map(Branch)
	}

	/// Returns a reference to the leaf node's window.
	#[inline(always)]
	pub fn window(&self) -> &Window {
		&RefCell::borrow(&self.0).window
	}

	/// Consumes the leaf node, returning its window.
	///
	/// # Errors
	/// A [`NodeUnwrapError`] is returned if there are still references to the leaf node.
	#[inline(always)]
	pub fn into_window(self) -> Result<Window, NodeUnwrapError> {
		match Rc::try_unwrap(self.0) {
			Ok(refcell) => Ok(refcell.into_inner().window),
			Err(_) => Err(NodeUnwrapError),
		}
	}

	/// Replaces the leaf node's window with the given `window`, returning the old one.
	#[inline]
	pub fn replace_window(&mut self, window: Window) -> Window {
		let this = RefCell::borrow_mut(&self.0);

		this.changes_made = Some(NodeChanges::Both);

		mem::replace(&mut this.window, window)
	}

	/// Swaps the windows of this leaf node and the `other` leaf node.
	pub fn swap_window(&mut self, other: &Leaf<Window>) {
		let this = RefCell::borrow_mut(&self.0);
		let other = RefCell::borrow_mut(&other.0);

		this.changes_made = Some(NodeChanges::Both);
		other.changes_made = Some(NodeChanges::Both);

		mem::swap(&mut this.window, &mut other.window);
	}
}

impl BranchChanges {
	fn new() -> Self {
		Self {
			new_orientation: None,
			additions: VecDeque::new(),

			other_changes_made: None,
		}
	}
}

impl<Window> Branch<Window> {
	/// Returns the branch node's parent, if any.
	#[inline]
	pub fn parent(&self) -> Option<Branch<Window>> {
		RefCell::borrow(&self.0)
			.parent
			.as_ref()
			.map(Weak::upgrade)
			.flatten()
			.map(Branch)
	}

	/// Returns the branch node's [orientation].
	///
	/// # See also
	/// To set the branch node's orientation, use [`set_orientation`]. To rotate it by a certain
	/// number of rotations, use [`rotate_by`].
	///
	/// [orientation]: Orientation
	///
	/// [`set_orientation`]: Self::set_orientation
	/// [`rotate_by`]: Self::rotate_by
	pub fn orientation(&self) -> Orientation {
		let borrow = RefCell::borrow(&self.0);

		if let Some(BranchChanges {
			new_orientation: Some(orientation),
			..
		}) = borrow.changes_made
		{
			orientation
		} else {
			borrow.orientation
		}
	}

	/// Sets the branch node's [orientation].
	///
	/// # See also
	/// To rotate the branch node's [`orientation`] by a certain number of rotations, use
	/// [`rotate_by`].
	///
	/// [orientation]: Orientation
	/// [`orientation`]: Self::orientation()
	/// [`rotate_by`]: Self::rotate_by
	#[inline]
	pub fn set_orientation(&mut self, orientation: Orientation) {
		RefCell::borrow_mut(&self.0)
			.changes_made
			.get_or_insert_with(BranchChanges::new)
			.new_orientation = Some(orientation);
	}

	/// Rotates the branch node's [orientation] by the given number of `rotations`.
	///
	/// A positive number of rotations will rotate the [`orientation`] clockwise, while a negative
	/// number of rotations will rotate the [`orientation`] counter-clockwise.
	///
	/// # See also
	/// To set the [`orientation`] to a specific [orientation], use [`set_orientation`].
	///
	/// [orientation]: Orientation
	///
	/// [`orientation`]: Self::orientation()
	/// [`set_orientation`]: Self::set_orientation
	#[inline(always)]
	pub fn rotate_by(&mut self, rotations: i32) {
		self.set_orientation(self.orientation().rotated_by(rotations));
	}
}

impl<Window> Branch<Window> {
	/// Determines the appropriate index based on whether the branch node's orientation is reversed.
	///
	/// # Safety
	/// `index` must be within bounds.
	#[inline]
	unsafe fn determine_index(&self, index: usize) -> usize {
		let this = RefCell::borrow(&self.0);

		if this.orientation.reversed() {
			this.children.len() - index - 1
		} else {
			index
		}
	}

	#[inline]
	fn get_or_create_additions(changes_made: &mut Option<BranchChanges>) -> &mut VecDeque<usize> {
		&mut changes_made.get_or_insert_with(BranchChanges::new).additions
	}

	/// Tracks the removal of a node at the given `index`.
	fn track_remove(index: usize, additions: &mut VecDeque<usize>) {
		let shifted_additions = match additions.binary_search(&index) {
			// An addition we were tracking was removed.
			Ok(addition) => {
				additions.remove(addition);

				addition..
			},

			// The removed node was not an addition we were tracking.
			Err(removal_point) => removal_point..,
		};

		// Move additions following the removal point back by 1.
		for addition in additions.range_mut(shifted_additions) {
			*addition -= 1;
		}
	}

	/// Tracks a node being popped at the given `index`.
	#[inline]
	fn track_pop_back(index: usize, additions: &mut VecDeque<usize>) {
		if !additions.is_empty() {
			// If it was one of our own additions, pop that addition.
			if additions[additions.len() - 1] == index {
				additions.pop_back();
			}
		}
	}

	/// Tracks a node being popped from the front.
	fn track_pop_front(additions: &mut VecDeque<usize>) {
		if !additions.is_empty() {
			// The index that the node was popped from.
			const INDEX: usize = 0;

			// If it was one of our own additions, pop that addition.
			if additions[0] == INDEX {
				additions.pop_front();
			}

			// Move all the other additions back by one.
			for addition in additions {
				*addition -= 1;
			}
		}
	}

	/// Tracks the insertion of a node at the given `index`.
	#[inline]
	fn track_insert(index: usize, additions: &mut VecDeque<usize>) {
		additions.insert(additions.partition_point(|i| *i < index), index)
	}

	/// Tracks a node being pushed to the front.
	#[inline]
	fn track_push_front(additions: &mut VecDeque<usize>) {
		// Move every other addition over by 1.
		for addition in additions {
			*addition += 1;
		}

		additions.push_front(0);
	}

	/// Tracks a node being pushed to the back.
	#[inline(always)]
	fn track_push_back(index: usize, additions: &mut VecDeque<usize>) {
		additions.push_back(index);
	}
}

impl<Window> Branch<Window> {
	/// Removes the [node] at the given `index`, returning [`None`] if `index` is out of bounds.
	///
	/// # See also
	/// The first and last [nodes][node] respectively can be removed with [`pop_front`] and
	/// [`pop_back`].
	///
	/// [node]: NewNode
	///
	/// [`pop_front`]: Self::pop_front
	/// [`pop_back`]: Self::pop_back
	pub fn remove(&mut self, index: usize) -> Option<Rc<RefCell<NewNode<Window>>>> {
		let borrow = RefCell::borrow_mut(&self.0);

		if index < borrow.children.len() {
			let index = unsafe { self.determine_index(index) };

			// Track the removal of the node.
			Self::track_remove(index, Self::get_or_create_additions(&mut borrow.changes_made));

			let mut ret = borrow.children.remove(index);

			// Remove the node's parent.
			if let Some(ret) = ret.as_ref() {
				RefCell::borrow_mut(ret).remove_parent();
			}

			ret
		} else {
			None
		}
	}

	/// Pops the branch node's first child.
	pub fn pop_front(&mut self) -> Option<Rc<RefCell<NewNode<Window>>>> {
		let borrow = RefCell::borrow_mut(&self.0);

		let mut ret = if borrow.orientation.reversed() {
			// Track the pop.
			Self::track_pop_back(
				borrow.children.len(),
				Self::get_or_create_additions(&mut borrow.changes_made),
			);

			borrow.children.pop_back()
		} else {
			// Track the pop.
			Self::track_pop_front(Self::get_or_create_additions(&mut borrow.changes_made));

			borrow.children.pop_front()
		};

		// Remove the node's parent.
		if let Some(ret) = ret.as_ref() {
			RefCell::borrow_mut(ret).remove_parent();
		}

		ret
	}

	/// Pop the branch node's last child.
	pub fn pop_back(&mut self) -> Option<Rc<RefCell<NewNode<Window>>>> {
		let borrow = RefCell::borrow_mut(&self.0);

		let mut ret = if borrow.orientation.reversed() {
			// Track the pop.
			Self::track_pop_front(Self::get_or_create_additions(&mut borrow.changes_made));

			borrow.children.pop_front()
		} else {
			// Track the pop.
			Self::track_pop_back(
				borrow.children.len(),
				Self::get_or_create_additions(&mut borrow.changes_made),
			);

			borrow.children.pop_back()
		};

		// Remove the node's parent.
		if let Some(ret) = ret.as_ref() {
			RefCell::borrow_mut(ret).remove_parent();
		}

		ret
	}

	/// Inserts the given `node` at the given `index`.
	///
	/// # Panics
	/// Panics if `index` is greater than the number of children in the branch node.
	pub fn insert(&mut self, index: usize, mut node: NewNode<Window>) {
		let len = RefCell::borrow(&self.0).children.len();

		if index < len {
			let index = unsafe { self.determine_index(index) };

			// Set the node's parent to this branch node.
			node.set_parent(Rc::downgrade(&self.0));

			let borrow = RefCell::borrow_mut(&self.0);

			// Track the insertion.
			Self::track_insert(index, Self::get_or_create_additions(&mut borrow.changes_made));

			borrow.children.insert(index, Rc::new(RefCell::new(node)));
		} else {
			panic!("the given `index` ({index}) was greater than the number of children ({len})");
		}
	}

	/// Pushes the given `node` to the beginning of the branch node's children.
	pub fn push_front(&mut self, mut node: NewNode<Window>) {
		// Set the node's parent to this branch node.
		node.set_parent(Rc::downgrade(&self.0));
		let node = Rc::new(RefCell::new(node));

		let borrow = RefCell::borrow_mut(&self.0);

		// Push the node.
		if borrow.orientation.reversed() {
			let index = borrow.children.len();

			// Track the push.
			Self::track_push_back(index, Self::get_or_create_additions(&mut borrow.changes_made));

			borrow.children.push_back(node);
		} else {
			// Track the push.
			Self::track_push_front(Self::get_or_create_additions(&mut borrow.changes_made));

			borrow.children.push_front(node);
		};
	}

	/// Pushes the given `node` to the end of the branch node's children.
	pub fn push_back(&mut self, mut node: NewNode<Window>) {
		// Set the node's parent to this branch node.
		node.set_parent(Rc::downgrade(&self.0));
		let node = Rc::new(RefCell::new(node));

		let borrow = RefCell::borrow_mut(&self.0);

		// Push the node.
		if borrow.orientation.reversed() {
			// Track the push.
			Self::track_push_front(Self::get_or_create_additions(&mut borrow.changes_made));

			borrow.children.push_front(node);
		} else {
			let index = borrow.children.len();

			// Track the push.
			Self::track_push_back(index, Self::get_or_create_additions(&mut borrow.changes_made));

			borrow.children.push_back(node);
		};
	}
}
