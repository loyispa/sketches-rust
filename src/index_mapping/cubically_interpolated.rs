use super::*;
use crate::index_mapping::IndexMapping;
use crate::serde;

#[derive(PartialEq, Debug)]
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
    const BASE: f64 = 2.0;

    fn log(&self, value: f64) -> f64 {
        let long_bits = value.to_bits() as i64;
        let s: f64 = serde::get_significand_plus_one(long_bits) - 1.0;
        let e: f64 = serde::get_exponent(long_bits) as f64;
        ((CubicallyInterpolatedMapping::A * s + CubicallyInterpolatedMapping::B) * s
            + CubicallyInterpolatedMapping::C)
            * s
            + e
    }

    fn log_inverse(&self, index: f64) -> f64 {
        let exponent = index.floor() as i64;
        // Derived from Cardano's formula
        let d0: f64 = CubicallyInterpolatedMapping::B * CubicallyInterpolatedMapping::B
            - 3.0 * CubicallyInterpolatedMapping::A * CubicallyInterpolatedMapping::C;
        let d1: f64 = 2.0
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
                * (index - index.floor());
        let p: f64 = ((d1 - (d1 * d1 - 4.0 * d0 * d0 * d0).sqrt()) / 2.0).cbrt();
        let significand_plus_one: f64 = -(CubicallyInterpolatedMapping::B + p + d0 / p)
            / (3.0 * CubicallyInterpolatedMapping::A)
            + 1.0;
        serde::build_double(exponent, significand_plus_one)
    }
}

impl IndexMapping for CubicallyInterpolatedMapping {
    fn gamma(&self) -> f64 {
        self.gamma
    }

    fn index_offset(&self) -> f64 {
        self.index_offset
    }

    fn layout(&self) -> IndexMappingLayout {
        IndexMappingLayout::LogCubic
    }

    fn index(&self, value: f64) -> i32 {
        let index: f64 = self.log(value) * self.multiplier + self.index_offset;
        if index >= 0.0 {
            index as i32
        } else {
            (index - 1.0) as i32
        }
    }

    fn value(&self, index: i32) -> f64 {
        self.lower_bound(index) * (1.0 + self.relative_accuracy)
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

    fn with_relative_accuracy(
        relative_accuracy: f64,
    ) -> Result<CubicallyInterpolatedMapping, Error> {
        if relative_accuracy <= 0.0 || relative_accuracy >= 1.0 {
            return Err(Error::InvalidArgument(
                "The relative accuracy must be between 0 and 1.",
            ));
        }
        let gamma = calculate_gamma(
            relative_accuracy,
            CubicallyInterpolatedMapping::CORRECTING_FACTOR,
        );
        let index_offset: f64 = 0.0;

        let multiplier = CubicallyInterpolatedMapping::BASE.ln() / (gamma - 1.0).ln_1p();
        let relative_accuracy =
            calculate_relative_accuracy(gamma, CubicallyInterpolatedMapping::CORRECTING_FACTOR);
        Ok(CubicallyInterpolatedMapping {
            relative_accuracy,
            gamma,
            index_offset,
            multiplier,
        })
    }

    fn with_gamma_offset(
        gamma: f64,
        index_offset: f64,
    ) -> Result<CubicallyInterpolatedMapping, Error> {
        let multiplier = CubicallyInterpolatedMapping::BASE.ln() / gamma.ln();
        let relative_accuracy =
            calculate_relative_accuracy(gamma, CubicallyInterpolatedMapping::CORRECTING_FACTOR);
        Ok(CubicallyInterpolatedMapping {
            relative_accuracy,
            gamma,
            index_offset,
            multiplier,
        })
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
