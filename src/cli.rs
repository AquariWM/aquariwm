// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use clap::Parser;

#[derive(Debug, Parser)]
pub struct Cli {
	/// Whether AquariWM should be launched in a testing window.
	#[cfg(feature = "testing")]
	#[arg(long, alias = "test")]
	pub testing: bool,

	#[command(subcommand)]
	pub subcommand: Option<Subcommand>,
}

#[derive(Debug, clap::Subcommand)]
pub enum Subcommand {
	/// Launch AquariWM running in Wayland mode.
	#[cfg(feature = "wayland")]
	Wayland,
	/// Launch AquariWM running in X11 mode.
	#[cfg(feature = "x11")]
	X11,
}
