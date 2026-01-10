# ESP32-C3 No-STD Examples

A collection of bare-metal (no_std) Rust examples for ESP32-C3, featuring LED blinking, BLE scanning, WS2812B LED control, OLED display graphics, and Embassy async examples.

## Why No-STD?

This project uses the **no_std** (bare-metal) approach with `esp-hal` to support:
- **BLE functionality** via `trouble-host` stack
- **WS2812B LED control** via RMT peripheral
- **OLED display graphics** via SSD1306 driver with embedded-graphics
- **Embassy async runtime** for concurrent tasks

The no_std approach enables:
- Direct hardware control with minimal overhead
- Pure Rust implementations without C dependencies
- Smaller binary sizes (~300KB)
- Lower power consumption

## Examples

### 1. LED Blinky (`blinky`) ✅ **WORKING**
Classic "Hello World" for embedded systems - blinks an LED on GPIO8.

**Hardware:**
- ESP32-C3 board with LED on GPIO8
- Airtip ESP32-C3 OLED board (has LED on GPIO8)
- Or external LED with current-limiting resistor on GPIO8

**Run:**
```bash
cargo build --release --example blinky
espflash flash --monitor target/riscv32imc-unknown-none-elf/release/examples/blinky
```

**Features:**
- Simple GPIO output control
- 500ms blink interval
- No external dependencies needed
- Perfect starting point for beginners

### 2. BLE Scanner (`ble_scanner`) ✅ **WORKING**
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

### 3. WS2812B Rainbow (`ws2812b_rainbow`) ✅ **WORKING**
Displays a smooth rainbow color cycle on a WS2812B addressable RGB LED using the ESP32-C3 RMT peripheral.

**Hardware:**
- ESP32-C3-DevKitM-1 board (WS2812B on GPIO8)
- Or external WS2812B LED connected to GPIO8

**Run:**
```bash
./scripts/build-example.sh ws2812b_rainbow
```

**Features:**
- Direct RMT peripheral control for precise WS2812B timing
- Smooth rainbow color cycling effect using HSV to RGB conversion
- Hardware-based pulse generation (no CPU bit-banging needed)
- Comfortable brightness setting (30%)
- Uses 40MHz RMT clock (80MHz base / 2 divider) for accurate timing

### 4. OLED Display (`oled_display`) ✅ **WORKING**
Demonstrates drawing text and graphics on an SSD1306 OLED display using embedded-graphics.

**Hardware:**
- Airtip ESP32-C3 OLED board or similar
- 0.42" OLED display (72x40 pixels) on I2C (GPIO5/GPIO6)
- Or external SSD1306 OLED connected to GPIO5 (SDA) and GPIO6 (SCL)

**Run:**
```bash
cargo build --release --example oled_display
espflash flash --monitor target/riscv32imc-unknown-none-elf/release/examples/oled_display
```

**Features:**
- I2C communication with SSD1306 OLED controller
- Graphics primitives (rectangles, circles, text)
- Uses embedded-graphics for drawing
- Buffered graphics mode for smooth updates
- Works with various SSD1306 display sizes

**See:** [Detailed OLED documentation](docs/oled_display.md)

### 5. Embassy Hello World (`embassy_hello_world`) ✅ **WORKING**
Simple example demonstrating Embassy async/await with concurrent tasks.

**Hardware:**
- Any ESP32-C3 board

**Run:**
```bash
cargo build --release --example embassy_hello_world --features embassy
espflash flash --monitor target/riscv32imc-unknown-none-elf/release/examples/embassy_hello_world
```

**Features:**
- Embassy executor with multiple concurrent tasks
- Async timers with embassy-time
- Demonstrates task spawning and scheduling
- Uses esp-rtos for Embassy integration

## Project Structure

```
esp-c3-nostd-examples/
├── examples/
│   ├── blinky.rs              # ✅ Simple LED blinky (working)
│   ├── ble_scanner.rs         # ✅ BLE device scanner (working)
│   ├── ws2812b_rainbow.rs     # ✅ WS2812B rainbow LED (working)
│   ├── oled_display.rs        # ✅ OLED graphics display (working)
│   └── embassy_hello_world.rs # ✅ Embassy async example (working)
├── docs/
│   ├── ESP32_C3_DevKitM_1.md  # ESP32-C3-DevKitM-1 hardware documentation
│   ├── AITRIP_ESP32_C3_OLED.md # Aitrip ESP32-C3 OLED hardware documentation
│   └── oled_display.md        # OLED display example guide
├── scripts/
│   ├── build-example.sh       # Build and flash script
│   ├── export-esp.sh          # ESP toolchain environment setup
│   └── identify-target.sh     # Debug probe target identification
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

### Build and Flash Examples

```bash
cd ~/projects/esp-c3-nostd-examples

# LED Blinky (simplest example - great starting point!)
cargo build --release --example blinky
espflash flash --monitor target/riscv32imc-unknown-none-elf/release/examples/blinky

# BLE Scanner
./scripts/build-example.sh ble_scanner

# WS2812B Rainbow LED
./scripts/build-example.sh ws2812b_rainbow

# OLED Display
cargo build --release --example oled_display
espflash flash --monitor target/riscv32imc-unknown-none-elf/release/examples/oled_display

# Embassy Hello World
cargo build --release --example embassy_hello_world --features embassy
espflash flash --monitor target/riscv32imc-unknown-none-elf/release/examples/embassy_hello_world

# Specify port if needed
./scripts/build-example.sh ble_scanner --port /dev/ttyUSB0
```

### Manual Build

```bash
# Source ESP environment
. scripts/export-esp.sh

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
- **ESP32-C3-DevKitM-1** - Official Espressif development board
  - See [detailed hardware documentation](docs/ESP32_C3_DevKitM_1.md)
  - 15 GPIO pins broken out
  - WS2812 RGB LED on GPIO8
  - Breadboard-compatible design
