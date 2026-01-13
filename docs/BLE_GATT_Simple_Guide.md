# BLE GATT Simple - Complete Guide

This guide explains the simple BLE GATT server/client example that demonstrates bidirectional communication between an ESP32-C3 and a Linux computer.

## Overview

This example consists of two components:
- **`ble_gatt_simple.rs`**: ESP32-C3 BLE GATT server (Peripheral)
- **`ble_gatt_client.py`**: Python BLE client (Central) for Linux

Together they demonstrate:
- Reading characteristics
- Writing characteristics
- Receiving notifications from the server
- Auto-updating counter values

---

## ESP32-C3 GATT Server (`examples/ble_gatt_simple.rs`)

### Architecture

The server uses the **trouble-host** BLE stack with procedural macros to define the GATT structure at compile time.

### GATT Structure

```
Server
└── Simple Service (UUID: f0debc9a-7856-3412-7856-341278563412)
    ├── Counter Characteristic (UUID: f1debc9a-7856-3412-7856-341278563412)
    │   ├── Properties: Read, Notify
    │   ├── Type: u32
    │   └── Behavior: Auto-increments every 5 seconds
    │
    └── Data Characteristic (UUID: f2debc9a-7856-3412-7856-341278563412)
        ├── Properties: Read, Write, Notify
        ├── Type: u8
        └── Behavior: Can be written by client, sends notification on write
```

### UUID Definition

The UUIDs are defined as 128-bit values using `Uuid::new_long()`:

```rust
const CUSTOM_SERVICE_UUID: Uuid = Uuid::new_long([
    0x12, 0x34, 0x56, 0x78, 0x12, 0x34, 0x56, 0x78,
    0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc, 0xde, 0xf0,
]);
```

**Note**: Due to BLE byte ordering, these appear differently when transmitted:
- Code: `12345678-1234-5678-1234-56789abcdef0`
- Wire format: `f0debc9a-7856-3412-7856-341278563412`

### GATT Server Definition

The server structure is defined using procedural macros that generate all the necessary GATT attribute table code at compile time:

```rust
#[gatt_server]
struct Server {
    simple_service: SimpleService,
}

#[gatt_service(uuid = CUSTOM_SERVICE_UUID)]
struct SimpleService {
    /// Counter characteristic (read + notify)
    #[characteristic(uuid = COUNTER_CHAR_UUID, read, notify, value = 0u32)]
    counter: u32,

    /// Data characteristic (read + write + notify)
    #[characteristic(uuid = DATA_CHAR_UUID, read, write, notify, value = 0u8)]
    data: u8,
}
```

**Key Points**:
- `#[gatt_server]`: Marks the top-level server struct
- `#[gatt_service]`: Defines a BLE service with a specific UUID
- `#[characteristic]`: Defines a characteristic with properties and initial value
- Properties: `read`, `write`, `notify` control what operations are allowed

### Main Function Flow

```rust
#[esp_rtos::main]
async fn main(_s: Spawner) {
    // 1. Initialize logger
    esp_println::logger::init_logger_from_env();

    // 2. Initialize ESP32-C3 peripherals
    let peripherals = esp_hal::init(...);
    esp_alloc::heap_allocator!(size: 72 * 1024);

    // 3. Start Embassy RTOS
    esp_rtos::start(...);

    // 4. Initialize BLE controller
    let connector = BleConnector::new(bluetooth, Default::default()).unwrap();
    let controller: ExternalController<_, 20> = ExternalController::new(connector);

    // 5. Run BLE server
    run_ble_server(controller).await;
}
```

### BLE Server Tasks

The server runs multiple concurrent async tasks:

#### 1. BLE Task (`ble_task`)
```rust
async fn ble_task<C: Controller, P: PacketPool>(mut runner: Runner<'_, C, P>) {
    loop {
        if let Err(e) = runner.run().await {
            log::error!("BLE task error: {:?}", e);
        }
    }
}
```
- **Purpose**: Runs the BLE stack's background operations
- **Must run continuously** for BLE to function
- Handles low-level HCI communication with the BLE controller

#### 2. Advertising Loop
```rust
loop {
    match advertise(&mut peripheral, &server).await {
        Ok(conn) => {
            log::info!("Client connected!");
            let gatt_task = gatt_events_task(&server, &conn);
            let counter_task = counter_update_task(&server, &conn);
            select(gatt_task, counter_task).await;
            log::info!("Client disconnected");
        }
        Err(e) => {
            log::error!("Advertising error: {:?}", e);
        }
    }
}
```
- Advertises as "ESP32-GATT"
- Waits for client connection
- Spawns connection-specific tasks
- Returns to advertising when connection closes

