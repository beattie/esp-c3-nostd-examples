# ESP32-C3 BLE GATT Server Examples Explained

This document explains the BLE GATT server examples from the trouble project, specifically the ESP32-C3 peripheral implementations.

## Overview

Both files are **ESP32-C3 BLE GATT server examples** that demonstrate a **peripheral** (server) role. They share the same hardware initialization but differ in their BLE functionality.

## File Locations

```
/home/beattie/projects/trouble/examples/esp32/src/bin/
├── ble_bas_peripheral.rs          ← Simple (RECOMMENDED FOR LEARNING)
└── ble_bas_peripheral_bonding.rs  ← Advanced (security/bonding)

Shared application logic:
/home/beattie/projects/trouble/examples/apps/src/
├── ble_bas_peripheral.rs          ← Simple GATT logic
└── ble_bas_peripheral_bonding.rs  ← Advanced with bonding
```

**Important:** The ESP32 bin files are just **hardware wiring** - they initialize peripherals and call the shared application code. The real GATT server logic is in `examples/apps/src/`.

---

## `ble_bas_peripheral.rs` - Simple GATT Server

### What It Does

A basic BLE peripheral that advertises a **Battery Service** (BAS) and allows centrals (phones/computers) to connect and read/write battery data.

### Key Features

- **One service**: Battery Service (UUID 0x180F)
- **Two characteristics**:
  - `level` (u8) - Battery level 0-100, can read/notify
  - `status` (bool) - Custom status, can read/write/notify
- **No security** - Open connection, no pairing required
- **No persistence** - Forgets everything on reboot

### Hardware Setup

```rust
#[esp_rtos::main]
async fn main(_s: Spawner) {
    esp_println::logger::init_logger_from_env();

    // Initialize peripherals with max CPU clock
    let peripherals = esp_hal::init(
        esp_hal::Config::default().with_cpu_clock(CpuClock::max())
    );

    // Allocate 72KB heap for BLE stack
    esp_alloc::heap_allocator!(size: 72 * 1024);

    // Set up timer group
    let timg0 = TimerGroup::new(peripherals.TIMG0);

    // RISC-V specific: software interrupts
    let software_interrupt = esp_hal::interrupt::software::SoftwareInterruptControl::new(
        peripherals.SW_INTERRUPT
    );

    // Start ESP-RTOS with timer and interrupt
    esp_rtos::start(timg0.timer0, software_interrupt.software_interrupt0);

    // Initialize BLE controller
    let bluetooth = peripherals.BT;
    let connector = BleConnector::new(bluetooth, Default::default()).unwrap();
    let controller: ExternalController<_, 20> = ExternalController::new(connector);

    // Run the BLE peripheral application
    ble_bas_peripheral::run(controller).await;
}
```

### The Core Application Logic

From `examples/apps/src/ble_bas_peripheral.rs`:

```rust
// GATT Server definition
#[gatt_server]
struct Server {
    battery_service: BatteryService,
}

// Battery service
#[gatt_service(uuid = service::BATTERY)]
struct BatteryService {
    // Battery level characteristic with descriptors
    #[descriptor(uuid = descriptors::VALID_RANGE, read, value = [0, 100])]
    #[descriptor(uuid = descriptors::MEASUREMENT_DESCRIPTION, name = "hello", read, value = "Battery Level")]
    #[characteristic(uuid = characteristic::BATTERY_LEVEL, read, notify, value = 10)]
    level: u8,

    // Custom status characteristic
    #[characteristic(uuid = "408813df-5dd4-1f87-ec11-cdb001100000", write, read, notify)]
    status: bool,
}
```

### Application Flow

1. **Initialize BLE stack** with random address
2. **Build host** with peripheral role
3. **Create GATT server** with Battery Service
4. **Advertise** as "TrouBLE" device
5. **Wait for connection** from central
6. **Handle GATT events**:
   - Read requests
   - Write requests
   - Notifications
7. **Custom task** sends battery level updates every 2 seconds
8. **On disconnect** - return to advertising

### Use Case

**Perfect starting point for:**
- Learning BLE peripheral basics
- Simple sensor data reporting
- Devices that don't need pairing/security
- **Your time display project!**

---

## `ble_bas_peripheral_bonding.rs` - Secure GATT Server with Bonding

### What It Does

Advanced BLE peripheral with **pairing, bonding, and persistent storage**. Includes both Battery Service and HID (keyboard) service.

### Key Features

- **Two services**:
  - Battery Service (0x180F)
  - HID Service (0x1812) - Human Interface Device (keyboard)
- **Security**: Pairing required with encryption
- **Bonding**: Stores pairing keys in flash memory
- **Persistent**: Remembers paired devices across reboots
- **More complex**: HID keyboard emulation with input/output reports

### Additional Hardware Setup

