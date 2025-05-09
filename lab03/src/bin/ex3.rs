// +---------------------------------------------------------------------------+
// |                             Lab03 Solution                                |
// +---------------------------------------------------------------------------+

#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_rp::{
    gpio::{Input, Pull},
    pwm::{self, Pwm, SetDutyCycle},
};
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

struct RgbLed<'a> {
    red_green: Pwm<'a>,
    blue: Pwm<'a>,
}

#[derive(Clone, Copy)]
struct Color((u8, u8, u8));

const RED: Color = Color((255, 0, 0));
const YELLOW: Color = Color((255, 255, 0));
const BLUE: Color = Color((0, 0, 255));

impl<'a> RgbLed<'a> {
    fn new(mut red_green: Pwm<'a>, mut blue: Pwm<'a>) -> RgbLed<'a> {
        red_green.set_duty_cycle_fully_on().unwrap();
        blue.set_duty_cycle_fully_on().unwrap();

        RgbLed { red_green, blue }
    }

    fn display_color(&mut self, color: Color) {
        let (r, g, b) = color.0;

        // invert PWM for the common anode LED.
        let r = 255 - r as u16;
        let g = 255 - g as u16;
        let b = 255 - b as u16;

        let mut config = pwm::Config::default();
        config.top = 255;
        config.compare_a = r;
        config.compare_b = g;
        self.red_green.set_config(&config);

        self.blue.set_duty_cycle_fraction(b, 255).unwrap();
    }
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    // Get a handle to the RP's peripherals.
    let peripherals = embassy_rp::init(Default::default());

    let mut btn = Input::new(peripherals.PIN_5, Pull::Up);

    let red_green = Pwm::new_output_ab(
        peripherals.PWM_SLICE1,
        peripherals.PIN_2,
        peripherals.PIN_3,
        Default::default(),
    );
    let blue = Pwm::new_output_a(
        peripherals.PWM_SLICE2,
        peripherals.PIN_4,
        Default::default(),
    );

    let mut rgb_led = RgbLed::new(red_green, blue);

    let mut colors = [RED, YELLOW, BLUE].into_iter().cycle();
    loop {
        let color = colors.next().unwrap();
        rgb_led.display_color(color);
        btn.wait_for_falling_edge().await;

        // Delay needed for debounce
        Timer::after_millis(200).await;
    }
}
