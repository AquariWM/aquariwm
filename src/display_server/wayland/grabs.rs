// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

pub mod move_grab;
pub mod resize_grab;

/// The primary mouse button.
///
/// This is far more commonly known as the left mouse button, but we use the term 'primary mouse
/// button' instead, as a number of people may switch the order of their mouse buttons such that it
/// is not on the left of the mouse.
///
/// This is defined as [`BTN_LEFT`] in the Linux kernel's `input-event-codes.h` header file.
///
/// [`BTN_LEFT`]: https://github.com/torvalds/linux/blob/2b93c2c3c02f4243d4c773b880fc86e2788f013d/include/uapi/linux/input-event-codes.h#L356
pub const PRIMARY_BUTTON: u32 = 0x110;
