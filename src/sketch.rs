use crate::error::{Error};
use crate::util::serde;
use crate::index_mapping::impls::{CubicallyInterpolatedMapping};
use crate::index_mapping::{IndexMapping};
use crate::input::Input;
use crate::store::{Store, BinEncodingMode};
use crate::store::impls::{CollapsingLowestDenseStore};

pub struct DDSketch {
    index_mapping: CubicallyInterpolatedMapping,
    min_indexed_value: f64,
    max_indexed_value: f64,
    negative_value_store: CollapsingLowestDenseStore,
    positive_value_store: CollapsingLowestDenseStore,
    zero_count: f64,
}

#[derive(PartialEq)]
pub struct Flag {
    marker: u8,
}

pub enum FlagType {
    SketchFeatures = 0b00,
    PositiveStore = 0b01,
    IndexMapping = 0b10,
    NegativeStore = 0b11,
}


impl DDSketch {

    pub fn collapsing_lowest_dense(relative_accuracy: f64, max_num_bins: i32) -> DDSketch {
        let index_mapping = CubicallyInterpolatedMapping::with_relative_accuracy(relative_accuracy);
        let negative_value_store = CollapsingLowestDenseStore::new(max_num_bins);
        let positive_value_store = CollapsingLowestDenseStore::new(max_num_bins);
        let min_indexed_value = f64::max(0.0, index_mapping.min_indexable_value());
        let max_indexed_value = index_mapping.max_indexable_value();
        let zero_count = 0.0;
        DDSketch { index_mapping, negative_value_store, positive_value_store, min_indexed_value, max_indexed_value, zero_count }
    }

    pub fn accept(&mut self, value: f64) {
        self.accept_with_count(value, 1.0);
    }

    pub fn accept_with_count(&mut self, value: f64, count: f64) {
        if count < 0.0 {
            return;
        }

        if value < -self.max_indexed_value || value > self.max_indexed_value {
            return;
        }

        if value > self.min_indexed_value {
            self.positive_value_store.add(self.index_mapping.index(value), 1.0);
        } else if value < -self.min_indexed_value {
            self.negative_value_store.add(self.index_mapping.index(-value), 1.0);
        } else {
            self.zero_count += 1.0;
        }
    }

    pub fn merge_with(&mut self, other: DDSketch) {
        if self.index_mapping != other.index_mapping {
            return;
        }
        self.negative_value_store.merge_with(other.negative_value_store);
        self.positive_value_store.merge_with(other.positive_value_store);
        self.zero_count += other.zero_count;
    }

    pub fn is_empty(&self) -> bool {
        self.zero_count == 0.0 && self.negative_value_store.is_empty() && self.positive_value_store.is_empty()
    }

    pub fn clear(&mut self) {
        self.negative_value_store.clear();
        self.positive_value_store.clear();
        self.zero_count = 0.0;
    }

    pub fn get_count(&mut self) -> f64 {
        self.zero_count + self.negative_value_store.get_total_count() + self.positive_value_store.get_total_count()
    }

    pub fn get_sum(&mut self) -> f64 {
        let mut sum = 0.0;

        self.negative_value_store.foreach(|index: i32, count: f64| {
            sum -= self.index_mapping.value(index) * count;
        });

        self.positive_value_store.foreach(|index: i32, count: f64| {
            sum += self.index_mapping.value(index) * count;
        });

        sum
    }

    pub fn get_max(&mut self) -> Result<f64,Error> {
        return if !self.positive_value_store.is_empty() {
            Ok(self.index_mapping.value(self.positive_value_store.get_max_index()))
        } else if self.zero_count > 0.0 {
            Ok(0.0)
        } else if !self.negative_value_store.is_empty() {
            Ok(-self.index_mapping.value(self.negative_value_store.get_min_index()))
        } else {
            Err(Error::NoSuchElement)
        };
    }

    pub fn get_min(&mut self) -> Result<f64,Error> {
        return if !self.negative_value_store.is_empty() {
            Ok(-self.index_mapping.value(self.negative_value_store.get_max_index()))
        } else if self.zero_count > 0.0 {
            Ok(0.0)
        } else if !self.positive_value_store.is_empty() {
            Ok(self.index_mapping.value(self.positive_value_store.get_min_index()))
        } else {
            Err(Error::NoSuchElement)
        };
    }

    pub fn get_average(&mut self) -> Result<f64,Error> {
        let count = self.get_count();
        if count <= 0.0 {
            return Err(Error::InvalidArgument);
        }
        return Ok(self.get_sum() / count);
    }

    pub fn get_value_at_quantile(self: &mut DDSketch, quantile: f64) -> Result<f64,Error> {
        if quantile < 0.0 || quantile > 1.0 {
            return Err(Error::InvalidArgument);
        }

        let count = self.get_count();
        if count <= 0.0 {
            return Err(Error::NoSuchElement);
        }

        let rank = quantile * (count - 1.0);

        let mut n: f64 = 0.0;

        let negative_bin_iterator = self.negative_value_store.get_descending_iter();
        for bin in negative_bin_iterator {
            n += bin.1;
            if n > rank {
                return Ok(-self.index_mapping.value(bin.0));
            }
        }

        n += self.zero_count;
        if n > rank {
            return Ok(0.0);
        }

        let positive_bin_iterator = self.positive_value_store.get_ascending_iter();
        for bin in positive_bin_iterator {
            n += bin.1;
            if n > rank {
                return Ok(self.index_mapping.value(bin.0));
            }
        }

        Err(Error::NoSuchElement)
    }

