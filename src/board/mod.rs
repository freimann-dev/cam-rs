// mod.rs

pub mod esp32s3;

pub trait Board {
    type Error: core::fmt::Debug;

    fn xclk_on(&mut self, freq_hz: u32) -> Result<(), Self::Error>;
    // fn activate_pipeline(&mut self) -> Result<(), Self::Error>;
    fn capture_frame(&mut self) -> Result<&[u8], Self::Error>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VsyncPolarity {
    ActiveHigh,
    ActiveLow,
}

#[derive(Debug)]
pub enum BoardError {
    ClockError,
    SyncError,
    CaptureError,
}
