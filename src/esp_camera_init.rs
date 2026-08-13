use esp_hal::{
    delay::Delay,
    gpio::InputPin,
    i2c::master::{Config as I2cConfig, I2c},
    lcd_cam::{
        cam::{Camera, Config as CameraConfig},
        LcdCam,
    },
    peripherals::{DMA_CH0, I2C0, LCD_CAM},
    time::{Instant, Rate},
    Blocking,
};
use esp_println::println;

const SENSOR_I2C_ADDR: u8 = 0x3C;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SensorType {
    Ov5640,
}

impl SensorType {
    pub fn name(&self) -> &'static str {
        match self {
            SensorType::Ov5640 => "OV5640",
        }
    }
}

pub struct CamObj<'d> {
    pub i2c: I2c<'d, Blocking>,
    pub camera: Camera<'d>,
    pub sensor_type: SensorType,
    pub width: u16,
    pub height: u16,
}

fn write_reg(i2c: &mut I2c<'_, Blocking>, reg: u16, value: u8) -> Result<(), &'static str> {
    i2c.write(
        SENSOR_I2C_ADDR,
        &[(reg >> 8) as u8, (reg & 0xFF) as u8, value],
    )
    .map_err(|_| "I2C write failed")
}

fn read_reg(i2c: &mut I2c<'_, Blocking>, reg: u16) -> Result<u8, &'static str> {
    let mut val = [0u8; 1];
    i2c.write_read(
        SENSOR_I2C_ADDR,
        &[(reg >> 8) as u8, (reg & 0xFF) as u8],
        &mut val,
    )
    .map_err(|_| "I2C read failed")?;
    Ok(val[0])
}

fn verify_pclk_active(pclk_pin: &impl InputPin) -> Result<(), &'static str> {
    let initial_state = pclk_pin.is_high();
    let start = Instant::now();

    while start.elapsed().as_micros() < 1000 {
        if pclk_pin.is_high() != initial_state {
            return Ok(());
        }
    }

    Err("PCLK line static (no clock detected)")
}

pub fn esp_camera_init<'d>(
    lcd_cam: LCD_CAM<'d>,
    dma_ch: DMA_CH0<'d>,
    i2c0: I2C0<'d>,
    pin_xclk: impl esp_hal::gpio::interconnect::PeripheralOutput<'d>,
    pin_pclk: &impl InputPin,
    pin_sda: impl esp_hal::gpio::interconnect::PeripheralInput<'d>
        + esp_hal::gpio::interconnect::PeripheralOutput<'d>,
    pin_scl: impl esp_hal::gpio::interconnect::PeripheralInput<'d>
        + esp_hal::gpio::interconnect::PeripheralOutput<'d>,
    frame_width: u16,
    frame_height: u16,
    xclk_freq_hz: u32,
    delay: &Delay,
) -> Result<CamObj<'d>, &'static str> {
    println!(
        "[INFO] camera: Initializing master clock (XCLK) at {} Hz...",
        xclk_freq_hz
    );

    let lcd_cam_instance = LcdCam::new(lcd_cam);
    let camera_config = CameraConfig::default().with_frequency(Rate::from_hz(xclk_freq_hz));
    let camera = Camera::new(lcd_cam_instance.cam, dma_ch, camera_config)
        .map_err(|_| "Camera peripheral init failed")?
        .with_master_clock(pin_xclk);

    delay.delay_millis(100);

    let mut i2c = I2c::new(i2c0, I2cConfig::default())
        .map_err(|_| "I2C init failed")?
        .with_sda(pin_sda)
        .with_scl(pin_scl);

    let pid = read_reg(&mut i2c, 0x300A)?;
    let ver = read_reg(&mut i2c, 0x300B)?;

    let sensor_type = match (pid, ver) {
        (0x56, 0x40) => SensorType::Ov5640,
        (p, v) => {
            println!(
                "[ERROR] camera: Unsupported sensor ID: 0x{:02X}{:02X}",
                p, v
            );
            return Err("Unsupported camera sensor");
        }
    };

    println!(
        "[INFO] camera: Detected {} sensor (ID: 0x{:02X}{:02X}) at I2C 0x{:02X}",
        sensor_type.name(),
        pid,
        ver,
        SENSOR_I2C_ADDR
    );

    match sensor_type {
        SensorType::Ov5640 => {
            write_reg(&mut i2c, 0x3017, 0xFF)?;
            println!("[DEBUG] camera: Enabled DVP parallel output (REG 0x3017 = 0xFF)");
        }
    }

    verify_pclk_active(pin_pclk)?;
    println!("[DEBUG] camera: PCLK line active (edge detected)");

    let def_w = ((read_reg(&mut i2c, 0x3808)? as u16) << 8) | (read_reg(&mut i2c, 0x3809)? as u16);
    let def_h = ((read_reg(&mut i2c, 0x380A)? as u16) << 8) | (read_reg(&mut i2c, 0x380B)? as u16);

    println!(
        "[INFO] camera: Initialization complete. Native sensor frame: {}x{} px",
        def_w, def_h
    );

    Ok(CamObj {
        i2c,
        camera,
        sensor_type,
        width: frame_width,
        height: frame_height,
    })
}
