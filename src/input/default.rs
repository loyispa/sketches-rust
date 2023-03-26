use super::*;
use crate::error::Error;
use std::io::ErrorKind;

pub struct DefaultInput {
    vec: Vec<u8>,
    pos: usize,
    end: usize,
}

impl DefaultInput {
    pub fn wrap(vec: Vec<u8>) -> DefaultInput {
        DefaultInput {
            pos: 0,
            end: vec.len(),
            vec,
        }
    }
}

impl Input for DefaultInput {
    fn has_remaining(&self) -> bool {
        self.pos < self.end
    }

    fn read_byte(&mut self) -> Result<u8, Error> {
        if self.pos >= self.end {
            return Err(Error::IoError(ErrorKind::UnexpectedEof));
        }
        let value = self.vec[self.pos];
        self.pos += 1;
        Ok(value)
    }

    fn read_long_le(&mut self) -> Result<u64, Error> {
        if self.pos + 8 > self.end {
            return Err(Error::IoError(ErrorKind::UnexpectedEof));
        }

        let value = u64::from_le_bytes([
            self.vec[self.pos],
            self.vec[self.pos + 1],
            self.vec[self.pos + 2],
            self.vec[self.pos + 3],
            self.vec[self.pos + 4],
            self.vec[self.pos + 5],
            self.vec[self.pos + 6],
            self.vec[self.pos + 7],
        ]);
        self.pos += 8;
        Ok(value)
    }

    fn read_double_le(&mut self) -> Result<f64, Error> {
        let value = f64::from_bits(self.read_long_le()?);
        Ok(value)
    }
}
