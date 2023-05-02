use crate::sketch::{Flag, FlagType};
use crate::{serde, Error};

use crate::output::Output;

#[derive(PartialEq, Debug)]
pub enum IndexMapping {
    LogarithmicMapping(f64, f64, f64, f64),
    CubicallyInterpolatedMapping(f64, f64, f64, f64),
}

const CUBICALLY_INTERPOLATED_MAPPING_A: f64 = 6.0 / 35.0;
const CUBICALLY_INTERPOLATED_MAPPING_B: f64 = -3.0 / 5.0;
const CUBICALLY_INTERPOLATED_MAPPING_C: f64 = 10.0 / 7.0;
const CUBICALLY_INTERPOLATED_MAPPING_CORRECTING_FACTOR: f64 =
    1.0 / (CUBICALLY_INTERPOLATED_MAPPING_C * std::f64::consts::LN_2);
const CUBICALLY_INTERPOLATED_MAPPING_BASE: f64 = 2.0;
const LOGARITHMIC_MAPPING_CORRECTING_FACTOR: f64 = 1.0;
const LOGARITHMIC_MAPPING_BASE: f64 = std::f64::consts::E;

impl IndexMapping {
    pub fn layout(&self) -> IndexMappingLayout {
        match self {
            IndexMapping::LogarithmicMapping(
                _gamma,
                _index_offset,
                _multiplier,
                _relative_accuracy,
            ) => IndexMappingLayout::LOG,
            IndexMapping::CubicallyInterpolatedMapping(
                _gamma,
                _index_offset,
                _multiplier,
                _relative_accuracy,
            ) => IndexMappingLayout::LogCubic,
        }
    }

    pub fn gamma(&self) -> f64 {
        match self {
            IndexMapping::LogarithmicMapping(
                gamma,
                _index_offset,
                _multiplier,
                _relative_accuracy,
            ) => *gamma,
            IndexMapping::CubicallyInterpolatedMapping(
                gamma,
                _index_offset,
                _multiplier,
                _relative_accuracy,
            ) => *gamma,
        }
    }

    pub fn index_offset(&self) -> f64 {
        match self {
            IndexMapping::LogarithmicMapping(
                _gamma,
                index_offset,
                _multiplier,
                _relative_accuracy,
            ) => *index_offset,
            IndexMapping::CubicallyInterpolatedMapping(
                _gamma,
                index_offset,
                _multiplier,
                _relative_accuracy,
            ) => *index_offset,
        }
    }

    pub fn multiplier(&self) -> f64 {
        match self {
            IndexMapping::LogarithmicMapping(
                _gamma,
                _index_offset,
                multiplier,
                _relative_accuracy,
            ) => *multiplier,
            IndexMapping::CubicallyInterpolatedMapping(
                _gamma,
                _index_offset,
                multiplier,
                _relative_accuracy,
            ) => *multiplier,
        }
    }

    pub fn relative_accuracy(&self) -> f64 {
        match self {
            IndexMapping::LogarithmicMapping(
                _gamma,
                _index_offset,
                _multiplier,
                relative_accuracy,
            ) => *relative_accuracy,
            IndexMapping::CubicallyInterpolatedMapping(
                _gamma,
                _index_offset,
                _multiplier,
                relative_accuracy,
            ) => *relative_accuracy,
        }
    }

    fn log(&self, value: f64) -> f64 {
        match self {
            IndexMapping::LogarithmicMapping(
                _gamma,
                _index_offset,
                _multiplier,
                _relative_accuracy,
            ) => value.ln(),
            IndexMapping::CubicallyInterpolatedMapping(
                _gamma,
                _index_offset,
                _multiplier,
                _relative_accuracy,
            ) => {
                let long_bits = value.to_bits() as i64;
                let s: f64 = serde::get_significand_plus_one(long_bits) - 1.0;
                let e: f64 = serde::get_exponent(long_bits) as f64;
                ((CUBICALLY_INTERPOLATED_MAPPING_A * s + CUBICALLY_INTERPOLATED_MAPPING_B) * s
                    + CUBICALLY_INTERPOLATED_MAPPING_C)
                    * s
                    + e
            }
        }
    }

