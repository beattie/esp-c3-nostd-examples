//! ESP32-C3 BME280 driver — no_std async Embassy
//!
//! Hardware:
//!   SDA = GPIO5, SCL = GPIO6
//!   BME280 at 0x76 (SDO=GND)
//!
//! Run with:
//!   cargo run --release

#![no_std]
#![no_main]

mod bme280;
use bme280::Bme280;

use embassy_time::{Duration, Timer};
use esp_backtrace as _;
use esp_hal::i2c::master::{Config as I2cConfig, I2c};
use esp_rtos::main;

esp_bootloader_esp_idf::esp_app_desc!();

#[main]
async fn main(_spawner: embassy_executor::Spawner) {
    esp_println::logger::init_logger(log::LevelFilter::Info);
    log::info!("BME280 driver starting");

    let peripherals = esp_hal::init(esp_hal::Config::default());

    // Set up I2C — same pins as your C project
    let i2c = I2c::new(peripherals.I2C0, I2cConfig::default())
        .unwrap()
        .with_sda(peripherals.GPIO5)
        .with_scl(peripherals.GPIO6)
        .into_async();

    let mut sensor = Bme280::new(i2c);

    match sensor.init().await {
        Ok(_) => log::info!("BME280 initialized"),
        Err(e) => {
            log::error!("BME280 init failed: {:?}", e);
            loop {}
        }
    }

    loop {
        match sensor.read().await {
            Ok(r) => {
                log::info!(
                    "Temp: {}.{:02} C  Pressure: {} Pa  Humidity: {}.{:02} %",
                    r.temperature_cdeg / 100,
                    r.temperature_cdeg.abs() % 100,
                    r.pressure_pa,
                    r.humidity_pct / 100,
                    r.humidity_pct % 100,
                );
            }
            Err(_) => log::error!("Read failed"),
        }

        Timer::after(Duration::from_secs(2)).await;
    }
}
