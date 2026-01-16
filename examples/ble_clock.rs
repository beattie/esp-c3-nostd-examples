//! BLE Clock with OLED Display Example
//!
//! This example demonstrates a BLE GATT server with time synchronization and OLED display.
//! The clock receives time from a BLE client and displays it on an OLED screen.
//!
//! GATT Structure:
//! - Service UUID: 12345678-1234-5678-1234-56789abcdef0
//!   - Counter Characteristic (UUID: 12345678-1234-5678-1234-56789abcdef1)
//!     Properties: Read, Notify
//!     Value: u32 counter that increments every 5 seconds
//!   - Data Characteristic (UUID: 12345678-1234-5678-1234-56789abcdef2)
//!     Properties: Read, Write, Notify
//!     Value: u8 data that can be read/written
//!   - Time Characteristic (UUID: 12345678-1234-5678-1234-56789abcdef3)
//!     Properties: Read, Write, Notify
//!     Value: u32 Unix timestamp (seconds since 1970-01-01)
//!   - Timezone Characteristic (UUID: 12345678-1234-5678-1234-56789abcdef4)
//!     Properties: Read, Write
//!     Value: i8 timezone offset from UTC in quarter hours (-48 to +56)
//!            Supports 15-minute increments (e.g., 22 = UTC+5:30 for India)
//!
//! Hardware:
//! - Airtip ESP32-C3 OLED board (or similar with SSD1306 display)
//! - 0.42" OLED display (72x40 visible pixels) on I2C
//! - SDA: GPIO5
//! - SCL: GPIO6
//!
//! Run with:
//! ESP_LOG=info cargo run --release --example ble_clock --features ble
//!
//! Or use the build script:
//! ESP_LOG=info ./scripts/build-example.sh ble_clock
//!
//! Connect from Linux using the provided Python script: scripts/ble_gatt_client.py

#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_futures::join::join;
use embassy_futures::select::select3;
use embassy_time::Timer;
use embassy_time::Instant;
use esp_backtrace as _;
use esp_hal::clock::CpuClock;
use esp_hal::i2c::master::{I2c, Config};
use esp_hal::timer::timg::TimerGroup;
use esp_hal::Blocking;
use esp_radio::ble::controller::BleConnector;
use trouble_host::prelude::*;
use core::sync::atomic::Ordering;
use core::fmt::Write;
use embedded_graphics::{
    mono_font::{MonoTextStyle, ascii::FONT_6X10},
    pixelcolor::BinaryColor,
    prelude::*,
    primitives::{PrimitiveStyle, Rectangle},
    text::Text,
};
use ssd1306::{I2CDisplayInterface, Ssd1306};
use ssd1306::mode::DisplayConfig;
use ssd1306::size::DisplaySize72x40;
use ssd1306::rotation::DisplayRotation;

static UNIX_AT_BOOT: core::sync::atomic::AtomicU32 = core::sync::atomic::AtomicU32::new(0);
static TIMEZONE_OFFSET: core::sync::atomic::AtomicI8 = core::sync::atomic::AtomicI8::new(0);

esp_bootloader_esp_idf::esp_app_desc!();

const CONNECTIONS_MAX: usize = 1;
const L2CAP_CHANNELS_MAX: usize = 2;

// Custom service UUID: 12345678-1234-5678-1234-56789abcdef0
const CUSTOM_SERVICE_UUID: Uuid = Uuid::new_long([
    0x12, 0x34, 0x56, 0x78, 0x12, 0x34, 0x56, 0x78,
    0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc, 0xde, 0xf0,
]);

// Counter characteristic UUID: 12345678-1234-5678-1234-56789abcdef1
const COUNTER_CHAR_UUID: Uuid = Uuid::new_long([
    0x12, 0x34, 0x56, 0x78, 0x12, 0x34, 0x56, 0x78,
    0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc, 0xde, 0xf1,
]);

// Data characteristic UUID: 12345678-1234-5678-1234-56789abcdef2
const DATA_CHAR_UUID: Uuid = Uuid::new_long([
    0x12, 0x34, 0x56, 0x78, 0x12, 0x34, 0x56, 0x78,
    0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc, 0xde, 0xf2,
]);

// Time characteristic UUID: 12345678-1234-5678-1234-56789abcdef3
const TIME_CHAR_UUID: Uuid = Uuid::new_long([
    0x12, 0x34, 0x56, 0x78, 0x12, 0x34, 0x56, 0x78,
    0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc, 0xde, 0xf3,
]);

// Timezone characteristic UUID: 12345678-1234-5678-1234-56789abcdef4
const TIMEZONE_CHAR_UUID: Uuid = Uuid::new_long([
    0x12, 0x34, 0x56, 0x78, 0x12, 0x34, 0x56, 0x78,
    0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc, 0xde, 0xf4,
]);

