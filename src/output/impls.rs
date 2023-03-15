use super::*;

pub struct DefaultOutput {
    vec: Vec<u8>,
}

impl DefaultOutput {
    pub fn new(size: usize) -> DefaultOutput {
        DefaultOutput { vec: Vec::with_capacity(size) }
    }
}

impl Output for DefaultOutput {
    fn write_byte(&mut self, value: u8) -> Result<(),Error> {
        self.vec.push(value);
        Ok(())
    }

    fn write_long_le(&mut self, value: i64) -> Result<(),Error> {
        let bytes = i64::to_le_bytes(value);
        for b in bytes {
            self.vec.push(b);
        }
        Ok(())
    }

    fn write_double_le(&mut self, value: f64) -> Result<(),Error> {
        let bytes = f64::to_le_bytes(value);
        self.vec.extend(bytes);
        Ok(())
    }
}