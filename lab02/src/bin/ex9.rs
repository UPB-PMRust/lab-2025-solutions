// +---------------------------------------------------------------------------+
// |                             Lab02 Solution                                |
// +---------------------------------------------------------------------------+

#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_rp::gpio::{Level, Output};
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

struct MorseDisplay<'a>([Output<'a>; 3]);

#[derive(PartialEq, Eq)]
enum MorseSignal {
    Dit,
    Dot,
    None,
}

struct MorseChar([MorseSignal; 5]);

impl TryFrom<char> for MorseChar {
    type Error = ();

    fn try_from(value: char) -> Result<Self, Self::Error> {
        let signals = match value.to_ascii_lowercase() {
            'h' => [
                MorseSignal::Dot,
                MorseSignal::Dot,
                MorseSignal::Dot,
                MorseSignal::Dot,
                MorseSignal::None,
            ],
            'e' => [
                MorseSignal::Dot,
                MorseSignal::None,
                MorseSignal::None,
                MorseSignal::None,
                MorseSignal::None,
            ],
            'l' => [
                MorseSignal::Dot,
                MorseSignal::Dit,
                MorseSignal::Dot,
                MorseSignal::Dot,
                MorseSignal::None,
            ],
            'o' => [
                MorseSignal::Dit,
                MorseSignal::Dit,
                MorseSignal::Dit,
                MorseSignal::None,
                MorseSignal::None,
            ],
            // ...
            _ => Err(())?
        };

        Ok(MorseChar(signals))
    }
}

impl<'a> MorseDisplay<'a> {
    async fn display(&mut self, text: &str) {
        for c in text.chars() {
            // Word break
            if c == ' ' {
                Timer::after_secs(3).await;
            } else {
                self.display_char(c).await;
            }
        }
    }

    async fn display_char(&mut self, c: char) {
        let conversion_result = TryInto::<MorseChar>::try_into(c);
        let Ok(morse) = conversion_result else {
            // We simply ignore unsupported characters
            return;
        };

        for signal in morse.0 {
            match signal {
                MorseSignal::None => continue,
                MorseSignal::Dot => {
                    self.0[1].set_low();
                },
                MorseSignal::Dit => {
                    self.0[0].set_low();
                    self.0[1].set_low();
                    self.0[2].set_low();
                },
            }

            // 90% of one second for the signal (dit or dot)
            Timer::after_millis(900).await;
            // 10% break
            self.clear();
            Timer::after_millis(100).await
        }
    }

    fn clear(&mut self) {
        self.0[0].set_high();
        self.0[1].set_high();
        self.0[2].set_high();
    }
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    // Get a handle to the RP's peripherals.
    let peripherals = embassy_rp::init(Default::default());
    let led3 = Output::new(peripherals.PIN_3, Level::High);
    let led4 = Output::new(peripherals.PIN_4, Level::High);
    let led5 = Output::new(peripherals.PIN_5, Level::High);

    let mut morse = MorseDisplay([led3, led4, led5]);

    loop {
        morse.display("Hello").await;
        Timer::after_secs(5).await;
    }
}
