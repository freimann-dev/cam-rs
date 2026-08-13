pub mod ov5640;

use crate::platform::CameraPlatform;

#[derive(Debug, Clone, Copy)]
pub struct SensorSettings {
    pub width: u16,
    pub height: u16,
}

pub trait CameraSensor {
    fn name(&self) -> &'static str;
    fn detect(&self, platform: &mut dyn CameraPlatform) -> Result<(), ()>;
    fn init(&mut self, platform: &mut dyn CameraPlatform) -> Result<(), ()>;
    fn get_resolution(&self, platform: &mut dyn CameraPlatform) -> Result<(u16, u16), ()>;
    fn get_settings(&self, platform: &mut dyn CameraPlatform) -> Result<SensorSettings, ()>;
}
