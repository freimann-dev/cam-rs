pub mod host;
pub mod hosts;
pub mod sccb;
pub mod sensor;

use host::CameraHost;
use sensor::CameraSensor;

use esp_hal::{i2c::master::I2c, Blocking};
use esp_println::println;

pub struct CameraDriver<'d, S: CameraSensor, H: CameraHost> {
    pub i2c: I2c<'d, Blocking>,
    pub host: H,
    pub sensor: S,
    pub width: u16,
    pub height: u16,
}

impl<'d, S: CameraSensor, H: CameraHost> CameraDriver<'d, S, H> {
    pub fn new(mut i2c: I2c<'d, Blocking>, host: H, sensor: S) -> Result<Self, &'static str> {
        println!("[INFO] camera: Probing sensor...");

        sensor.detect(&mut i2c)?;
        println!(
            "[INFO] camera: Detected {} sensor at I2C 0x{:02X}",
            sensor.name(),
            sensor.i2c_address()
        );

        sensor.init(&mut i2c)?;
        println!("[DEBUG] camera: Sensor initialized");

        let (width, height) = sensor.get_native_resolution(&mut i2c)?;
        println!("[INFO] camera: Sensor frame size: {}x{} px", width, height);

        Ok(Self {
            i2c,
            host,
            sensor,
            width,
            height,
        })
    }
}
