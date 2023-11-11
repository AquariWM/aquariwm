// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use super::*;

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct Stack<Window>(TilingLayout<Window>);

unsafe impl<Window> TilingLayoutManager<Window> for Stack<Window> {
	const ORIENTATION: Orientation = Orientation::LeftToRight;

	fn init<WindowsIter>(mut layout: TilingLayout<Window>, windows: WindowsIter) -> Self
	where
		WindowsIter: IntoIterator<Item = Window>,
		WindowsIter: ExactSizeIterator,
	{
		let mut windows = windows.into_iter();

		// TODO: main and stack

		Self(layout)
	}

	fn add_window(&mut self, window: Window) {
		if self.main.is_none() {
			layout.insert_window(0, window);

			self.main = Some(layout.first_mut());
		} else {
			self.stack.push_window(window);
		}
	}

	fn remove_window(&mut self, window: &Window) {
		todo!()
	}
}
