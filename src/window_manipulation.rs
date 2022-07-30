// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use tracing::{debug, trace};

use xcb::x::{self, ConfigWindow, Window};
use xcb::{Connection, Xid};

/// Represents the state of an ongoing window manipulation.
///
/// Contains the [`window()`](WindowManipulation::window) being manipulated, the
/// [`cursor_pos()`](WindowManipulation::cursor_pos) relative to the root window when the
/// manipulation commenced, and the original position or size of the
/// [`window()`](WindowManipulation::window) to calculate the offset and to allow the
/// [`window()`](WindowManipulation::window) to be returned to its original state with
/// [`WindowManipulation::cancel()`].
///
/// Only one window can be manipulated at one time, and only one mode of window manipulation may
/// operate on that window at one time. If the window is being moved, it cannot also be resized
/// in the same window manipulation.
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum WindowManipulation {
	/// The [`window()`](WindowManipulation::window)'s position is being manipulated; the user is
	/// moving the [`window()`](WindowManipulation::window).
	Moving {
		/// The window this window manipulation applies to.
		window: Window,
		/// The current parent window of the `window` being manipulated.
		///
		/// The [`Moving`](WindowManipulation::Moving) window manipulation keeps track of the
		/// `window`'s `parent` because window coordinates are relative to the `window`'s `parent`
		/// window, _not_ the root window. Since the window manipulation stores the original
		/// position (`orig_coords`) of the `window`, the coordinates must be translated from the
		/// `parent` window's coordinates to the `window`'s new parent if the `window` happens to
		/// be reparented during manipulation.
		parent: Window,
		/// The original position of the `window`, relative to its `parent`.
		///
		/// If the `window` being manipulated is reparented during the course of its manipulation,
		/// these coordinates should be translated from the old `parent` to the new one.
		orig_coords: (i16, i16),
		/// The starting position of the cursor when the window manipulation begins.
		///
		/// The `cursor_pos` is relative to the `window`'s root window.
		///
		/// This is used to calculate how far the cursor has moved during window manipulation, and
		/// thus the offset that should be applied to the `window`'s position.
		cursor_pos: (i16, i16),
	},
	/// The [`window()`](WindowManipulation::window)'s dimensions are being manipulated; the user
	/// is resizing the [`window()`](WindowManipulation::window).
	Resizing {
		/// The window this window manipulation applies to.
		window: Window,
		/// The original dimensions of the `window`.
		orig_size: (u16, u16),
		/// The starting position of the cursor when the window manipulation begins.
		///
		/// The `cursor_pos` is relative to the `window`'s root window.
		///
		/// This is used to calculate how far the cursor has moved during window manipulation, and
		/// thus the offset that should be applied to the `window`'s dimensions.
		cursor_pos: (i16, i16),
	},
}

#[allow(dead_code)]
impl WindowManipulation {
	/// Creates a [`WindowManipulation::Moving`] window manipulation.
	///
	/// Blocking: waits for one [GetGeometry](xcb::x::GetGeometry) request, one
	/// [QueryTree](xcb::x::QueryTree) request.
	///
	/// Flushes the connection.
	pub fn moving(conn: &Connection, window: Window, cursor_pos: (i16, i16)) -> xcb::Result<Self> {
		// Get geometry request to get the window's coordinates.
		let geometry_req = get_geometry(conn, window);

		// Window tree request to get the window's parent.
		trace!(window = window.resource_id(), "Requesting window tree");
		let tree_req = conn.send_request(&x::QueryTree { window });

		conn.flush()?;

		let geometry = conn.wait_for_reply(geometry_req)?;
		let tree = conn.wait_for_reply(tree_req)?;

		debug!(
			window = window.resource_id(),
			"Beginning window manipulation"
		);
		Ok(Self::Moving {
			window,
			parent: tree.parent(),
			orig_coords: (geometry.x(), geometry.y()),
			cursor_pos,
		})
	}

	/// Creates a [`WindowManipulation::Resizing`] window manipulation.
	///
	/// Blocking: waits for one [GetGeometry](xcb::x::GetGeometry) request.
	///
	/// Flushes the connection.
	pub fn resizing(
		conn: &Connection,
		window: Window,
		cursor_pos: (i16, i16),
	) -> xcb::Result<Self> {
		// Get geometry request to get the window's dimensions.
		let geometry = conn.wait_for_reply(get_geometry(conn, window))?;

		debug!(
			window = window.resource_id(),
			"Beginning window manipulation"
		);
		Ok(Self::Resizing {
			window,
			orig_size: (geometry.width(), geometry.height()),
			cursor_pos,
		})
	}

