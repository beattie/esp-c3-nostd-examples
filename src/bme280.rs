// BME280 async I2C driver
// Datasheet: https://www.bosch-sensortec.com/media/boschsensortec/downloads/datasheets/bst-bme280-ds002.pdf
//
// Key datasheet sections:
//   5.3   - Memory map
//   5.4   - Register descriptions
//   5.4.1 - Register 0xD0 "id" (chip ID)
//   5.4.2 - Register 0xE0 "reset"
//   5.4.3 - Register 0xF2 "ctrl_hum"
//   5.4.4 - Register 0xF3 "status"
//   5.4.5 - Register 0xF4 "ctrl_meas"
//   5.4.6 - Register 0xF5 "config"
//   5.4.7 - Register 0xF7..0xF9 "press"
//   5.4.8 - Register 0xFA..0xFC "temp"
//   5.4.9 - Register 0xFD..0xFE "hum"
//   4.2.2 - Trimming parameter (calibration) readout
//   4.2.3 - Compensation formulas
//   6.2   - I2C interface and address

use embedded_hal_async::i2c::I2c;

// I2C address (datasheet section 6.2, SDO=GND)
const BME280_ADDR: u8 = 0x76;

// Register addresses (datasheet section 5.4)
const REG_CHIP_ID: u8    = 0xd0;
const REG_RESET: u8      = 0xe0;
const REG_CTRL_HUM: u8   = 0xf2;
const REG_STATUS: u8     = 0xf3;
const REG_CTRL_MEAS: u8  = 0xf4;
const REG_CONFIG: u8     = 0xf5;
const REG_DATA_START: u8 = 0xf7;  // first of 8 data registers (press + temp + hum)

// Calibration data register blocks (datasheet section 4.2.2 "Trimming parameter readout")
const REG_CALIB_TP: u8   = 0x88;  // T1..P9: 24 bytes (0x88..0x9F)
const REG_CALIB_H1: u8   = 0xa1;  // H1: 1 byte
const REG_CALIB_H2: u8   = 0xe1;  // H2..H6: 7 bytes (0xe1..0xe7), H4/H5 bit-packed

// Expected chip ID value (datasheet section 5.4.1)
const CHIP_ID: u8 = 0x60;

// ---- Calibration data -------------------------------------------------------

struct CalibData {
    // Temperature
    dig_t1: u16,
    dig_t2: i16,
    dig_t3: i16,
    // Pressure
    dig_p1: u16,
    dig_p2: i16,
    dig_p3: i16,
    dig_p4: i16,
    dig_p5: i16,
    dig_p6: i16,
    dig_p7: i16,
    dig_p8: i16,
    dig_p9: i16,
    // Humidity
    dig_h1: u8,
    dig_h2: i16,
    dig_h3: u8,
    dig_h4: i16,
    dig_h5: i16,
    dig_h6: i8,
}

// ---- Public types -----------------------------------------------------------

pub struct Reading {
    pub temperature_cdeg: i32,  // centidegrees C (2345 = 23.45 C)
    pub pressure_pa: u32,       // Pascals
    pub humidity_pct: u32,      // percent * 100 (5432 = 54.32 %RH)
}

#[derive(Debug)]
pub enum Error<E> {
    I2c(E),
    BadChipId(u8),
    NotInitialized,
}

// Lets ? operator convert I2c errors automatically
impl<E> From<E> for Error<E> {
    fn from(e: E) -> Self {
        Error::I2c(e)
    }
}

// ---- Driver struct ----------------------------------------------------------

pub struct Bme280<I2C> {
    i2c: I2C,
    calib: Option<CalibData>,
}

