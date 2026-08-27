#![no_std]
#![no_main]

use camera_rs::Board;

esp_bootloader_esp_idf::esp_app_desc!();

extern crate alloc;

use esp_backtrace as _;
use esp_hal::{delay::Delay, main};
use esp_println::println;

#[main]
fn main() -> ! {
    println!("[example]------------------------------------");

    let _board = Board::init();

    // let camera = Ov5640::new();

    // let mut board = board
    //     .init(camera)
    //     .expect("[example] Failed to attach camera");

    let delay = Delay::new();
    loop {
        // match board.capture_frame() {
        //     Ok(frame) => {
        //         println!("[example] Frame captured! Size: {} bytes", frame.len());
        //         // TODO: Кодирование кадра в Base64
        //     }
        //     Err(_) => {
        //         println!("[example] Error capturing frame");
        //     }
        // }

        // println!("[example] loop");

        delay.delay_millis(2000);
    }
}
