// +---------------------------------------------------------------------------+
// |                             PM/MA lab skel                                |
// +---------------------------------------------------------------------------+

//! By default, this app prints a "Hello world" message with `defmt`.

#![no_std]
#![no_main]

use cyw43::JoinOptions;
use embassy_executor::Spawner;
use embassy_futures::select::{self, select, select3};
use embassy_net::{tcp::TcpSocket, StackResources};
use embassy_time::Timer;
use embedded_io_async::Write as _;
use static_cell::StaticCell;
use {defmt_rtt as _, panic_probe as _};

// Use the logging macros provided by defmt.
use defmt::*;

const SOCK: usize = 4;
static RESOURCES: StaticCell<StackResources<SOCK>> = StaticCell::<StackResources<SOCK>>::new();
const WIFI_NETWORK: &str = "my_ssid";
const WIFI_PASSWORD: &str = "password";

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    // Get a handle to the RP's peripherals.
    let peripherals = embassy_rp::init(Default::default());

    // Init WiFi driver
    let (net_device, mut control) = embassy_lab_utils::init_wifi!(&spawner, peripherals).await;

    let config = embassy_net::Config::dhcpv4(Default::default());

    // Init network stack
    let stack = embassy_lab_utils::init_network_stack(&spawner, net_device, &RESOURCES, config);

    loop {
        match control
            .join(WIFI_NETWORK, JoinOptions::new(WIFI_PASSWORD.as_bytes()))
            .await
        {
            Ok(_) => break,
            Err(err) => {
                info!("join failed with status={}", err.status);
            }
        }
    }

    info!("waiting for DHCP...");
    while !stack.is_config_up() {
        Timer::after_millis(100).await;
    }
    info!("DHCP is now up!");

    let mut sw4 = embassy_rp::gpio::Input::new(peripherals.PIN_4, embassy_rp::gpio::Pull::None);
    let mut sw5 = embassy_rp::gpio::Input::new(peripherals.PIN_5, embassy_rp::gpio::Pull::None);
    let mut sw6 = embassy_rp::gpio::Input::new(peripherals.PIN_6, embassy_rp::gpio::Pull::None);

    let mut red = embassy_rp::gpio::Output::new(peripherals.PIN_8, embassy_rp::gpio::Level::Low);
    let mut green = embassy_rp::gpio::Output::new(peripherals.PIN_10, embassy_rp::gpio::Level::Low);

    let mut rx_buffer = [0; 4096];
    let mut tx_buffer = [0; 4096];

    loop {
        let mut socket = TcpSocket::new(stack, &mut rx_buffer, &mut tx_buffer);
        // If we want to keep the connection open regardless of inactivity, we can set the timeout
        // to `None`
        socket.set_timeout(None);

        if let Err(e) = socket.accept(6000).await {
            warn!("accept error: {:?}", e);
            continue;
        }

        let mut buf = [0; 4096];

        loop {
            let button_pressed = select3(
                sw4.wait_for_falling_edge(),
                sw5.wait_for_falling_edge(),
                sw6.wait_for_falling_edge(),
            );
            let socket_read = socket.read(&mut buf);

            match select(button_pressed, socket_read).await {
                select::Either::First(select3sw) => {
                    let message = match select3sw {
                        embassy_futures::select::Either3::First(_) => "led:toggle",
                        embassy_futures::select::Either3::Second(_) => "led:red ",
                        embassy_futures::select::Either3::Third(_) => "led:green",
                        // TODO: Add blue here...
                    };

                    match socket.write_all(message.as_bytes()).await {
                        Ok(()) => {}
                        Err(e) => {
                            warn!("write error: {:?}", e);
                            break;
                        }
                    }
                }

                select::Either::Second(recv) => {
                    let n = match recv {
                        Ok(0) => {
                            warn!("read EOF");
                            break;
                        }
                        Ok(n) => n,
                        Err(e) => {
                            warn!("read error: {:?}", e);
                            break;
                        }
                    };

                    let message = core::str::from_utf8(&buf[0..n]).unwrap().trim();
                    match message {
                        "ok" => {
                            red.set_low();
                            green.set_high();
                        }
                        "error" => {
                            red.set_high();
                            green.set_low();
                        }
                        _ => info!("received message not known"),
                    }
                }
            }
        }
    }
}
