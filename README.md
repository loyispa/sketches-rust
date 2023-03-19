# sketches-rust
This is a partial port of the [Java DDSketch](https://github.com/DataDog/sketches-java) quantile implementation writen by Rust. DDSketch is mergeable, meaning that multiple sketches from distributed systems can be combined in a central node.

# Features
It aims at as compatible as possible with Java implementations, here is some features has support: 
- ✅  CubicallyInterpolatedMapping implemention
- ✅  CollapsingLowestDense implemention
- ✅  merge with otehr sketch instance
- ✅  decode from input

Below will be add in the future:
- [ ] CollapsingHighestDense
- [ ] LogarithmicMapping 
- [ ] encode to output


# Usage
```rust
use self::sketches_rust::sketch::DDSketch;
let mut d = DDSketch::collapsing_lowest_dense(0.02,100);
d.accept(1.0);
d.accept(2.0);
d.accept(3.0);
let c = d.get_count();
assert_eq!(c, 3.0);
let q = d.get_value_at_quantile(0.5).unwrap();
assert!(q < 2.01 && q > 1.99);
```

