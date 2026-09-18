#![no_std]
#![no_main]

use cam_rs::{Board, Esp32S3, pins::s3eye, sensors::ov5640::Ov5640};
use esp_backtrace as _;
use esp_hal::main;
use esp_println::println;

esp_bootloader_esp_idf::esp_app_desc!();

#[main]
fn main() -> ! {
    let p = esp_hal::init(esp_hal::Config::default());

    let chip = Esp32S3::new();
    let sensor = Ov5640::default();
    let pins = s3eye(p);

    match Board::new(chip, pins, sensor) {
        Ok(_cam) => println!("[EXAMPLE] Board successfully initialized!"),
        Err(_) => println!("[EXAMPLE] Failed to initialize board!"),
    }

    loop {
        core::hint::spin_loop();
    }
}