// GATT Server definition using procedural macros
#[gatt_server]
struct Server {
    simple_service: SimpleService,
}

#[gatt_service(uuid = CUSTOM_SERVICE_UUID)]
struct SimpleService {
    /// Counter characteristic (read + notify)
    #[characteristic(uuid = COUNTER_CHAR_UUID, read, notify, value = 0u32)]
    counter: u32,

    /// Data characteristic (read + write + notify)
    #[characteristic(uuid = DATA_CHAR_UUID, read, write, notify, value = 0u8)]
    data: u8,

    /// Time characteristic (read + write + notify)
    /// Value: u32 Unix timestamp (seconds since 1970-01-01)
    #[characteristic(uuid = TIME_CHAR_UUID, read, write, notify, value = 0u32)]
    time: u32,

    /// Timezone characteristic (read + write)
    /// Value: i8 offset from UTC in quarter hours (-48 to +56)
    /// Quarter hours support 15-minute increments (e.g., India UTC+5:30 = 22)
    #[characteristic(uuid = TIMEZONE_CHAR_UUID, read, write, value = 0i8)]
    timezone: i8,
}

#[esp_rtos::main]
async fn main(spawner: Spawner) {
    esp_println::logger::init_logger_from_env();
    log::info!("=== BLE Clock with Display Starting ===");

    let peripherals = esp_hal::init(esp_hal::Config::default().with_cpu_clock(CpuClock::max()));
    esp_alloc::heap_allocator!(size: 72 * 1024);

    // Initialize I2C for OLED display
    // Airtip ESP32-C3 OLED: SDA=GPIO5, SCL=GPIO6
    let i2c = I2c::new(peripherals.I2C0, Config::default())
        .unwrap()
        .with_sda(peripherals.GPIO5)
        .with_scl(peripherals.GPIO6);
    log::info!("I2C initialized on GPIO5 (SDA) and GPIO6 (SCL)");

    let timg0 = TimerGroup::new(peripherals.TIMG0);
    #[cfg(target_arch = "riscv32")]
    let software_interrupt =
        esp_hal::interrupt::software::SoftwareInterruptControl::new(peripherals.SW_INTERRUPT);

    esp_rtos::start(
        timg0.timer0,
        #[cfg(target_arch = "riscv32")]
        software_interrupt.software_interrupt0,
    );

    // Spawn display task to run independently
    spawner.spawn(display_task(i2c)).expect("Failed to spawn display task");
    log::info!("Display task spawned");

    let bluetooth = peripherals.BT;
    let connector = BleConnector::new(bluetooth, Default::default()).unwrap();
    let controller: ExternalController<_, 20> = ExternalController::new(connector);

    run_ble_server(controller).await;
}

async fn run_ble_server<C>(controller: C)
where
    C: Controller,
{
    let address: Address = Address::random([0xff, 0x8f, 0x1a, 0x05, 0xe4, 0xff]);
    log::info!("BLE Address: {:?}", address);

    let mut resources: HostResources<DefaultPacketPool, CONNECTIONS_MAX, L2CAP_CHANNELS_MAX> =
        HostResources::new();
    let stack = trouble_host::new(controller, &mut resources).set_random_address(address);
    let Host {
        mut peripheral, runner, ..
    } = stack.build();

    log::info!("Creating GATT server");
    let server = Server::new_with_config(GapConfig::Peripheral(PeripheralConfig {
        name: "ESP32-GATT",
        appearance: &appearance::computer::GENERIC_COMPUTER,
    }))
    .unwrap();

    log::info!("GATT Service UUIDs:");
    log::info!("  Service:  12345678-1234-5678-1234-56789abcdef0");
    log::info!("  Counter:  12345678-1234-5678-1234-56789abcdef1 (read, notify)");
    log::info!("  Data:     12345678-1234-5678-1234-56789abcdef2 (read, write, notify)");
    log::info!("  Time:     12345678-1234-5678-1234-56789abcdef3 (read, write, notify)");
    log::info!("  Timezone: 12345678-1234-5678-1234-56789abcdef4 (read, write)");

    let _ = join(ble_task(runner), async {
        loop {
            match advertise(&mut peripheral, &server).await {
                Ok(conn) => {
                    log::info!("Client connected!");
                    let gatt_task = gatt_events_task(&server, &conn);
                    let counter_task = counter_update_task(&server, &conn);
                    let time_task = update_time_task(&server, &conn);
                    select3(gatt_task, counter_task, time_task ).await;
                    log::info!("Client disconnected");
                }
                Err(e) => {
                    log::error!("Advertising error: {:?}", e);
                }
            }
        }
    })
    .await;
}

