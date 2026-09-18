#![no_std]
#![no_main]

use cam_rs::{Board, Esp32S3, pins::s3eye, sensors::ov5640::Ov5640};
use esp_backtrace as _;
use esp_hal::dma::DmaRxBuf;
use esp_hal::dma_buffers;
use esp_hal::main;
use esp_println::println;

esp_bootloader_esp_idf::esp_app_desc!();

#[main]
fn main() -> ! {
    let peripherals = esp_hal::init(esp_hal::Config::default());
    let pins = s3eye(peripherals);
    let chip = Esp32S3::new();
    let sensor = Ov5640::default();

    let mut board = match Board::init(chip, pins, sensor) {
        Ok(b) => {
            println!("[EXAMPLE] Board successfully initialized!");
            b
        }
        Err(_) => {
            println!("[EXAMPLE] Failed to initialize board!");
            loop {}
        }
    };

    //***************************************************************

    let (rx_buffer_a, rx_descriptors_a, _, _) = dma_buffers!(4092, 0);
    let (rx_buffer_b, rx_descriptors_b, _, _) = dma_buffers!(4092, 0);

    let buf_a = DmaRxBuf::new(rx_descriptors_a, rx_buffer_a).unwrap();
    let buf_b = DmaRxBuf::new(rx_descriptors_b, rx_buffer_b).unwrap();

    let mut current_buffer = buf_a;
    let mut next_buffer = buf_b;

    loop {
        let transfer = match board.capture_chunk(current_buffer) {
            Ok(t) => t,
            Err(e) => {
                panic!("Capture chunk failed with error: {:?}", e);
            }
        };

        let (dma_result, camera, filled_buffer) = transfer.wait();

        if let Err(e) = dma_result {
            panic!("DMA transfer error: {:?}", e);
        }

        board.camera = Some(camera);

        current_buffer = next_buffer;

        // 5. Выводим адрес четвертого байта буфера и первые 4 байта
        let slice = filled_buffer.as_slice();
        let buffer_address = unsafe { slice.as_ptr().add(1) }; // Смещаемся на 4 байта внутрь данных
        let bytes = &slice[0..4];

        println!("Addr (+1B): {:p} | Bytes: {:02X?}", buffer_address, bytes);

        next_buffer = filled_buffer;
    }
}
