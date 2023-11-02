// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::{env, error::Error, process};

use clap::Parser;
use tracing::{event, Level};

mod cli;
pub mod display_server;
pub mod layout;

fn main() -> Result<(), Box<dyn Error>> {
	if let Ok(env_filter) = tracing_subscriber::EnvFilter::try_from_default_env() {
		tracing_subscriber::fmt().with_env_filter(env_filter).init();
	} else {
		tracing_subscriber::fmt().init();
	}

	let args = cli::Cli::parse();

	#[cfg(feature = "testing")]
	let testing = args.testing();
	#[cfg(not(feature = "testing"))]
	let testing = false;

	#[cfg(any(feature = "wayland", feature = "x11"))]
	match &args.subcommand {
		#[cfg(feature = "wayland")]
		Some(cli::Subcommand::Wayland) => display_server::wayland::run(testing)?,
		#[cfg(feature = "x11")]
		Some(cli::Subcommand::X11) => display_server::x11::run(testing)?,

		None => todo!("Automatically determine running display server..."),
	}

	Ok(())
}

pub fn launch_terminal() {
	if let Some(terminal) = env::var_os("TERM") {
		match process::Command::new(&terminal).spawn() {
			Ok(_) => event!(Level::INFO, "Launched {terminal:?}"),

			Err(error) => event!(Level::WARN, "Failed to launch {terminal:?}: {error}"),
		}
	}
}
