// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use super::*;

pub struct Stack<Window: Send + Sync + PartialEq + 'static> {
	layout: TilingLayout<Window>,
}

#[allow(unused)]
impl<Window: Send + Sync + PartialEq + 'static> Stack<Window> {
	/// Returns a shared reference to the main window, if there is one.
	fn main(&self) -> Option<&WindowNode<Window>> {
		self.layout.first().and_then(|node| match node {
			Node::Group(_) => None,
			Node::Window(node) => Some(node),
		})
	}

	/// Returns a shared reference to the stack, if there is one.
	fn stack(&self) -> Option<&GroupNode<Window>> {
		self.layout.get(1).and_then(|node| match node {
			Node::Group(node) => Some(node),
			Node::Window(_) => None,
		})
	}

	/// Returns a mutable reference to the main window, if there is one.
	fn main_mut(&mut self) -> Option<&mut WindowNode<Window>> {
		self.layout.first_mut().and_then(|node| match node {
			Node::Group(_) => None,
			Node::Window(node) => Some(node),
		})
	}

	/// Returns a mutable reference to the stack, if there is one.
	fn stack_mut(&mut self) -> Option<&mut GroupNode<Window>> {
		self.layout.get_mut(1).and_then(|node| match node {
			Node::Group(node) => Some(node),
			Node::Window(_) => None,
		})
	}
}

unsafe impl<Window> TilingLayoutManager<Window> for Stack<Window>
where
	Window: Send + Sync + PartialEq + 'static,
{
	#[inline(always)]
	fn orientation() -> Orientation
	where
		Self: Sized,
	{
		Orientation::LeftToRight
	}

	fn init<WindowsIter>(layout: TilingLayout<Window>, windows: WindowsIter) -> Self
	where
		Self: Sized,
		WindowsIter: IntoIterator<Item = Window>,
		WindowsIter::IntoIter: ExactSizeIterator,
	{
		let mut stack = Self { layout };

		let mut windows = windows.into_iter();

		if let Some(main) = windows.next() {
			// Push the main window.
			stack.layout.push_window_back(main);

			// If there are more windows, then add them in a stack.
			if windows.len() > 0 {
				stack
					.layout
					.push_group_back_with(Orientation::TopToBottom, |stack| stack.push_windows_back(windows));
			}
		}

		stack
	}

	#[inline(always)]
	fn layout(&self) -> &TilingLayout<Window> {
		&self.layout
	}

	#[inline(always)]
	fn layout_mut(&mut self) -> &mut TilingLayout<Window> {
		&mut self.layout
	}

	fn add_window(&mut self, window: Window) {
		if self.layout.is_empty() {
			// No main, no stack.

			// Add the window as a main.
			self.layout.push_window_back(window);
		} else if let Some(stack) = self.stack_mut() {
			// Main and stack.

			// Add the window to the stack.
			stack.push_window_back(window);
		} else {
			// Main, no stack.

			// Add the window to a new stack.
			self.layout
				.push_group_back_with(Orientation::TopToBottom, |stack| stack.push_window_back(window));
		}
	}

	fn remove_window(&mut self, window: &Window) {
		if let Some(main) = self.main() {
			if main.window() == window {
				if let Some(Node::Window(new_main)) = self.stack_mut().and_then(|stack| stack.remove(0)) {
					// If there is a window to replace the main window with, do that.
					self.main_mut()
						.expect("We've already established `main` is present.")
						.set_window(new_main.into_window());
				} else {
					// Otherwise, if there is no window to replace the main window with, remove the
					// node.
					self.layout.pop_front();
				}

				return;
			}
		}

		// Otherwise, if the main window does not match...

		// If there is a stack...
		if let Some(stack) = self.stack_mut() {
			let window_nodes = stack.iter_mut().enumerate().filter_map(|(i, node)| match node {
				Node::Group(_) => None,
				Node::Window(window_node) => Some((i, window_node)),
			});

			// For every window node in the stack...
			for (i, node) in window_nodes {
				// If the window matches, remove it and return.
				if node.window() == window {
					if stack.len() > 1 {
						// If it is not the last window in the stack, remove the node.
						stack.remove(i);
					} else {
						// Otherwise, if it is the last window in the stack, remove the stack.
						self.layout.pop_back();
					}

					return;
				}
			}
		}
	}
}

