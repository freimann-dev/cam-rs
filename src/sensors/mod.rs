pub mod ov5640;
use crate::BoardResult;

pub trait Sensor {
    const I2C_ADDR: u8;

    fn new<I2C>(&mut self, i2c: &mut I2C) -> BoardResult<()>
    where
        I2C: embedded_hal::i2c::I2c;
}
