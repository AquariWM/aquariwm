// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::mem;

use futures::StreamExt;

use super::*;

impl<Window> GroupNode<Window> {
	/// Rotates the group's [`orientation`] by the given number of `rotations`.
	///
	/// A positive number of rotations will rotate the [`orientation`] clockwise, while a negative
	/// number of rotations will rotate the [`orientation`] counter-clockwise.
	///
	/// # See also
	/// - [`set_orientation`](Self::set_orientation)
	///
	/// [`orientation`]: Self::orientation()
	pub fn rotate_by(&mut self, rotations: i32) {
		let current = match self.orientation {
			Orientation::LeftToRight => 0,
			Orientation::TopToBottom => 1,
			Orientation::RightToLeft => 2,
			Orientation::BottomToTop => 3,
		};

		let new = match (current + rotations).div_euclid(4) {
			0 => Orientation::LeftToRight,
			1 => Orientation::TopToBottom,
			2 => Orientation::RightToLeft,
			3 => Orientation::BottomToTop,

			_ => unreachable!(".div_euclid(4) returns a value within 0..4"),
		};

		self.set_orientation(new);
	}

	/// Returns the group's [orientation].
	///
	/// # See also
	/// - [`set_orientation`](Self::set_orientation)
	/// - [`rotate_by`](Self::rotate_by)
	///
	/// [orientation]: Orientation
	// NOTE: This will return the `new_orientation`	if it is set - for the current orientation
	//       before that is applied, use the `self.orientation` field.
	pub fn orientation(&self) -> Orientation {
		if let Some(orientation) = self.new_orientation {
			orientation
		} else {
			self.orientation
		}
	}

	/// Sets the group's [`orientation`].
	///
	/// # See also
	/// - [`rotate_by`](Self::rotate_by)
	///
	/// [`orientation`]: Self::orientation()
	pub fn set_orientation(&mut self, new: Orientation) {
		self.new_orientation = Some(new);
	}
}

impl<Window> GroupNode<Window> {
	/// Removes the [node] at the given `index` from the group.
	///
	/// [node]: Node
	pub fn remove(&mut self, index: usize) -> Node<Window> {
		let node = self.nodes.remove(index);
		self.track_remove(index);

		let size_fn = Node::size_fn(self.orientation.axis());
		self.total_removed_primary += size_fn(&node);

		node
	}

	/// Pushes a new [window node] with the given `window` to the end of the group.
	///
	/// [window node]: WindowNode
	pub fn push_window(&mut self, window: Window) {
		self.push_node(Node::new_window(window, 0, 0));
	}

	/// Inserts a new [window node] with the given `window` at the given `index` in the group.
	///
	/// [window node]: WindowNode
	pub fn insert_window(&mut self, index: usize, window: Window) {
		self.nodes.insert(index, Node::new_window(window, 0, 0));
		self.track_insert(index);
	}

	/// Pushes a new [group node] of the given `orientation` to the end of the group.
	///
	/// [group node]: GroupNode
	pub fn push_group(&mut self, orientation: Orientation) {
		self.push_node(Node::new_group(orientation, 0, 0));
	}

	/// Inserts a new [group node] of the given `orientation` at the given `index` in the group.
	///
	/// [group node]: GroupNode
	pub fn insert_group(&mut self, index: usize, orientation: Orientation) {
		self.nodes.insert(index, Node::new_group(orientation, 0, 0));
		self.track_insert(index);
	}

	fn push_node(&mut self, node: Node<Window>) {
		if !self.orientation.reversed() {
			// The orientation is not reversed; we push to the end of the list as usual.

			self.nodes.push(node);
			self.track_push();
		} else {
			// The orientation is reversed; we push to the front of the list to give the impression
			// we are pushing to the back in the non-reversed orientation equivalent.

			self.nodes.insert(0, node);
			self.track_insert(0);
		}
	}

	/// Update `additions` to reflect a node being inserted at `index`.
	fn track_insert(&mut self, index: usize) {
		let insertion_point = self.additions.partition_point(|&i| i < index);
		self.additions.insert(insertion_point, index);

		// Move following additions over by 1.
		for addition in &mut self.additions[(insertion_point + 1)..] {
			*addition += 1;
		}
	}

	/// Update `additions` to reflect a node being pushed to the end of `nodes`.
	fn track_push(&mut self) {
		let index = self.len() - 1;

		// If the node has been pushed to the end, then it must have the greatest index.
		self.additions.push(index);

		// There will be no additions following it to move over, as it was pushed to the end.
	}