#### 3. GATT Events Task (`gatt_events_task`)
```rust
async fn gatt_events_task<P: PacketPool>(
    server: &Server<'_>,
    conn: &GattConnection<'_, '_, P>,
) -> Result<(), Error>
```

Handles GATT operations from the client:

**Read Events**:
```rust
GattEvent::Read(read_event) => {
    if read_event.handle() == counter.handle {
        log::info!("[READ] Client read Counter characteristic");
    }
}
```
- Logs when client reads a characteristic
- The BLE stack automatically sends the current value

**Write Events**:
```rust
GattEvent::Write(write_event) => {
    if write_event.handle() == data.handle {
        let bytes = write_event.data();
        let value = bytes[0];
        log::info!("[WRITE] Client wrote {} to Data characteristic", value);

        // Send notification to client
        data.notify(conn, &value).await;
    }
}
```
- Receives data written by client
- Sends notification back to confirm the write

**Response Handling**:
```rust
match event.accept() {
    Ok(reply) => reply.send().await,
    Err(e) => log::warn!("Error sending GATT response: {:?}", e),
}
```
- Every GATT operation requires a response
- `accept()` creates the response
- `send()` transmits it to the client

#### 4. Counter Update Task (`counter_update_task`)
```rust
async fn counter_update_task<P: PacketPool>(
    server: &Server<'_>,
    conn: &GattConnection<'_, '_, P>,
)
```

Autonomous task that updates the counter:

```rust
loop {
    Timer::after_secs(5).await;
    count = count.wrapping_add(1);
    log::info!("[UPDATE] Counter incremented to: {}", count);

    // Notify connected client
    if let Err(e) = counter.notify(conn, &count).await {
        log::warn!("[NOTIFY] Failed to notify counter: {:?}", e);
        break;
    }
}
```
- Runs every 5 seconds
- Increments counter
- Sends notification to client with new value
- Exits if notification fails (connection closed)

### Building and Running

```bash
# Build with logging enabled
ESP_LOG=info cargo build --release --example ble_gatt_simple --features ble

# Or use the build script
ESP_LOG=info ./scripts/build-example.sh ble_gatt_simple
```

### Log Output Example

```
=== Simple BLE GATT Server Starting ===
BLE Address: Address::Random([255, 228, 5, 26, 143, 255])
Creating GATT server
GATT Service UUIDs:
  Service:  12345678-1234-5678-1234-56789abcdef0
  Counter:  12345678-1234-5678-1234-56789abcdef1 (read, notify)
  Data:     12345678-1234-5678-1234-56789abcdef2 (read, write, notify)
Advertising as 'ESP32-GATT'...
Client connected!
[READ] Client read Counter characteristic
[READ] Client read Data characteristic
[WRITE] Client wrote 42 to Data characteristic
[NOTIFY] Sent notification with value 42
[UPDATE] Counter incremented to: 1
[NOTIFY] Sent counter notification: 1
```

---

## Python BLE Client (`scripts/ble_gatt_client.py`)

### Architecture

The client uses the **Bleak** library, a cross-platform Python BLE library that works on Linux, macOS, and Windows.

### Dependencies

```bash
pip install bleak
```

Bleak uses the native BLE stack on each platform:
- **Linux**: BlueZ (via D-Bus)
- **macOS**: CoreBluetooth
- **Windows**: Windows BLE APIs

### Main Functions

#### 1. Device Discovery (`find_device`)
```python
async def find_device():
    """Scan for the ESP32-GATT device."""
    print(f"Scanning for '{DEVICE_NAME}'...")
    devices = await BleakScanner.discover(timeout=10.0)

    for device in devices:
        if device.name == DEVICE_NAME:
            print(f"  Found device: {device.name} ({device.address})")
            return device.address

    return None
```

**Process**:
1. Scans for BLE devices for 10 seconds
2. Looks for device with name "ESP32-GATT"
3. Returns the device's Bluetooth address
4. Returns `None` if not found

**BLE Address Format**: `FF:E4:05:1A:8F:FF` (the random address set in the ESP32 code)

#### 2. Connection and Interaction (`main`)

**Connect to Device**:
```python
async with BleakClient(address) as client:
    print(f"  Connected: {client.is_connected}")
```
- Uses async context manager for automatic connection/disconnection
- `BleakClient` handles all connection management

