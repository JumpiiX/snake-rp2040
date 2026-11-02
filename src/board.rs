/*!
Driver for the ZHAW Lumatrix board.

Adapted from the course repository.
*/

use core::cmp::Ordering;

use defmt::warn;
use embedded_hal::digital::InputPin;
use rp_pico::hal::{
    self, Clock,
    gpio::{
        FunctionPio0, FunctionSio, Pin, PullDown, PullUp, SioInput,
        bank0::{Gpio2, Gpio3, Gpio6, Gpio7, Gpio8, Gpio9, Gpio19},
    },
    pac::{self, PIO0},
    pio::{PIOExt, SM0},
    timer::{CountDown, Timer},
};
use smart_leds::{SmartLedsWrite, brightness};
use ws2812_pio::Ws2812;

#[path = "billboard.rs"]
pub mod billboard;
#[path = "character.rs"]
pub mod character;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum JoystickPosition {
    #[default]
    Center,
    Up,
    Down,
    Left,
    Right,
}

// Remove these when enabling support for multiple matrices to discover all
// places where code needs to change.
pub const HEIGHT: u8 = 8;
pub const WIDTH: u8 = 8;

/**
A high-level interface for programming the Lumatrix.

Construct it with [`Lumatrix::take`].

To update the color of an LED, you can index the `Lumatrix` with a tuple
of integers (x, y) in the range `0..8` each.

After changing the values of one or several LEDs, don't forget to call
[`apply`](Lumatrix::apply) to actually apply these changes in a batch.

Here is the coordinate system visualized:

```txt
╭─────┬─────┬─────┬─────┬─────┬─────┬─────┬─────╮
│ 7,0 │ 7,1 │ 7,2 │ 7,3 │ 7,4 │ 7,5 │ 7,6 │ 7,7 │
├─────┼─────┼─────┼─────┼─────┼─────┼─────┼─────┤
│ 6,0 │ 6,1 │ 6,2 │ 6,3 │ 6,4 │ 6,5 │ 6,6 │ 6,7 │
├─────┼─────┼─────┼─────┼─────┼─────┼─────┼─────┤
│ 5,0 │ 5,1 │ 5,2 │ 5,3 │ 5,4 │ 5,5 │ 5,6 │ 5,7 │
├─────┼─────┼─────┼─────┼─────┼─────┼─────┼─────┤
│ 4,0 │ 4,1 │ 4,2 │ 4,3 │ 4,4 │ 4,5 │ 4,6 │ 4,7 │
├─────┼─────┼─────┼─────┼─────┼─────┼─────┼─────┤
│ 3,0 │ 3,1 │ 3,2 │ 3,3 │ 3,4 │ 3,5 │ 3,6 │ 3,7 │
├─────┼─────┼─────┼─────┼─────┼─────┼─────┼─────┤
│ 2,0 │ 2,1 │ 2,2 │ 2,3 │ 2,4 │ 2,5 │ 2,6 │ 2,7 │
├─────┼─────┼─────┼─────┼─────┼─────┼─────┼─────┤
│ 1,0 │ 1,1 │ 1,2 │ 1,3 │ 1,4 │ 1,5 │ 1,6 │ 1,7 │
├─────┼─────┼─────┼─────┼─────┼─────┼─────┼─────┤
│ 0,0 │ 0,1 │ 0,2 │ 0,3 │ 0,4 │ 0,5 │ 0,6 │ 0,7 │
╰─────┴─────┴─────┴─────┴─────┴─────┴─────┴─────╯
```
*/
pub struct Lumatrix {
    ws: Ws2812<PIO0, SM0, CountDown<'static>, Pin<Gpio19, FunctionPio0, PullDown>>,
    delay: cortex_m::delay::Delay,

    joystick_up: Pin<Gpio3, FunctionSio<SioInput>, PullUp>,
    joystick_down: Pin<Gpio6, FunctionSio<SioInput>, PullUp>,
    joystick_left: Pin<Gpio7, FunctionSio<SioInput>, PullUp>,
    joystick_right: Pin<Gpio2, FunctionSio<SioInput>, PullUp>,
    joystick_pressed: Pin<Gpio8, FunctionSio<SioInput>, PullUp>,

