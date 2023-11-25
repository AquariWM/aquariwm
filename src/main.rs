// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

#![warn(clippy::missing_const_for_fn)]
// Feature flags
#![feature(impl_trait_in_assoc_type)]
#![feature(iterator_try_collect)]
#![feature(doc_cfg)]

use std::{env, ffi::OsString, io, process};

use aquariwm::layout::LayoutSettings;
use clap::Parser;
use thiserror::Error;

use crate::display_server::DisplayServer;

mod cli;
pub mod display_server;
pub mod state;

#[cfg(not(any(feature = "wayland", feature = "x11")))]
compile_error!("At least one display server feature must be enabled for AquariWM to function.");

#[derive(Debug, Error)]
pub enum Error {
	#[cfg(feature = "wayland")]
	#[error(transparent)]
	Wayland(#[from] display_server::wayland::Error),

	#[cfg(feature = "x11")]
	#[error(transparent)]
	X11(#[from] display_server::x11::Error),
}

pub type Result<T, E = Error> = std::result::Result<T, E>;

fn main() -> Result<()> {
	// Initiate `tracing_subscriber` for formatting logs.
	match tracing_subscriber::EnvFilter::try_from_default_env() {
		Ok(env_filter) => tracing_subscriber::fmt().with_env_filter(env_filter).init(),

		Err(_) => tracing_subscriber::fmt().init(),
	}

	// Parse command line subcommand and options.
	let args = cli::Cli::parse();
	// Whether testing is enabled.
	let testing = args.testing();

	let settings = match args.window_gap {
		Some(window_gap) => LayoutSettings::new().window_gap(window_gap),
		None => LayoutSettings::default(),
	};

	match &args.subcommand {
		#[cfg(feature = "wayland")]
		cli::Subcommand::Wayland => Ok(display_server::Wayland::run(testing, settings)?),

		#[cfg(feature = "x11")]
		cli::Subcommand::X11 => Ok(tokio::runtime::Builder::new_multi_thread()
			.enable_all()
			.build()
			.unwrap()
			.block_on(async { display_server::X11::run(testing, settings).await })?),
	}
}

/// An error returned by [`launch_terminal`].
#[derive(Debug, Error)]
pub enum LaunchTerminalError {
	/// The `TERM` environment variable was not set to any terminal.
	#[error("the `TERM` environment variable is not set")]
	VarNotPresent,

	/// An IO error occurred trying to launch the `TERM` terminal.
	#[error(transparent)]
	Io(#[from] io::Error),
}

/// Attempts to launch the terminal set in the `TERM` environment variable.
///
/// If successful, returns the launched terminal process and the contents of the `TERM` environment
/// variable launched.
pub fn launch_terminal() -> Result<(OsString, process::Child), LaunchTerminalError> {
	match env::var_os("TERM") {
		// `TERM` is present.
		Some(terminal) => {
			let process = process::Command::new(&terminal).spawn()?;

			Ok((terminal, process))
		},

		// `TERM` is not present.
		None => Err(LaunchTerminalError::VarNotPresent),
	}
}
