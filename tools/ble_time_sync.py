#!/usr/bin/env python3
"""
BLE Time Sync Tool for ESP32-C3 Time Display

This tool connects to an ESP32-C3 BLE peripheral and sends the current system
time in the Bluetooth Current Time Service format.

Requirements:
    pip install bleak

Usage:
    # Send time once
    ./ble_time_sync.py

    # Send time once to specific device
    ./ble_time_sync.py --device "ESP32-Clock"

    # Continuous sync every 60 seconds
    ./ble_time_sync.py --continuous --interval 60

    # Scan for devices only
    ./ble_time_sync.py --scan

Author: Brian Beattie
License: MIT OR Apache-2.0
"""

import asyncio
import argparse
import struct
import sys
from datetime import datetime
from typing import Optional

try:
    from bleak import BleakClient, BleakScanner
    from bleak.backends.device import BLEDevice
except ImportError:
    print("Error: bleak library not found")
    print("Install with: pip install bleak")
    sys.exit(1)


# Default UUIDs - customize these to match your GATT server
DEFAULT_DEVICE_NAME = "ESP32-Clock"
DEFAULT_SERVICE_UUID = "12345678-1234-5678-1234-56789abcdef0"
DEFAULT_TIME_CHAR_UUID = "12345678-1234-5678-1234-56789abcdef1"

# Official Bluetooth CTS UUIDs (alternative)
CTS_SERVICE_UUID = "00001805-0000-1000-8000-00805f9b34fb"
CTS_CHAR_UUID = "00002a2b-0000-1000-8000-00805f9b34fb"


class BLETimeSync:
    """BLE Time Synchronization Client"""

    def __init__(
        self,
        device_name: str = DEFAULT_DEVICE_NAME,
        service_uuid: str = DEFAULT_SERVICE_UUID,
        char_uuid: str = DEFAULT_TIME_CHAR_UUID,
    ):
        self.device_name = device_name
        self.service_uuid = service_uuid
        self.char_uuid = char_uuid
        self.device: Optional[BLEDevice] = None

    async def scan_devices(self, timeout: float = 5.0) -> list[BLEDevice]:
        """Scan for BLE devices"""
        print(f"Scanning for BLE devices ({timeout}s)...")
        devices = await BleakScanner.discover(timeout=timeout)
        return devices

    async def find_device(self, timeout: float = 5.0) -> Optional[BLEDevice]:
        """Find the target device by name"""
        print(f"Searching for device: {self.device_name}")
        devices = await self.scan_devices(timeout)

        for device in devices:
            if device.name and self.device_name in device.name:
                print(f"Found device: {device.name} [{device.address}]")
                self.device = device
                return device

        print(f"Device '{self.device_name}' not found")
        return None

    def pack_current_time(self) -> bytes:
        """
        Pack current system time into BLE Current Time format (10 bytes)

        Format:
            - Bytes 0-1: Year (uint16 little-endian)
            - Byte 2: Month (1-12)
            - Byte 3: Day (1-31)
            - Byte 4: Hours (0-23)
            - Byte 5: Minutes (0-59)
            - Byte 6: Seconds (0-59)
            - Byte 7: Day of week (1=Monday, 7=Sunday)
            - Byte 8: Fractions of second (1/256)
            - Byte 9: Adjust reason (0=none)
        """
        now = datetime.now()

        # Pack into 10 bytes using struct
        time_data = struct.pack(
            "<HBBBBBBBB",
            now.year,  # uint16 little-endian
            now.month,  # uint8
            now.day,  # uint8
            now.hour,  # uint8
            now.minute,  # uint8
            now.second,  # uint8
            (now.weekday() + 1) % 7 + 1,  # uint8 (1=Monday, convert from Python's 0=Monday)
            0,  # fractions256 (not implemented)
            0,  # adjust_reason (manual update)
        )

        return time_data

    def format_time_string(self) -> str:
        """Format current time as string"""
        now = datetime.now()
        weekdays = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"]
        weekday = weekdays[now.weekday()]
        return f"{now.year}-{now.month:02d}-{now.day:02d} {weekday} {now.hour:02d}:{now.minute:02d}:{now.second:02d}"

    def format_time_bytes(self, data: bytes) -> str:
        """Format time bytes as hex string"""
        return " ".join(f"{b:02x}" for b in data)

    async def send_time(self) -> bool:
        """Send current time to the device"""
        if not self.device:
            if not await self.find_device():
                return False

        try:
            async with BleakClient(self.device) as client:
                print(f"Connected to {self.device.name}")

                # Pack current time
                time_data = self.pack_current_time()
                time_str = self.format_time_string()
                time_hex = self.format_time_bytes(time_data)

                print(f"Sending time: {time_str}")
                print(f"  Hex data: {time_hex}")

                # Write to characteristic
                await client.write_gatt_char(self.char_uuid, time_data)
                print("✓ Time sent successfully")

                return True

        except Exception as e:
            print(f"✗ Error: {e}")
            return False

    async def continuous_sync(self, interval: int = 60):
        """Continuously sync time at specified interval (seconds)"""
        print(f"Starting continuous sync (interval: {interval}s)")
        print("Press Ctrl+C to stop")

        try:
            while True:
                success = await self.send_time()
                if success:
                    print(f"Waiting {interval}s until next sync...")
                else:
                    print(f"Failed to sync. Retrying in {interval}s...")

                await asyncio.sleep(interval)

        except KeyboardInterrupt:
            print("\nStopped by user")


