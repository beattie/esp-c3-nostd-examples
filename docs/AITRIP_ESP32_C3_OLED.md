# Aitrip ESP32-C3 OLED Development Board

Comprehensive hardware documentation for the Aitrip ESP32-C3 OLED development board with integrated 0.42" OLED display.

![Board Type](https://img.shields.io/badge/Board-ESP32--C3-blue)
![Display](https://img.shields.io/badge/Display-0.42%22%20OLED-green)
![Connectivity](https://img.shields.io/badge/Connectivity-WiFi%20%2B%20BLE-orange)

## Table of Contents

- [Overview](#overview)
- [Technical Specifications](#technical-specifications)
- [Display Specifications](#display-specifications)
- [Pinout](#pinout)
- [GPIO Capabilities](#gpio-capabilities)
- [Power Supply](#power-supply)
- [Programming](#programming)
- [Display Programming](#display-programming)
- [Use Cases](#use-cases)
- [Important Notes](#important-notes)
- [Resources](#resources)

## Overview

The Aitrip ESP32-C3 OLED is an ultra-compact development board featuring:
- ESP32-C3 RISC-V microcontroller
- Integrated 0.42" OLED display (72x40 resolution)
- Built-in WiFi 802.11 b/g/n
- Bluetooth 5 (LE) connectivity
- Ceramic antenna
- 13 GPIO pins
- Micro-USB interface

**Board Dimensions:** Ultra-compact form factor ideal for wearable and space-constrained applications

**Manufacturer:** Aitrip / 01Space

## Technical Specifications

### Microcontroller
- **Chip:** ESP32-C3FN4 / ESP32-C3FH4
- **Architecture:** 32-bit RISC-V single-core processor
- **Clock Speed:** Up to 160 MHz
- **Flash Memory:** 4 MB (built-in)
- **SRAM:** 400 KB
- **ROM:** 384 KB

### Connectivity
- **WiFi:** IEEE 802.11 b/g/n (2.4 GHz)
- **Bluetooth:** Bluetooth 5 (LE)
- **Antenna:** Integrated ceramic antenna

### Interfaces
- **USB:** Micro-USB for programming and power
- **GPIO:** 13 general-purpose I/O pins
- **ADC:** 6 channels, 12-bit resolution (GPIO0-GPIO5)
- **Communication:**
  - UART (GPIO20, GPIO21)
  - I2C (hardware support)
  - SPI (hardware support)
  - I2S (hardware support)
- **PWM:** Multiple PWM channels available

### Buttons
- **BOOT Button:** GPIO9 (for entering bootloader mode)
- **RESET Button:** Hardware reset

## Display Specifications

### OLED Screen
- **Size:** 0.42 inch diagonal
- **Type:** OLED (Organic LED)
- **Resolution:** 72×40 pixels (effective display area)
- **Controller:** SSD1306-compatible chip
- **Interface:** I2C
- **I2C Address:** 0x3C (default)
- **Connection:**
  - SDA: GPIO5
  - SCL: GPIO6
- **Color:** Monochrome (typically white or blue)

### Display Characteristics
- **Library Resolution Setting:** 128×64 (required for U8g2 and similar libraries)
- **Display Offset:** (30, 12) for proper rendering
- **Active Drawing Area:** 72×40 pixels to avoid clipping
- **Starting Point:** 12864 (13, 14) - unique configuration for this screen

**Important:** This 0.42" OLED has a unique configuration and cannot directly replace other 0.42" screens without configuration adjustments.

## Pinout

### Pin Layout

```
        ┌─────────────┐
        │   USB Port  │
        └─────────────┘
              │
    ┌─────────────────────┐
    │                     │
3V3 ○                     │ Built-in
GND ○    [ESP32-C3]      │ 0.42" OLED
    ○                     │ Display
GPIO0 ○                   │ 72x40
GPIO1 ○   [    SoC    ]  │
GPIO2 ○                   │
GPIO3 ○                   │
GPIO4 ○                   │
GPIO5 ○ (SDA)             │
GPIO6 ○ (SCL)             │
GPIO7 ○                   │
GPIO8 ○ (LED)             │
GPIO9 ○ (BOOT)            │
GPIO10○                   │
GPIO20○ (UART RX)         │
GPIO21○ (UART TX)         │
    └─────────────────────┘
```

### Pin Descriptions

| Pin | Function | Notes |
|-----|----------|-------|
| 3V3 | Power Supply | 3.3V output/input |
| GND | Ground | Common ground |
| GPIO0 | GPIO / ADC1_CH0 | ADC capable |
| GPIO1 | GPIO / ADC1_CH1 | ADC capable |
| GPIO2 | GPIO / ADC1_CH2 | ADC capable |
| GPIO3 | GPIO / ADC1_CH3 | ADC capable |
| GPIO4 | GPIO / ADC1_CH4 | ADC capable |
| GPIO5 | GPIO / ADC1_CH5 / **I2C SDA** | **OLED Display SDA** |
| GPIO6 | GPIO / **I2C SCL** | **OLED Display SCL** |
| GPIO7 | GPIO | General purpose |
| GPIO8 | GPIO | **Built-in LED** |
| GPIO9 | GPIO | **BOOT Button** |
| GPIO10 | GPIO | General purpose |
| GPIO20 | GPIO / UART RX | Serial communication |
| GPIO21 | GPIO / UART TX | Serial communication |

### Reserved Pins

**Do not use for general GPIO:**
- **GPIO5** - I2C SDA (OLED display)
- **GPIO6** - I2C SCL (OLED display)
- **GPIO8** - Built-in LED
- **GPIO9** - Boot button
- **GPIO20** - UART RX (USB serial)
- **GPIO21** - UART TX (USB serial)

**Available for general use:** GPIO0, GPIO1, GPIO2, GPIO3, GPIO4, GPIO7, GPIO10

## GPIO Capabilities

### Analog-to-Digital Converter (ADC)
- **Channels:** 6 (GPIO0-GPIO5)
- **Resolution:** 12-bit (0-4095)
- **Voltage Range:** 0-3.3V
- **Note:** GPIO5 is shared with I2C SDA (OLED), use with caution

### Pulse Width Modulation (PWM)
- **Channels:** Multiple PWM channels available on most GPIO pins
- **Use Cases:** LED dimming, motor control, servo control

### Communication Interfaces

#### I2C
- **Default Configuration:**
  - SDA: GPIO5
  - SCL: GPIO6
- **Speed:** Up to 400 kHz (Fast mode)
- **Used by:** Built-in OLED display

#### SPI
- Can be configured on available GPIO pins
- Full duplex communication
- Multiple chip select support

#### UART
- **Primary UART:**
  - TX: GPIO21
  - RX: GPIO20
- Used for USB serial communication

## Power Supply

### Input Power
- **Voltage Range:** 3.3V - 6V
- **Recommended:** 5V via USB
- **Input Methods:**
  - Micro-USB port (5V)
  - 3V3 pin (3.3V regulated)
  - Battery (3.7V LiPo recommended with proper regulation)

### Power Consumption
- **Active Mode (WiFi on):** ~80-120 mA
- **Active Mode (WiFi off):** ~20-40 mA
- **Light Sleep:** ~3 mA
- **Deep Sleep:** ~5-10 µA
- **OLED Display:** ~10-20 mA (depending on content)

### Power Management
- Built-in 3.3V voltage regulator
- Supports deep sleep mode for battery-operated applications
- Wake-up sources: GPIO, Timer, ULP co-processor

## Programming

### Development Environments

#### Arduino IDE
1. Install ESP32 board support
2. Select board: "ESP32C3 Dev Module"
3. Install required libraries:
   - Adafruit GFX Library
   - Adafruit SSD1306
   - U8g2 (recommended for this display)

#### ESP-IDF
- Official Espressif IoT Development Framework
- Full control over ESP32-C3 features
- C/C++ based development

#### Rust (no_std)
- esp-hal for hardware abstraction
- embassy for async runtime
- See main project README for BLE examples

#### PlatformIO
- Cross-platform IDE
- Easy library management
- Supports multiple frameworks

### Uploading Code

1. **Connect Board:** Plug in via Micro-USB
2. **Select Port:** Choose the correct COM/tty port
3. **Enter Bootloader Mode (if needed):**
   - Hold BOOT button
   - Press RESET button
   - Release RESET button
   - Release BOOT button
4. **Upload:** Flash your program

### USB Serial Driver
- **macOS/Linux:** Usually works out of the box
- **Windows:** May require CH340 or CP2102 driver installation

## Display Programming

### I2C Configuration

```cpp
// Arduino example
#define SCREEN_WIDTH 128
#define SCREEN_HEIGHT 64
#define OLED_RESET -1
#define SCREEN_ADDRESS 0x3C

// Display offset for proper rendering
#define DISPLAY_OFFSET_X 30
#define DISPLAY_OFFSET_Y 12

// Effective drawing area
#define ACTIVE_WIDTH 72
#define ACTIVE_HEIGHT 40
```

### Arduino Example (Adafruit SSD1306)

```cpp
#include <Wire.h>
#include <Adafruit_GFX.h>
#include <Adafruit_SSD1306.h>

#define SCREEN_WIDTH 128
#define SCREEN_HEIGHT 64
#define OLED_RESET -1
#define SCREEN_ADDRESS 0x3C

Adafruit_SSD1306 display(SCREEN_WIDTH, SCREEN_HEIGHT, &Wire, OLED_RESET);

void setup() {
  Wire.begin(5, 6); // SDA, SCL

  if(!display.begin(SSD1306_SWITCHCAPVCC, SCREEN_ADDRESS)) {
    Serial.println(F("SSD1306 allocation failed"));
    for(;;);
  }

  display.clearDisplay();
  display.setTextSize(1);
  display.setTextColor(SSD1306_WHITE);
  display.setCursor(30, 12); // Offset for proper display
  display.println(F("Hello!"));
  display.display();
}
```

### U8g2 Example (Recommended)

```cpp
#include <U8g2lib.h>
#include <Wire.h>

// Constructor for this specific display
U8G2_SSD1306_72X40_ER_F_HW_I2C u8g2(U8G2_R0, U8X8_PIN_NONE);

void setup() {
  u8g2.begin();
  u8g2.clearBuffer();
  u8g2.setFont(u8g2_font_ncenB08_tr);
  u8g2.drawStr(0, 10, "ESP32-C3");
  u8g2.drawStr(0, 25, "OLED");
  u8g2.sendBuffer();
}
```

### Display Considerations

1. **Resolution:** Set library to 128x64 but only draw in 72x40 active area
2. **Offset:** Apply (30, 12) offset for proper alignment
3. **Font Size:** Use small fonts (6x8 or 8x8) for readability
4. **Refresh Rate:** Avoid excessive updates to reduce flicker
5. **Power:** Display consumes ~10-20mA when active

### Rust (no_std) Example

```rust
// Example with embedded-graphics and ssd1306 crate
use esp_hal::i2c::I2C;
use ssd1306::{prelude::*, I2CDisplayInterface, Ssd1306};
use embedded_graphics::prelude::*;

// I2C pins: SDA=GPIO5, SCL=GPIO6
let i2c = I2C::new(peripherals.I2C0, io.pins.gpio5, io.pins.gpio6, 400.kHz());

let interface = I2CDisplayInterface::new(i2c);
let mut display = Ssd1306::new(interface, DisplaySize72x40, DisplayRotation::Rotate0)
    .into_buffered_graphics_mode();

display.init().unwrap();
// Draw to display...
display.flush().unwrap();
```

## Use Cases

### Ideal Applications
- 🎨 **Wearable Devices:** Smart watches, fitness trackers
- 📊 **IoT Sensors:** Environmental monitoring with visual feedback
- 🏠 **Smart Home:** Status displays, mini control panels
- 🔧 **Development:** Debugging, status monitoring
- 🎮 **Mini Projects:** Tiny games, notification displays
- 📡 **Network Tools:** WiFi scanner, BLE beacon detector

### Project Ideas
1. **BLE Device Scanner:** Scan and display nearby Bluetooth devices
2. **WiFi Signal Monitor:** Show WiFi strength and connected networks
3. **Environmental Sensor Display:** Temperature, humidity readings
4. **Notification Display:** Display messages from phone/computer
5. **Crypto Price Ticker:** Show real-time cryptocurrency prices
6. **Weather Station:** Compact weather information display
7. **Smart Badge:** Wearable name tag with dynamic content

## Important Notes

### Display Configuration
⚠️ **Critical:** This 0.42" OLED has a unique configuration with starting point (13, 14) and requires specific offset settings. It **cannot** directly replace other 0.42" OLED displays without code modifications.

### Pin Usage
- **GPIO5 & GPIO6** are dedicated to the OLED display (I2C)
- **GPIO8** has a built-in LED that may interfere with other uses
- **GPIO9** is the BOOT button, avoid using as general input
- **GPIO20 & GPIO21** are used for USB serial communication

### ADC Limitations
- GPIO5 has ADC capability but is used by OLED display
- Only GPIO0-GPIO4 are freely available for analog input
- ADC2 is not available on ESP32-C3

### Power Considerations
- OLED display adds 10-20mA to power consumption
- Consider display sleep/off modes for battery applications
- Deep sleep is available but OLED will not update

### Memory
- With 4MB flash and 400KB SRAM, the board is suitable for moderate-complexity applications
- Be mindful of memory usage with graphics operations

## Resources

### Official Documentation
- [ESP32-C3 Datasheet](https://www.espressif.com/sites/default/files/documentation/esp32-c3_datasheet_en.pdf)
- [ESP32-C3 Technical Reference Manual](https://www.espressif.com/sites/default/files/documentation/esp32-c3_technical_reference_manual_en.pdf)

### Board-Specific Resources
- [ESP32-C3 OLED 0.42" Board Details](https://www.espboards.dev/esp32/esp32-c3-oled-042/)
- [Cirkit Designer Component Documentation](https://docs.cirkitdesigner.com/component/d356ca80-2207-43dd-95de-361b47687a81/esp32c3-042-oled)
- [Zephyr Project Board Documentation](https://docs.zephyrproject.org/latest/boards/01space/esp32c3_042_oled/doc/index.html)

### Development Resources
- [ESP-IDF Programming Guide](https://docs.espressif.com/projects/esp-idf/en/latest/esp32c3/)
- [Arduino ESP32 Core](https://github.com/espressif/arduino-esp32)
- [esp-hal (Rust)](https://docs.esp-rs.org/esp-hal/)

### Display Libraries
- [Adafruit SSD1306 Library](https://github.com/adafruit/Adafruit_SSD1306)
- [U8g2 Library](https://github.com/olikraus/u8g2)
- [embedded-graphics (Rust)](https://github.com/embedded-graphics/embedded-graphics)

### Community & Examples
- [ESP32-C3 OLED Arduino Examples](https://github.com/peff74/ESP32-C3_OLED)
- [Kevin's Blog - ESP32-C3 0.42 OLED](https://emalliab.wordpress.com/2025/02/12/esp32-c3-0-42-oled/)

### Where to Buy
- [Amazon - Aitrip ESP32-C3 OLED](https://www.amazon.com/AITRIP-ESP32-C3-Development-Bluetooth-0-42-Inch/dp/B0F32WS96L)
- AliExpress, eBay (search "ESP32-C3 0.42 OLED")

## Comparison with Other ESP32-C3 Boards

| Feature | ESP32-C3 OLED | ESP32-C3 SuperMini | ESP32-C3 DevKit |
|---------|---------------|-------------------|-----------------|
| Display | ✅ 0.42" OLED | ❌ None | ❌ None |
| Size | Ultra-compact | Ultra-compact | Standard |
| GPIO Pins | 13 | 13 | 22 |
| Built-in LED | ✅ GPIO8 | ✅ GPIO8 | ✅ GPIO8 |
| Form Factor | Integrated display | Minimal | Development-friendly |
| Use Case | Visual feedback projects | General embedded | Prototyping |
| Price | ~$8-12 | ~$3-5 | ~$5-8 |

## License

This documentation is provided for educational and development purposes.

---

**Document Version:** 1.0
**Last Updated:** 2026-01-08
**Maintained by:** Community Contributors

Sources:
- [ESP32-C3 OLED Board Details](https://www.espboards.dev/esp32/esp32-c3-oled-042/)
- [Cirkit Designer Documentation](https://docs.cirkitdesigner.com/component/d356ca80-2207-43dd-95de-361b47687a81/esp32c3-042-oled)
- [Zephyr Project Documentation](https://docs.zephyrproject.org/latest/boards/01space/esp32c3_042_oled/doc/index.html)
- [Kevin's Blog](https://emalliab.wordpress.com/2025/02/12/esp32-c3-0-42-oled/)
- [ESP32-C3 User Manuals](https://manuals.plus/ae/3256807670573244)