    pub fn decode_and_merge_with(&mut self, input: &mut impl Input) -> Result<(), Error> {
        while input.has_remaining() {
            let flag = Flag::decode(input)?;
            let flag_type = flag.get_type()?;
            match flag_type {
                FlagType::PositiveStore => {
                    let mode = BinEncodingMode::of_flag(flag.get_marker())?;
                    self.positive_value_store.decode_and_merge_with(input, mode)?;
                }
                FlagType::NegativeStore => {
                    let mode = BinEncodingMode::of_flag(flag.get_marker())?;
                    self.negative_value_store.decode_and_merge_with(input, mode)?;
                }
                FlagType::IndexMapping => {
                    let decoded_index_mapping =
                        CubicallyInterpolatedMapping::decode(input)?;
                    if self.index_mapping != decoded_index_mapping {
                        return Err(Error::InvalidArgument);
                    }
                }
                FlagType::SketchFeatures => {
                    if Flag::ZERO_COUNT == flag {
                        self.zero_count += serde::decode_var_double(input)?;
                    } else {
                        DDSketch::ignore_exact_summary_statistic_flags(input, flag)?;
                    }
                }
            }
        }
        Ok(())
    }

    fn ignore_exact_summary_statistic_flags(input: &mut impl Input, flag: Flag) -> Result<(), Error> {
        return if flag == Flag::COUNT {
            serde::decode_var_double(input)?;
            Ok(())
        } else if flag == Flag::SUM || flag == Flag::MIN || flag == Flag::MAX {
            input.read_double_le()?;
            Ok(())
        } else {
            Err(Error::InvalidArgument)
        };
    }
}

impl Flag {
    pub const ZERO_COUNT: Flag = Flag::with_type(FlagType::SketchFeatures, 1);
    pub const COUNT: Flag = Flag::with_type(FlagType::SketchFeatures, 0x28);
    pub const SUM: Flag = Flag::with_type(FlagType::SketchFeatures, 0x21);
    pub const MIN: Flag = Flag::with_type(FlagType::SketchFeatures, 0x22);
    pub const MAX: Flag = Flag::with_type(FlagType::SketchFeatures, 0x23);

    pub const fn new(marker: u8) -> Flag {
        Flag { marker }
    }

    pub fn decode(input: &mut impl Input) -> Result<Flag, Error> {
        let marker = input.read_byte()?;
        Ok(Flag::new(marker))
    }

    pub fn get_type(&self) -> Result<FlagType, Error> {
        FlagType::value_of(self.marker & 3)
    }

    pub fn get_marker(&self) -> u8 {
        self.marker
    }

    const fn with_type(flag_type: FlagType, sub_flag: u8) -> Flag {
        let t = flag_type as u8;
        Flag::new(t | (sub_flag << 2))
    }
}

impl FlagType {
    pub fn value_of(t: u8) -> Result<FlagType, Error> {
        match t {
            0b00 => { Ok(FlagType::SketchFeatures) }
            0b01 => { Ok(FlagType::PositiveStore) }
            0b10 => { Ok(FlagType::IndexMapping) }
            0b11 => { Ok(FlagType::NegativeStore) }
            _ => { Err(Error::UnknownType) }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sketch_quantile() {
        let mut sketch = DDSketch::collapsing_lowest_dense(0.02,100);
        sketch.accept(1.0);
        sketch.accept(2.0);
        sketch.accept(3.0);
        sketch.accept(4.0);
        sketch.accept(5.0);

        assert!((f64::abs(sketch.get_value_at_quantile(0.0).unwrap() - 1.0) / 1.0) < 0.021);
        assert!((f64::abs(sketch.get_value_at_quantile(0.5).unwrap() - 3.0) / 3.0) < 0.021);
        assert!((f64::abs(sketch.get_value_at_quantile(1.0).unwrap() - 5.0) / 5.0) < 0.021);
    }

    #[test]
    fn test_sketch_add() {
        let accuracy = 2e-2;

        let mut sketch = DDSketch::collapsing_lowest_dense(accuracy, 50);

        for i in -99..101 {
            sketch.accept(i as f64);
        }

        assert_eq!(200.0, sketch.get_count());
        assert!((f64::abs(sketch.get_min().unwrap() - -99.0) / -99.0) <= accuracy);
        assert!((f64::abs(sketch.get_max().unwrap() - 100.0) / 100.0) <= accuracy);
        assert!((f64::abs(sketch.get_average().unwrap() - 0.5) / 0.5) <= accuracy);
        assert!((f64::abs(sketch.get_sum() - 100.0) / 100.0) <= accuracy);
    }
}