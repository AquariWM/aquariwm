// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use xcb::x::Window;

/// Represents an ongoing manipulation of a window's position or size.
///
/// The [WindowManipulation] struct is responsible for keeping track of the manipulation of a
/// window. It contains information about which [Type] of manipulation is occurring on the window
/// and the start position of the cursor relative to the window (so that changes to the window can
/// be applied relative to where they started).
#[derive(Debug)]
pub struct WindowManipulation {
	/// The window that is currently being manipulated. Can only ever be one at a time.
	_window: Window,
	/// The position of the pointer when the manipulation commences.
	///
	/// Used to calculate the manipulation relative to where it began. For example, if you start
	/// moving a window, and you move the cursor 20 pixels to the right, you want the window to
	/// move 20 pixels to the right. The cursor position is used to calculate how far the cursor
	/// has moved.
	_cursor_pos: (i16, i16),
	/// Represents the [Type] of manipulation being used on the window and how to reverse it.
	///
	/// See the [Type] enum documentation for more information.
	_mode: Type,
}

/// [Type] represents the current window manipulation operation of the window manager.
///
/// At any given time, the user cannot be manipulating more than one window, and the user cannot
/// be performing more than one type of manipulation on a window at a time.
///
/// The [Type] also contains either the original position (in case of [Type::_Moving]) or the
/// original size (in case of [Type::_Resizing]), so that the window manipulation can be
/// cancelled if necessary.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Type {
	/// Represents a state where the user is currently manipulating a window's position.
	///
	/// Contains the starting position of the window being manipulated so the window can be
	/// returned to its original position if the manipulation is cancelled.
	_Moving { original_pos: (i16, i16) },
	/// Represents a state where the user is currently manipulating a window's dimensions.
	///
	/// Contains the starting dimensions of the window being manipulated so the window can be
	/// returned to its original size if the manipulation is cancelled.
	_Resizing { original_size: (u16, u16) },
}
