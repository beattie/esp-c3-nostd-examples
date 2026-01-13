# Vendored Dependencies

This directory contains vendored (copied) dependencies for the ESP32-C3 BLE examples.

## Why Vendor?

These dependencies are vendored to make the project self-contained and avoid external path dependencies that may not be available on other machines.

## Vendored Crates

### trouble-host (816K)

**Original:** https://github.com/embassy-rs/trouble
**Path:** `vendor/trouble-host`
**Version:** 0.5.1
**License:** Apache-2.0 OR MIT

An async Rust BLE host implementation. This is the core BLE stack used for peripheral and central roles.

**Modifications:**
- Updated `readme` path in Cargo.toml to point to local README.md
- Updated `trouble-host-macros` dependency path to `../trouble-host-macros`

### trouble-host-macros (72K)

**Original:** https://github.com/embassy-rs/trouble
**Path:** `vendor/trouble-host-macros`
**Version:** 0.3.0
**License:** Apache-2.0 OR MIT

Procedural macros for trouble-host, including:
- `#[gatt_server]` - Define GATT servers
- `#[gatt_service]` - Define GATT services
- `#[characteristic]` - Define GATT characteristics

**Modifications:** None

### trouble-example-apps (168K)

**Original:** https://github.com/embassy-rs/trouble
**Path:** `vendor/trouble-example-apps`
**Version:** 0.1.0
**License:** MIT OR Apache-2.0

Shared BLE application code used by multiple examples, including:
- `ble_bas_peripheral` - Battery service peripheral
- `ble_bas_central` - Battery service central
- Various security and bonding examples

**Modifications:**
- Updated `trouble-host` dependency path to `../trouble-host`

## Updating Vendored Dependencies

To update to a newer version:

```bash
# 1. Update the source repo
cd ~/projects/trouble
git pull

# 2. Remove old vendored copies
rm -rf vendor/trouble-*

# 3. Copy new versions
cp -r ~/projects/trouble/host vendor/trouble-host
cp -r ~/projects/trouble/host-macros vendor/trouble-host-macros
cp -r ~/projects/trouble/examples/apps vendor/trouble-example-apps

# 4. Apply modifications
# - Fix trouble-host/Cargo.toml readme and path to macros
# - Fix trouble-example-apps/Cargo.toml path to trouble-host
# - Add README.md to trouble-host

# 5. Test build
cargo build --release --example ble_bas_peripheral --features ble
```

## License

All vendored crates retain their original licenses:
- **Apache-2.0 OR MIT** for trouble-host and trouble-host-macros
- **MIT OR Apache-2.0** for trouble-example-apps

See individual crate directories for full license text.

## Original Repository

All vendored code originates from:
**https://github.com/embassy-rs/trouble**

Git commit at time of vendoring: Check the original repo for latest version.
