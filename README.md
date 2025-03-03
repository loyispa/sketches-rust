# sketches-rust
The port of the [DDSketch](https://github.com/DataDog/sketches-java) rust implementation. 

# Features
- CubicallyInterpolatedMapping 
- LogarithmicMapping
- CollapsingHighestDenseStore: collapse the highest bucket when reach specified size
- CollapsingLowestDenseStore: collapse the lowest bucket when reach specified size
- UnboundedSizeDenseStore: unlimited bucket
- Merge with other instance
- Serialize & Deserialize

# Usage

Run the following Cargo command in your project directory:
```
cargo add sketches-rust
```


Or add the following line to your Cargo.toml:
```
sketches-rust = "0.2.3"
```


Query:
```rust
    use sketches_rust::DDSketch;
    let mut d = DDSketch::collapsing_lowest_dense(0.02,100).unwrap();
    d.accept(1.0);
    d.accept(2.0);
    d.accept(3.0);
    let c = d.get_count();
    assert_eq!(c, 3.0);
    let q = d.get_value_at_quantile(0.5).unwrap();
    assert!(q < 2.01 && q > 1.99);
```

Merge:
```rust
    use sketches_rust::DDSketch;
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

Serialize:
```rust
    use sketches_rust::DDSketch;
    let mut d = DDSketch::unbounded_dense(2e-2).unwrap();
    d.accept(1.0);
    d.accept(2.0);
    d.accept(3.0);
    d.accept(4.0);
    d.accept(5.0);
    println!("encode: {:?}", d.encode().unwrap());

```

Deserialize:
```rust
    use sketches_rust::DDSketch;
    let mut input = vec![
        2, 42, 120, 57, 5, 47, 167, 240, 63, 0, 0, 0, 0, 0, 0, 0, 0, 13, 50, 130, 1, 2, 136, 32, 0,
        3, 0, 0, 0, 3, 0, 2, 0, 0, 3, 3, 2, 2, 3, 3, 2, 0, 0, 0, 0, 2, 0, 2, 2, 2, 4, 4, 132, 64,
        0, 4, 2, 0, 2, 2, 3, 132, 64, 4, 132, 64, 4, 2, 2, 0, 6, 4, 6, 132, 64, 2, 6,
    ];
    let mut d = DDSketch::decode(&input).unwrap();
    assert_eq!(d.get_count(), 100.0);
```

