extern crate alloc;

// use esp_alloc::{HEAP, HeapRegion, MemoryCapability};
use esp_hal::clock::CpuClock;
use esp_hal::delay::Delay;
use esp_hal::i2c::master::{Config as I2cConfig, I2c};
use esp_hal::lcd_cam::{
    LcdCam,
    cam::{Camera, Config as CamConfig},
};
use esp_hal::time::Rate;
use esp_println::println;

const I2C_ADDR: u8 = 0x3C;
const CHIP_ID_REG: u16 = 0x300A;
const REG_DLY: u16 = 0xFFFF;

static SENSOR_DEFAULT_REGS: &[(u16, u16)] = &[
    (0x3008, 0x82), // SYSTEM_CTROL0: software reset
    (REG_DLY, 10),  // delay 10ms
    (0x3008, 0x42), // power down
    // enable pll
    (0x3103, 0x13),
    // io direction
    (0x3017, 0xff),
    (0x3018, 0xff),
    (0x3108, 0xc3), // DRIVE_CAPABILITY
    // (0x3100, 0x21), // CLOCK_POL_CONTROL
    (0x4713, 0x02), // jpg mode select
    (0x5001, 0x83), // ISP_CONTROL_01 turn color matrix, awb and SDE
    // sys reset
    (0x3000, 0x20), // reset MCU
    (REG_DLY, 10),  // delay 10ms
    (0x3002, 0x1c),
    // clock enable
    (0x3004, 0xff),
    (0x3006, 0xc3),
    // isp control
    (0x5000, 0xa7),
    (0x5001, 0xa3), // ISP_CONTROL_01 +scaling?
    (0x5003, 0x08), // special_effect
    // unknown
    (0x370c, 0x02), // !!IMPORTANT
    (0x3634, 0x40), // !!IMPORTANT
    // AEC/AGC
    (0x3a02, 0x03),
    (0x3a03, 0xd8),
    (0x3a08, 0x01),
    (0x3a09, 0x27),
    (0x3a0a, 0x00),
    (0x3a0b, 0xf6),
    (0x3a0d, 0x04),
    (0x3a0e, 0x03),
    (0x3a0f, 0x30), // ae_level
    (0x3a10, 0x28), // ae_level
    (0x3a11, 0x60), // ae_level
    (0x3a13, 0x43),
    (0x3a14, 0x03),
    (0x3a15, 0xd8),
    (0x3a18, 0x00), // gainceiling
    (0x3a19, 0xf8), // gainceiling
    (0x3a1b, 0x30), // ae_level
    (0x3a1e, 0x26), // ae_level
    (0x3a1f, 0x14), // ae_level
    // vcm debug
    (0x3600, 0x08),
    (0x3601, 0x33),
    // 50/60Hz
    (0x3c01, 0xa4),
    (0x3c04, 0x28),
    (0x3c05, 0x98),
    (0x3c06, 0x00),
    (0x3c07, 0x08),
    (0x3c08, 0x00),
    (0x3c09, 0x1c),
    (0x3c0a, 0x9c),
    (0x3c0b, 0x40),
    (0x460c, 0x22), // disable jpeg footer
    // BLC
    (0x4001, 0x02),
    (0x4004, 0x02),
    // AWB
    (0x5180, 0xff),
    (0x5181, 0xf2),
    (0x5182, 0x00),
    (0x5183, 0x14),
    (0x5184, 0x25),
    (0x5185, 0x24),
    (0x5186, 0x09),
    (0x5187, 0x09),
    (0x5188, 0x09),
    (0x5189, 0x75),
    (0x518a, 0x54),
    (0x518b, 0xe0),
    (0x518c, 0xb2),
    (0x518d, 0x42),
    (0x518e, 0x3d),
    (0x518f, 0x56),
    (0x5190, 0x46),
    (0x5191, 0xf8),
    (0x5192, 0x04),
    (0x5193, 0x70),
    (0x5194, 0xF0),
    (0x5195, 0xF0),
    (0x5196, 0x03),
    (0x5197, 0x01),
    (0x5198, 0x04),
    (0x5199, 0x12),
    (0x519a, 0x04),
    (0x519b, 0x00),
    (0x519c, 0x06),
    (0x519d, 0x82),
    (0x519e, 0x38),
    // color matrix (Saturation)
    (0x5381, 0x1e),
    (0x5382, 0x5b),
    (0x5383, 0x08),
    (0x5384, 0x0a),
    (0x5385, 0x7e),
    (0x5386, 0x88),
    (0x5387, 0x7c),
    (0x5388, 0x6c),
    (0x5389, 0x10),
    (0x538a, 0x01),
    (0x538b, 0x98),
    // CIP control (Sharpness)
    (0x5300, 0x10), // sharpness
    (0x5301, 0x10), // sharpness
    (0x5302, 0x18), // sharpness
    (0x5303, 0x19), // sharpness
    (0x5304, 0x10),
    (0x5305, 0x10),
    (0x5306, 0x08), // denoise
    (0x5307, 0x16),
    (0x5308, 0x40),
    (0x5309, 0x10), // sharpness
    (0x530a, 0x10), // sharpness
    (0x530b, 0x04), // sharpness
    (0x530c, 0x06), // sharpness
    // GAMMA
    (0x5480, 0x01),
    (0x5481, 0x00),
    (0x5482, 0x1e),
    (0x5483, 0x3b),
    (0x5484, 0x58),
    (0x5485, 0x66),
    (0x5486, 0x71),
    (0x5487, 0x7d),
    (0x5488, 0x83),
    (0x5489, 0x8f),
    (0x548a, 0x98),
    (0x548b, 0xa6),
    (0x548c, 0xb8),
    (0x548d, 0xca),
    (0x548e, 0xd7),
    (0x548f, 0xe3),
    (0x5490, 0x1d),
    // Special Digital Effects (SDE) (UV adjust)
    (0x5580, 0x06), // enable brightness and contrast
    (0x5583, 0x40), // special_effect
    (0x5584, 0x10), // special_effect
    (0x5586, 0x20), // contrast
    (0x5587, 0x00), // brightness
    (0x5588, 0x00), // brightness
    (0x5589, 0x10),
    (0x558a, 0x00),
    (0x558b, 0xf8),
    (0x501d, 0x40), // enable manual offset of contrast
    // power on
    (0x3008, 0x02),
    // 50Hz
    (0x3c00, 0x04),
    (REG_DLY, 300),
];