- **ESP32-C3 SuperMini** - Ultra-compact development board
  - 13 GPIO pins
  - Minimal form factor
- **Aitrip ESP32-C3 OLED** - Board with integrated 0.42" OLED display
  - See [detailed hardware documentation](docs/AITRIP_ESP32_C3_OLED.md)
  - Features 72x40 OLED on GPIO5/GPIO6 (I2C)
  - 13 GPIO pins available
- **Generic ESP32-C3 boards** - Most ESP32-C3 boards should work

## Features

### Default
- Core ESP32-C3 support (no_std)
- Basic functionality without optional features
- OLED display and WS2812B examples work without features

### BLE
- Enable with `--features ble`
- Adds Trouble BLE host stack
- Required for BLE scanner example

### Embassy
- Enable with `--features embassy`
- Adds esp-rtos for Embassy async runtime support
- Required for embassy_hello_world example

## Dependencies

### Core Dependencies (no_std)
- `esp-hal` 1.0.0 - Hardware Abstraction Layer (bare-metal, patched from lulf/esp-hal fork)
- `esp-backtrace` 0.18.1 - Panic handler with backtraces
- `esp-println` 0.16.0 - Logging support
- `esp-alloc` 0.9.0 - Heap allocator
- `embassy-executor` 0.9.1 - Async runtime
- `embassy-time` 0.5.0 - Async timers
- `heapless` 0.8 - Stack-allocated data structures

### WS2812B Dependencies
- `smart-leds` 0.4 - LED color traits and utilities
- Uses esp-hal's RMT peripheral directly for precise timing control

### OLED Display Dependencies
- `ssd1306` 0.9 - SSD1306 OLED driver with embedded-hal support
- `embedded-graphics` 0.8 - Graphics library for drawing primitives and text

### BLE Dependencies (optional)
- `esp-radio` 0.17.0 - ESP32 BLE/WiFi radio driver (patched)
- `esp-rtos` 0.2.0 - RTOS wrapper with Embassy support (patched)
- `trouble-host` 0.5.1 - Pure Rust BLE host stack (local from ~/projects/trouble/host)
- `bt-hci` 0.7 - Bluetooth HCI definitions

### Important Notes
- All esp-* dependencies are patched to use a specific git revision from `lulf/esp-hal` to ensure compatibility with trouble-host. See `[patch.crates-io]` section in Cargo.toml.
- The WS2812B example uses esp-hal's RMT peripheral directly instead of a separate driver library, providing precise hardware control.

## Comparison: No-STD vs STD

| Feature | This Project (no_std) | (std) |
|---------|----------------------|----------------------|
| Approach | Bare-metal, esp-hal | ESP-IDF, esp-idf-hal |
| BLE Support | ✅ Yes (trouble-host) | ❌ Dependency conflicts |
| WS2812B Support | ✅ Yes (RMT peripheral) | ✅ Yes (ws2812-esp32-rmt-driver) |
| OLED Display | ✅ Yes (SSD1306 driver) | ✅ Yes (SSD1306 driver) |
| Embassy Async | ✅ Yes (esp-rtos) | ✅ Yes (esp-idf-svc) |
| Binary Size | Smaller (~300KB) | Larger |
| Features | Limited to bare-metal | Full ESP-IDF features |
| Complexity | More low-level control | Higher-level abstractions |
| Boot Time | Faster | Slower |

## Troubleshooting

### ESP Environment Not Found
```bash
espup install
. scripts/export-esp.sh
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

### OLED Display Not Working
- Check wiring: SDA on GPIO5, SCL on GPIO6, power and ground connected
- Verify I2C address (usually 0x3C for SSD1306)
- Ensure pull-up resistors are present (many modules have them built-in)
- See [detailed troubleshooting guide](docs/oled_display.md#troubleshooting)

### Embassy Example Fails to Build
Make sure to enable the embassy feature:
```bash
cargo build --example embassy_hello_world --features embassy --release
```

## Resources

### Example Documentation
- [OLED Display Example Guide](docs/oled_display.md) - Complete guide for SSD1306 OLED displays

### Hardware Documentation
- [ESP32-C3-DevKitM-1 Board Guide](docs/ESP32_C3_DevKitM_1.md) - Official Espressif board documentation
- [Aitrip ESP32-C3 OLED Board Guide](docs/AITRIP_ESP32_C3_OLED.md) - Board with integrated OLED display

### Official Documentation
- [ESP32-C3 Datasheet](https://www.espressif.com/sites/default/files/documentation/esp32-c3_datasheet_en.pdf)
- [ESP32-C3 Technical Reference Manual](https://www.espressif.com/sites/default/files/documentation/esp32-c3_technical_reference_manual_en.pdf)
- [Espressif ESP32-C3-DevKitM-1 User Guide](https://docs.espressif.com/projects/esp-dev-kits/en/latest/esp32c3/esp32-c3-devkitm-1/user_guide.html)

### Development Resources
- [esp-hal Documentation](https://docs.esp-rs.org/esp-hal/)
- [Rust on ESP Book](https://esp-rs.github.io/book/)
- [Trouble BLE Stack](https://github.com/embassy-rs/trouble)
- [Embassy Async Runtime](https://embassy.dev/)
- [embedded-graphics](https://docs.rs/embedded-graphics/) - Graphics library
- [SSD1306 Driver](https://docs.rs/ssd1306/) - OLED display driver

## License

MIT OR Apache-2.0