async def scan_command(args):
    """Scan for BLE devices"""
    syncer = BLETimeSync()
    devices = await syncer.scan_devices(timeout=args.timeout)

    if not devices:
        print("No devices found")
        return

    print(f"\nFound {len(devices)} device(s):")
    print("-" * 70)
    for device in devices:
        name = device.name or "(Unknown)"
        print(f"  {name:30s} [{device.address}]")
    print("-" * 70)


async def sync_command(args):
    """Send time once or continuously"""
    syncer = BLETimeSync(
        device_name=args.device,
        service_uuid=args.service or DEFAULT_SERVICE_UUID,
        char_uuid=args.characteristic or DEFAULT_TIME_CHAR_UUID,
    )

    if args.continuous:
        await syncer.continuous_sync(interval=args.interval)
    else:
        success = await syncer.send_time()
        sys.exit(0 if success else 1)


def main():
    parser = argparse.ArgumentParser(
        description="BLE Time Sync Tool for ESP32-C3",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Examples:
  # Scan for devices
  %(prog)s --scan

  # Send time once
  %(prog)s

  # Send time to specific device
  %(prog)s --device "MyESP32"

  # Continuous sync every 30 seconds
  %(prog)s --continuous --interval 30

  # Use official CTS UUIDs
  %(prog)s --service 00001805-0000-1000-8000-00805f9b34fb \\
           --characteristic 00002a2b-0000-1000-8000-00805f9b34fb
        """,
    )

    parser.add_argument(
        "--scan",
        action="store_true",
        help="Scan for BLE devices and exit",
    )

    parser.add_argument(
        "--device",
        "-d",
        default=DEFAULT_DEVICE_NAME,
        help=f"Device name to connect to (default: {DEFAULT_DEVICE_NAME})",
    )

    parser.add_argument(
        "--service",
        "-s",
        help=f"Service UUID (default: {DEFAULT_SERVICE_UUID})",
    )

    parser.add_argument(
        "--characteristic",
        "-c",
        help=f"Time characteristic UUID (default: {DEFAULT_TIME_CHAR_UUID})",
    )

    parser.add_argument(
        "--continuous",
        action="store_true",
        help="Continuously sync time at intervals",
    )

    parser.add_argument(
        "--interval",
        "-i",
        type=int,
        default=60,
        help="Sync interval in seconds for continuous mode (default: 60)",
    )

    parser.add_argument(
        "--timeout",
        "-t",
        type=float,
        default=5.0,
        help="Scan timeout in seconds (default: 5.0)",
    )

    args = parser.parse_args()

    # Run async command
    try:
        if args.scan:
            asyncio.run(scan_command(args))
        else:
            asyncio.run(sync_command(args))
    except KeyboardInterrupt:
        print("\nInterrupted by user")
        sys.exit(1)


if __name__ == "__main__":
    main()
