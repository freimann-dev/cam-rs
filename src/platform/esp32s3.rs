// esp32s3.rs

use super::Platform;
use esp_hal::peripherals::{DMA_CH0, LCD_CAM};
use esp_hal::{gpio::Input, time::Instant};
use esp_hal::{
    lcd_cam::{
        LcdCam,
        cam::{Camera, Config as CamConfig},
    },
    time::Rate,
};
use esp_println::println;

#[derive(Debug)]
pub enum Esp32S3Error {
    XclkConfigFailed,
}

pub struct Esp32S3<'d> {
    camera: Option<Camera<'d>>,
    pclk_pin: Input<'d>,
}

impl<'d> Esp32S3<'d> {
    pub fn new(
        lcd_cam: LCD_CAM<'d>,
        dma: DMA_CH0<'d>,
        xclk_pin: impl esp_hal::gpio::interconnect::PeripheralOutput<'d>,
        pclk_pin: Input<'d>,
    ) -> Self {
        let lcd_cam_instance = LcdCam::new(lcd_cam);
        let camera_config = CamConfig::default().with_frequency(Rate::from_hz(20_000_000));
        let camera = Camera::new(lcd_cam_instance.cam, dma, camera_config)
            .expect("Camera peripheral init failed")
            .with_master_clock(xclk_pin);

        Self {
            camera: Some(camera),
            pclk_pin,
        }
    }
}

impl<'d> Platform for Esp32S3<'d> {
    type Error = Esp32S3Error;

    fn xclk_on(&mut self, freq_hz: u32) -> Result<(), Self::Error> {
        if self.camera.is_none() {
            return Err(Esp32S3Error::XclkConfigFailed);
        }
        println!("[platform] XCLK active at {} Hz", freq_hz);
        Ok(())
    }

    fn verify_pclk(&mut self) -> Result<(), Self::Error> {
        let initial_state = self.pclk_pin.is_high();
        let start = Instant::now();

        while start.elapsed().as_micros() < 1000 {
            if self.pclk_pin.is_high() != initial_state {
                println!("[platform] ✅ PCLK line active (edge detected)");
                return Ok(());
            }
        }

        println!("[platform] ❌ PCLK line static (no clock detected)");
        Err(Esp32S3Error::XclkConfigFailed)
    }
}
