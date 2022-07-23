// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

/// The current window manipulation state.
pub enum ManipulationState {
    /// The window manager is not currently manipulating any windows.
    None,
    /// The window manager is currently moving the focused window.
    ///
    /// The [`Moving`] state holds two values, representing the `x` and `y` position of the cursor
    /// when the window manipulation started, relative to the focused window.
    Moving(i16, i16),
    /// The window manager is currently resizing the focused window.
    ///
    /// The [`Resizing`] state holds two values, representing the `x` and `y` position of the
    /// cursor when the window manipulation started, relative to the focused window.
    Resizing(i16, i16),
}

// Set the default [ManipulationState] to [None].
impl Default for ManipulationState {
    fn default() -> Self {
        ManipulationState::None
    }
}