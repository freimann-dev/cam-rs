use crate::target::esp32s3::ll_cam::{ll_cam_config, ll_cam_set_pin, CamObj};
use esp_hal::{lcd_cam::LcdCam, peripherals::Peripherals};
use esp_println::println;

pub fn cam_init(
    peripherals: Peripherals,
    xclk_freq_hz: u32,
) -> Result<CamObj<'static>, &'static str> {
    if xclk_freq_hz == 0 {
        return Err("config pointer is invalid");
    }

    ll_cam_set_pin(
        peripherals.GPIO13,
        peripherals.GPIO6,
        peripherals.GPIO7,
        peripherals.GPIO11,
        peripherals.GPIO9,
        peripherals.GPIO8,
        peripherals.GPIO10,
        peripherals.GPIO12,
        peripherals.GPIO18,
        peripherals.GPIO17,
        peripherals.GPIO16,
        Some(peripherals.GPIO15),
    )?;

    let lcd_cam = LcdCam::new(peripherals.LCD_CAM);
    let camera = ll_cam_config(lcd_cam, peripherals.DMA_CH0, xclk_freq_hz)?;

    let cam_obj = CamObj {
        swap_data: false,
        vsync_pin: 6,
        vsync_invert: true,
        camera,
    };

    println!("cam init ok");
    Ok(cam_obj)
}
