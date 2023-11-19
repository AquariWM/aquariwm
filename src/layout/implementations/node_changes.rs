// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::mem;

use truncate_integer::Shrink;

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
		if index < self.children.len() {
			let index = if !self.orientation.reversed() {
				index
			} else {
				let last = self.children.len() - 1;
				last - index
			};

			let node = self.children.remove(index);

			if let Some(node) = &node {
				self.track_remove(index);

				self.total_removed_primary += node.primary(self.orientation.axis());
			}

			node
		} else {
			None
		}
	}

	/// Removes the [node] at the end of the group.
	///
	/// [node]: Node
	pub fn pop_back(&mut self) -> Option<Node<Window>> {
		match self.children.len() {
			// `children` is empty
			0 => None,
			// `children` is not empty
			_ => {
				if !self.orientation.reversed() {
					let node = self.children.pop_back();

					if node.is_some() {
						self.track_pop_back();
					}

					node
				} else {
					let node = self.children.pop_front();

					if node.is_some() {
						self.track_pop_front();
					}

					node
				}
			},
		}
	}

	/// Removes the [node] at the front of the group.
	///
	/// [node]: Node
	pub fn pop_front(&mut self) -> Option<Node<Window>> {
		match self.children.len() {
			// `children` is empty
			0 => None,
			// `children` is not empty
			_ => {
				if !self.orientation.reversed() {
					let node = self.children.pop_front();

					if node.is_some() {
						self.track_pop_front();
					}

					node
				} else {
					let node = self.children.pop_back();

					if node.is_some() {
						self.track_pop_back();
					}

					node
				}
			},
		}
	}

	/// Pushes a new [window node] with the given `window` to the end of the group.
	///
	/// [window node]: WindowNode
	#[inline]
	pub fn push_window_back(&mut self, window: Window) {
		self.push_node_back(Node::new_window(window));
	}

	/// Pushes new [window nodes] of the given `windows` to the end of the group.
	///
	/// [window nodes]: WindowNode
	#[inline]
	pub fn push_windows_back(&mut self, windows: impl IntoIterator<Item = Window>) {
		self.push_nodes_back(windows.into_iter().map(Node::new_window));
	}

	/// Pushes a new [window node] with the given `window` to the beginning of the group.
	///
	/// [window node]: WindowNode
	#[inline]
	pub fn push_window_front(&mut self, window: Window) {
		self.push_node_front(Node::new_window(window));
	}

	/// Pushes new [window nodes] of the given `windows` to the beginning of the group.
	///
	/// [window nodes]: WindowNode
	#[inline]
	pub fn push_windows_front(&mut self, windows: impl IntoIterator<Item = Window>) {
		self.push_nodes_front(windows.into_iter().map(Node::new_window));
	}

	/// Inserts a new [window node] with the given `window` at the given `index` in the group.
	///
	/// [window node]: WindowNode
	#[inline]
	pub fn insert_window(&mut self, index: usize, window: Window) {
		self.insert_node(index, Node::new_window(window));
	}

	/// Inserts new [window nodes] of the given `windows` at the given `index` in the group.
	///
	/// [window nodes]: WindowNode
	#[inline]
	pub fn insert_windows(&mut self, index: usize, windows: impl IntoIterator<Item = Window>) {
		self.insert_nodes(index, windows.into_iter().map(Node::new_window));
	}

	/// Pushes a new [group node] of the given `orientation` to the end of the group.
	///
	/// [group node]: GroupNode
	#[inline]
	pub fn push_group_back(&mut self, orientation: Orientation) {
		self.push_node_back(Node::new_group(orientation));
	}

	/// Pushes a new [group node] of the given `orientation` to the end of the group, then
	/// initialises it with the given `init` function.
	///
	/// [group node]: GroupNode
	#[inline]
	pub fn push_group_back_with(&mut self, orientation: Orientation, init: impl FnOnce(&mut GroupNode<Window>)) {
		let index = self.push_node_back(Node::new_group(orientation));

		match &mut self.children[index] {
			Node::Group(group) => init(group),
			Node::Window(_) => unreachable!("we know the this node is a group, because we just added it"),
		}
	}

	/// Pushes new [group nodes] of the given `orientations` to the end of the group.
	///
	/// [group nodes]: GroupNode
	#[inline]
	pub fn push_groups_back(&mut self, orientations: impl IntoIterator<Item = Orientation>) {
		self.push_nodes_back(orientations.into_iter().map(Node::new_group));
	}

	/// Pushes a new [group node] of the given `orientation` to the beginning of the group.
	///
	/// [group node]: GroupNode
	#[inline]
	pub fn push_group_front(&mut self, orientation: Orientation) {
		self.push_node_front(Node::new_group(orientation));
	}

	/// Pushes a new [group node] of the given `orientation` to the beginning of the group, then
	/// initialises it with the given `init` function.
	///
	/// [group node]: GroupNode
	#[inline]
	pub fn push_group_front_with(&mut self, orientation: Orientation, init: impl FnOnce(&mut GroupNode<Window>)) {
		let index = self.push_node_front(Node::new_group(orientation));

		match &mut self.children[index] {
			Node::Group(group) => init(group),
			Node::Window(_) => unreachable!("we know the this node is a group, because we just added it"),
		}
	}

	/// Pushes new [group nodes] of the given `orientations` to the beginning of the group.
	///
	/// [group nodes]: GroupNode
	#[inline]
	pub fn push_groups_front(&mut self, orientations: impl IntoIterator<Item = Orientation>) {
		self.push_nodes_front(orientations.into_iter().map(Node::new_group))
	}

	/// Inserts a new [group node] of the given `orientation` at the given `index` in the group.
	///
	/// [group node]: GroupNode
	#[inline]
	pub fn insert_group(&mut self, index: usize, orientation: Orientation) {
		self.insert_node(index, Node::new_group(orientation));
	}

	/// Inserts a new [group node] of the given `orientation` at the given `index` in the group,
	/// then initialises it with the given `init` function.
	///
	/// [group node]: GroupNode
	#[inline]
	pub fn insert_group_with(
		&mut self,
		index: usize,
		orientation: Orientation,
		init: impl FnOnce(&mut GroupNode<Window>),
	) {
		let index = self.insert_node(index, Node::new_group(orientation));

		match &mut self.children[index] {
			Node::Group(group) => init(group),
			Node::Window(_) => unreachable!("we know this node is a group, because we just added it"),
		}
	}

	/// Inserts new [group nodes] of the given `orientations` at the given `index` in the group.
	///
	/// [group nodes]: GroupNode
	#[inline]
	pub fn insert_groups(&mut self, index: usize, orientations: impl IntoIterator<Item = Orientation>) {
		self.insert_nodes(index, orientations.into_iter().map(Node::new_group));
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
				self.track_push_front();
			}
		} else {
			for node in nodes {
				self.children.push_back(node);
				self.track_push_front();
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

	#[inline]
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
	pub(crate) fn apply_resizes<Error>(
		&mut self,
		resize_window: &mut impl FnMut(&Window, u32, u32) -> Result<(), Error>,
	) -> Result<(), Error> {
		// If no changes have been made to this group, apply all the child groups' changes and return.
		if !self.changes_made() {
			for node in self {
				match node {
					Node::Group(group) => group.apply_resizes(resize_window)?,

					Node::Window(WindowNode {
						window,
						window_changed,
						width,
						height,
					}) => {
						if mem::take(window_changed) {
							resize_window(window, *width, *height)?
						}
					},
				}
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

		// `u64` is used because we will be multiplying two 'u32' values, and `u64::MAX` is
		// `u32::MAX * u32::MAX`.
		let old_total_node_primary = (self.total_node_primary - total_removed_primary) as u64;

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
				Node::Group(group) => group.apply_resizes(resize_window),

				Node::Window(WindowNode {
					window,
					window_changed,
					width,
					height,
				}) => {
					*window_changed = false;
					resize_window(window, *width, *height)
				},
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
		//
		// `u64` is used because we will be multiplying two 'u32' values, and `u64::MAX` is
		// `u32::MAX * u32::MAX`.
		let rescaling_primary = (group_primary - new_total_node_primary) as u64;

		let mut additions = additions.into_iter();
		let mut next_addition = additions.next();

		// Resize all the nodes appropriately.
		for (index, node) in self.children.iter_mut().enumerate() {
			// If `node` is an addition, resize it with the new size.
			if let Some(addition) = next_addition {
				if index == addition {
					set_node_dimensions(node, new_primary, group_secondary)?;

					next_addition = additions.next();
					continue;
				}
			}

			// `node` is not an addition: rescale it.

			// `u64` is used because we will be multiplying two 'u32' values, and `u64::MAX` is
			// `u32::MAX * u32::MAX`.
			let old_primary = node.primary(old_axis) as u64;
			// Determine the rescaled size.
			//
			// This is `shrink`ed back into a `u32` value (a value `> u32::MAX` will be clipped to
			// `u32::MAX`), though in practice it almost certainly will never get anywhere near that
			// large - monitors don't tend to be millions of pixels in width or height.
			let rescaled_primary: u32 = ((old_primary * rescaling_primary) / old_total_node_primary).shrink();

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

	/// No-op resize_window function to pass to [`apply_resizes`].
	///
	/// [`apply_resizes`]: GroupNode::apply_resizes
	const fn resize_window<Window>(_window: &Window, _width: u32, _height: u32) -> Result<(), ()> {
		Ok(())
	}

	#[test]
	fn group_orientations() {
		const INITIAL_ORIENTATION: Orientation = Orientation::LeftToRight;
		const ROTATIONS: i32 = -6;
		const NEW_ORIENTATION: Orientation = Orientation::RightToLeft;

		let mut group: GroupNode<()> = GroupNode::with_dimensions(INITIAL_ORIENTATION, 0, 0);

		assert_eq!(group.orientation, INITIAL_ORIENTATION);
		assert_eq!(group.new_orientation, None);
		assert_eq!(group.orientation(), INITIAL_ORIENTATION);

		group.rotate_by(ROTATIONS);

		assert_eq!(group.orientation, INITIAL_ORIENTATION);
		assert_eq!(group.new_orientation, Some(NEW_ORIENTATION));
		assert_eq!(group.orientation(), NEW_ORIENTATION);

		// Apply the change in orientation.
		group.apply_resizes(&mut resize_window).unwrap();

		assert_eq!(group.orientation, NEW_ORIENTATION);
		assert_eq!(group.new_orientation, None);
		assert_eq!(group.orientation(), NEW_ORIENTATION);
	}

	#[test]
	fn push_windows() {
		const WINDOWS: [u32; 5] = [1, 2, 3, 4, 5];

		let non_reversed_nodes = VecDeque::from(WINDOWS.map(Node::new_window));
		let reversed_nodes: VecDeque<_> = non_reversed_nodes.iter().cloned().rev().collect();

		// Test a non-reversed group.
		let mut non_reversed_group: GroupNode<u32> = GroupNode::new(Orientation::LeftToRight);
		assert_eq!(non_reversed_group.children, VecDeque::new());

		non_reversed_group.push_windows_back(WINDOWS);
		assert_eq!(non_reversed_group.children, non_reversed_nodes);

		// Test a reversed group.
		let mut reversed_group: GroupNode<u32> = GroupNode::new(Orientation::RightToLeft);

		reversed_group.push_windows_back(WINDOWS);
		assert_eq!(reversed_group.children, reversed_nodes);
	}

	/// Tests [`apply_changes`] in response to adding and removing windows and changing the group
	/// [`orientation`].
	///
	/// [`apply_changes`]: GroupNode::apply_resizes
	/// [`orientation`]: GroupNode::orientation()
	#[test]
	fn resizing() {
		const GROUP_WIDTH: u32 = 3000;
		const GROUP_HEIGHT: u32 = 1000;

		// The width of each node after three nodes have been added.
		const THREE_NODES_WIDTH: u32 = GROUP_WIDTH / 3;
		// The width of each node after two nodes have been added.
		const TWO_NODES_WIDTH: u32 = GROUP_WIDTH / 2;

		// The height of each node after two nodes have been added and the axis has been set to
		// vertical.
		const TWO_NODES_HEIGHT: u32 = GROUP_HEIGHT / 2;

		let mut group: GroupNode<u32> = GroupNode::with_dimensions(Orientation::LeftToRight, GROUP_WIDTH, GROUP_HEIGHT);

		group.push_window_back(1);

		assert!(
			matches!(
				&group[0],
				Node::Window(WindowNode {
					width: 0,
					height: 0,
					..
				}),
			),
			"node = {:?}",
			&group[0],
		);

		// Resize the added window.
		group.apply_resizes(&mut resize_window).unwrap();

		assert!(
			matches!(
				&group[0],
				Node::Window(WindowNode {
					width: GROUP_WIDTH,
					height: GROUP_HEIGHT,
					..
				}),
			),
			"node = {:?}",
			&group[0],
		);

		group.push_windows_back([2, 3]);

		// Resize the existing window and two added windows.
		group.apply_resizes(&mut resize_window).unwrap();

		for node in &group {
			assert!(
				matches!(
					node,
					Node::Window(WindowNode {
						width: THREE_NODES_WIDTH,
						height: GROUP_HEIGHT,
						..
					}),
				),
				"node = {:?}",
				node,
			);
		}

		// Remove the first node.
		group.remove(0);

		// Resize the two remaining windows.
		group.apply_resizes(&mut resize_window).unwrap();

		for node in &group {
			assert!(
				matches!(
					node,
					Node::Window(WindowNode {
						width: TWO_NODES_WIDTH,
						height: GROUP_HEIGHT,
						..
					}),
				),
				"node = {:?}",
				node,
			);
		}

		let mut clone = group.clone();

		// Add a window and immediately remove it.
		clone.push_window_front(1);
		clone.remove(0);
		// Apply the changes (of which there should be none).
		clone.apply_resizes(&mut resize_window).unwrap();

		assert_eq!(group, clone);

		group.set_orientation(Orientation::TopToBottom);
		// Apply the orientation change.
		group.apply_resizes(&mut resize_window).unwrap();

		for node in &group {
			assert!(
				matches!(
					node,
					Node::Window(WindowNode {
						width: GROUP_WIDTH,
						height: TWO_NODES_HEIGHT,
						..
					}),
				),
				"node = {:?}",
				node
			);
		}
	}
}
