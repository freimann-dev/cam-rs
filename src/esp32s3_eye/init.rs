use crate::sensors::Sensor;
use crate::{BoardError, BoardResult};
use esp_hal::delay::Delay;
use esp_hal::dma::DmaChannel;
use esp_hal::i2c::master::{Config as I2cConfig, I2c};
use esp_hal::lcd_cam::LcdCam;
use esp_hal::lcd_cam::cam::{Camera, Config};
use esp_hal::peripherals::Peripherals;
use esp_hal::psram::{Psram, PsramConfig};
use esp_hal::time::Rate;
use esp_println::println;

use super::Esp32S3Eye;

impl Esp32S3Eye {
    pub fn init<S: Sensor>(p: Peripherals) -> BoardResult<(Self, S)> {
        let psram_config = PsramConfig::default();
        let psram = Psram::new(p.PSRAM, psram_config);
        let (psram_base, psram_size) = psram.raw_parts();
        println!(
            "[board] PSRAM: base_ptr {:p}, total_size {}",
            psram_base, psram_size
        );

        let lcd_cam = LcdCam::new(p.LCD_CAM);
        let (rx_channel, _tx_channel) = p.DMA_CH0.split();

        let camera = Camera::new(
            lcd_cam.cam,
            rx_channel,
            Config::default().with_frequency(Rate::from_mhz(20)),
        )
        .map_err(|_| BoardError::ChipInitFailed)?
        .with_master_clock(p.GPIO15)
        .with_pixel_clock(p.GPIO13)
        .with_vsync(p.GPIO6)
        .with_h_enable(p.GPIO7)
        .with_data0(p.GPIO11)
        .with_data1(p.GPIO9)
        .with_data2(p.GPIO8)
        .with_data3(p.GPIO47)
        .with_data4(p.GPIO12)
        .with_data5(p.GPIO18)
        .with_data6(p.GPIO17)
        .with_data7(p.GPIO16)
        .into_async();

        println!("[board] Camera OK! (XCLK: 20 MHz, DVP 8-bit)");

        Delay::new().delay_millis(10);
        let mut i2c = I2c::new(
            p.I2C0,
            I2cConfig::default().with_frequency(Rate::from_khz(400)),
        )
        .map_err(|_| BoardError::ChipInitFailed)?
        .with_sda(p.GPIO4)
        .with_scl(p.GPIO5);

        println!("[board] SCCB OK! (Master mode, 400 kHz)");

        let mut sensor = S::new();
        sensor.init(&mut i2c)?;

        // 5. VSYNC
        // let gpio_regs = unsafe { &*esp_hal::peripherals::GPIO::PTR };
        // let mut transitions = 0;
        // let mut last_state = (gpio_regs.in_().read().bits() >> 6) & 1;
        // let delay = Delay::new();

        // for _ in 0..1_000 {
        //     let current_state = (gpio_regs.in_().read().bits() >> 6) & 1;
        //     if current_state != last_state {
        //         transitions += 1;
        //         if transitions == 2 {
        //             break;
        //         }
        //         last_state = current_state;
        //     }
        //     delay.delay_micros(100);
        //     // core::hint::spin_loop();
        // }

        // if transitions != 2 {
        //     return Err(BoardError::InitFailed);
        // }
        // println!("[board] VSYNC signal detected OK!");

        let board = Self {
            i2c,
            camera: Some(camera),
            psram_base,
            psram_size,
        };

        Ok((board, sensor))
    }
}