pub struct Spiral<Window: Send + Sync + PartialEq + 'static> {
	layout: TilingLayout<Window>,
}

unsafe impl<Window> TilingLayoutManager<Window> for Spiral<Window>
where
	Window: Send + Sync + PartialEq + 'static,
{
	#[inline(always)]
	fn orientation() -> Orientation
	where
		Self: Sized,
	{
		Orientation::LeftToRight
	}

	fn init<WindowsIter>(layout: TilingLayout<Window>, windows: WindowsIter) -> Self
	where
		Self: Sized,
		WindowsIter: IntoIterator<Item = Window>,
		WindowsIter::IntoIter: ExactSizeIterator,
	{
		let mut spiral = Self { layout };

		let mut target_group: Option<&mut GroupNode<_>> = Some(&mut spiral.layout);

		for window in windows {
			match target_group.take() {
				Some(target) => {
					target.push_group_back(target.orientation.rotated_by(1));

					if let Node::Group(group) = &mut target[1] {
						group.push_window_front(window);
						target_group = Some(group);
					}
				},

				// TODO: see if there is a way to avoid using `Option` here (its usage avoids
				//     : multiple mutable borrows issues)
				None => unreachable!(),
			}
		}

		spiral
	}

	#[inline(always)]
	fn layout(&self) -> &TilingLayout<Window> {
		&self.layout
	}

	#[inline(always)]
	fn layout_mut(&mut self) -> &mut TilingLayout<Window> {
		&mut self.layout
	}

	fn add_window(&mut self, window: Window) {
		let group = {
			// TODO: avoid using `Option` if possible (only used to avoid issues with multiple
			//     : mutable borrows, which using `take()` solves)
			let mut target_group: Option<&mut GroupNode<_>> = Some(&mut self.layout);

			while let Some(Node::Group(group)) = target_group.take().unwrap().get_mut(1) {
				target_group = Some(group);
			}

			target_group.unwrap()
		};

		group.push_group_back_with(group.orientation(), |group| {
			group.push_window_front(window);
		});
	}

	fn remove_window(&mut self, window: &Window) {
		// If the first window matches, remove it and return.
		if let Some(Node::Window(window_node)) = self.layout.get(0) {
			if window_node.window() == window {
				if self.layout.get(1).is_some() {
					// If there are more groups, move the windows up to replace the first window.

					Self::move_window_up(&mut self.layout);
				} else {
					// Otherwise, remove the first window node.

					self.layout.pop_front();
				}

				return;
			}
		}

		let mut target_group: Option<&mut GroupNode<_>> = Some(&mut self.layout);

		while let Some(target) = target_group.take() {
			if let Some(Node::Group(group)) = target.get_mut(1) {
				if group[0].unwrap_window_ref().window() == window {
					// Window matches.

					match group.get_mut(1) {
						// Has a child group: shift all the windows up to replace the removed one.
						Some(Node::Group(group)) => {
							Self::move_window_up(group);
						},

						// Doesn't have a child group: remove the node.
						_ => {
							// FIXME: Why doesn't this work?! It's fine if you remove the `else`
							//      : branch below, but it is so obviously not accessible because
							//      : this if branch ends in `return`. The compiler should
							//      : absolutely be able to work that out.
							// target.remove(1);
						},
					}

					return;
				} else {
					// Window doesn't match; move onto the next group.

					target_group = Some(group);
				}
			}
		}
	}
}

impl<Window: Send + Sync + PartialEq + 'static> Spiral<Window> {
	fn move_window_up(group: &mut GroupNode<Window>) -> Window {
		let window = if let Some(Node::Group(group)) = group[1].unwrap_group_mut().get_mut(1) {
			Self::move_window_up(group)
		} else {
			group.remove(1).unwrap().unwrap_window().into_window()
		};

		group[0].unwrap_window_mut().replace_window(window)
	}
}
