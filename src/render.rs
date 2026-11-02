/*!
Drawing a game onto the LED matrix.

This is the only place that knows both the game rules and the board, which
keeps [`crate::game`] free of hardware types.
*/

use crate::board::{Lumatrix, color};
use crate::game::Game;

/** Colour of the snake's head. */
const HEAD: (u8, u8, u8) = (0, 255, 80);

/** Colour of the body segments behind the head. */
const BODY: (u8, u8, u8) = (0, 90, 30);

/** Colour of the food. */
const FOOD: (u8, u8, u8) = color::RED;

/** Colour the board flashes in when the snake dies. */
const DEATH: (u8, u8, u8) = (120, 0, 0);

/** Colour the board flashes in when the board has been filled. */
const VICTORY: (u8, u8, u8) = (0, 120, 120);

/** Milliseconds between the flashes of an end-of-game animation. */
const FLASH_MS: u32 = 120;

/** How many times the board flashes when a game ends. */
const FLASH_COUNT: u8 = 4;

/**
Draws the current state of `game`.

The head is brighter than the rest of the body so the player can tell which
way the snake is pointing at a glance.
*/
pub fn draw(lumatrix: &mut Lumatrix, game: &Game) {
    lumatrix.clear();

    let food = game.food();
    lumatrix[(food.x as usize, food.y as usize)] = FOOD;

    for (index, segment) in game.snake().segments().enumerate() {
        let colour = if index == 0 { HEAD } else { BODY };
        lumatrix[(segment.x as usize, segment.y as usize)] = colour;
    }

    lumatrix.apply();
}

/** Flashes the board to announce that the snake died. */
pub fn draw_death(lumatrix: &mut Lumatrix) {
    flash(lumatrix, DEATH);
}

/** Flashes the board to announce that the player filled it. */
pub fn draw_victory(lumatrix: &mut Lumatrix) {
    flash(lumatrix, VICTORY);
}

/** Alternates the whole board between `colour` and black. */
fn flash(lumatrix: &mut Lumatrix, colour: (u8, u8, u8)) {
    for _ in 0..FLASH_COUNT {
        lumatrix.fill(colour);
        lumatrix.apply();
        lumatrix.sleep_ms(FLASH_MS);
        lumatrix.clear();
        lumatrix.apply();
        lumatrix.sleep_ms(FLASH_MS);
    }
}
