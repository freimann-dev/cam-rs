use super::Camera;
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

impl<'d> Camera for Ov5640<'d> {
    type Error = Ov5640Error;

    fn enable(&mut self) -> Result<(), Self::Error> {
        let id_high = self.read_reg16(0x300A)?;
        let id_low = self.read_reg16(0x300B)?;
        let chip_id = ((id_high as u16) << 8) | (id_low as u16);

        if chip_id != 0x5640 {
            println!("[camera] Unexpected Chip ID: 0x{:04X}", chip_id);
            return Err(Ov5640Error::InvalidChipId(chip_id));
        }
        println!("[camera] Chip ID: 0x{:04X}", chip_id);

        // self.write_reg16(0x3008, 0x80)?;
        // esp_hal::delay::Delay::new().delay_millis(5);
        // self.write_reg16(0x3008, 0x02)?;

        // self.write_reg16(0x3017, 0xFF)?;
        // println!("[camera] reg 0x3017 -> 0xFF)...");

        Ok(())
    }

    fn init(&mut self) -> Result<(), Self::Error> {
        println!("[camera] Waiting 20ms for sensor internal PLL lock...");
        self.delay.delay_ms(20);

        println!("[camera] Reading OV5640 Chip ID...");
        let id_high = self.read_reg16(0x300A)?;
        let id_low = self.read_reg16(0x300B)?;
        let chip_id = ((id_high as u16) << 8) | (id_low as u16);

        if chip_id != 0x5640 {
            println!("[camera] ❌ Unexpected Chip ID: 0x{:04X}", chip_id);
            return Err(Ov5640Error::InvalidChipId(chip_id));
        }
        println!("[camera] ✅ OV5640 Chip ID verified: 0x{:04X}", chip_id);

        println!("[camera] Enabling DVP output strobe pins (reg 0x3017 -> 0xFF)...");
        self.write_reg16(0x3017, 0xFF)?;
        println!("[camera] ✅ DVP pins active");

        println!("[camera] 📷 Full sensor initialization for JPEG...");

        // Software reset
        self.write_reg16(0x3008, 0x82)?;
        self.delay.delay_ms(10);
        self.write_reg16(0x3008, 0x42)?; // power down

        // Enable PLL
        self.write_reg16(0x3103, 0x13)?;

        // IO direction
        self.write_reg16(0x3017, 0xFF)?;
        self.write_reg16(0x3018, 0xFF)?;

        self.write_reg16(0x302C, 0xC3)?; // DRIVE_CAPABILITY
        self.write_reg16(0x4740, 0x21)?; // CLOCK_POL_CONTROL

        self.write_reg16(0x4713, 0x02)?; // jpg mode select

        self.write_reg16(0x5001, 0x83)?; // ISP_CONTROL_01: turn color matrix, awb and SDE

        // Sys reset
        self.write_reg16(0x3000, 0x20)?; // reset MCU
        self.delay.delay_ms(10);
        self.write_reg16(0x3002, 0x1C)?;

        // Clock enable
        self.write_reg16(0x3004, 0xFF)?;
        self.write_reg16(0x3006, 0xC3)?;

        // ISP control
        self.write_reg16(0x5000, 0xA7)?;
        self.write_reg16(0x5001, 0xA3)?;
        self.write_reg16(0x5003, 0x08)?;

        // Important
        self.write_reg16(0x370C, 0x02)?;
        self.write_reg16(0x3634, 0x40)?;

        // AEC/AGC
        self.write_reg16(0x3A02, 0x03)?;
        self.write_reg16(0x3A03, 0xD8)?;
        self.write_reg16(0x3A08, 0x01)?;
        self.write_reg16(0x3A09, 0x27)?;
        self.write_reg16(0x3A0A, 0x00)?;
        self.write_reg16(0x3A0B, 0xF6)?;
        self.write_reg16(0x3A0D, 0x04)?;
        self.write_reg16(0x3A0E, 0x03)?;
        self.write_reg16(0x3A0F, 0x30)?;
        self.write_reg16(0x3A10, 0x28)?;
        self.write_reg16(0x3A11, 0x60)?;
        self.write_reg16(0x3A13, 0x43)?;
        self.write_reg16(0x3A14, 0x03)?;
        self.write_reg16(0x3A15, 0xD8)?;
        self.write_reg16(0x3A18, 0x00)?;
        self.write_reg16(0x3A19, 0xF8)?;
        self.write_reg16(0x3A1B, 0x30)?;
        self.write_reg16(0x3A1E, 0x26)?;
        self.write_reg16(0x3A1F, 0x14)?;

        // VCM debug
        self.write_reg16(0x3600, 0x08)?;
        self.write_reg16(0x3601, 0x33)?;

        // 50/60Hz
        self.write_reg16(0x3C01, 0xA4)?;
        self.write_reg16(0x3C04, 0x28)?;
        self.write_reg16(0x3C05, 0x98)?;
        self.write_reg16(0x3C06, 0x00)?;
        self.write_reg16(0x3C07, 0x08)?;
        self.write_reg16(0x3C08, 0x00)?;
        self.write_reg16(0x3C09, 0x1C)?;
        self.write_reg16(0x3C0A, 0x9C)?;
        self.write_reg16(0x3C0B, 0x40)?;

        self.write_reg16(0x460C, 0x22)?; // disable jpeg footer

        // BLC
        self.write_reg16(0x4001, 0x02)?;
        self.write_reg16(0x4004, 0x02)?;

        // AWB
        self.write_reg16(0x5180, 0xFF)?;
        self.write_reg16(0x5181, 0xF2)?;
        self.write_reg16(0x5182, 0x00)?;
        self.write_reg16(0x5183, 0x14)?;
        self.write_reg16(0x5184, 0x25)?;
        self.write_reg16(0x5185, 0x24)?;
        self.write_reg16(0x5186, 0x09)?;
        self.write_reg16(0x5187, 0x09)?;
        self.write_reg16(0x5188, 0x09)?;
        self.write_reg16(0x5189, 0x75)?;
        self.write_reg16(0x518A, 0x54)?;
        self.write_reg16(0x518B, 0xE0)?;
        self.write_reg16(0x518C, 0xB2)?;
        self.write_reg16(0x518D, 0x42)?;
        self.write_reg16(0x518E, 0x3D)?;
        self.write_reg16(0x518F, 0x56)?;
        self.write_reg16(0x5190, 0x46)?;
        self.write_reg16(0x5191, 0xF8)?;
        self.write_reg16(0x5192, 0x04)?;
        self.write_reg16(0x5193, 0x70)?;
        self.write_reg16(0x5194, 0xF0)?;
        self.write_reg16(0x5195, 0xF0)?;
        self.write_reg16(0x5196, 0x03)?;
        self.write_reg16(0x5197, 0x01)?;
        self.write_reg16(0x5198, 0x04)?;
        self.write_reg16(0x5199, 0x12)?;
        self.write_reg16(0x519A, 0x04)?;
        self.write_reg16(0x519B, 0x00)?;
        self.write_reg16(0x519C, 0x06)?;
        self.write_reg16(0x519D, 0x82)?;
        self.write_reg16(0x519E, 0x38)?;

        // Color matrix
        self.write_reg16(0x5381, 0x1E)?;
        self.write_reg16(0x5382, 0x5B)?;
        self.write_reg16(0x5383, 0x08)?;
        self.write_reg16(0x5384, 0x0A)?;
        self.write_reg16(0x5385, 0x7E)?;
        self.write_reg16(0x5386, 0x88)?;
        self.write_reg16(0x5387, 0x7C)?;
        self.write_reg16(0x5388, 0x6C)?;
        self.write_reg16(0x5389, 0x10)?;
        self.write_reg16(0x538A, 0x01)?;
        self.write_reg16(0x538B, 0x98)?;

        // CIP control
        self.write_reg16(0x5300, 0x10)?;
        self.write_reg16(0x5301, 0x10)?;
        self.write_reg16(0x5302, 0x18)?;
        self.write_reg16(0x5303, 0x19)?;
        self.write_reg16(0x5304, 0x10)?;
        self.write_reg16(0x5305, 0x10)?;
        self.write_reg16(0x5306, 0x08)?;
        self.write_reg16(0x5307, 0x16)?;
        self.write_reg16(0x5308, 0x40)?;
        self.write_reg16(0x5309, 0x10)?;
        self.write_reg16(0x530A, 0x10)?;
        self.write_reg16(0x530B, 0x04)?;
        self.write_reg16(0x530C, 0x06)?;

        // GAMMA
        self.write_reg16(0x5480, 0x01)?;
        self.write_reg16(0x5481, 0x00)?;
        self.write_reg16(0x5482, 0x1E)?;
        self.write_reg16(0x5483, 0x3B)?;
        self.write_reg16(0x5484, 0x58)?;
        self.write_reg16(0x5485, 0x66)?;
        self.write_reg16(0x5486, 0x71)?;
        self.write_reg16(0x5487, 0x7D)?;
        self.write_reg16(0x5488, 0x83)?;
        self.write_reg16(0x5489, 0x8F)?;
        self.write_reg16(0x548A, 0x98)?;
        self.write_reg16(0x548B, 0xA6)?;
        self.write_reg16(0x548C, 0xB8)?;
        self.write_reg16(0x548D, 0xCA)?;
        self.write_reg16(0x548E, 0xD7)?;
        self.write_reg16(0x548F, 0xE3)?;
        self.write_reg16(0x5490, 0x1D)?;

        // SDE
        self.write_reg16(0x5580, 0x06)?;
        self.write_reg16(0x5583, 0x40)?;
        self.write_reg16(0x5584, 0x10)?;
        self.write_reg16(0x5586, 0x20)?;
        self.write_reg16(0x5587, 0x00)?;
        self.write_reg16(0x5588, 0x00)?;
        self.write_reg16(0x5589, 0x10)?;
        self.write_reg16(0x558A, 0x00)?;
        self.write_reg16(0x558B, 0xF8)?;
        self.write_reg16(0x501D, 0x40)?;

        // Power on
        self.write_reg16(0x3008, 0x02)?;

        // 50Hz
        self.write_reg16(0x3C00, 0x04)?;

        self.delay.delay_ms(300);

        // JPEG format
        self.write_reg16(0x501F, 0x00)?;
        self.write_reg16(0x4300, 0x30)?;
        self.write_reg16(0x3002, 0x00)?;
        self.write_reg16(0x3006, 0xFF)?;
        self.write_reg16(0x471C, 0x50)?;

        // JPEG compression enable
        self.write_reg16(0x4401, 0x21)?;
        self.write_reg16(0x4407, 0x04)?;
        self.write_reg16(0x3821, 0x20)?;

        println!("[camera] ✅ JPEG compression enabled");

        // self.delay.delay_ms(100);

        Ok(())
    }