	/// Returns whether this [WindowManipulation] is [`Moving`](WindowManipulation::Moving).
	///
	/// This is the inverse of [WindowManipulation::is_resizing()], and is simply another shorthand
	/// for:
	/// ```rust
	/// matches!(self, Self::Moving { .. })
	/// ```
	///
	/// See also: [`matches!`]
	pub fn is_moving(self) -> bool {
		matches!(self, Self::Moving { .. })
	}

	/// Returns whether this [WindowManipulation] is [`Resizing`](WindowManipulation::Resizing).
	///
	/// This is the inverse of [WindowManipulation::is_moving()], and is simply another shorthand
	/// for:
	/// ```rust
	/// matches!(self, Self::Resizing { .. })
	/// ```
	///
	/// See also: [`matches!`]
	pub fn is_resizing(self) -> bool {
		matches!(self, Self::Resizing { .. })
	}

	/// Returns the [`window()`](WindowManipulation::window) being manipulated to its original
	/// position or size.
	///
	/// __This does not end the window manipulation.__ Make sure you don't continue to manipulate
	/// the window if you want this to have an effect.
	///
	/// Flushes the connection.
	pub fn cancel(self, conn: &Connection) -> xcb::Result<()> {
		match self {
			Self::Moving {
				window,
				orig_coords,
				..
			} => {
				// If `Moving`, set the window's position to `orig_coords`.
				debug!(
					window = window.resource_id(),
					orig_x = orig_coords.0,
					orig_y = orig_coords.1,
					"Cancelling window manipulation and undoing changes"
				);
				set_position(conn, window, (orig_coords.0 as i32, orig_coords.1 as i32));
			}
			Self::Resizing {
				window, orig_size, ..
			} => {
				// If `Resizing`, set the window's dimensions to `orig_size`.
				debug!(
					window = window.resource_id(),
					orig_width = orig_size.0,
					orig_height = orig_size.1,
					"Cancelling window manipulation and undoing changes"
				);
				set_dimensions(conn, window, (orig_size.0 as u32, orig_size.1 as u32));
			}
		}

		conn.flush()?;
		Ok(())
	}

	/// The window this window manipulation applies to.
	pub fn window(self) -> Window {
		match self {
			Self::Moving { window, .. } => window,
			Self::Resizing { window, .. } => window,
		}
	}

	/// The starting position of the cursor when the window manipulation begins.
	///
	/// See also: [`cursor_x()`](WindowManipulation::cursor_x),
	/// [`cursor_y()`](WindowManipulation::cursor_y)
	pub fn cursor_pos(self) -> (i16, i16) {
		match self {
			Self::Moving { cursor_pos, .. } => cursor_pos,
			Self::Resizing { cursor_pos, .. } => cursor_pos,
		}
	}

	/// The x coordinate of the cursor when the window manipulation begins.
	///
	/// See also: [`cursor_pos()`](WindowManipulation::cursor_pos),
	/// [`cursor_y()`](WindowManipulation::cursor_y)
	pub fn cursor_x(self) -> i16 {
		self.cursor_pos().0
	}

	/// The y coordinate of the cursor when the window manipulation begins.
	///
	/// See also: [`cursor_pos()`](WindowManipulation::cursor_pos),
	/// [`cursor_y()`](WindowManipulation::cursor_y)
	pub fn cursor_y(self) -> i16 {
		self.cursor_pos().1
	}

	/// The difference between the given cursor position and the starting
	/// [`cursor_pos()`](WindowManipulation::cursor_pos).
	///
	/// See also: [`diff_x(x: i16)`], [`diff_y(y: i16)`]
	pub fn diff(self, cursor_pos: (i16, i16)) -> (i16, i16) {
		(self.diff_x(cursor_pos.0), self.diff_y(cursor_pos.1))
	}

	/// The difference between the given x coordinate and the starting
	/// [`cursor_x()`](WindowManipulation::cursor_x).
	///
	/// See also: [`diff(cursor_pos: (i16, i16))`](WindowManipulation::diff),
	/// [`diff_y(y: i16)`](WindowManipulation::diff_y)
	pub fn diff_x(self, x: i16) -> i16 {
		x - self.cursor_pos().0
	}