#[derive(Debug)]
pub enum Esp32S3Error {
    NotInitialized,
    XclkConfigFailed,
    CaptureFailed,
    VsyncNotFound,
    PclkNotFound,
    DmaTimeout,
    DmaInitFailed,
    DmaError,
}

pub struct Esp32S3Eye {}

impl Esp32S3Eye {
    pub fn new() -> Self {
        println!("[board]------------------------------------------------");

        let config = esp_hal::Config::default().with_cpu_clock(CpuClock::_240MHz);
        let peripherals = esp_hal::init(config);

        let lcd_cam = LcdCam::new(peripherals.LCD_CAM);
        let cam_config = CamConfig::default().with_frequency(Rate::from_mhz(20));
        let _cam_clk = Camera::new(lcd_cam.cam, peripherals.DMA_CH0, cam_config)
            .expect("[board] Failed to init LCD_CAM peripheral")
            .with_master_clock(peripherals.GPIO15);

        let delay = Delay::new();
        delay.delay_millis(10);

        let mut i2c = I2c::new(
            peripherals.I2C0,
            I2cConfig::default().with_frequency(Rate::from_khz(100)),
        )
        .unwrap()
        .with_sda(peripherals.GPIO4)
        .with_scl(peripherals.GPIO5);

        let mut chip_id = [0u8; 2];
        let reg_bytes = [(CHIP_ID_REG >> 8) as u8, (CHIP_ID_REG & 0xFF) as u8];

        if i2c.write_read(I2C_ADDR, &reg_bytes, &mut chip_id).is_ok() {
            println!("[board] Chip ID: 0x{:02X}{:02X}", chip_id[0], chip_id[1]);
        } else {
            println!("[board] ERROR: Camera sensor not responding via I2C!");
        }

        for &(reg, val) in SENSOR_DEFAULT_REGS {
            if reg == REG_DLY {
                delay.delay_millis(val as u32);
            } else {
                let buf = [(reg >> 8) as u8, (reg & 0xFF) as u8, val as u8];
                if let Err(e) = i2c.write(I2C_ADDR, &buf) {
                    println!("[board] I2C write error at reg 0x{:04X}: {:?}", reg, e);
                }
            }
        }
        println!("[board] OV5640 setup complete!");

        //-------------------------------------------------------

        println!("[board] before psram_allocator");

        esp_alloc::heap_allocator!(size: 64 * 1024);
        // esp_alloc::psram_allocator!(peripherals.PSRAM, esp_hal::psram);

        println!("[board] after psram_allocator");

        let stats = esp_alloc::HEAP.stats();
        println!("{}", stats);
        //-------------------------------------------------------

        Self {}
    }
}
