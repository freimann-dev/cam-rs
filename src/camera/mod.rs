pub mod ov5640;

pub trait Camera {
    type Error;

    fn enable(&mut self) -> Result<(), Self::Error>;
    fn init(&mut self) -> Result<(), Self::Error>;
    fn write_sensor_table(&mut self) -> Result<(), Self::Error>;
}
