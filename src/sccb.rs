use esp_hal::{i2c::master::I2c, Blocking};

pub struct SccbBus;

impl SccbBus {
    /// Запись 8-битного значения в 16-битный адрес регистра (OV5640)
    pub fn write_reg_u16(
        i2c: &mut I2c<'_, Blocking>,
        addr: u8,
        reg: u16,
        val: u8,
    ) -> Result<(), &'static str> {
        i2c.write(addr, &[(reg >> 8) as u8, (reg & 0xFF) as u8, val])
            .map_err(|_| "SCCB write_u16 failed")
    }

    /// Чтение 8-битного значения из 16-битного адреса регистра (OV5640)
    pub fn read_reg_u16(
        i2c: &mut I2c<'_, Blocking>,
        addr: u8,
        reg: u16,
    ) -> Result<u8, &'static str> {
        let mut buf = [0u8; 1];
        i2c.write_read(addr, &[(reg >> 8) as u8, (reg & 0xFF) as u8], &mut buf)
            .map_err(|_| "SCCB read_u16 failed")?;
        Ok(buf[0])
    }

    /// Запись 8-битного значения в 8-битный адрес регистра (для будущих OV2640 / GC0308)
    pub fn write_reg_u8(
        i2c: &mut I2c<'_, Blocking>,
        addr: u8,
        reg: u8,
        val: u8,
    ) -> Result<(), &'static str> {
        i2c.write(addr, &[reg, val])
            .map_err(|_| "SCCB write_u8 failed")
    }

    /// Чтение 8-битного значения из 8-битного адреса регистра (для будущих OV2640 / GC0308)
    pub fn read_reg_u8(i2c: &mut I2c<'_, Blocking>, addr: u8, reg: u8) -> Result<u8, &'static str> {
        let mut buf = [0u8; 1];
        i2c.write_read(addr, &[reg], &mut buf)
            .map_err(|_| "SCCB read_u8 failed")?;
        Ok(buf[0])
    }
}
