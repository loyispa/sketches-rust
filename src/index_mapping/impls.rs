use crate::error::Error;
use crate::index_mapping::IndexMapping;
use crate::input::Input;
use crate::util::serde;

#[derive(PartialEq)]
pub struct CubicallyInterpolatedMapping {
    gamma: f64,
    index_offset: f64,
    multiplier: f64,
    relative_accuracy: f64,
}

impl CubicallyInterpolatedMapping {
    const A: f64 = 6.0 / 35.0;
    const B: f64 = -3.0 / 5.0;
    const C: f64 = 10.0 / 7.0;
    const CORRECTING_FACTOR: f64 = 1.0 / (CubicallyInterpolatedMapping::C * std::f64::consts::LN_2);

    pub fn with_relative_accuracy(relative_accuracy: f64) -> CubicallyInterpolatedMapping {
        let gamma = CubicallyInterpolatedMapping::calculate_gamma(
            relative_accuracy,
            CubicallyInterpolatedMapping::CORRECTING_FACTOR,
        );
        let index_offset: f64 = 0.0;
        let multiplier = std::f64::consts::LN_2 / gamma.ln();
        let relative_accuracy = CubicallyInterpolatedMapping::calculate_relative_accuracy(
            gamma,
            CubicallyInterpolatedMapping::CORRECTING_FACTOR,
        );
        CubicallyInterpolatedMapping {
            relative_accuracy,
            gamma,
            index_offset,
            multiplier,
        }
    }

    pub fn with_gamma_offset(gamma: f64, index_offset: f64) -> CubicallyInterpolatedMapping {
        let multiplier = std::f64::consts::LN_2 / gamma.ln();
        let relative_accuracy = CubicallyInterpolatedMapping::calculate_relative_accuracy(
            gamma,
            CubicallyInterpolatedMapping::CORRECTING_FACTOR,
        );
        CubicallyInterpolatedMapping {
            relative_accuracy,
            gamma,
            index_offset,
            multiplier,
        }
    }

    pub fn decode(input: &mut impl Input) -> Result<CubicallyInterpolatedMapping, Error> {
        let gamma = input.read_double_le()?;
        let index_offset = input.read_double_le()?;
        Ok(CubicallyInterpolatedMapping::with_gamma_offset(
            gamma,
            index_offset,
        ))
    }

    fn log(&self, value: f64) -> f64 {
        let long_bits = value.to_bits() as i64;
        let s: f64 = serde::get_significand_plus_one(long_bits) - 1.0;
        let e: f64 = serde::get_exponent(long_bits) as f64;
        return ((CubicallyInterpolatedMapping::A * s + CubicallyInterpolatedMapping::B) * s
            + CubicallyInterpolatedMapping::C)
            * s
            + e;
    }

    fn log_inverse(&self, index: f64) -> f64 {
        let exponent = index.floor() as i64;
        // Derived from Cardano's formula
        let d0 = CubicallyInterpolatedMapping::B * CubicallyInterpolatedMapping::B
            - 3.0 * CubicallyInterpolatedMapping::A * CubicallyInterpolatedMapping::C;
        let d1 = 2.0
            * CubicallyInterpolatedMapping::B
            * CubicallyInterpolatedMapping::B
            * CubicallyInterpolatedMapping::B
            - 9.0
                * CubicallyInterpolatedMapping::A
                * CubicallyInterpolatedMapping::B
                * CubicallyInterpolatedMapping::C
            - 27.0
                * CubicallyInterpolatedMapping::A
                * CubicallyInterpolatedMapping::A
                * (index - exponent as f64);
        let p = ((d1 - (d1 * d1 - 4.0 * d0 * d0 * d0).sqrt()) / 2.0).cbrt();
        let significand_plus_one = -(CubicallyInterpolatedMapping::B + p + d0 / p)
            / (3.0 * CubicallyInterpolatedMapping::A)
            + 1.0;
        return serde::build_double(exponent, significand_plus_one);
    }

    fn calculate_relative_accuracy(gamma: f64, correcting_factor: f64) -> f64 {
        let exact_log_gamma = gamma.powf(correcting_factor);
        return (exact_log_gamma - 1.0) / (exact_log_gamma + 1.0);
    }

    fn calculate_gamma(relative_accuracy: f64, correcting_factor: f64) -> f64 {
        let exact_log_gamma = (1.0 + relative_accuracy) / (1.0 - relative_accuracy);
        exact_log_gamma.powf(1.0 / correcting_factor)
    }
}

impl IndexMapping for CubicallyInterpolatedMapping {
    fn index(&self, value: f64) -> i32 {
        let index: f64 = self.log(value) * self.multiplier + self.index_offset;
        return if index >= 0.0 {
            index as i32
        } else {
            (index - 1.0) as i32
        };
    }

    fn value(&self, index: i32) -> f64 {
        return self.lower_bound(index) * (1.0 + self.relative_accuracy);
    }

    fn lower_bound(&self, index: i32) -> f64 {
        self.log_inverse((index as f64 - self.index_offset) / self.multiplier)
    }

    fn upper_bound(&self, index: i32) -> f64 {
        self.lower_bound(index + 1)
    }

    fn get_relative_accuracy(&self) -> f64 {
        self.relative_accuracy
    }

    fn min_indexable_value(&self) -> f64 {
        f64::max(
            f64::powf(
                2.0,
                (i32::MIN as f64 - self.index_offset) / self.multiplier + 1.0,
            ),
            f64::MIN_POSITIVE * (1.0 + self.relative_accuracy) / (1.0 - self.relative_accuracy),
        )
    }

    fn max_indexable_value(&self) -> f64 {
        f64::max(
            f64::powf(
                2.0,
                (i32::MAX as f64 - self.index_offset) / self.multiplier - 1.0,
            ),
            f64::MAX / (1.0 + self.relative_accuracy),
        )
    }
}

impl ToString for CubicallyInterpolatedMapping {
    fn to_string(&self) -> String {
        format!(
            "CubicallyInterpolatedMapping{{gamma:{},indexOffset: {}}}",
            self.gamma, self.index_offset
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::index_mapping::impls::CubicallyInterpolatedMapping;
    use crate::index_mapping::IndexMapping;

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
        let mapping = CubicallyInterpolatedMapping::with_relative_accuracy(1e-2);

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