    switch: Pin<Gpio9, FunctionSio<SioInput>, PullUp>,

    leds: [[(u8, u8, u8); WIDTH as usize]; HEIGHT as usize],

    // Bring down the overall brightness of the strip to not blow
    // the USB power supply: every LED draws ~60mA, RGB means 3 LEDs per
    // ws2812 LED, for 3 LEDs that would be: 3 * 3 * 60mA, which is
    // already 540mA for just 3 white LEDs!
    //
    // default: 50 (~ 20%)
    brightness: u8,
}

impl Lumatrix {
    pub fn take() -> Self {
        // This function corresponds closely to the initilization code of the
        // example from the rp_pico repository.

        // Grab our singleton objects
        let mut pac = pac::Peripherals::take().unwrap();
        let core = pac::CorePeripherals::take().unwrap();

        // Set up the watchdog driver - needed by the clock setup code
        let mut watchdog = hal::Watchdog::new(pac.WATCHDOG);

        // Configure the clocks
        //
        // The default is to generate a 125 MHz system clock
        let clocks = hal::clocks::init_clocks_and_plls(
            rp_pico::XOSC_CRYSTAL_FREQ,
            pac.XOSC,
            pac.CLOCKS,
            pac.PLL_SYS,
            pac.PLL_USB,
            &mut pac.RESETS,
            &mut watchdog,
        )
        .unwrap();

        // The single-cycle I/O block controls our GPIO pins
        let sio = hal::Sio::new(pac.SIO);

        // Set the pins up according to their function on this particular board
        let pins = rp_pico::Pins::new(
            pac.IO_BANK0,
            pac.PADS_BANK0,
            sio.gpio_bank0,
            &mut pac.RESETS,
        );

        // "pull up input" copied from Python version.
        // what is the difference to "pull down" ??
        let joystick_up = pins.gpio3.into_pull_up_input();
        let joystick_down = pins.gpio6.into_pull_up_input();
        let joystick_left = pins.gpio7.into_pull_up_input();
        let joystick_right = pins.gpio2.into_pull_up_input();
        let joystick_pressed = pins.gpio8.into_pull_up_input();

        let switch = pins.gpio9.into_pull_up_input();

        // Setup a delay for the LED blink signals:
        let delay = cortex_m::delay::Delay::new(core.SYST, clocks.system_clock.freq().to_Hz());

        // Create a count down timer for the Ws2812 instance:
        let count_down = unsafe {
            static mut TIMER: Option<Timer> = None;
            TIMER = Some(Timer::new(pac.TIMER, &mut pac.RESETS, &clocks));
            #[allow(static_mut_refs)] // this is fine
            TIMER.as_ref().unwrap().count_down()
        };

        // Split the PIO state machine 0 into individual objects, so that
        // Ws2812 can use it:
        let (mut pio, sm0, _, _, _) = pac.PIO0.split(&mut pac.RESETS);

        // Instanciate a Ws2812 LED strip:
        let ws = Ws2812::new(
            // Use pin 25 on the Raspberry Pi Pico (which is GPIO19 of the rp2040 chip)
            // for the LED data output:
            pins.gpio19.into_function(),
            &mut pio,
            sm0,
            clocks.peripheral_clock.freq(),
            count_down,
        );

        Self {
            ws,
            delay,
            joystick_up,
            joystick_down,
            joystick_left,
            joystick_right,
            joystick_pressed,
            switch,
            leds: Default::default(),
            brightness: 50, // default brightness of about 20%
        }
    }

    /**
    Tell the Lumatrix to display the currently stored color values for
    each LED.

    If you are drawing in an endless loop, consider calling
    [`sleep_ms`](Self::sleep_ms) at some point to slow down the execution.
    */
    pub fn apply(&mut self) {
        self.ws
            .write(brightness(
                self.leds
                    .iter()
                    .rev()
                    .flat_map(|row| row.iter())
                    .map(|&c| c.into()),
                self.brightness,
            ))
            .unwrap();
    }

