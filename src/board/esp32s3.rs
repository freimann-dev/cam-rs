extern crate alloc;

use super::Board;
use esp_hal::dma::DmaRxBuf;
// use esp_hal::dma_descriptors;
use esp_hal::gpio::interconnect::{PeripheralInput, PeripheralOutput};
use esp_hal::lcd_cam::LcdCam;
use esp_hal::lcd_cam::cam::{Camera, Config as CamConfig};
use esp_hal::peripherals::{DMA_CH0, LCD_CAM};
use esp_hal::time::{Duration, Instant, Rate};
use esp_println::println;

#[derive(Debug)]
pub enum Esp32S3Error {
    NotInitialized,
    XclkConfigFailed,
    CaptureFailed,
    VsyncNotFound,
    PclkNotFound,
    DmaTimeout,
    DmaInitFailed,
    DmaError,
}

pub struct Esp32S3<'d, XCLK, PCLK, VSYNC, HREF, D0, D1, D2, D3, D4, D5, D6, D7> {
    lcd_cam: Option<LCD_CAM<'d>>,
    dma: Option<DMA_CH0<'d>>,
    xclk_pin: Option<XCLK>,
    pclk_pin: Option<PCLK>,
    vsync_pin: Option<VSYNC>,
    href_pin: Option<HREF>,
    data_pins: Option<(D0, D1, D2, D3, D4, D5, D6, D7)>,
    // frame_buf: Option<&'static mut [u8]>,

    // Аппаратные ресурсы после активации XCLK
    camera: Option<Camera<'d>>,
    rx_buf: Option<DmaRxBuf>,
}

impl<'d, XCLK, PCLK, VSYNC, HREF, D0, D1, D2, D3, D4, D5, D6, D7>
    Esp32S3<'d, XCLK, PCLK, VSYNC, HREF, D0, D1, D2, D3, D4, D5, D6, D7>
where
    XCLK: PeripheralOutput<'d> + 'd,
    PCLK: PeripheralInput<'d> + 'd,
    VSYNC: PeripheralInput<'d> + 'd,
    HREF: PeripheralInput<'d> + 'd,
    D0: PeripheralInput<'d> + 'd,
    D1: PeripheralInput<'d> + 'd,
    D2: PeripheralInput<'d> + 'd,
    D3: PeripheralInput<'d> + 'd,
    D4: PeripheralInput<'d> + 'd,
    D5: PeripheralInput<'d> + 'd,
    D6: PeripheralInput<'d> + 'd,
    D7: PeripheralInput<'d> + 'd,
{
    pub fn new(
        lcd_cam: LCD_CAM<'d>,
        dma: DMA_CH0<'d>,
        xclk_pin: XCLK,
        pclk_pin: PCLK,
        vsync_pin: VSYNC,
        href_pin: HREF,
        data_pins: (D0, D1, D2, D3, D4, D5, D6, D7),
        // frame_buf: &'static mut [u8],
    ) -> Self {
        Self {
            lcd_cam: Some(lcd_cam),
            dma: Some(dma),
            xclk_pin: Some(xclk_pin),
            pclk_pin: Some(pclk_pin),
            vsync_pin: Some(vsync_pin),
            href_pin: Some(href_pin),
            data_pins: Some(data_pins),
            // frame_buf: Some(frame_buf),
            camera: None,
            rx_buf: None,
        }
    }
}

impl<'d, XCLK, PCLK, VSYNC, HREF, D0, D1, D2, D3, D4, D5, D6, D7> Board
    for Esp32S3<'d, XCLK, PCLK, VSYNC, HREF, D0, D1, D2, D3, D4, D5, D6, D7>
