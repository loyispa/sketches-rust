use sketches_rust::DDSketch;
use sketches_rust::{
    CollapsingHighestDenseStore, CollapsingLowestDenseStore, CubicallyInterpolatedMapping,
    LogarithmicMapping, UnboundedSizeDenseStore,
};

#[test]
#[should_panic]
fn test_sketch_crate_panic_0() {
    DDSketch::collapsing_lowest_dense(0.00, 100).unwrap();
}
#[test]
#[should_panic]
fn test_sketch_crate_panic_1() {
    DDSketch::collapsing_lowest_dense(1.00, 100).unwrap();
}
#[test]
#[should_panic]
fn test_sketch_crate_panic_2() {
    DDSketch::collapsing_lowest_dense(0.02, 2147483648).unwrap();
}

#[test]
fn test_sketch_quantile_0() {
    let mut sketch = DDSketch::collapsing_lowest_dense(0.02, 100).unwrap();
    sketch.accept(1.0);
    sketch.accept(2.0);
    sketch.accept(3.0);
    sketch.accept(4.0);
    sketch.accept(5.0);

    assert!((f64::abs(sketch.get_value_at_quantile(0.0).unwrap() - 1.0) / 1.0) < 0.021);
    assert!((f64::abs(sketch.get_value_at_quantile(0.5).unwrap() - 3.0) / 3.0) < 0.021);
    assert!((f64::abs(sketch.get_value_at_quantile(1.0).unwrap() - 5.0) / 5.0) < 0.021);
}

#[test]
fn test_sketch_quantile_1() {
    let mut sketch = DDSketch::collapsing_highest_dense(0.02, 100).unwrap();
    sketch.accept(1.0);
    sketch.accept(2.0);
    sketch.accept(3.0);
    sketch.accept(4.0);
    sketch.accept(5.0);

    assert!((f64::abs(sketch.get_value_at_quantile(0.0).unwrap() - 1.0) / 1.0) < 0.021);
    assert!((f64::abs(sketch.get_value_at_quantile(0.5).unwrap() - 3.0) / 3.0) < 0.021);
    assert!((f64::abs(sketch.get_value_at_quantile(1.0).unwrap() - 5.0) / 5.0) < 0.021);
}

#[test]
fn test_sketch_quantile_2() {
    let mut sketch = DDSketch::unbounded_dense(0.02).unwrap();
    sketch.accept(1.0);
    sketch.accept(2.0);
    sketch.accept(3.0);
    sketch.accept(4.0);
    sketch.accept(5.0);

    assert!((f64::abs(sketch.get_value_at_quantile(0.0).unwrap() - 1.0) / 1.0) < 0.021);
    assert!((f64::abs(sketch.get_value_at_quantile(0.5).unwrap() - 3.0) / 3.0) < 0.021);
    assert!((f64::abs(sketch.get_value_at_quantile(1.0).unwrap() - 5.0) / 5.0) < 0.021);
}

#[test]
fn test_sketch_quantile_3() {
    let mut sketch = DDSketch::logarithmic_collapsing_lowest_dense(0.02, 100).unwrap();
    sketch.accept(1.0);
    sketch.accept(2.0);
    sketch.accept(3.0);
    sketch.accept(4.0);
    sketch.accept(5.0);

    assert!((f64::abs(sketch.get_value_at_quantile(0.0).unwrap() - 1.0) / 1.0) < 0.021);
    assert!((f64::abs(sketch.get_value_at_quantile(0.5).unwrap() - 3.0) / 3.0) < 0.021);
    assert!((f64::abs(sketch.get_value_at_quantile(1.0).unwrap() - 5.0) / 5.0) < 0.021);
}

#[test]
fn test_sketch_quantile_4() {
    let mut sketch = DDSketch::logarithmic_collapsing_highest_dense(0.02, 100).unwrap();
    sketch.accept(1.0);
    sketch.accept(2.0);
    sketch.accept(3.0);
    sketch.accept(4.0);
    sketch.accept(5.0);

    assert!((f64::abs(sketch.get_value_at_quantile(0.0).unwrap() - 1.0) / 1.0) < 0.021);
    assert!((f64::abs(sketch.get_value_at_quantile(0.5).unwrap() - 3.0) / 3.0) < 0.021);
    assert!((f64::abs(sketch.get_value_at_quantile(1.0).unwrap() - 5.0) / 5.0) < 0.021);
}