    /**
    Set the brightness of the display.

    The display is set at a default brightness of `50` (about 20%).
    */
    pub fn set_brightness(&mut self, brightness: u8) {
        self.brightness = brightness
    }

    /** Sleep for the specified amount of milliseconds. */
    pub fn sleep_ms(&mut self, duration: u32) {
        self.delay.delay_ms(duration)
    }

    /** Get the current joystick position. */
    pub fn joystick_position(&mut self) -> JoystickPosition {
        if self.joystick_up.is_low().unwrap() {
            return JoystickPosition::Up;
        }
        if self.joystick_down.is_low().unwrap() {
            return JoystickPosition::Down;
        }
        if self.joystick_left.is_low().unwrap() {
            return JoystickPosition::Left;
        }
        if self.joystick_right.is_low().unwrap() {
            return JoystickPosition::Right;
        }
        JoystickPosition::Center
    }

    /** Check if the joystick is currently pressed. */
    pub fn joystick_pressed(&mut self) -> bool {
        self.joystick_pressed.is_low().unwrap()
    }

    /** Get the current value of the switch. */
    pub fn switch(&mut self) -> bool {
        self.switch.is_low().unwrap()
    }

    /**
    Set every LED to a single color at the same time.

    You still need to call [`apply`](Self::apply) afterwards.
    */
    pub fn fill(&mut self, color: (u8, u8, u8)) {
        for x in 0..WIDTH as usize {
            for y in 0..HEIGHT as usize {
                self[(x, y)] = color;
            }
        }
    }

    /** Turn off all LEDs. */
    pub fn clear(&mut self) {
        self.fill((0, 0, 0));
    }

    /**
    Set a list of LEDs to a single color.

    LEDs are specified as an iterator of coordinates (x, y).
    */
    pub fn draw_coordinates(
        &mut self,
        coords: &mut dyn Iterator<Item = (usize, usize)>,
        color: (u8, u8, u8),
    ) {
        for (x, y) in coords {
            self[(x, y)] = color;
        }
    }

    /**
    Draw a bitmap file with a color depth of 24 bit.

    Bitmap support is a bit shaky. If you download and try your own bitmap,
    there's a good chance it won't work.

    bitmap format: <https://en.wikipedia.org/wiki/BMP_file_format>
    */
    pub fn draw_bitmap(&mut self, bitmap: &[u8]) {
        let color_depth = u16::from_le_bytes(bitmap[28..30].try_into().unwrap());
        let bitmap_size = u32::from_le_bytes(bitmap[2..6].try_into().unwrap());
        let bitmap_offset = u32::from_le_bytes(bitmap[10..14].try_into().unwrap());
        let image_size = u32::from_le_bytes(bitmap[34..38].try_into().unwrap());
        let bitmap_height = u16::from_be_bytes(bitmap[22..24].try_into().unwrap()) as u32;
        // let upper_left_origin = bitmap_height < 0;
        let pic = &bitmap[bitmap_offset as usize..];
        let bitmap_width = (image_size / 3) / bitmap_height;

        if bitmap_height > HEIGHT as u32 || bitmap_width > WIDTH as u32 {
            // triggers false positives for some reason
            // warn!(
            //     "bitmap is larger than matrix: {}x{}",
            //     bitmap_width, bitmap_height
            // );
        }
        if (color_depth) != 24 {
            panic!(
                "Wrong color-depth ({color_depth}) detected. Use bitmaps with a color-depth of 24 bits."
            );
        }
        if (bitmap_size) != 246 {
            warn!("The bitmap size is different than expected. The image may be defective.");
        }
        for x in 0..WIDTH as usize {
            for y in 0..HEIGHT as usize {
                let i = (y * WIDTH as usize + x) * 3;
                let y = HEIGHT as usize - y - 1;
                self[(x, y)] = (pic[i + 2], pic[i + 1], pic[i]);
            }
        }
    }

