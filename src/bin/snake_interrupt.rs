/*!
Snake driven by a timer interrupt.

Where the blocking version burns cycles in delay loops, this one hands the
tick to hardware alarm 0 and parks the processor in `wfi` between frames. The
board and the game live in `LockMut` statics so the handler can reach them,
and `main` does nothing after setup.
*/

#![no_std]
#![no_main]

use critical_section_lock_mut::LockMut;
use defmt_rtt as _;
use panic_probe as _;
use rp_pico::hal::fugit::MicrosDurationU32;
use rp_pico::hal::timer::{Alarm, Alarm0};
use rp_pico::pac::{self, NVIC, interrupt};

use snake_rp2040::game::{Game, Outcome};
use snake_rp2040::{Lumatrix, input, render};

/** The board, shared between setup and the timer handler. */
static LUMATRIX: LockMut<Lumatrix> = LockMut::new();

/** The alarm that paces the game, rescheduled from its own handler. */
static ALARM: LockMut<Alarm0> = LockMut::new();

/** The running game, advanced once per alarm. */
static GAME: LockMut<Game> = LockMut::new();

/** Microseconds between game ticks. */
const TICK_US: u32 = 220_000;

/** Display brightness, low enough to stay within the USB power budget. */
const BRIGHTNESS: u8 = 40;

#[rp_pico::entry]
fn main() -> ! {
    let (mut lumatrix, alarm) = Lumatrix::take_with_alarm();
    lumatrix.set_brightness(BRIGHTNESS);

    defmt::info!("press the joystick to start");
    let seed = input::wait_for_press_counting(&mut lumatrix);
    defmt::info!("new game, seed {}", seed);

    LUMATRIX.init(lumatrix);
    GAME.init(Game::new(seed));
    ALARM.init(alarm);

    ALARM.with_lock(|alarm| {
        alarm.enable_interrupt();
        alarm.schedule(MicrosDurationU32::micros(TICK_US)).unwrap();
    });

    /* SAFETY: `TIMER_IRQ_0` is the only interrupt enabled here, its handler
    is defined below, and every static it touches has been initialised above. */
    unsafe {
        NVIC::unmask(pac::Interrupt::TIMER_IRQ_0);
    }

    loop {
        cortex_m::asm::wfi();
    }
}

/**
Advances the game by one tick.

Steering happens here too: polling once per tick is coarser than the blocking
version, but it keeps the handler short and leaves the processor idle the
rest of the time.
*/
#[interrupt]
fn TIMER_IRQ_0() {
    ALARM.with_lock(|alarm| {
        alarm.clear_interrupt();
        alarm.schedule(MicrosDurationU32::micros(TICK_US)).unwrap();
    });

    LUMATRIX.with_lock(|lumatrix| {
        GAME.with_lock(|game| {
            if let Some(direction) = input::read(lumatrix) {
                game.steer(direction);
            }

            render::draw(lumatrix, game);

            match game.tick() {
                Outcome::Moved => {}
                Outcome::Ate => defmt::info!("score {}", game.score()),
                Outcome::Died => defmt::info!("died with score {}", game.score()),
                Outcome::Won => defmt::info!("filled the board"),
            }
        });
    });
}
