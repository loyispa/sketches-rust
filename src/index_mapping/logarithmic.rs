use super::*;
use crate::index_mapping::IndexMapping;

#[derive(PartialEq, Debug)]
pub struct LogarithmicMapping {
    gamma: f64,
    index_offset: f64,
    multiplier: f64,
    relative_accuracy: f64,
}

impl LogarithmicMapping {
    const CORRECTING_FACTOR: f64 = 1.0;
    const BASE: f64 = std::f64::consts::E;

    fn log(&self, value: f64) -> f64 {
        value.ln()
    }

    fn log_inverse(&self, index: f64) -> f64 {
        index.exp()
    }
}

impl IndexMapping for LogarithmicMapping {
    fn gamma(&self) -> f64 {
        self.gamma
    }

    fn index_offset(&self) -> f64 {
        self.index_offset
    }

    fn layout(&self) -> IndexMappingLayout {
        IndexMappingLayout::LOG
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

    fn with_relative_accuracy(relative_accuracy: f64) -> Result<LogarithmicMapping, Error> {
        if relative_accuracy <= 0.0 || relative_accuracy >= 1.0 {
            return Err(Error::InvalidArgument(
                "The relative accuracy must be between 0 and 1.",
            ));
        }

        let gamma = calculate_gamma(relative_accuracy, LogarithmicMapping::CORRECTING_FACTOR);
        let index_offset: f64 = 0.0;
        let multiplier = LogarithmicMapping::BASE.ln() / (gamma - 1.0).ln_1p();
        let relative_accuracy = calculate_relative_accuracy(gamma, 1.0);
        Ok(LogarithmicMapping {
            relative_accuracy,
            gamma,
            index_offset,
            multiplier,
        })
    }

    fn with_gamma_offset(gamma: f64, index_offset: f64) -> Result<LogarithmicMapping, Error> {
        let multiplier = LogarithmicMapping::BASE.ln() / gamma.ln();
        let relative_accuracy =
            calculate_relative_accuracy(gamma, LogarithmicMapping::CORRECTING_FACTOR);
        Ok(LogarithmicMapping {
            relative_accuracy,
            gamma,
            index_offset,
            multiplier,
        })
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