    fn write_sensor_table(&mut self) -> Result<(), Ov5640Error> {
        enum RegisterOp {
            Val(u16, u8),
            Delay(u32),
        }

        static DEFAULT_REGS: &[RegisterOp] = &[
            // Software reset
            RegisterOp::Val(0x3008, 0x82),
            RegisterOp::Delay(10),
            RegisterOp::Val(0x3008, 0x42), // power down
            // Enable PLL
            RegisterOp::Val(0x3103, 0x13),
            // IO direction
            RegisterOp::Val(0x3017, 0xFF),
            RegisterOp::Val(0x3018, 0xFF),
            RegisterOp::Val(0x302C, 0xC3), // DRIVE_CAPABILITY
            RegisterOp::Val(0x4740, 0x21), // CLOCK_POL_CONTROL
            RegisterOp::Val(0x4713, 0x02), // jpg mode select
            RegisterOp::Val(0x5001, 0x83), // ISP_CONTROL_01: turn color matrix, awb and SDE
            // Sys reset
            RegisterOp::Val(0x3000, 0x20), // reset MCU
            RegisterOp::Delay(10),
            RegisterOp::Val(0x3002, 0x1C),
            // Clock enable
            RegisterOp::Val(0x3004, 0xFF),
            RegisterOp::Val(0x3006, 0xC3),
            // ISP control
            RegisterOp::Val(0x5000, 0xA7),
            RegisterOp::Val(0x5001, 0xA3),
            RegisterOp::Val(0x5003, 0x08),
            // Important
            RegisterOp::Val(0x370C, 0x02),
            RegisterOp::Val(0x3634, 0x40),
            // AEC/AGC
            RegisterOp::Val(0x3A02, 0x03),
            RegisterOp::Val(0x3A03, 0xD8),
            RegisterOp::Val(0x3A08, 0x01),
            RegisterOp::Val(0x3A09, 0x27),
            RegisterOp::Val(0x3A0A, 0x00),
            RegisterOp::Val(0x3A0B, 0xF6),
            RegisterOp::Val(0x3A0D, 0x04),
            RegisterOp::Val(0x3A0E, 0x03),
            RegisterOp::Val(0x3A0F, 0x30),
            RegisterOp::Val(0x3A10, 0x28),
            RegisterOp::Val(0x3A11, 0x60),
            RegisterOp::Val(0x3A13, 0x43),
            RegisterOp::Val(0x3A14, 0x03),
            RegisterOp::Val(0x3A15, 0xD8),
            RegisterOp::Val(0x3A18, 0x00),
            RegisterOp::Val(0x3A19, 0xF8),
            RegisterOp::Val(0x3A1B, 0x30),
            RegisterOp::Val(0x3A1E, 0x26),
            RegisterOp::Val(0x3A1F, 0x14),
            // VCM debug
            RegisterOp::Val(0x3600, 0x08),
            RegisterOp::Val(0x3601, 0x33),
            // 50/60Hz
            RegisterOp::Val(0x3C01, 0xA4),
            RegisterOp::Val(0x3C04, 0x28),
            RegisterOp::Val(0x3C05, 0x98),
            RegisterOp::Val(0x3C06, 0x00),
            RegisterOp::Val(0x3C07, 0x08),
            RegisterOp::Val(0x3C08, 0x00),
            RegisterOp::Val(0x3C09, 0x1C),
            RegisterOp::Val(0x3C0A, 0x9C),
            RegisterOp::Val(0x3C0B, 0x40),
            RegisterOp::Val(0x460C, 0x22), // disable jpeg footer
            // BLC
            RegisterOp::Val(0x4001, 0x02),
            RegisterOp::Val(0x4004, 0x02),
            // AWB
            RegisterOp::Val(0x5180, 0xFF),
            RegisterOp::Val(0x5181, 0xF2),
            RegisterOp::Val(0x5182, 0x00),
            RegisterOp::Val(0x5183, 0x14),
            RegisterOp::Val(0x5184, 0x25),
            RegisterOp::Val(0x5185, 0x24),
            RegisterOp::Val(0x5186, 0x09),
            RegisterOp::Val(0x5187, 0x09),
            RegisterOp::Val(0x5188, 0x09),
            RegisterOp::Val(0x5189, 0x75),
            RegisterOp::Val(0x518A, 0x54),
            RegisterOp::Val(0x518B, 0xE0),
            RegisterOp::Val(0x518C, 0xB2),
            RegisterOp::Val(0x518D, 0x42),
            RegisterOp::Val(0x518E, 0x3D),
            RegisterOp::Val(0x518F, 0x56),
            RegisterOp::Val(0x5190, 0x46),
            RegisterOp::Val(0x5191, 0xF8),
            RegisterOp::Val(0x5192, 0x04),
            RegisterOp::Val(0x5193, 0x70),
            RegisterOp::Val(0x5194, 0xF0),
            RegisterOp::Val(0x5195, 0xF0),
            RegisterOp::Val(0x5196, 0x03),
            RegisterOp::Val(0x5197, 0x01),
            RegisterOp::Val(0x5198, 0x04),
            RegisterOp::Val(0x5199, 0x12),
            RegisterOp::Val(0x519A, 0x04),
            RegisterOp::Val(0x519B, 0x00),
            RegisterOp::Val(0x519C, 0x06),
            RegisterOp::Val(0x519D, 0x82),
            RegisterOp::Val(0x519E, 0x38),
            // Color matrix
            RegisterOp::Val(0x5381, 0x1E),
            RegisterOp::Val(0x5382, 0x5B),
            RegisterOp::Val(0x5383, 0x08),
            RegisterOp::Val(0x5384, 0x0A),
            RegisterOp::Val(0x5385, 0x7E),
            RegisterOp::Val(0x5386, 0x88),
            RegisterOp::Val(0x5387, 0x7C),
            RegisterOp::Val(0x5388, 0x6C),
            RegisterOp::Val(0x5389, 0x10),
            RegisterOp::Val(0x538A, 0x01),
            RegisterOp::Val(0x538B, 0x98),
            // CIP control
            RegisterOp::Val(0x5300, 0x10),
            RegisterOp::Val(0x5301, 0x10),
            RegisterOp::Val(0x5302, 0x18),
            RegisterOp::Val(0x5303, 0x19),
            RegisterOp::Val(0x5304, 0x10),
            RegisterOp::Val(0x5305, 0x10),
            RegisterOp::Val(0x5306, 0x08),
            RegisterOp::Val(0x5307, 0x16),
            RegisterOp::Val(0x5308, 0x40),
            RegisterOp::Val(0x5309, 0x10),
            RegisterOp::Val(0x530A, 0x10),
            RegisterOp::Val(0x530B, 0x04),
            RegisterOp::Val(0x530C, 0x06),
            // GAMMA
            RegisterOp::Val(0x5480, 0x01),
            RegisterOp::Val(0x5481, 0x00),
            RegisterOp::Val(0x5482, 0x1E),
            RegisterOp::Val(0x5483, 0x3B),
            RegisterOp::Val(0x5484, 0x58),
            RegisterOp::Val(0x5485, 0x66),
            RegisterOp::Val(0x5486, 0x71),
            RegisterOp::Val(0x5487, 0x7D),
            RegisterOp::Val(0x5488, 0x83),
            RegisterOp::Val(0x5489, 0x8F),
            RegisterOp::Val(0x548A, 0x98),
            RegisterOp::Val(0x548B, 0xA6),
            RegisterOp::Val(0x548C, 0xB8),
            RegisterOp::Val(0x548D, 0xCA),
            RegisterOp::Val(0x548E, 0xD7),
            RegisterOp::Val(0x548F, 0xE3),
            RegisterOp::Val(0x5490, 0x1D),
            // SDE
            RegisterOp::Val(0x5580, 0x06),
            RegisterOp::Val(0x5583, 0x40),
            RegisterOp::Val(0x5584, 0x10),
            RegisterOp::Val(0x5586, 0x20),
            RegisterOp::Val(0x5587, 0x00),
            RegisterOp::Val(0x5588, 0x00),
            RegisterOp::Val(0x5589, 0x10),
            RegisterOp::Val(0x558A, 0x00),
            RegisterOp::Val(0x558B, 0xF8),
            RegisterOp::Val(0x501D, 0x40),
            // Power on
            RegisterOp::Val(0x3008, 0x02),
            // 50Hz
            RegisterOp::Val(0x3C00, 0x04),
            RegisterOp::Delay(300),
            // JPEG format
            RegisterOp::Val(0x501F, 0x00),
            RegisterOp::Val(0x4300, 0x30),
            RegisterOp::Val(0x3002, 0x00),
            RegisterOp::Val(0x3006, 0xFF),
            RegisterOp::Val(0x471C, 0x50),
            // JPEG compression enable
            RegisterOp::Val(0x4401, 0x21),
            RegisterOp::Val(0x4407, 0x04),
            RegisterOp::Val(0x3821, 0x20),
        ];

        // println!("[camera] ⚙️ Uploading custom register table...");

        for entry in DEFAULT_REGS {
            match *entry {
                RegisterOp::Val(reg, val) => {
                    self.write_reg16(reg, val)?;
                }
                RegisterOp::Delay(ms) => {
                    // println!("[camera] ⏳ Waiting {} ms...", ms);
                    self.delay.delay_ms(ms);
                }
            }
        }

        println!("[camera] Default regs loaded!");
        Ok(())
    }
}
