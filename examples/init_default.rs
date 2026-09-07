#![no_std]
#![no_main]

use camera_rs::{Camera, Esp32S3, Ov5640, pins::s3_eye};
use esp_backtrace as _;
use esp_hal::main;
use esp_println::println;

esp_bootloader_esp_idf::esp_app_desc!();

#[main]
fn main() -> ! {
    let p = esp_hal::init(esp_hal::Config::default());

    let chip = Esp32S3::new();
    let sensor = Ov5640::default();
    let pins = s3_eye(p);

    match Camera::init(chip, pins, sensor) {
        Ok(_cam) => println!("[EXAMPLE] Camera successfully initialized!"),
        Err(_) => println!("[EXAMPLE] Failed to initialize camera!"),
    }

    loop {}
}
