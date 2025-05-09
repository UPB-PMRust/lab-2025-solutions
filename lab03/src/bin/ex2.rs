// +---------------------------------------------------------------------------+
// |                             Lab03 Solution                                |
// +---------------------------------------------------------------------------+

#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_rp::{
    adc::{Adc, Channel, InterruptHandler},
    bind_interrupts,
    gpio::Pull,
    pwm::{self, Pwm},
};
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    ADC_IRQ_FIFO => InterruptHandler;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    // Get a handle to the RP's peripherals.
    let peripherals = embassy_rp::init(Default::default());

    let mut adc = Adc::new(peripherals.ADC, Irqs, Default::default());
    let mut potentiometer = Channel::new_pin(peripherals.PIN_26, Pull::None);

    let mut config: pwm::Config = Default::default();
    config.top = 4095; // Maximum adc value, due to the 12 bit resolution.

    let mut led = Pwm::new_output_a(peripherals.PWM_SLICE1, peripherals.PIN_2, config.clone());

    loop {
        config.compare_a = adc.read(&mut potentiometer).await.unwrap();
        led.set_config(&config);
    }
}
