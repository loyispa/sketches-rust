use crate::error::Error;

mod default;
pub use default::DefaultOutput;

pub trait Output {
    fn write_byte(&mut self, value: u8) -> Result<(), Error>;
    fn write_long_le(&mut self, value: i64) -> Result<(), Error>;
    fn write_double_le(&mut self, value: f64) -> Result<(), Error>;
}
