// +---------------------------------------------------------------------------+
// |                             PM/MA lab skel                                |
// +---------------------------------------------------------------------------+

//! By default, this app prints a "Hello world" message with `defmt`.

#![no_std]
#![no_main]

use cyw43::JoinOptions;
use embassy_executor::Spawner;
use embassy_net::{tcp::TcpSocket, StackResources};
use embassy_time::Timer;
use embedded_io_async::Write;
use static_cell::StaticCell;
use {defmt_rtt as _, panic_probe as _};

// Use the logging macros provided by defmt.
use defmt::*;

const SOCK: usize = 4;
static RESOURCES: StaticCell<StackResources<SOCK>> = StaticCell::<StackResources<SOCK>>::new();
const WIFI_NETWORK: &str = "UPB-Guest";

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
        match control.join(WIFI_NETWORK, JoinOptions::new_open()).await {
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
    let ip = stack.config_v4().unwrap().address.address();
    info!("IP Addr: {:?}", ip);

    let mut red = embassy_rp::gpio::Output::new(peripherals.PIN_8, embassy_rp::gpio::Level::Low);
    let mut blue = embassy_rp::gpio::Output::new(peripherals.PIN_9, embassy_rp::gpio::Level::Low);
    let mut green = embassy_rp::gpio::Output::new(peripherals.PIN_10, embassy_rp::gpio::Level::Low);

    let mut rx_buffer = [0; 4096];
    let mut tx_buffer = [0; 4096];

    let mut chosen_led = &mut red;

    loop {
        let mut socket = TcpSocket::new(stack, &mut rx_buffer, &mut tx_buffer);
        // If we want to keep the connection open regardless of inactivity, we can set the timeout
        // to `None`
        socket.set_timeout(None);

        if let Err(e) = socket.accept(6000).await {
            warn!("accept error: {:?}", e);
            continue;
        }

        info!("Connected");

        let mut buf = [0; 4096];

        loop {
            let n = match socket.read(&mut buf).await {
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
                "led:toggle" => chosen_led.toggle(),

                "led:red" => chosen_led = &mut red,
                "led:blue" => chosen_led = &mut blue,
                "led:green" => chosen_led = &mut green,

                _ => {
                    let _ = socket.write_all(b"error\n").await;
                    continue;
                }
            }

            let _ = socket.write_all(b"ok\n").await;
        }
    }
}
