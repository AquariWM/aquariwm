// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::{process, sync::mpsc, time::Duration};

use winit::{
	event::{Event as WinitEvent, WindowEvent as WinitWindowEvent},
	event_loop::EventLoopBuilder as WinitEventLoopBuilder,
	platform::x11::EventLoopBuilderExtX11,
	window::WindowBuilder as WinitWindowBuilder,
};

use crate::display_server::x11::*;

pub struct Xephyr(pub process::Child);

impl Drop for Xephyr {
	fn drop(&mut self) {
		let Self(child) = self;

		child.kill().expect("Failed to kill Xephyr");
	}
}

impl Xephyr {
	pub fn spawn() -> io::Result<Self> {
		const TESTING_DISPLAY: &str = ":1";

		let (transmitter, receiver) = mpsc::channel();

		// Create and run a `winit` window for `Xephyr` to use in another thread so it doesn't block the
		// main thread.
		// TODO: use tokio for this instead!
		thread::spawn(move || {
			event!(Level::DEBUG, "Initialising winit window");

			let event_loop = WinitEventLoopBuilder::new().with_any_thread(true).build().unwrap();
			let window = WinitWindowBuilder::new()
				.with_title(X11::title())
				.build(&event_loop)
				.unwrap();

			// Send the window's window ID back to the main thread so it can be supplied to `Xephyr`.
			transmitter.send(u64::from(window.id())).unwrap();

			event_loop
				.run(move |event, target| {
					if let WinitEvent::WindowEvent {
						event: WinitWindowEvent::CloseRequested,
						..
					} = event
					{
						target.exit()
					}
				})
				.unwrap();
		});
		let window_id = receiver.recv().unwrap();

		event!(Level::DEBUG, "Initialising Xephyr");
		match process::Command::new("Xephyr")
			.arg("-resizeable")
			// Run `Xephyr` in the `winit` window.
			.args(["-parent", &window_id.to_string()])
			.arg(TESTING_DISPLAY)
			.spawn()
		{
			Ok(process) => {
				// Set the `DISPLAY` env variable to `TESTING_DISPLAY`.
				env::set_var("DISPLAY", TESTING_DISPLAY);

				// Sleep for 1s to wait for Xephyr to launch. Not ideal.
				thread::sleep(Duration::from_secs(1));

				// Spawn the `picom` compositor, if possible.
				let _ = process::Command::new("picom").spawn();

				Ok(Self(process))
			},

			Err(error) => {
				event!(Level::ERROR, "Error while attempting to initialise Xephyr: {error}");

				Err(error)
			},
		}
	}
}
