use super::*;
use crate::index_mapping::IndexMapping;

#[derive(PartialEq)]
pub struct LogarithmicMapping {
    gamma: f64,
    index_offset: f64,
    multiplier: f64,
    relative_accuracy: f64,
}

impl LogarithmicMapping {
    const CORRECTING_FACTOR: f64 = 1.0;
    const BASE: f64 = std::f64::consts::E;

    pub fn with_relative_accuracy(relative_accuracy: f64) -> LogarithmicMapping {
        let gamma = calculate_gamma(relative_accuracy, LogarithmicMapping::CORRECTING_FACTOR);
        let index_offset: f64 = 0.0;
        let multiplier = LogarithmicMapping::BASE.ln() / gamma.ln();
        let relative_accuracy = calculate_relative_accuracy(gamma, 1.0);
        LogarithmicMapping {
            relative_accuracy,
            gamma,
            index_offset,
            multiplier,
        }
    }

    pub fn with_gamma_offset(gamma: f64, index_offset: f64) -> LogarithmicMapping {
        let multiplier = LogarithmicMapping::BASE.ln() / gamma.ln();
        let relative_accuracy =
            calculate_relative_accuracy(gamma, LogarithmicMapping::CORRECTING_FACTOR);
        LogarithmicMapping {
            relative_accuracy,
            gamma,
            index_offset,
            multiplier,
        }
    }

    fn log(&self, value: f64) -> f64 {
        value.ln()
    }

    fn log_inverse(&self, index: f64) -> f64 {
        index.exp()
    }
}

impl IndexMapping for LogarithmicMapping {
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

impl ToString for LogarithmicMapping {
    fn to_string(&self) -> String {
        format!(
            "LogarithmicMapping{{gamma:{},indexOffset: {}}}",
            self.gamma, self.index_offset
        )
    }
}