    fn log_inverse(&self, index: f64) -> f64 {
        match self {
            IndexMapping::LogarithmicMapping(
                _gamma,
                _index_offset,
                _multiplier,
                _relative_accuracy,
            ) => index.exp(),
            IndexMapping::CubicallyInterpolatedMapping(
                _gamma,
                _index_offset,
                _multiplier,
                _relative_accuracy,
            ) => {
                let exponent = index.floor() as i64;
                // Derived from Cardano's formula
                let d0: f64 = CUBICALLY_INTERPOLATED_MAPPING_B * CUBICALLY_INTERPOLATED_MAPPING_B
                    - 3.0 * CUBICALLY_INTERPOLATED_MAPPING_A * CUBICALLY_INTERPOLATED_MAPPING_C;
                let d1: f64 = 2.0
                    * CUBICALLY_INTERPOLATED_MAPPING_B
                    * CUBICALLY_INTERPOLATED_MAPPING_B
                    * CUBICALLY_INTERPOLATED_MAPPING_B
                    - 9.0
                        * CUBICALLY_INTERPOLATED_MAPPING_A
                        * CUBICALLY_INTERPOLATED_MAPPING_B
                        * CUBICALLY_INTERPOLATED_MAPPING_C
                    - 27.0
                        * CUBICALLY_INTERPOLATED_MAPPING_A
                        * CUBICALLY_INTERPOLATED_MAPPING_A
                        * (index - index.floor());
                let p: f64 = ((d1 - (d1 * d1 - 4.0 * d0 * d0 * d0).sqrt()) / 2.0).cbrt();
                let significand_plus_one: f64 = -(CUBICALLY_INTERPOLATED_MAPPING_B + p + d0 / p)
                    / (3.0 * CUBICALLY_INTERPOLATED_MAPPING_A)
                    + 1.0;
                serde::build_double(exponent, significand_plus_one)
            }
        }
    }

    pub fn index(&self, value: f64) -> i32 {
        let index: f64 = self.log(value) * self.multiplier() + self.index_offset();
        if index >= 0.0 {
            index as i32
        } else {
            (index - 1.0) as i32
        }
    }

    pub fn value(&self, index: i32) -> f64 {
        self.lower_bound(index) * (1.0 + self.relative_accuracy())
    }

    fn lower_bound(&self, index: i32) -> f64 {
        self.log_inverse((index as f64 - self.index_offset()) / self.multiplier())
    }

    #[allow(dead_code)]
    fn upper_bound(&self, index: i32) -> f64 {
        self.lower_bound(index + 1)
    }

    pub(crate) fn min_indexable_value(&self) -> f64 {
        f64::max(
            f64::powf(
                2.0,
                (i32::MIN as f64 - self.index_offset()) / self.multiplier() + 1.0,
            ),
            f64::MIN_POSITIVE * (1.0 + self.relative_accuracy()) / (1.0 - self.relative_accuracy()),
        )
    }

    pub(crate) fn max_indexable_value(&self) -> f64 {
        f64::max(
            f64::powf(
                2.0,
                (i32::MAX as f64 - self.index_offset()) / self.multiplier() - 1.0,
            ),
            f64::MAX / (1.0 + self.relative_accuracy()),
        )
    }

    pub fn encode(&self, output: &mut Output) -> Result<(), Error> {
        self.layout().to_flag().encode(output)?;
        output.write_double_le(self.gamma())?;
        output.write_double_le(self.index_offset())?;
        Ok(())
    }

