// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::mem;

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
		self.set_orientation(self.orientation().rotated_by(rotations));
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
	pub const fn orientation(&self) -> Orientation {
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
	pub fn remove(&mut self, index: usize) -> Option<Node<Window>> {
		let node = self.children.remove(index);

		if let Some(node) = self.children.remove(index) {
			self.track_remove(index);

			self.total_removed_primary += node.primary(self.orientation.axis());
		}

		node
	}

	/// Pushes a new [window node] with the given `window` to the end of the group.
	///
	/// [window node]: WindowNode
	pub fn push_window_back(&mut self, window: Window) {
		self.push_node_back(Node::new_window(window, 0, 0));
	}

	pub fn push_windows_back(&mut self, windows: impl IntoIterator<Item = Window>) {
		self.push_nodes_back(windows.into_iter().map(|window| Node::new_window(window, 0, 0)));
	}

	/// Inserts a new [window node] with the given `window` at the given `index` in the group.
	///
	/// [window node]: WindowNode
	pub fn insert_window(&mut self, index: usize, window: Window) {
		self.children.insert(index, Node::new_window(window, 0, 0));
		self.track_insert(index);
	}

	pub fn insert_windows(&mut self, index: usize, windows: impl IntoIterator<Item = Window>) {
		self.insert_nodes(index, windows.into_iter().map(|window| Node::new_window(window, 0, 0)));
	}

	/// Pushes a new [group node] of the given `orientation` to the end of the group.
	///
	/// [group node]: GroupNode
	pub fn push_group_back(&mut self, orientation: Orientation) {
		self.push_node_back(Node::new_group(orientation, 0, 0));
	}

	/// Pushes a new [group node] of the given `orientation` to the end of the group, then
	/// initialises it with the given `init` function.
	///
	/// [group node]: GroupNode
	pub fn push_group_back_with(&mut self, orientation: Orientation, init: impl FnOnce(&mut GroupNode<Window>)) {
		let index = self.push_node_back(Node::new_group(orientation, 0, 0));

		match &mut self.children[index] {
			Node::Group(group) => init(group),
			Node::Window(_) => unreachable!("we know the this node is a group, because we just added it"),
		}
	}

	pub fn push_groups_back(&mut self, orientations: impl IntoIterator<Item = Orientation>) {
		self.push_nodes_back(
			orientations
				.into_iter()
				.map(|orientation| Node::new_group(orientation, 0, 0)),
		);
	}

	/// Inserts a new [group node] of the given `orientation` at the given `index` in the group.
	///
	/// [group node]: GroupNode
	#[inline]
	pub fn insert_group(&mut self, index: usize, orientation: Orientation) {
		self.insert_node(index, Node::new_group(orientation, 0, 0));
	}

	/// Inserts a new [group node] of the given `orientation` at the given `index` in the group,
	/// then initialises it with the given `init` function.
	///
	/// [group node]: GroupNode
	pub fn insert_group_with(
		&mut self,
		index: usize,
		orientation: Orientation,
		init: impl FnOnce(&mut GroupNode<Window>),
	) {
		let index = self.insert_node(index, Node::new_group(orientation, 0, 0));

		match &mut self.children[index] {
			Node::Group(group) => init(group),
			Node::Window(_) => unreachable!("we know this node is a group, because we just added it"),
		}
	}

	pub fn insert_groups(&mut self, index: usize, orientations: impl IntoIterator<Item = Orientation>) {
		self.insert_nodes(
			index,
			orientations
				.into_iter()
				.map(|orientation| Node::new_group(orientation, 0, 0)),
		);
	}

	/// Push the given `node` to the list, and return the index it was pushed to.
	///
	/// The index is affected by whether this group is [reversed] or not.
	///
	/// [reversed]: Orientation::reversed
	fn push_node_back(&mut self, node: Node<Window>) -> usize {
		if !self.orientation().reversed() {
			// The orientation is not reversed; we push to the end of the list as usual.

			let index = self.children.len();

			self.children.push_back(node);
			self.track_push_back();

			index
		} else {
			// The orientation is reversed; we push to the front of the list to give the impression
			// we are pushing to the back in the non-reversed orientation equivalent.

			const INDEX: usize = 0;

			self.children.push_front(node);
			self.track_push_front();

			INDEX
		}
	}

	fn push_node_front(&mut self, node: Node<Window>) -> usize {
		if !self.orientation().reversed() {
			const INDEX: usize = 0;

			self.children.push_front(node);
			self.track_push_front();

			INDEX
		} else {
			let index = self.children.len();

			self.children.push_back(node);
			self.track_push_back();

			index
		}
	}

	fn push_nodes_back(&mut self, nodes: impl IntoIterator<Item = Node<Window>>) {
		let nodes = nodes.into_iter();

		let (min_nodes, _) = nodes.size_hint();
		self.children.reserve(min_nodes);

		if !self.orientation().reversed() {
			for node in nodes {
				self.children.push_back(node);
				self.track_push_back();
			}
		} else {
			for node in nodes {
				self.children.push_front(node);
				self.track_push_back();
			}
		}
	}

	fn push_nodes_front(&mut self, nodes: impl IntoIterator<Item = Node<Window>>) {
		let nodes = nodes.into_iter();

		let (min_nodes, _) = nodes.size_hint();
		self.children.reserve(min_nodes);

		if !self.orientation().reversed() {
			for node in nodes {
				self.children.push_front(node);
			}
		} else {
			for node in nodes {
				self.children.push_back(node);
			}
		}
	}

	/// Insert the given `node` to the list, and return the index it was pushed to.
	///
	/// The index is affected by whether this group is [reversed] or not.
	///
	/// [reversed]: Orientation::reversed
	fn insert_node(&mut self, index: usize, node: Node<Window>) -> usize {
		if !self.orientation().reversed() {
			// The orientation is not reversed; we insert as usual.

			self.children.insert(index, node);
			self.track_insert(index);

			index
		} else {
			// The orientation is reversed; we insert at the `index` _counting back from the end_ to give the
			// impression we are inserting at `index` counting from the front in the non-reversed orientation
			// equivalent.

			let last = self.children.len() - 1;
			let index = last - index;

			self.children.insert(index, node);
			self.track_insert(index);

			index
		}
	}

	fn insert_nodes(&mut self, index: usize, nodes: impl IntoIterator<Item = Node<Window>>) {
		let nodes = nodes.into_iter();

		let (min_nodes, _) = nodes.size_hint();
		self.children.reserve(min_nodes);

		if !self.orientation().reversed() {
			for (index, node) in nodes.enumerate().map(|(i, node)| (index + i, node)) {
				self.children.insert(index, node);
				self.track_insert(index);
			}
		} else {
			let last = self.children.len() - 1;
			let index = last - index;

			for node in nodes {
				self.children.insert(index, node);
				self.track_insert(index);
			}
		}
	}

	/// Update `additions` to reflect a node being inserted at `index`.
	fn track_insert(&mut self, index: usize) {
		let insertion_point = self.additions.partition_point(|&i| i < index);
		self.additions.insert(insertion_point, index);

		// Move following additions over by 1.
		for addition in &mut self.additions.make_contiguous()[(insertion_point + 1)..] {
			*addition += 1;
		}
	}

	/// Update `additions` to reflect a node being pushed to the end of `nodes`.
	#[inline]
	fn track_push_back(&mut self) {
		let index = self.children.len() - 1;

		// If the node has been pushed to the end, then it must have the greatest index.
		self.additions.push_back(index);

		// There will be no additions following it to move over, as it was pushed to the end.
	}

	fn track_push_front(&mut self) {
		// Move every existing addition over by 1; if the node has been pushed to the front, it has the
		// lowest index.
		for addition in &mut self.additions {
			*addition += 1;
		}

		// Push the addition to the front.
		self.additions.push_front(0);
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
		for addition in &mut self.additions.make_contiguous()[shifted_additions] {
			*addition -= 1;
		}
	}

	fn track_pop_back(&mut self) {
		if !self.additions.is_empty() {
			// The index that the node was popped from.
			let index = self.children.len();

			// If it was one of our own additions, pop that addition.
			if self.additions[self.additions.len() - 1] == index {
				self.additions.pop_back();
			}
		}
	}

	fn track_pop_front(&mut self) {
		if !self.additions.is_empty() {
			// The index that the node was popped from.
			const INDEX: usize = 0;

			// If it was one of our own additions, pop that addition.
			if self.additions[0] == INDEX {
				self.additions.pop_front();
			}
		}

		// Move all the additions back by one.
		for addition in &mut self.additions {
			*addition -= 1;
		}
	}
}

