//! BLE Clock Example for Airtip ESP32-C3 OLED Board
//! 
//! This example connects to a BLE devices and receives the current time.
//! The current time is displayed on the SSD1306 OLED display.
//! 
//! Hardware:
//! - Airtip ESP32-C3 OLED board (or similar with SSD1306 display)
//! - 0.42" OLED display (72x40 visible pixels) on I2C
//! - SDA: GPIO5
//! - SCL: GPIO6
//!
//! Run with:
//! cargo run --release --example ble_clock
#![no_std]
#![no_main]

use core::cell::RefCell;
use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use esp_backtrace as _;
use esp_hal::i2c::master::{Config, I2c};
use esp_hal::{clock::CpuClock, timer::timg::TimerGroup};
use esp_radio::ble::controller::BleConnector;
use heapless::Deque;
use ssd1306::{prelude::*, I2CDisplayInterface, Ssd1306};
use trouble_host::prelude::*;
use embedded_graphics::{
    mono_font::{ascii::FONT_6X10, MonoTextStyle},
    pixelcolor::BinaryColor,
    prelude::*,
    primitives::{PrimitiveStyle, Rectangle},
    text::Text,
};

esp_bootloader_esp_idf::esp_app_desc!();    

struct CurrentTime {
    year: u16,      // Bytes 0-1: Year (1582-9999)
    month: u8,      // Byte 2: Month (1-12)
    day: u8,        // Byte 3: Day (1-31)
    hours: u8,      // Byte 4: Hours (0-23)
    minutes: u8,    // Byte 5: Minutes (0-59)
    seconds: u8,    // Byte 6: Seconds (0-59)
    day_of_week: u8,// Byte 7: Day of Week (1=Monday to 7=Sunday)
    fractions256: u8,// Byte 8: Fractions of a second (0-255)
    adjust_reason: u8, // Byte 9: Adjust reason (0=none, 1=manual, 2=auto)
}

#[gatt_service(uuid = "12345678-1234-5678-1234-56789abcdef0")]
struct TimeService {
    #[characteristic(uuid = "12345678-1234-5678-1234-56789abcdef1", write, read, notify)]
    current_time: RefCell<CurrentTime>,
}

#[esp_rtos::main]
async fn main(_spawner: Spawner) {
    esp_println::logger::init_logger_from_env();
    esp_println::println!("Starting BLE Clock Example"); 
    let peripherals = esp_hal::init(esp_hal::Config::default().with_cpu_clock(CpuClock::max()));
    esp_alloc::heap_allocator!(size: 72 * 1024);
    let timg0 = TimerGroup::new(peripherals.TIMG0);
    let software_interrupt = esp_hal::interrupt::software::SoftwareInterruptControl::new(peripherals.SW_INTERRUPT);
    esp_rtos::start(timg0.timer0, software_interrupt.software_interrupt0);
    // Initialize I2C for OLED display
    // Airtip ESP32-C3 OLED: SDA=GPIO5, SCL=GPIO6
    let i2c = I2c::new(peripherals.I2C0, Config::default())
        .unwrap()
        .with_sda(peripherals.GPIO5)
        .with_scl(peripherals.GPIO6);
    esp_println::println!("I2C initialized on GPIO5 (SDA) and GPIO6 (SCL)");
    // Create display interface
    let interface = I2CDisplayInterface::new(i2c);
    // Create SSD1306 display driver
    // The Airtip board uses a 72x40 visible area, but SSD1306 buffer is 128x64
    let mut display = Ssd1306::new(interface, DisplaySize72x40, DisplayRotation::Rotate0)
        .into_buffered_graphics_mode();
    display.init().unwrap();
    display.clear_buffer();
    esp_println::println!("Display initialized");
    // Initialize BLE controller
    let bluetooth = peripherals.BT;
    let connector = match BleConnector::new(bluetooth, Default::default()) {
        Ok(c) => c,
        Err(e) => {
            esp_println::println!("ERROR: Failed to create BLE connector: {:?}", e);
            loop {}
        }
    };

    #[embassy_executor::task]
    async fn display_task<C>(i2c: I2c, mut display: Ssd1306<I2CDisplayInterface<I2c>, DisplaySize72x40>) {
        let text_style = MonoTextStyle::new(&FONT_6X10, BinaryColor::On);
        loop {
            // For demonstration, just display a static time
            display.clear_buffer();
            // Draw a border rectangle
            Rectangle::new(Point::new(0, 0), Size::new(72, 40))
                .into_styled(PrimitiveStyle::with_stroke(BinaryColor::On, 1))
                .draw(&mut display)
                .unwrap();
            // Display time
            Text::new("Time:", Point::new(8, 12), text_style)
                .draw(&mut display)
                .unwrap();
            Text::new("12:34:56", Point::new(6, 24), text_style)
                .draw(&mut display)
                .unwrap();
            display.flush().unwrap();
            Timer::after(Duration::from_secs(1)).await;
        }
    }   