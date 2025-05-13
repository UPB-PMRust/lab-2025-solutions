// +---------------------------------------------------------------------------+
// |                             Lab04 Solution                                |
// +---------------------------------------------------------------------------+

#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_futures::{
    join::join,
    select::{Either, select},
};
use embassy_rp::gpio::{Input, Level, Output};
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    // Get a handle to the RP's peripherals.
    let peripherals = embassy_rp::init(Default::default());

    let mut red = Output::new(peripherals.PIN_2, Level::High);
    let mut green = Output::new(peripherals.PIN_3, Level::High);
    let mut yellow = Output::new(peripherals.PIN_4, Level::High);

    let mut sw4 = Input::new(peripherals.PIN_5, embassy_rp::gpio::Pull::None);
    let mut sw5 = Input::new(peripherals.PIN_6, embassy_rp::gpio::Pull::None);

    loop {
        // GREEN
        green.set_low();
        let _ = select(
            Timer::after_secs(5),
            join(sw4.wait_for_falling_edge(), sw5.wait_for_falling_edge()),
        )
        .await;
        green.set_high();

        // YELLOW
        for _ in 0..8 {
            yellow.toggle();

            let event = select(
                Timer::after_millis(1000 / 8),
                join(sw4.wait_for_falling_edge(), sw5.wait_for_falling_edge()),
            )
            .await;
            if let Either::Second(_) = event {
                yellow.set_high();
                break;
            }
        }

        // RED
        red.set_low();
        let _ = select(
            Timer::after_secs(2),
            join(sw4.wait_for_falling_edge(), sw5.wait_for_falling_edge()),
        )
        .await;
        red.set_high()
    }
}
