use crate::error::Error;

pub mod impls;

pub trait Input {
    fn has_remaining(&self) -> bool;
    fn read_byte(&mut self) -> Result<u8, Error>;
    fn read_long_le(&mut self) -> Result<u64, Error>;
    fn read_double_le(&mut self) -> Result<f64,Error>;
}

