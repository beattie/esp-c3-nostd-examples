# VS Code Configuration for ESP32-C3 Rust Development

This document explains the VS Code configuration for building and flashing ESP32-C3 Rust examples.

## Overview

The VS Code setup uses **input prompts** instead of fixed menu items, making it easy to work with any example without modifying configuration files. When you add a new example, it automatically works with the existing tasks.

## Files

### `.vscode/tasks.json`
Defines build and flash tasks with dynamic input prompts.

**Tasks:**
1. **Build Example** - Builds a release binary for any example
2. **Build + Flash + Monitor** (default) - Builds, flashes to ESP32-C3, and opens serial monitor

**Key Features:**
- Prompts for example name and features instead of fixed menu items
- No configuration changes needed when adding new examples
- Handles feature flags (`--features ble`, `--features embassy`)
- Chains build and flash with `&&` to ensure flash only runs if build succeeds

### `.vscode/settings.json`
Configures rust-analyzer for no_std ESP32-C3 development.

**Settings:**
- `rust-analyzer.cargo.target`: `riscv32imc-unknown-none-elf`
- `rust-analyzer.check.command`: `clippy` for better linting
- `rust-analyzer.check.allTargets`: `false` to avoid test crate errors
- File watcher excludes `target/` directory for better performance

## How to Use

### Build + Flash + Monitor (Most Common Workflow)

This is the default build task and the fastest way to get your code running on hardware.

1. Press **`Ctrl+Shift+B`** (or Run Build Task from Command Palette)
2. When prompted, enter the example name:
   - `blinky`
   - `ws2812b_rainbow`
   - `oled_display`
   - `ble_scanner`
   - `embassy_hello_world`
3. Select features if needed:
   - `` (blank) - No features (for blinky, ws2812b_rainbow, oled_display)
   - `--features ble` - For BLE examples (ble_scanner)
   - `--features embassy` - For Embassy examples (embassy_hello_world)
4. The task will:
   - Build the example in release mode
   - Flash it to your ESP32-C3
   - Open the serial monitor automatically

### Build Only

If you want to build without flashing:

1. Open Command Palette: **`Ctrl+Shift+P`**
2. Type and select: **Tasks: Run Task**
3. Select **Build Example**
4. Enter example name and select features

### Stopping the Serial Monitor

After flashing, the serial monitor will be running in the terminal:

- Press **`Ctrl+C`** to stop the monitor
- Press **`Ctrl+`** (backtick) to hide the terminal panel
- Click the **trash can icon** to close the terminal completely

## Adding New Examples

**No configuration changes needed!** The tasks work with any example automatically.

1. Create your example file: `examples/my_new_example.rs`
2. Add it to `Cargo.toml`:
   ```toml
   [[example]]
   name = "my_new_example"
   path = "examples/my_new_example.rs"
   # Add required-features if needed
   ```
3. Run the build task (**`Ctrl+Shift+B`**)
4. Type `my_new_example` when prompted
5. Select features if needed

The task system uses string input prompts, so any example name works immediately.

## Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| `Ctrl+Shift+B` | Build + Flash + Monitor (default) |
| `Ctrl+Shift+P` | Open Command Palette (to run other tasks) |
| `Ctrl+C` | Stop serial monitor |
| `Ctrl+\`` (backtick) | Toggle terminal panel |
| `Ctrl+J` | Toggle bottom panel |

## Common Issues and Solutions

### Error: Failed to open serial port

**Cause:** Another process is using the serial port (usually a previous monitor session).

**Symptoms:**
```
Error: × Failed to open serial port /dev/ttyACM0
       ╰─▶ Error while connecting to device
```

**Solution:**
```bash
# Find the process using the port
fuser /dev/ttyACM0

# Kill it (replace XXXX with the process ID shown)
kill XXXX
```

**Prevention:** Always press `Ctrl+C` to stop the monitor before running another flash task.

### Build succeeds but flash doesn't start

**Cause:** The `&&` operator in the task stops execution if the build fails.

**Solution:** Check the terminal output for build errors, fix them, and try again.

### Permission denied on serial port

**Cause:** User lacks permissions to access `/dev/ttyACM0`.

**Solution:**
```bash
# Add your user to the plugdev group
sudo usermod -a -G plugdev $USER

# Log out and log back in for changes to take effect
```

### rust-analyzer shows errors for examples with features

**Cause:** rust-analyzer doesn't know which features to enable.

**Solution:** This is expected. The examples will still build correctly when you run the build task with the appropriate features selected. To reduce noise, rust-analyzer is configured to check only the default feature set.

**Alternative:** If working extensively on a specific feature, temporarily modify `.vscode/settings.json`:
```json
{
    "rust-analyzer.cargo.features": ["ble"]
}
```

Remember to revert this when switching to work on different examples.

### Task prompts are annoying for repeated builds

**Tip:** VS Code remembers your last input. When you run the task again:
- The previous example name appears as the default
- Just press Enter to use it again

**Alternative:** For rapid iteration on a single example, use the terminal:
```bash
cargo build --release --example blinky && espflash flash --monitor target/riscv32imc-unknown-none-elf/release/examples/blinky
```

## Examples by Features

### No features required:
- `blinky` - Simple LED blink
- `ws2812b_rainbow` - WS2812B rainbow effect
- `oled_display` - OLED graphics display

### Requires `--features ble`:
- `ble_scanner` - BLE device scanner

### Requires `--features embassy`:
- `embassy_hello_world` - Embassy async example

## Tips and Tricks

### Quick Rebuild and Flash
After making code changes, just press `Ctrl+Shift+B` and then Enter (to accept the previous example name). This is the fastest way to iterate.

### Multiple Terminals
If you want to keep the serial monitor running while starting a new build:
1. Click the **split terminal** icon (two panels) in the terminal
2. The new build will use a fresh terminal
3. Note: You'll need to stop the old monitor in terminal 1 before the new flash succeeds

### Custom Port Selection
If your ESP32-C3 is on a different port, modify the flash command in `.vscode/tasks.json`:
```json
"command": "cargo build --release --example ${input:exampleName} ${input:features} && espflash flash --monitor --port /dev/ttyUSB0 target/riscv32imc-unknown-none-elf/release/examples/${input:exampleName}"
```

Or use the command line helper script:
```bash
./scripts/flash-example.sh blinky --port /dev/ttyUSB0
```

## Troubleshooting Workflow

1. **Build fails?**
   - Read the error messages in the terminal
   - Fix the code issues
   - Run `Ctrl+Shift+B` again

2. **Flash fails?**
   - Check if another process is using the port: `fuser /dev/ttyACM0`
   - Kill the blocking process: `kill <pid>`
   - Try again

3. **Monitor shows nothing?**
   - Check your serial baud rate (most examples use default ESP settings)
   - Verify the ESP32-C3 is running (LED should blink if using blinky)
   - Press the RESET button on the board
   - Try unplugging and replugging the USB cable

4. **rust-analyzer constantly complaining?**
   - This is normal for examples with features
   - The code will still build correctly
   - Ignore the squiggly lines if you're working on a feature-specific example

## Further Reading

- [Main README](../README.md) - Project overview and hardware setup
- [OLED Display Guide](oled_display.md) - Detailed OLED example documentation
- [ESP32-C3 Hardware Docs](ESP32_C3_DevKitM_1.md) - Hardware pinout and specifications
