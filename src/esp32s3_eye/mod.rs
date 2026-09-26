pub mod async_capture;
pub mod capture;
pub mod init;

use esp_hal::i2c::master::I2c;
use esp_hal::lcd_cam::cam::Camera;
use esp_hal::{Async, Blocking};

pub struct Esp32S3Eye {
    pub i2c: I2c<'static, Blocking>,
    pub camera: Option<Camera<'static, Async>>,
    pub psram_base: *mut u8,
    pub psram_size: usize,
}