impl<I2C, E> Bme280<I2C>
where
    I2C: I2c<Error = E>,
{
    pub fn new(i2c: I2C) -> Self {
        Self { i2c, calib: None }
    }

    // ---- Low-level I2C helpers ----------------------------------------------

    // TODO: write a single register
    // Hint: self.i2c.write(addr, &[reg, value]).await
    async fn write_reg(&mut self, reg: u8, val: u8) -> Result<(), E> {
        todo!()
    }

    // TODO: read a single register
    // Hint: self.i2c.write_read(addr, &[reg], &mut buf).await
    async fn read_reg(&mut self, reg: u8) -> Result<u8, E> {
        todo!()
    }

    // TODO: read multiple consecutive registers into buf
    async fn read_regs(&mut self, reg: u8, buf: &mut [u8]) -> Result<(), E> {
        todo!()
    }

    // ---- Init ---------------------------------------------------------------

    pub async fn init(&mut self) -> Result<(), Error<E>> {
        // TODO: read chip ID register, return Err(Error::BadChipId(id)) if wrong

        // TODO: read calibration data
        //   - 24 bytes from REG_CALIB_TP  -> T1..T3, P1..P9
        //   - 1 byte  from REG_CALIB_H1   -> H1
        //   - 7 bytes from REG_CALIB_H2   -> H2..H6
        // Hint: calibration bytes are little-endian pairs
        //   T1 = u16::from_le_bytes([buf[0], buf[1]])
        //   T2 = i16::from_le_bytes([buf[2], buf[3]])
        //   H4/H5 are packed oddly -- see datasheet section 4.2.2

        // TODO: configure sensor
        //   - REG_CONFIG:    0x00  IIR filter off, no standby time
        //   - REG_CTRL_HUM:  0x01  humidity oversampling x1  (must be written before ctrl_meas)
        //   - REG_CTRL_MEAS: 0x00  temp x1, pressure x1, sleep mode
        //     (forced mode triggered per-read)

        todo!()
    }

    // ---- Read ---------------------------------------------------------------

    pub async fn read(&mut self) -> Result<Reading, Error<E>> {
        // TODO: return NotInitialized if calib is None

        // TODO: trigger forced mode measurement
        //   Write REG_CTRL_MEAS with osrs_t=001, osrs_p=001, mode=01 (forced)
        //   Bits: [7:5]=osrs_t [4:2]=osrs_p [1:0]=mode
        //   Value: 0b00100101 = 0x25

        // TODO: poll REG_STATUS bit 3 (measuring) until clear
        //   Use embassy_time::Timer::after(embassy_time::Duration::from_millis(10)).await between polls

        // TODO: read 8 bytes from REG_DATA_START
        //   Bytes 0-2: pressure ADC  (20-bit, bits [19:4] in bytes 0-1, bits [3:0] in byte 2 high nibble)
        //   Bytes 3-5: temperature ADC (same packing)
        //   Bytes 6-7: humidity ADC  (16-bit big-endian)

        // TODO: call compensation functions (implement below)
        // let (t_fine, temperature_cdeg) = Self::compensate_temperature(calib, adc_t);
        // let pressure_pa = Self::compensate_pressure(calib, t_fine, adc_p);
        // let humidity_pct = Self::compensate_humidity(calib, t_fine, adc_h);

        todo!()
    }

    // ---- Compensation formulas (datasheet section 4.2.3) --------------------
    // These are translated directly from the datasheet C code.
    // They look intimidating but it's just fixed-point arithmetic.

    // TODO: temperature compensation
    // Returns (t_fine, temperature_centidegrees)
    // t_fine is a shared intermediate value used by pressure and humidity too
    fn compensate_temperature(calib: &CalibData, adc_t: i32) -> (i32, i32) {
        todo!()
    }

    // TODO: pressure compensation
    // Returns pressure in Pa
    fn compensate_pressure(calib: &CalibData, t_fine: i32, adc_p: i32) -> u32 {
        todo!()
    }

    // TODO: humidity compensation
    // Returns humidity as percent * 100
    fn compensate_humidity(calib: &CalibData, t_fine: i32, adc_h: i32) -> u32 {
        todo!()
    }
}
