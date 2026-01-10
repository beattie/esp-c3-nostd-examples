//! LED Blinky Example for ESP32-C3
//!
//! Simple example that blinks an LED connected to GPIO8.
//! Demonstrates the logging framework with different log levels.
//!
//! Hardware:
//! - ESP32-C3 board with LED on GPIO8
//! - Airtip ESP32-C3 OLED board has LED on GPIO8
//! - Or external LED connected to GPIO8 (with current-limiting resistor)
//!
//! Run with:
//! cargo run --release --example blinky
//!
//! Logging:
//! This example uses the log crate with esp_println logger.
//! Available log levels (from least to most verbose):
//! - log::error!() - Critical errors
//! - log::warn!()  - Warnings
//! - log::info!()  - Informational messages (used here)
//! - log::debug!() - Debug information
//! - log::trace!() - Very verbose tracing
//!
//! The logger is initialized with LevelFilter::Info, so info!(), warn!(),
//! and error!() messages will be shown. Use LevelFilter::Debug or
//! LevelFilter::Trace for more verbose output.

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
    // Initialize logger with Info level (always shows log::info! messages)
    // This is better than init_logger_from_env() for examples as it doesn't
    // require setting RUST_LOG environment variable
    esp_println::logger::init_logger(log::LevelFilter::Info);

    log::info!("Starting LED Blinky Example");

    let peripherals = esp_hal::init(esp_hal::Config::default());

    // Configure GPIO8 as output for LED
    let mut led = Output::new(peripherals.GPIO8, Level::Low, OutputConfig::default());

    log::info!("LED initialized on GPIO8");
    log::info!("Starting blink pattern...");

    // Example of different log levels (debug won't show with Info level filter)
    log::debug!("This debug message won't appear with LevelFilter::Info");
    log::warn!("This is an example warning message");

    let delay = Delay::new();
    let mut blink_count = 0u32;

    // Blink forever
    loop {
        log::info!("LED ON (count: {})", blink_count);
        led.set_high();
        delay.delay_millis(500);

        log::info!("LED OFF");
        led.set_low();
        delay.delay_millis(500);

        blink_count += 1;

        // Example: Log a warning every 10 blinks
        if blink_count % 10 == 0 {
            log::warn!("Blinked {} times", blink_count);
        }
    }
}
