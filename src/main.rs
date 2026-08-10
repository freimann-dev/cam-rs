#![no_std]
#![no_main]

use esp_alloc as _;
use esp_backtrace as _;
use esp_hal::delay::Delay;
use esp_hal::{init, main, Config};
use esp_println::println;

use esp32_camera_rs::driver::esp_camera_init::esp_camera_init;

#[global_allocator]
static ALLOCATOR: esp_alloc::EspHeap = esp_alloc::EspHeap::empty();

fn init_heap() {
    const HEAP_SIZE: usize = 64 * 1024;
    static mut HEAP: core::mem::MaybeUninit<[u8; HEAP_SIZE]> = core::mem::MaybeUninit::uninit();
    unsafe {
        ALLOCATOR.init(core::ptr::addr_of_mut!(HEAP) as *mut u8, HEAP_SIZE);
    }
}

esp_bootloader_esp_idf::esp_app_desc!();

#[main]
fn main() -> ! {
    init_heap();

    let peripherals = init(Config::default());
    let delay = Delay::new();

    let _cam_obj = match esp_camera_init(
        // &mut peripherals, // аргумент #1
        peripherals.LCD_CAM,
        peripherals.DMA_CH0,
        peripherals.I2C0,
        // peripherals.GPIO13, // pclk (не Option)
        // peripherals.GPIO6,  // vsync
        // peripherals.GPIO7,  // href
        // peripherals.GPIO11, // d0
        // peripherals.GPIO9,  // d1
        // peripherals.GPIO8,  // d2
        // peripherals.GPIO10, // d3
        // peripherals.GPIO12, // d4
        // peripherals.GPIO18, // d5
        // peripherals.GPIO17, // d6
        // peripherals.GPIO16, // d7
        peripherals.GPIO15, // xclk
        peripherals.GPIO4,  // sda (не Option)
        peripherals.GPIO5,  // scl (не Option)
        // PixelFormat::Jpeg,
        320,
        240,
        20_000_000,
        // 1,
        // 20,
        &delay,
    ) {
        Ok(obj) => obj,
        Err(e) => {
            println!("❌ Camera init failed: {}", e);
            loop {}
        }
    };

    // println!("🏁 Initialization complete. Sensor registers applied.");
    // println!("   Next step: verify data output (VSYNC/HREF/PCLK) with oscilloscope,");
    // println!("   then add DMA stage separately.");

    loop {
        delay.delay_millis(1000);
    }
}