Beyond the simple peripheral setup, this example adds:

```rust
// True Random Number Generator for cryptographic keys
let _trng_source = TrngSource::new(peripherals.RNG, peripherals.ADC1);
let mut trng = Trng::try_new().unwrap();

// Flash storage for persistent bonding data
let mut flash = embassy_embedded_hal::adapter::BlockingAsync::new(
    FlashStorage::new(peripherals.FLASH)
);

// Run with TRNG and flash for security
ble_bas_peripheral_bonding::run(controller, &mut trng, &mut flash).await;
```

### The Core Application Logic

From `examples/apps/src/ble_bas_peripheral_bonding.rs`:

```rust
// GATT Server with multiple services
#[gatt_server]
struct Server {
    battery_service: BatteryService,
    hid_service: HidService,        // Additional HID service
}

// HID Service for keyboard emulation
#[gatt_service(uuid = service::HUMAN_INTERFACE_DEVICE)]
pub(crate) struct HidService {
    #[characteristic(uuid = "2a4a", read, value = [0x01, 0x01, 0x00, 0x03])]
    pub(crate) hid_info: [u8; 4],

    #[characteristic(uuid = "2a4b", read, value = DESC)]
    pub(crate) report_map: [u8; 67],

    #[characteristic(uuid = "2a4c", write_without_response)]
    pub(crate) hid_control_point: u8,

    #[characteristic(uuid = "2a4d", read, notify)]
    pub(crate) input_keyboard: [u8; 8],

    // ... more HID characteristics
}

// Bonding information stored in flash
struct StoredBondInformation {
    ltk: LongTermKey,                    // Long Term Key for encryption
    security_level: SecurityLevel,
}
```

### Security Features

1. **Pairing**: Device and central establish secure connection
2. **Bonding**: Pairing keys stored in flash memory
3. **Persistent**: Automatically reconnects to known devices
4. **Encryption**: All data encrypted with AES-128
5. **Flash Storage**: Uses `sequential-storage` for key persistence

### Use Case

**For devices that need:**
- Pairing/PIN codes
- Encrypted connections
- Persistent pairing (auto-reconnect)
- HID functionality (keyboard, mouse)
- Security requirements
- Commercial products requiring security

---

## Quick Comparison

| Feature | `ble_bas_peripheral.rs` | `ble_bas_peripheral_bonding.rs` |
|---------|------------------------|--------------------------------|
| **Services** | Battery only | Battery + HID (keyboard) |
| **Security** | None (open) | Pairing + encryption |
| **Storage** | None | Flash (persistent bonding) |
| **RNG** | Not needed | Required for crypto |
| **Complexity** | Simple (~200 lines) | Advanced (~400+ lines) |
| **L2CAP Channels** | 2 (Signal + ATT) | 4 (adds Security Manager) |
| **Hardware** | BLE radio only | BLE + RNG + Flash |
| **Best For** | Learning, sensors | Secure devices, HID |
| **Connect Time** | Instant | Requires pairing |
| **Persistence** | None | Remembers devices |

---

## Which One for Your Time Display?

### Recommendation: Use `ble_bas_peripheral.rs`

#### Why?

1. **Simpler** - No security complexity needed for a clock
2. **Smaller** - Less code to understand and modify
3. **Faster** - No pairing process delays
4. **Sufficient** - Time sync doesn't need encryption
5. **Same structure** - Easy to replace battery service with time service

#### How to Adapt It

Replace the Battery Service with a Time Service:

```rust
// Instead of:
#[gatt_service(uuid = service::BATTERY)]
struct BatteryService {
    #[characteristic(uuid = characteristic::BATTERY_LEVEL, read, notify, value = 10)]
    level: u8,
}

// Use:
#[gatt_service(uuid = "12345678-1234-5678-1234-56789abcdef0")]
struct TimeService {
    #[characteristic(uuid = "12345678-1234-5678-1234-56789abcdef1", write, read, notify)]
    current_time: [u8; 10],
}
```

### When to Use Bonding Version?

Only if you need:
- **Security requirements** - Medical devices, personal data
- **Trusted devices only** - Prevent unauthorized access
- **Persistent pairing** - Device auto-connects to known phone
- **HID functionality** - Keyboard, mouse emulation

---

## Code Structure Pattern

Both examples follow the same pattern:

### 1. Hardware Initialization (`esp32/src/bin/*.rs`)

```rust
#[esp_rtos::main]
async fn main(_s: Spawner) {
    // 1. Initialize ESP32-C3 peripherals
    // 2. Allocate heap memory
    // 3. Set up timers and interrupts
    // 4. Initialize BLE controller
    // 5. Call application logic
}
```

### 2. Application Logic (`apps/src/*.rs`)