    /**
    Draw a frame of a horizontal billboard at a specified offset.

    Construct such a billboard with [`billboard::horizontal`].

    This function only draws a single frame, you probably want to
    loob over offsets and draw each frame with a desired delay using
    [`sleep_ms`](Self::sleep_ms).

    See the module documentation of [`billboard`] for more information.
    */
    pub fn draw_horizontal_billboard_frame(
        &mut self,
        billboard: billboard::Billboard,
        offset: usize,
    ) {
        for (x, column) in (offset..offset + WIDTH as usize).enumerate() {
            for y in 0..HEIGHT as usize {
                self[(x, y)] = match billboard
                    .get(column)
                    .map(|col| col[HEIGHT as usize - y - 1])
                {
                    Some(true) => color::WHITE,
                    _ => color::BLACK,
                }
            }
        }
    }

    /**
    Draw a frame of a vertical billboard at a specified offset.

    Construct such a billboard with [`billboard::vertical`].

    This function is analogous to [`draw_horizontal_billboard_frame`](Self::draw_horizontal_billboard_frame).
    */
    pub fn draw_vertical_billboard_frame(
        &mut self,
        billboard: billboard::Billboard,
        offset: usize,
    ) {
        for (y, row) in (offset..offset + HEIGHT as usize).enumerate() {
            let y = HEIGHT as usize - y - 1;
            for x in 0..WIDTH as usize {
                self[(x, y)] = match billboard.get(row).map(|row| row[x]) {
                    Some(true) => color::WHITE,
                    _ => color::BLACK,
                }
            }
        }
    }

    /**
    Draw a frame of a strip of text at a specified offset.

    Construct such a strip of text with [`character::convert_str`].

    Like [`draw_horizontal_billboard_frame`](Self::draw_horizontal_billboard_frame),
    this function only draws a single frame. You probably want to loob
    over offsets and draw each frame with a desired delay using [`sleep_ms`](Self::sleep_ms).
    */
    pub fn draw_text_billboard_frame(
        &mut self,
        text: &[character::Character],
        frame_offset: usize,
    ) {
        // TODO: custom color support

        self.clear();

        for &c in text.iter() {
            if c.offset + c.width < frame_offset {
                continue;
            } else if frame_offset + WIDTH as usize - 1 < c.offset {
                break;
            }
            let mut coords = c.coordinates.iter().copied().filter_map(|(mut x, y)| {
                // remove out-of-bounds coordinates and apply offset
                match frame_offset.cmp(&c.offset) {
                    Ordering::Less => {
                        x += c.offset - frame_offset;
                        if x >= WIDTH as usize {
                            // Character is partially in frame, but this
                            // specific pixel is beyond the right border of
                            // the frame.
                            return None;
                        }
                    }
                    Ordering::Equal => {}
                    Ordering::Greater => {
                        let offset_diff = frame_offset - c.offset;
                        if offset_diff > x {
                            // Character is partially in frame, but this
                            // specific pixel is beyond the left border of
                            // the frame.
                            return None;
                        }
                        x -= offset_diff;
                    }
                }
                Some((x, y))
            });

            self.draw_coordinates(&mut coords, color::WHITE);
        }
    }
}

impl core::ops::Index<(usize, usize)> for Lumatrix {
    type Output = (u8, u8, u8);

    fn index(&self, (x, y): (usize, usize)) -> &Self::Output {
        assert!((0..HEIGHT as usize).contains(&x));
        assert!((0..WIDTH as usize).contains(&y));
        &self.leds[HEIGHT as usize - y - 1][x]
    }
}
impl core::ops::IndexMut<(usize, usize)> for Lumatrix {
    fn index_mut(&mut self, (x, y): (usize, usize)) -> &mut Self::Output {
        assert!((0..HEIGHT as usize).contains(&x));
        assert!((0..WIDTH as usize).contains(&y));
        &mut self.leds[HEIGHT as usize - y - 1][x]
    }
}

