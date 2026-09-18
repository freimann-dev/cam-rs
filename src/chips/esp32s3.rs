use crate::Chip;
use crate::pins::Pins;
use crate::{BoardError, BoardResult};

use esp_hal::Blocking;
use esp_hal::delay::Delay;
use esp_hal::i2c::master::{Config as I2cConfig, I2c};
use esp_hal::lcd_cam::LcdCam;
use esp_hal::lcd_cam::cam::{Camera, Config};
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
    type Camera = Camera<'a>;

    fn new(&mut self, gpio: Pins<'a>) -> BoardResult<(Self::I2c, Self::Camera)> {
        let lcd_cam = LcdCam::new(unsafe { LCD_CAM::steal() });
        let dma_ch = unsafe { DMA_CH0::steal() };
        let i2c_raw = unsafe { I2C0::steal() };

        let config = Config::default().with_frequency(Rate::from_mhz(20));

        let camera = Camera::new(lcd_cam.cam, dma_ch, config)
            .map_err(|_| BoardError::ChipInitFailed)?
            .with_master_clock(gpio.mclk)
            .with_pixel_clock(gpio.pclk)
            .with_vsync(gpio.vsync)
            .with_h_enable(gpio.hsync)
            .with_data0(gpio.d0)
            .with_data1(gpio.d1)
            .with_data2(gpio.d2)
            .with_data3(gpio.d3)
            .with_data4(gpio.d4)
            .with_data5(gpio.d5)
            .with_data6(gpio.d6)
            .with_data7(gpio.d7);

        println!("[board] Camera OK! (XCLK: 20 MHz, DVP 8-bit)");

        let delay = Delay::new();
        delay.delay_millis(10);

        let i2c_config = I2cConfig::default().with_frequency(Rate::from_khz(400));
        let i2c = I2c::new(i2c_raw, i2c_config)
            .map_err(|_| BoardError::ChipInitFailed)?
            .with_sda(gpio.sda)
            .with_scl(gpio.scl);

        println!("[board] I2C OK! (Master mode, 400 kHz)");

        Ok((i2c, camera))
    }

    fn check_vsync(&self) -> BoardResult<bool> {
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

    fn setup_dma(&mut self) -> BoardResult<()> {
        Ok(())
    }

    fn name(&self) -> &'static str {
        "ESP32-S3 (LCD_CAM + DMA)"
    }
}
