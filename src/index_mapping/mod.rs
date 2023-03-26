use crate::sketch::Flag;
use crate::Error;

mod cubically_interpolated;
mod logarithmic;

pub use cubically_interpolated::CubicallyInterpolatedMapping;
pub use logarithmic::LogarithmicMapping;

pub trait IndexMapping: ToString {
    fn index(&self, value: f64) -> i32;
    fn value(&self, index: i32) -> f64;
    fn lower_bound(&self, index: i32) -> f64;
    fn upper_bound(&self, index: i32) -> f64;
    fn get_relative_accuracy(&self) -> f64;
    fn min_indexable_value(&self) -> f64;
    fn max_indexable_value(&self) -> f64;
}

#[warn(dead_code)]
pub enum IndexMappingLayout {
    LOG,
    LogLinear,
    LogQuadratic,
    LogCubic,
    LogQuartic,
}

impl IndexMappingLayout {
    pub fn of_flag(flag: &Flag) -> Result<IndexMappingLayout, Error> {
        let index = flag.get_marker() >> 2;
        return match index {
            0 => Ok(IndexMappingLayout::LOG),
            1 => Ok(IndexMappingLayout::LogLinear),
            2 => Ok(IndexMappingLayout::LogQuadratic),
            3 => Ok(IndexMappingLayout::LogCubic),
            4 => Ok(IndexMappingLayout::LogQuartic),
            _ => Err(Error::InvalidArgument("unknown flag")),
        };
    }
}

fn calculate_relative_accuracy(gamma: f64, correcting_factor: f64) -> f64 {
    let exact_log_gamma = gamma.powf(correcting_factor);
    return (exact_log_gamma - 1.0) / (exact_log_gamma + 1.0);
}

fn calculate_gamma(relative_accuracy: f64, correcting_factor: f64) -> f64 {
    let exact_log_gamma = (1.0 + relative_accuracy) / (1.0 - relative_accuracy);
    exact_log_gamma.powf(1.0 / correcting_factor)
}

#[cfg(test)]
mod tests {
    use crate::index_mapping::{CubicallyInterpolatedMapping, IndexMapping};

    const TEST_GAMMAS: [f64; 3] = [1.0 + 1e-6, 1.02, 1.5];
    const TEST_INDEX_OFFSETS: [f64; 4] = [0.0, 1.0, -12.23, 7768.3];
    const EPSILON: f64 = 1e-10;

    #[test]
    fn test_accuracy() {
        let accuracy = [
            5.04943011787191E-7,
            5.04943011787191E-7,
            5.04943011787191E-7,
            5.04943011787191E-7,
            0.009998870028530763,
            0.009998870028530763,
            0.009998870028530763,
            0.009998870028530763,
            0.20192337476263508,
            0.20192337476263508,
            0.20192337476263508,
            0.20192337476263508,
        ];
        let mut index = 0;
        for gamma in TEST_GAMMAS {
            for index_offset in TEST_INDEX_OFFSETS {
                let index_mapping =
                    CubicallyInterpolatedMapping::with_gamma_offset(gamma, index_offset);
                assert_eq!(accuracy[index], index_mapping.get_relative_accuracy());
                index += 1;
            }
        }
    }

    #[test]
    fn test_offset() {
        for gamma in TEST_GAMMAS {
            for index_offset in TEST_INDEX_OFFSETS {
                let index_mapping =
                    CubicallyInterpolatedMapping::with_gamma_offset(gamma, index_offset);
                let index_of1 = index_mapping.index(1.0) as f64;
                // If 1 is on a bucket boundary, its associated index can be either of the ones of the previous
                // and the next buckets.
                assert!(index_offset.ceil() - 1.0 <= index_of1);
                assert!(index_of1 <= index_offset.floor());
            }
        }
    }

    #[test]
    fn test_validity() {
        let mapping = CubicallyInterpolatedMapping::with_relative_accuracy(1e-2).unwrap();

        let min_index = -50;
        let max_index = 50;

        let mut index = min_index;
        let mut bound = mapping.upper_bound(index - 1);

        while index <= max_index {
            assert!(f64::abs(mapping.lower_bound(index) - bound) <= 1e10);
            assert!(mapping.value(index) >= mapping.lower_bound(index));
            assert!(mapping.upper_bound(index) >= mapping.value(index));

            assert!(mapping.index(mapping.lower_bound(index) - EPSILON) < index);
            assert!(mapping.index(mapping.lower_bound(index) + EPSILON) >= index);

            assert!(mapping.index(mapping.upper_bound(index) - EPSILON) <= index);
            assert!(mapping.index(mapping.upper_bound(index) + EPSILON) > index);

            bound = mapping.upper_bound(index);
            index += 1;
        }
    }
}