where
    XCLK: PeripheralOutput<'d> + 'd,
    PCLK: PeripheralInput<'d> + esp_hal::gpio::Pin + 'd,
    VSYNC: PeripheralInput<'d> + esp_hal::gpio::Pin + 'd,
    HREF: PeripheralInput<'d> + 'd,
    D0: PeripheralInput<'d> + 'd,
    D1: PeripheralInput<'d> + 'd,
    D2: PeripheralInput<'d> + 'd,
    D3: PeripheralInput<'d> + 'd,
    D4: PeripheralInput<'d> + 'd,
    D5: PeripheralInput<'d> + 'd,
    D6: PeripheralInput<'d> + 'd,
    D7: PeripheralInput<'d> + 'd,
{
    type Error = Esp32S3Error;

    fn xclk_on(&mut self, freq_hz: u32) -> Result<(), Self::Error> {
        if self.camera.is_some() {
            return Ok(());
        }

        // Извлекаем все периферийные ресурсы
        let lcd_cam = self.lcd_cam.take().ok_or(Esp32S3Error::NotInitialized)?;
        let dma = self.dma.take().ok_or(Esp32S3Error::NotInitialized)?;
        let xclk_pin = self.xclk_pin.take().ok_or(Esp32S3Error::NotInitialized)?;

        let pclk_pin = self.pclk_pin.take().ok_or(Esp32S3Error::PclkNotFound)?;
        let vsync_pin = self.vsync_pin.take().ok_or(Esp32S3Error::VsyncNotFound)?;
        let href_pin = self.href_pin.take().ok_or(Esp32S3Error::NotInitialized)?;
        let (d0, d1, d2, d3, d4, d5, d6, d7) =
            self.data_pins.take().ok_or(Esp32S3Error::NotInitialized)?;

        let lcd_cam_instance = LcdCam::new(lcd_cam);
        let camera_config = CamConfig::default().with_frequency(Rate::from_hz(freq_hz));

        // В esp-hal v1.1.2 привязка пинов делается к экземпляру Camera при создании/конфигурации
        let camera = Camera::new(lcd_cam_instance.cam, dma, camera_config)
            .map_err(|_| Esp32S3Error::XclkConfigFailed)?
            .with_master_clock(xclk_pin)
            .with_pixel_clock(pclk_pin)
            .with_vsync(vsync_pin)
            .with_hsync(href_pin)
            .with_data0(d0)
            .with_data1(d1)
            .with_data2(d2)
            .with_data3(d3)
            .with_data4(d4)
            .with_data5(d5)
            .with_data6(d6)
            .with_data7(d7);

        self.camera = Some(camera);
        println!(
            "[board_] XCLK active at {} Hz and DVP pins configured",
            freq_hz
        );

        Ok(())
    }

    // fn activate_pipeline(&mut self) -> Result<(), Self::Error> {
    //     if self.rx_buf.is_some() {
    //         return Ok(());
    //     }

    //     // Проверяем, что камера была инициализирована в xclk_on
    //     if self.camera.is_none() {
    //         return Err(Esp32S3Error::NotInitialized);
    //     }

    //     // Извлекаем фреймбуфер
    //     let frame_buf = self.frame_buf.take().ok_or(Esp32S3Error::NotInitialized)?;

    //     println!(
    //         "[board] Frame buffer ptr: {:p}, len: {}",
    //         frame_buf.as_ptr(),
    //         frame_buf.len()
    //     );

    //     let (rx_descriptors, _) = esp_hal::dma_descriptors!(36768);
    //     let rx_buf = DmaRxBuf::new(rx_descriptors, frame_buf).map_err(|e| {
    //         println!("[board] ❌ DmaRxBuf error: {:?}", e); // Выведет enum с точной причиной!
    //         Esp32S3Error::DmaInitFailed
    //     })?;

    //     self.rx_buf = Some(rx_buf);

    //     println!("[board] ✅ DMA and DVP Camera pipeline activated!");
    //     Ok(())
    // }

    fn capture_frame(&mut self) -> Result<&[u8], Esp32S3Error> {
        // 1. Извлекаем ресурсы Camera и DmaRxBuf
        let camera = self.camera.take().ok_or(Esp32S3Error::NotInitialized)?;
        let rx_buf = self.rx_buf.take().ok_or(Esp32S3Error::NotInitialized)?;

        // 2. Статуем прием DMA
        let transfer = match camera.receive(rx_buf) {
            Ok(t) => t,
            Err((err, cam, buf)) => {
                println!("[board] camera.receive failed: {:?}", err);
                self.camera = Some(cam);
                self.rx_buf = Some(buf);
                return Err(Esp32S3Error::CaptureFailed);
            }
        };

        // 3. Ждем окончания кадра по VSYNC с таймаутом (200 мс)
        let start = Instant::now();
        let timeout = Duration::from_millis(200);

        while !transfer.is_done() {
            if start.elapsed() >= timeout {
                println!("[board] ⚠️ VSYNC / DMA Timeout!");
                break;
            }
            esp_hal::delay::Delay::new().delay_millis(1);
        }

        // 4. Останавливаем трансфер и забираем объекты назад
        let (cam, rx_buf) = transfer.stop();

        // 5. Возвращаем ресурсы в плату
        self.camera = Some(cam);

        // 6. Получаем полученный срез кадра
        let frame_len = rx_buf.len();

        // Возвращаем DmaRxBuf на место
        self.rx_buf = Some(rx_buf);

        if frame_len == 0 {
            return Err(Esp32S3Error::CaptureFailed);
        }

        // Забираем ссылку на данные кадра из сохранённого rx_buf
        let frame_ref = self.rx_buf.as_ref().unwrap();
        Ok(&frame_ref.as_slice()[..frame_len])
    }
}
