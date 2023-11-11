// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::mem;

use futures::StreamExt;

use super::*;

impl<Window> GroupNode<Window> {
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

	pub fn set_orientation(&mut self, new: Orientation) {
		// Exclusive OR is used because if the axis has already been changed, then changing it again
		// means it is changed back to what it was.
		self.axis_changed ^= self.orientation.axis() != new.axis();

		self.orientation = new;
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
		self.total_removed_size += size_fn(&node);

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
	fn apply_changes<ResizeRet>(&mut self, resize_window: impl FnMut(&Window, Option<u32>, Option<u32>) -> ResizeRet) {
		let additions = mem::take(&mut self.additions);
		let total_removed_size = mem::take(&mut self.total_removed_size);
		// TODO: take `axis_changed` into account - requires modifying how `total_removed_size` is
		//     : determined, as what 'size' means is going to change at the point the axis changes.
		//     : could just track the removed width and height or something...
		//
		// let axis_changed = mem::take(&mut self.axis_changed);

		let total_node_size = self.total_node_size - total_removed_size;

		let (group_size, group_other) = (self.size(), self.other());
		let set_node_dimensions = |node, width, height| {
			if let Some(width) = width {
				node.set_width(width);
			}
			if let Some(height) = height {
				node.set_height(height);
			}

			if let Node::Window(WindowNode { window, .. }) = &node {
				resize_window(window, width, height);
			}
		};
		let (node_size, set_node_dimensions) = match self.orientation.axis() {
			Axis::Horizontal => (Node::width, |node, size, other| {
				set_node_dimensions(node, size, other);
			}),

			Axis::Vertical => (Node::height, |node, size, other| {
				set_node_dimensions(node, other, size);
			}),
		};

		// The size of new additions.
		let new_size = group_size / self.nodes.len();
		let mut new_total_node_size = new_size * additions.len();
		// The new total size for the existing nodes to be resized to fit within.
		let rescaling_size = group_size - new_total_node_size;

		let mut additions = additions.into_iter();
		let mut next_addition = additions.next();

		// Resize all the nodes appropriately.
		for index in 0..self.nodes.len() {
			let node = &mut self.nodes[index];

			// If `node` is an addition, resize it with the new size.
			if let Some(addition) = next_addition {
				if index == addition {
					set_node_dimensions(node, Some(new_size), Some(group_other));

					next_addition = additions.next();
					continue;
				}
			}

			// `node` is not an addition: rescale it.

			// Determine the rescaled size.
			let rescaled_size = (node_size(node) * rescaling_size) / total_node_size;

			set_node_dimensions(node, Some(rescaled_size), None);
			new_total_node_size += rescaled_size;
		}

		self.total_node_size = new_total_node_size;
	}
}
