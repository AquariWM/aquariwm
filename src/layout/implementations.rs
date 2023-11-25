// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::{
	borrow::{Borrow, BorrowMut},
	mem,
	ops::{Deref, DerefMut},
};

use thiserror::Error;
use truncate_integer::Shrink;

use super::*;

mod iter;

impl<Window> CurrentLayout<Window> {
	/// Creates a new [tiled layout] using the given layout `Manager` type parameter.
	///
	/// [tiled layout]: Self::Tiled
	#[inline(always)]
	pub(crate) fn new_tiled<Manager>(x: i32, y: i32, width: u32, height: u32, settings: &LayoutSettings) -> Self
	where
		Manager: TilingLayoutManager<Window>,
	{
		Self::tiled_with_windows::<Manager, std::iter::Empty<Window>>(x, y, width, height, std::iter::empty(), settings)
	}

	/// Creates a new [tiled layout] using the given layout `Manager` type parameter containing the
	/// given `windows`.
	///
	/// [tiled layout]: Self::Tiled
	#[inline]
	pub(crate) fn tiled_with_windows<Manager, Windows>(
		x: i32,
		y: i32,
		width: u32,
		height: u32,
		windows: Windows,
		settings: &LayoutSettings,
	) -> Self
	where
		Manager: TilingLayoutManager<Window>,
		Windows: IntoIterator<Item = Window>,
		Windows::IntoIter: ExactSizeIterator,
	{
		let layout = TilingLayout::new(Manager::orientation(), x, y, width, height, settings);

		Self::Tiled(Box::new(Manager::init(layout, windows)))
	}
}

impl<Window> TilingLayout<Window> {
	/// Creates an empty layout of the given `orientation`.
	#[inline]
	pub(crate) fn new(
		orientation: Orientation,
		x: i32,
		y: i32,
		width: u32,
		height: u32,
		settings: &LayoutSettings,
	) -> Self {
		let padding = settings.window_gap;

		Self {
			x,
			y,

			width,
			height,

			root: Branch::with(
				orientation,
				x + (padding as i32),
				y + (padding as i32),
				width - (2 * padding),
				height - (2 * padding),
			),
		}
	}
}

impl<Window> Borrow<Branch<Window>> for TilingLayout<Window> {
	fn borrow(&self) -> &Branch<Window> {
		&self.root
	}
}

impl<Window> BorrowMut<Branch<Window>> for TilingLayout<Window> {
	fn borrow_mut(&mut self) -> &mut Branch<Window> {
		&mut self.root
	}
}

impl<Window> Deref for TilingLayout<Window> {
	type Target = Branch<Window>;

	fn deref(&self) -> &Self::Target {
		self
	}
}

impl<Window> DerefMut for TilingLayout<Window> {
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
	/// Creates a new branch node with the given `orientation`.
	#[inline(always)]
	pub fn new(orientation: Orientation) -> Self {
		Self::with(orientation, 0, 0, 0, 0)
	}

	/// Creates a new branch node with the given `orientation`, coordinates, and dimensions.
	pub(crate) fn with(orientation: Orientation, x: i32, y: i32, width: u32, height: u32) -> Self {
		Self(Rc::new(RefCell::new(BranchData {
			orientation,

			parent: None,
			children: VecDeque::new(),

			x,
			y,

			width,
			height,

			total_children_primary_dimensions: 0,

			changes_made: None,
		})))
	}
}

impl<Window> Node<Window> {
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

impl<Window> Node<Window> {
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

fn coordinates_changed(changes: &mut Option<NodeChanges>) {
	match changes {
		None => *changes = Some(NodeChanges::Coordinates),
		Some(NodeChanges::Dimensions) => *changes = Some(NodeChanges::Both),

		_ => (),
	}
}

fn dimensions_changed(changes: &mut Option<NodeChanges>) {
	match changes {
		None => *changes = Some(NodeChanges::Dimensions),
		Some(NodeChanges::Coordinates) => *changes = Some(NodeChanges::Both),

		_ => (),
	}
}

impl<Window> Node<Window> {
	#[inline(always)]
	pub fn x(&self) -> i32 {
		match self {
			Self::Leaf(leaf) => leaf.x(),
			Self::Branch(branch) => branch.x(),
		}
	}

