// +---------------------------------------------------------------------------+
// |                             Lab03 Solution                                |
// +---------------------------------------------------------------------------+

#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_rp::{
    adc::{self, Adc},
    bind_interrupts,
    gpio::Pull,
    pwm::{self, Pwm, SetDutyCycle},
};
use {defmt_rtt as _, panic_probe as _};

struct RgbLed<'a> {
    red_green: Pwm<'a>,
    blue: Pwm<'a>,
}

#[derive(Clone, Copy)]
struct Color((u8, u8, u8));

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

bind_interrupts!(struct Irqs {
    ADC_IRQ_FIFO => adc::InterruptHandler;
});

const MAX_INTENSITY: usize = 4095;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    // Get a handle to the RP's peripherals.
    let peripherals = embassy_rp::init(Default::default());

    let mut adc = Adc::new(peripherals.ADC, Irqs, adc::Config::default());
    let mut photoresistor = adc::Channel::new_pin(peripherals.PIN_26, Pull::None);

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

    loop {
        adc.read(&mut photoresistor).await.ok().map(|value| {
            let value = value as usize;
            let mid_point = MAX_INTENSITY / 2;

            let red;
            let green;
            let blue;

            if value <= mid_point {
                // Between RED and GREEN
                // `value` is in the interval [0, 2047]
                red = 255 * (mid_point - value) / mid_point;
                green = 255 * value as usize / mid_point;
                blue = 0;
            } else {
                // Between GREEN and BLUE
                // value is in the interval [2048, 4095]
                let value = value - (mid_point + 1); // We do this to bring `value` in the interval [0, 2047]
                red = 0;
                green = 255 * (mid_point - value) / mid_point;
                blue = 255 * value as usize / mid_point;
            }

            rgb_led.display_color(Color((red as u8, green as u8, blue as u8)));
        });
    }
}
