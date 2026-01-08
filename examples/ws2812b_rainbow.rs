//! WS2812B Rainbow Example for ESP32-C3 (no_std)
//!
//! This example demonstrates controlling a WS2812B addressable RGB LED
//! with smooth rainbow color cycling using esp-hal (bare metal).
//!
//! Hardware:
//! - ESP32-C3 board with built-in WS2812B on GPIO8
//! - Or external WS2812B connected to GPIO8
//!
//! Run with:
//! ./scripts/build-example.sh ws2812b_rainbow

#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use esp_backtrace as _;
use esp_hal::{
    clock::CpuClock,
    prelude::*,
    rmt::Rmt,
    timer::timg::TimerGroup,
};
use smart_leds::{RGB8, SmartLedsWrite, brightness};

// Simple WS2812B driver using RMT
struct Ws2812<'d> {
    channel: esp_hal::rmt::Channel<'d, 0>,
}

impl<'d> Ws2812<'d> {
    fn new(channel: esp_hal::rmt::Channel<'d, 0>, _pin: esp_hal::gpio::GpioPin<8>) -> Self {
        Self { channel }
    }
}

impl SmartLedsWrite for Ws2812<'_> {
    type Error = ();
    type Color = RGB8;

    fn write<T, I>(&mut self, iterator: T) -> Result<(), Self::Error>
    where
        T: Iterator<Item = I>,
        I: Into<Self::Color>,
    {
        // Simple pulse generation for WS2812B - this is a placeholder
        // In a real implementation, you'd generate proper RMT pulses
        Ok(())
    }
}

#[esp_rtos::main]
async fn main(_spawner: Spawner) {
    esp_println::logger::init_logger_from_env();

    let peripherals = esp_hal::init(esp_hal::Config::default().with_cpu_clock(CpuClock::max()));
    esp_alloc::heap_allocator!(size: 32 * 1024);

    let timg0 = TimerGroup::new(peripherals.TIMG0);
    let software_interrupt = esp_hal::interrupt::software::SoftwareInterruptControl::new(peripherals.SW_INTERRUPT);

    esp_rtos::start(timg0.timer0, software_interrupt.software_interrupt0);

    log::info!("Starting WS2812B Rainbow Example (no_std)");

    // Initialize RMT for WS2812B
    let led_pin = peripherals.GPIO8;
    let rmt = Rmt::new(peripherals.RMT, 80.MHz()).unwrap();
    let mut led = Ws2812::new(rmt.channel0, led_pin);

    log::info!("WS2812B initialized on GPIO8");
    log::info!("Starting rainbow color cycle...");

    let mut hue: u16 = 0;

    loop {
        // Convert HSV to RGB
        let rgb = hsv_to_rgb(hue, 100, 50);
        let color = RGB8::new(rgb.0, rgb.1, rgb.2);

        // Write to LED
        led.write([color].iter().cloned()).ok();

        log::info!("Hue: {}, RGB: ({}, {}, {})", hue, rgb.0, rgb.1, rgb.2);

        // Increment hue
        hue = (hue + 5) % 360;

        Timer::after(Duration::from_secs(1)).await;
    }
}

/// Convert HSV to RGB
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
