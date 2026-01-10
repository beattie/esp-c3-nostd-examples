# OLED Display Example

This example demonstrates how to use an SSD1306 OLED display with the ESP32-C3 using Rust's `no_std` embedded ecosystem.

## Hardware Requirements

- **ESP32-C3 board** (Airtip ESP32-C3 OLED board or similar)
- **SSD1306 OLED Display** (typically 0.42" with 72x40 visible pixels)
  - Connected via I2C
  - SDA: GPIO5
  - SCL: GPIO6

### Supported Boards

This example is designed for boards with integrated OLED displays such as:
- Airtip ESP32-C3 OLED
- 01Space ESP32-C3 0.42" OLED
- Other ESP32-C3 boards with SSD1306 displays on GPIO5/6

## Wiring (for external OLED)

If using an external OLED display with a standard ESP32-C3 board:

```
ESP32-C3          SSD1306 OLED
--------          ------------
GPIO5    ------>  SDA
GPIO6    ------>  SCL
3.3V     ------>  VCC
GND      ------>  GND
```

**Note:** Most I2C OLED modules have built-in pull-up resistors. If your display doesn't work, you may need to add 4.7kΩ pull-up resistors on SDA and SCL lines.

## What This Example Does

The example:

1. Initializes the I2C peripheral on GPIO5 (SDA) and GPIO6 (SCL)
2. Initializes the SSD1306 display driver
3. Draws graphics on the display:
   - A border rectangle around the edges
   - Text: "ESP32-C3" and "OLED Demo"
   - A small filled circle
4. Keeps the display on indefinitely

## Building and Running

### Build the Example

```bash
cargo build --release --example oled_display
```

### Flash to ESP32-C3

Using `espflash`:

```bash
espflash flash --monitor target/riscv32imc-unknown-none-elf/release/examples/oled_display
```

Or if you prefer separate steps:

```bash
# Flash
espflash flash target/riscv32imc-unknown-none-elf/release/examples/oled_display

# Monitor (in separate terminal after flashing)
espflash monitor
```

### Expected Output

**Serial Monitor:**
```
[INFO ] Starting OLED Display Example
[INFO ] I2C initialized on GPIO5 (SDA) and GPIO6 (SCL)
[INFO ] Display initialized
[INFO ] Display updated with graphics
[INFO ] OLED example complete - display should show border, text, and a circle
```

**OLED Display:**
- Border rectangle around the display edges
- "ESP32-C3" text near the top
- "OLED Demo" text in the middle
- Small filled circle in the lower right

## Code Overview

### Dependencies

The example uses these key crates:

- **`ssd1306`** (v0.9) - SSD1306 OLED driver with embedded-hal support
- **`embedded-graphics`** (v0.8) - Graphics library for drawing primitives and text
- **`esp-hal`** - ESP32-C3 hardware abstraction layer

### Key Code Sections

#### I2C Initialization

```rust
let i2c = I2c::new(peripherals.I2C0, Config::default())
    .unwrap()
    .with_sda(peripherals.GPIO5)
    .with_scl(peripherals.GPIO6);
```

#### Display Setup

```rust
let interface = I2CDisplayInterface::new(i2c);
let mut display = Ssd1306::new(interface, DisplaySize72x40, DisplayRotation::Rotate0)
    .into_buffered_graphics_mode();

display.init().unwrap();
```

#### Drawing Graphics

```rust
// Draw rectangle border
Rectangle::new(Point::new(0, 0), Size::new(72, 40))
    .into_styled(PrimitiveStyle::with_stroke(BinaryColor::On, 1))
    .draw(&mut display)
    .unwrap();

// Draw text
Text::new("ESP32-C3", Point::new(8, 12), text_style)
    .draw(&mut display)
    .unwrap();

// Flush to display
display.flush().unwrap();
```

## Customization

### Changing Display Size

For different SSD1306 display sizes, change the `DisplaySize`:

```rust
// For 128x64 displays
DisplaySize128x64

// For 128x32 displays
DisplaySize128x32

// For 96x16 displays
DisplaySize96x16
```

### Drawing Other Shapes

Using `embedded-graphics`, you can draw:

**Lines:**
```rust
use embedded_graphics::primitives::Line;

Line::new(Point::new(0, 0), Point::new(71, 39))
    .into_styled(PrimitiveStyle::with_stroke(BinaryColor::On, 1))
    .draw(&mut display)
    .unwrap();
```

**Triangles:**
```rust
use embedded_graphics::primitives::Triangle;

Triangle::new(
    Point::new(36, 5),
    Point::new(20, 25),
    Point::new(52, 25)
)
.into_styled(PrimitiveStyle::with_fill(BinaryColor::On))
.draw(&mut display)
.unwrap();
```

**Images:**
```rust
use embedded_graphics::image::{Image, ImageRaw};

// 1-bit bitmap data
let raw_image: ImageRaw<BinaryColor> = ImageRaw::new(&[
    0xFF, 0x00, 0xFF, 0x00,
], 16);

Image::new(&raw_image, Point::new(10, 10))
    .draw(&mut display)
    .unwrap();
```

### Using Different Fonts

```rust
use embedded_graphics::mono_font::ascii::{FONT_4X6, FONT_5X8, FONT_6X10, FONT_9X15};

// Smaller font
let small_style = MonoTextStyle::new(&FONT_4X6, BinaryColor::On);

// Larger font
let large_style = MonoTextStyle::new(&FONT_9X15, BinaryColor::On);
```

### Animations

For animations, update the display in a loop:

```rust
loop {
    display.clear_buffer();

    // Draw your animated content
    Circle::new(Point::new(x, y), 5)
        .into_styled(PrimitiveStyle::with_fill(BinaryColor::On))
        .draw(&mut display)
        .unwrap();

    display.flush().unwrap();

    // Update position
    x += 1;
    if x > 72 { x = 0; }

    esp_hal::delay::Delay::new().delay_millis(50);
}
```

## Troubleshooting

### Display Shows Nothing

1. **Check wiring** - Verify SDA is on GPIO5, SCL is on GPIO6, and power/ground are connected
2. **Check I2C address** - Default is 0x3C. Some displays use 0x3D. Check with an I2C scanner or your display's documentation
3. **Verify power** - Ensure the display is getting 3.3V (some displays are 5V tolerant but work better at 3.3V)
4. **Check pull-up resistors** - If the display doesn't respond, try adding 4.7kΩ pull-ups on SDA and SCL

### Wrong Display Size

If graphics appear cut off or in the wrong position:

1. Verify your display's actual resolution (use a multimeter or datasheet)
2. Try different `DisplaySize` settings in the code
3. For 0.42" displays, the buffer is typically 128x64 but only 72x40 is visible

### Display Shows Garbage

1. **I2C speed** - Try lowering the I2C frequency in Config:
   ```rust
   use esp_hal::time::RateExtU32;

   let config = Config {
       frequency: 100.kHz(),
       ..Config::default()
   };
   let i2c = I2c::new(peripherals.I2C0, config)
       .unwrap()
       .with_sda(peripherals.GPIO5)
       .with_scl(peripherals.GPIO6);
   ```

2. **Reset the display** - Power cycle the ESP32-C3

### Build Errors

If you get compile errors:

1. **Check esp-hal version** - This example requires esp-hal 1.0+ with the patches specified in Cargo.toml
2. **Update dependencies** - Run `cargo update`
3. **Clean build** - Run `cargo clean` then rebuild

## Advanced Usage

### Using with Embassy (Async)

For async operation with Embassy:

```rust
use esp_hal::i2c::master::I2c;

let i2c = I2c::new(peripherals.I2C0, Config::default())
    .unwrap()
    .with_sda(peripherals.GPIO5)
    .with_scl(peripherals.GPIO6)
    .into_async();

// Use with async display operations
```

### Multiple I2C Devices

To share the I2C bus with other devices, you can use `embedded-hal-bus`:

```rust
use embedded_hal_bus::i2c;

let i2c_bus = shared_bus::BusManagerSimple::new(i2c);
let display_i2c = i2c_bus.acquire_i2c();
let sensor_i2c = i2c_bus.acquire_i2c();
```

## Resources

- [embedded-graphics Documentation](https://docs.rs/embedded-graphics/)
- [SSD1306 Driver Documentation](https://docs.rs/ssd1306/)
- [esp-hal Documentation](https://docs.espressif.com/projects/rust/esp-hal/)
- [ESP32-C3 Technical Reference Manual](https://www.espressif.com/sites/default/files/documentation/esp32-c3_technical_reference_manual_en.pdf)

## License

This example is part of the esp-c3-nostd-examples project and is licensed under MIT OR Apache-2.0.
