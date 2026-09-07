pub mod esp32s3;

use crate::CameraError;
use crate::CameraResult;

pub trait Chip<P> {
    type I2c;

    fn init(&mut self, periph: P) -> CameraResult<Self::I2c>;

    fn check_vsync(&self) -> Result<bool, CameraError>;

    fn setup_dma(&mut self) -> CameraResult<()>;
    fn name(&self) -> &'static str;
}
