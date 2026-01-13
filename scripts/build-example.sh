#!/bin/bash
set -e

EXAMPLE_NAME="$1"

if [ -z "$EXAMPLE_NAME" ]; then
    echo "Usage: $0 <example_name> [additional_args...]"
    echo "Available examples:"
    echo "  blinky              - Blink the user LED"
    echo "  ws2812b_rainbow     - WS2812B LED rainbow effect"
    echo "  oled_display        - OLED display graphics"
    echo "  oled_display_more   - OLED display with more shapes"
    echo "  ble_scanner         - BLE device scanner (requires ble feature)"
    echo "  ble_bas_peripheral  - BLE Battery Service peripheral (requires ble feature)"
    echo "  ble_clock           - BLE time display clock (requires ble feature)"
    echo "  ble_gatt_simple     - Simple BLE GATT server (requires ble feature)"
    echo "  embassy_hello_world - Embassy RTOS hello world (requires embassy feature)"
    exit 1
fi

# Shift to get remaining arguments for espflash
shift

echo "Loading ESP environment..."
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
if [ -f "$SCRIPT_DIR/export-esp.sh" ]; then
    . "$SCRIPT_DIR/export-esp.sh"
fi

# Check if --features are needed
case "$EXAMPLE_NAME" in
    ble_scanner|ble_bas_peripheral|ble_clock|ble_gatt_simple)
        echo "Building $EXAMPLE_NAME with BLE features..."
        FEATURES="--features ble"
        ;;
    embassy_hello_world)
        echo "Building $EXAMPLE_NAME with Embassy features..."
        FEATURES="--features embassy"
        ;;
    *)
        echo "Building $EXAMPLE_NAME..."
        FEATURES=""
        ;;
esac

# Build the example
cargo build --release --example "$EXAMPLE_NAME" $FEATURES

if [ $? -eq 0 ]; then
    echo "Build successful! Flashing to ESP32-C3..."
    espflash flash "target/riscv32imc-unknown-none-elf/release/examples/$EXAMPLE_NAME" --monitor "$@"
else
    echo "Build failed!"
    exit 1
fi
