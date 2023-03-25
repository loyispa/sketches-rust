use crate::Error;
use crate::sketch::Flag;

pub mod impls;

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
            0 => { Ok(IndexMappingLayout::LOG) }
            1 => { Ok(IndexMappingLayout::LogLinear) }
            2 => { Ok(IndexMappingLayout::LogQuadratic) }
            3 => { Ok(IndexMappingLayout::LogCubic) }
            4 => { Ok(IndexMappingLayout::LogQuartic) }
            _ => { Err(Error::InvalidArgument("unknown flag")) }
        };
    }
}