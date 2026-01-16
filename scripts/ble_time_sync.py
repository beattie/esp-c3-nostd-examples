#!/usr/bin/env python3
"""
BLE Time Synchronization Tool for ESP32-C3 Clock

This script connects to the ESP32-C3 BLE clock and can:
- Read the current time from the device
- Set the device time to the current system time
- Set the device time to a specific Unix timestamp
- Set the timezone offset (quarter hours from UTC)

Timezone is specified in quarter-hour increments (15 minutes) to support
timezones like India (UTC+5:30) and Nepal (UTC+5:45).

Requirements:
    pip install bleak

Usage:
    # Read current time from device
    python3 scripts/ble_time_sync.py --read

    # Set device time with timezone (whole hours)
    python3 scripts/ble_time_sync.py --set --timezone -8.0    # PST (UTC-8:00)
    python3 scripts/ble_time_sync.py --set --timezone 5.5     # IST (UTC+5:30)
    python3 scripts/ble_time_sync.py --set --timezone 5.75    # NPT (UTC+5:45)

    # Set device time to specific Unix timestamp
    python3 scripts/ble_time_sync.py --set --timestamp 1736899200 --timezone -8.0
"""

import asyncio
import sys
import time
import argparse
from datetime import datetime, timezone
from bleak import BleakClient, BleakScanner
from bleak.exc import BleakError

# Service and Characteristic UUIDs
SERVICE_UUID = "f0debc9a-7856-3412-7856-341278563412"
TIME_CHAR_UUID = "f3debc9a-7856-3412-7856-341278563412"
TIMEZONE_CHAR_UUID = "f4debc9a-7856-3412-7856-341278563412"  # New: timezone offset

DEVICE_NAME = "ESP32-GATT"


def format_time_utc(unix_time):
    """Format Unix timestamp as UTC time."""
    dt = datetime.fromtimestamp(unix_time, tz=timezone.utc)
    return dt.strftime("%Y-%m-%d %H:%M:%S")

def format_time_local(unix_time):
    """Format Unix timestamp as system local time."""
    dt = datetime.fromtimestamp(unix_time)
    return dt.strftime("%Y-%m-%d %H:%M:%S")


def hours_to_quarter_hours(hours_float):
    """Convert hours (float) to quarter hours (i8).

    Args:
        hours_float: Timezone offset in hours (e.g., -8.0, 5.5, 5.75)

    Returns:
        Quarter hours as integer (e.g., -32, 22, 23)
    """
    return int(hours_float * 4)


def quarter_hours_to_hours(quarter_hours):
    """Convert quarter hours (i8) to hours (float).

    Args:
        quarter_hours: Timezone offset in quarter hours

    Returns:
        Hours as float (e.g., -8.0, 5.5, 5.75)
    """
    return quarter_hours / 4.0


def format_timezone(quarter_hours):
    """Format timezone offset as UTC±HH:MM string.

    Args:
        quarter_hours: Timezone offset in quarter hours

    Returns:
        Formatted string like "UTC-8:00", "UTC+5:30", "UTC+5:45"
    """
    hours = abs(quarter_hours) // 4
    minutes = (abs(quarter_hours) % 4) * 15
    sign = '+' if quarter_hours >= 0 else '-'
    return f"UTC{sign}{hours}:{minutes:02d}"


async def find_device():
    """Scan for the ESP32-GATT device."""
    print(f"Scanning for '{DEVICE_NAME}'...")

    for attempt in range(3):
        try:
            devices = await BleakScanner.discover(timeout=10.0)

            for device in devices:
                if device.name == DEVICE_NAME:
                    print(f"  Found device: {device.name} ({device.address})")
                    return device.address

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


async def read_time(address):
    """Read the current time and timezone from the ESP32 device."""
    print(f"\nConnecting to {address}...")

    try:
        async with BleakClient(address) as client:
            print(f"  Connected: {client.is_connected}")

            # Read the time characteristic
            print(f"\nReading time characteristic...")
            data = await client.read_gatt_char(TIME_CHAR_UUID)

            if len(data) >= 4:
                unix_time = int.from_bytes(data[0:4], byteorder='little')

                if unix_time == 0:
                    print("  ⚠ Time not set on device (reads as 0)")
                    print("  Run with --set to synchronize the clock")
                else:
                    print(f"  Device time (Unix): {unix_time}")
                    print(f"  Device time (UTC):  {format_time_utc(unix_time)}")

                    # Try to read timezone
                    try:
                        tz_data = await client.read_gatt_char(TIMEZONE_CHAR_UUID)
                        if len(tz_data) >= 1:
                            quarter_hours = int.from_bytes(tz_data, byteorder='little', signed=True)
                            print(f"  Timezone offset:     {format_timezone(quarter_hours)}")

                            # Calculate and show local time (quarter_hours * 15 minutes * 60 seconds)
                            local_time = unix_time + (quarter_hours * 15 * 60)
                            print(f"  Device time (local): {format_time_utc(local_time)}")
                    except Exception:
                        print("  Timezone: Not available (characteristic not found)")

                    # Show offset from system time
                    system_time = int(time.time())
                    offset = unix_time - system_time
                    if abs(offset) > 2:
                        print(f"  ⚠ Device is {abs(offset)} seconds {'ahead' if offset > 0 else 'behind'} system time")
                    else:
                        print(f"  ✓ Device time is synchronized (within {abs(offset)}s)")

                return 0
            else:
                print(f"  ERROR: Invalid data length: {len(data)} bytes")
                return 1

    except BleakError as e:
        print(f"ERROR: {e}")
        return 1