/**
Contains a number of predefined color values.

You can set an LED to one of these colors like this:

```
*lumatrix.get_mut(2, 6) = color::PURPLE;
```
*/
pub mod color {
    pub const YELLOW: (u8, u8, u8) = (255, 255, 0);
    pub const ORANGE: (u8, u8, u8) = (255, 165, 0);
    pub const RED: (u8, u8, u8) = (255, 0, 0);
    pub const PURPLE: (u8, u8, u8) = (128, 0, 128);
    pub const PINK: (u8, u8, u8) = (255, 0, 255);
    pub const BLUE: (u8, u8, u8) = (0, 0, 255);
    pub const TEAL: (u8, u8, u8) = (0, 128, 128);
    pub const AQUA: (u8, u8, u8) = (0, 255, 255);
    pub const LIME: (u8, u8, u8) = (0, 255, 0);
    pub const GREEN: (u8, u8, u8) = (0, 128, 0);
    pub const LIGHT_GREY: (u8, u8, u8) = (119, 136, 153);
    pub const GREY: (u8, u8, u8) = (100, 100, 100);
    pub const BROWN: (u8, u8, u8) = (139, 69, 19);
    pub const LIGHT_BROWN: (u8, u8, u8) = (205, 133, 63);
    pub const WHITE: (u8, u8, u8) = (255, 255, 255);
    pub const BLACK: (u8, u8, u8) = (0, 0, 0);
}

/** Contains a number of predefined bitmaps. */
pub mod bitmap {
    pub static APPLE: &[u8] = include_bytes!("../bitmaps/apple.bmp");
    pub static BAT: &[u8] = include_bytes!("../bitmaps/bat.bmp");
    pub static BIG_IMG: &[u8] = include_bytes!("../bitmaps/big_img.bmp");
    pub static BIRD: &[u8] = include_bytes!("../bitmaps/bird.bmp");
    pub static CHICKEN: &[u8] = include_bytes!("../bitmaps/chicken.bmp");
    pub static CRAB: &[u8] = include_bytes!("../bitmaps/crab.bmp");
    pub static DINO: &[u8] = include_bytes!("../bitmaps/dino.bmp");
    pub static DRINK: &[u8] = include_bytes!("../bitmaps/drink.bmp");
    pub static DUCK: &[u8] = include_bytes!("../bitmaps/duck.bmp");
    pub static ERLENMEYER: &[u8] = include_bytes!("../bitmaps/erlenmeyer.bmp");
    pub static FOX: &[u8] = include_bytes!("../bitmaps/fox.bmp");
    pub static MC_CREEPER: &[u8] = include_bytes!("../bitmaps/mc_creeper.bmp");
    pub static MC_PIG: &[u8] = include_bytes!("../bitmaps/mc_pig.bmp");
    pub static MOB_1: &[u8] = include_bytes!("../bitmaps/mob_1.bmp");
    pub static MOUSE: &[u8] = include_bytes!("../bitmaps/mouse.bmp");
    pub static MUSHROOM: &[u8] = include_bytes!("../bitmaps/mushroom.bmp");
    pub static PIKACHU: &[u8] = include_bytes!("../bitmaps/pikachu.bmp");
    pub static RABBIT: &[u8] = include_bytes!("../bitmaps/rabbit.bmp");
    pub static SKULL: &[u8] = include_bytes!("../bitmaps/skull.bmp");
    pub static TEST_GRID: &[u8] = include_bytes!("../bitmaps/test_grid.bmp");
    pub static WINE: &[u8] = include_bytes!("../bitmaps/wine.bmp");
}

/**
Returns an iterator over the coordinates of all LEDs. Useful for avoiding
nested loops.
*/
pub fn all_led_coordinates() -> impl Iterator<Item = (usize, usize)> {
    (0..HEIGHT as usize).flat_map(|y| (0..WIDTH as usize).map(move |x| (x, y)))
}

/** Pico-specific sinus function. */
pub fn sin(x: f32) -> f32 {
    hal::rom_data::float_funcs::fsin::ptr()(x)
}
