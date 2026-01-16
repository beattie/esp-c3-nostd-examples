# BLE Clock Timezone Implementation Guide

## Summary of Python Changes

The Python script `ble_time_sync.py` has been updated to support timezone with **quarter-hour precision** (15-minute increments).

### Why Quarter Hours?

Some timezones use 30 or 45-minute offsets:
- **India (IST)**: UTC+5:30
- **Nepal (NPT)**: UTC+5:45
- **Newfoundland (NST)**: UTC-3:30
- **Australia (ACST)**: UTC+9:30

### New Features:
1. **Read timezone** from device (in quarter hours)
2. **Write timezone** offset to device (quarter hours from UTC)
3. **Display local time** calculated from UTC + timezone offset
4. **Supports fractional hours**: 0.25 = 15min, 0.5 = 30min, 0.75 = 45min

### New UUID:
```python
TIMEZONE_CHAR_UUID = "f4debc9a-7856-3412-7856-341278563412"
```

### Data Format:
- **Type**: `i8` (signed 8-bit integer)
- **Unit**: Quarter hours (15 minutes)
- **Range**: -48 to +56 (UTC-12:00 to UTC+14:00)
- **Examples**:
  - `0` = UTC+0:00
  - `-32` = UTC-8:00 (PST: -8 × 4)
  - `22` = UTC+5:30 (IST: 5 × 4 + 2)
  - `23` = UTC+5:45 (NPT: 5 × 4 + 3)

### Usage Examples:
```bash
# Set time with PST timezone (UTC-8:00)
python3 scripts/ble_time_sync.py --set --timezone -8.0

# Set time with IST timezone (UTC+5:30)
python3 scripts/ble_time_sync.py --set --timezone 5.5

# Set time with NPT timezone (UTC+5:45)
python3 scripts/ble_time_sync.py --set --timezone 5.75

# Set time with NST timezone (UTC-3:30)
python3 scripts/ble_time_sync.py --set --timezone -3.5

# Read time and timezone
python3 scripts/ble_time_sync.py --read
```

---

## Required Changes to ble_clock.rs

### 1. Add Timezone Characteristic UUID

```rust
// After TIME_CHAR_UUID definition (around line 87):

// Timezone characteristic UUID: 12345678-1234-5678-1234-56789abcdef4
const TIMEZONE_CHAR_UUID: Uuid = Uuid::new_long([
    0x12, 0x34, 0x56, 0x78, 0x12, 0x34, 0x56, 0x78,
    0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc, 0xde, 0xf4,
]);
```

### 2. Add Timezone to GATT Service

```rust
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

    /// NEW: Timezone characteristic (read + write)
    /// Value: i8 offset from UTC in quarter hours (-48 to +56)
    /// Quarter hours support 15-minute increments (e.g., India UTC+5:30 = 22)
    /// Note: No notify needed - timezone rarely changes
    #[characteristic(uuid = TIMEZONE_CHAR_UUID, read, write, value = 0i8)]
    timezone: i8,
}
```

### 3. Add Global Timezone Storage

```rust
// After UNIX_AT_BOOT (around line 60):
static TIMEZONE_OFFSET: core::sync::atomic::AtomicI8 = core::sync::atomic::AtomicI8::new(0);
```

**Note:** ESP32-C3 has `AtomicI8` support since it's only 1 byte.

### 4. Update time conversion to use timezone

```rust
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
```

### 5. Add Timezone Write Handler

In the `gatt_events_task` function, add a handler for timezone writes:

```rust
GattEvent::Write(write_event) => {
    // ... existing data.handle check ...
    // ... existing time.handle check ...

    // NEW: Handle timezone writes
    else if write_event.handle() == timezone.handle {
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
```

### 6. Update Documentation Header

Update the header comment in `ble_clock.rs`:

```rust
//!   - Timezone Characteristic (UUID: 12345678-1234-5678-1234-56789abcdef4)
//!     Properties: Read, Write
//!     Value: i8 timezone offset from UTC in quarter hours (-48 to +56)
//!            Supports 15-minute increments (e.g., 22 = UTC+5:30 for India)
```

---

## Does Timezone Need Notify?

### Short Answer: **No, notify is not necessary**

### Reasoning:

1. **Timezone is basically static**
   - Users set timezone once when configuring the device
   - Timezone changes are rare (only when traveling or DST policy changes)
   - Not time-critical - can be read on-demand

2. **Write + Read is sufficient**
   - Client writes timezone when setting up device
   - Client can read timezone if needed (e.g., when reconnecting)
   - No need for real-time updates

3. **Comparison to Time characteristic**
   - Time uses notify because it changes every second
   - Timezone might change once per trip/year
   - Different update patterns

### When Notify Would Be Useful:

- **Multi-client scenario**: If multiple clients connect simultaneously and need to stay in sync
- **Configuration apps**: If you have a separate config app that changes timezone while clock app is connected
- **Not needed for single-client use case**

### Implementation Decision:

**Recommended:** `read + write` only (no notify)

```rust
#[characteristic(uuid = TIMEZONE_CHAR_UUID, read, write, value = 0i8)]
timezone: i8,
```

