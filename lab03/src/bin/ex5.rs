// +---------------------------------------------------------------------------+
// |                             Lab03 Solution                                |
// +---------------------------------------------------------------------------+

#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_rp::pwm::{self, Pwm};
use embassy_time::Instant;
use fixed::traits::ToFixed;
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    // Get a handle to the RP's peripherals.
    let peripherals = embassy_rp::init(Default::default());

    let mut config = pwm::Config::default(); // Set the calculated TOP value for 50 Hz PWM
    config.top = 0xB71A;

    // Set the clock divider to 64
    config.divider = 64_i32.to_fixed(); // Clock divider = 64

    // Servo timing constants
    const PERIOD_US: usize = 20_000; // 20 ms period for 50 Hz
    const MIN_PULSE_US: usize = 500; // 0.5 ms pulse for 0 degrees
    const MAX_PULSE_US: usize = 2500; // 2.5 ms pulse for 180 degrees

    let min_pulse = (MIN_PULSE_US * config.top as usize) / PERIOD_US;
    let max_pulse = (MAX_PULSE_US * config.top as usize) / PERIOD_US;

    let mut servo = Pwm::new_output_a(peripherals.PWM_SLICE1, peripherals.PIN_2, config.clone());
    let start = Instant::now();
    loop {
        let elapsed = start.elapsed().as_millis();

        let seconds = elapsed / 1000;
        let milliseconds = elapsed % 1000;

        if seconds % 2 == 0 {
            config.compare_a =
                min_pulse as u16 + ((milliseconds * (max_pulse - min_pulse) as u64) / 999) as u16;
        } else {
            let milliseconds = 999 - milliseconds;
            config.compare_a =
                min_pulse as u16 + ((milliseconds * (max_pulse - min_pulse) as u64) / 999) as u16;
        }

        servo.set_config(&config);
    }
}
