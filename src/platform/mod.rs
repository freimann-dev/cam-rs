// mod.rs

pub mod esp32s3;

pub trait Platform {
    type Error;

    fn xclk_on(&mut self, freq_hz: u32) -> Result<(), Self::Error>;
    fn verify_pclk(&mut self) -> Result<(), Self::Error>;
    fn capture_frame(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error>;
}
