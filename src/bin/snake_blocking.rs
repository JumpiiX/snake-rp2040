/*!
Snake driven by a blocking loop.

The simplest of the three styles: one loop polls the joystick, sleeps out the
rest of the tick, then advances the game. Easy to follow, but the processor
spins through every delay instead of idling.
*/

#![no_std]
#![no_main]

use defmt_rtt as _;
use panic_probe as _;

use snake_rp2040::game::{Game, Outcome};
use snake_rp2040::{Lumatrix, input, render};

/** Milliseconds between game ticks. */
const TICK_MS: u32 = 220;

/** Milliseconds between joystick polls within a tick. */
const POLL_MS: u32 = 10;

/** Display brightness, low enough to stay within the USB power budget. */
const BRIGHTNESS: u8 = 40;

#[rp_pico::entry]
fn main() -> ! {
    let mut lumatrix = Lumatrix::take();
    lumatrix.set_brightness(BRIGHTNESS);

    defmt::info!("press the joystick to start");

    loop {
        let seed = input::wait_for_press_counting(&mut lumatrix);
        play(&mut lumatrix, seed);
    }
}

/** Runs one game to its end. */
fn play(lumatrix: &mut Lumatrix, seed: u32) {
    let mut game = Game::new(seed);
    defmt::info!("new game, seed {}", seed);

    while !game.is_over() {
        render::draw(lumatrix, &game);
        poll_until_next_tick(lumatrix, &mut game);

        match game.tick() {
            Outcome::Moved => {}
            Outcome::Ate => defmt::info!("score {}", game.score()),
            Outcome::Died => {
                defmt::info!("died with score {}", game.score());
                render::draw_death(lumatrix);
            }
            Outcome::Won => {
                defmt::info!("filled the board");
                render::draw_victory(lumatrix);
            }
        }
    }
}

/**
Polls the joystick for one tick's worth of time.

Steering on every poll rather than once per tick means a flick of the stick
registers even if the player lets go before the tick lands.
*/
fn poll_until_next_tick(lumatrix: &mut Lumatrix, game: &mut Game) {
    for _ in 0..TICK_MS / POLL_MS {
        if let Some(direction) = input::read(lumatrix) {
            game.steer(direction);
        }
        lumatrix.sleep_ms(POLL_MS);
    }
}
