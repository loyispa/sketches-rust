# sketches-rust
This is a partial port of the [Java DDSketch](https://github.com/DataDog/sketches-java) quantile implementation writen by Rust. DDSketch is mergeable, meaning that multiple sketches from distributed systems can be combined in a central node.

# Features
It aims at as compatible as possible with Java implementations, here is some features has support: 
- [x] CubicallyInterpolatedMapping 
- [x] LogarithmicMapping
- [x] CollapsingHighestDense
- [x] CollapsingLowestDense 
- [x] UnboundedSizeDenseStore
- [x] Mergeable
- [x] Decode from input

Below will be add in the future:
- [ ] Encode to output


# Usage
```rust
let mut d = DDSketch::collapsing_lowest_dense(0.02, 100).unwrap();
d.accept(1.0);
d.accept(2.0);
d.accept(3.0);
assert_eq!(d.get_count(), 3.0);
println!("{}", d.get_max().unwrap());
println!("{}", d.get_min().unwrap());
println!("{}", d.get_sum().unwrap());
println!("{}", d.get_average().unwrap());
```

