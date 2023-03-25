use crate::error::Error;
use Result;

pub mod impls;

pub trait Output {
    fn write_byte(&mut self, value: u8) -> Result<(), Error>;
    fn write_long_le(&mut self, value: i64) -> Result<(), Error>;
    fn write_double_le(&mut self, value: f64) -> Result<(), Error>;
}
