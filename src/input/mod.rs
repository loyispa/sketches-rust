use crate::error::Error;
use std::io::ErrorKind;

pub struct Input<'a> {
    vec: &'a Vec<u8>,
    pos: usize,
    end: usize,
}

impl<'a> Input<'a> {
    pub fn wrap(vec: &'a Vec<u8>) -> Input {
        Input {
            pos: 0,
            end: vec.len(),
            vec,
        }
    }

    pub(crate) fn has_remaining(&self) -> bool {
        self.pos < self.end
    }

    pub(crate) fn read_byte(&mut self) -> Result<u8, Error> {
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

    pub(crate) fn read_double_le(&mut self) -> Result<f64, Error> {
        let value = f64::from_bits(self.read_long_le()?);
        Ok(value)
    }
}