/// Display task - updates OLED with current time every second
#[embassy_executor::task]
async fn display_task(i2c: I2c<'static, Blocking>) {
    // Create display interface
    let interface = I2CDisplayInterface::new(i2c);

    // Create SSD1306 display driver
    // The Airtip board uses a 72x40 visible area, but SSD1306 buffer is 128x64
    let mut display = Ssd1306::new(interface, DisplaySize72x40, DisplayRotation::Rotate0)
        .into_buffered_graphics_mode();

    display.init().unwrap();
    display.clear_buffer();

    let text_style = MonoTextStyle::new(&FONT_6X10, BinaryColor::On);

    log::info!("Display initialized");
    loop {
        let unix_time = get_unix_time();
        let (hours, minutes, seconds) = unix_to_time_of_day(unix_time);

        display.clear_buffer();

        // Draw border
        Rectangle::new(Point::new(0, 0), Size::new(72, 40))
            .into_styled(PrimitiveStyle::with_stroke(BinaryColor::On, 1))
            .draw(&mut display)
            .unwrap();

        // Erase block for time
        Rectangle::new(Point::new(5, 0), Size::new(62, 16))
            .into_styled(PrimitiveStyle::with_fill(BinaryColor::Off))
            .draw(&mut display)
            .unwrap();
            
        // Draw time in HH:MM:SS format (use fixed-size buffer for no_std)
        let mut time_text = heapless::String::<16>::new();
        write!(time_text, "{:02}:{:02}:{:02}", hours, minutes, seconds).unwrap();
        Text::new(&time_text, Point::new(12, 8), text_style)
            .draw(&mut display)
            .unwrap();

        // Flush to display
        display.flush().unwrap();

        Timer::after_secs(1).await;
    }
}

/// Background BLE task (must run continuously)
async fn ble_task<C: Controller, P: PacketPool>(mut runner: Runner<'_, C, P>) {
    loop {
        if let Err(e) = runner.run().await {
            log::error!("BLE task error: {:?}", e);
        }
    }
}

/// Advertise and wait for connection
async fn advertise<'values, 'server, C: Controller>(
    peripheral: &mut Peripheral<'values, C, DefaultPacketPool>,
    server: &'server Server<'values>,
) -> Result<GattConnection<'values, 'server, DefaultPacketPool>, BleHostError<C::Error>> {
    let mut advertiser_data = [0; 31];
    let len = AdStructure::encode_slice(
        &[
            AdStructure::Flags(LE_GENERAL_DISCOVERABLE | BR_EDR_NOT_SUPPORTED),
            AdStructure::CompleteLocalName(b"ESP32-GATT"),
        ],
        &mut advertiser_data[..],
    )?;

    let advertiser = peripheral
        .advertise(
            &Default::default(),
            Advertisement::ConnectableScannableUndirected {
                adv_data: &advertiser_data[..len],
                scan_data: &[],
            },
        )
        .await?;

    log::info!("Advertising as 'ESP32-GATT'...");
    let conn = advertiser.accept().await?.with_attribute_server(server)?;
    Ok(conn)
}