	#[inline(always)]
	pub fn y(&self) -> i32 {
		match self {
			Self::Leaf(leaf) => leaf.y(),
			Self::Branch(branch) => branch.y(),
		}
	}

	#[inline(always)]
	pub fn primary_coord(&self, axis: Axis) -> i32 {
		match self {
			Self::Leaf(leaf) => leaf.primary_coord(axis),
			Self::Branch(branch) => branch.primary_coord(axis),
		}
	}

	#[inline(always)]
	pub fn secondary_coord(&self, axis: Axis) -> i32 {
		match self {
			Self::Leaf(leaf) => leaf.secondary_coord(axis),
			Self::Branch(branch) => branch.secondary_coord(axis),
		}
	}

	#[inline(always)]
	pub fn width(&self) -> u32 {
		match self {
			Self::Leaf(leaf) => leaf.width(),
			Self::Branch(branch) => branch.width(),
		}
	}

	#[inline(always)]
	pub fn height(&self) -> u32 {
		match self {
			Self::Leaf(leaf) => leaf.height(),
			Self::Branch(branch) => branch.height(),
		}
	}

	#[inline(always)]
	pub fn primary_dimension(&self, axis: Axis) -> u32 {
		match self {
			Self::Leaf(leaf) => leaf.primary_dimension(axis),
			Self::Branch(branch) => branch.primary_dimension(axis),
		}
	}

	#[inline(always)]
	pub fn secondary_dimension(&self, axis: Axis) -> u32 {
		match self {
			Self::Leaf(leaf) => leaf.secondary_dimension(axis),
			Self::Branch(branch) => branch.secondary_dimension(axis),
		}
	}

	#[inline(always)]
	pub fn set_x(&mut self, x: i32) {
		match self {
			Self::Leaf(leaf) => leaf.set_x(x),
			Self::Branch(branch) => branch.set_x(x),
		}
	}

	#[inline(always)]
	pub fn set_y(&mut self, y: i32) {
		match self {
			Self::Leaf(leaf) => leaf.set_y(y),
			Self::Branch(branch) => branch.set_y(y),
		}
	}

	#[inline(always)]
	pub fn set_primary_coord(&mut self, primary: i32, axis: Axis) {
		match self {
			Self::Leaf(leaf) => leaf.set_primary_coord(primary, axis),
			Self::Branch(branch) => branch.set_primary_coord(primary, axis),
		}
	}

	#[inline(always)]
	pub fn set_secondary_coord(&mut self, secondary: i32, axis: Axis) {
		match self {
			Self::Leaf(leaf) => leaf.set_secondary_coord(secondary, axis),
			Self::Branch(branch) => branch.set_secondary_coord(secondary, axis),
		}
	}

	#[inline(always)]
	pub fn set_width(&mut self, width: u32) {
		match self {
			Self::Leaf(leaf) => leaf.set_width(width),
			Self::Branch(branch) => branch.set_width(width),
		}
	}

	#[inline(always)]
	pub fn set_height(&mut self, height: u32) {
		match self {
			Self::Leaf(leaf) => leaf.set_height(height),
			Self::Branch(branch) => branch.set_height(height),
		}
	}

	#[inline(always)]
	pub fn set_primary_dimension(&mut self, primary: u32, axis: Axis) {
		match self {
			Self::Leaf(leaf) => leaf.set_primary_dimension(primary, axis),
			Self::Branch(branch) => branch.set_primary_dimension(primary, axis),
		}
	}

