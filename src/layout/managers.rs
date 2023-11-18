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
		if self.layout().is_empty() {
			// No main, no stack.

			// Add the window as a main.
			self.layout_mut().push_window_back(window);
		} else if let Some(stack) = self.stack_mut() {
			// Main and stack.

			// Add the window to the stack.
			stack.push_window_back(window);
		} else {
			// Main, no stack.

			// Add the window to a new stack.
			self.layout_mut()
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
						.set_window(new_main.unwrap());
				} else {
					// Otherwise, if there is no window to replace the main window with, remove the
					// node.

					self.layout.remove(0);
				}

				return;
			}
		}

		// Otherwise, if the main window does not match...

		// If there is a stack...
		if let Some(stack) = self.stack() {
			// For each window in the stack...
			for i in 0..stack.len() {
				if let Node::Window(stack_node) = &stack[i] {
					// If the window matches, remove it and return.
					if stack_node.window() == window {
						self.stack_mut()
							.expect("We've already established the stack is present.")
							.remove(i);

						return;
					}
				}
			}
		}
	}
}
