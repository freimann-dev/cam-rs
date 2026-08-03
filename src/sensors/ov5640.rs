use esp_hal::lcd_cam::cam::{Config, VhdeMode};

pub const OV5640_ADDR: u8 = 0x3C;
pub const OV5640_ID: u16 = 0x5640;

pub struct OV5640;

impl OV5640 {
    /// Пины для ESP32-S3-EYE
    pub const PINS: SensorPins = SensorPins {
        sda: 4,
        scl: 5,
        xclk: 15,
        pclk: 13,
        vsync: 6,
        href: 7,
        d0: 11,
        d1: 9,
        d2: 8,
        d3: 10,
        d4: 12,
        d5: 18,
        d6: 17,
        d7: 16,
    };

    pub const INIT_REGS: &'static [(u16, u8)] = &[
        // 1. Сброс
        (0x3103, 0x11),
        (0x3008, 0x82),
        (0x3008, 0x42),
        // 2. Базовые настройки и PLL (КРИТИЧНО для генерации PCLK из 20 МГц XCLK)
        (0x3017, 0x00),
        (0x3018, 0x44),
        (0x3019, 0x02),
        (0x3034, 0x1A),
        (0x3035, 0x21),
        (0x3036, 0x69),
        (0x3037, 0x10), // PLL для 20 МГц
        (0x3108, 0x01),
        (0x3630, 0x36),
        (0x3631, 0x0e),
        (0x3632, 0xe2),
        (0x3633, 0x12),
        (0x3621, 0xe0),
        (0x3704, 0xa0),
        (0x3703, 0x5a),
        (0x3715, 0x78),
        (0x3717, 0x01),
        (0x370b, 0x60),
        (0x3705, 0x1a),
        (0x3905, 0x02),
        (0x3906, 0x10),
        (0x3901, 0x0a),
        (0x3731, 0x12),
        (0x3600, 0x08),
        (0x3601, 0x33),
        (0x302d, 0x60),
        (0x3620, 0x52),
        (0x371b, 0x20),
        (0x471c, 0x50),
        (0x3a13, 0x43),
        (0x3a18, 0x00),
        (0x3a19, 0xf8),
        (0x3635, 0x13),
        (0x3636, 0x03),
        (0x3634, 0x40),
        (0x3622, 0x01),
        (0x3c01, 0x34),
        (0x3c04, 0x28),
        (0x3c05, 0x98),
        (0x3c06, 0x00),
        (0x3c07, 0x08),
        (0x3c08, 0x00),
        (0x3c09, 0x1c),
        (0x3c0a, 0x9c),
        (0x3c0b, 0x40),
        // 3. Формат JPEG
        (0x4300, 0x30),
        (0x501f, 0x00),
        (0x4400, 0x04),
        (0x4407, 0x04),
        (0x440e, 0x00),
        (0x460b, 0x35),
        (0x460c, 0x22),
        (0x3824, 0x02),
        (0x5000, 0xa7),
        (0x5001, 0xa3),
        // 4. Разрешение VGA (640x480) - как в вашем рабочем Arduino коде
        (0x3800, 0x00),
        (0x3801, 0x00),
        (0x3802, 0x00),
        (0x3803, 0x04),
        (0x3804, 0x0a),
        (0x3805, 0x3f),
        (0x3806, 0x07),
        (0x3807, 0x9b),
        (0x3808, 0x02),
        (0x3809, 0x80), // Width: 640
        (0x380a, 0x01),
        (0x380b, 0xe0), // Height: 480
        (0x380c, 0x07),
        (0x380d, 0xb0),
        (0x380e, 0x03),
        (0x380f, 0xd8),
        (0x3810, 0x00),
        (0x3811, 0x10),
        (0x3812, 0x00),
        (0x3813, 0x06),
        (0x3814, 0x31),
        (0x3815, 0x31),
        // 5. Vflip и Hmirror (как в вашем Arduino коде)
        (0x3820, 0x41),
        (0x3821, 0x07),
        // 6. Запуск стриминга
        (0x3503, 0x00),
        (0x4202, 0x00),
    ];

    /// Конфигурация DMA для этого сенсора
    pub fn dma_config() -> Config {
        Config::default()
            .with_vh_de_mode(VhdeMode::VsyncHsync)
            .with_invert_vsync(true)
            .with_invert_h_enable(true)
            .with_invert_pixel_clock(true)
    }
}

pub struct SensorPins {
    pub sda: u8,
    pub scl: u8,
    pub xclk: u8,
    pub pclk: u8,
    pub vsync: u8,
    pub href: u8,
    pub d0: u8,
    pub d1: u8,
    pub d2: u8,
    pub d3: u8,
    pub d4: u8,
    pub d5: u8,
    pub d6: u8,
    pub d7: u8,
}
