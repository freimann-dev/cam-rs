// ov5640.rs

use super::Sensor;
use embedded_hal::delay::DelayNs;
use esp_hal::{
    Blocking,
    delay::Delay,
    i2c::master::{Config as I2cConfig, I2c},
    peripherals::I2C0,
    time::Rate,
};
use esp_println::println;

pub const OV5640_I2C_ADDR: u8 = 0x3C;

#[derive(Debug)]
pub enum Ov5640Error {
    I2cError,
    InvalidChipId(u16),
}

pub struct Ov5640<'d> {
    i2c: I2c<'d, Blocking>,
    delay: Delay,
}

impl<'d> Ov5640<'d> {
    pub fn new(
        i2c0: I2C0<'d>,
        sda: impl esp_hal::gpio::interconnect::PeripheralInput<'d>
        + esp_hal::gpio::interconnect::PeripheralOutput<'d>,
        scl: impl esp_hal::gpio::interconnect::PeripheralInput<'d>
        + esp_hal::gpio::interconnect::PeripheralOutput<'d>,
    ) -> Result<Self, Ov5640Error> {
        let i2c = I2c::new(
            i2c0,
            I2cConfig::default().with_frequency(Rate::from_khz(100)),
        )
        .map_err(|_| Ov5640Error::I2cError)?
        .with_sda(sda)
        .with_scl(scl);

        Ok(Self {
            i2c,
            delay: Delay::new(),
        })
    }

    fn read_reg16(&mut self, reg: u16) -> Result<u8, Ov5640Error> {
        let mut val = [0u8; 1];
        self.i2c
            .write_read(
                OV5640_I2C_ADDR,
                &[(reg >> 8) as u8, (reg & 0xFF) as u8],
                &mut val,
            )
            .map_err(|_| Ov5640Error::I2cError)?;
        Ok(val[0])
    }

    fn write_reg16(&mut self, reg: u16, val: u8) -> Result<(), Ov5640Error> {
        self.i2c
            .write(
                OV5640_I2C_ADDR,
                &[(reg >> 8) as u8, (reg & 0xFF) as u8, val],
            )
            .map_err(|_| Ov5640Error::I2cError)
    }
}

impl<'d> Sensor for Ov5640<'d> {
    type Error = Ov5640Error;

    fn init(&mut self) -> Result<(), Self::Error> {
        println!("[sensor] Waiting 20ms for sensor internal PLL lock...");
        self.delay.delay_ms(20);

        println!("[sensor] Reading OV5640 Chip ID...");
        let id_high = self.read_reg16(0x300A)?;
        let id_low = self.read_reg16(0x300B)?;
        let chip_id = ((id_high as u16) << 8) | (id_low as u16);

        if chip_id != 0x5640 {
            println!("[sensor] ❌ Unexpected Chip ID: 0x{:04X}", chip_id);
            return Err(Ov5640Error::InvalidChipId(chip_id));
        }
        println!("[sensor] ✅ OV5640 Chip ID verified: 0x{:04X}", chip_id);

        println!("[sensor] Enabling DVP output strobe pins (reg 0x3017 -> 0xFF)...");
        self.write_reg16(0x3017, 0xFF)?;
        println!("[sensor] ✅ DVP pins active");

        Ok(())
    }
}
