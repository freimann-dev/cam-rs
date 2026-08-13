pub mod esp32s3;

pub trait CameraPlatform {
    fn start_capture(&mut self) -> Result<(), ()>;
    fn get_current_buffer<'a>(&'a mut self) -> Result<&'a [u8], ()>;
}
