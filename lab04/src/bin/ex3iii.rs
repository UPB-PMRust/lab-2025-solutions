// +---------------------------------------------------------------------------+
// |                             Lab04 Solution                                |
// +---------------------------------------------------------------------------+

#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_futures::select::select;
use embassy_rp::{
    gpio::{AnyPin, Input, Pin, Pull},
    pwm::{Pwm, SetDutyCycle},
};
use embassy_sync::{blocking_mutex::raw::ThreadModeRawMutex, signal::Signal};
use {defmt_rtt as _, panic_probe as _};

static SIGNAL: Signal<ThreadModeRawMutex, u8> = Signal::new();

#[embassy_executor::task]
async fn button_task(increase: AnyPin, decrease: AnyPin) {
    let mut btn1 = Input::new(increase, Pull::None);
    let mut btn2 = Input::new(decrease, Pull::None);

    let mut intensity: u8 = 50;

    loop {
        intensity = match select(btn1.wait_for_falling_edge(), btn2.wait_for_falling_edge()).await {
            embassy_futures::select::Either::First(_) => intensity.saturating_sub(10),
            embassy_futures::select::Either::Second(_) => core::cmp::min(intensity + 10, 100),
        };

        SIGNAL.signal(intensity);
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    // Get a handle to the RP's peripherals.
    let peripherals = embassy_rp::init(Default::default());
    let mut led = Pwm::new_output_a(
        peripherals.PWM_SLICE1,
        peripherals.PIN_2,
        Default::default(),
    );

    let sw4 = peripherals.PIN_4.degrade();
    let sw5 = peripherals.PIN_5.degrade();

    spawner.spawn(button_task(sw4, sw5)).unwrap();

    loop {
        let intensity = SIGNAL.wait().await;
        led.set_duty_cycle_percent(intensity).unwrap();
    }
}