#[test]
fn test_sketch_add() {
    let accuracy = 2e-2;

    let mut sketch = DDSketch::collapsing_lowest_dense(accuracy, 50).unwrap();

    for i in -99..101 {
        sketch.accept(i as f64);
    }

    assert_eq!(200.0, sketch.get_count());
    assert!((f64::abs(sketch.get_min().unwrap() - -99.0) / -99.0) <= accuracy);
    assert!((f64::abs(sketch.get_max().unwrap() - 100.0) / 100.0) <= accuracy);
    assert!((f64::abs(sketch.get_average().unwrap() - 0.5) / 0.5) <= accuracy);
    assert!((f64::abs(sketch.get_sum().unwrap() - 100.0) / 100.0) <= accuracy);
}

#[test]
fn test_sketch_merge_1() {
    let accuracy = 2e-2;

    let mut sketch1 = DDSketch::collapsing_lowest_dense(accuracy, 50).unwrap();
    for i in -99..101 {
        sketch1.accept(i as f64);
    }

    let mut sketch2 = DDSketch::collapsing_lowest_dense(accuracy, 50).unwrap();
    for i in 100..200 {
        sketch2.accept(i as f64);
    }

    sketch1.merge_with(&mut sketch2).unwrap();
    assert_eq!(300.0, sketch1.get_count());
}

#[test]
fn test_sketch_merge_2() {
    let accuracy = 2e-2;

    let mut sketch1 = DDSketch::collapsing_lowest_dense(accuracy, 50).unwrap();
    for i in -99..101 {
        sketch1.accept(i as f64);
    }

    let mut sketch2 = DDSketch::unbounded_dense(accuracy).unwrap();
    for i in 100..200 {
        sketch2.accept(i as f64);
    }

    sketch1.merge_with(&mut sketch2).unwrap();
    assert_eq!(300.0, sketch1.get_count());
}

#[test]
#[should_panic]
fn test_sketch_merge_panic() {
    let mut sketch1 = DDSketch::collapsing_lowest_dense(1e-2, 50).unwrap();
    for i in -99..101 {
        sketch1.accept(i as f64);
    }

    let mut sketch2 = DDSketch::collapsing_lowest_dense(2e-2, 50).unwrap();
    for i in 100..200 {
        sketch2.accept(i as f64);
    }

    sketch1.merge_with(&mut sketch2).unwrap();
}

#[test]
fn test_sketch_decode_1() {
    let accuracy = 2e-2;
    let input = vec![
        14, 100, 244, 7, 173, 131, 165, 240, 63, 0, 0, 0, 0, 0, 0, 0, 0, 5, 21, 0, 140, 48, 34,
        150, 241, 16, 20, 148, 191, 96, 14, 142, 62, 12, 139, 16, 10, 134, 96, 8, 3, 6, 2, 6, 2, 6,
        2, 4, 2, 42, 2, 26, 2, 6, 2, 20, 2, 6, 2, 2, 2, 10, 2, 20, 2, 14, 2, 10, 2,
    ];
    let mut sketch = DDSketch::collapsing_lowest_dense(accuracy, 50).unwrap();
    sketch.decode_and_merge_with(input).unwrap();
    assert_eq!(4538.0, sketch.get_count());
}

#[test]
fn test_sketch_decode_2() {
    let accuracy = 2e-2;
    let input = vec![
        14, 100, 244, 7, 173, 131, 165, 240, 63, 0, 0, 0, 0, 0, 0, 0, 0, 5, 10, 7, 2, 18, 2, 38, 2,
        2, 4, 4, 2, 4, 2, 12, 3, 6, 2, 2, 2, 12, 140, 100,
    ];
    let mut sketch = DDSketch::collapsing_highest_dense(accuracy, 50).unwrap();
    sketch.decode_and_merge_with(input).unwrap();
    assert_eq!(100.0, sketch.get_count());
}

