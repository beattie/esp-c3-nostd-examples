//! LED Blinky Example for ESP32-C3
//!
//! Simple example that blinks an LED connected to GPIO8.
//!
//! Hardware:
//! - ESP32-C3 board with LED on GPIO8
//! - Airtip ESP32-C3 OLED board has LED on GPIO8
//! - Or external LED connected to GPIO8 (with current-limiting resistor)
//!
//! Run with:
//! cargo run --release --example blinky

#![no_std]
#![no_main]

use esp_backtrace as _;
use esp_hal::{
    delay::Delay,
    gpio::{Level, Output, OutputConfig},
};

esp_bootloader_esp_idf::esp_app_desc!();

#[esp_hal::main]
fn main() -> ! {
    esp_println::logger::init_logger_from_env();
    log::info!("Starting LED Blinky Example");

    let peripherals = esp_hal::init(esp_hal::Config::default());

    // Configure GPIO8 as output for LED
    let mut led = Output::new(peripherals.GPIO8, Level::Low, OutputConfig::default());

    log::info!("LED initialized on GPIO8");
    log::info!("Starting blink pattern...");

    let delay = Delay::new();

    // Blink forever
    loop {
        log::info!("LED ON");
        led.set_high();
        delay.delay_millis(500);

        log::info!("LED OFF");
        led.set_low();
        delay.delay_millis(500);
    }
}
