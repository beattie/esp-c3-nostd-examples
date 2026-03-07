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
        self.i2c.write(BME280_ADDR, &[reg, val]).await
    }

    // TODO: read a single register
    // Hint: self.i2c.write_read(addr, &[reg], &mut buf).await
    async fn read_reg(&mut self, reg: u8) -> Result<u8, E> {
        let mut buf = [0u8; 1];
        self.i2c.write_read(BME280_ADDR, &[reg], &mut buf).await?;
        Ok(buf[0])
    }

    // TODO: read multiple consecutive registers into buf
    async fn read_regs(&mut self, reg: u8, buf: &mut [u8]) -> Result<(), E> {
        self.i2c.write_read(BME280_ADDR, &[reg], buf).await
    }

    // ---- Init ---------------------------------------------------------------

    pub async fn init(&mut self) -> Result<(), Error<E>> {
        // TODO: read chip ID register, return Err(Error::BadChipId(id)) if wrong
        let id = self.read_reg(REG_CHIP_ID).await?;
        if id != CHIP_ID {
            return Err(Error::BadChipId(id));
        }

        // TODO: read calibration data
        //   - 24 bytes from REG_CALIB_TP  -> T1..T3, P1..P9
        //   - 1 byte  from REG_CALIB_H1   -> H1
        //   - 7 bytes from REG_CALIB_H2   -> H2..H6
        // Hint: calibration bytes are little-endian pairs
        //   T1 = u16::from_le_bytes([buf[0], buf[1]])
        //   T2 = i16::from_le_bytes([buf[2], buf[3]])
        //   H4/H5 are packed oddly -- see datasheet section 4.2.2
        let mut buf = [0u8; 24];
        self.read_regs(REG_CALIB_TP, &mut buf).await?;
        let dig_t1 = u16::from_le_bytes([buf[0], buf[1]]);
        let dig_t2 = i16::from_le_bytes([buf[2], buf[3]]);
        let dig_t3 = i16::from_le_bytes([buf[4], buf[5]]);
        let dig_p1 = u16::from_le_bytes([buf[6], buf[7]]);
        let dig_p2 = i16::from_le_bytes([buf[8], buf[9]]);
        let dig_p3 = i16::from_le_bytes([buf[10], buf[11]]);
        let dig_p4 = i16::from_le_bytes([buf[12], buf[13]]);
        let dig_p5 = i16::from_le_bytes([buf[14], buf[15]]);
        let dig_p6 = i16::from_le_bytes([buf[16], buf[17]]);
        let dig_p7 = i16::from_le_bytes([buf[18], buf[19]]);
        let dig_p8 = i16::from_le_bytes([buf[20], buf[21]]);
        let dig_p9 = i16::from_le_bytes([buf[22], buf[23]]);
        
        let dig_h1 = self.read_reg(REG_CALIB_H1).await?;

        let mut buf = [0u8; 7];
        self.read_regs(REG_CALIB_H2, &mut buf).await?;
        let dig_h2 = i16::from_le_bytes([buf[0], buf[1]]);
        let dig_h3 = buf[2];
            // H4 is bits [11:4] of this pair
        let dig_h4 = ((buf[3] as i8 as i16) << 4) | ((buf[4] & 0x0f) as i16);
            // H5 is bits [3:0] of this pair + bits [3:0] of next byte
        let dig_h5 = ((buf[5] as i8 as i16) << 4) | ((buf[4] >> 4) as i16);
        let dig_h6 = buf[6] as i8;
        
        self.calib = Some(CalibData {
            dig_t1, dig_t2, dig_t3,
            dig_p1, dig_p2, dig_p3, dig_p4, dig_p5, dig_p6, dig_p7, dig_p8,
                    dig_p9,
            dig_h1, dig_h2, dig_h3, dig_h4, dig_h5, dig_h6,
        });

        // TODO: configure sensor
        //   - REG_CONFIG:    0x00  IIR filter off, no standby time
        //   - REG_CTRL_HUM:  0x01  humidity oversampling x1  (must be written before ctrl_meas)
        //   - REG_CTRL_MEAS: 0x00  temp x1, pressure x1, sleep mode
        //     (forced mode triggered per-read)
        self.write_reg(REG_CONFIG, 0x00).await?;    // no IIR filter,
                                                    // no standby time
        self.write_reg(REG_CTRL_HUM, 0x01).await?;  // humidity oversampling x1
        self.write_reg(REG_CTRL_MEAS, 0x00).await?; // temp x1, pressure x1,
                                                    // sleep mode

        Ok(())
    }

    // ---- Read ---------------------------------------------------------------

    pub async fn read(&mut self) -> Result<Reading, Error<E>> {
        // TODO: return NotInitialized if calib is None
        if self.calib.is_none() {
            return Err(Error::NotInitialized);
        }

        // TODO: trigger forced mode measurement
        //   Write REG_CTRL_MEAS with osrs_t=001, osrs_p=001, mode=01 (forced)
        //   Bits: [7:5]=osrs_t [4:2]=osrs_p [1:0]=mode
        //   Value: 0b00100101 = 0x25
        self.write_reg(REG_CTRL_MEAS, 0x25).await?;

        // TODO: poll REG_STATUS bit 3 (measuring) until clear
        //   Use embassy_time::Timer::after(embassy_time::Duration::from_millis(10)).await between polls
        loop {
            let status = self.read_reg(REG_STATUS).await?;
            if (status & 0x08) == 0 {
                break;
            }
            embassy_time::Timer::after(
                    embassy_time::Duration::from_millis(10)).await;
        }

        // TODO: read 8 bytes from REG_DATA_START
        //   Bytes 0-2: pressure ADC  (20-bit, bits [19:4] in bytes 0-1, bits [3:0] in byte 2 high nibble)
        //   Bytes 3-5: temperature ADC (same packing)
        //   Bytes 6-7: humidity ADC  (16-bit big-endian)
        let mut buf = [0u8; 8];
        self.read_regs(REG_DATA_START, &mut buf).await?;
        let adc_p = ((buf[0] as i32) << 12) |
                    ((buf[1] as i32) << 4) |
                    ((buf[2] as i32) >> 4);
        let adc_t = ((buf[3] as i32) << 12) |
                    ((buf[4] as i32) << 4) |
                    ((buf[5] as i32) >> 4);
        let adc_h = ((buf[6] as i32) << 8) |
                    (buf[7] as i32);

        // TODO: call compensation functions (implement below)
        // let (t_fine, temperature_cdeg) = Self::compensate_temperature(calib, adc_t);
        // let pressure_pa = Self::compensate_pressure(calib, t_fine, adc_p);
        // let humidity_pct = Self::compensate_humidity(calib, t_fine, adc_h);
        let calib = self.calib.as_ref().unwrap();
        let (t_fine, temperature_cdeg) = Self::compensate_temperature(calib, adc_t);
        let pressure_pa = Self::compensate_pressure(calib, t_fine, adc_p);
        let humidity_pct = Self::compensate_humidity(calib, t_fine, adc_h);

        Ok(Reading {
            temperature_cdeg,
            pressure_pa,
            humidity_pct,
        })
    }

    // ---- Compensation formulas (datasheet section 4.2.3) --------------------
    // These are translated directly from the datasheet C code.
    // They look intimidating but it's just fixed-point arithmetic.

    // TODO: temperature compensation
    // Returns (t_fine, temperature_centidegrees)
    // t_fine is a shared intermediate value used by pressure and humidity too
    fn compensate_temperature(calib: &CalibData, adc_t: i32) -> (i32, i32) {
        let var1 = (((adc_t >> 3) - ((calib.dig_t1 as i32) << 1)) *
                (calib.dig_t2 as i32)) >> 11;
        let var2 = (((((adc_t >> 4) - (calib.dig_t1 as i32)) *  ((adc_t >> 4) -
                (calib.dig_t1 as i32))) >> 12) * (calib.dig_t3 as i32)) >> 14;
        let t_fine = var1 + var2;

        let temp_cdeg = (t_fine * 5 + 128) >> 8;
        (t_fine, temp_cdeg)
    }

    // TODO: pressure compensation
    // Returns pressure in Pa
    fn compensate_pressure(calib: &CalibData, t_fine: i32, adc_p: i32) -> u32 {
        let var1 = (t_fine as i64) - 128000;
        let var2 = var1 * var1 * (calib.dig_p6 as i64);
        let var2 = var2 + ((var1 * (calib.dig_p5 as i64)) << 17);
        let var2 = var2 + ((calib.dig_p4 as i64) << 35);
        let var1 = ((var1 * var1 * (calib.dig_p3 as i64))>>8) +
                ((var1 * (calib.dig_p2 as i64)) << 12);
        let var1 = (((1i64 << 47) + var1 )) * (calib.dig_p1 as i64) >> 33;
        if var1 == 0 {
            return 0;
        }
        let p = 1048576 - adc_p as i64;
        let p = (((p << 31)- var2) * 3125) / var1;
        let var1 = ((calib.dig_p9 as i64) * (p >> 13) * (p >> 13)) >> 25;
        let var2 = ((calib.dig_p8 as i64) * p) >> 19;
        let p = ((p + var1 + var2) >> 8) + ((calib.dig_p7 as i64) << 4);
        p as u32
    }

    // TODO: humidity compensation
    // Returns humidity as percent * 100
    fn compensate_humidity(calib: &CalibData, t_fine: i32, adc_h: i32) -> u32 {
        let v_x1 = t_fine - 76800;
        let v_x1 = ((((adc_h << 14) - ((calib.dig_h4 as i32) << 20) -
                ((calib.dig_h5 as i32) * v_x1)) + 16384) >> 15) * 
                (((((((v_x1 * (calib.dig_h6 as i32)) >> 10) *
                (((v_x1 * (calib.dig_h3 as i32)) >> 11) + 32768)) >> 10) +
                2097152) * (calib.dig_h2 as i32) + 8192) >> 14);
        let v_x1 = v_x1 - (((((v_x1 >> 15) * (v_x1 >> 15)) >> 7) *
                (calib.dig_h1 as i32)) >> 4);
        let v_x1 = v_x1.max(0);
        let v_x1 = v_x1.min(419430400);
        ((v_x1 as u64 * 100) >> 22) as u32
    }
}