**List Services**:
```python
for service in client.services:
    print(f"  Service: {service.uuid}")
    for char in service.characteristics:
        props = ', '.join(char.properties)
        print(f"    Characteristic: {char.uuid} ({props})")
```
- Enumerates all GATT services and characteristics
- Shows UUIDs and properties (read, write, notify)

**Enable Notifications**:
```python
await client.start_notify(COUNTER_CHAR_UUID, counter_notification_handler)
await client.start_notify(DATA_CHAR_UUID, data_notification_handler)
```
- Registers callback functions for notifications
- Notifications arrive asynchronously
- Callbacks are called whenever the ESP32 sends a notification

**Notification Handlers**:
```python
def counter_notification_handler(sender, data):
    """Handle notifications from the counter characteristic."""
    value = int.from_bytes(data, byteorder='little')
    print(f"  [COUNTER NOTIFICATION] Value: {value}")

def data_notification_handler(sender, data):
    """Handle notifications from the data characteristic."""
    value = int.from_bytes(data, byteorder='little')
    print(f"  [DATA NOTIFICATION] Value: {value}")
```
- `sender`: Handle of the characteristic that sent the notification
- `data`: Raw bytes received
- `byteorder='little'`: ESP32 uses little-endian byte order

**Reading Characteristics**:
```python
data = await client.read_gatt_char(COUNTER_CHAR_UUID)
counter_value = int.from_bytes(data, byteorder='little')
print(f"  Counter: {counter_value}")
```
- Sends a GATT read request
- Returns the current value
- Must convert bytes to appropriate type

**Writing Characteristics**:
```python
await client.write_gatt_char(DATA_CHAR_UUID, (42).to_bytes(1, byteorder='little'))
```
- Sends a GATT write request
- Must convert value to bytes
- Data characteristic is `u8`, so 1 byte
- Counter characteristic is `u32`, so 4 bytes

### Demo Sequence

The script demonstrates all operations in sequence:

```python
# 1. Read counter (current value)
# 2. Read data (initial value, should be 0)
# 3. Write 42 to data → triggers notification
# 4. Read data again (should be 42)
# 5. Write multiple values (100, 150, 200, 255) → each triggers notification
# 6. Wait 15 seconds for counter notifications (counter updates every 5 seconds)
# 7. Disable notifications
```

### Example Output

```
Scanning for 'ESP32-GATT'...
  Found device: ESP32-GATT (FF:E4:05:1A:8F:FF)

Connecting to FF:E4:05:1A:8F:FF...
  Connected: True

Available services:
  Service: f0debc9a-7856-3412-7856-341278563412
    Characteristic: f1debc9a-7856-3412-7856-341278563412 (read, notify)
    Characteristic: f2debc9a-7856-3412-7856-341278563412 (read, write, notify)

Enabling notifications...
  Notifications enabled

1. Reading counter characteristic...
  Counter: 0

2. Reading data characteristic (initial value)...
  Data: 0

3. Writing value 42 to data characteristic...
  Write successful
  [DATA NOTIFICATION] Value: 42

4. Reading data characteristic again...
  Data: 42

5. Writing several values to data characteristic...
  Writing 100...
  [DATA NOTIFICATION] Value: 100
  Writing 150...
  [DATA NOTIFICATION] Value: 150
  Writing 200...
  [DATA NOTIFICATION] Value: 200
  Writing 255...
  [DATA NOTIFICATION] Value: 255

6. Waiting 15 seconds for counter notifications...
   (Counter increments every 5 seconds)
  [COUNTER NOTIFICATION] Value: 1
  [COUNTER NOTIFICATION] Value: 2
  [COUNTER NOTIFICATION] Value: 3

Disabling notifications...

Demo complete!
```

### Running the Client

```bash
# Make executable (if needed)
chmod +x scripts/ble_gatt_client.py

# Run the client
./scripts/ble_gatt_client.py

# Or with python3
python3 scripts/ble_gatt_client.py
```

---

## Key BLE Concepts Demonstrated

### 1. GATT (Generic Attribute Profile)

GATT is the protocol used for reading/writing data in BLE:

- **Service**: A collection of related characteristics
- **Characteristic**: A single data value with properties
- **Properties**: What operations are allowed (read, write, notify)
- **Handle**: A unique identifier for each attribute

### 2. Characteristic Properties

