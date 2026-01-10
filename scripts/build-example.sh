#!/bin/bash
set -e

EXAMPLE_NAME="$1"

if [ -z "$EXAMPLE_NAME" ]; then
    echo "Usage: $0 <example_name> [additional_args...]"
    echo "Available examples:"
    echo "  ws2812b_rainbow - WS2812B LED rainbow effect"
    echo "  ble_scanner     - BLE device scanner"
    echo "  embassy_hello_world - Embassy RTOS hello world example"
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
if [ "$EXAMPLE_NAME" = "ble_scanner" ]; then
    echo "Building $EXAMPLE_NAME with BLE features..."
    FEATURES="--features ble"
elif [ "$EXAMPLE_NAME" = "embassy_hello_world" ]; then
    echo "Building $EXAMPLE_NAME with Embassy features..."
    FEATURES="--features embassy"
else
    echo "Building $EXAMPLE_NAME..."
    FEATURES=""
fi

# Build the example
cargo build --release --example "$EXAMPLE_NAME" $FEATURES

if [ $? -eq 0 ]; then
    echo "Build successful! Flashing to ESP32-C3..."
    espflash flash "target/riscv32imc-unknown-none-elf/release/examples/$EXAMPLE_NAME" --monitor "$@"
else
    echo "Build failed!"
    exit 1
fi
