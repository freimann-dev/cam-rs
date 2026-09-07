use crate::chips::Chip;
use crate::pins::Pins;
use crate::{CameraError, CameraResult};

use esp_hal::Blocking;
use esp_hal::delay::Delay;
use esp_hal::i2c::master::{Config as I2cConfig, I2c};
use esp_hal::lcd_cam::LcdCam;
use esp_hal::lcd_cam::cam::{Camera as DvpCamera, Config as CamConfig};
use esp_hal::peripherals::{DMA_CH0, I2C0, LCD_CAM};
use esp_hal::time::Rate;
use esp_println::println;

pub struct Esp32S3;

impl Esp32S3 {
    pub fn new() -> Self {
        Self
    }
}

impl<'a> Chip<Pins<'a>> for Esp32S3 {
    type I2c = I2c<'a, Blocking>;

    fn init(&mut self, pins: Pins<'a>) -> CameraResult<Self::I2c> {
        let lcd_cam = LcdCam::new(unsafe { LCD_CAM::steal() });
        let dma_ch = unsafe { DMA_CH0::steal() };
        let i2c_raw = unsafe { I2C0::steal() };

        let cam_config = CamConfig::default().with_frequency(Rate::from_mhz(20));

        let _dvp = DvpCamera::new(lcd_cam.cam, dma_ch, cam_config)
            .map_err(|_| CameraError::ChipInitFailed)?
            .with_master_clock(pins.mclk)
            .with_pixel_clock(pins.pclk)
            .with_vsync(pins.vsync)
            .with_h_enable(pins.hsync)
            .with_data0(pins.d0)
            .with_data1(pins.d1)
            .with_data2(pins.d2)
            .with_data3(pins.d3)
            .with_data4(pins.d4)
            .with_data5(pins.d5)
            .with_data6(pins.d6)
            .with_data7(pins.d7);

        println!("[chip] DvpCamera OK! (XCLK: 20 MHz, DVP 8-bit)");

        let delay = Delay::new();
        delay.delay_millis(10);

        let i2c_config = I2cConfig::default().with_frequency(Rate::from_khz(400));
        let i2c = I2c::new(i2c_raw, i2c_config)
            .map_err(|_| CameraError::ChipInitFailed)?
            .with_sda(pins.sda)
            .with_scl(pins.scl);

        println!("[chip] I2C OK! (Master mode, 100 kHz)");

        Ok(i2c)
    }

    fn check_vsync(&self) -> CameraResult<bool> {
        let gpio_regs = unsafe { &*esp_hal::peripherals::GPIO::PTR };

        let mut transitions = 0;

        let mut last_state = (gpio_regs.in_().read().bits() >> 6) & 1;

        let delay = Delay::new();
        for _ in 0..1_000 {
            let current_state = (gpio_regs.in_().read().bits() >> 6) & 1;

            if current_state != last_state {
                transitions += 1;
                if transitions == 2 {
                    break;
                }
                last_state = current_state;
            }
            delay.delay_micros(100);
            core::hint::spin_loop();
        }

        Ok(transitions == 2)
    }

    fn setup_dma(&mut self) -> CameraResult<()> {
        Ok(())
    }

    fn name(&self) -> &'static str {
        "ESP32-S3 (LCD_CAM + DMA)"
    }
}
