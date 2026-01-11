# ESP32-C3 BLE Tools

Python tools for testing and interacting with ESP32-C3 BLE examples.

## Setup

### Install Dependencies

```bash
# Create virtual environment (recommended)
python3 -m venv venv
source venv/bin/activate

# Install dependencies
pip install -r requirements.txt
```

### System Requirements (Debian/Ubuntu)

```bash
# Install BlueZ (Bluetooth stack)
sudo apt update
sudo apt install bluez bluetooth

# Add user to bluetooth group
sudo usermod -a -G bluetooth $USER

# Log out and back in for group changes to take effect
```

## BLE Time Sync Tool

### Overview

The `ble_time_sync.py` tool sends the current system time to an ESP32-C3 running the BLE time display example. It formats the time according to the Bluetooth Current Time Service specification and writes it to the GATT characteristic.

### Quick Start

```bash
# Make executable
chmod +x ble_time_sync.py

# Send time once
./ble_time_sync.py

# Or with python3
python3 ble_time_sync.py
```

### Usage Examples

#### Scan for Devices

```bash
./ble_time_sync.py --scan
```

Output:
```
Scanning for BLE devices (5s)...

Found 3 device(s):
----------------------------------------------------------------------
  ESP32-Clock                    [AA:BB:CC:DD:EE:FF]
  iPhone                         [11:22:33:44:55:66]
  (Unknown)                      [77:88:99:AA:BB:CC]
----------------------------------------------------------------------
```

#### Send Time Once

```bash
# To default device (ESP32-Clock)
./ble_time_sync.py

# To specific device
./ble_time_sync.py --device "MyESP32"
```

Output:
```
Searching for device: ESP32-Clock
Scanning for BLE devices (5s)...
Found device: ESP32-Clock [AA:BB:CC:DD:EE:FF]
Connected to ESP32-Clock
Sending time: 2026-01-11 Sat 15:30:45
  Hex data: ea 07 01 0b 0f 1e 2d 06 00 00
✓ Time sent successfully
```

#### Continuous Sync

Automatically resync time at regular intervals:

```bash
# Sync every 60 seconds (default)
./ble_time_sync.py --continuous

# Sync every 30 seconds
./ble_time_sync.py --continuous --interval 30

# Sync every 5 minutes
./ble_time_sync.py --continuous --interval 300
```

Output:
```
Starting continuous sync (interval: 60s)
Press Ctrl+C to stop
Searching for device: ESP32-Clock
...
✓ Time sent successfully
Waiting 60s until next sync...
```

Press `Ctrl+C` to stop.

#### Custom UUIDs

If you're using custom service/characteristic UUIDs:

```bash
./ble_time_sync.py \
  --service "12345678-1234-5678-1234-56789abcdef0" \
  --characteristic "12345678-1234-5678-1234-56789abcdef1"
```

#### Official Current Time Service

To use the official Bluetooth CTS UUIDs:

```bash
./ble_time_sync.py \
  --service "00001805-0000-1000-8000-00805f9b34fb" \
  --characteristic "00002a2b-0000-1000-8000-00805f9b34fb"
```

### Command Line Options

```
usage: ble_time_sync.py [-h] [--scan] [--device DEVICE] [--service SERVICE]
                        [--characteristic CHARACTERISTIC] [--continuous]
                        [--interval INTERVAL] [--timeout TIMEOUT]

Options:
  -h, --help            Show help message
  --scan                Scan for BLE devices and exit
  -d, --device DEVICE   Device name to connect to (default: ESP32-Clock)
  -s, --service UUID    Service UUID
  -c, --characteristic UUID
                        Time characteristic UUID
  --continuous          Continuously sync time at intervals
  -i, --interval N      Sync interval in seconds (default: 60)
  -t, --timeout N       Scan timeout in seconds (default: 5.0)
```

### Time Data Format

The tool sends time data in 10-byte format according to Bluetooth Current Time Service specification:

| Bytes | Field | Description | Example |
|-------|-------|-------------|---------|
| 0-1   | Year  | uint16 little-endian (1582-9999) | `0xEA 0x07` = 2026 |
| 2     | Month | uint8 (1-12) | `0x01` = January |
| 3     | Day   | uint8 (1-31) | `0x0B` = 11th |
| 4     | Hours | uint8 (0-23) | `0x0F` = 15 (3 PM) |
| 5     | Minutes | uint8 (0-59) | `0x1E` = 30 |
| 6     | Seconds | uint8 (0-59) | `0x2D` = 45 |
| 7     | Day of week | uint8 (1=Mon, 7=Sun) | `0x06` = Saturday |
| 8     | Fractions | uint8 (1/256 second) | `0x00` = 0 |
| 9     | Adjust reason | uint8 (flags) | `0x00` = Manual |

Example: `2026-01-11 Sat 15:30:45` = `EA 07 01 0B 0F 1E 2D 06 00 00`

### Troubleshooting

#### Permission Denied

```
Error: [org.bluez.Error.NotPermitted] Operation not permitted
```

**Solution:**
```bash
# Add user to bluetooth group
sudo usermod -a -G bluetooth $USER

# Log out and back in
```

#### Device Not Found

```
Device 'ESP32-Clock' not found
```

**Solutions:**
1. Check ESP32-C3 is powered on and advertising
2. Increase scan timeout: `./ble_time_sync.py --timeout 10`
3. Scan manually to see all devices: `./ble_time_sync.py --scan`
4. Verify device name matches (case-sensitive)

#### Connection Failed

```
Error: Failed to connect to peripheral
```

**Solutions:**
1. Make sure ESP32-C3 is not already connected to another device
2. Reset ESP32-C3
3. Restart Bluetooth service: `sudo systemctl restart bluetooth`
4. Move closer to the device

#### Import Error: bleak not found

```
Error: bleak library not found
```

**Solution:**
```bash
pip install bleak
```

### Integration with Other Tools

#### Run from systemd timer

Create a systemd timer to sync time every hour:

```bash
# /etc/systemd/system/esp32-time-sync.service
[Unit]
Description=ESP32 BLE Time Sync
After=bluetooth.service

[Service]
Type=oneshot
User=beattie
ExecStart=/path/to/ble_time_sync.py
```

```bash
# /etc/systemd/system/esp32-time-sync.timer
[Unit]
Description=ESP32 BLE Time Sync Timer

[Timer]
OnCalendar=hourly
Persistent=true

[Install]
WantedBy=timers.target
```

Enable:
```bash
sudo systemctl daemon-reload
sudo systemctl enable --now esp32-time-sync.timer
```

#### Run from cron

```bash
# Sync every hour
0 * * * * /path/to/ble_time_sync.py
```

## Development

### Testing

```bash
# Test scan
python3 ble_time_sync.py --scan

# Test single sync with verbose output
python3 ble_time_sync.py

# Test continuous mode with short interval
python3 ble_time_sync.py --continuous --interval 10
```

### Adding Features

The `BLETimeSync` class is designed to be extensible:

```python
from ble_time_sync import BLETimeSync

# Custom usage
async def custom_sync():
    syncer = BLETimeSync(device_name="MyDevice")
    await syncer.send_time()

asyncio.run(custom_sync())
```

## Related Documentation

- [BLE Time Display Implementation Guide](../docs/BLE_Time_Display.md)
- [Bluetooth Current Time Service Specification](https://www.bluetooth.com/specifications/specs/cts-1-1/)

## License

MIT OR Apache-2.0