```rust
pub async fn run<C>(controller: C) {
    // 1. Create BLE address
    // 2. Build BLE stack (host + peripheral)
    // 3. Define GATT server
    // 4. Start advertising loop:
    //    - Advertise
    //    - Accept connection
    //    - Handle GATT events
    //    - Run custom tasks
    //    - Disconnect -> back to advertising
}
```

### 3. GATT Server Definition

```rust
#[gatt_server]
struct Server {
    service_name: ServiceType,
}

#[gatt_service(uuid = "...")]
struct ServiceType {
    #[characteristic(uuid = "...", read, write, notify)]
    field_name: DataType,
}
```

### 4. Event Handling

```rust
async fn gatt_events_task(server: &Server, conn: &GattConnection) {
    loop {
        match conn.next().await {
            GattConnectionEvent::Gatt { event } => {
                match event {
                    GattEvent::Read(e) => { /* handle read */ },
                    GattEvent::Write(e) => { /* handle write */ },
                }
                event.accept()?.send().await;
            }
            GattConnectionEvent::Disconnected { reason } => break,
        }
    }
}
```

---

## Key Concepts Explained

### GATT Server Macros

The `trouble-host` library provides macros to define GATT servers:

**`#[gatt_server]`** - Defines the top-level server
```rust
#[gatt_server]
struct Server {
    battery_service: BatteryService,
    time_service: TimeService,
}
```

**`#[gatt_service]`** - Defines a service with UUID
```rust
#[gatt_service(uuid = service::BATTERY)]  // Use predefined UUID
// OR
#[gatt_service(uuid = "12345678-1234-5678-1234-56789abcdef0")]  // Custom UUID
struct BatteryService { ... }
```

**`#[characteristic]`** - Defines a characteristic
```rust
#[characteristic(
    uuid = "...",          // Characteristic UUID
    read,                  // Supports read operations
    write,                 // Supports write operations
    notify,                // Supports notifications
    value = 10             // Initial value
)]
field_name: u8,
```

**`#[descriptor]`** - Adds metadata to characteristics
```rust
#[descriptor(uuid = descriptors::VALID_RANGE, read, value = [0, 100])]
#[characteristic(uuid = characteristic::BATTERY_LEVEL, read, notify)]
level: u8,
```

### L2CAP Channels

Logical Link Control and Adaptation Protocol channels:

- **Signal Channel** (1) - Control messages
- **ATT Channel** (1) - GATT attribute protocol
- **Security Manager** (2) - Only needed for bonding

Simple: 2 channels (Signal + ATT)
Bonding: 4 channels (Signal + ATT + SM channels)

### Controller vs Host vs Application

```
┌─────────────────────────────────┐
│   Application (Your Code)       │  ← GATT server, event handling
├─────────────────────────────────┤
│   Host (trouble-host)           │  ← BLE stack, L2CAP, ATT
├─────────────────────────────────┤
│   Controller (esp-radio)        │  ← Radio hardware control
├─────────────────────────────────┤
│   Hardware (ESP32-C3 BLE radio) │  ← Physical radio
└─────────────────────────────────┘
```

---

## Common Patterns

### Advertising Loop

```rust
loop {
    match advertise(&mut peripheral, &server).await {
        Ok(conn) => {
            // Handle connection
            handle_connection(&conn).await;
            // After disconnect, loop back to advertising
        }
        Err(e) => {
            panic!("Advertising failed: {:?}", e);
        }
    }
}
```

### GATT Event Processing

```rust
loop {
    match conn.next().await {
        GattConnectionEvent::Gatt { event } => {
            // Process event
            match event.accept() {
                Ok(reply) => reply.send().await,
                Err(e) => warn!("Error: {:?}", e),
            }
        }
        GattConnectionEvent::Disconnected { reason } => {
            info!("Disconnected: {:?}", reason);
            break;
        }
    }
}
```

### Running Multiple Tasks

```rust
use embassy_futures::select::select;

// Run two tasks, exit when either completes
select(
    gatt_events_task(&server, &conn),
    custom_task(&server, &conn)
).await;
```

---

## Related Documentation

- [BLE Time Display Implementation Guide](BLE_Time_Display.md)
- [trouble-host Documentation](https://docs.embassy.dev/trouble-host/git/default/index.html)
- [ESP32-C3 Examples in trouble repo](https://github.com/embassy-rs/trouble/tree/main/examples/esp32)
- [Bluetooth GATT Specification](https://www.bluetooth.com/specifications/specs/)

## Next Steps

1. **Study** `ble_bas_peripheral.rs` - Understand the basic structure
2. **Copy** the example to your project
3. **Modify** the GATT service to use time instead of battery
4. **Test** with `ble_time_sync.py` tool
5. **Integrate** OLED display
6. **Deploy** to ESP32-C3

For a complete implementation guide, see [BLE_Time_Display.md](BLE_Time_Display.md).