	#[inline(always)]
	pub fn set_secondary_dimension(&mut self, secondary: u32, axis: Axis) {
		match self {
			Self::Leaf(leaf) => leaf.set_secondary_dimension(secondary, axis),
			Self::Branch(branch) => branch.set_secondary_dimension(secondary, axis),
		}
	}
}

macro_rules! coords_dimensions {
	(
		$(
			impl $Node:ident<$Window:ident>($borrow:ident => $node_changes:expr)
		);*$(;)?
	) => {
		$(
			impl<$Window> $Node<$Window> {
				#[inline(always)]
				pub fn x(&self) -> i32 {
					RefCell::borrow(&self.0).x
				}

				#[inline(always)]
				pub fn y(&self) -> i32 {
					RefCell::borrow(&self.0).y
				}

				#[inline(always)]
				pub fn primary_coord(&self, axis: Axis) -> i32 {
					match axis {
						Axis::Horizontal => RefCell::borrow(&self.0).x,
						Axis::Vertical => RefCell::borrow(&self.0).y,
					}
				}

				#[inline(always)]
				pub fn secondary_coord(&self, axis: Axis) -> i32 {
					match axis {
						Axis::Horizontal => RefCell::borrow(&self.0).y,
						Axis::Vertical => RefCell::borrow(&self.0).x,
					}
				}

				#[inline(always)]
				pub fn width(&self) -> u32 {
					RefCell::borrow(&self.0).width
				}

				#[inline(always)]
				pub fn height(&self) -> u32 {
					RefCell::borrow(&self.0).height
				}

				#[inline(always)]
				pub fn primary_dimension(&self, axis: Axis) -> u32 {
					match axis {
						Axis::Horizontal => RefCell::borrow(&self.0).width,
						Axis::Vertical => RefCell::borrow(&self.0).height,
					}
				}

				#[inline(always)]
				pub fn secondary_dimension(&self, axis: Axis) -> u32 {
					match axis {
						Axis::Horizontal => RefCell::borrow(&self.0).height,
						Axis::Vertical => RefCell::borrow(&self.0).width,
					}
				}

				#[inline(always)]
				pub fn set_x(&mut self, x: i32) {
					let $borrow = RefCell::borrow_mut(&self.0);

					$borrow.x = x;
					coordinates_changed($node_changes);
				}

				#[inline(always)]
				pub fn set_y(&mut self, y: i32) {
					let $borrow = RefCell::borrow_mut(&self.0);

					$borrow.y = y;
					coordinates_changed($node_changes);
				}

				#[inline(always)]
				pub fn set_primary_coord(&mut self, primary: i32, axis: Axis) {
					match axis {
						Axis::Horizontal => self.set_x(primary),
						Axis::Vertical => self.set_y(primary),
					}
				}

				#[inline(always)]
				pub fn set_secondary_coord(&mut self, secondary: i32, axis: Axis) {
					match axis {
						Axis::Horizontal => self.set_y(secondary),
						Axis::Vertical => self.set_x(secondary),
					}
				}

				#[inline(always)]
				pub fn set_width(&mut self, width: u32) {
					let $borrow = RefCell::borrow_mut(&self.0);

					$borrow.width = width;
					dimensions_changed($node_changes);
				}

				#[inline(always)]
				pub fn set_height(&mut self, height: u32) {
					let $borrow = RefCell::borrow_mut(&self.0);

					$borrow.height = height;
					dimensions_changed($node_changes);
				}

				#[inline(always)]
				pub fn set_primary_dimension(&mut self, primary: u32, axis: Axis) {
					match axis {
						Axis::Horizontal => self.set_width(primary),
						Axis::Vertical => self.set_height(primary),
					}
				}

				#[inline(always)]
				pub fn set_secondary_dimension(&mut self, secondary: u32, axis: Axis) {
					match axis {
						Axis::Horizontal => self.set_height(secondary),
						Axis::Vertical => self.set_width(secondary),
					}
				}
			}
		)*
	};
}

coords_dimensions! {
	impl Leaf<Window>(borrow => &mut borrow.changes_made);
	impl Branch<Window>(borrow => &mut borrow.changes_made.get_or_insert_with(BranchChanges::new).other_changes_made);
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
	/// Returns a reference to the node at the given `index`.
	///
	/// If `index` is out of bounds, [`None`] is returned.
	#[inline]
	pub fn get(&self, index: usize) -> Option<&Node<Window>> {
		let borrow = RefCell::borrow(&self.0);

		if index < borrow.children.len() {
			let index = unsafe { self.determine_index(index) };

			Some(&borrow.children[index])
		} else {
			None
		}
	}

	/// Returns a mutable reference to the node at the given `index`.
	///
	/// If `index` is out of bounds, [`None`] is returned.
	#[inline]
	pub fn get_mut(&mut self, index: usize) -> Option<&mut Node<Window>> {
		let borrow = RefCell::borrow_mut(&self.0);

		if index < borrow.children.len() {
			let index = unsafe { self.determine_index(index) };

			Some(&mut borrow.children[index])
		} else {
			None
		}
	}

	/// Returns a reference to the first child in the branch.
	///
	/// If the branch has no children, [`None`] is returned.
	#[inline]
	pub fn first(&self) -> Option<&Node<Window>> {
		let borrow = RefCell::borrow(&self.0);

		match borrow.children.len() {
			0 => None,
			_ => {
				let index = unsafe { self.determine_index(0) };

				Some(&borrow.children[index])
			},
		}
	}

	/// Returns a mutable reference to the first child in the branch.
	///
	/// If the branch has no children, [`None`] is returned.
	#[inline]
	pub fn first_mut(&mut self) -> Option<&mut Node<Window>> {
		let borrow = RefCell::borrow_mut(&self.0);

		match borrow.children.len() {
			0 => None,
			_ => {
				let index = unsafe { self.determine_index(0) };

				Some(&mut borrow.children[index])
			},
		}
	}

	/// Returns a reference to the last child in the branch.
	///
	/// If the branch has no children, [`None`] is returned.
	#[inline]
	pub fn last(&self) -> Option<&Node<Window>> {
		let borrow = RefCell::borrow(&self.0);

		match borrow.children.len() {
			0 => None,
			len => {
				let index = unsafe { self.determine_index(len - 1) };

				Some(&borrow.children[index])
			},
		}
	}

	/// Returns a mutable reference to the last child in the branch.
	///
	/// If the branch has no children, [`None`] is returned.
	#[inline]
	pub fn last_mut(&mut self) -> Option<&mut Node<Window>> {
		let borrow = RefCell::borrow_mut(&self.0);

		match borrow.children.len() {
			0 => None,
			len => {
				let index = unsafe { self.determine_index(len - 1) };

				Some(&mut borrow.children[index])
			},
		}
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
		let insertion_point = additions.partition_point(|i| *i < index);

		// Move the additions after the insertion point forward by one.
		for addition in additions.range_mut(insertion_point..) {
			*addition += 1;
		}

		additions.insert(insertion_point, index);
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

		// No additions need to be moved over, because there are no additions following the pushed
		// node.
	}
}

impl<Window> Branch<Window> {
	/// Removes the [node] at the given `index`, returning [`None`] if `index` is out of bounds.
	///
	/// # See also
	/// The first and last [nodes][node] respectively can be removed with [`pop_front`] and
	/// [`pop_back`].
	///
	/// [node]: Node
	///
	/// [`pop_front`]: Self::pop_front
	/// [`pop_back`]: Self::pop_back
	pub fn remove(&mut self, index: usize) -> Option<Node<Window>> {
		let borrow = RefCell::borrow_mut(&self.0);

		if index < borrow.children.len() {
			let index = unsafe { self.determine_index(index) };

			// Track the removal of the node.
			Self::track_remove(index, Self::get_or_create_additions(&mut borrow.changes_made));

			let mut ret = borrow.children.remove(index);

			if let Some(ret) = ret.as_mut() {
				ret.remove_parent();

				// Record changes to the branch due to the removal.
				// NOTE: this is intentionally not `orientation()`.
				borrow.total_children_primary_dimensions -= ret.primary_dimension(borrow.orientation.axis());
				borrow
					.changes_made
					.get_or_insert_with(BranchChanges::new)
					.other_changes_made = Some(NodeChanges::Both);
			}

			ret
		} else {
			None
		}
	}

