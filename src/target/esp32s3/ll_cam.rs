use esp_hal::{
    gpio::{
        interconnect::{PeripheralInput, PeripheralOutput},
        DriveMode, InputConfig, InputSignal, OutputConfig, OutputSignal,
    },
    lcd_cam::{
        cam::{Camera, Config as CamConfig},
        LcdCam,
    },
    peripherals::LCD_CAM,
    time::Rate,
    Blocking,
};

pub struct CamObj<'d> {
    pub swap_data: bool,
    pub vsync_pin: u8,
    pub vsync_invert: bool,
    pub camera: Camera<'d>,
}

pub fn ll_cam_set_pin<'d>(
    pin_pclk: impl PeripheralInput<'d>,
    pin_vsync: impl PeripheralInput<'d>,
    pin_href: impl PeripheralInput<'d>,
    pin_d0: impl PeripheralInput<'d>,
    pin_d1: impl PeripheralInput<'d>,
    pin_d2: impl PeripheralInput<'d>,
    pin_d3: impl PeripheralInput<'d>,
    pin_d4: impl PeripheralInput<'d>,
    pin_d5: impl PeripheralInput<'d>,
    pin_d6: impl PeripheralInput<'d>,
    pin_d7: impl PeripheralInput<'d>,
    pin_xclk: Option<impl PeripheralOutput<'d>>,
) -> Result<(), &'static str> {
    let input_cfg = InputConfig::default();

    let p = pin_pclk.into();
    p.apply_input_config(&input_cfg);
    p.set_input_enable(true);
    InputSignal::CAM_PCLK.connect_to(&p);

    let v = pin_vsync.into();
    v.apply_input_config(&input_cfg);
    v.set_input_enable(true);
    InputSignal::CAM_V_SYNC.connect_to(&v);

    let h = pin_href.into();
    h.apply_input_config(&input_cfg);
    h.set_input_enable(true);
    InputSignal::CAM_H_ENABLE.connect_to(&h);

    let d0 = pin_d0.into();
    d0.apply_input_config(&input_cfg);
    d0.set_input_enable(true);
    InputSignal::CAM_DATA_0.connect_to(&d0);
    let d1 = pin_d1.into();
    d1.apply_input_config(&input_cfg);
    d1.set_input_enable(true);
    InputSignal::CAM_DATA_1.connect_to(&d1);
    let d2 = pin_d2.into();
    d2.apply_input_config(&input_cfg);
    d2.set_input_enable(true);
    InputSignal::CAM_DATA_2.connect_to(&d2);
    let d3 = pin_d3.into();
    d3.apply_input_config(&input_cfg);
    d3.set_input_enable(true);
    InputSignal::CAM_DATA_3.connect_to(&d3);
    let d4 = pin_d4.into();
    d4.apply_input_config(&input_cfg);
    d4.set_input_enable(true);
    InputSignal::CAM_DATA_4.connect_to(&d4);
    let d5 = pin_d5.into();
    d5.apply_input_config(&input_cfg);
    d5.set_input_enable(true);
    InputSignal::CAM_DATA_5.connect_to(&d5);
    let d6 = pin_d6.into();
    d6.apply_input_config(&input_cfg);
    d6.set_input_enable(true);
    InputSignal::CAM_DATA_6.connect_to(&d6);
    let d7 = pin_d7.into();
    d7.apply_input_config(&input_cfg);
    d7.set_input_enable(true);
    InputSignal::CAM_DATA_7.connect_to(&d7);

    if let Some(xclk) = pin_xclk {
        let output_cfg = OutputConfig::default().with_drive_mode(DriveMode::PushPull);
        let x = xclk.into();
        x.apply_output_config(&output_cfg);
        x.set_output_enable(true);
        OutputSignal::CAM_CLK.connect_to(&x);
    }

    Ok(())
}

pub fn ll_cam_config<'d>(
    lcd_cam: LcdCam<'d, Blocking>,
    dma_ch: impl esp_hal::dma::DmaChannel + esp_hal::dma::RxChannelFor<LCD_CAM<'d>>,
    xclk_freq_hz: u32,
) -> Result<Camera<'d>, &'static str> {
    let cam_config = CamConfig::default().with_frequency(Rate::from_hz(xclk_freq_hz));

    Camera::new(lcd_cam.cam, dma_ch, cam_config).map_err(|_| "Camera config failed")
}
