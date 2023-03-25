use crate::error::Error;
use crate::index_mapping::impls::CubicallyInterpolatedMapping;
use crate::index_mapping::{IndexMapping, IndexMappingLayout};
use crate::input::Input;
use crate::store::impls::{CollapsingHighestDenseStore, CollapsingLowestDenseStore};
use crate::store::{BinEncodingMode, Store};
use crate::util::serde;

pub struct DDSketch<I: IndexMapping, S: Store> {
    index_mapping: I,
    min_indexed_value: f64,
    max_indexed_value: f64,
    negative_value_store: S,
    positive_value_store: S,
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

impl<I: IndexMapping, S: Store> DDSketch<I, S> {
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
            self.positive_value_store
                .add(self.index_mapping.index(value), 1.0);
        } else if value < -self.min_indexed_value {
            self.negative_value_store
                .add(self.index_mapping.index(-value), 1.0);
        } else {
            self.zero_count += 1.0;
        }
    }

    pub fn is_empty(&self) -> bool {
        self.zero_count == 0.0
            && self.negative_value_store.is_empty()
            && self.positive_value_store.is_empty()
    }

    pub fn clear(&mut self) {
        self.negative_value_store.clear();
        self.positive_value_store.clear();
        self.zero_count = 0.0;
    }

    pub fn get_count(&mut self) -> f64 {
        self.zero_count
            + self.negative_value_store.get_total_count()
            + self.positive_value_store.get_total_count()
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

    pub fn get_max(&mut self) -> Result<f64, Error> {
        return if !self.positive_value_store.is_empty() {
            Ok(self
                .index_mapping
                .value(self.positive_value_store.get_max_index()))
        } else if self.zero_count > 0.0 {
            Ok(0.0)
        } else if !self.negative_value_store.is_empty() {
            Ok(-self
                .index_mapping
                .value(self.negative_value_store.get_min_index()))
        } else {
            Err(Error::NoSuchElement)
        };
    }

    pub fn get_min(&mut self) -> Result<f64, Error> {
        return if !self.negative_value_store.is_empty() {
            Ok(-self
                .index_mapping
                .value(self.negative_value_store.get_max_index()))
        } else if self.zero_count > 0.0 {
            Ok(0.0)
        } else if !self.positive_value_store.is_empty() {
            Ok(self
                .index_mapping
                .value(self.positive_value_store.get_min_index()))
        } else {
            Err(Error::NoSuchElement)
        };
    }

    pub fn get_average(&mut self) -> Result<f64, Error> {
        let count = self.get_count();
        if count <= 0.0 {
            return Err(Error::InvalidArgument("count <= 0"));
        }
        return Ok(self.get_sum() / count);
    }

    pub fn get_value_at_quantile(self: &mut DDSketch<I, S>, quantile: f64) -> Result<f64, Error> {
        if quantile < 0.0 || quantile > 1.0 {
            return Err(Error::InvalidArgument("quantile"));
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
                    self.positive_value_store
                        .decode_and_merge_with(input, mode)?;
                }
                FlagType::NegativeStore => {
                    let mode = BinEncodingMode::of_flag(flag.get_marker())?;
                    self.negative_value_store
                        .decode_and_merge_with(input, mode)?;
                }
                FlagType::IndexMapping => {
                    let layout = IndexMappingLayout::of_flag(&flag)?;
                    match layout {
                        IndexMappingLayout::LogCubic => {
                            let decoded_index_mapping =
                                CubicallyInterpolatedMapping::decode(input)?;

                            if self.index_mapping.to_string() != decoded_index_mapping.to_string() {
                                return Err(Error::InvalidArgument("Unmatched IndexMapping"));
                            }
                        }
                        _ => {
                            return Err(Error::InvalidArgument("IndexMapping"));
                        }
                    }
                }
                FlagType::SketchFeatures => {
                    if Flag::ZERO_COUNT == flag {
                        self.zero_count += serde::decode_var_double(input)?;
                    } else {
                        serde::ignore_exact_summary_statistic_flags(input, flag)?;
                    }
                }
            }
        }
        Ok(())
    }

    pub fn merge_with(&mut self, other: &mut DDSketch<I, S>) {
        if self.index_mapping.to_string() != other.index_mapping.to_string() {
            return;
        }
        self.negative_value_store
            .merge_with(&mut other.negative_value_store);
        self.positive_value_store
            .merge_with(&mut other.positive_value_store);
        self.zero_count += other.zero_count;
    }
}

