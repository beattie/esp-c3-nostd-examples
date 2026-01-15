# Time and Clocks in Embassy, Trouble, and ESP-HAL

## The Question: Real-Time Clock (Wall-Clock Time)

You're asking about a "real-time clock" like Linux's `time()` function, which returns:
- **Unix timestamp**: Seconds since 1970-01-01 00:00:00 UTC
- **Wall-clock time**: Actual calendar date/time (year, month, day, hour, minute, second)
- **Persistent**: Survives reboots (with battery-backed RTC hardware)

## The Answer: There Isn't One (Built-In)

**Neither `embassy`, `trouble-host`, nor `esp-hal` provide a real-time clock (wall-clock time).**

All three crates provide **monotonic clocks** (time since boot), not wall-clock time.

---

## 1. Embassy-Time: Monotonic Clock Since Boot

### What It Provides

`embassy-time` provides a **monotonic clock** that counts ticks since the MCU started:

```rust
use embassy_time::{Instant, Duration, Timer};

// Get current time since boot
let now = Instant::now();

// Time is in ticks, convertible to seconds/millis/micros
let secs = now.as_secs();      // u64: seconds since boot
let millis = now.as_millis();  // u64: milliseconds since boot
let micros = now.as_micros();  // u64: microseconds since boot

// Wait for a duration
Timer::after_secs(5).await;
Timer::after_millis(100).await;
```

### Key Characteristics

- ✅ **Monotonic**: Always increases, never goes backwards
- ✅ **Precise**: Based on hardware timer (typically 1MHz or 1KHz)
- ✅ **Async-friendly**: Works with Embassy's async runtime
- ❌ **NOT wall-clock**: Starts at 0 when MCU boots
- ❌ **NOT persistent**: Resets to 0 on every boot
- ❌ **NO calendar**: No concept of year, month, day, etc.

### From the Source Code

```rust
/// An Instant in time, based on the MCU's clock ticks since startup.
pub struct Instant {
    ticks: u64,
}
```

**Comment says it all**: "ticks since startup", not Unix time.

---

## 2. ESP-HAL RTC: Monotonic Clock Since Boot

### What It Provides

ESP-HAL's RTC (Real-Time Control) module provides:
- Low-power timekeeping (survives deep sleep)
- Microsecond precision timing
- Watchdog timer functionality

```rust
use esp_hal::rtc_cntl::Rtc;

let rtc = Rtc::new(peripherals.LPWR);

// Get time in microseconds since boot
let time_us = rtc.current_time_us();

// Set the time (can set to arbitrary value)
rtc.set_current_time_us(1000000); // Set to 1 second
```

### Key Characteristics

- ✅ **Survives deep sleep**: Keeps counting during low-power modes
- ✅ **Microsecond precision**: 1 µs resolution
- ✅ **Can be set**: You can manually set the counter value
- ❌ **NOT wall-clock**: Still counts from boot (or arbitrary start)
- ❌ **NOT persistent**: Resets on power cycle (unless battery-backed)
- ❌ **NO calendar**: No concept of dates

### Purpose

The ESP32's "RTC" is misnamed from a Linux perspective:
- **Linux RTC**: Real-Time Clock with calendar (date/time)
- **ESP32 RTC**: Real-Time Control/Counter (low-power timer)

It's used for:
- Deep sleep timekeeping
- Low-power wake timers
- Watchdog functionality
- NOT for wall-clock time

---

## 3. Trouble-Host: Uses Embassy Time

The `trouble-host` BLE stack uses `embassy-time` for:
- Connection timeouts
- Pairing timeouts
- Scan durations
- Advertising intervals

```rust
// From trouble-host source
use embassy_time::{Duration, Instant, TimeoutError};

// Example: 30 second timeout for pairing
let timeout = Duration::from_secs(30);
let result = operation.with_timeout(timeout).await?;
```

**No wall-clock time functionality** - it's all relative timing.

---