async def write_time(address, timestamp=None, timezone=None):
    """Write the current time and timezone to the ESP32 device."""
    print(f"\nConnecting to {address}...")

    try:
        async with BleakClient(address) as client:
            print(f"  Connected: {client.is_connected}")

            # Get timestamp (use provided or current time)
            if timestamp is not None:
                unix_time = timestamp
                print(f"\nSetting device time to: {timestamp} ({format_time_utc(timestamp)})")
            else:
                unix_time = int(time.time())
                print(f"\nSetting device time to current system time:")
                print(f"  Unix timestamp: {unix_time}")
                print(f"  UTC time:       {format_time_utc(unix_time)}")

            # Convert to 4-byte little-endian u32
            time_bytes = unix_time.to_bytes(4, byteorder='little')

            # Write to device
            await client.write_gatt_char(TIME_CHAR_UUID, time_bytes)
            print(f"  ✓ Time written successfully")

            # Write timezone if provided
            if timezone is not None:
                quarter_hours = hours_to_quarter_hours(timezone)
                print(f"\nSetting timezone offset to {format_timezone(quarter_hours)}...")
                try:
                    # Convert timezone to signed i8 (quarter hours)
                    tz_bytes = quarter_hours.to_bytes(1, byteorder='little', signed=True)
                    await client.write_gatt_char(TIMEZONE_CHAR_UUID, tz_bytes)
                    print(f"  ✓ Timezone written successfully")

                    # Calculate and show local time (quarter_hours * 15 minutes * 60 seconds)
                    local_time = unix_time + (quarter_hours * 15 * 60)
                    print(f"  Local time will be: {format_time_utc(local_time)}")
                except Exception as e:
                    print(f"  ⚠ Timezone characteristic not available: {e}")
                    print(f"  Device may not support timezone (update ble_clock.rs)")

            # Wait a moment, then read back to verify
            await asyncio.sleep(0.5)
            print(f"\nVerifying...")
            data = await client.read_gatt_char(TIME_CHAR_UUID)

            if len(data) >= 4:
                readback_time = int.from_bytes(data[0:4], byteorder='little')
                offset = readback_time - unix_time

                if abs(offset) <= 1:  # Allow 1 second difference due to latency
                    print(f"  ✓ Verification successful (offset: {offset}s)")
                else:
                    print(f"  ⚠ Time may not have set correctly (offset: {offset}s)")
                    return 1

            return 0

    except BleakError as e:
        print(f"ERROR: {e}")
        return 1


async def main():
    """Main function."""
    parser = argparse.ArgumentParser(
        description="BLE Time Synchronization Tool for ESP32-C3 Clock",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Examples:
  Read current time from device:
    python3 scripts/ble_time_sync.py --read

  Set device time to current system time with timezone:
    python3 scripts/ble_time_sync.py --set --timezone -8.0   # PST (UTC-8:00)
    python3 scripts/ble_time_sync.py --set --timezone 5.5    # IST (UTC+5:30)
    python3 scripts/ble_time_sync.py --set --timezone 5.75   # NPT (UTC+5:45)
    python3 scripts/ble_time_sync.py --set --timezone -3.5   # NST (UTC-3:30)

  Set device time to specific timestamp:
    python3 scripts/ble_time_sync.py --set --timestamp 1736899200 --timezone -8.0
        """
    )

    group = parser.add_mutually_exclusive_group(required=True)
    group.add_argument('--read', action='store_true', help='Read time from device')
    group.add_argument('--set', action='store_true', help='Set device time')

    parser.add_argument('--timestamp', type=int, help='Unix timestamp to set (default: current time)')
    parser.add_argument('--timezone', type=float, help='Timezone offset from UTC in hours (e.g., -8.0 for PST, 5.5 for IST, 5.75 for NPT)')
    parser.add_argument('--device', type=str, default=DEVICE_NAME, help=f'Device name to connect to (default: {DEVICE_NAME})')

    args = parser.parse_args()

    # Validate arguments
    if args.timestamp is not None and not args.set:
        print("ERROR: --timestamp can only be used with --set")
        return 1

    if args.timezone is not None:
        # Validate range: UTC-12:00 to UTC+14:00
        if args.timezone < -12.0 or args.timezone > 14.0:
            print("ERROR: --timezone must be between -12.0 and +14.0")
            return 1

        # Validate quarter-hour increments (0.00, 0.25, 0.50, 0.75)
        quarter_hours = hours_to_quarter_hours(args.timezone)
        if abs(quarter_hours * 0.25 - args.timezone) > 0.01:
            print(f"ERROR: --timezone must be in quarter-hour increments (0.25, 0.5, 0.75)")
            print(f"       Examples: -8.0, -8.25, 5.5, 5.75")
            return 1

    # Find the device
    address = await find_device()
    if address is None:
        print(f"\nERROR: Could not find device '{DEVICE_NAME}'")
        print("Make sure the ESP32-C3 is running and advertising.")
        return 1

    # Perform the requested operation
    if args.read:
        return await read_time(address)
    elif args.set:
        return await write_time(address, args.timestamp, args.timezone)

    return 1


if __name__ == "__main__":
    try:
        exit_code = asyncio.run(main())
        sys.exit(exit_code)
    except KeyboardInterrupt:
        print("\n\nInterrupted by user")
        sys.exit(0)
