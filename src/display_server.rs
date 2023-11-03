// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::error::Error;

#[cfg(feature = "wayland")]
pub mod wayland;
#[cfg(feature = "x11")]
pub mod x11;

#[cfg(feature = "wayland")]
pub use wayland::Wayland;
#[cfg(feature = "x11")]
pub use x11::X11;

pub trait DisplayServer {
	type Error: Error;
	const NAME: &'static str;

	fn run(testing: bool) -> Result<(), Self::Error>;

	fn title() -> String {
		format!("AquariWM ({})", Self::NAME)
	}
}
