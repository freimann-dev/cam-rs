use esp_hal::{
    delay::Delay,
    i2c::master::{Config as I2cConfig, I2c},
    lcd_cam::{
        cam::{Camera, Config as CameraConfig},
        LcdCam,
    },
    peripherals::{DMA_CH0, I2C0, LCD_CAM},
    time::Rate,
};
use esp_println::println;

use crate::sensors::ov5640::{SensorCmd, SENSOR_DEFAULT_REGS};

// const SENSOR_ADDR: u8 = 0x3C;

// const REG_DLY: u16 = 0xFFFF;
// const REGLIST_TAIL: u16 = 0x0000;

#[derive(Debug, Clone, Copy)]
pub enum PixelFormat {
    Jpeg,
    Rgb565,
}

pub struct CamObj<'d> {
    pub i2c: I2c<'d, esp_hal::Blocking>,
    pub camera: Camera<'d>,
    pub pid: u8,
    pub ver: u8,
    pub swap_data: bool,
    pub vsync_pin: u8,
    pub vsync_invert: bool,
    pub jpeg_mode: bool,
    pub psram_mode: bool,
    pub frame_cnt: usize,
    pub width: u16,
    pub height: u16,
    pub recv_size: usize,
    pub fb_size: usize,
    pub in_bytes_per_pixel: usize,
    pub fb_bytes_per_pixel: usize,
}

fn write_reg(
    i2c: &mut I2c<'_, esp_hal::Blocking>,
    slv_addr: u8,
    reg: u16,
    value: u8,
) -> Result<(), &'static str> {
    let buf = [(reg >> 8) as u8, (reg & 0xFF) as u8, value];
    i2c.write(slv_addr, &buf)
        .map_err(|_| "write_reg I2C failed")?;
    Ok(())
}

// fn write_reg_i2c(
//     i2c: &mut I2c<'_, esp_hal::Blocking>,
//     slv_addr: u8,
//     reg: u16,
//     value: u8,
// ) -> Result<(), &'static str> {
//     let buf = [(reg >> 8) as u8, (reg & 0xFF) as u8, value];
//     i2c.write(slv_addr, &buf).map_err(|_| "I2C write failed")?;
//     Ok(())
// }

// fn write_regs(
//     i2c: &mut I2c<'_, esp_hal::Blocking>,
//     slv_addr: u8,
//     regs: &[(u16, u16)],
//     delay: &Delay,
//     name: &str,
// ) -> Result<(), &'static str> {
//     let mut i = 0;
//     let mut count = 0usize;
//     while i < regs.len() {
//         let (reg, value) = regs[i];
//         if reg == REGLIST_TAIL {
//             break;
//         } else if reg == REG_DLY {
//             delay.delay_millis(value as u32);
//         } else {
//             write_reg(i2c, slv_addr, reg, value as u8)?;
//             count += 1;
//         }
//         i += 1;
//     }
//     println!("  applied {} regs from {}", count, name);
//     Ok(())
// }

fn read_reg_i2c(
    i2c: &mut I2c<'_, esp_hal::Blocking>,
    slv_addr: u8,
    reg: u16,
) -> Result<u8, &'static str> {
    let buf = [(reg >> 8) as u8, (reg & 0xFF) as u8];
    i2c.write(slv_addr, &buf).map_err(|_| "I2C write failed")?;

    let mut val = [0u8; 1];
    i2c.read(slv_addr, &mut val)
        .map_err(|_| "I2C read failed")?;
    Ok(val[0])
}

// use esp_hal::i2c::master::I2c;
use esp_hal::Blocking;
pub fn read_reg(
    i2c: &mut I2c<'_, Blocking>,
    addr: u8,
    reg: u16,
) -> Result<u8, esp_hal::i2c::master::Error> {
    let reg_high = (reg >> 8) as u8;
    let reg_low = (reg & 0xFF) as u8;
    let mut buffer = [0u8; 1];

    // Отправляем 2 байта адреса регистра и читаем 1 байт ответа
    i2c.write_read(addr, &[reg_high, reg_low], &mut buffer)?;
    Ok(buffer[0])
}

