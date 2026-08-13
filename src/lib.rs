#![no_std]

pub mod platform;
pub mod sensor;

use esp_println::println;
use platform::esp32s3::{Esp32S3Platform, Esp32S3Resources};
use sensor::ov5640::{Ov5640, Ov5640Resources};

#[derive(Debug)]
pub enum CameraError {
    PlatformInitFailed,
    SensorInitFailed,
    CaptureFailed,
}

pub struct Frame<'a> {
    data: &'a [u8],
}

impl<'a> Frame<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self { data }
    }
    pub fn data(&self) -> &[u8] {
        self.data
    }
}

pub struct CameraDriver<'d> {
    platform: Esp32S3Platform<'d>,
    _sensor: Ov5640<'d>,
}

impl<'d> CameraDriver<'d> {
    pub fn new(
        platform_res: Esp32S3Resources<'d>,
        sensor_res: Ov5640Resources<'d>,
    ) -> Result<Self, CameraError> {
        println!("[camera-rs] Initializing Esp32S3 platform...");
        let platform =
            Esp32S3Platform::new(platform_res).map_err(|_| CameraError::PlatformInitFailed)?;

        println!("[camera-rs] Initializing OV5640 sensor...");
        let mut sensor = Ov5640::new(sensor_res).map_err(|_| CameraError::SensorInitFailed)?;

        sensor.init().map_err(|_| CameraError::SensorInitFailed)?;

        println!("[camera-rs] Driver initialized successfully.");

        Ok(Self {
            platform,
            _sensor: sensor,
        })
    }

    pub fn get_frame(&mut self) -> Result<Frame<'_>, CameraError> {
        use platform::CameraPlatform;
        let buf = self
            .platform
            .get_current_buffer()
            .map_err(|_| CameraError::CaptureFailed)?;
        Ok(Frame::new(buf))
    }
}
