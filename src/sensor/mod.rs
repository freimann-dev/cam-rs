// mod.rs

pub mod ov5640;

pub trait Sensor {
    type Error;

    fn init(&mut self) -> Result<(), Self::Error>;
}
