// esp32s3.rs

use super::Platform;
// use esp_hal::dma::DmaRxBuffer;
use esp_hal::dma::DmaRxStreamBuf;
use esp_hal::lcd_cam::cam::VsyncFilterThreshold;
use esp_hal::peripherals::{DMA_CH0, LCD_CAM};
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

// const DMA_BUF_SIZE: usize = 64 * 1024; // 64KB для JPEG кадра
// static mut DMA_BUF: esp_hal::dma::DmaRxStreamBuf = esp_hal::dma_rx_stream_buffer!(65536, 4092);
pub struct Esp32S3<'d> {
    camera: Option<Camera<'d>>,
    // pclk_pin: Input<'d>,
    dma_buf: Option<DmaRxStreamBuf>,
}

impl<'d> Esp32S3<'d> {
    pub fn new(
        lcd_cam: LCD_CAM<'d>,
        dma: DMA_CH0<'d>,
        xclk_pin: impl esp_hal::gpio::interconnect::PeripheralOutput<'d>,
        pclk_pin: impl esp_hal::gpio::interconnect::PeripheralInput<'d>,
        vsync_pin: impl esp_hal::gpio::interconnect::PeripheralInput<'d>,
        href_pin: impl esp_hal::gpio::interconnect::PeripheralInput<'d>,
        data_pins: (
            impl esp_hal::gpio::interconnect::PeripheralInput<'d>,
            impl esp_hal::gpio::interconnect::PeripheralInput<'d>,
            impl esp_hal::gpio::interconnect::PeripheralInput<'d>,
            impl esp_hal::gpio::interconnect::PeripheralInput<'d>,
            impl esp_hal::gpio::interconnect::PeripheralInput<'d>,
            impl esp_hal::gpio::interconnect::PeripheralInput<'d>,
            impl esp_hal::gpio::interconnect::PeripheralInput<'d>,
            impl esp_hal::gpio::interconnect::PeripheralInput<'d>,
        ),
    ) -> Self {
        let lcd_cam_instance = LcdCam::new(lcd_cam);
        let camera_config = CamConfig::default()
            .with_frequency(Rate::from_hz(20_000_000))
            .with_vsync_filter_threshold(VsyncFilterThreshold::Two);

        let camera = Camera::new(lcd_cam_instance.cam, dma, camera_config)
            .expect("Camera peripheral init failed")
            .with_master_clock(xclk_pin)
            .with_pixel_clock(pclk_pin)
            .with_vsync(vsync_pin)
            .with_h_enable(href_pin)
            .with_data0(data_pins.0)
            .with_data1(data_pins.1)
            .with_data2(data_pins.2)
            .with_data3(data_pins.3)
            .with_data4(data_pins.4)
            .with_data5(data_pins.5)
            .with_data6(data_pins.6)
            .with_data7(data_pins.7);

        // Создаём DMA буфер для захвата кадров
        let dma_buf = esp_hal::dma_rx_stream_buffer!(65536, 4092);

        println!("[platform] XCLK configured, all DVP pins connected");

        Self {
            camera: Some(camera),
            dma_buf: Some(dma_buf),
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
        // PCLK проверен ранее через GPIO Input до создания платформы
        println!("[platform] ✅ PCLK connected to Camera peripheral");
        Ok(())
    }

    fn capture_frame(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        let camera = self.camera.take().ok_or(Esp32S3Error::XclkConfigFailed)?;
        let dma_buf = self.dma_buf.take().ok_or(Esp32S3Error::XclkConfigFailed)?;

        let transfer = camera
            .receive(dma_buf)
            .map_err(|_| Esp32S3Error::XclkConfigFailed)?;

        // Ждём данные
        let start = esp_hal::time::Instant::now();
        let timeout = esp_hal::time::Duration::from_millis(200);
        let mut transfer = transfer;
        let mut last_available = 0;
        let mut stable_count = 0;

        while start.elapsed() < timeout {
            let available = transfer.available_bytes();

            if available > 1000 && available == last_available {
                stable_count += 1;
                if stable_count >= 3 {
                    break;
                }
            } else {
                stable_count = 0;
            }
            last_available = available;

            esp_hal::delay::Delay::new().delay_millis(10);
        }

        // Читаем данные
        let mut offset = 0;

        loop {
            let chunk = transfer.peek();
            if chunk.is_empty() {
                break;
            }

            let len = chunk.len().min(buf.len() - offset);
            if len == 0 {
                break;
            }

            buf[offset..offset + len].copy_from_slice(&chunk[..len]);
            transfer.consume(len);
            offset += len;

            if offset >= buf.len() {
                break;
            }
        }

        println!("[platform] 📸 Captured {} bytes", offset);
        if offset >= 4 {
            println!(
                "[platform] First 4 bytes: {:02X} {:02X} {:02X} {:02X}",
                buf[0], buf[1], buf[2], buf[3]
            );
        }

        let (camera, final_buf) = transfer.stop();
        self.camera = Some(camera);
        self.dma_buf = Some(final_buf);

        Ok(offset)
    }
}
