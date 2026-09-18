pub mod esp32s3;

use crate::BoardError;
use crate::BoardResult;

pub trait Chip<P> {
    type I2c;
    type Camera;

    fn new(&mut self, gpio: P) -> BoardResult<(Self::I2c, Self::Camera)>;
    fn check_vsync(&self) -> Result<bool, BoardError>;

    fn setup_dma(&mut self) -> BoardResult<()>;
    fn name(&self) -> &'static str;
}
