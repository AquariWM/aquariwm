// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use super::*;

pub struct Stack<Window: Send + Sync + 'static>(TilingLayout<Window>);

#[allow(unused)]
impl<Window: Send + Sync + 'static> Stack<Window> {
	/// Returns a shared reference to the main window, if there is one.
	fn main(&self) -> Option<&WindowNode<Window>> {
		self.layout().first().and_then(|node| match node {
			Node::Group(_) => None,
			Node::Window(node) => Some(node),
		})
	}

	/// Returns a shared reference to the stack, if there is one.
	fn stack(&self) -> Option<&GroupNode<Window>> {
		self.layout().get(1).and_then(|node| match node {
			Node::Group(node) => Some(node),
			Node::Window(_) => None,
		})
	}

	/// Returns a mutable reference to the main window, if there is one.
	fn main_mut(&mut self) -> Option<&mut WindowNode<Window>> {
		self.layout_mut().first_mut().and_then(|node| match node {
			Node::Group(_) => None,
			Node::Window(node) => Some(node),
		})
	}

	/// Returns a mutable reference to the stack, if there is one.
	fn stack_mut(&mut self) -> Option<&mut GroupNode<Window>> {
		self.layout_mut().get_mut(1).and_then(|node| match node {
			Node::Group(node) => Some(node),
			Node::Window(_) => None,
		})
	}
}

unsafe impl<Window: Send + Sync + 'static> TilingLayoutManager<Window> for Stack<Window> {
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
		let mut stack_layout = Self(layout);
		let layout = &mut stack_layout.0;

		let mut windows = windows.into_iter();

		if let Some(main) = windows.next() {
			layout.push_window(main);

			// If there are more windows, then add them in a stack.
			if windows.len() > 0 {
				layout.push_group_with(Orientation::TopToBottom, |stack| stack.push_windows(windows));
			}
		}

		stack_layout
	}

	#[inline(always)]
	fn layout(&self) -> &TilingLayout<Window> {
		&self.0
	}

	#[inline(always)]
	fn layout_mut(&mut self) -> &mut TilingLayout<Window> {
		&mut self.0
	}

	fn add_window(&mut self, window: Window) {
		if self.layout().is_empty() {
			// No main, no stack.

			// Add the window as a main.
			self.layout_mut().push_window(window);
		} else if let Some(stack) = self.stack_mut() {
			// Main and stack.

			// Add the window to the stack.
			stack.push_window(window);
		} else {
			// Main, no stack.

			// Add the window to a new stack.
			self.layout_mut()
				.push_group_with(Orientation::TopToBottom, |stack| stack.push_window(window));
		}
	}

	fn remove_window(&mut self, _window: &Window) {
		todo!("Need to implement iterators for groups first.")
	}
}