    pub fn with_relative_accuracy(
        index_layout: IndexMappingLayout,
        relative_accuracy: f64,
    ) -> Result<IndexMapping, Error> {
        if relative_accuracy <= 0.0 || relative_accuracy >= 1.0 {
            return Err(Error::InvalidArgument(
                "The relative accuracy must be between 0 and 1.",
            ));
        }

        match index_layout {
            IndexMappingLayout::LOG => {
                if relative_accuracy <= 0.0 || relative_accuracy >= 1.0 {
                    return Err(Error::InvalidArgument(
                        "The relative accuracy must be between 0 and 1.",
                    ));
                }

                let gamma =
                    calculate_gamma(relative_accuracy, LOGARITHMIC_MAPPING_CORRECTING_FACTOR);
                let index_offset: f64 = 0.0;
                let multiplier = LOGARITHMIC_MAPPING_BASE.ln() / (gamma - 1.0).ln_1p();
                let relative_accuracy = calculate_relative_accuracy(gamma, 1.0);
                Ok(IndexMapping::LogarithmicMapping(
                    gamma,
                    index_offset,
                    multiplier,
                    relative_accuracy,
                ))
            }

            IndexMappingLayout::LogCubic => {
                let gamma = calculate_gamma(
                    relative_accuracy,
                    CUBICALLY_INTERPOLATED_MAPPING_CORRECTING_FACTOR,
                );
                let index_offset: f64 = 0.0;

                let multiplier = CUBICALLY_INTERPOLATED_MAPPING_BASE.ln() / (gamma - 1.0).ln_1p();
                let relative_accuracy = calculate_relative_accuracy(
                    gamma,
                    CUBICALLY_INTERPOLATED_MAPPING_CORRECTING_FACTOR,
                );
                Ok(IndexMapping::CubicallyInterpolatedMapping(
                    gamma,
                    index_offset,
                    multiplier,
                    relative_accuracy,
                ))
            }
            _ => Err(Error::InvalidArgument("Unsupported IndexLayout")),
        }
    }

    pub fn with_gamma_offset(
        index_layout: IndexMappingLayout,
        gamma: f64,
        index_offset: f64,
    ) -> Result<IndexMapping, Error> {
        match index_layout {
            IndexMappingLayout::LOG => {
                let multiplier = LOGARITHMIC_MAPPING_BASE.ln() / gamma.ln();
                let relative_accuracy =
                    calculate_relative_accuracy(gamma, LOGARITHMIC_MAPPING_CORRECTING_FACTOR);
                Ok(IndexMapping::LogarithmicMapping(
                    gamma,
                    index_offset,
                    multiplier,
                    relative_accuracy,
                ))
            }

            IndexMappingLayout::LogCubic => {
                let multiplier = CUBICALLY_INTERPOLATED_MAPPING_BASE.ln() / gamma.ln();
                let relative_accuracy = calculate_relative_accuracy(
                    gamma,
                    CUBICALLY_INTERPOLATED_MAPPING_CORRECTING_FACTOR,
                );
                Ok(IndexMapping::CubicallyInterpolatedMapping(
                    gamma,
                    index_offset,
                    multiplier,
                    relative_accuracy,
                ))
            }

            _ => Err(Error::InvalidArgument("Unsupported IndexLayout")),
        }
    }
}

pub enum IndexMappingLayout {
    LOG = 0,
    LogLinear = 1,
    LogQuadratic = 2,
    LogCubic = 3,
    LogQuartic = 4,
}

impl IndexMappingLayout {
    pub fn of_flag(flag: &Flag) -> Result<IndexMappingLayout, Error> {
        let index = flag.get_marker() >> 2;
        match index {
            0 => Ok(IndexMappingLayout::LOG),
            1 => Ok(IndexMappingLayout::LogLinear),
            2 => Ok(IndexMappingLayout::LogQuadratic),
            3 => Ok(IndexMappingLayout::LogCubic),
            4 => Ok(IndexMappingLayout::LogQuartic),
            _ => Err(Error::InvalidArgument("Unknown Index Flag.")),
        }
    }

    pub fn to_flag(self) -> Flag {
        let sub_flag = self as u8;
        Flag::with_type(FlagType::IndexMapping, sub_flag)
    }
}

fn calculate_relative_accuracy(gamma: f64, correcting_factor: f64) -> f64 {
    let exact_log_gamma = gamma.powf(correcting_factor);
    (exact_log_gamma - 1.0) / (exact_log_gamma + 1.0)
}

fn calculate_gamma(relative_accuracy: f64, correcting_factor: f64) -> f64 {
    let exact_log_gamma = (1.0 + relative_accuracy) / (1.0 - relative_accuracy);
    exact_log_gamma.powf(1.0 / correcting_factor)
}

