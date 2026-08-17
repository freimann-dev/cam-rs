#![no_std]
#![no_main]

use camera_rs::Driver;
use camera_rs::board::esp32s3::Esp32S3;
use camera_rs::camera::ov5640::Ov5640;
use embassy_executor::Spawner;
use embassy_net::{Config as NetConfig, Runner, Stack, StackResources, tcp::TcpSocket};
use embassy_time::{Duration, Timer};
use embedded_io_async::Write;
use esp_backtrace as _;
use esp_hal::{
    clock::CpuClock, interrupt::software::SoftwareInterruptControl, rng::Rng,
    timer::timg::TimerGroup,
};
use esp_println::println;
use esp_radio::wifi::{Config, ControllerConfig, Interface, WifiController, sta::StationConfig};

extern crate alloc;

const WIFI_SSID: &str = "BALTICOM2G_3124301";
const WIFI_PASS: &str = "zeyhd4xpgmte";

esp_bootloader_esp_idf::esp_app_desc!();

macro_rules! mk_static {
    ($t:ty, $val:expr) => {{
        static STATIC_CELL: static_cell::StaticCell<$t> = static_cell::StaticCell::new();
        #[deny(unused_attributes)]
        let x = STATIC_CELL.uninit().write(($val));
        x
    }};
}

#[embassy_executor::task]
async fn connection_task(mut controller: WifiController<'static>) {
    println!("[example] Starting connection task");
    loop {
        println!("[example] Connecting to SSID: {}...", WIFI_SSID);
        match controller.connect_async().await {
            Ok(info) => {
                println!("[example] ✅ Connected: {:?}", info);
                let reason = controller.wait_for_disconnect_async().await.ok();
                println!("[example] ⚠️ Disconnected: {:?}", reason);
            }
            Err(e) => {
                println!("[example] ❌ Failed to connect: {:?}, retrying...", e);
            }
        }
        Timer::after(Duration::from_secs(5)).await;
    }
}

#[embassy_executor::task]
async fn net_task(mut runner: Runner<'static, Interface>) -> ! {
    runner.run().await
}

#[esp_rtos::main]
async fn main(spawner: Spawner) -> ! {
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::_240MHz);
    let peripherals = esp_hal::init(config);
    let timg0 = TimerGroup::new(peripherals.TIMG0);
    let sw_int = SoftwareInterruptControl::new(peripherals.SW_INTERRUPT);
    esp_rtos::start(timg0.timer0, sw_int.software_interrupt0);

    esp_alloc::heap_allocator!(size: 72 * 1024);
    let psram =
        esp_hal::psram::Psram::new(peripherals.PSRAM, esp_hal::psram::PsramConfig::default());
    let (psram_ptr, psram_size) = psram.raw_parts();
    println!(
        "[example] PSRAM at {:p} ({} KB total)",
        psram_ptr,
        psram_size / 1024
    );

    // let psram_frame_buf: &'static mut [u8] =
    //     unsafe { core::slice::from_raw_parts_mut(psram_ptr, 512 * 1024) };

    let board = Esp32S3::new(
        peripherals.LCD_CAM,
        peripherals.DMA_CH0,
        peripherals.GPIO15, // xclk
        peripherals.GPIO13, // pclk для camera
        peripherals.GPIO6,  // vsync
        peripherals.GPIO7,  // href
        (
            peripherals.GPIO11, // data0
            peripherals.GPIO9,  // data1
            peripherals.GPIO8,  // data2
            peripherals.GPIO10, // data3
            peripherals.GPIO12, // data4
            peripherals.GPIO18, // data5
            peripherals.GPIO17, // data6
            peripherals.GPIO16, // data7
        ),
        // psram_frame_buf,
    );

    let camera = Ov5640::new(peripherals.I2C0, peripherals.GPIO4, peripherals.GPIO5)
        .expect("Failed to create OV5640 instance");

    let mut _driver = match Driver::new(board, camera) {
        Ok(driver) => driver,
        Err(err) => {
            println!("[example] Camera initialization failed: {:?}", err);

            loop {
                embassy_time::Timer::after_secs(1).await;
            }
        }
    };

    let station_config = Config::Station(
        StationConfig::default()
            .with_ssid(WIFI_SSID)
            .with_password(WIFI_PASS.into()),
    );

    let wifi_interface = esp_radio::wifi::Interface::station();
    let controller = esp_radio::wifi::WifiController::new(
        peripherals.WIFI,
        ControllerConfig::default().with_initial_config(station_config),
    )
    .unwrap();

    let net_config = NetConfig::dhcpv4(Default::default());
    let rng = Rng::new();
    let seed = (rng.random() as u64) << 32 | rng.random() as u64;

    let (stack, runner) = embassy_net::new(
        wifi_interface,
        net_config,
        mk_static!(StackResources<5>, StackResources::<5>::new()),
        seed,
    );

    let stack = mk_static!(Stack<'static>, stack);

    spawner.spawn(connection_task(controller).unwrap());
    spawner.spawn(net_task(runner).unwrap());

    stack.wait_config_up().await;

    if let Some(config) = stack.config_v4() {
        println!("[example] ✅ IP Address: {}", config.address);
        println!(
            "[example] 🌐 Stream URL: http://{}/",
            config.address.address()
        );
    }

    let mut rx_buffer = [0u8; 1024];
    let mut tx_buffer = [0u8; 2048];

    loop {
        let mut socket = TcpSocket::new(*stack, &mut rx_buffer, &mut tx_buffer);
        socket.set_timeout(Some(Duration::from_secs(10)));

        if let Err(e) = socket.accept(80).await {
            println!("[example] Accept error: {:?}", e);
            continue;
        }

        println!("[example] Client connected!");

        let header = "HTTP/1.1 200 OK\r\n\
              Content-Type: multipart/x-mixed-replace; boundary=frame\r\n\r\n";
        socket.write_all(header.as_bytes()).await.ok();
        println!("[example] Headers sent! Starting stream loop...");

        // loop {
        //     println!("[example] Requesting capture()...");

        //     match driver.capture() {
        //         Ok(frame) => {
        //             println!("[example] Frame captured! Size: {} bytes", frame.len());

        //             let frame_header = alloc::format!(
        //                 "--frame\r\nContent-Type: image/jpeg\r\nContent-Length: {}\r\n\r\n",
        //                 frame.len()
        //             );

        //             if socket.write_all(frame_header.as_bytes()).await.is_err() {
        //                 break;
        //             }
        //             if socket.write_all(frame).await.is_err() {
        //                 break;
        //             }
        //             if socket.write_all(b"\r\n").await.is_err() {
        //                 break;
        //             }
        //         }
        //         Err(e) => {
        //             println!("[example] Capture error: {:?}", e);
        //             embassy_time::Timer::after_millis(50).await;
        //         }
        //     }
        // }

        println!("[http] Client disconnected.");
    }
}
