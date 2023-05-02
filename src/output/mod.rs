use crate::error::Error;

pub struct Output {
    vec: Vec<u8>,
}

impl Output {
    pub fn with_capacity(capacity: usize) -> Output {
        Output {
            vec: Vec::with_capacity(capacity),
        }
    }

    pub fn trim(self) -> Vec<u8> {
        self.vec
    }

    pub(crate) fn write_byte(&mut self, value: u8) -> Result<(), Error> {
        self.vec.push(value);
        Ok(())
    }

    // fn write_long_le(&mut self, value: i64) -> Result<(), Error> {
    //     let bytes = i64::to_le_bytes(value);
    //     for b in bytes {
    //         self.vec.push(b);
    //     }
    //     Ok(())
    // }

    pub(crate) fn write_double_le(&mut self, value: f64) -> Result<(), Error> {
        let bytes = f64::to_le_bytes(value);
        self.vec.extend(bytes);
        Ok(())
    }
}
