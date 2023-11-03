// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::{env, error::Error, process};

use clap::Parser;
use tracing::{event, Level};

use crate::display_server::DisplayServer;

mod cli;
pub mod display_server;
pub mod layout;

#[cfg(not(any(feature = "wayland", feature = "x11")))]
compile_error!("At least one display server feature must be enabled for AquariWM to function.");

fn main() -> Result<(), Box<dyn Error>> {
	// Initiate `tracing_subscriber` for formatting logs.
	match tracing_subscriber::EnvFilter::try_from_default_env() {
		Ok(env_filter) => tracing_subscriber::fmt().with_env_filter(env_filter).init(),

		Err(_) => tracing_subscriber::fmt().init(),
	}

	// Parse command line subcommand and options.
	let args = cli::Cli::parse();
	// Whether testing is enabled.
	let testing = args.testing();

	match &args.subcommand {
		#[cfg(feature = "wayland")]
		cli::Subcommand::Wayland => Ok(display_server::Wayland::run(testing)?),
		#[cfg(feature = "x11")]
		cli::Subcommand::X11 => Ok(display_server::X11::run(testing)?),
	}
}

pub fn launch_terminal() {
	if let Some(terminal) = env::var_os("TERM") {
		match process::Command::new(&terminal).spawn() {
			Ok(_) => event!(Level::INFO, "Launched {terminal:?}"),

			Err(error) => event!(Level::WARN, "Failed to launch {terminal:?}: {error}"),
		}
	}
}