#[cfg(test)]
mod tests {
    use crate::index_mapping::IndexMapping;
    use crate::index_mapping::IndexMappingLayout::{LogCubic, LOG};

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
                    IndexMapping::with_gamma_offset(LogCubic, gamma, index_offset).unwrap();
                assert_eq!(accuracy[index], index_mapping.relative_accuracy());
                index += 1;
            }
        }
    }

    #[test]
    fn test_cubically_interpolated_mapping_offset() {
        for gamma in TEST_GAMMAS {
            for index_offset in TEST_INDEX_OFFSETS {
                let index_mapping =
                    IndexMapping::with_gamma_offset(LogCubic, gamma, index_offset).unwrap();
                let index_of1 = index_mapping.index(1.0) as f64;
                // If 1 is on a bucket boundary, its associated index can be either of the ones of the previous
                // and the next buckets.
                assert!(index_offset.ceil() - 1.0 <= index_of1);
                assert!(index_of1 <= index_offset.floor());
            }
        }
    }

    #[test]
    fn test_logarithmic_mapping_offset() {
        for gamma in TEST_GAMMAS {
            for index_offset in TEST_INDEX_OFFSETS {
                let index_mapping =
                    IndexMapping::with_gamma_offset(LOG, gamma, index_offset).unwrap();
                let index_of1 = index_mapping.index(1.0) as f64;
                // If 1 is on a bucket boundary, its associated index can be either of the ones of the previous
                // and the next buckets.
                assert!(index_offset.ceil() - 1.0 <= index_of1);
                assert!(index_of1 <= index_offset.floor());
            }
        }
    }

    #[test]
    fn test_cubically_interpolated_mapping_validity_manual_check() {
        let d0: f64 = -0.37469387755102035;
        let d1: f64 = 0.8904489795918369;
        let s1 = d1 * d1 - 4.0_f64 * d0 * d0 * d0;
        let s2 = s1.sqrt();
        let s3 = d1 - s2;
        let s4 = s3 / 2.0;
        let s5 = s4.cbrt();
        eprintln!(
            "test_cubically_interpolated_mapping_validity_manual_check: {} {} {} {} {}",
            s1, s2, s3, s4, s5
        );
    }

    #[test]
    fn test_cubically_interpolated_mapping_validity() {
        let mapping = IndexMapping::with_relative_accuracy(LogCubic, 1e-2).unwrap();

        println!("CubicallyInterpolatedMapping: {:?}", mapping);

        let min_index = -50;
        let max_index = 50;

        let mut index = min_index;
        let mut bound = mapping.upper_bound(index - 1);

        while index <= max_index {
            println!(
                "test_cubically_interpolated_mapping_validity {} {} {} {}",
                index,
                mapping.value(index),
                mapping.lower_bound(index),
                mapping.upper_bound(index)
            );

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

    #[test]
    fn test_logarithmic_mapping_validity() {
        let mapping = IndexMapping::with_relative_accuracy(LOG, 1e-2).unwrap();

        println!("LogarithmicMapping: {:?}", mapping);

        let min_index = -50;
        let max_index = 50;

        let mut index = min_index;
        let mut bound = mapping.upper_bound(index - 1);

        while index <= max_index {
            println!(
                "test_logarithmic_mapping_validity {} {} {} {}",
                index,
                mapping.value(index),
                mapping.lower_bound(index),
                mapping.upper_bound(index)
            );

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

    #[test]
    fn test_logarithmic_mapping_index() {
        let mapping = IndexMapping::with_relative_accuracy(LOG, 2e-2).unwrap();
        let values: Vec<f64> = vec![
            1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0, 13.0, 14.0, 15.0, 16.0,
            17.0, 18.0, 19.0, 228.0, 484.0, 499.0, 559.0, 584.0, 629.0, 722.0, 730.0, 777.0, 805.0,
            846.0, 896.0, 997.0, 999.0, 1065.0, 1178.0, 1189.0, 1218.0, 1255.0, 1308.0, 1343.0,
            1438.0, 1819.0, 2185.0, 2224.0, 2478.0, 2574.0, 2601.0, 2745.0, 2950.0, 3013.0, 3043.0,
            3064.0, 3116.0, 3188.0, 3224.0, 3254.0, 3390.0, 3476.0, 3543.0, 3836.0, 3921.0, 4014.0,
            4074.0, 4332.0, 4344.0, 4456.0, 4736.0, 4984.0, 5219.0, 5244.0, 5259.0, 5341.0, 5467.0,
            5536.0, 5600.0, 6054.0, 6061.0, 6118.0, 6137.0, 6222.0, 6263.0, 6320.0, 6454.0, 6499.0,
            6732.0, 6922.0, 6988.0, 7047.0, 7057.0, 7202.0, 7205.0, 7330.0, 7507.0, 7616.0, 7971.0,
            8056.0, 8381.0, 8416.0, 8684.0, 8784.0, 8790.0, 8823.0, 8841.0, 8945.0, 8967.0, 8982.0,
            9142.0, 9181.0, 9284.0, 9320.0, 9331.0, 9596.0, 9699.0, 9850.0, 9884.0, 9947.0,
        ];
        let indexes = vec![
            0, 17, 27, 34, 40, 44, 48, 51, 54, 57, 59, 62, 64, 65, 67, 69, 70, 72, 73, 135, 154,
            155, 158, 159, 161, 164, 164, 166, 167, 168, 169, 172, 172, 174, 176, 176, 177, 178,
            179, 180, 181, 187, 192, 192, 195, 196, 196, 197, 199, 200, 200, 200, 201, 201, 201,
            202, 203, 203, 204, 206, 206, 207, 207, 209, 209, 210, 211, 212, 213, 214, 214, 214,
            215, 215, 215, 217, 217, 217, 218, 218, 218, 218, 219, 219, 220, 221, 221, 221, 221,
            222, 222, 222, 223, 223, 224, 224, 225, 225, 226, 226, 227, 227, 227, 227, 227, 227,
            227, 228, 228, 228, 228, 229, 229, 229, 229, 230,
        ];
        for i in 0..values.len() {
            assert_eq!(indexes[i], mapping.index(values[i]));
        }
    }

    #[test]
    fn test_cubically_interpolated_index() {
        let mapping = IndexMapping::with_relative_accuracy(LogCubic, 2e-2).unwrap();
        let values: Vec<f64> = vec![
            1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0, 13.0, 14.0, 15.0, 16.0,
            17.0, 18.0, 19.0, 125.0, 189.0, 379.0, 444.0, 613.0, 639.0, 671.0, 834.0, 983.0,
            1067.0, 1093.0, 1159.0, 1336.0, 1370.0, 1481.0, 1527.0, 1633.0, 1662.0, 1735.0, 1822.0,
            1864.0, 1957.0, 2018.0, 2308.0, 2327.0, 2337.0, 2433.0, 2450.0, 2491.0, 2520.0, 2582.0,
            2599.0, 2719.0, 2907.0, 3086.0, 3153.0, 3170.0, 3288.0, 3372.0, 3397.0, 3508.0, 3517.0,
            3987.0, 4020.0, 4092.0, 4114.0, 4141.0, 4194.0, 4308.0, 4552.0, 4950.0, 5152.0, 5422.0,
            5452.0, 5997.0, 6076.0, 6100.0, 6132.0, 6170.0, 6202.0, 6210.0, 6259.0, 6285.0, 6345.0,
            6389.0, 6390.0, 6441.0, 6650.0, 6897.0, 6898.0, 6909.0, 6923.0, 6944.0, 6970.0, 7233.0,
            7289.0, 7304.0, 7437.0, 7585.0, 7756.0, 7808.0, 7862.0, 7953.0, 8054.0, 8095.0, 8161.0,
            8422.0, 8551.0, 8567.0, 8766.0, 8922.0, 8966.0, 9206.0, 9250.0, 9372.0, 9397.0, 9434.0,
            9505.0,
        ];
        let indexes = vec![
            0, 17, 27, 34, 40, 45, 49, 52, 55, 58, 60, 62, 64, 66, 68, 69, 71, 72, 74, 121, 132,
            149, 153, 162, 163, 164, 169, 173, 176, 176, 178, 181, 182, 184, 185, 186, 187, 188,
            189, 190, 191, 192, 195, 195, 195, 196, 196, 197, 197, 198, 198, 199, 201, 202, 203,
            203, 204, 205, 205, 206, 206, 209, 209, 209, 210, 210, 210, 211, 212, 214, 215, 217,
            217, 219, 219, 220, 220, 220, 220, 220, 220, 220, 221, 221, 221, 221, 222, 223, 223,
            223, 223, 223, 223, 224, 224, 224, 225, 225, 226, 226, 226, 226, 227, 227, 227, 228,
            228, 228, 229, 229, 229, 230, 230, 230, 230, 231, 231,
        ];
        for i in 0..values.len() {
            assert_eq!(indexes[i], mapping.index(values[i]));
        }
    }
}
