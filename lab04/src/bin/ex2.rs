// +---------------------------------------------------------------------------+
// |                             Lab04 Solution                                |
// +---------------------------------------------------------------------------+

#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_futures::yield_now;
use embassy_rp::gpio::{AnyPin, Level, Output, Pin};
use embassy_time::{Duration, Timer};
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::task(pool_size = 4)]
async fn blink(pin: AnyPin, freq: u64) {
    let period = Duration::from_hz(freq * 2);
    let mut led = Output::new(pin, Level::High);

    loop {
        led.toggle();
        Timer::after(period).await;
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    // Get a handle to the RP's peripherals.
    let peripherals = embassy_rp::init(Default::default());

    let yellow = peripherals.PIN_2.degrade();
    spawner.spawn(blink(yellow, 3)).unwrap();

    let red = peripherals.PIN_3.degrade();
    spawner.spawn(blink(red, 4)).unwrap();

    let green = peripherals.PIN_4.degrade();
    spawner.spawn(blink(green, 5)).unwrap();

    let blue = peripherals.PIN_5.degrade();
    spawner.spawn(blink(blue, 1)).unwrap();

    loop {
        yield_now().await;
    }
}
