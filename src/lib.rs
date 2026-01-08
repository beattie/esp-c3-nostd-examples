//! ESP32-C3 No-STD Examples Library
//!
//! This crate provides common utilities for no_std ESP32-C3 examples.

#![no_std]

pub mod utils {
    /// HSV to RGB color conversion
    pub fn hsv_to_rgb(h: u16, s: u8, v: u8) -> (u8, u8, u8) {
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
}
