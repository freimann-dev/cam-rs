// lib.rs

#![no_std]

pub mod platform;
pub mod sensor;

use esp_println::println;
use platform::Platform;
use sensor::Sensor;

#[derive(Debug)]
pub enum CameraError<PE, SE> {
    PlatformError(PE),
    SensorError(SE),
}
#[derive(Debug, Clone)]
pub struct Frame {
    pub width: usize,
    pub height: usize,
    pub data: &'static [u8],
}

pub struct Camera<P, S> {
    pub platform: P,
    pub sensor: S,
}

impl<P, S> Camera<P, S>
where
    P: Platform,
    S: Sensor,
{
    pub fn init(mut platform: P, mut sensor: S) -> Result<Self, CameraError<P::Error, S::Error>> {
        platform
            .xclk_on(20_000_000)
            .map_err(CameraError::PlatformError)?;

        // Ждём стабилизации clock
        esp_hal::delay::Delay::new().delay_millis(100);
        println!("[camera] ⏱ Waiting for sensor stabilization...");

        // Инициализация сенсора: проверка Chip ID + активация DVP
        sensor.init().map_err(CameraError::SensorError)?;

        // Проверка PCLK
        platform.verify_pclk().map_err(CameraError::PlatformError)?;

        Ok(Self { platform, sensor })
    }
    pub fn capture(&self) -> Result<Frame, &'static str> {
        static MOCK_FRAME: [u8; 12] = [
            255, 0, 0, // Красный
            0, 255, 0, // Зеленый
            0, 0, 255, // Синий
            255, 255, 0, // Желтый
        ];

        Ok(Frame {
            width: 2,
            height: 2,
            data: &MOCK_FRAME,
        })
    }
}
