/*!
This crate provides a direct port of the [Java](https://github.com/DataDog/sketches-java)
[DDSketch](https://arxiv.org/pdf/1908.10693.pdf) implementation to Rust.
# Usage
Add multiple samples to a DDSketch and invoke the `get_value_at_quantile` method to pull any quantile from
*0.0* to *1.0*.

```rust
    use self::sketches_rust::DDSketch;
    let mut d = DDSketch::collapsing_lowest_dense(0.02,100).unwrap();
    d.accept(1.0);
    d.accept(2.0);
    d.accept(3.0);
    let c = d.get_count();
    assert_eq!(c, 3.0);
    let q = d.get_value_at_quantile(0.5).unwrap();
    assert!(q < 2.01 && q > 1.99);
```

Also you could merge other DDSketch:
```rust
    use self::sketches_rust::DDSketch;
    let mut d1 = DDSketch::collapsing_lowest_dense(0.02,100).unwrap();
    d1.accept(1.0);
    d1.accept(2.0);
    d1.accept(3.0);
    assert_eq!(3.0,  d1.get_count());
    let mut d2 = DDSketch::collapsing_lowest_dense(0.02,100).unwrap();
    d2.accept(1.0);
    d2.accept(2.0);
    d2.accept(3.0);
    assert_eq!(3.0,  d2.get_count());
    d2.merge_with(&mut d1).unwrap();
    assert_eq!(6.0,  d2.get_count());
```

 */

mod error;
pub mod index_mapping;
mod input;
mod output;
mod serde;
mod sketch;
pub mod store;

pub use self::error::Error;
pub use self::index_mapping::CubicallyInterpolatedMapping;
pub use self::index_mapping::LogarithmicMapping;
pub use self::input::DefaultInput;
pub use self::output::DefaultOutput;
pub use self::sketch::DDSketch;
pub use self::store::CollapsingHighestDenseStore;
pub use self::store::CollapsingLowestDenseStore;
pub use self::store::UnboundedSizeDenseStore;
