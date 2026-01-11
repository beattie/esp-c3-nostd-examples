#!/bin/bash
set -e

EXAMPLE_NAME="$1"

if [ -z "$EXAMPLE_NAME" ]; then
    echo "Usage: $0 <example_name> [additional_args...]"
    echo "Available examples:"
    echo "  ws2812b_rainbow - WS2812B LED rainbow effect"
    echo "  ble_scanner     - BLE device scanner"
    echo "  embassy_hello_world - Embassy RTOS hello world example"
	echo "  oled_display"
	echo "  blinky         - Blink the user LED"
    exit 1
fi

# Shift to get remaining arguments for espflash
shift

echo "Loading ESP environment..."
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
if [ -f "$SCRIPT_DIR/export-esp.sh" ]; then
    . "$SCRIPT_DIR/export-esp.sh"
fi

echo "Building $EXAMPLE_NAME..."

# Build the example
espflash flash "target/riscv32imc-unknown-none-elf/release/examples/$EXAMPLE_NAME" --monitor "$@"
