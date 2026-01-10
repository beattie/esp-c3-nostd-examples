# VS Code Setup Instructions for ESP32-C3 No-STD Examples

This guide explains how to use the VS Code tasks and launch configurations that have been set up for the `esp-c3-nostd-examples` project.

## 1. Using Build and Flash Tasks

Tasks allow you to run specific commands (like `cargo build` or `espflash`) directly from within VS Code.

### How to Run a Task:

1.  **Open the Command Palette:**
    *   Press `Ctrl+Shift+P` (Windows/Linux)
    *   Press `Cmd+Shift+P` (macOS)
    *   (Alternatively, go to `View > Command Palette...` in the top menu bar).
2.  **Type "Tasks: Run Task"** into the Command Palette search bar and select it.
3.  A list of available tasks will appear.

### Available Build/Flash Tasks:

*   **Build Active Example**:
    *   **Purpose:** Builds the Rust example corresponding to the `.rs` file you currently have open in the editor.
    *   **How to use:**
        1.  Open the `.rs` file of the example you want to build (e.g., `examples/blinky.rs`).
        2.  Run the "Build Active Example" task.
    *   *This is the recommended way to build when you are actively editing an example.*

*   **Flash Active Example**:
    *   **Purpose:** Flashes the compiled binary of the Rust example corresponding to the `.rs` file you currently have open onto your ESP32-C3 board.
    *   **How to use:**
        1.  Open the `.rs` file of the example you want to flash.
        2.  Run the "Flash Active Example" task.
    *   *Ensure you have successfully built the example before flashing.*

*   **Build Example (Prompt for Name)**:
    *   **Purpose:** Builds any Rust example by prompting you to enter its name.
    *   **How to use:**
        1.  Run the "Build Example (Prompt for Name)" task.
        2.  A small input box will appear. Type the exact name of the example (e.g., `blinky`, `ble_scanner`, `embassy_hello_world`) and press Enter.
    *   *Useful when you don't have the example's `.rs` file open.*

*   **Flash Example (Prompt for Name)**:
    *   **Purpose:** Flashes any compiled Rust example onto your ESP32-C3 board by prompting you to enter its name.
    *   **How to use:**
        1.  Run the "Flash Example (Prompt for Name)" task.
        2.  Type the exact name of the example and press Enter.
    *   *Ensure the example has been built successfully before flashing.*

*   **Build blinky / Flash blinky / Build ble_scanner / Flash ble_scanner (etc.)**:
    *   **Purpose:** These are specific tasks for each example.
    *   **How to use:** Simply select the desired task from the list.

## 2. Debugging with `probe-run`

A launch configuration has been set up to allow debugging your Rust examples on the ESP32-C3 using `probe-run`.

### How to Start a Debugging Session:

1.  **Open the "Run and Debug" View:** Click the Run icon in the Activity Bar on the side of VS Code (`Ctrl+Shift+D` or `Cmd+Shift+D`).
2.  **Select "Debug Example"** from the dropdown menu at the top of the "Run and Debug" view.
3.  **Press the green play button (or F5).**
4.  A small input box will appear. Type the exact name of the example you wish to debug (e.g., `blinky`, `embassy_hello_world`) and press Enter.
5.  VS Code will build (if necessary), flash, and then start a debug session, allowing you to set breakpoints, step through code, and inspect variables.

### Important Notes for Debugging:

*   **`probe-run` Installation:** You must have `probe-run` installed globally:
    ```bash
    cargo install probe-run
    ```
*   **Debug Probe:** Ensure you have a compatible debug probe connected to your ESP32-C3. The `launch.json` is currently configured for a **J-Link** probe (`"probe": "J-Link"`). If you are using a different probe (e.g., ESP-PROG, FTDI, Black Magic Probe), you will need to edit the `esp-c3-nostd-examples/.vscode/launch.json` file and change the `"probe"` setting to match your hardware.
*   **Serial Port Permissions:** You might need appropriate permissions to access the serial port of your debug probe/ESP32-C3. On Linux, this often involves adding your user to the `dialout` group:
    ```bash
    sudo usermod -a -G dialout $USER
    ```
    *After running this command, you will need to log out and log back in for the changes to take effect.*

This Markdown file should provide a clear reference for setting up and using VS Code for your embedded Rust development with the ESP32-C3.