## Why No Built-In Wall-Clock Time?

### 1. Embedded Systems Have No Time Source

Unlike Linux systems, embedded devices typically lack:
- ❌ Internet connection (for NTP)
- ❌ Battery-backed RTC hardware (on many boards)
- ❌ User input for setting time
- ❌ Persistent storage for time

### 2. Time Must Be Set From External Source

To get wall-clock time on ESP32-C3, you need to:
1. **Receive time from external source** (BLE, WiFi, GPS, etc.)
2. **Store Unix timestamp** in memory
3. **Add elapsed time** from monotonic clock

### 3. Most Embedded Apps Don't Need It

Many embedded applications only need:
- Relative timing (delays, timeouts)
- Event ordering (before/after)
- Durations (how long since X)

Wall-clock time is optional.

---

## How to Implement Wall-Clock Time (If Needed)

If you need actual calendar time, you must implement it yourself:

### Approach 1: Receive Time via BLE

This is what your `ble_clock.rs` example attempts to do:

```rust
// Receive time from BLE client
struct CurrentTime {
    year: u16,
    month: u8,
    day: u8,
    hours: u8,
    minutes: u8,
    seconds: u8,
    day_of_week: u8,
    fractions256: u8,
    adjust_reason: u8,
}

// Store Unix timestamp + boot time offset
static UNIX_EPOCH_AT_BOOT: AtomicU64 = AtomicU64::new(0);

fn set_time(time: CurrentTime) {
    // Convert calendar time to Unix timestamp
    let unix_timestamp = calendar_to_unix(&time);

    // Store when boot happened in Unix time
    let now_ticks = Instant::now().as_secs();
    let unix_at_boot = unix_timestamp - now_ticks;

    UNIX_EPOCH_AT_BOOT.store(unix_at_boot, Ordering::SeqCst);
}

fn get_unix_time() -> u64 {
    let unix_at_boot = UNIX_EPOCH_AT_BOOT.load(Ordering::SeqCst);
    let now_ticks = Instant::now().as_secs();
    unix_at_boot + now_ticks
}
```

### Approach 2: Receive Time via WiFi (NTP)

If you have WiFi connectivity:

```rust
use esp_wifi::wifi::*;
// Use NTP protocol to get time from internet time servers
// Set the Unix timestamp as above
```

### Approach 3: Hardware RTC Module

Use an external RTC chip with battery backup:
- DS3231 (I2C RTC)
- DS1307 (I2C RTC)
- PCF8523 (I2C RTC)

These chips:
- ✅ Store actual calendar time (year, month, day, etc.)
- ✅ Have battery backup (keep time during power loss)
- ✅ Are precise (crystal oscillator)
- ❌ Require external hardware
- ❌ Require I2C communication

---

## Comparison Table

| Feature | Linux `time()` | `embassy_time::Instant` | ESP-HAL RTC | BLE Time Transfer |
|---------|---------------|------------------------|-------------|-------------------|
| **Type** | Wall-clock | Monotonic | Monotonic | Wall-clock |
| **Unit** | Seconds since 1970 | Ticks since boot | Microseconds since boot | Calendar format |
| **Survives reboot** | ✅ Yes | ❌ No | ❌ No (unless battery) | ❌ No (must resync) |
| **Calendar aware** | ✅ Yes | ❌ No | ❌ No | ✅ Yes |
| **Precision** | 1 second | ~1µs-1ms | 1µs | Depends on protocol |
| **Set by** | System/NTP | Hardware | Hardware | External device |
| **Accuracy** | Very high (NTP) | Hardware dependent | Hardware dependent | Depends on source |

---

## Example: Implementing Basic Wall-Clock Time

Here's a minimal implementation for your BLE clock:

