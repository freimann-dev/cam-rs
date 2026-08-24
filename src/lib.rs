#![no_std]

#[cfg(feature = "esp32")]
pub mod esp32_cam;
#[cfg(feature = "esp32s3")]
pub mod esp32s3_eye;

pub type BoardResult<T> = Result<T, BoardError>;

#[derive(Debug)]
pub enum BoardError {
    InitFailed,
    SensorNotResponding,
    UnsupportedSensor,
    CaptureFailed,

    #[cfg(feature = "esp32")]
    Esp32CamInit(esp32_cam::Esp32CamError),

    #[cfg(feature = "esp32s3")]
    Esp32S3Init(esp32s3_eye::Esp32S3EyeError),
}

#[cfg(feature = "esp32")]
impl From<esp32_cam::Esp32CamError> for BoardError {
    fn from(err: esp32_cam::Esp32CamError) -> Self {
        BoardError::Esp32CamInit(err)
    }
}

#[cfg(feature = "esp32s3")]
impl From<esp32s3_eye::Esp32S3EyeError> for BoardError {
    fn from(err: esp32s3_eye::Esp32S3EyeError) -> Self {
        BoardError::Esp32S3Init(err)
    }
}

pub struct Frame {}

pub struct Board {
    #[cfg(feature = "esp32")]
    inner: esp32_cam::Esp32Cam,

    #[cfg(feature = "esp32s3")]
    inner: esp32s3_eye::Esp32S3Eye,
}

impl Board {
    pub fn new() -> BoardResult<Self> {
        esp_alloc::heap_allocator!(size: 64 * 1024);

        Ok(Self {
            #[cfg(feature = "esp32s3")]
            inner: esp32s3_eye::Esp32S3Eye::new()?,

            #[cfg(feature = "esp32")]
            inner: esp32_cam::Esp32Cam::new()?,
        })
    }

    pub fn capture(&mut self) -> BoardResult<Frame> {
        self.inner.capture()
    }
}