**Read**: Client can request the current value
```rust
#[characteristic(uuid = ..., read, ...)]
```

**Write**: Client can update the value
```rust
#[characteristic(uuid = ..., write, ...)]
```

**Notify**: Server can push updates to client
```rust
#[characteristic(uuid = ..., notify, ...)]
```
- Client must enable notifications first
- Server can then send updates asynchronously
- More efficient than polling with reads

### 3. Client-Server Roles

**Peripheral (ESP32-C3)**:
- Advertises its presence
- Hosts the GATT server
- Provides characteristics
- Responds to read/write requests
- Sends notifications

**Central (Linux computer)**:
- Scans for peripherals
- Initiates connections
- Acts as GATT client
- Reads/writes characteristics
- Receives notifications

### 4. Data Types and Byte Order

BLE transmits raw bytes, so type conversion is important:

**ESP32 (Rust)**:
```rust
counter: u32,  // 4 bytes, little-endian
data: u8,      // 1 byte
```

**Python**:
```python
# Reading u32
value = int.from_bytes(data, byteorder='little')  # 4 bytes → int

# Writing u8
bytes_to_send = value.to_bytes(1, byteorder='little')  # int → 1 byte
```

### 5. Async/Await Pattern

Both sides use async/await for non-blocking I/O:

**ESP32 (Rust)**:
```rust
async fn gatt_events_task(...) {
    match conn.next().await {  // Wait for next event
        GattConnectionEvent::Gatt { event } => {
            // Handle event
            event.accept()?.send().await;  // Send response
        }
    }
}
```

**Python**:
```python
async def main():
    async with BleakClient(address) as client:
        data = await client.read_gatt_char(uuid)  # Wait for response
        await client.write_gatt_char(uuid, data)  # Wait for completion
```

---

## Troubleshooting

### ESP32 Not Advertising

**Check**:
1. ESP32 is powered and running
2. Build with `ESP_LOG=info` to see log output
3. Serial monitor shows "Advertising as 'ESP32-GATT'..."

### Python Can't Find Device

**Check**:
1. Bluetooth is enabled on Linux: `bluetoothctl power on`
2. Python has permissions: May need to run as root or add user to `bluetooth` group
3. Bleak is installed: `pip install bleak`
4. Device is within range and advertising

### Connection Fails

**Check**:
1. No other device is connected to the ESP32
2. ESP32 supports only 1 connection at a time
3. Restart ESP32 if in a bad state

### Notifications Not Received

**Check**:
1. Notifications were enabled with `start_notify()`
2. ESP32 logs show "[NOTIFY]" messages
3. Client is still connected

### UUID Mismatch

**Remember**: UUIDs appear different on the wire due to byte ordering:
- Code: `12345678-1234-5678-1234-56789abcdef0`
- BLE: `f0debc9a-7856-3412-7856-341278563412`

Always use the UUIDs as reported by the client's service discovery.

---

## Extending the Example

### Adding More Characteristics

```rust
#[gatt_service(uuid = CUSTOM_SERVICE_UUID)]
struct SimpleService {
    #[characteristic(uuid = ..., read, notify, value = 0u32)]
    counter: u32,

    #[characteristic(uuid = ..., read, write, notify, value = 0u8)]
    data: u8,

    // Add new characteristic
    #[characteristic(uuid = NEW_UUID, read, write, value = 0u16)]
    new_value: u16,
}
```

### Adding Descriptors

```rust
#[characteristic(
    uuid = ...,
    read,
    notify,
    value = 100u8
)]
#[descriptor(uuid = descriptors::VALID_RANGE, read, value = [0, 100])]
#[descriptor(uuid = descriptors::MEASUREMENT_DESCRIPTION, read, value = "Battery Level")]
level: u8,
```

### Custom UUID Generation

Use a UUID generator to create unique UUIDs:
```bash
# Linux
uuidgen

# Or online: https://www.uuidgenerator.net/
```

Then define in Rust:
```rust
const MY_UUID: Uuid = Uuid::new_long([
    0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff, 0x00, 0x11,
    0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88, 0x99,
]);
```

---

## References

- **trouble-host Documentation**: BLE stack for embedded Rust
- **Bleak Documentation**: https://bleak.readthedocs.io/
- **Bluetooth SIG GATT Specifications**: https://www.bluetooth.com/specifications/gatt/
- **ESP32-C3 Documentation**: https://docs.espressif.com/projects/esp-idf/en/latest/esp32c3/