#[test]
fn test_sketch_decode_3() {
    let input = vec![
        2, 42, 120, 57, 5, 47, 167, 240, 63, 0, 0, 0, 0, 0, 0, 0, 0, 13, 50, 130, 1, 2, 136, 32, 0,
        3, 0, 0, 0, 3, 0, 2, 0, 0, 3, 3, 2, 2, 3, 3, 2, 0, 0, 0, 0, 2, 0, 2, 2, 2, 4, 4, 132, 64,
        0, 4, 2, 0, 2, 2, 3, 132, 64, 4, 132, 64, 4, 2, 2, 0, 6, 4, 6, 132, 64, 2, 6,
    ];
    let mut sketch = DDSketch::logarithmic_collapsing_lowest_dense(2e-2, 50).unwrap();
    sketch.decode_and_merge_with(input).unwrap();
    assert_eq!(sketch.get_count(), 100.0);
}

#[test]
#[should_panic]
fn test_sketch_decode_panic_1() {
    let input = vec![
        14, 100, 244, 7, 173, 131, 165, 240, 63, 0, 0, 0, 0, 0, 0, 0, 0, 5, 21, 0, 140, 48, 34,
        150, 241, 16, 20, 148, 191, 96, 14, 142, 62, 12, 139, 16, 10, 134, 96, 8, 3, 6, 2, 6, 2, 6,
        2, 4, 2, 42, 2, 26, 2, 6, 2, 20, 2, 6, 2, 2, 2, 10, 2, 20, 2, 14, 2, 10, 2,
    ];
    let mut sketch = DDSketch::collapsing_lowest_dense(1e-2, 50).unwrap();
    sketch.decode_and_merge_with(input).unwrap();
}

#[test]
#[should_panic]
fn test_sketch_decode_panic_2() {
    let input = vec![
        2, 42, 120, 57, 5, 47, 167, 240, 63, 0, 0, 0, 0, 0, 0, 0, 0, 13, 50, 130, 1, 2, 136, 32, 0,
        3, 0, 0, 0, 3, 0, 2, 0, 0, 3, 3, 2, 2, 3, 3, 2, 0, 0, 0, 0, 2, 0, 2, 2, 2, 4, 4, 132, 64,
        0, 4, 2, 0, 2, 2, 3, 132, 64, 4, 132, 64, 4, 2, 2, 0, 6, 4, 6, 132, 64, 2, 6,
    ];
    let mut sketch = DDSketch::collapsing_highest_dense(2e-2, 50).unwrap();
    sketch.decode_and_merge_with(input).unwrap();
}

#[test]
fn test_sketch_encode() {
    let mut sketch1 = DDSketch::unbounded_dense(2e-2).unwrap();
    sketch1.accept(1.0);
    sketch1.accept(2.0);
    sketch1.accept(3.0);
    sketch1.accept(4.0);
    sketch1.accept(5.0);
    println!("encode: {:?}", sketch1.encode().unwrap());
    let mut sketch2 = DDSketch::unbounded_dense(2e-2).unwrap();
    sketch2
        .decode_and_merge_with(sketch1.encode().unwrap())
        .unwrap();
    assert_eq!(5.0, sketch2.get_count());
}

#[test]
fn test_sketch_create() {
    let mut sketch1: DDSketch<CubicallyInterpolatedMapping, CollapsingLowestDenseStore> =
        DDSketch::collapsing_lowest_dense(2e-2, 100).unwrap();
    sketch1.accept(1.0);
    let mut sketch2: DDSketch<CubicallyInterpolatedMapping, CollapsingHighestDenseStore> =
        DDSketch::collapsing_highest_dense(2e-2, 100).unwrap();
    sketch2.accept(1.0);
    let mut sketch3: DDSketch<LogarithmicMapping, CollapsingLowestDenseStore> =
        DDSketch::logarithmic_collapsing_lowest_dense(2e-2, 100).unwrap();
    sketch3.accept(1.0);
    let mut sketch4: DDSketch<LogarithmicMapping, CollapsingHighestDenseStore> =
        DDSketch::logarithmic_collapsing_highest_dense(2e-2, 100).unwrap();
    sketch4.accept(1.0);
    let mut sketch5: DDSketch<CubicallyInterpolatedMapping, UnboundedSizeDenseStore> =
        DDSketch::unbounded_dense(2e-2).unwrap();
    sketch5.accept(1.0);
    let mut sketch6: DDSketch<LogarithmicMapping, UnboundedSizeDenseStore> =
        DDSketch::logarithmic_unbounded_size_dense_store(2e-2).unwrap();
    sketch6.accept(1.0);
}