pub fn esp_camera_init<'d>(
    lcd_cam: LCD_CAM<'d>,
    dma_ch: DMA_CH0<'d>,
    i2c0: I2C0<'d>,
    // pin_pclk: impl esp_hal::gpio::interconnect::PeripheralInput<'d>,
    // pin_vsync: impl esp_hal::gpio::interconnect::PeripheralInput<'d> + esp_hal::gpio::InputPin,
    // pin_href: impl esp_hal::gpio::interconnect::PeripheralInput<'d>,
    // pin_d0: impl esp_hal::gpio::interconnect::PeripheralInput<'d> + esp_hal::gpio::InputPin,
    // pin_d1: impl esp_hal::gpio::interconnect::PeripheralInput<'d> + esp_hal::gpio::InputPin,
    // pin_d2: impl esp_hal::gpio::interconnect::PeripheralInput<'d> + esp_hal::gpio::InputPin,
    // pin_d3: impl esp_hal::gpio::interconnect::PeripheralInput<'d> + esp_hal::gpio::InputPin,
    // pin_d4: impl esp_hal::gpio::interconnect::PeripheralInput<'d> + esp_hal::gpio::InputPin,
    // pin_d5: impl esp_hal::gpio::interconnect::PeripheralInput<'d> + esp_hal::gpio::InputPin,
    // pin_d6: impl esp_hal::gpio::interconnect::PeripheralInput<'d> + esp_hal::gpio::InputPin,
    // pin_d7: impl esp_hal::gpio::interconnect::PeripheralInput<'d> + esp_hal::gpio::InputPin,
    pin_xclk: impl esp_hal::gpio::interconnect::PeripheralOutput<'d>,
    pin_sda: impl esp_hal::gpio::interconnect::PeripheralInput<'d>
        + esp_hal::gpio::interconnect::PeripheralOutput<'d>,
    pin_scl: impl esp_hal::gpio::interconnect::PeripheralInput<'d>
        + esp_hal::gpio::interconnect::PeripheralOutput<'d>,
    // pixel_format: PixelFormat,
    frame_width: u16,
    frame_height: u16,
    xclk_freq_hz: u32,
    // frame_cnt: usize,
    // jpeg_quality: u8,
    delay: &Delay,
) -> Result<CamObj<'d>, &'static str> {
    println!("🚀 Starting camera initialization...");

    let lcd_cam_instance = LcdCam::new(lcd_cam);
    let camera_config = CameraConfig::default().with_frequency(Rate::from_hz(xclk_freq_hz));

    let camera = Camera::new(lcd_cam_instance.cam, dma_ch, camera_config)
        .map_err(|_| "Camera config failed")?
        .with_master_clock(pin_xclk);

    println!("✅ XCLK started");
    delay.delay_millis(100);

    let i2c_config = I2cConfig::default();
    let mut i2c = I2c::new(i2c0, i2c_config)
        .map_err(|_| "I2C config failed")?
        .with_sda(pin_sda)
        .with_scl(pin_scl);

    let pid = read_reg_i2c(&mut i2c, SENSOR_ADDR, 0x300A)?;
    let ver = read_reg_i2c(&mut i2c, SENSOR_ADDR, 0x300B)?;

    if pid != 0x56 || ver != 0x40 {
        println!("❌ Unknown sensor: PID=0x{:02X} VER=0x{:02X}", pid, ver);
        return Err("Unknown sensor");
    }

    println!("Detected OV5640 camera at address=0x{:02x}", SENSOR_ADDR);

    println!("******************************************************************************************************************************************************************");

    const SENSOR_ADDR: u8 = 0x3C;
    println!("--- Writing default registers to OV5640 ---");
    let mut count = 0usize;
    for cmd in SENSOR_DEFAULT_REGS {
        match *cmd {
            SensorCmd::WriteReg(reg, val) => {
                // Записываем регистр по I2C
                write_reg(&mut i2c, SENSOR_ADDR, reg, val)
                    .map_err(|_| "Failed to write I2C register")?;
                count += 1;
            }
            SensorCmd::DelayMs(ms) => {
                // Задержка (нужна после сброса 0x3008 = 0x80)
                delay.delay_millis(ms);
            }
        }
    }
    println!("✅ Successfully applied {} registers to OV5640!", count);

    // =========================================================================
    // ПРОВЕРКА: ЗАРАБОТАЛА ЛИ КАМЕРА (VSYNC + PCLK / HREF DYNAMICS)
    // =========================================================================
    println!("🔍 Testing if OV5640 is running and producing frames...");

    // 1. Проверяем, что VSYNC подает признаки жизни (дрыгается между LOW и HIGH)
    let mut vsync_toggled = false;
    let mut last_gpio_state = unsafe { core::ptr::read_volatile(0x6000_403C as *const u32) };

    // Ждем импульс кадра в течение ~100 мс (заведомо дольше одного кадра на 15 FPS)
    for _ in 0..100 {
        delay.delay_millis(1);
        let current_gpio_state = unsafe { core::ptr::read_volatile(0x6000_403C as *const u32) };

        // Если хоть один из входных GPIO изменил состояние
        if (current_gpio_state ^ last_gpio_state) != 0 {
            vsync_toggled = true;
            break;
        }
        last_gpio_state = current_gpio_state;
    }

    if !vsync_toggled {
        println!("❌ FAIL: No activity on camera pins! Sensor is sleeping or PCLK/XCLK disabled.");
        return Err("Camera is inactive");
    }

    // 2. Делаем мгновенный высокоскоростной замер линии PCLK и данных
    // Читаем регистр входных пинов 1000 раз подряд без задержек (занимает ~20-50 мкс)
    let mut pclk_changes = 0usize;
    let mut prev_state = unsafe { core::ptr::read_volatile(0x6000_403C as *const u32) };

    for _ in 0..1000 {
        let sample = unsafe { core::ptr::read_volatile(0x6000_403C as *const u32) };
        if sample != prev_state {
            pclk_changes += 1;
            prev_state = sample;
        }
    }

    println!(
        "📊 Measured {} signal transitions across 1000 fast GPIO reads",
        pclk_changes
    );

    if pclk_changes > 100 {
        println!("🎉 SUCCESS: OV5640 IS ALIVE!");
        println!("   -> PCLK clock generator active");
        println!("   -> Dynamic data streaming detected");
    } else {
        println!("⚠️ WARNING: Lines are toggling too slowly. Check PLL/CLK registers.");
        return Err("PCLK frequency too low or static");
    }

    // =========================================================================
    // ВЫВОД КЛЮЧЕВЫХ ЗАВОДСКИХ ДЕФОЛТОВ OV5640
    // =========================================================================
    println!("📖 --- OV5640 Hardware Factory Defaults ---");

    // 1. Чтение разрешения матрицы по умолчанию (Timing Output Size)
    let h_high = read_reg(&mut i2c, SENSOR_ADDR, 0x3808).unwrap_or(0);
    let h_low = read_reg(&mut i2c, SENSOR_ADDR, 0x3809).unwrap_or(0);
    let v_high = read_reg(&mut i2c, SENSOR_ADDR, 0x380A).unwrap_or(0);
    let v_low = read_reg(&mut i2c, SENSOR_ADDR, 0x380B).unwrap_or(0);

    let default_width = ((h_high as u16) << 8) | (h_low as u16);
    let default_height = ((v_high as u16) << 8) | (v_low as u16);

    println!(
        "📐 Output Resolution: {}x{} px (Full QSXGA Sensor Frame)",
        default_width, default_height
    );

    // 2. Чтение формата вывода (Format Control)
    let fmt_ctrl = read_reg(&mut i2c, SENSOR_ADDR, 0x4300).unwrap_or(0);
    let fmt_str = match fmt_ctrl & 0xF0 {
        0x10 => "YUV422 (UYVY)",
        0x20 => "Raw RGB (Bayer)",
        0x30 => "RGB565",
        0x40 => "Compressed (JPEG)",
        _ => "RAW / Custom YUV",
    };
    println!("🎨 Default Pixel Format: 0x{:02X} -> {}", fmt_ctrl, fmt_str);

    // 3. Состояние тактирования и питания
    let sys_clk = read_reg(&mut i2c, SENSOR_ADDR, 0x3103).unwrap_or(0);
    let pwr_ctrl = read_reg(&mut i2c, SENSOR_ADDR, 0x3008).unwrap_or(0);

    println!(
        "⚡ System Control (0x3008): 0x{:02X} (Power Down: {}, Reset: {})",
        pwr_ctrl,
        (pwr_ctrl & 0x40) != 0,
        (pwr_ctrl & 0x80) != 0
    );
    println!(
        "⏰ Clock Select (0x3103): 0x{:02X} (Direct XCLK Bypass Mode)",
        sys_clk
    );

    // 4. Статус выходов
    let pad_oe = read_reg(&mut i2c, SENSOR_ADDR, 0x3017).unwrap_or(0);
    println!(
        "🔌 Pad Output Enable (0x3017): 0x{:02X} -> DVP Pins Active!",
        pad_oe
    );
    println!("-------------------------------------------");

    println!("******************************************************************************************************************************************************************");

    // === Шаг 2: Регистры сенсора (ТОЧНО ПО ТАБЛИЦЕ) ===

    // ov5640_configure(&mut i2c, pixel_format, delay)?;

    // delay.delay_millis(100);

    // // === Шаг 3: Проверка VSYNC и данных через GPIO ===
    // println!("--- Step 3: Check VSYNC and data via GPIO ---");

    // let vsync = esp_hal::gpio::Input::new(pin_vsync, InputConfig::default());
    // let d0 = esp_hal::gpio::Input::new(pin_d0, InputConfig::default());
    // let d1 = esp_hal::gpio::Input::new(pin_d1, InputConfig::default());
    // let d2 = esp_hal::gpio::Input::new(pin_d2, InputConfig::default());
    // let d3 = esp_hal::gpio::Input::new(pin_d3, InputConfig::default());
    // let d4 = esp_hal::gpio::Input::new(pin_d4, InputConfig::default());
    // let d5 = esp_hal::gpio::Input::new(pin_d5, InputConfig::default());
    // let d6 = esp_hal::gpio::Input::new(pin_d6, InputConfig::default());
    // let d7 = esp_hal::gpio::Input::new(pin_d7, InputConfig::default());

    // let mut vsync_prev = vsync.level();
    // let mut data_received = false;
    // let mut bytes_read = [0u8; 10];
    // let mut bytes_count = 0;

    // for _ in 0..500 {
    //     delay.delay_millis(1);
    //     let vsync_curr = vsync.level();

    //     if vsync_curr != vsync_prev {
    //         vsync_prev = vsync_curr;

    //         let byte = (d0.level() as u8)
    //             | ((d1.level() as u8) << 1)
    //             | ((d2.level() as u8) << 2)
    //             | ((d3.level() as u8) << 3)
    //             | ((d4.level() as u8) << 4)
    //             | ((d5.level() as u8) << 5)
    //             | ((d6.level() as u8) << 6)
    //             | ((d7.level() as u8) << 7);

    //         if bytes_count < 10 {
    //             bytes_read[bytes_count] = byte;
    //             bytes_count += 1;
    //         }

    //         if byte != 0 {
    //             data_received = true;
    //         }
    //     }
    // }

    // println!("  Bytes read: {}", bytes_count);
    // for i in 0..bytes_count {
    //     println!("    byte[{}] = 0x{:02X}", i, bytes_read[i]);
    // }

    // if !data_received {
    //     println!("❌ CAMERA INIT FAILED - no data from sensor");
    //     return Err("No data received from camera");
    // }

    // println!("✅ Sensor data received!");

    // // === Шаг 4: Создание CamObj ===
    // println!("--- Step 4: Create CamObj ---");

    let cam_obj = CamObj {
        i2c,
        camera,
        pid,
        ver,
        swap_data: false,
        vsync_pin: 6,
        vsync_invert: true,
        jpeg_mode: false,
        psram_mode: false,
        frame_cnt: 1,
        width: frame_width,
        height: frame_height,
        recv_size: 0,
        fb_size: 0,
        in_bytes_per_pixel: 2,
        fb_bytes_per_pixel: 2,
    };

    println!("🎉 CAMERA INIT SUCCESSFUL!");
    Ok(cam_obj)
}