/// Handle GATT read/write events
async fn gatt_events_task<P: PacketPool>(
    server: &Server<'_>,
    conn: &GattConnection<'_, '_, P>,
) -> Result<(), Error> {
    let counter = server.simple_service.counter;
    let data = server.simple_service.data;
    let time = server.simple_service.time;
    let timezone = server.simple_service.timezone;

    let reason = loop {
        match conn.next().await {
            GattConnectionEvent::Disconnected { reason } => break reason,
            GattConnectionEvent::Gatt { event } => {
                match &event {
                    GattEvent::Read(read_event) => {
                        if read_event.handle() == counter.handle {
                            log::info!("[READ] Client read Counter characteristic");
                        } else if read_event.handle() == data.handle {
                            log::info!("[READ] Client read Data characteristic");
                        } else if read_event.handle() == time.handle {
                            log::info!("[READ] Client read Time characteristic");
                        }
                    }
                    GattEvent::Write(write_event) => {
                        if write_event.handle() == data.handle {
                            let bytes = write_event.data();
                            if !bytes.is_empty() {
                                let value = bytes[0];
                                log::info!("[WRITE] Client wrote {} to Data characteristic", value);
                                // Notify the client that the value changed
                                if let Err(e) = data.notify(conn, &value).await {
                                    log::warn!("[NOTIFY] Failed: {:?}", e);
                                } else {
                                    log::info!("[NOTIFY] Sent notification with value {}", value);
                                }
                            }
                        } else if write_event.handle() == time.handle {
                            let bytes = write_event.data();
                            if bytes.len() >= 4 {
                                let value = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
                                log::info!("[WRITE] Client wrote {} to Time characteristic", value);
                                // Update the UNIX_AT_BOOT offset
                                let now = Instant::now().as_secs() as u32;
                                let offset = value.wrapping_sub(now);
                                UNIX_AT_BOOT.store(offset, Ordering::Relaxed);
                                log::info!("[TIME] Updated UNIX_AT_BOOT offset to {}", offset);
                                // Notify the client that the value changed
                                if let Err(e) = time.notify(conn, &value).await {
                                    log::warn!("[NOTIFY] Failed: {:?}", e);
                                } else {
                                    log::info!("[NOTIFY] Sent notification with value {}", value);
                                }
                            }
                        } else if write_event.handle() == timezone.handle {
                            let bytes = write_event.data();
                            if !bytes.is_empty() {
                                // Read as signed i8 (quarter hours)
                                let quarter_hours = bytes[0] as i8;

                                // Validate range: UTC-12:00 to UTC+14:00 (-48 to +56 quarter hours)
                                if quarter_hours >= -48 && quarter_hours <= 56 {
                                    TIMEZONE_OFFSET.store(quarter_hours, Ordering::Relaxed);

                                    // Log in human-readable format
                                    let hours = quarter_hours / 4;
                                    let minutes = (quarter_hours.abs() % 4) * 15;
                                    log::info!("[WRITE] Timezone set to UTC{:+}:{:02}", hours, minutes);
                                } else {
                                    log::warn!("[WRITE] Invalid timezone offset: {} (must be -48 to +56 quarter hours)", quarter_hours);
                                }
                            }
                        }
                    }
                    _ => {}
                }
                // Send response to client
                match event.accept() {
                    Ok(reply) => reply.send().await,
                    Err(e) => log::warn!("Error sending GATT response: {:?}", e),
                }
            }
            _ => {}
        }
    };

    log::info!("Disconnected: {:?}", reason);
    Ok(())
}

/// Task that increments the counter every 5 seconds and notifies clients
async fn counter_update_task<P: PacketPool>(
    server: &Server<'_>,
    conn: &GattConnection<'_, '_, P>,
) {
    let counter = server.simple_service.counter;
    let mut count: u32 = 0;

    loop {
        Timer::after_secs(5).await;
        count = count.wrapping_add(1);
        log::info!("[UPDATE] Counter incremented to: {}", count);

        // Notify connected client of new counter value
        if let Err(e) = counter.notify(conn, &count).await {
            log::warn!("[NOTIFY] Failed to notify counter: {:?}", e);
            break;
        }
        log::info!("[NOTIFY] Sent counter notification: {}", count);
    }
}

fn get_unix_time() -> u32 {
    let offset = UNIX_AT_BOOT.load(Ordering::Relaxed);
    let now = Instant::now().as_secs() as u32;
    now.wrapping_add(offset)
}

/// Convert Unix timestamp to LOCAL time of day (hours, minutes, seconds)
/// Applies timezone offset for display
fn unix_to_time_of_day(unix_time: u32) -> (u8, u8, u8) {
    // Get timezone offset (in quarter hours)
    let tz_quarter_hours = TIMEZONE_OFFSET.load(Ordering::Relaxed) as i32;

    // Apply timezone offset (quarter_hours * 15 minutes * 60 seconds)
    let tz_seconds = tz_quarter_hours * 15 * 60;
    let local_unix_time = (unix_time as i32) + tz_seconds;

    // Handle negative wrap (shouldn't happen in practice with valid timestamps)
    let local_unix_time = if local_unix_time < 0 {
        0
    } else {
        local_unix_time as u32
    };

    let seconds_in_day = local_unix_time % 86400; // 86400 = seconds in a day
    let hours = (seconds_in_day / 3600) as u8;
    let minutes = ((seconds_in_day % 3600) / 60) as u8;
    let seconds = (seconds_in_day % 60) as u8;
    (hours, minutes, seconds)
}

async fn update_time_task<P: PacketPool>(
    server: &Server<'_>,
    conn: &GattConnection<'_, '_, P>,
) {
    let time = server.simple_service.time;

    loop {
        Timer::after_secs(1).await;
        let current_time = get_unix_time();
        log::info!("[UPDATE] Time updated to: {}", current_time);

        // Notify connected client of new time value
        if let Err(e) = time.notify(conn, &current_time).await {
            log::warn!("[NOTIFY] Failed to notify time: {:?}", e);
            break;
        }
        log::info!("[NOTIFY] Sent time notification: {}", current_time);
    }
}