impl DDSketch<CubicallyInterpolatedMapping, CollapsingLowestDenseStore> {
    pub fn collapsing_lowest_dense(
        relative_accuracy: f64,
        max_num_bins: i32,
    ) -> DDSketch<CubicallyInterpolatedMapping, CollapsingLowestDenseStore> {
        let index_mapping = CubicallyInterpolatedMapping::with_relative_accuracy(relative_accuracy);
        let negative_value_store = CollapsingLowestDenseStore::new(max_num_bins);
        let positive_value_store = CollapsingLowestDenseStore::new(max_num_bins);
        let min_indexed_value = f64::max(0.0, index_mapping.min_indexable_value());
        let max_indexed_value = index_mapping.max_indexable_value();
        let zero_count = 0.0;
        DDSketch {
            index_mapping,
            negative_value_store,
            positive_value_store,
            min_indexed_value,
            max_indexed_value,
            zero_count,
        }
    }
}

impl DDSketch<CubicallyInterpolatedMapping, CollapsingHighestDenseStore> {
    pub fn collapsing_highest_dense(
        relative_accuracy: f64,
        max_num_bins: i32,
    ) -> DDSketch<CubicallyInterpolatedMapping, CollapsingHighestDenseStore> {
        let index_mapping = CubicallyInterpolatedMapping::with_relative_accuracy(relative_accuracy);
        let negative_value_store = CollapsingHighestDenseStore::new(max_num_bins);
        let positive_value_store = CollapsingHighestDenseStore::new(max_num_bins);
        let min_indexed_value = f64::max(0.0, index_mapping.min_indexable_value());
        let max_indexed_value = index_mapping.max_indexable_value();
        let zero_count = 0.0;
        DDSketch {
            index_mapping,
            negative_value_store,
            positive_value_store,
            min_indexed_value,
            max_indexed_value,
            zero_count,
        }
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
            0b00 => Ok(FlagType::SketchFeatures),
            0b01 => Ok(FlagType::PositiveStore),
            0b10 => Ok(FlagType::IndexMapping),
            0b11 => Ok(FlagType::NegativeStore),
            _ => Err(Error::InvalidArgument("FlagType")),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::input::impls::DefaultInput;

    use super::*;

    #[test]
    fn test_sketch_quantile_0() {
        let mut sketch = DDSketch::collapsing_lowest_dense(0.02, 100);
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
    fn test_sketch_quantile_1() {
        let mut sketch = DDSketch::collapsing_highest_dense(0.02, 100);
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

    #[test]
    fn test_sketch_merge() {
        let accuracy = 2e-2;

        let mut sketch1 = DDSketch::collapsing_lowest_dense(accuracy, 50);
        for i in -99..101 {
            sketch1.accept(i as f64);
        }

        let mut sketch2 = DDSketch::collapsing_lowest_dense(accuracy, 50);
        for i in 100..200 {
            sketch2.accept(i as f64);
        }

        sketch1.merge_with(&mut sketch2);
        assert_eq!(300.0, sketch1.get_count());
    }

    #[test]
    fn test_sketch_decode() {
        let accuracy = 2e-2;
        let mut input = DefaultInput::wrap(vec![
            14, 100, 244, 7, 173, 131, 165, 240, 63, 0, 0, 0, 0, 0, 0, 0, 0, 5, 21, 0, 140, 48, 34,
            150, 241, 16, 20, 148, 191, 96, 14, 142, 62, 12, 139, 16, 10, 134, 96, 8, 3, 6, 2, 6,
            2, 6, 2, 4, 2, 42, 2, 26, 2, 6, 2, 20, 2, 6, 2, 2, 2, 10, 2, 20, 2, 14, 2, 10, 2,
        ]);
        let mut sketch = DDSketch::collapsing_lowest_dense(accuracy, 50);
        sketch.decode_and_merge_with(&mut input).unwrap();
        assert_eq!(4538.0, sketch.get_count());
    }
}
