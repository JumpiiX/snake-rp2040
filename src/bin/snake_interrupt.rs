/*!
Snake driven by a timer interrupt.

Where the blocking version burns cycles in delay loops, this one hands the
tick to hardware alarm 0 and parks the processor in `wfi` between frames.

The handler does as little as possible: it polls the joystick, advances the
game, and raises a flag. Drawing and the end-of-game animation happen back in
`main`, because writing a frame to the LEDs takes about a millisecond and
would hold off every other interrupt if it ran inside the handler.
*/

#![no_std]
#![no_main]

use core::sync::atomic::{AtomicBool, AtomicU8, Ordering};

use critical_section_lock_mut::LockMut;
use defmt_rtt as _;
use panic_probe as _;
use rp_pico::hal::fugit::MicrosDurationU32;
use rp_pico::hal::timer::{Alarm, Alarm0};
use rp_pico::pac::{self, NVIC, interrupt};

use snake_rp2040::game::{Game, Outcome};
use snake_rp2040::{Lumatrix, input, render};

/** The alarm that paces the game, rescheduled from its own handler. */
static ALARM: LockMut<Alarm0> = LockMut::new();

/** The running game, advanced by the handler and drawn by `main`. */
static GAME: LockMut<Game> = LockMut::new();

/**
Set by the handler once a tick has produced a frame worth drawing.

Only the handler ever sets it and only `main` ever clears it, so plain loads
and stores are enough. Cortex-M0+ has no atomic read-modify-write anyway.
*/
static FRAME_READY: AtomicBool = AtomicBool::new(false);

/** How the last tick ended, encoded as an [`Outcome`] discriminant. */
static LAST_OUTCOME: AtomicU8 = AtomicU8::new(OUTCOME_MOVED);

/** [`Outcome::Moved`] as stored in [`LAST_OUTCOME`]. */
const OUTCOME_MOVED: u8 = 0;

/** [`Outcome::Ate`] as stored in [`LAST_OUTCOME`]. */
const OUTCOME_ATE: u8 = 1;

/** [`Outcome::Died`] as stored in [`LAST_OUTCOME`]. */
const OUTCOME_DIED: u8 = 2;

/** [`Outcome::Won`] as stored in [`LAST_OUTCOME`]. */
const OUTCOME_WON: u8 = 3;

/** Microseconds between game ticks. */
const TICK_US: u32 = 220_000;

/** Display brightness, low enough to stay within the USB power budget. */
const BRIGHTNESS: u8 = 40;

#[rp_pico::entry]
fn main() -> ! {
    let (mut lumatrix, alarm) = Lumatrix::take_with_alarm();
    lumatrix.set_brightness(BRIGHTNESS);

    ALARM.init(alarm);
    GAME.init(Game::new(0));

    /* SAFETY: `TIMER_IRQ_0` is the only interrupt enabled here, its handler
    is defined below, and every static it touches has been initialised above. */
    unsafe {
        NVIC::unmask(pac::Interrupt::TIMER_IRQ_0);
    }

    loop {
        defmt::info!("press the joystick to start");
        let seed = input::wait_for_press_counting(&mut lumatrix);
        defmt::info!("new game, seed {}", seed);

        GAME.with_lock(|game| *game = Game::new(seed));
        FRAME_READY.store(false, Ordering::Release);
        LAST_OUTCOME.store(OUTCOME_MOVED, Ordering::Release);
        schedule_next_tick();

        run_until_game_over(&mut lumatrix);
    }
}

/**
Draws frames as the handler produces them, until the game ends.

The processor sleeps in `wfi` whenever there is nothing to draw, so the only
work between ticks is the frame itself.
*/
fn run_until_game_over(lumatrix: &mut Lumatrix) {
    loop {
        cortex_m::asm::wfi();

        if !FRAME_READY.load(Ordering::Acquire) {
            continue;
        }
        FRAME_READY.store(false, Ordering::Release);

        GAME.with_lock(|game| render::draw(lumatrix, game));

        match LAST_OUTCOME.load(Ordering::Acquire) {
            OUTCOME_ATE => {
                GAME.with_lock(|game| defmt::info!("score {}", game.score()));
            }
            OUTCOME_DIED => {
                GAME.with_lock(|game| defmt::info!("died with score {}", game.score()));
                render::draw_death(lumatrix);
                return;
            }
            OUTCOME_WON => {
                defmt::info!("filled the board");
                render::draw_victory(lumatrix);
                return;
            }
            _ => {}
        }
    }
}

/** Arms the alarm for the next tick. */
fn schedule_next_tick() {
    ALARM.with_lock(|alarm| {
        alarm.enable_interrupt();
        alarm.schedule(MicrosDurationU32::micros(TICK_US)).unwrap();
    });
}

/**
Advances the game by one tick.

Kept deliberately short: poll, tick, flag. Everything that takes real time
runs in `main`.
*/
#[interrupt]
fn TIMER_IRQ_0() {
    ALARM.with_lock(|alarm| {
        alarm.clear_interrupt();
        alarm.schedule(MicrosDurationU32::micros(TICK_US)).unwrap();
    });

    GAME.with_lock(|game| {
        if game.is_over() {
            return;
        }

        if let Some(direction) = read_joystick() {
            game.steer(direction);
        }

        let outcome = match game.tick() {
            Outcome::Moved => OUTCOME_MOVED,
            Outcome::Ate => OUTCOME_ATE,
            Outcome::Died => OUTCOME_DIED,
            Outcome::Won => OUTCOME_WON,
        };

        LAST_OUTCOME.store(outcome, Ordering::Release);
        FRAME_READY.store(true, Ordering::Release);
    });
}

/**
Reads the joystick without owning the board.

`main` keeps the [`Lumatrix`] so it can draw, but the handler still needs the
joystick pins. Reading them is a plain register read of pins nothing else
writes, so going through the peripheral block directly is sound here.
*/
fn read_joystick() -> Option<snake_rp2040::game::direction::Direction> {
    use snake_rp2040::game::direction::Direction;

    /* SAFETY: `SIO` is read-only here, and the GPIO input register reflects
    pin state that no other code in this program drives. */
    let pins = unsafe { (*pac::SIO::ptr()).gpio_in().read().bits() };

    const UP: u32 = 1 << 3;
    const DOWN: u32 = 1 << 6;
    const LEFT: u32 = 1 << 7;
    const RIGHT: u32 = 1 << 2;

    if pins & UP == 0 {
        return Some(Direction::Up);
    }
    if pins & DOWN == 0 {
        return Some(Direction::Down);
    }
    if pins & LEFT == 0 {
        return Some(Direction::Left);
    }
    if pins & RIGHT == 0 {
        return Some(Direction::Right);
    }
    None
}
