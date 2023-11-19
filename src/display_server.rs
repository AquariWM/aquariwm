// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::{error::Error, future::Future};

use cfg_attrs::cfg_attrs;
#[cfg(feature = "wayland")]
pub use wayland::Wayland;
#[cfg(feature = "x11")]
pub use x11::X11;

#[cfg(feature = "wayland")]
pub mod wayland;
#[cfg(feature = "x11")]
pub mod x11;

/// An implementation of AquariWM for a particular display server (i.e. X11 or Wayland).
pub trait DisplayServer {
	/// The return type used by the display server's [`run`] function.
	///
	/// [`run`]: Self::run
	type Output;
	/// The name of the display server (e.g. `"X11"`).
	const NAME: &'static str;

	/// Runs the AquariWM implementation for this display server.
	fn run(testing: bool) -> Self::Output;

	/// Returns AquariWM's title, formatted with the display server [`NAME`].
	///
	/// This can be used for the title of the testing window, for example.
	fn title() -> String {
		format!("AquariWM ({})", Self::NAME)
	}
}

/// A [display server] whose [`run`] function does not return a [future].
///
/// [display server]: DisplayServer
/// [future]: Future
/// [`run`]: Self::run
#[cfg_attrs(
	feature = "async",
	///
	/// # See also
	/// - [AsyncDisplayServer]
)]
pub trait SyncDisplayServer: DisplayServer<Output = Result<(), Self::Error>> {
	/// The error type returned from the display server's [`run`] function.
	///
	/// [`run`]: Self::run
	type Error: Error;
}

/// A [display server] whose [`run`] function returns a [future].
///
/// # See also
/// - [SyncDisplayServer]
///
/// [display server]: DisplayServer
/// [future]: Future
/// [`run`]: Self::run
#[cfg(feature = "async")]
pub trait AsyncDisplayServer: DisplayServer
where
	Self::Output: Future<Output = Result<(), Self::Error>>,
{
	/// The error type returned from the display server's [`run`] function.
	///
	/// [`run`]: Self::run
	type Error: Error;
}
