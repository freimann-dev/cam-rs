use crate::platform::CameraPlatform;
use esp_hal::{
    gpio::{Input, Output},
    peripherals::{DMA_CH0, LCD_CAM},
};

pub struct DvpPins<'d> {
    pub xclk: Output<'d>,
    pub pclk: Input<'d>,
    pub vsync: Input<'d>,
    pub href: Input<'d>,
    pub data: [Input<'d>; 8],
}

pub struct Esp32S3Resources<'d> {
    pub lcd_cam: LCD_CAM<'d>,
    pub dma_channel: DMA_CH0<'d>,
    pub pins: DvpPins<'d>,
}

pub struct Esp32S3Platform<'d> {
    _lcd_cam: LCD_CAM<'d>,
    _dma: DMA_CH0<'d>,
    _pins: DvpPins<'d>,
}

impl<'d> Esp32S3Platform<'d> {
    pub fn new(resources: Esp32S3Resources<'d>) -> Result<Self, ()> {
        Ok(Self {
            _lcd_cam: resources.lcd_cam,
            _dma: resources.dma_channel,
            _pins: resources.pins,
        })
    }
}

impl<'d> CameraPlatform for Esp32S3Platform<'d> {
    fn start_capture(&mut self) -> Result<(), ()> {
        Ok(())
    }

    fn get_current_buffer<'a>(&'a mut self) -> Result<&'a [u8], ()> {
        Ok(&[])
    }
}
