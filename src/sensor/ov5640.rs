use esp_hal::{gpio::Output, peripherals::I2C0};

pub struct Ov5640Resources<'d> {
    pub i2c: I2C0<'d>,
    pub reset_pin: Option<Output<'d>>,
    pub pwdn_pin: Option<Output<'d>>,
}

pub struct Ov5640<'d> {
    _i2c: I2C0<'d>,
    _reset: Option<Output<'d>>,
    _pwdn: Option<Output<'d>>,
}

impl<'d> Ov5640<'d> {
    pub fn new(resources: Ov5640Resources<'d>) -> Result<Self, ()> {
        Ok(Self {
            _i2c: resources.i2c,
            _reset: resources.reset_pin,
            _pwdn: resources.pwdn_pin,
        })
    }

    pub fn init(&mut self) -> Result<(), ()> {
        Ok(())
    }
}
