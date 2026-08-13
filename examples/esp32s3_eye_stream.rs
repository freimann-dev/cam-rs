#![no_std]
#![no_main]

use camera_rs::{
    platform::esp32s3::{DvpPins, Esp32S3Resources},
    sensor::ov5640::Ov5640Resources,
    CameraDriver,
};
use embassy_executor::Spawner;
use embassy_net::{tcp::TcpSocket, Config as NetConfig, Runner, Stack, StackResources};
use embassy_time::{Duration, Timer};
use embedded_io_async::Write;
use esp_backtrace as _;
use esp_hal::{
    clock::CpuClock,
    gpio::{Input, InputConfig, Level, Output, OutputConfig, Pull},
    interrupt::software::SoftwareInterruptControl,
    psram,
    rng::Rng,
    timer::timg::TimerGroup,
};
use esp_println::println;
use esp_radio::wifi::{sta::StationConfig, Config, ControllerConfig, Interface, WifiController};

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
    println!("[wifi] Starting connection task");
    loop {
        println!("[wifi] Connecting to SSID: {}...", WIFI_SSID);
        match controller.connect_async().await {
            Ok(info) => {
                println!("[wifi] ✅ Connected: {:?}", info);
                let reason = controller.wait_for_disconnect_async().await.ok();
                println!("[wifi] ⚠️ Disconnected: {:?}", reason);
            }
            Err(e) => {
                println!("[wifi] ❌ Failed to connect: {:?}, retrying...", e);
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
    println!("--------------------------------------------------");
    println!("[app] Bootstrapping camera firmware application");

    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::_240MHz);
    let peripherals = esp_hal::init(config);

    // Два хипа как в официальном примере
    esp_alloc::heap_allocator!(size: 64 * 1024);
    esp_alloc::heap_allocator!(size: 36 * 1024);

    // PSRAM для буферов камеры
    let _psram = psram::Psram::new(peripherals.PSRAM, psram::PsramConfig::default());
    unsafe {
        esp_alloc::HEAP.add_region(esp_alloc::HeapRegion::new(
            0x3D00_0000 as *mut u8,
            8 * 1024 * 1024,
            esp_alloc::MemoryCapability::External.into(),
        ));
    }

    println!(
        "[RAM] Total free heap: {} KB",
        esp_alloc::HEAP.free() / 1024
    );

    // Scheduler
    let timg0 = TimerGroup::new(peripherals.TIMG0);
    let sw_int = SoftwareInterruptControl::new(peripherals.SW_INTERRUPT);
    esp_rtos::start(timg0.timer0, sw_int.software_interrupt0);

    // Камера
    let dvp_pins = DvpPins {
        xclk: Output::new(peripherals.GPIO15, Level::Low, OutputConfig::default()),
        pclk: Input::new(
            peripherals.GPIO13,
            InputConfig::default().with_pull(Pull::None),
        ),
        vsync: Input::new(
            peripherals.GPIO6,
            InputConfig::default().with_pull(Pull::None),
        ),
        href: Input::new(
            peripherals.GPIO7,
            InputConfig::default().with_pull(Pull::None),
        ),
        data: [
            Input::new(
                peripherals.GPIO11,
                InputConfig::default().with_pull(Pull::None),
            ),
            Input::new(
                peripherals.GPIO9,
                InputConfig::default().with_pull(Pull::None),
            ),
            Input::new(
                peripherals.GPIO8,
                InputConfig::default().with_pull(Pull::None),
            ),
            Input::new(
                peripherals.GPIO10,
                InputConfig::default().with_pull(Pull::None),
            ),
            Input::new(
                peripherals.GPIO12,
                InputConfig::default().with_pull(Pull::None),
            ),
            Input::new(
                peripherals.GPIO18,
                InputConfig::default().with_pull(Pull::None),
            ),
            Input::new(
                peripherals.GPIO17,
                InputConfig::default().with_pull(Pull::None),
            ),
            Input::new(
                peripherals.GPIO16,
                InputConfig::default().with_pull(Pull::None),
            ),
        ],
    };

    let platform_res = Esp32S3Resources {
        lcd_cam: peripherals.LCD_CAM,
        dma_channel: peripherals.DMA_CH0,
        pins: dvp_pins,
    };

    let sensor_res = Ov5640Resources {
        i2c: peripherals.I2C0,
        reset_pin: Some(Output::new(
            peripherals.GPIO5,
            Level::High,
            OutputConfig::default(),
        )),
        pwdn_pin: Some(Output::new(
            peripherals.GPIO38,
            Level::Low,
            OutputConfig::default(),
        )),
    };

    let mut camera =
        CameraDriver::new(platform_res, sensor_res).expect("Failed to initialize camera driver");

    // WiFi
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

    println!("Wifi configured and started!");

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

    println!("[net] Waiting for DHCP IP address...");
    stack.wait_config_up().await;

    if let Some(config) = stack.config_v4() {
        println!("--------------------------------------------------");
        println!("[net] ✅ IP Address: {}", config.address);
        println!("[net] 🌐 Stream URL: http://{}/", config.address.address());
        println!("--------------------------------------------------");
    }

    let mut rx_buffer = [0u8; 1024];
    let mut tx_buffer = [0u8; 2048];

    loop {
        let mut socket = TcpSocket::new(*stack, &mut rx_buffer, &mut tx_buffer);
        socket.set_timeout(Some(Duration::from_secs(10)));

        if let Err(e) = socket.accept(80).await {
            println!("[http] Accept error: {:?}", e);
            continue;
        }

        println!("[http] Client connected!");

        let header = "HTTP/1.1 200 OK\r\n\
                      Content-Type: multipart/x-mixed-replace; boundary=frame\r\n\
                      Access-Control-Allow-Origin: *\r\n\
                      Connection: close\r\n\r\n";

        if socket.write_all(header.as_bytes()).await.is_err() {
            println!("[http] Failed to write HTTP header");
            continue;
        }

        loop {
            match camera.get_frame() {
                Ok(frame) => {
                    let frame_data = frame.data();
                    let part_header = alloc::format!(
                        "--frame\r\nContent-Type: image/jpeg\r\nContent-Length: {}\r\n\r\n",
                        frame_data.len()
                    );

                    if socket.write_all(part_header.as_bytes()).await.is_err() {
                        break;
                    }

                    if socket.write_all(frame_data).await.is_err() {
                        break;
                    }

                    if socket.write_all(b"\r\n").await.is_err() {
                        break;
                    }
                }
                Err(_) => {
                    Timer::after(Duration::from_millis(10)).await;
                }
            }
        }

        println!("[http] Client disconnected.");
    }
}