**If you want notify (optional):**

```rust
#[characteristic(uuid = TIMEZONE_CHAR_UUID, read, write, notify, value = 0i8)]
timezone: i8,
```

Then add notification in the write handler:

```rust
if write_event.handle() == timezone.handle {
    let value = bytes[0] as i8;
    if value >= -12 && value <= 14 {
        TIMEZONE_OFFSET.store(value, Ordering::Relaxed);
        log::info!("[WRITE] Timezone set to UTC{:+}", value);

        // Optional: notify other clients
        if let Err(e) = timezone.notify(conn, &value).await {
            log::warn!("[NOTIFY] Failed: {:?}", e);
        }
    }
}
```

---

## Complete Example Flow

### 1. User sets timezone on ESP32 (PST):
```bash
python3 scripts/ble_time_sync.py --set --timezone -8.0
```

### 2. ESP32 receives write:
- Stores `-32` (quarter hours) in `TIMEZONE_OFFSET`
- Logs: `[WRITE] Timezone set to UTC-8:00`

### 3. Display updates automatically:
- `unix_to_time_of_day()` reads `TIMEZONE_OFFSET` = -32
- Adds `-32 * 15 * 60 = -28800` seconds to Unix time
- Displays local PST time instead of UTC

### 4. User reads back:
```bash
python3 scripts/ble_time_sync.py --read
```

Output:
```
Device time (Unix): 1736973245
Device time (UTC):  2026-01-15 22:34:05
Timezone offset:     UTC-8:00
Device time (local): 2026-01-15 14:34:05
```

### 5. User sets India timezone (IST):
```bash
python3 scripts/ble_time_sync.py --set --timezone 5.5
```

### 6. ESP32 receives write:
- Stores `22` (quarter hours: 5 × 4 + 2) in `TIMEZONE_OFFSET`
- Logs: `[WRITE] Timezone set to UTC+5:30`

### 7. Display shows IST:
- Adds `22 * 15 * 60 = 19800` seconds to Unix time
- Displays local IST time

---

## Benefits of This Approach

1. **Simple**: Single i8 value, no complex data structures
2. **Efficient**: One byte storage, one atomic operation
3. **Standard**: Hours offset is the standard timezone representation
4. **Flexible**: Supports all world timezones (-12 to +14)
5. **Persistent**: Stored in static atomic, survives display updates

---

## Alternative: Store Timezone in Time Characteristic

Some implementations combine timezone with time in a single characteristic:

```rust
// Alternative: 5-byte characteristic [u32 unix_time, i8 timezone]
#[characteristic(uuid = TIME_CHAR_UUID, read, write, notify, value = [0u8; 5])]
time_with_tz: [u8; 5],
```

**Pros:**
- Single characteristic
- Atomic update of both values

**Cons:**
- More complex parsing
- Wastes bandwidth on every time update (timezone rarely changes)
- Less standard (time is usually just u32)

**Recommendation:** Use separate characteristics as designed above.

---

## Summary Table

| Aspect | Recommendation | Rationale |
|--------|---------------|-----------|
| **Characteristic** | Separate from time | Timezone rarely changes |
| **Type** | `i8` | -12 to +14 hours covers all timezones |
| **Properties** | `read + write` | Static value, no notify needed |
| **Storage** | `AtomicI8` | Thread-safe, single byte |
| **Display** | Apply offset in `unix_to_time_of_day()` | Clean separation of concerns |
| **Validation** | Range check -12 to +14 | Prevent invalid values |

---

## Testing

### Test Cases:

1. **Set PST (UTC-8:00)**:
   ```bash
   python3 scripts/ble_time_sync.py --set --timezone -8.0
   # Display should show PST time (8 hours behind UTC)
   ```

2. **Set IST India (UTC+5:30)**:
   ```bash
   python3 scripts/ble_time_sync.py --set --timezone 5.5
   # Display should show IST time (5.5 hours ahead of UTC)
   ```

3. **Set NPT Nepal (UTC+5:45)**:
   ```bash
   python3 scripts/ble_time_sync.py --set --timezone 5.75
   # Display should show NPT time (5.75 hours ahead of UTC)
   ```

4. **Set NST Newfoundland (UTC-3:30)**:
   ```bash
   python3 scripts/ble_time_sync.py --set --timezone -3.5
   # Display should show NST time (3.5 hours behind UTC)
   ```

5. **Set UTC (UTC+0:00)**:
   ```bash
   python3 scripts/ble_time_sync.py --set --timezone 0
   # Display should show UTC time
   ```

6. **Invalid timezone (out of range)**:
   ```bash
   python3 scripts/ble_time_sync.py --set --timezone 20
   # ERROR: --timezone must be between -12.0 and +14.0
   ```

7. **Invalid timezone (not quarter hour)**:
   ```bash
   python3 scripts/ble_time_sync.py --set --timezone 5.3
   # ERROR: --timezone must be in quarter-hour increments (0.25, 0.5, 0.75)
   ```

8. **Read timezone**:
   ```bash
   python3 scripts/ble_time_sync.py --read
   # Should show timezone offset in UTC±HH:MM format and local time
   ```
