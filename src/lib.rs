#![no_std]

pub mod chips;
pub mod pins;
pub mod sensors;

// extern crate alloc;

pub use chips::Chip;
pub use chips::esp32s3::Esp32S3;
use esp_hal::dma::DmaRxBuf;
use esp_hal::lcd_cam::cam::CameraTransfer;
pub use sensors::Sensor;

// use core::cell::UnsafeCell;

// Обертка для безопасных глобальных статических переменных в Rust 2024
// struct LibSyncCell<T>(UnsafeCell<T>);
// unsafe impl<T> Sync for LibSyncCell<T> {}

#[repr(C, align(4))]
// struct LibDescriptors([esp_hal::dma::DmaDescriptor; 1]);

// Выделяем дескрипторы и данные SRAM глобально со временем жизни 'static
// static DESC_A: LibSyncCell<LibDescriptors> = LibSyncCell(UnsafeCell::new(LibDescriptors(
// [esp_hal::dma::DmaDescriptor::EMPTY; 1],
// )));
// static DESC_B: LibSyncCell<LibDescriptors> = LibSyncCell(UnsafeCell::new(LibDescriptors(
// [esp_hal::dma::DmaDescriptor::EMPTY; 1],
// )));

// Вместо static mut DATA_A / DATA_B пишем безопасные статические обёртки
#[repr(C, align(4))]
// struct LibDataBuffer([u8; 4092]);

// static DATA_A: LibSyncCell<LibDataBuffer> = LibSyncCell(UnsafeCell::new(LibDataBuffer([0; 4092])));
// static DATA_B: LibSyncCell<LibDataBuffer> = LibSyncCell(UnsafeCell::new(LibDataBuffer([0; 4092])));
#[derive(Debug, Clone, Copy)]
pub enum BoardError {
    I2cError,
    SensorMismatch(u16),
    InitFailed,
    ChipInitFailed,
    CaptureTimeout,
}

pub type BoardResult<T> = Result<T, BoardError>;

impl<E: embedded_hal::i2c::Error> From<E> for BoardError {
    fn from(_: E) -> Self {
        BoardError::I2cError
    }
}

pub struct Board<'a, C, S, I2C> {
    pub chip: C,
    pub sensor: S,
    pub i2c: I2C,
    pub camera: Option<esp_hal::lcd_cam::cam::Camera<'a>>,
}

impl<'a, C, S, I2C> Board<'a, C, S, I2C>
where
    I2C: embedded_hal::i2c::I2c,
    S: Sensor,
{
    pub fn write_reg(&mut self, reg: u16, value: u8) -> BoardResult<()> {
        let bytes = [(reg >> 8) as u8, reg as u8, value];
        self.i2c
            .write(S::I2C_ADDR, &bytes)
            .map_err(|_| BoardError::I2cError)?;
        Ok(())
    }

    pub fn read_reg(&mut self, reg: u16) -> BoardResult<u8> {
        let reg_bytes = [(reg >> 8) as u8, reg as u8];
        let mut data = [0u8; 1];
        self.i2c
            .write_read(S::I2C_ADDR, &reg_bytes, &mut data)
            .map_err(|_| BoardError::I2cError)?;
        Ok(data[0])
    }

    pub fn new<P>(mut chip: C, pins: P, mut sensor: S) -> BoardResult<Self>
    where
        C: Chip<P, I2c = I2C, Camera = esp_hal::lcd_cam::cam::Camera<'a>>,
    {
        let (mut i2c, camera) = chip.new(pins)?;

        sensor.new(&mut i2c)?;

        if !chip.check_vsync()? {
            return Err(BoardError::InitFailed);
        }

        Ok(Board {
            chip,
            sensor,
            i2c,
            camera: Some(camera),
        })
    }

    pub fn capture_chunk(
        &mut self,
        sram_buffer: DmaRxBuf,
    ) -> Result<CameraTransfer<'a, DmaRxBuf>, BoardError> {
        let camera = self.camera.take().ok_or(BoardError::ChipInitFailed)?;

        let transfer = camera
            .receive(sram_buffer)
            .map_err(|_| BoardError::InitFailed)?;

        Ok(transfer)
    }

    // pub fn capture_frame(
    //     &mut self,
    //     psram_buf_0: &mut [u8],
    //     psram_buf_1: &mut [u8],
    // ) -> Result<(), BoardError> {
    //     let sram_buf_a = unsafe {
    //         let desc_slice = &mut (*DESC_A.0.get()).0;
    //         let data_slice = &mut (*DATA_A.0.get()).0;
    //         DmaRxBuf::new(desc_slice, data_slice).unwrap()
    //     };

    //     let sram_buf_b = unsafe {
    //         let desc_slice = &mut (*DESC_B.0.get()).0;
    //         let data_slice = &mut (*DATA_B.0.get()).0;
    //         DmaRxBuf::new(desc_slice, data_slice).unwrap()
    //     };

    //     let chunk_size = 4092;
    //     let mut write_offset = 0;
    //     let target_len = psram_buf_0.len();

    //     let mut current_sram = sram_buf_a;
    //     let mut next_sram = sram_buf_b;

    //     while write_offset < target_len {
    //         let chunk_transfer = self.capture_chunk(current_sram)?;
    //         let (_dma_status, returned_camera, filled_sram) = chunk_transfer.wait();
    //         self.camera = Some(returned_camera);

    //         let sram_slice = filled_sram.as_slice();
    //         let current_chunk_size = core::cmp::min(chunk_size, target_len - write_offset);

    //         psram_buf_0[write_offset..write_offset + current_chunk_size]
    //             .copy_from_slice(&sram_slice[0..current_chunk_size]);
    //         psram_buf_1[write_offset..write_offset + current_chunk_size]
    //             .copy_from_slice(&sram_slice[0..current_chunk_size]);

    //         write_offset += current_chunk_size;

    //         current_sram = next_sram;
    //         next_sram = filled_sram;
    //     }

    //     esp_println::println!(
    //         "[cam-rs] Кадр собран! Буфер 0: {:02X?}, Буфер 1: {:02X?}",
    //         &psram_buf_0[..4],
    //         &psram_buf_1[..4]
    //     );

    //     Ok(())
    // }
}
