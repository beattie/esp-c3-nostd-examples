# ESP32-C3 No-STD Examples

A collection of bare-metal (no_std) Rust examples for ESP32-C3. Currently features a working BLE scanner using the Trouble BLE host stack.

## Why No-STD?

This project uses the **no_std** (bare-metal) approach with `esp-hal` to support:
- **BLE functionality** via `trouble-host` stack
- **Future WS2812B LED control** (not yet implemented)

The no_std approach enables use of the pure Rust `trouble-host` BLE stack with esp-hal.

## Examples

### 1. BLE Scanner (`ble_scanner`) ✅ **WORKING**
Scans for BLE advertisements and prints discovered devices using the Trouble BLE stack.

**Hardware:**
- ESP32-C3 board (built-in BLE radio)

**Run:**
```bash
./scripts/build-example.sh ble_scanner
```

**Output:**
- Displays discovered BLE device addresses
- Automatically deduplicates repeated advertisements
- Scans continuously

### 2. WS2812B Rainbow (`ws2812b_rainbow`) ⚠️ **NOT YET IMPLEMENTED**
Placeholder for WS2812B LED control. The code structure is present but the RMT pulse generation is not implemented.

**Status:** Needs proper WS2812B RMT driver implementation to function.

## Project Structure

```
esp-c3-nostd-examples/
├── examples/
│   ├── ble_scanner.rs         # ✅ BLE device scanner (working)
│   └── ws2812b_rainbow.rs     # ⚠️ WS2812B placeholder (not implemented)
├── docs/
│   └── AITRIP_ESP32_C3_OLED.md # Hardware documentation for Aitrip board
├── scripts/
│   └── build-example.sh       # Build and flash script
├── .cargo/
│   └── config.toml            # Cargo configuration
├── build.rs                   # Linker script configuration
├── Cargo.toml                 # Dependencies and features
└── README.md
```

## Prerequisites

### Install Required Tools

```bash
# Install espup (ESP Rust toolchain installer)
cargo install espup

# Install ESP Rust toolchain
espup install

# Install espflash
cargo install espflash

# Install ldproxy
cargo install ldproxy
```

## Quick Start

### Build and Flash the BLE Scanner

```bash
cd ~/projects/esp-c3-nostd-examples

# BLE Scanner
./scripts/build-example.sh ble_scanner

# Specify port if needed
./scripts/build-example.sh ble_scanner --port /dev/ttyUSB0
```

### Manual Build

```bash
# Source ESP environment
. ~/export-esp.sh

# Build BLE example (requires ble feature)
cargo build --release --example ble_scanner --features ble

# Flash and monitor
espflash flash target/riscv32imc-unknown-none-elf/release/examples/ble_scanner --monitor
```

## Hardware Setup

### BLE Scanner
- No additional hardware needed
- ESP32-C3 has built-in BLE radio
- Works on any ESP32-C3 development board

### Supported Hardware
- **ESP32-C3 SuperMini** - Basic development board
- **Aitrip ESP32-C3 OLED** - Board with integrated 0.42" OLED display
  - See [detailed hardware documentation](docs/AITRIP_ESP32_C3_OLED.md)
  - Features 72x40 OLED on GPIO5/GPIO6 (I2C)
  - 13 GPIO pins available
- **Generic ESP32-C3 DevKit** - Any ESP32-C3 board

## Features

### Default
- Core ESP32-C3 support (no_std)
- Basic functionality without BLE

### BLE
- Enable with `--features ble`
- Adds Trouble BLE host stack
- Required for BLE scanner example

## Dependencies

### Core Dependencies (no_std)
- `esp-hal` 1.0.0 - Hardware Abstraction Layer (bare-metal, patched from lulf/esp-hal fork)
- `esp-backtrace` 0.18.1 - Panic handler with backtraces
- `esp-println` 0.16.0 - Logging support
- `esp-alloc` 0.9.0 - Heap allocator
- `embassy-executor` 0.9.1 - Async runtime
- `embassy-time` 0.5.0 - Async timers
- `heapless` 0.8 - Stack-allocated data structures

### BLE Dependencies (optional)
- `esp-radio` 0.17.0 - ESP32 BLE/WiFi radio driver (patched)
- `esp-rtos` 0.2.0 - RTOS wrapper with Embassy support (patched)
- `trouble-host` 0.5.1 - Pure Rust BLE host stack (local from ~/projects/trouble/host)
- `bt-hci` 0.7 - Bluetooth HCI definitions

### Important Notes
All esp-* dependencies are patched to use a specific git revision from `lulf/esp-hal` to ensure compatibility with trouble-host. See `[patch.crates-io]` section in Cargo.toml.

## Comparison: No-STD vs STD

| Feature | This Project (no_std) | esp-c3-examples (std) |
|---------|----------------------|----------------------|
| Approach | Bare-metal, esp-hal | ESP-IDF, esp-idf-hal |
| BLE Support | ✅ Yes (trouble-host) | ❌ Dependency conflicts |
| WS2812B Support | ⚠️ Not yet implemented | ✅ Yes (ws2812-esp32-rmt-driver) |
| Binary Size | Smaller (~300KB) | Larger |
| Features | Limited to bare-metal | Full ESP-IDF features |
| Complexity | More low-level control | Higher-level abstractions |

## Troubleshooting

### ESP Environment Not Found
```bash
espup install
. ~/export-esp.sh
```

### Build Fails
```bash
# Clean and rebuild
cargo clean
./scripts/build-example.sh ble_scanner
```

### Permission Denied on Serial Port
```bash
sudo usermod -a -G dialout $USER
# Log out and back in
```

### BLE Example Fails to Build
Make sure to enable the BLE feature:
```bash
cargo build --example ble_scanner --features ble --release
```

### No BLE Devices Discovered
- Ensure there are BLE devices nearby that are advertising
- Check that BLE is enabled on your phone/devices
- The scanner only shows unique devices once (deduplicated)
- Wait a few seconds for devices to appear

## Resources

### Hardware Documentation
- [Aitrip ESP32-C3 OLED Board Guide](docs/AITRIP_ESP32_C3_OLED.md) - Comprehensive hardware documentation

### Official Documentation
- [ESP32-C3 Datasheet](https://www.espressif.com/sites/default/files/documentation/esp32-c3_datasheet_en.pdf)
- [ESP32-C3 Technical Reference Manual](https://www.espressif.com/sites/default/files/documentation/esp32-c3_technical_reference_manual_en.pdf)

### Development Resources
- [esp-hal Documentation](https://docs.esp-rs.org/esp-hal/)
- [Rust on ESP Book](https://esp-rs.github.io/book/)
- [Trouble BLE Stack](https://github.com/embassy-rs/trouble)
- [Embassy Async Runtime](https://embassy.dev/)

## License

MIT OR Apache-2.0
