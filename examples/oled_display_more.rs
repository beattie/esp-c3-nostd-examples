//! OLED Display Example for Airtip ESP32-C3 OLED Board
//!
//! This example demonstrates drawing text and graphics on the SSD1306 OLED display.
//!
//! Hardware:
//! - Airtip ESP32-C3 OLED board (or similar with SSD1306 display)
//! - 0.42" OLED display (72x40 visible pixels) on I2C
//! - SDA: GPIO5
//! - SCL: GPIO6
//!
//! Run with:
//! cargo run --release --example oled_display

#![no_std]
#![no_main]

use embedded_graphics::{
    mono_font::{ascii::FONT_6X10, MonoTextStyle},
    pixelcolor::BinaryColor,
    prelude::*,
    primitives::{Circle, PrimitiveStyle, Rectangle, Triangle},
    text::Text,
};
use esp_backtrace as _;
use esp_hal::i2c::master::{Config, I2c};
use ssd1306::{prelude::*, I2CDisplayInterface, Ssd1306};

esp_bootloader_esp_idf::esp_app_desc!();

#[esp_hal::main]
fn main() -> ! {
    esp_println::logger::init_logger_from_env();
    log::info!("Starting OLED Display Example");

    let peripherals = esp_hal::init(esp_hal::Config::default());

    // Initialize I2C for OLED display
    // Airtip ESP32-C3 OLED: SDA=GPIO5, SCL=GPIO6
    let i2c = I2c::new(peripherals.I2C0, Config::default())
        .unwrap()
        .with_sda(peripherals.GPIO5)
        .with_scl(peripherals.GPIO6);

    log::info!("I2C initialized on GPIO5 (SDA) and GPIO6 (SCL)");

    // Create display interface
    let interface = I2CDisplayInterface::new(i2c);

    // Create SSD1306 display driver
    // The Airtip board uses a 72x40 visible area, but SSD1306 buffer is 128x64
    let mut display = Ssd1306::new(interface, DisplaySize72x40, DisplayRotation::Rotate0)
        .into_buffered_graphics_mode();

    display.init().unwrap();
    display.clear_buffer();

    log::info!("Display initialized");

    // Draw a border rectangle
    Rectangle::new(Point::new(0, 0), Size::new(72, 40))
        .into_styled(PrimitiveStyle::with_stroke(BinaryColor::On, 1))
        .draw(&mut display)
        .unwrap();

    // Draw text
    let text_style = MonoTextStyle::new(&FONT_6X10, BinaryColor::On);
    Text::new("ESP32-C3", Point::new(8, 12), text_style)
        .draw(&mut display)
        .unwrap();

    Text::new("OLED Demo", Point::new(6, 24), text_style)
        .draw(&mut display)
        .unwrap();

    // Draw a small filled circle
    Circle::new(Point::new(58, 28), 8)
        .into_styled(PrimitiveStyle::with_fill(BinaryColor::On))
        .draw(&mut display)
        .unwrap();

    // Draw a small filled square
    Rectangle::new(Point::new(5, 28), Size::new(8, 8))
        .into_styled(PrimitiveStyle::with_fill(BinaryColor::On))
        .draw(&mut display)
        .unwrap();

    // Draw a small un-filled square
    Rectangle::new(Point::new(20, 28), Size::new(8, 8))
        .into_styled(PrimitiveStyle::with_stroke(BinaryColor::On, 1))
        .draw(&mut display)
        .unwrap();
    
    // Draw a small filled triangle
    Triangle::new(Point::new(30, 35), Point::new(40, 36), Point::new(35, 28))
        .into_styled(PrimitiveStyle::with_fill(BinaryColor::On))
        .draw(&mut display)
        .unwrap();


    // Flush the buffer to the display
    display.flush().unwrap();

    log::info!("Display updated with graphics");
    log::info!("OLED example complete - display should show border, text, and a circle");

    // Keep the display on
    loop {
        esp_hal::delay::Delay::new().delay_millis(1000);
    }
}
