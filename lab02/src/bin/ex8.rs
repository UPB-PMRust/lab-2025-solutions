// +---------------------------------------------------------------------------+
// |                             Lab02 Solution                                |
// +---------------------------------------------------------------------------+

#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_rp::gpio::{Input, Level, Output, Pull};
use {defmt_rtt as _, panic_probe as _};

use defmt::*;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    // Get a handle to the RP's peripherals.
    let peripherals = embassy_rp::init(Default::default());
    let mut btn = Input::new(peripherals.PIN_2, Pull::Up);
    let mut red = Output::new(peripherals.PIN_3, Level::High);
    let mut green = Output::new(peripherals.PIN_4, Level::High);
    let mut blue = Output::new(peripherals.PIN_5, Level::High);
    let mut yellow = Output::new(peripherals.PIN_5, Level::High);

    loop {
        btn.wait_for_falling_edge().await;
        led.toggle();
        info!("The button was pressed");
    }
}
