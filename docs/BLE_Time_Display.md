# BLE Time Display Implementation Guide

This guide explains how to implement a BLE peripheral that receives time from a connected device (phone/computer) and displays it on the OLED.

## Terminology Clarification

**Your nomenclature was close, but here's the precise BLE terminology:**

- **Peripheral** (Server): Your ESP32-C3 - advertises, accepts connections, provides GATT services
- **Central** (Client): The phone/computer - scans, initiates connections, reads/writes to services
- ~~"Host" typically refers to the BLE stack layer, not the connecting device~~

## Architecture Overview

```
┌─────────────────┐                    ┌──────────────────┐
│  Phone/Computer │                    │    ESP32-C3      │
│   (Central)     │  ◄─── BLE ────►   │  (Peripheral)    │
│                 │                    │                  │
│  - Scans        │                    │  - Advertises    │
│  - Connects     │                    │  - Accepts conn  │
│  - Writes time  │                    │  - Receives time │
└─────────────────┘                    │  - Shows on OLED │
                                       └──────────────────┘
```

## Required Libraries and Modules

### Core BLE Stack

From `trouble_host::prelude::*` (already in your dependencies):

**Key Types:**
- `Controller` - BLE controller interface
- `Peripheral` - Peripheral role API
- `GattConnection` - Connection handle with GATT server
- `GattEvent` - Read/Write/Subscribe events
- `Address` - BLE device address

**GATT Macros (from `trouble_host_macros`):**
- `#[gatt_server]` - Define GATT server structure
- `#[gatt_service(uuid = ...)]` - Define a GATT service
- `#[characteristic(uuid = ..., read, write, notify)]` - Define characteristics

**UUID Constants (from `bt_hci::uuid`):**
- `service::CURRENT_TIME` - Current Time Service UUID (0x1805)
- `characteristic::CURRENT_TIME` - Current Time Characteristic UUID (0x2A2B)
- `service::BATTERY` - Battery Service UUID (0x180F) - for reference
- `characteristic::BATTERY_LEVEL` - Battery Level UUID (0x2A19) - for reference

### Documentation References

