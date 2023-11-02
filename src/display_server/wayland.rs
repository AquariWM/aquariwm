// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::{any::Any, env, time::Duration};

#[cfg(feature = "testing")]
use smithay::backend::winit::{self, WinitEvent};
use smithay::{
	backend::renderer::{
		damage::OutputDamageTracker,
		element::surface::WaylandSurfaceRenderElement,
		gles::GlesRenderer,
	},
	desktop::space::render_output,
	output::{self, Output, PhysicalProperties, Subpixel},
	reexports::{
		calloop,
		calloop::EventLoop,
		wayland_server::{backend::InitError, Display},
	},
	utils::{Rectangle, Transform},
};
use thiserror::Error;
use tracing::{event, span, Level};

pub mod grabs;
mod input;
/// Manages AquariWM's state.
pub mod state;

pub const FPS: i32 = 60;
pub const MS_PER_SECOND: i32 = 1000;

pub const REFRESH_RATE: i32 = FPS * MS_PER_SECOND;
pub const REFRESH_DELAY: u64 = (MS_PER_SECOND / FPS) as u64;

#[derive(Debug, Error)]
pub enum Error {
	#[error(transparent)]
	Calloop(#[from] calloop::Error),
	#[error(transparent)]
	CalloopInsert(#[from] calloop::InsertError<Box<dyn Any>>),

	#[error(transparent)]
	WaylandInit(#[from] InitError),

	#[cfg(feature = "testing")]
	#[error(transparent)]
	Winit(#[from] winit::Error),
}

pub fn run(testing: bool) -> Result<(), Error> {
	// Log initialisation.
	let init_span = span!(Level::INFO, "Initialising AquariWM (Wayland)").entered();

	// Create an event loop for the compositor to run with.
	let mut event_loop = <EventLoop<state::AquariWm>>::try_new()?;
	// Initialise the AquariWM state.
	let mut state = state::AquariWm::new(Display::new()?, &mut event_loop);

	// Init winit for testing if the testing feature is enabled.
	#[cfg(feature = "testing")]
	if testing {
		init_winit(&mut event_loop, &mut state)?;

		// Attempt to launch a terminal.
		crate::launch_terminal();
	}

	// End the initialisation span.
	init_span.exit();

	event_loop.run(None, &mut state, move |_| {})?;

	Ok(())
}

#[cfg(feature = "testing")]
pub fn init_winit(event_loop: &mut EventLoop<state::AquariWm>, state: &mut state::AquariWm) -> Result<(), Error> {
	let _span = span!(Level::DEBUG, "Initialising winit").entered();

	// Initialise winit.
	let (mut backend, winit) = winit::init()?;
	event!(Level::TRACE, "Initialised winit");

	// Log the creation of the output.
	let output_span = span!(Level::TRACE, "Creating fake AquariWM Winit output").entered();

	let output_mode = output::Mode {
		// Size of the winit window.
		size: backend.window_size(),
		// Refresh rate - 60 fps.
		refresh: FPS * MS_PER_SECOND,
	};

	// Create a fake output for the winit window.
	let output = Output::new(
		// Output name.
		"winit".to_owned(),
		// Properties of the fake output.
		PhysicalProperties {
			// No physical size because there is no physical monitor.
			size: (0, 0).into(),
			subpixel: Subpixel::Unknown,
			make: "AquariWM".into(),
			model: "Winit".into(),
		},
	);
	output.create_global::<state::AquariWm>(&mut state.display_handle);

	output.change_current_state(
		Some(output_mode),
		Some(Transform::Flipped180),
		None,
		// Move to 0,0.
		Some((0, 0).into()),
	);
	// Prefer the `output_mode`.
	output.set_preferred(output_mode);

	state.space.map_output(&output, (0, 0));
	event!(Level::TRACE, "Mapping the fake output");

	// Exit the output log span.
	output_span.exit();

	let mut damage_tracker = OutputDamageTracker::from_output(&output);

	// Set the `WAYLAND_DISPLAY` for the window manager to use to the socket name.
	env::set_var("WAYLAND_DISPLAY", &state.socket_name);

	event_loop
		.handle()
		.insert_source(winit, move |event, _, state| {
			let state::AquariWm {
				display_handle,
				popup_manager,
				space,
				start_time,
				loop_signal,
				..
			} = state;

			match event {
				WinitEvent::Resized { size, .. } => {
					output.change_current_state(
						Some(output::Mode {
							size,
							refresh: REFRESH_RATE,
						}),
						None,
						None,
						None,
					);
				},

				WinitEvent::Input(event) => state.process_input_event(event),

				WinitEvent::Redraw => {
					let size = backend.window_size();
					let damage = Rectangle::from_loc_and_size((0, 0), size);

					backend.bind().unwrap();

					// RustRover has a false negative here, I filed a report at:
					// https://youtrack.jetbrains.com/issue/RUST-12497/False-negative-for-E0107-wrong-number-of-type-arguments-when-some-type-arguments-are-feature-flag-gated
					// To be fair, this is an absolutely unhinged function signature.
					render_output::<_, WaylandSurfaceRenderElement<GlesRenderer>, _, _>(
						&output,
						backend.renderer(),
						1.0,
						0,
						[&*space],
						&[],
						&mut damage_tracker,
						[0.1, 0.1, 0.1, 0.1],
					)
					.unwrap();
					backend.submit(Some(&[damage])).unwrap();

					for window in space.elements() {
						window.send_frame(&output, start_time.elapsed(), Some(Duration::ZERO), |_, _| {
							Some(output.clone())
						});
					}

					space.refresh();
					popup_manager.cleanup();
					let _ = display_handle.flush_clients();

					// Ask for a redraw to schedule a new frame.
					backend.window().request_redraw();
				},

				WinitEvent::CloseRequested => {
					loop_signal.stop();
				},

				_ => (),
			}
		})
		.map_err(|error| calloop::InsertError::<Box<dyn Any>> {
			inserted: Box::new(error.inserted),
			error: error.error,
		})?;

	Ok(())
}
