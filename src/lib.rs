#![no_std]

pub mod board;
pub mod camera;

use board::Board;
use camera::Camera;
use esp_println::println;

#[derive(Debug)]
pub enum DriverError<BE, CE> {
    BoardError(BE),
    CameraError(CE),
}

pub struct Driver<B, C> {
    pub board: B,
    pub camera: C,
}

impl<B, C> Driver<B, C>
where
    B: Board,
    C: Camera,
{
    pub fn new(mut board: B, mut camera: C) -> Result<Self, DriverError<B::Error, C::Error>> {
        println!("[driver]--------------------------------------------");

        board.xclk_on(20_000_000).map_err(DriverError::BoardError)?;

        println!("[driver] Waiting {}ms for sensor stabilization...", 1);
        esp_hal::delay::Delay::new().delay_millis(1);

        camera.enable().map_err(DriverError::CameraError)?;
        // camera.init().map_err(DriverError::CameraError)?;

        camera
            .write_sensor_table()
            .map_err(DriverError::CameraError)?;

        // println!("[driver] ⏳ Verifying camera hardware pipeline...");
        // let _frame = board.capture().map_err(DriverError::BoardError)?;
        // println!("[driver] ✅ Hardware initialization complete! Frame captured.");

        // board.activate_pipeline().map_err(DriverError::BoardError)?;

        match board.capture_frame() {
            Ok(frame) => {
                println!(
                    "[driver] ✅ Test frame captured! Size: {} bytes",
                    frame.len()
                );
                if frame.len() >= 2 && frame[0] == 0xFF && frame[1] == 0xD8 {
                    println!("[driver] 🎉 Valid JPEG header (0xFFD8) confirmed!");
                } else {
                    println!("[driver] ⚠️ Frame received, but JPEG header missing!");
                }
            }
            Err(e) => {
                println!("[driver] ❌ Test frame capture failed: {:?}", e);
            }
        }

        println!("[driver]--------------------------------------------");

        Ok(Self { board, camera })
    }
}
