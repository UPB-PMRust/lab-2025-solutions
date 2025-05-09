// +---------------------------------------------------------------------------+
// |                             Lab02 Solution                                |
// +---------------------------------------------------------------------------+

#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_rp::gpio::{Input, Level, Output, Pull};
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    // Get a handle to the RP's peripherals.
    let peripherals = embassy_rp::init(Default::default());
    let mut btn = Input::new(peripherals.PIN_2, Pull::Up);
    let mut red = Output::new(peripherals.PIN_3, Level::High);
    let mut green = Output::new(peripherals.PIN_4, Level::Low); // initial traffic light
    let mut blue = Output::new(peripherals.PIN_5, Level::High);
    let mut yellow = Output::new(peripherals.PIN_6, Level::High);

    loop {
        btn.wait_for_low().await;

        green.set_high();
        yellow.set_low();

        Timer::after_secs(1).await;

        red.set_low();
        yellow.set_high();

        for _ in 0..5 {
            blue.toggle();
            Timer::after_millis(500).await;
        }

        red.set_high();
        blue.set_high();
        green.set_low();
    }
}
