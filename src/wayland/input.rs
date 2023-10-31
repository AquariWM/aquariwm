// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use smithay::{
	backend::input::{
		AbsolutePositionEvent,
		Axis,
		AxisSource,
		ButtonState,
		Event,
		InputBackend,
		InputEvent,
		KeyboardKeyEvent,
		PointerAxisEvent,
		PointerButtonEvent,
	},
	input::{keyboard::FilterResult, pointer, pointer::AxisFrame},
	utils::SERIAL_COUNTER,
};

use crate::wayland::state;

impl state::AquariWm {
	pub fn process_input_event<Backend: InputBackend>(&mut self, event: InputEvent<Backend>) {
		match event {
			InputEvent::Keyboard { event, .. } => {
				let serial = SERIAL_COUNTER.next_serial();
				let time = Event::time_msec(&event);

				self.seat.get_keyboard().unwrap().input::<(), _>(
					self,
					event.key_code(),
					event.state(),
					serial,
					time,
					|_, _, _| FilterResult::Forward,
				);
			},

			InputEvent::PointerMotion { .. } => (),
			InputEvent::PointerMotionAbsolute { event, .. } => {
				let output = self.space.outputs().next().unwrap();

				let output_geometry = self.space.output_geometry(output).unwrap();
				let (output_size, output_loc) = (output_geometry.size, output_geometry.loc);

				// Location of the pointer.
				let location = event.position_transformed(output_size) + output_loc.to_f64();

				// The serial number to use in the pointer motion event.
				let serial = SERIAL_COUNTER.next_serial();

				let pointer = self.seat.get_pointer().unwrap();

				let under = self.surface_under(location);

				pointer.motion(
					self,
					under,
					&pointer::MotionEvent {
						location,
						serial,
						time: event.time_msec(),
					},
				);
				pointer.frame(self);
			},

			InputEvent::PointerButton { event, .. } => {
				let pointer = self.seat.get_pointer().unwrap();
				// FIXME: the keyboard may not be connected.
				let keyboard = self.seat.get_keyboard().unwrap();

				let serial = SERIAL_COUNTER.next_serial();

				let button = event.button_code();
				let button_state = event.state();

				if ButtonState::Pressed == button_state && !pointer.is_grabbed() {
					match self
						.space
						.element_under(pointer.current_location())
						.map(|(window, location)| (window.clone(), location))
					{
						Some((window, _)) => {
							self.space.raise_element(&window, true);

							let wl_surface = window.toplevel().wl_surface().clone();
							keyboard.set_focus(self, Some(wl_surface), serial);

							for window in self.space.elements() {
								window.toplevel().send_pending_configure();
							}
						},

						None => {
							for window in self.space.elements() {
								window.set_activated(false);
								window.toplevel().send_pending_configure();
							}

							keyboard.set_focus(self, None, serial);
						},
					}
				}

				pointer.button(
					self,
					&pointer::ButtonEvent {
						button,
						state: button_state,
						serial,
						time: event.time_msec(),
					},
				);
				pointer.frame(self);
			},

			InputEvent::PointerAxis { event, .. } => {
				let source = event.source();

				let pointer = self.seat.get_pointer().unwrap();

				let horizontal_amount_discrete = event.amount_discrete(Axis::Horizontal);
				let vertical_amount_discrete = event.amount_discrete(Axis::Vertical);

				let horizontal_amount = event
					.amount(Axis::Horizontal)
					.or_else(|| horizontal_amount_discrete.map(|discrete| discrete * 3.0));
				let vertical_amount = event
					.amount(Axis::Vertical)
					.or_else(|| vertical_amount_discrete.map(|discrete| discrete * 3.0));

				let frame = {
					let mut frame = AxisFrame::new(event.time_msec()).source(source);

					let mut update_amount = |axis| {
						let (amount, discrete) = match axis {
							Axis::Horizontal => (horizontal_amount, horizontal_amount_discrete),
							Axis::Vertical => (vertical_amount, vertical_amount_discrete),
						};

						if let Some(amount) = amount {
							frame = frame.value(axis, amount);

							if let Some(discrete) = discrete {
								frame = frame.discrete(Axis::Horizontal, discrete as i32);
							}
						}

						if source == AxisSource::Finger {
							if event.amount(axis) == Some(0.0) {
								frame = frame.stop(axis);
							}
						}
					};

					update_amount(Axis::Horizontal);
					update_amount(Axis::Vertical);

					frame
				};

				pointer.axis(self, frame);
				pointer.frame(self);
			},

			_ => (),
		}
	}
}
