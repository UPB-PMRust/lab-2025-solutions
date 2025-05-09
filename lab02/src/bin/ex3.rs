// +---------------------------------------------------------------------------+
// |                             Lab02 Solution                                |
// +---------------------------------------------------------------------------+

#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_rp::gpio::{Level, Output};
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    // Get a handle to the RP's peripherals.
    let peripherals = embassy_rp::init(Default::default());
    let mut pin = Output::new(peripherals.PIN_2, Level::Low);

    loop {
        pin.toggle();
        Timer::after_millis(300).await;
    }
}