	/// Pops the branch node's first child.
	pub fn pop_front(&mut self) -> Option<Node<Window>> {
		let borrow = RefCell::borrow_mut(&self.0);

		if !borrow.children.is_empty() {
			let mut ret = if borrow.orientation.reversed() {
				// Track the pop.
				Self::track_pop_back(
					borrow.children.len() - 1,
					Self::get_or_create_additions(&mut borrow.changes_made),
				);

				borrow.children.pop_back()
			} else {
				// Track the pop.
				Self::track_pop_front(Self::get_or_create_additions(&mut borrow.changes_made));

				borrow.children.pop_front()
			};

			if let Some(ret) = ret.as_mut() {
				ret.remove_parent();

				// Record changes to the branch due to the removal.
				// NOTE: this is intentionally not `orientation()`.
				borrow.total_children_primary_dimensions -= ret.primary_dimension(borrow.orientation.axis());
				borrow
					.changes_made
					.get_or_insert_with(BranchChanges::new)
					.other_changes_made = Some(NodeChanges::Both);
			}

			ret
		} else {
			None
		}
	}

	/// Pop the branch node's last child.
	pub fn pop_back(&mut self) -> Option<Node<Window>> {
		let borrow = RefCell::borrow_mut(&self.0);

		if !borrow.children.is_empty() {
			let mut ret = if borrow.orientation.reversed() {
				// Track the pop.
				Self::track_pop_front(Self::get_or_create_additions(&mut borrow.changes_made));

				borrow.children.pop_front()
			} else {
				// Track the pop.
				Self::track_pop_back(
					borrow.children.len() - 1,
					Self::get_or_create_additions(&mut borrow.changes_made),
				);

				borrow.children.pop_back()
			};

			if let Some(ret) = ret.as_mut() {
				ret.remove_parent();

				// Record changes to the branch due to the removal.
				// NOTE: this is intentionally not `orientation()`.
				borrow.total_children_primary_dimensions -= ret.primary_dimension(borrow.orientation.axis());
				borrow
					.changes_made
					.get_or_insert_with(BranchChanges::new)
					.other_changes_made = Some(NodeChanges::Both);
			}

			ret
		} else {
			None
		}
	}

