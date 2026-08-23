extern crate alloc;

use crate::{BoardError, BoardResult, Frame};

use esp_hal::clock::CpuClock;
use esp_hal::delay::Delay;
use esp_hal::gpio::DriveMode;
use esp_hal::i2c::master::{Config as I2cConfig, I2c};
use esp_hal::ledc::{
    LSGlobalClkSource, Ledc, LowSpeed,
    channel::{self, ChannelIFace},
    timer::{self, TimerIFace},
};
use esp_hal::time::Rate;
use esp_println::println;

// Распиновка AI Thinker ESP32-CAM для OV2640
const I2C_ADDR: u8 = 0x30; // I2C адрес OV2640

// Регистры для чтения Chip ID OV2640 (требуется переключение на Bank 0)
const REG_BANK_SEL: u8 = 0xFF;
const REG_BANK0: u8 = 0x00;
const CHIP_ID_HIGH_REG: u8 = 0x0A; // Ожидаемое значение: 0x26
// const CHIP_ID_LOW_REG: u8 = 0x0B; // Ожидаемое значение: 0x42

// Упрощенная таблица инициализации OV2640.
// Для полноценной работы (JPEG, нужное разрешение) необходимо использовать
// полную таблицу из официального драйвера esp32-camera (sensor_regs.h).
static SENSOR_DEFAULT_REGS: &[(u8, u8)] = &[
    (REG_BANK_SEL, REG_BANK0), // Переключаемся на Bank 0
    (0x12, 0x80),              // COM7: Software reset
                               // (Здесь должны быть остальные регистры настройки формата, частоты и т.д.)
];

#[derive(Debug)]
pub enum Esp32CamError {
    NotInitialized,
    I2cConfigFailed,
    SensorNotFound,
    I2sConfigFailed,
}

pub struct Esp32Cam {
    // В реальном коде здесь будут храниться дескрипторы I2C и I2S.
    // Обратите внимание на управление временем жизни (lifetimes) в esp-hal v1.x.
    // Часто используется тип AnyI2c<'static> для упрощения хранения.
}

impl Esp32Cam {
    pub fn new() -> Result<Self, Esp32CamError> {
        println!("[board] Initializing ESP32-CAM...");

        let config = esp_hal::Config::default().with_cpu_clock(CpuClock::_240MHz);
        let peripherals = esp_hal::init(config);

        let mut ledc = Ledc::new(peripherals.LEDC);
        ledc.set_global_slow_clock(LSGlobalClkSource::APBClk);
        let mut lstimer0 = ledc.timer::<LowSpeed>(timer::Number::Timer0);
        lstimer0
            .configure(timer::config::Config {
                duty: timer::config::Duty::Duty2Bit,
                clock_source: timer::LSClockSource::APBClk,
                frequency: Rate::from_mhz(20),
            })
            .expect("LEDC timer config failed");
        let mut channel0 = ledc.channel(channel::Number::Channel0, peripherals.GPIO0);
        channel0
            .configure(channel::config::Config {
                timer: &lstimer0,
                duty_pct: 50,
                drive_mode: DriveMode::PushPull,
            })
            .expect("LEDC channel config failed");

        //---------------------------------------------------------------------

        // let _pwdn = Output::new(peripherals.GPIO32, Level::Low, OutputConfig::default());
        let delay = Delay::new();
        delay.delay_millis(10);

        // 1. Инициализация I2C для OV2640 (SDA: GPIO 26, SCL: GPIO 27)
        let mut i2c = I2c::new(
            peripherals.I2C0,
            I2cConfig::default().with_frequency(Rate::from_khz(100)),
        )
        .map_err(|_| Esp32CamError::I2cConfigFailed)?
        .with_sda(peripherals.GPIO26)
        .with_scl(peripherals.GPIO27);

        println!("[board] Scanning I2C bus...");
        for addr in 0x08..0x78u8 {
            if i2c.write(addr, &[]).is_ok() {
                println!("[board]   Found device at 0x{:02X}", addr);
            }
        }
        println!("[board] I2C scan done.");

        let reg_bytes = [0x30, 0x0A]; // 16-битный адрес регистра Chip ID
        let mut chip_id = [0u8; 2];

        if i2c.write_read(0x3C, &reg_bytes, &mut chip_id).is_ok() {
            println!("[board] Chip ID: 0x{:02X}{:02X}", chip_id[0], chip_id[1]);

            // if chip_id[0] == 0x56 && chip_id[1] == 0x40 {
            //     println!("[board] CONFIRMED: This is an OV5640 sensor.");
            // } else {
            //     println!(
            //         "[board] Sensor responded, but Chip ID is 0x{:02X}{:02X} (expected 0x5640).",
            //         chip_id[0], chip_id[1]
            //     );
            // }
        } else {
            println!("[board] ERROR: Sensor at 0x3C did not respond to register read.");
            println!("[board] This usually means XCLK (GPIO 0) is not stable.");
        }

        let _ = i2c.write(I2C_ADDR, &[REG_BANK_SEL, REG_BANK0]);

        let mut chip_id = [0u8; 2];
        if i2c
            .write_read(I2C_ADDR, &[CHIP_ID_HIGH_REG], &mut chip_id)
            .is_ok()
        {
            println!(
                "[board] OV2640 Chip ID: 0x{:02X}{:02X}",
                chip_id[0], chip_id[1]
            );
            if chip_id[0] != 0x26 || chip_id[1] != 0x42 {
                println!("[board] WARNING: Unexpected Chip ID! Check wiring.");
            }
        } else {
            println!("[board] ERROR: Camera sensor not responding via I2C!");
            return Err(Esp32CamError::SensorNotFound);
        }

        for &(reg, val) in SENSOR_DEFAULT_REGS {
            if reg == 0x12 && val == 0x80 {
                delay.delay_millis(10); // Задержка после сброса
            } else {
                let buf = [reg, val];
                if let Err(e) = i2c.write(I2C_ADDR, &buf) {
                    println!("[board] I2C write error at reg 0x{:02X}: {:?}", reg, e);
                }
            }
        }
        println!("[board] OV2640 basic I2C setup complete!");

        // 4. Настройка захвата данных (КЛЮЧЕВОЕ ОТЛИЧИЕ ОТ ESP32-S3)
        // Классический ESP32 НЕ имеет периферии LCD_CAM.
        // Захват данных осуществляется через I2S в параллельном режиме.
        println!("[board] NOTE: Data capture requires I2S parallel mode setup.");
        println!("[board] Pins: XCLK=0, PCLK=22, VSYNC=25, HREF=23");
        println!("[board] Data: D0=5, D1=18, D2=19, D3=21, D4=36, D5=39, D6=34, D7=35");

        /*
        ЗДЕСЬ ДОЛЖНА БЫТЬ ИНИЦИАЛИЗАЦИЯ I2S:
        В esp-hal для классического ESP32 это требует настройки I2S в режим
        параллельного приема (часто через нестандартные конфигурации или прямую
        запись в регистры, так как высокоуровневый API "Camera" для ESP32 еще
        не стабилизирован, в отличие от ESP32-S3).

        Если вам нужен стабильный захват кадров (особенно в JPEG),
        настоятельно рекомендуется перейти на стек:
        - esp-idf-hal
        - esp32-camera (официальный C-драйвер, обернутый через esp-idf-sys)
        */

        Ok(Self {})
    }

    pub fn capture(&mut self) -> BoardResult<Frame> {
        Err(BoardError::CaptureFailed)
    }
}
