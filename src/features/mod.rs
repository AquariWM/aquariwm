// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

/// Keeps track of the manipulation of a window. See the
/// [WindowManipulation](window_manipulation::WindowManipulation) enum for more information.
mod window_manipulation;

// Re-export the [window_manipulation] module, so as to avoid repeating the name of the feature
// twice.
pub use window_manipulation::WindowManipulation;