	/// The difference between the given y coordinate and the starting
	/// [`cursor_y()`](WindowManipulation::cursor_y).
	///
	/// See also: [`diff(cursor_pos: (i16, i16))`](WindowManipulation::diff),
	/// [`diff_x(x: i16)`](WindowManipulation::diff_x)
	pub fn diff_y(self, y: i16) -> i16 {
		y - self.cursor_pos().1
	}

	/// Applies the window manipulation to the given window.
	pub fn apply(self, conn: &Connection, cursor_pos: (i16, i16)) -> xcb::Result<()> {
		let diff_x = self.diff_x(cursor_pos.0);
		let diff_y = self.diff_y(cursor_pos.1);

		match self {
			Self::Moving {
				window,
				orig_coords,
				..
			} => {
				set_position(
					conn,
					window,
					(
						orig_coords.0 as i32 + diff_x as i32,
						orig_coords.1 as i32 + diff_y as i32,
					),
				);
			}
			Self::Resizing {
				window, orig_size, ..
			} => {
				// Since the difference is a signed integer but the window dimensions are
				// unsigned integers, we must apply the difference to the dimensions carefully.
				let dimensions = (
					// x
					if diff_x > 0 {
						// If `diff_x` is positive, we can just add the numbers.
						orig_size.0 as u32 + diff_x as u32
					} else {
						// If `diff_x` is negative, we need to make sure it won't shrink the width
						// too far.
						if (-diff_x as u16) < orig_size.0 {
							// If `diff_x` decreases the width by less than the width itself, we
							// can subtract it.
							orig_size.0 as u32 - -diff_x as u32
						} else {
							// If `diff_x` shrinks the width too much, we'll set the width to 1.
							1
						}
					},
					// y
					if diff_y > 0 {
						// If `diff_y` is positive, we can just add the numbers.
						orig_size.1 as u32 + diff_y as u32
					} else {
						// If `diff_y` is negative, we need to make sure it won't shrink the
						// height too far.
						if (-diff_y as u16) < orig_size.1 {
							// If `diff_y` decreases the height by less than the height itself, we
							// can substract it.
							orig_size.1 as u32 - -diff_y as u32
						} else {
							// If `diff_y` shrinks the height too much, we'll set the height to 1.
							1
						}
					},
				);

				// TODO: This sends far too many requests! We need to make sure that not too many
				//       changes to the window's dimensions are sent, or the X server melts.
				set_dimensions(conn, window, dimensions);
			}
		}

		Ok(())
	}
}

/// Sends a [GetGeometry](xcb::x::GetGeometry) request for the given window and returns its reply.
///
/// Does not flush the connection.
///
/// TODO: Move out to utility module.
fn get_geometry(conn: &Connection, window: Window) -> x::GetGeometryCookie {
	trace!(window = window.resource_id(), "Requesting window geometry");
	conn.send_request(&x::GetGeometry {
		drawable: x::Drawable::Window(window),
	})
}

/// Sends a [ConfigureWindow](xcb::x::ConfigureWindow) request to change the coordinates of the
/// given window.
///
/// Does not flush the connection.
///
/// TODO: Move out to utility module.
fn set_position(conn: &Connection, window: Window, coords: (i32, i32)) {
	trace!(
		window = window.resource_id(),
		x = coords.0,
		y = coords.1,
		"Configuring window coordinates"
	);
	conn.send_request(&x::ConfigureWindow {
		window,
		value_list: &[ConfigWindow::X(coords.0), ConfigWindow::Y(coords.1)],
	});
}

/// Sends a [ConfigureWindow](xcb::x::ConfigureWindow) request to change the dimensions of the
/// given window.
///
/// Does not flush the connection.
///
/// TODO: Move out to utility module.
fn set_dimensions(conn: &Connection, window: Window, dimensions: (u32, u32)) {
	trace!(
		window = window.resource_id(),
		width = dimensions.0,
		y = dimensions.1,
		"Configuring window dimensions"
	);
	conn.send_request(&x::ConfigureWindow {
		window,
		value_list: &[
			ConfigWindow::Width(dimensions.0),
			ConfigWindow::Height(dimensions.1),
		],
	});
}
