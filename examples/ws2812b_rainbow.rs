//! WS2812B Rainbow Example for ESP32-C3 (no_std)
//!
//! This example demonstrates controlling a WS2812B addressable RGB LED
//! with smooth rainbow color cycling using esp-hal RMT peripheral directly.
//!
//! Hardware:
//! - ESP32-C3-DevKitM-1 board with built-in WS2812B on GPIO8
//! - Or external WS2812B connected to GPIO8
//!
//! Run with:
//! ./scripts/build-example.sh ws2812b_rainbow
//!
//! Note: This example uses a custom WS2812B driver implementation using
//! the RMT peripheral directly, demonstrating low-level hardware control.

#![no_std]
#![no_main]

use esp_backtrace as _;
use esp_hal::gpio::Level;
use esp_hal::rmt::{PulseCode, Rmt, TxChannelConfig, TxChannelCreator};
use esp_hal::time::Rate;
use smart_leds::RGB8;

esp_bootloader_esp_idf::esp_app_desc!();

#[esp_hal::main]
fn main() -> ! {
    use Level::{High, Low};

    esp_println::logger::init_logger_from_env();
    log::info!("Starting WS2812B Rainbow Example");

    let peripherals = esp_hal::init(esp_hal::Config::default());

    // Initialize RMT peripheral at 80MHz
    let rmt = Rmt::new(peripherals.RMT, Rate::from_mhz(80)).unwrap();

    // Configure RMT channel 0 for WS2812B on GPIO8
    // Use divider=2 for 40MHz effective clock (80MHz / 2 = 40MHz, 25ns per tick)
    let config = TxChannelConfig::default()
        .with_clk_divider(2)
        .with_idle_output(true)
        .with_idle_output_level(Level::Low);

    let mut channel = rmt
        .channel0
        .configure_tx(&config)
        .unwrap()
        .with_pin(peripherals.GPIO8);

    log::info!("WS2812B initialized on GPIO8 using RMT (40MHz, divider=2)");
    log::info!("Starting rainbow color cycle...");

    // WS2812B timing constants (at 40MHz RMT clock, 25ns per tick)
    // T0: 400ns high (16 ticks), 850ns low (34 ticks)
    // T1: 800ns high (32 ticks), 450ns low (18 ticks)
    const T0H: PulseCode = PulseCode::new(High, 16, Low, 34);
    const T1H: PulseCode = PulseCode::new(High, 32, Low, 18);

    let mut hue: u16 = 0;

    loop {
        // Convert HSV to RGB (30% brightness for comfortable viewing)
        let rgb = hsv_to_rgb(hue, 100, 30);
        let color = RGB8::new(rgb.0, rgb.1, rgb.2);

        // Build WS2812B pulse sequence
        let mut pulses = heapless::Vec::<PulseCode, 32>::new();

        // WS2812B expects GRB order
        let bytes = [color.g, color.r, color.b];

        for byte in bytes {
            for bit in (0..8).rev() {
                let pulse = if (byte >> bit) & 1 == 1 { T1H } else { T0H };
                pulses.push(pulse).ok();
            }
        }

        // Add reset pulse (>50µs low) - 2000 ticks at 40MHz = 50µs
        pulses.push(PulseCode::new(Low, 2000, Low, 0)).ok();

        // Transmit pulses
        channel = channel.transmit(&pulses).unwrap().wait().unwrap();

        // Log once per full rainbow cycle
        if hue == 0 {
            log::info!("Rainbow cycle completed, restarting...");
        }

        // Increment hue for smooth rainbow cycling
        hue = (hue + 5) % 360;

        // Simple delay (blocking)
        esp_hal::delay::Delay::new().delay_millis(50);
    }
}

/// Convert HSV to RGB
///
/// # Arguments
/// * `h` - Hue (0-359)
/// * `s` - Saturation (0-100)
/// * `v` - Value/Brightness (0-100)
///
/// # Returns
/// RGB tuple (0-255, 0-255, 0-255)
fn hsv_to_rgb(h: u16, s: u8, v: u8) -> (u8, u8, u8) {
    let h = h % 360;
    let s = s.min(100) as f32 / 100.0;
    let v = v.min(100) as f32 / 100.0;

    let c = v * s;
    let x = c * (1.0 - ((h as f32 / 60.0) % 2.0 - 1.0).abs());
    let m = v - c;

    let (r, g, b) = match h {
        0..=59 => (c, x, 0.0),
        60..=119 => (x, c, 0.0),
        120..=179 => (0.0, c, x),
        180..=239 => (0.0, x, c),
        240..=299 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };

    (
        ((r + m) * 255.0) as u8,
        ((g + m) * 255.0) as u8,
        ((b + m) * 255.0) as u8,
    )
}
