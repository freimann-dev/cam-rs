#![no_std]
#![no_main]

// extern crate alloc;

// use esp_alloc as _;
use esp_backtrace as _;
use esp_hal::{
    clock::CpuClock,
    delay::Delay,
    main,
    psram::{Psram, PsramConfig},
    time::Duration,
};
use esp_println::println;

esp_bootloader_esp_idf::esp_app_desc!();

#[main]
fn main() -> ! {
    esp_println::logger::init_logger(log::LevelFilter::Info);
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);
    let psram_config = PsramConfig::default();
    let psram = Psram::new(peripherals.PSRAM, psram_config);
    let (base_ptr, total_size) = psram.raw_parts();
    println!("psram: base_ptr {:p}, total_size {}", base_ptr, total_size);

    let delay = Delay::new();

    loop {
        delay.delay(Duration::from_secs(2));
    }
}
