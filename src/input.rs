/*!
Reading the joystick as game input.

The joystick is polled far more often than the game ticks, so this translates
raw positions into the one direction that mattered since the last tick.
*/

use crate::board::{JoystickPosition, Lumatrix};
use crate::game::direction::Direction;

/**
Translates a joystick position into a direction.

Returns `None` while the stick rests in the centre.
*/
pub fn direction_of(position: JoystickPosition) -> Option<Direction> {
    match position {
        JoystickPosition::Center => None,
        JoystickPosition::Up => Some(Direction::Up),
        JoystickPosition::Down => Some(Direction::Down),
        JoystickPosition::Left => Some(Direction::Left),
        JoystickPosition::Right => Some(Direction::Right),
    }
}

/**
Reads the joystick once.

Returns `None` while the stick rests in the centre.
*/
pub fn read(lumatrix: &mut Lumatrix) -> Option<Direction> {
    direction_of(lumatrix.joystick_position())
}

/**
Blocks until the player presses the joystick button, then until they let go.

Waiting for the release stops a single long press from being read as the next
game's input as well.
*/
pub fn wait_for_press(lumatrix: &mut Lumatrix) {
    while !lumatrix.joystick_pressed() {
        lumatrix.sleep_ms(POLL_MS);
    }
    while lumatrix.joystick_pressed() {
        lumatrix.sleep_ms(POLL_MS);
    }
}

/**
Counts polling intervals until the player presses the button.

The result varies with how long the player waited, which makes it a usable
seed for food placement on a board that has no clock to read.
*/
pub fn wait_for_press_counting(lumatrix: &mut Lumatrix) -> u32 {
    let mut ticks: u32 = 0;
    while !lumatrix.joystick_pressed() {
        ticks = ticks.wrapping_add(1);
        lumatrix.sleep_ms(POLL_MS);
    }
    while lumatrix.joystick_pressed() {
        lumatrix.sleep_ms(POLL_MS);
    }
    ticks
}

/** Milliseconds between polls while waiting for a button press. */
const POLL_MS: u32 = 5;
