// +---------------------------------------------------------------------------+
// |                             Lab04 Solution                                |
// +---------------------------------------------------------------------------+

#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_rp::{
    gpio::{AnyPin, Input, Pin, Pull},
    pwm::{Pwm, SetDutyCycle},
};
use embassy_sync::{blocking_mutex::raw::ThreadModeRawMutex, channel::Channel};
use {defmt_rtt as _, panic_probe as _};

#[derive(Clone, Copy)]
enum Command {
    Increase,
    Decrease,
}

static CHANNEL: Channel<ThreadModeRawMutex, Command, 64> = Channel::new();

#[embassy_executor::task(pool_size = 2)]
async fn button_task(pin: AnyPin, cmd: Command) {
    let mut btn = Input::new(pin, Pull::None);

    loop {
        btn.wait_for_falling_edge().await;
        CHANNEL.send(cmd).await;
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
    let mut intensity = 50;
    led.set_duty_cycle_percent(intensity).unwrap();

    let sw4 = peripherals.PIN_4.degrade();
    let sw5 = peripherals.PIN_5.degrade();

    spawner.spawn(button_task(sw4, Command::Increase)).unwrap();
    spawner.spawn(button_task(sw5, Command::Decrease)).unwrap();

    loop {
        let cmd = CHANNEL.receive().await;

        match cmd {
            Command::Increase => intensity = intensity.saturating_sub(10),
            Command::Decrease => intensity = core::cmp::min(intensity + 10, 100),
        }

        led.set_duty_cycle_percent(intensity).unwrap();
    }
}
