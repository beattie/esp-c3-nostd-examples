#!/usr/bin/env python3
"""
BLE GATT Client for ESP32-C3 Simple GATT Server

This script connects to the ESP32-C3 BLE GATT server and demonstrates:
- Reading characteristics
- Writing characteristics
- Receiving notifications

Requirements:
    pip install bleak

Usage:
    python3 scripts/ble_gatt_client.py
"""

import asyncio
import sys
import time
from bleak import BleakClient, BleakScanner
from bleak.exc import BleakError

# Service and Characteristic UUIDs (as reported by BLE stack)
# Note: BLE transmits UUIDs with specific byte ordering
SERVICE_UUID = "f0debc9a-7856-3412-7856-341278563412"
COUNTER_CHAR_UUID = "f1debc9a-7856-3412-7856-341278563412"
DATA_CHAR_UUID = "f2debc9a-7856-3412-7856-341278563412"
TIME_CHAR_UUID = "f3debc9a-7856-3412-7856-341278563412"

DEVICE_NAME = "ESP32-GATT"

# Global state for notification counting
counter_notification_count = 0
ble_client = None


def counter_notification_handler(sender, data):
    """Handle notifications from the counter characteristic."""
    global counter_notification_count, ble_client

    value = int.from_bytes(data, byteorder='little')
    counter_notification_count += 1
    print(f"  [COUNTER NOTIFICATION #{counter_notification_count}] Value: {value}")

    # After 3 notifications, write the current time
    if counter_notification_count == 3:
        print(f"\n  >>> Received 3 counter notifications, writing current time to ESP32...")
        asyncio.create_task(write_current_time())


def data_notification_handler(sender, data):
    """Handle notifications from the data characteristic."""
    value = int.from_bytes(data, byteorder='little')
    print(f"  [DATA NOTIFICATION] Value: {value}")


def time_notification_handler(sender, data):
    """Handle notifications from the time characteristic."""
    if len(data) >= 4:
        unix_time = int.from_bytes(data[0:4], byteorder='little')
        print(f"  [TIME NOTIFICATION] Unix timestamp: {unix_time}")
    else:
        print(f"  [TIME NOTIFICATION] Data: {data.hex()}")


async def write_current_time():
    """Write the current Unix timestamp to the ESP32."""
    global ble_client

    if ble_client is None:
        print("  ERROR: Client not connected")
        return

    # Get current Unix timestamp (as u32 - works until 2106)
    current_time = int(time.time())
    print(f"  Current Unix time: {current_time}")

    # Convert to 4-byte little-endian u32
    time_bytes = current_time.to_bytes(4, byteorder='little')

    try:
        await ble_client.write_gatt_char(TIME_CHAR_UUID, time_bytes)
        print(f"  ✓ Time written successfully: {current_time}")
    except Exception as e:
        print(f"  ✗ Failed to write time: {e}")


async def find_device():
    """Scan for the ESP32-GATT device."""
    print(f"Scanning for '{DEVICE_NAME}'...")

    # Try to scan, with retry if there's a scan already in progress
    for attempt in range(3):
        try:
            devices = await BleakScanner.discover(timeout=10.0)

            for device in devices:
                if device.name == DEVICE_NAME:
                    print(f"  Found device: {device.name} ({device.address})")
                    return device.address

            # If we get here, device wasn't found
            return None

        except BleakError as e:
            if "already in progress" in str(e).lower():
                if attempt < 2:
                    print(f"  Scan already in progress, retrying in 2 seconds... (attempt {attempt + 1}/3)")
                    await asyncio.sleep(2)
                else:
                    print("  ERROR: Bluetooth scanner is busy.")
                    print("  Try: sudo systemctl restart bluetooth")
                    raise
            else:
                raise

    return None


async def main():
    """Main function to interact with the BLE GATT server."""
    global ble_client

    # Find the device
    address = await find_device()
    if address is None:
        print(f"ERROR: Could not find device '{DEVICE_NAME}'")
        print("Make sure the ESP32-C3 is running and advertising.")
        return 1

    print(f"\nConnecting to {address}...")

    try:
        async with BleakClient(address) as client:
            ble_client = client  # Set global reference for notification handlers
            print(f"  Connected: {client.is_connected}")

            # List all services and characteristics
            print("\nAvailable services:")
            for service in client.services:
                print(f"  Service: {service.uuid}")
                for char in service.characteristics:
                    props = ', '.join(char.properties)
                    print(f"    Characteristic: {char.uuid} ({props})")

            # Enable notifications on all characteristics
            print(f"\nEnabling notifications...")
            await client.start_notify(COUNTER_CHAR_UUID, counter_notification_handler)
            await client.start_notify(DATA_CHAR_UUID, data_notification_handler)
            await client.start_notify(TIME_CHAR_UUID, time_notification_handler)
            print("  Notifications enabled")

            # Read the counter characteristic
            print(f"\n1. Reading counter characteristic...")
            data = await client.read_gatt_char(COUNTER_CHAR_UUID)
            counter_value = int.from_bytes(data, byteorder='little')
            print(f"  Counter: {counter_value}")

            # Read the data characteristic (should be 0 initially)
            print(f"\n2. Reading data characteristic (initial value)...")
            data = await client.read_gatt_char(DATA_CHAR_UUID)
            data_value = int.from_bytes(data, byteorder='little')
            print(f"  Data: {data_value}")

            # Write to the data characteristic
            print(f"\n3. Writing value 42 to data characteristic...")
            await client.write_gatt_char(DATA_CHAR_UUID, (42).to_bytes(1, byteorder='little'))
            print("  Write successful")
            await asyncio.sleep(0.5)  # Wait for notification

            # Read it back
            print(f"\n4. Reading data characteristic again...")
            data = await client.read_gatt_char(DATA_CHAR_UUID)
            data_value = int.from_bytes(data, byteorder='little')
            print(f"  Data: {data_value}")

            # Write more values to trigger notifications
            print(f"\n5. Writing several values to data characteristic...")
            for value in [100, 150, 200, 255]:
                print(f"  Writing {value}...")
                await client.write_gatt_char(DATA_CHAR_UUID, value.to_bytes(1, byteorder='little'))
                await asyncio.sleep(0.5)  # Wait for notification

            # Wait for counter notifications (counter updates every 5 seconds)
            print("\n6. Waiting 15 seconds for counter notifications...")
            print("   (Counter increments every 5 seconds)")
            await asyncio.sleep(15)

            # Stop notifications
            print("\nDisabling notifications...")
            await client.stop_notify(COUNTER_CHAR_UUID)
            await client.stop_notify(DATA_CHAR_UUID)
            await client.stop_notify(TIME_CHAR_UUID)

            print("\nDemo complete!")

    except BleakError as e:
        print(f"ERROR: {e}")
        return 1
    except Exception as e:
        print(f"Unexpected error: {e}")
        import traceback
        traceback.print_exc()
        return 1
    finally:
        ble_client = None  # Clear global reference

    return 0


if __name__ == "__main__":
    try:
        exit_code = asyncio.run(main())
        sys.exit(exit_code)
    except KeyboardInterrupt:
        print("\nInterrupted by user")
        sys.exit(0)
