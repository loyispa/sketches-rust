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

Encode to bytes:
```rust
    use self::sketches_rust::{DDSketch};
    let mut d = DDSketch::unbounded_dense(2e-2).unwrap();
    d.accept(1.0);
    d.accept(2.0);
    d.accept(3.0);
    d.accept(4.0);
    d.accept(5.0);
    println!("encode: {:?}", d.encode().unwrap());
```

Decode from bytes:
```rust
    use self::sketches_rust::{DDSketch};
    let mut input = vec![
        2, 42, 120, 57, 5, 47, 167, 240, 63, 0, 0, 0, 0, 0, 0, 0, 0, 13, 50, 130, 1, 2, 136, 32, 0,
        3, 0, 0, 0, 3, 0, 2, 0, 0, 3, 3, 2, 2, 3, 3, 2, 0, 0, 0, 0, 2, 0, 2, 2, 2, 4, 4, 132, 64,
        0, 4, 2, 0, 2, 2, 3, 132, 64, 4, 132, 64, 4, 2, 2, 0, 6, 4, 6, 132, 64, 2, 6,
    ];
    let mut d = DDSketch::decode(&input).unwrap();
    assert_eq!(d.get_count(), 100.0);
```
 */

mod error;
mod index_mapping;
mod input;
mod output;
mod serde;
mod sketch;
mod store;

pub use self::error::Error;
pub use self::sketch::DDSketch;