	/// Inserts the given `node` at the given `index`.
	///
	/// # Panics
	/// Panics if `index` is greater than the number of children in the branch node.
	pub fn insert(&mut self, index: usize, mut node: Node<Window>) {
		let len = RefCell::borrow(&self.0).children.len();

		if index < len {
			let index = unsafe { self.determine_index(index) };

			// Set the node's parent to this branch node.
			node.set_parent(Rc::downgrade(&self.0));

			let borrow = RefCell::borrow_mut(&self.0);

			// Track the insertion.
			Self::track_insert(index, Self::get_or_create_additions(&mut borrow.changes_made));

			borrow.children.insert(index, node);
		} else {
			panic!("the given `index` ({index}) was greater than the number of children ({len})");
		}
	}

	/// Pushes the given `node` to the beginning of the branch node's children.
	pub fn push_front(&mut self, mut node: Node<Window>) {
		// Set the node's parent to this branch node.
		node.set_parent(Rc::downgrade(&self.0));

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
	pub fn push_back(&mut self, mut node: Node<Window>) {
		// Set the node's parent to this branch node.
		node.set_parent(Rc::downgrade(&self.0));

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

impl<Window> TilingLayout<Window> {
	#[inline]
	pub(crate) fn apply_changes<Error>(
		&mut self,
		reconfigure_window: &mut impl FnMut(&Window, Option<(i32, i32)>, Option<(u32, u32)>) -> Result<(), Error>,
		settings: &LayoutSettings,
		settings_changed: bool,
	) -> Result<(), Error> {
		// If the settings have changed, set the other changes made to include both coordinates and
		// dimensions. Changing the layout settings means everything needs to be recalculated.
		if settings_changed {
			RefCell::borrow_mut(&self.root.0)
				.changes_made
				.get_or_insert_with(BranchChanges::new)
				.other_changes_made = Some(NodeChanges::Both);
		}

		self.root.apply_changes(reconfigure_window, settings)
	}
}

impl<Window> Node<Window> {
	#[inline(always)]
	fn apply_changes<Error>(
		&mut self,
		reconfigure_window: &mut impl FnMut(&Window, Option<(i32, i32)>, Option<(u32, u32)>) -> Result<(), Error>,
		settings: &LayoutSettings,
	) -> Result<(), Error> {
		match self {
			Self::Leaf(leaf) => leaf.apply_changes(reconfigure_window),
			Self::Branch(branch) => branch.apply_changes(reconfigure_window, settings),
		}
	}
}

impl<Window> Leaf<Window> {
	fn apply_changes<Error>(
		&self,
		reconfigure_window: &mut impl FnMut(&Window, Option<(i32, i32)>, Option<(u32, u32)>) -> Result<(), Error>,
	) -> Result<(), Error> {
		let borrow = RefCell::borrow_mut(&self.0);

		let (coordinates, dimensions) = match mem::take(&mut borrow.changes_made) {
			// If no changes have been made, then don't call `reconfigure_window`.
			None => return Ok(()),

			Some(NodeChanges::Coordinates) => (Some((borrow.x, borrow.y)), None),
			Some(NodeChanges::Dimensions) => (None, Some((borrow.width, borrow.height))),

			Some(NodeChanges::Both) => (Some((borrow.x, borrow.y)), Some((borrow.width, borrow.height))),
		};

		reconfigure_window(&borrow.window, coordinates, dimensions)
	}
}

impl<Window> Branch<Window> {
	fn apply_changes<Error>(
		&mut self,
		reconfigure_window: &mut impl FnMut(&Window, Option<(i32, i32)>, Option<(u32, u32)>) -> Result<(), Error>,
		settings: &LayoutSettings,
	) -> Result<(), Error> {
		let borrow = RefCell::borrow_mut(&self.0);

		match mem::take(&mut borrow.changes_made) {
			// No changes have been made to the branch node directly, so just apply any changes made
			// to the branch's children.
			None => {
				for node in self {
					node.apply_changes(reconfigure_window, settings)?;
				}
			},

			// Changes have been made to the branch node itself.
			Some(BranchChanges {
				new_orientation,
				additions,
				mut other_changes_made,
			}) => {
				// The old axis of the group, before any orientation change.
				let old_axis = borrow.orientation.axis();
				// Whether the direction (whether it's reversed or not) and axis of the orientation
				// have changed.
				let (direction_changed, axis_changed) = match &new_orientation {
					None => (false, false),

					Some(orientation) => (
						orientation.reversed() != borrow.orientation.reversed(),
						orientation.axis() != old_axis,
					),
				};

				if axis_changed || !additions.is_empty() {
					match &other_changes_made {
						None | Some(NodeChanges::Coordinates) => other_changes_made = Some(NodeChanges::Both),

						_ => (),
					}
				} else if direction_changed {
					match &other_changes_made {
						None => other_changes_made = Some(NodeChanges::Coordinates),

						_ => (),
					}
				}

				// Apply the change in orientation, if any.
				if let Some(orientation) = new_orientation {
					borrow.orientation = orientation;
				}

				let new_axis = borrow.orientation.axis();
				let new_reversed = borrow.orientation.reversed();

				let (primary_dimension, secondary_dimension) =
					(self.primary_dimension(new_axis), self.secondary_dimension(new_axis));
				let (primary_coord, secondary_coord) = (self.primary_coord(new_axis), self.secondary_coord(new_axis));

				match other_changes_made {
					// No further changes need to be made.
					None => (),

					// Only coordinates need to be updated.
					Some(NodeChanges::Coordinates) => {
						if new_reversed {
							// The orientation is reversed, so do the coordinates from right to left.

							let mut node_primary_coord = primary_coord + (primary_dimension as i32);

							for node in self {
								node_primary_coord -= node.primary_dimension(new_axis) as i32;

								node.set_primary_coord(node_primary_coord, new_axis);
								node.set_secondary_coord(secondary_coord, new_axis);

								node.apply_changes(reconfigure_window, settings)?;
							}
						} else {
							// The orientation is not reversed, so do the coordinates from left to right.

							let mut node_primary_coord = primary_coord;

							for node in self {
								node.set_primary_coord(node_primary_coord, new_axis);
								node.set_secondary_coord(secondary_coord, new_axis);

								node.apply_changes(reconfigure_window, settings)?;

								node_primary_coord += node.primary_dimension(new_axis) as i32;
							}
						}
					},

					// Both coordinates and dimensions need to be updated.
					_ => {
						// Reconfigures the given `node` to the given coordinates and dimensions.
						let mut configure_node =
							|node: &mut Node<Window>, mut node_primary_coord, node_primary_dimension| {
								// If the orientation is reversed, then reverse the coordinates.
								if new_reversed {
									node_primary_coord = (primary_dimension as i32)
										- node_primary_coord - (node_primary_dimension as i32);
								}

								node.set_primary_coord(primary_coord + node_primary_coord, new_axis);
								node.set_secondary_coord(secondary_coord, new_axis);

								node.set_primary_dimension(node_primary_dimension, new_axis);
								node.set_secondary_dimension(secondary_dimension, new_axis);

								node.apply_changes(reconfigure_window, settings)
							};

						let new_children_len = (borrow.children.len() + additions.len()) as u32;
						// The total window gap between the child nodes.
						let total_gap = if new_children_len == 0 {
							0
						} else {
							(new_children_len - 1) * settings.window_gap
						};
						// The primary dimension of new additions.
						let addition_primary_dimension = if new_children_len == 0 {
							primary_dimension
						} else {
							(primary_dimension - total_gap) / new_children_len
						};
						// The old total primary dimensions of existing children.
						let old_existing_children_total_primary_dimensions =
							borrow.total_children_primary_dimensions as u64;
						// The target total primary dimensions of existing children for rescaling.
						//
						// `u64` is used because we will be multiplying two `u32` values
						// (`u32::MAX * u32::MAX = u64::MAX`).
						let new_existing_children_total_primary_dimensions = (primary_dimension
							- (addition_primary_dimension * (additions.len() as u32))
							- total_gap) as u64;

						let mut new_total_children_primary_dimensions = 0;

						let mut additions = additions.into_iter();
						let mut next_addition = additions.next();

						for (index, node) in borrow.children.iter_mut().enumerate() {
							let gap = (settings.window_gap as i32) * (index as i32);
							let node_primary_coord = (new_total_children_primary_dimensions as i32) + gap;

							// If `node` is an addition, configure it and `continue` to the next node.
							if let Some(addition) = next_addition {
								if index == addition {
									// Configure the node.
									configure_node(node, node_primary_coord, addition_primary_dimension)?;

									next_addition = additions.next();

									new_total_children_primary_dimensions += addition_primary_dimension;
									continue;
								}
							}

							// If `node` is an existing node, not a new addition, rescale it.

							let old_node_primary_dimension = node.primary_dimension(old_axis) as u64;
							// Rescale the node's primary dimension.
							let new_node_primary_dimension: u32 = ((old_node_primary_dimension
								* new_existing_children_total_primary_dimensions)
								/ old_existing_children_total_primary_dimensions)
								.shrink();

							// Configure the node.
							configure_node(node, node_primary_coord, new_node_primary_dimension)?;

							new_total_children_primary_dimensions += new_node_primary_dimension;
						}

						borrow.total_children_primary_dimensions = new_total_children_primary_dimensions;
					},
				}
			},
		}

		Ok(())
	}
}