	/// Update `additions` to reflect the removal of a node at `index`.
	fn track_remove(&mut self, index: usize) {
		let shifted_additions = match self.additions.binary_search(&index) {
			// An addition we were tracking was removed.
			Ok(addition) => {
				self.additions.remove(addition);

				addition..
			},

			// The removed node was not an addition we were tracking.
			Err(removal_point) => removal_point..,
		};

		// Move following additions back by 1.
		for addition in &mut self.additions[shifted_additions] {
			*addition -= 1;
		}
	}
}

impl<Window> GroupNode<Window> {
	/// Applies the changes made by the [layout manager].
	///
	/// `resize_window` is a function that resizes the given window based on the given [primary] and
	/// [secondary] dimensions.
	///
	/// [layout manager]: TilingLayoutManager
	///
	/// [primary]: Node::primary
	/// [secondary]: Node::secondary
	// TODO: take changes in the group's dimensions into account. the group should have `new_width`
	//     : and `new_height` fields.
	fn apply_changes<ResizeRet>(&mut self, resize_window: impl FnMut(&Window, Option<u32>, Option<u32>) -> ResizeRet) {
		// The old axis of the group, before any orientation change.
		let old_axis = self.orientation.axis();
		// Apply the change in orientation, if it is to be changed.
		if let Some(new) = mem::take(&mut self.new_orientation) {
			self.orientation = new;
		}

		// The order of dimensions used for nodes depends on the orientation of the group. The first
		// dimension, `primary`, is the dimension that is affected by the node's size within the
		// group, while the second dimension, `secondary`, is the dimension that is only affected by
		// the group's size.
		//
		// ┏━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┓
		// ┃           Dimensions (primary, secondary)             ┃
		// ┡━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┯━━━━━━━━━━━━━━━━━━━━━┩
		// │           Horizontal            │       Vertical      │
		// ├╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┤
		// │                                 │         ⟵secondary⟶ │
		// │                                 │       ↑ ╔═════════╗ │
		// │                                 │ primary ║  Node   ║ │
		// │           ⟵primary⟶             │       ↓ ╚═════════╝ │
		// │         ↑ ╔═══════╗╔════╗╔════╗ │         ╔═════════╗ │
		// │ secondary ║ Node  ║║    ║║    ║ │         ║         ║ │
		// │         ↓ ╚═══════╝╚════╝╚════╝ │         ╚═════════╝ │
		// │                                 │         ╔═════════╗ │
		// │                                 │         ║         ║ │
		// │                                 │         ╚═════════╝ │
		// └─────────────────────────────────┴─────────────────────┘

		let old_total_node_primary = self.total_node_primary - mem::take(&mut self.total_removed_primary);

		let (group_primary, group_secondary) = (self.primary(), self.secondary());
		// Set a node's dimensions and call `resize_window` if it is a window.
		let set_node_dimensions = |node, primary, secondary| {
			if let Some(primary) = primary {
				node.set_primary(primary);
			}
			if let Some(secondary) = secondary {
				node.set_secondary(secondary);
			}

			if let Node::Window(WindowNode { window, .. }) = &node {
				resize_window(window, primary, secondary);
			}
		};

		let additions = mem::take(&mut self.additions);
		// The size of new additions.
		let new_primary = group_primary / self.nodes.len();
		let mut new_total_node_primary = new_primary * additions.len();
		// The new total size for the existing nodes to be resized to fit within.
		let rescaling_primary = group_primary - new_total_node_primary;

		let mut additions = additions.into_iter();
		let mut next_addition = additions.next();

		// Resize all the nodes appropriately.
		for index in 0..self.nodes.len() {
			let node = &mut self.nodes[index];

			// If `node` is an addition, resize it with the new size.
			if let Some(addition) = next_addition {
				if index == addition {
					set_node_dimensions(node, Some(new_primary), Some(group_secondary));

					next_addition = additions.next();
					continue;
				}
			}

			// `node` is not an addition: rescale it.

			// Determine the rescaled size.
			let old_primary = node.primary(old_axis);
			let rescaled_primary = (old_primary * rescaling_primary) / old_total_node_primary;

			set_node_dimensions(
				node,
				Some(rescaled_primary),
				// If the axis has changed, the node's secondary has to be updated too.
				(old_axis != self.orientation.axis()).then_some(group_secondary),
			);
			new_total_node_primary += rescaled_primary;
		}

		self.total_node_primary = new_total_node_primary;
	}
}
