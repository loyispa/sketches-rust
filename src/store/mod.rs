use crate::error::Error;
use crate::input::Input;
use crate::util::serde;

pub mod impls;

pub trait Store {
    fn add(&mut self, index: i32, count: f64);
    fn add_bin(&mut self, bin: (i32, f64));
    fn merge_with(&mut self, store: &mut impl Store) {
        for bin in store.get_descending_stream() {
            self.add_bin(bin)
        }
    }
    fn clear(&mut self);
    fn is_empty(&self) -> bool;
    fn get_total_count(&mut self) -> f64;
    fn get_min_index(&self) -> i32;
    fn get_max_index(&self) -> i32;
    fn decode_and_merge_with(
        &mut self,
        input: &mut impl Input,
        mode: BinEncodingMode,
    ) -> Result<(), Error> {
        return match mode {
            BinEncodingMode::IndexDeltasAndCounts => {
                let num_bins = serde::decode_unsigned_var_long(input)?;
                let mut index: i64 = 0;
                let mut i = 0;
                while i < num_bins {
                    let index_delta = serde::decode_signed_var_long(input)?;
                    let count = serde::decode_var_double(input)?;
                    index += index_delta;
                    self.add(serde::i64_to_i32_exact(index)?, count);
                    i += 1;
                }

                Ok(())
            }

            BinEncodingMode::IndexDeltas => {
                let num_bins = serde::decode_unsigned_var_long(input)?;
                let mut index: i64 = 0;
                let mut i = 0;
                while i < num_bins {
                    let index_delta = serde::decode_signed_var_long(input)?;
                    index += index_delta;
                    self.add(serde::i64_to_i32_exact(index)?, 1.0);
                    i += 1;
                }
                Ok(())
            }

            BinEncodingMode::ContiguousCounts => {
                let num_bins = serde::decode_unsigned_var_long(input)?;
                let mut index: i64 = serde::decode_signed_var_long(input)?;
                let index_delta = serde::decode_signed_var_long(input)?;

                let mut i = 0;
                while i < num_bins {
                    let count = serde::decode_var_double(input)?;
                    self.add(serde::i64_to_i32_exact(index)?, count);
                    index += index_delta;
                    i += 1;
                }
                Ok(())
            }
        };
    }
    fn get_descending_stream(&mut self) -> Vec<(i32, f64)>;
    fn get_ascending_stream(&mut self) -> Vec<(i32, f64)>;
    fn get_descending_iter(&mut self) -> StoreIter;
    fn get_ascending_iter(&mut self) -> StoreIter;
    fn foreach<F>(&mut self, acceptor: F)
    where
        F: FnMut(i32, f64);
}

pub struct StoreIter<'a> {
    min_index: i32,
    max_index: i32,
    offset: i32,
    desc: bool,
    counts: &'a [f64],
}

impl<'a> StoreIter<'a> {
    pub fn new(
        min_index: i32,
        max_index: i32,
        offset: i32,
        desc: bool,
        counts: &'a [f64],
    ) -> StoreIter {
        StoreIter {
            desc,
            min_index,
            max_index,
            offset,
            counts,
        }
    }
}

impl<'a> Iterator for StoreIter<'a> {
    type Item = (i32, f64);
    fn next(&mut self) -> Option<Self::Item> {
        return if self.desc {
            if self.max_index < self.min_index {
                return None;
            }

            let index = self.max_index as i32;
            self.max_index -= 1;

            while self.max_index >= self.min_index {
                let count = self.counts[(self.max_index - self.offset) as usize];
                if count != 0.0 {
                    break;
                }
                self.max_index -= 1;
            }

            let count = self.counts[(index - self.offset) as usize];
            Some((index, count))
        } else {
            if self.min_index > self.max_index {
                return None;
            }

            let index = self.min_index as i32;
            self.min_index += 1;

            while self.min_index <= self.max_index {
                let count = self.counts[(self.min_index - self.offset) as usize];
                if count != 0.0 {
                    break;
                }
                self.min_index += 1;
            }

            let count = self.counts[(index - self.offset) as usize];
            Some((index, count))
        };
    }
}

#[derive(Clone, Copy, Debug)]
pub enum BinEncodingMode {
    IndexDeltasAndCounts = 1,
    IndexDeltas = 2,
    ContiguousCounts = 3,
}

impl BinEncodingMode {
    pub fn of_flag(marker: u8) -> Result<BinEncodingMode, Error> {
        let index = (marker >> 2) - 1;
        match index {
            0 => Ok(BinEncodingMode::IndexDeltasAndCounts),
            1 => Ok(BinEncodingMode::IndexDeltas),
            2 => Ok(BinEncodingMode::ContiguousCounts),
            _ => Err(Error::InvalidArgument("marker")),
        }
    }
}