- **trouble-host**: [docs.embassy.dev/trouble-host](https://docs.embassy.dev/trouble-host/git/default/index.html)
- **bt-hci crate**: [docs.rs/bt-hci](https://docs.rs/bt-hci/latest/bt_hci/)
- **BLE Current Time Service Spec**: [bluetooth.com CTS-1.1](https://www.bluetooth.com/specifications/specs/cts-1-1/)
- **Current Time Service Documentation**: [bluetooth.com CTS HTML spec](https://www.bluetooth.com/wp-content/uploads/Files/Specification/HTML/CTS_v1.0/out/en/index-en.html)

## Current Time Characteristic Data Format

According to the [Bluetooth CTS specification](https://www.bluetooth.com/wp-content/uploads/Files/Specification/HTML/CTS_v1.0/out/en/index-en.html), the Current Time characteristic (UUID 0x2A2B) is 10 bytes:

```rust
// Byte layout for Current Time characteristic (0x2A2B)
struct CurrentTime {
    year: u16,           // Bytes 0-1: Year (1582-9999)
    month: u8,           // Byte 2: Month (1-12)
    day: u8,             // Byte 3: Day (1-31)
    hours: u8,           // Byte 4: Hours (0-23)
    minutes: u8,         // Byte 5: Minutes (0-59)
    seconds: u8,         // Byte 6: Seconds (0-59)
    day_of_week: u8,     // Byte 7: Day of week (1=Monday, 7=Sunday, 0=Unknown)
    fractions256: u8,    // Byte 8: Fractions of a second (1/256th)
    adjust_reason: u8,   // Byte 9: Reason for adjustment (flags)
}
```

**Total: 10 bytes, little-endian for multi-byte values**

## Implementation Steps

### 1. Define Your GATT Server

Create a GATT server with a custom time service:

```rust
use trouble_host::prelude::*;

#[gatt_server]
struct TimeServer {
    time_service: TimeService,
}

#[gatt_service(uuid = "12345678-1234-5678-1234-56789abcdef0")] // Custom UUID
struct TimeService {
    // Use a byte array to receive the 10-byte time data
    #[characteristic(uuid = "12345678-1234-5678-1234-56789abcdef1", write, read, notify)]
    current_time: [u8; 10],
}
```

**Note:** We're using a custom service UUID instead of the official Current Time Service (0x1805) because:
- The official CTS has complex requirements (multiple characteristics, specific behaviors)
- For a simple app, a custom service is easier and more flexible
- You can define exactly the data format you want

**Alternative:** If you want standard CTS compliance:
```rust
#[gatt_service(uuid = service::CURRENT_TIME)] // 0x1805
struct TimeService {
    #[characteristic(uuid = characteristic::CURRENT_TIME, write, read, notify)] // 0x2A2B
    current_time: [u8; 10],
}
```

### 2. Set Up the BLE Peripheral

Based on your existing `ble_scanner.rs`, modify to run as peripheral:

```rust
use esp_hal::{clock::CpuClock, timer::timg::TimerGroup};
use esp_radio::ble::controller::BleConnector;
use trouble_host::prelude::*;

const CONNECTIONS_MAX: usize = 1;
const L2CAP_CHANNELS_MAX: usize = 2; // Signal + ATT

#[esp_rtos::main]
async fn main(_spawner: Spawner) {
    // ... initialization same as ble_scanner ...

    let bluetooth = peripherals.BT;
    let connector = BleConnector::new(bluetooth, Default::default()).unwrap();
    let controller: ExternalController<_, 20> = ExternalController::new(connector);

    run_time_server(controller).await;
}
```

### 3. Implement the Peripheral Logic

```rust
async fn run_time_server<C>(controller: C)
where
    C: Controller,
{
    let address: Address = Address::random([0xff, 0x8f, 0x1b, 0x05, 0xe4, 0xff]);

    let mut resources: HostResources<DefaultPacketPool, CONNECTIONS_MAX, L2CAP_CHANNELS_MAX>
        = HostResources::new();

    let stack = trouble_host::new(controller, &mut resources)
        .set_random_address(address);

    let Host {
        mut peripheral, runner, ..
    } = stack.build();

    let server = TimeServer::new_with_config(GapConfig::Peripheral(PeripheralConfig {
        name: "ESP32-Clock",
        appearance: &appearance::generic::GENERIC_CLOCK,
    }))
    .unwrap();

    // Run BLE stack and application tasks
    join(
        runner.run(),
        peripheral_task(&mut peripheral, &server, &stack)
    ).await;
}
```

### 4. Handle Advertising and Connections

```rust
async fn peripheral_task<C: Controller, P: PacketPool>(
    peripheral: &mut Peripheral<'_, C, P>,
    server: &TimeServer<'_>,
    stack: &Stack<'_, C, P>,
) {
    loop {
        // Advertise
        let conn = match advertise("ESP32-Clock", peripheral, server).await {
            Ok(c) => c,
            Err(e) => {
                esp_println::println!("Advertise error: {:?}", e);
                continue;
            }
        };

        esp_println::println!("Connected!");

        // Handle connection
        if let Err(e) = handle_connection(&conn, server, stack).await {
            esp_println::println!("Connection error: {:?}", e);
        }

        esp_println::println!("Disconnected");
    }
}

async fn advertise<'a, C: Controller, P: PacketPool>(
    name: &'a str,
    peripheral: &mut Peripheral<'a, C, P>,
    server: &'a TimeServer<'a>,
) -> Result<GattConnection<'a, 'a, P>, BleHostError<C::Error>> {
    let mut adv_data = [0; 31];
    let len = AdStructure::encode_slice(
        &[
            AdStructure::Flags(LE_GENERAL_DISCOVERABLE | BR_EDR_NOT_SUPPORTED),
            AdStructure::CompleteLocalName(name.as_bytes()),
        ],
        &mut adv_data,
    )?;

    let advertiser = peripheral
        .advertise(
            &Default::default(),
            Advertisement::ConnectableScannableUndirected {
                adv_data: &adv_data[..len],
                scan_data: &[],
            },
        )
        .await?;

    esp_println::println!("Advertising...");
    let conn = advertiser.accept().await?.with_attribute_server(server)?;
    Ok(conn)
}
```

### 5. Handle GATT Events (Receive Time)

```rust
use embassy_sync::signal::Signal;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;

// Shared signal to notify display task of time updates
static TIME_UPDATED: Signal<CriticalSectionRawMutex, [u8; 10]> = Signal::new();

async fn handle_connection<P: PacketPool>(
    conn: &GattConnection<'_, '_, P>,
    server: &TimeServer<'_>,
    stack: &Stack<'_, impl Controller, P>,
) -> Result<(), Error> {
    let time_char = server.time_service.current_time;

    loop {
        match conn.next().await {
            GattConnectionEvent::Disconnected { reason } => {
                esp_println::println!("Disconnected: {:?}", reason);
                return Ok(());
            }
            GattConnectionEvent::Gatt { event } => {
                match &event {
                    GattEvent::Write(write_event) => {
                        if write_event.handle() == time_char.handle {
                            let data = write_event.data();
                            if data.len() == 10 {
                                // Parse the time data
                                let mut time_bytes = [0u8; 10];
                                time_bytes.copy_from_slice(data);

                                // Extract time components
                                let year = u16::from_le_bytes([data[0], data[1]]);
                                let month = data[2];
                                let day = data[3];
                                let hours = data[4];
                                let minutes = data[5];
                                let seconds = data[6];

                                esp_println::println!(
                                    "Received time: {:04}-{:02}-{:02} {:02}:{:02}:{:02}",
                                    year, month, day, hours, minutes, seconds
                                );

                                // Store in server
                                server.set(&time_char, &time_bytes);

                                // Signal display task
                                TIME_UPDATED.signal(time_bytes);
                            }
                        }
                    }
                    GattEvent::Read(read_event) => {
                        if read_event.handle() == time_char.handle {
                            let value = server.get(&time_char);
                            esp_println::println!("Time read: {:?}", value);
                        }
                    }
                    _ => {}
                }

                // Send response
                match event.accept() {
                    Ok(reply) => reply.send().await,
                    Err(e) => esp_println::println!("Error sending response: {:?}", e),
                }
            }
            _ => {}
        }
    }
}
```

### 6. Display Time on OLED

Create a separate task to update the OLED:

```rust
use embassy_executor::Spawner;
use embedded_graphics::{
    mono_font::{ascii::FONT_6X10, MonoTextStyle},
    pixelcolor::BinaryColor,
    prelude::*,
    text::Text,
};
use ssd1306::{prelude::*, Ssd1306};

#[embassy_executor::task]
async fn display_task(i2c: /* your I2C peripheral */) {
    // Set up OLED (same as your oled_display example)
    let interface = I2CDisplayInterface::new(i2c);
    let mut display = Ssd1306::new(interface, DisplaySize72x40, DisplayRotation::Rotate0)
        .into_buffered_graphics_mode();
    display.init().unwrap();

    let text_style = MonoTextStyle::new(&FONT_6X10, BinaryColor::On);

    loop {
        // Wait for time update
        let time_data = TIME_UPDATED.wait().await;

        // Parse time
        let hours = time_data[4];
        let minutes = time_data[5];
        let seconds = time_data[6];

        // Clear display
        display.clear(BinaryColor::Off).unwrap();

        // Format and display time
        let mut time_str = heapless::String::<16>::new();
        use core::fmt::Write;
        write!(&mut time_str, "{:02}:{:02}:{:02}", hours, minutes, seconds).unwrap();

        Text::new(&time_str, Point::new(10, 20), text_style)
            .draw(&mut display)
            .unwrap();

        display.flush().unwrap();
    }
}
```

### 7. Launch the Display Task

In your `main` function:

```rust
#[esp_rtos::main]
async fn main(spawner: Spawner) {
    // ... initialization ...

    // Set up I2C for OLED
    let i2c = I2c::new(
        peripherals.I2C0,
        peripherals.GPIO5,  // SDA
        peripherals.GPIO6,  // SCL
        Config::default(),
    );

    // Spawn display task
    spawner.spawn(display_task(i2c)).unwrap();

    // Run BLE peripheral
    run_time_server(controller).await;
}
```

## Testing with a Phone/Computer

### Using nRF Connect (Mobile App)

1. Install **nRF Connect** app (Android/iOS)
2. Scan for devices, find "ESP32-Clock"
3. Connect to the device
4. Find your custom service UUID
5. Click the time characteristic
6. Write 10 bytes in hex format:
   ```
   Example for 2026-01-11 14:30:45:
   EA 07 01 0B 0E 1E 2D 06 00 00

   Breakdown:
   EA 07    = 2026 (0x07EA little-endian)
   01       = January
   0B       = 11th day
   0E       = 14 hours (2 PM)
   1E       = 30 minutes
   2D       = 45 seconds
   06       = Saturday (1=Mon, 6=Sat)
   00       = Fractions (0)
   00       = Adjust reason (none)
   ```

### Using Python (Linux/Mac/Windows)

```python
import asyncio
from bleak import BleakClient, BleakScanner
import struct
from datetime import datetime

DEVICE_NAME = "ESP32-Clock"
TIME_CHAR_UUID = "12345678-1234-5678-1234-56789abcdef1"

async def send_time():
    # Find device
    device = await BleakScanner.find_device_by_name(DEVICE_NAME)
    if not device:
        print(f"Device {DEVICE_NAME} not found")
        return

    async with BleakClient(device) as client:
        # Get current time
        now = datetime.now()

        # Pack into 10 bytes
        time_data = struct.pack(
            '<HBBBBBBBB',
            now.year,           # uint16 little-endian
            now.month,          # uint8
            now.day,            # uint8
            now.hour,           # uint8
            now.minute,         # uint8
            now.second,         # uint8
            now.weekday() + 1,  # uint8 (1=Monday)
            0,                  # fractions256
            0                   # adjust_reason
        )

        # Write to characteristic
        await client.write_gatt_char(TIME_CHAR_UUID, time_data)
        print(f"Sent time: {now}")

asyncio.run(send_time())
```

## Key Library Functions Summary

### trouble_host::prelude

| Function/Type | Purpose |
|---------------|---------|
| `trouble_host::new(controller, resources)` | Create BLE stack |
| `stack.build()` | Build and get Peripheral/Central/Runner |
| `peripheral.advertise(...)` | Start advertising |
| `advertiser.accept()` | Wait for connection |
| `conn.with_attribute_server(server)` | Attach GATT server |
| `conn.next()` | Get next GATT event |
| `server.get(&characteristic)` | Read characteristic value |
| `server.set(&characteristic, &value)` | Write characteristic value |
| `characteristic.notify(conn, &value)` | Send notification |

### GATT Macros

| Macro | Purpose |
|-------|---------|
| `#[gatt_server]` | Define GATT server struct |
| `#[gatt_service(uuid = ...)]` | Define service |
| `#[characteristic(uuid = ..., read, write, notify)]` | Define characteristic |

### UUIDs (bt_hci::uuid)

| Constant | UUID | Purpose |
|----------|------|---------|
| `service::CURRENT_TIME` | 0x1805 | Official Current Time Service |
| `characteristic::CURRENT_TIME` | 0x2A2B | Current Time Characteristic |
| `service::BATTERY` | 0x180F | Battery Service (example) |
| `appearance::generic::GENERIC_CLOCK` | - | Device appearance |

## Next Steps

1. Start with the BLE peripheral example from `trouble/examples/esp32/src/bin/ble_bas_peripheral.rs`
2. Modify the GATT server to use your time service
3. Test receiving time data with nRF Connect
4. Integrate OLED display code from your existing `oled_display.rs` example
5. Use `embassy_sync::signal::Signal` to communicate between BLE and display tasks

## Resources

- [Bluetooth CTS Specification](https://www.bluetooth.com/specifications/specs/cts-1-1/)
- [Current Time Service HTML Spec](https://www.bluetooth.com/wp-content/uploads/Files/Specification/HTML/CTS_v1.0/out/en/index-en.html)
- [trouble-host Documentation](https://docs.embassy.dev/trouble-host/git/default/index.html)
- [bt-hci Rust Documentation](https://docs.rs/bt-hci/latest/bt_hci/)
- [embassy-rs Documentation](https://embassy.dev/)
- [Nordic Developer Zone CTS Discussion](https://devzone.nordicsemi.com/f/nordic-q-a/67682/how-are-cts-0x1805-services-discovered-when-acting-as-a-gatt-server-with-an-esp32-gatt-client)

## Common Pitfalls

1. **Byte order**: Multi-byte values (year) are little-endian
2. **Data length**: Current Time is exactly 10 bytes, validate length
3. **Invalid values**: Month (1-12), Day (1-31), Hours (0-23), etc.
4. **Memory**: Ensure sufficient heap (72KB already allocated in your examples)
5. **Task communication**: Use `embassy_sync::signal::Signal` for thread-safe communication
6. **Display updates**: Don't block BLE tasks with display operations