```rust
use core::sync::atomic::{AtomicU64, Ordering};
use embassy_time::Instant;

// Store Unix timestamp when we received time
static UNIX_EPOCH_AT_BOOT: AtomicU64 = AtomicU64::new(0);
static TIME_SET: AtomicBool = AtomicBool::new(false);

/// Set the wall-clock time from BLE
pub fn set_time_from_ble(time_bytes: &[u8; 10]) {
    // Parse BLE time format
    let year = u16::from_le_bytes([time_bytes[0], time_bytes[1]]);
    let month = time_bytes[2];
    let day = time_bytes[3];
    let hours = time_bytes[4];
    let minutes = time_bytes[5];
    let seconds = time_bytes[6];

    // Convert to Unix timestamp (you need a date/time library for this)
    // Or receive Unix timestamp directly from client
    let unix_timestamp = calculate_unix_timestamp(year, month, day, hours, minutes, seconds);

    // Calculate when boot happened in Unix time
    let boot_time_secs = Instant::now().as_secs();
    let unix_at_boot = unix_timestamp - boot_time_secs;

    UNIX_EPOCH_AT_BOOT.store(unix_at_boot, Ordering::SeqCst);
    TIME_SET.store(true, Ordering::SeqCst);
}

/// Get current Unix timestamp (if time has been set)
pub fn get_unix_time() -> Option<u64> {
    if !TIME_SET.load(Ordering::SeqCst) {
        return None;
    }

    let unix_at_boot = UNIX_EPOCH_AT_BOOT.load(Ordering::SeqCst);
    let now_secs = Instant::now().as_secs();
    Some(unix_at_boot + now_secs)
}

/// Convert Unix timestamp to human-readable time
pub fn format_time(unix_time: u64) -> (u16, u8, u8, u8, u8, u8) {
    // Convert Unix timestamp to calendar format
    // (Year, Month, Day, Hour, Minute, Second)
    // This requires date/time calculation code
    todo!("Implement Unix to calendar conversion")
}

// Helper function (simplified - real implementation is complex)
fn calculate_unix_timestamp(year: u16, month: u8, day: u8, hours: u8, minutes: u8, seconds: u8) -> u64 {
    // This is complex! You need to:
    // 1. Calculate days since 1970-01-01
    // 2. Account for leap years
    // 3. Add time of day
    // Consider using a crate like `chrono-tz` or `time` (if available for no_std)
    todo!("Implement calendar to Unix timestamp conversion")
}
```

---

## Recommended Crates for Time Conversion

If you need calendar time operations in `no_std`:

### 1. `time` crate (no_std compatible)
```toml
[dependencies]
time = { version = "0.3", default-features = false }
```

Provides:
- Calendar date/time types
- Unix timestamp conversion
- Formatting
- No allocator required

### 2. `chrono` (no_std with some features)
```toml
[dependencies]
chrono = { version = "0.4", default-features = false, features = ["alloc"] }
```

More powerful but requires allocator.

---

## Summary

### For Your BLE Clock Application

To implement a working clock display:

1. **Receive time via BLE** from a client (phone/computer)
2. **Store the Unix timestamp** and the boot time offset
3. **Calculate current time** = stored timestamp + elapsed seconds since boot
4. **Convert to calendar format** for display (hour:minute:second)
5. **Update display** every second using `embassy_time::Timer`

### The Key Point

- **`embassy_time::Instant`**: Time since boot (monotonic)
- **`esp_hal::rtc`**: Time since boot, survives deep sleep (monotonic)
- **Wall-clock time**: Must be implemented by receiving time from external source

Neither Embassy nor ESP-HAL provides a Linux-style `time()` function that returns Unix timestamps. You must build this yourself using monotonic clocks + external time source.

### Example Architecture

```
External Time Source (BLE/WiFi/GPS)
           ↓
    Set Unix Timestamp
           ↓
Store: unix_at_boot = received_time - Instant::now()
           ↓
Get Current Time: unix_at_boot + Instant::now()
           ↓
    Convert to Calendar Format
           ↓
    Display on OLED
```

This is what you need to implement for a functioning clock!