impl<Window> GroupNode<Window> {
	/// Returns whether any changes have been made by the [layout manager] to this group (directly
	/// or indirectly).
	///
	/// [layout manager]: TilingLayoutManager
	fn changes_made(&self) -> bool {
		!self.additions.is_empty()
			|| self.total_removed_primary != 0
			|| self.new_orientation.is_some()
			|| self.new_width.is_some()
			|| self.new_height.is_some()
	}

	/// Applies the changes made by the [layout manager].
	///
	/// `resize_window` is a function that resizes the given window based on the given [primary] and
	/// [secondary] dimensions.
	///
	/// [layout manager]: TilingLayoutManager
	///
	/// [primary]: Node::primary
	/// [secondary]: Node::secondary
	fn apply_changes<Error, ResizeWindowFn>(&mut self, mut resize_window: ResizeWindowFn) -> Result<(), Error>
	where
		ResizeWindowFn: FnMut(&Window, u32, u32) -> Result<(), Error>,
		ResizeWindowFn: Clone,
	{
		// If no changes have been made to this group, apply all the child groups' changes and return.
		if !self.changes_made() {
			let groups = self.children.iter_mut().filter_map(|node| match node {
				Node::Group(group) => Some(group),

				Node::Window(_) => None,
			});

			for group in groups {
				group.apply_changes(resize_window.clone())?;
			}

			return Ok(());
		}

		let additions = mem::take(&mut self.additions);
		let total_removed_primary = mem::take(&mut self.total_removed_primary);

		let new_orientation = mem::take(&mut self.new_orientation);
		let new_width = mem::take(&mut self.new_width);
		let new_height = mem::take(&mut self.new_height);

		// The old axis of the group, before any orientation change.
		let old_axis = self.orientation.axis();

		// Apply the change in orientation, if it is to be changed.
		if let Some(orientation) = new_orientation {
			self.orientation = orientation;
		}
		// Apply the change in width, if any.
		if let Some(width) = new_width {
			self.width = width;
		}
		// Apply the change in height, if any.
		if let Some(height) = new_height {
			self.height = height;
		}

		let new_axis = self.orientation.axis();

		let old_total_node_primary = self.total_node_primary - total_removed_primary;

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

		let (group_primary, group_secondary) = (self.primary(), self.secondary());
		// Set a node's dimensions and call `resize_window` if it is a window.
		let mut set_node_dimensions = |node: &mut Node<Window>, primary, secondary| {
			node.set_primary(primary, new_axis);
			node.set_secondary(secondary, new_axis);

			match node {
				Node::Group(group) => group.apply_changes(resize_window.clone()),
				Node::Window(WindowNode { window, .. }) => resize_window(window, primary, secondary),
			}
		};

		let new_nodes_len = (self.children.len() + self.additions.len()) as u32;
		// The size of new additions.
		let new_primary = if new_nodes_len == 0 {
			group_primary
		} else {
			group_primary / new_nodes_len
		};
		let mut new_total_node_primary = new_primary * (additions.len() as u32);
		// The new total size for the existing nodes to be resized to fit within.
		let rescaling_primary = group_primary - new_total_node_primary;

		let mut additions = additions.into_iter();
		let mut next_addition = additions.next();

		// Resize all the nodes appropriately.
		for index in 0..self.children.len() {
			let node = &mut self.children[index];

			// If `node` is an addition, resize it with the new size.
			if let Some(addition) = next_addition {
				if index == addition {
					set_node_dimensions(node, new_primary, group_secondary)?;

					next_addition = additions.next();
					continue;
				}
			}

			// `node` is not an addition: rescale it.

			// Determine the rescaled size.
			let old_primary = node.primary(old_axis);
			let rescaled_primary = (old_primary * rescaling_primary) / old_total_node_primary;

			set_node_dimensions(node, rescaled_primary, group_secondary)?;

			new_total_node_primary += rescaled_primary;
		}

		self.total_node_primary = new_total_node_primary;

		Ok(())
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn group_orientations() {
		const INITIAL_ORIENTATION: Orientation = Orientation::LeftToRight;
		const ROTATIONS: i32 = 6;
		const NEW_ORIENTATION: Orientation = Orientation::RightToLeft;

		let mut group: GroupNode<()> = GroupNode::new(INITIAL_ORIENTATION, 0, 0);

		assert_eq!(group.orientation, INITIAL_ORIENTATION);
		assert_eq!(group.new_orientation, None);
		assert_eq!(group.orientation(), INITIAL_ORIENTATION);

		group.rotate_by(ROTATIONS);

		assert_eq!(group.orientation, INITIAL_ORIENTATION);
		assert_eq!(group.new_orientation, Some(NEW_ORIENTATION));
		assert_eq!(group.orientation(), NEW_ORIENTATION);

		// Apply the change in orientation.
		group
			.apply_changes(|_window, _primary, _secondary| -> Result<(), ()> { Ok(()) })
			.unwrap();

		assert_eq!(group.orientation, NEW_ORIENTATION);
		assert_eq!(group.new_orientation, None);
		assert_eq!(group.orientation(), NEW_ORIENTATION);
	}

	#[test]
	fn push_windows() {
		const WINDOWS: [u32; 5] = [1, 2, 3, 4, 5];

		let non_reversed_nodes = VecDeque::from(WINDOWS.map(|window| Node::new_window(window, 0, 0)));
		let reversed_nodes: VecDeque<_> = non_reversed_nodes.iter().cloned().rev().collect();

		// Test a non-reversed group.
		let mut non_reversed_group: GroupNode<u32> = GroupNode::new(Orientation::LeftToRight, 0, 0);
		assert_eq!(non_reversed_group.children, VecDeque::new());

		non_reversed_group.push_windows_back(WINDOWS);
		assert_eq!(non_reversed_group.children, non_reversed_nodes);

		// Test a reversed group.
		let mut reversed_group: GroupNode<u32> = GroupNode::new(Orientation::RightToLeft, 0, 0);

		reversed_group.push_windows_back(WINDOWS);
		assert_eq!(reversed_group.children, reversed_nodes);
	}

	#[test]
	fn resize_additions() {
		// No-op resize_window function to pass to `apply_changes`.
		const fn resize_window(_window: &u32, _primary: u32, _secondary: u32) -> Result<(), ()> {
			Ok(())
		}

		const GROUP_WIDTH: u32 = 3000;
		const GROUP_HEIGHT: u32 = 1000;

		// The width of each node after three nodes have been added.
		const THREE_NODES_WIDTH: u32 = GROUP_WIDTH / 3;

		let mut group: GroupNode<u32> = GroupNode::new(Orientation::LeftToRight, GROUP_WIDTH, GROUP_HEIGHT);

		group.push_window_back(1);

		assert!(matches!(
			&group[0],
			Node::Window(WindowNode {
				width: 0,
				height: 0,
				..
			})
		));

		// Resize the added window.
		group.apply_changes(resize_window).unwrap();

		assert!(
			matches!(
				&group[0],
				Node::Window(WindowNode {
					width: GROUP_WIDTH,
					height: GROUP_HEIGHT,
					..
				})
			),
			"node = {:?}",
			&group[0]
		);

		group.push_windows_back([2, 3]);

		// Resize the existing window and two added windows.
		group.apply_changes(resize_window).unwrap();

		for node in group {
			assert!(
				matches!(
					&node,
					Node::Window(WindowNode {
						width: THREE_NODES_WIDTH,
						height: GROUP_HEIGHT,
						..
					})
				),
				"node = {:?}",
				node
			);
		}
	}
}
