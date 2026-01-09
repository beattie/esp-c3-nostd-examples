# ESP32-C3-DevKitM-1 Development Board

Comprehensive hardware documentation for the official Espressif ESP32-C3-DevKitM-1 development board and compatible clones.

![Board Type](https://img.shields.io/badge/Board-ESP32--C3--DevKitM--1-blue)
![Module](https://img.shields.io/badge/Module-ESP32--C3--MINI--1-green)
![Connectivity](https://img.shields.io/badge/Connectivity-WiFi%20%2B%20BLE-orange)

## Table of Contents

- [Overview](#overview)
- [Technical Specifications](#technical-specifications)
- [Board Layout and Components](#board-layout-and-components)
- [Pinout](#pinout)
- [GPIO Capabilities](#gpio-capabilities)
- [Power Supply](#power-supply)
- [RGB LED](#rgb-led)
- [Buttons](#buttons)
- [Programming](#programming)
- [Power Consumption](#power-consumption)
- [Use Cases](#use-cases)
- [Strapping Pins](#strapping-pins)
- [Clone Boards](#clone-boards)
- [Resources](#resources)

## Overview

The ESP32-C3-DevKitM-1 is an **entry-level development board** from Espressif Systems based on the ESP32-C3-MINI-1 or ESP32-C3-MINI-1U module. The board is named for its compact size while providing complete WiFi and Bluetooth Low Energy functionality.

**Key Features:**
- ESP32-C3-MINI-1 module (PCB antenna) or ESP32-C3-MINI-1U (external antenna connector)
- 32-bit RISC-V single-core processor (up to 160 MHz)
- 4 MB integrated flash memory
- WiFi 802.11 b/g/n (2.4 GHz)
- Bluetooth 5 (LE)
- Addressable RGB LED (WS2812) on GPIO8
- Most I/O pins broken out to headers
- Micro-USB for programming and power
- Breadboard-compatible

**Manufacturer:** Espressif Systems (official), various clone manufacturers (FancyWhoop, etc.)

## Technical Specifications

### Microcontroller Module
- **Module:** ESP32-C3-MINI-1 (PCB antenna) or ESP32-C3-MINI-1U (U.FL connector)
- **Chip:** ESP32-C3FN4
- **Architecture:** 32-bit RISC-V single-core processor
- **Clock Speed:** Up to 160 MHz
- **FPU:** 32-bit single-precision floating point unit
- **Flash Memory:** 4 MB SPI flash (integrated in module)
- **SRAM:** 400 KB
- **ROM:** 384 KB
- **RTC SRAM:** 8 KB
- **Operating Voltage:** 3.0 - 3.6V
- **Operating Temperature:** -40°C to +85°C

### Wireless Connectivity
- **WiFi:**
  - IEEE 802.11 b/g/n
  - 2.4 GHz band
  - Supports Station mode, SoftAP mode, and Station+SoftAP mode
  - Maximum data rate: 150 Mbps
- **Bluetooth:**
  - Bluetooth 5 (LE)
  - Bluetooth mesh support
  - Long Range mode
  - Multiple connections supported

### Interfaces
- **USB:** Micro-USB port with USB-to-UART bridge (up to 3 Mbps)
- **GPIO:** 15 GPIO pins broken out (GPIO0-10, GPIO18-21)
- **ADC:** 6 channels, 12-bit resolution (GPIO0-5)
- **PWM:** Multiple PWM channels on GPIO pins
- **Communication:**
  - UART (GPIO20 RX, GPIO21 TX)
  - I2C (configurable on any GPIO)
  - SPI (GPIO2, 6, 7, 10 - used by flash, but can be used for other purposes with care)
  - I2S (configurable on any GPIO)

### Peripherals
- **RGB LED:** WS2812 addressable LED on GPIO8
- **Power LED:** Indicates USB power connection
- **BOOT Button:** GPIO9 (enters firmware download mode)
- **RESET Button:** CHIP_PU (restarts the system)

## Board Layout and Components

```
                    ┌─────────────────┐
                    │   Micro-USB     │
                    └─────────────────┘
                           │
         ┌─────────────────────────────────┐
         │          Power LED              │
         │                                 │
         │    ┌─────────────────────┐     │
         │    │  ESP32-C3-MINI-1    │     │
         │    │     Module with      │     │
    J1   │    │    PCB Antenna       │     │  J3
    ○────┼────┤                      ├─────┼────○
    ○    │    │   [ESP32-C3FN4]     │     │    ○
    ○    │    │                      │     │    ○
    ○    │    │    4MB Flash         │     │    ○
    ○    │    └─────────────────────┘     │    ○
    ○    │                                 │    ○
    ○    │    ┌──────┐      ┌──────┐      │    ○
    ○    │    │ BOOT │      │ RESET│      │    ○
    ○    │    └──────┘      └──────┘      │    ○
    ○    │                                 │    ○
    ○    │         RGB LED                 │    ○
    ○    │    ┌────┐  (GPIO8)              │    ○
    ○    │    │5V  │                       │    ○
         │    │to  │  USB-UART Bridge      │
         │    │3.3V│                       │
         └─────────────────────────────────┘
```

### Key Components

| Component | Description | Location |
|-----------|-------------|----------|
| **ESP32-C3-MINI-1** | Main module with WiFi/BLE and 4MB flash | Center of board |
| **5V to 3.3V LDO** | Power regulator | Below module |
| **Micro-USB Port** | Power and USB-UART communication | Top edge |
| **USB-UART Bridge** | Serial communication (up to 3 Mbps) | Integrated |
| **Power LED** | Red LED indicating power | Near USB port |
| **RGB LED** | WS2812 addressable LED | Connected to GPIO8 |
| **BOOT Button** | GPIO9, firmware download mode | Bottom left |
| **RESET Button** | CHIP_PU, system reset | Bottom right |
| **Pin Headers J1/J3** | 2.54mm pitch breakout headers | Both sides |

## Pinout

### J1 Header (Left Side)

```
Pin    Name      Type    Function
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
 1     GND       G       Ground
 2     GND       G       Ground
 3     3V3       P       3.3V power output
 4     3V3       P       3.3V power output
 5     IO2       I/O     GPIO2, ADC1_CH2, FSPIQ (strapping pin)
 6     IO3       I/O     GPIO3, ADC1_CH3
 7     GND       G       Ground
 8     RST       I       CHIP_PU (Reset)
 9     GND       G       Ground
10     IO0       I/O     GPIO0, ADC1_CH0, XTAL_32K_P
11     IO1       I/O     GPIO1, ADC1_CH1, XTAL_32K_N
12     IO10      I/O     GPIO10, FSPICS0
13     GND       G       Ground
14     5V        P       5V power input/output
15     5V        P       5V power input/output
16     GND       G       Ground
```

### J3 Header (Right Side)

```
Pin    Name      Type    Function
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
 1     GND       G       Ground
 2     TX        I/O     GPIO21, U0TXD (UART transmit)
 3     RX        I/O     GPIO20, U0RXD (UART receive)
 4     GND       G       Ground
 5     IO9       I/O     GPIO9, BOOT button (strapping pin)
 6     IO8       I/O     GPIO8, RGB LED (strapping pin)
 7     GND       G       Ground
 8     IO7       I/O     GPIO7, FSPID, MTDO
 9     IO6       I/O     GPIO6, FSPICLK, MTCK
10     IO5       I/O     GPIO5, ADC2_CH0, FSPIWP, MTDI
11     IO4       I/O     GPIO4, ADC1_CH4, FSPIHD, MTMS
12     GND       G       Ground
13     IO18      I/O     GPIO18, USB_D-
14     IO19      I/O     GPIO19, USB_D+
15     GND       G       Ground
16     GND       G       Ground
```

### Pin Type Legend
- **G** = Ground
- **P** = Power supply
- **I** = Input only
- **I/O** = Input/Output

### Available GPIO Pins

**Freely available for general use:**
- GPIO0, GPIO1, GPIO3, GPIO4, GPIO5, GPIO7, GPIO10, GPIO18, GPIO19

**Special function pins (use with caution):**
- GPIO2, GPIO6 - Used by SPI flash (FSPIQ, FSPICLK)
- GPIO8 - RGB LED (can be used but LED will respond)
- GPIO9 - BOOT button (strapping pin)
- GPIO20, GPIO21 - UART RX/TX (USB serial communication)

**Not broken out:**
- GPIO11-17 - Reserved for SPI flash communication

## GPIO Capabilities

### Analog-to-Digital Converter (ADC)

**ADC1 (6 channels, recommended for general use):**
- GPIO0 (ADC1_CH0)
- GPIO1 (ADC1_CH1)
- GPIO2 (ADC1_CH2)
- GPIO3 (ADC1_CH3)
- GPIO4 (ADC1_CH4)

**ADC2 (1 channel, limited availability):**
- GPIO5 (ADC2_CH0)

**Note:** ADC2 may not be available when WiFi is in use.

**Specifications:**
- Resolution: 12-bit (0-4095)
- Voltage range: 0-3.3V
- Attenuation options: 0dB, 2.5dB, 6dB, 11dB

### PWM (Pulse Width Modulation)

- Available on most GPIO pins
- 8 channels
- 16-bit resolution
- Frequency range: 1 Hz to 40 MHz
- Use cases: LED dimming, motor control, servo control

### Communication Peripherals

#### UART
- **UART0:** GPIO20 (RX), GPIO21 (TX) - used by USB-UART bridge
- **UART1:** Can be configured on any available GPIO
- Baud rates up to 5 Mbps

#### I2C
- Can be configured on any available GPIO pins
- Standard mode (100 kHz) and Fast mode (400 kHz)
- Master and slave modes supported

#### SPI
- **Hardware SPI (Flash):** GPIO2, GPIO6, GPIO7, GPIO10
- **User SPI:** Can be configured on available GPIO pins
- Up to 80 MHz

#### I2S
- Configurable on any available GPIO pins
- Used for audio applications

## Power Supply

### Power Input Options

Three mutually exclusive power supply methods:

1. **Micro-USB Port** (Recommended)
   - Voltage: 5V
   - Current: Up to 500 mA from standard USB port
   - Automatically regulated to 3.3V by onboard LDO

2. **5V Pin Headers**
   - J1 pins 14-15
   - Voltage: 5V
   - Regulated to 3.3V by onboard LDO

3. **3V3 Pin Headers**
   - J1 pins 3-4
   - Voltage: 3.3V (directly to module, bypass regulator)
   - Use only pre-regulated 3.3V supply

### Power Output
- **3V3 Pins:** Can supply 3.3V to external devices (limited by LDO capacity, typically ~500mA max)
- **5V Pins:** Can supply 5V if USB powered

### Power Requirements
- **Module operating voltage:** 3.0 - 3.6V
- **Recommended:** 3.3V ±5%

## Power Consumption

### Typical Current Draw

| Mode | Current Consumption | Notes |
|------|-------------------|-------|
| **Active (no WiFi)** | ~20 mA | Normal operation |
| **WiFi TX (peak)** | 180-240 mA | Depends on TX power and modulation |
| **WiFi RX** | ~90-100 mA | Receiving data |
| **Modem Sleep** | ~20-25 mA | WiFi connected but not transmitting |
| **Light Sleep** | ~500 µA | CPU paused, peripherals active |
| **Deep Sleep** | ~5-10 µA | Minimal power, RTC running |
| **Hibernation** | <5 µA | Lowest power mode |

### Power Optimization Tips
1. Use deep sleep when not actively transmitting/receiving
2. Reduce WiFi TX power if possible
3. Disable unused peripherals
4. Use modem sleep during idle periods
5. Turn off RGB LED when not needed (GPIO8 = LOW)

## RGB LED

### Specifications
- **Type:** WS2812 (NeoPixel-compatible)
- **GPIO:** GPIO8
- **Voltage:** 3.3V
- **Protocol:** Single-wire timing-sensitive protocol
- **Colors:** Full RGB (24-bit color)

### Control Methods

#### Arduino (FastLED)
```cpp
#include <FastLED.h>

#define LED_PIN     8
#define NUM_LEDS    1

CRGB leds[NUM_LEDS];

void setup() {
  FastLED.addLeds<WS2812, LED_PIN, GRB>(leds, NUM_LEDS);
}

void loop() {
  leds[0] = CRGB::Red;
  FastLED.show();
  delay(500);
  leds[0] = CRGB::Blue;
  FastLED.show();
  delay(500);
}
```

#### ESP-IDF (RMT Driver)
```c
#include "driver/rmt.h"

#define RMT_TX_CHANNEL RMT_CHANNEL_0
#define RMT_TX_GPIO_NUM 8

// Configure RMT for WS2812 timing
// See examples in ESP-IDF for complete implementation
```

#### Rust (esp-hal RMT)
```rust
use esp_hal::rmt::Rmt;
use smart_leds::RGB8;

let rmt = Rmt::new(peripherals.RMT, 80.MHz()).unwrap();
let mut led = Ws2812Rmt::new(rmt.channel0, peripherals.GPIO8);

led.write([RGB8::new(255, 0, 0)].iter().cloned()).ok(); // Red
```

### Important Notes
- GPIO8 is a strapping pin; RGB LED state during boot affects chip behavior
- RGB LED consumes ~20mA when fully bright white
- WS2812 requires precise timing (use RMT peripheral)

## Buttons

### BOOT Button
- **GPIO:** GPIO9
- **Function:**
  - Enters firmware download mode when held during reset
  - Can be used as general input in user code
- **Strapping Pin:** Yes (affects boot mode)
- **Pull-up:** Internal pull-up resistor
- **Active:** LOW when pressed

### RESET Button
- **Pin:** CHIP_PU (RST on J1 header)
- **Function:** Hardware reset, restarts the system
- **Active:** LOW when pressed

### Programming Mode Entry
1. Hold BOOT button
2. Press and release RESET button
3. Release BOOT button
4. Board is now in firmware download mode

## Programming

### Development Environments

#### ESP-IDF (Official)
- Official Espressif IoT Development Framework
- C/C++ based
- Full hardware access
- Complete documentation

```bash
# Install ESP-IDF
git clone --recursive https://github.com/espressif/esp-idf.git
cd esp-idf
./install.sh esp32c3
. ./export.sh

# Create and build project
idf.py create-project hello_world
cd hello_world
idf.py set-target esp32c3
idf.py build
idf.py flash monitor
```

#### Arduino IDE
1. Install ESP32 board support in Arduino IDE
2. Select **Tools > Board > ESP32C3 Dev Module**
3. Select correct COM port
4. Upload sketch

**Board Settings:**
- Board: "ESP32C3 Dev Module"
- Upload Speed: 921600
- USB CDC On Boot: "Enabled" (for Serial)
- Flash Size: "4MB"
- Partition Scheme: Default

#### Rust (no_std)
- esp-hal for hardware abstraction
- embassy for async runtime
- See main project README for examples

```bash
# Set target
rustup target add riscv32imc-unknown-none-elf

# Build and flash
cargo build --release --target riscv32imc-unknown-none-elf
espflash flash target/riscv32imc-unknown-none-elf/release/your-binary
```

#### PlatformIO
```ini
[env:esp32-c3-devkitm-1]
platform = espressif32
board = esp32-c3-devkitm-1
framework = arduino
```

### USB Drivers
- **macOS/Linux:** Usually automatic (CDC/ACM driver)
- **Windows:** May require Silicon Labs CP210x driver
- Check Device Manager for COM port number

## Strapping Pins

**Strapping pins** are special GPIO pins that control chip functions during boot based on their voltage levels at power-up or system reset.

### ESP32-C3 Strapping Pins

| GPIO | Default State | Function if LOW | Function if HIGH |
|------|--------------|-----------------|------------------|
| **GPIO2** | Pull-down | Boot mode control | Boot mode control |
| **GPIO8** | Pull-down | Boot mode control | Boot mode control |
| **GPIO9** | Pull-up | Download boot mode | SPI boot mode |

### Strapping Pin Combinations

**Normal SPI Boot (Flash):**
- GPIO9 = HIGH (pulled up internally, default)

**Download Boot (UART/USB programming):**
- GPIO9 = LOW (BOOT button pressed during reset)

### Important Notes
1. **GPIO9** has internal pull-up, so default is normal boot
2. **BOOT button** pulls GPIO9 LOW when pressed
3. **GPIO2 and GPIO8** typically don't need external control
4. RGB LED on GPIO8 will affect boot mode if externally driven during reset
5. After boot, strapping pins can be used as normal GPIOs

## Use Cases

### Ideal Applications
- 🌐 **IoT Devices:** WiFi-connected sensors, smart home devices
- 🎓 **Learning Platform:** ESP32-C3 and RISC-V development
- 🔧 **Prototyping:** Quick development and testing
- 📡 **Wireless Projects:** BLE beacons, WiFi access points
- 🤖 **Robotics:** WiFi-controlled robots, drones
- 💡 **Smart Lighting:** RGB LED projects, WS2812 controllers
- 📊 **Data Logging:** Environmental monitoring, data collection

### Project Ideas
1. **WiFi Weather Station:** Display temperature, humidity, and weather data
2. **BLE Beacon Scanner:** Detect and log nearby Bluetooth devices
3. **Smart RGB Lamp:** WiFi-controlled mood lighting
4. **IoT Sensor Node:** Temperature, motion, or light sensor with cloud connectivity
5. **Web Server:** Host a simple web interface for device control
6. **MQTT Client:** Publish sensor data to MQTT broker
7. **Home Automation Controller:** Control lights, switches, sensors
8. **Wireless Serial Bridge:** WiFi-to-Serial converter

## Clone Boards

### Common Manufacturers
Many Chinese manufacturers produce ESP32-C3-DevKitM-1 clones:
- **FancyWhoop** (Amazon seller)
- **Aitrip**
- Various AliExpress sellers
- Generic "ESP32-C3 Dev Board" listings

### Identifying Clones
**Look for:**
- ESP32-C3-MINI-1 module (not ESP32-WROOM or other modules)
- Identical pin layout to official board
- RGB LED on GPIO8
- BOOT and RESET buttons
- Micro-USB port

**Common Differences:**
- May use different USB-UART chip (CH340 instead of CP2102)
- RGB LED may be different brand (still WS2812 compatible)
- Board color may vary (official is black)
- Silkscreen text and logos different
- May lack Espressif logo

### Compatibility
- **100% compatible** with official board pinout
- **Works with same software** and programming tools
- **Same GPIO capabilities** and peripherals
- **Only difference** is typically USB-UART chip and physical appearance

## Resources

### Official Espressif Documentation
- [ESP32-C3-DevKitM-1 User Guide](https://docs.espressif.com/projects/esp-dev-kits/en/latest/esp32c3/esp32-c3-devkitm-1/user_guide.html)
- [ESP32-C3 Datasheet](https://www.espressif.com/sites/default/files/documentation/esp32-c3_datasheet_en.pdf)
- [ESP32-C3 Technical Reference Manual](https://www.espressif.com/sites/default/files/documentation/esp32-c3_technical_reference_manual_en.pdf)
- [ESP-IDF Programming Guide (ESP32-C3)](https://docs.espressif.com/projects/esp-idf/en/stable/esp32c3/)

### Board-Specific Resources
- [ESPBoards.dev - DevKitM-1 Details](https://www.espboards.dev/esp32/esp32-c3-devkitm-1/)
- [Cirkit Designer - DevKitM-1 Component](https://docs.cirkitdesigner.com/component/4c8ef31c-622b-4add-88e3-da13bb1d1324/esp32-c3-devkitm-1)
- [Arduino ESP32 Board Definition](https://docs.espressif.com/projects/arduino-esp32/en/latest/boards/ESP32-C3-DevKitM-1.html)

### Development Resources
- [ESP-IDF GitHub Repository](https://github.com/espressif/esp-idf)
- [Arduino ESP32 Core](https://github.com/espressif/arduino-esp32)
- [esp-hal (Rust)](https://docs.esp-rs.org/esp-hal/)
- [Rust on ESP Book](https://esp-rs.github.io/book/)

### Community Examples
- [RGB LED on ESP32-C3-DevKitM-1 (Rust)](https://dev.to/jajera/getting-rgb-led-working-on-esp32-c3-devkitm-1-rust-1-57h1)
- [WS2812B LED Strip Control](https://github.com/cashoefman/ESP32-C3-Rainbow-LED-Strip)
- [Controlling LED with ESP-IDF](https://www.electronics-lab.com/deep-dive-on-controlling-led-with-esp32-c3-devkitm-1-development-board-using-esp-idf/)

### Where to Buy
- **Official:** [Espressif](https://www.espressif.com/), [DigiKey](https://www.digikey.com/en/products/detail/espressif-systems/ESP32-C3-DEVKITM-1/13684315), [Mouser](https://www.mouser.com/)
- **Clones:** Amazon, AliExpress, eBay (search "ESP32-C3 DevKitM-1")

### Power Management Resources
- [ESP32-C3 Sleep Modes Documentation](https://docs.espressif.com/projects/esp-idf/en/stable/esp32c3/api-reference/system/sleep_modes.html)
- [Current Consumption Measurement Guide](https://docs.espressif.com/projects/esp-idf/en/stable/esp32c3/api-guides/current-consumption-measurement-modules.html)
- [ESP32 Sleep Modes Tutorial](https://lastminuteengineers.com/esp32-sleep-modes-power-consumption/)

## Comparison with Other ESP32-C3 Boards

| Feature | ESP32-C3-DevKitM-1 | ESP32-C3 SuperMini | ESP32-C3 OLED (0.42") |
|---------|-------------------|-------------------|----------------------|
| Module | ESP32-C3-MINI-1 | ESP32-C3FH4 | ESP32-C3FN4 |
| Flash | 4 MB | 4 MB | 4 MB |
| Size | Standard dev board | Ultra-compact | Compact with display |
| RGB LED | ✅ WS2812 (GPIO8) | ✅ (GPIO8) | ❌ None |
| Display | ❌ None | ❌ None | ✅ 0.42" OLED |
| GPIO Pins | 15 broken out | 13 broken out | 13 broken out |
| Buttons | BOOT, RESET | BOOT, RESET | BOOT, RESET |
| Antenna | PCB or U.FL | PCB | Ceramic |
| Breadboard | ✅ Yes | ✅ Yes | ⚠️ Tight fit |
| Price | ~$5-8 | ~$3-5 | ~$8-12 |
| Best For | General development | Minimal projects | Projects needing display |

---

**Document Version:** 1.0
**Last Updated:** 2026-01-08
**Maintained by:** Community Contributors

Sources:
- [ESP32-C3-DevKitM-1 Official User Guide](https://docs.espressif.com/projects/esp-dev-kits/en/latest/esp32c3/esp32-c3-devkitm-1/user_guide.html)
- [ESP-IDF DevKitM-1 Documentation](https://docs.espressif.com/projects/esp-idf/en/stable/esp32c3/hw-reference/esp32c3/user-guide-devkitm-1.html)
- [ESPBoards.dev DevKitM-1 Specifications](https://www.espboards.dev/esp32/esp32-c3-devkitm-1/)
- [Cirkit Designer DevKitM-1 Component](https://docs.cirkitdesigner.com/component/4c8ef31c-622b-4add-88e3-da13bb1d1324/esp32-c3-devkitm-1)
- [ESP32-C3 Power Consumption Guide](https://docs.espressif.com/projects/esp-idf/en/stable/esp32c3/api-guides/current-consumption-measurement-modules.html)
- [Arduino ESP32 Board Documentation](https://docs.espressif.com/projects/arduino-esp32/en/latest/boards/ESP32-C3-DevKitM-1.html)
