pub mod impls;

pub trait IndexMapping {
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