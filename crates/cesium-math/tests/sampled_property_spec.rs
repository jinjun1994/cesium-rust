use std::cell::Cell;
use std::rc::Rc;

use cesium_math::{
    ExtrapolationType, InterpolationAlgorithm, JulianDate, SampledProperty, TimeInterval,
    TimeStandard,
};

#[test]
fn constructor_sets_expected_defaults() {
    let property = SampledProperty::<f64>::new();
    assert_eq!(property.interpolation_degree(), 1);
    assert_eq!(
        property.interpolation_algorithm(),
        InterpolationAlgorithm::Linear
    );
    assert!(property.is_constant());
    assert_eq!(
        property.forward_extrapolation_type(),
        ExtrapolationType::None
    );
    assert_eq!(property.forward_extrapolation_duration(), 0.0);
    assert_eq!(
        property.backward_extrapolation_type(),
        ExtrapolationType::None
    );
    assert_eq!(property.backward_extrapolation_duration(), 0.0);
}

#[test]
fn is_constant_works() {
    let mut property = SampledProperty::<f64>::new();
    assert!(property.is_constant());
    property.add_sample(JulianDate::new(0.0, 0.0, TimeStandard::TAI), 1.0);
    assert!(!property.is_constant());
}

#[test]
fn add_samples_packed_array_works() {
    let data = vec![0.0, 7.0, 1.0, 8.0, 2.0, 9.0];
    let epoch = JulianDate::new(0.0, 0.0, TimeStandard::TAI);

    let mut property = SampledProperty::<f64>::new();
    let called = Rc::new(Cell::new(false));
    let called_clone = Rc::clone(&called);
    property.definition_changed().add_listener(move |_| {
        called_clone.set(true);
    });

    property.add_samples_packed_array(&data, &epoch);
    assert!(called.get());

    assert_eq!(property.get_value(Some(&epoch)), Some(7.0));
    let half_sec = JulianDate::new(0.0, 0.5, TimeStandard::TAI);
    assert_eq!(property.get_value(Some(&half_sec)), Some(7.5));
}

#[test]
fn add_sample_works() {
    let times = [
        JulianDate::new(0.0, 0.0, TimeStandard::TAI),
        JulianDate::new(1.0, 0.0, TimeStandard::TAI),
        JulianDate::new(2.0, 0.0, TimeStandard::TAI),
    ];
    let values = [7.0, 8.0, 9.0];

    let mut property = SampledProperty::<f64>::new();
    let call_count = Rc::new(Cell::new(0));
    let cc_clone = Rc::clone(&call_count);
    property.definition_changed().add_listener(move |_| {
        cc_clone.set(cc_clone.get() + 1);
    });

    property.add_sample(times[0], values[0]);
    assert_eq!(call_count.get(), 1);

    property.add_sample(times[1], values[1]);
    assert_eq!(call_count.get(), 2);

    property.add_sample(times[2], values[2]);
    assert_eq!(call_count.get(), 3);

    assert_eq!(property.get_value(Some(&times[0])), Some(values[0]));
    assert_eq!(property.get_value(Some(&times[1])), Some(values[1]));
    assert_eq!(property.get_value(Some(&times[2])), Some(values[2]));

    let mid = JulianDate::new(0.5, 0.0, TimeStandard::TAI);
    assert_eq!(property.get_value(Some(&mid)), Some(7.5));
}

#[test]
fn add_samples_works() {
    let times = [
        JulianDate::new(0.0, 0.0, TimeStandard::TAI),
        JulianDate::new(1.0, 0.0, TimeStandard::TAI),
        JulianDate::new(2.0, 0.0, TimeStandard::TAI),
    ];
    let values = [7.0, 8.0, 9.0];

    let mut property = SampledProperty::<f64>::new();
    let called = Rc::new(Cell::new(false));
    let called_clone = Rc::clone(&called);
    property.definition_changed().add_listener(move |_| {
        called_clone.set(true);
    });

    property.add_samples(&times, &values);
    assert!(called.get());

    assert_eq!(property.get_value(Some(&times[0])), Some(values[0]));
    assert_eq!(property.get_value(Some(&times[1])), Some(values[1]));
    assert_eq!(property.get_value(Some(&times[2])), Some(values[2]));
    let mid = JulianDate::new(0.5, 0.0, TimeStandard::TAI);
    assert_eq!(property.get_value(Some(&mid)), Some(7.5));
}

#[test]
fn get_sample_works() {
    let times = [
        JulianDate::new(0.0, 0.0, TimeStandard::TAI),
        JulianDate::new(1.0, 0.0, TimeStandard::TAI),
        JulianDate::new(2.0, 0.0, TimeStandard::TAI),
    ];
    let values = [7.0, 8.0, 9.0];

    let mut property = SampledProperty::<f64>::new();
    property.add_samples(&times, &values);

    assert_eq!(property.get_sample(0), Some(times[0]));
    assert_eq!(property.get_sample(1), Some(times[1]));
    assert_eq!(property.get_sample(2), Some(times[2]));
    assert_eq!(property.get_sample(3), None);

    assert_eq!(property.get_sample(-1), Some(times[2]));
    assert_eq!(property.get_sample(-2), Some(times[1]));
    assert_eq!(property.get_sample(-3), Some(times[0]));
    assert_eq!(property.get_sample(-4), None);
}

#[test]
fn can_remove_a_sample_at_a_date() {
    let times = vec![
        JulianDate::new(0.0, 0.0, TimeStandard::TAI),
        JulianDate::new(1.0, 0.0, TimeStandard::TAI),
        JulianDate::new(2.0, 0.0, TimeStandard::TAI),
    ];
    let values = vec![1.0, 8.0, 3.0];

    let mut property = SampledProperty::<f64>::new();
    property.add_samples(&times, &values);

    let called = Rc::new(Cell::new(false));
    let called_clone = Rc::clone(&called);
    property.definition_changed().add_listener(move |_| {
        called_clone.set(true);
    });

    let non_existent = JulianDate::new(4.0, 0.0, TimeStandard::TAI);
    assert!(!property.remove_sample(&non_existent));

    assert!(property.remove_sample(&times[1]));
    assert!(called.get());
    assert_eq!(property.times().len(), 2);
    assert_eq!(property.values().len(), 2);

    assert_eq!(property.get_value(Some(&times[0])), Some(values[0]));
    // 移除中间样本后，在 times[1] 处应线性插值为 (1 + 3) / 2 = 2.0
    assert_eq!(property.get_value(Some(&times[1])), Some(2.0));
    assert_eq!(property.get_value(Some(&times[2])), Some(values[2]));
}

#[test]
fn can_remove_samples_for_a_time_interval() {
    let times = vec![
        JulianDate::new(0.0, 0.0, TimeStandard::TAI),
        JulianDate::new(1.0, 0.0, TimeStandard::TAI),
        JulianDate::new(2.0, 0.0, TimeStandard::TAI),
        JulianDate::new(3.0, 0.0, TimeStandard::TAI),
        JulianDate::new(4.0, 0.0, TimeStandard::TAI),
    ];
    let values = vec![1.0, 8.0, 13.0, 1.0, 3.0];

    let mut property = SampledProperty::<f64>::new();
    property.add_samples(&times, &values);

    let interval = TimeInterval::new(times[1], times[3], true, true);
    assert!(property.remove_samples(&interval));

    assert_eq!(property.times().len(), 2);
    assert_eq!(property.values().len(), 2);
    assert_eq!(property.times(), &[times[0], times[4]]);

    // 插值 times[2] 应为 (1 + 3) / 2 = 2.0
    assert_eq!(property.get_value(Some(&times[2])), Some(2.0));
}

#[test]
fn extrapolation_hold_and_none_works() {
    let times = vec![
        JulianDate::new(0.0, 0.0, TimeStandard::TAI),
        JulianDate::new(1.0, 0.0, TimeStandard::TAI),
    ];
    let values = vec![5.0, 10.0];

    let mut property = SampledProperty::<f64>::new();
    property.add_samples(&times, &values);

    let before = JulianDate::new(-1.0, 0.0, TimeStandard::TAI);
    let after = JulianDate::new(2.0, 0.0, TimeStandard::TAI);

    // 默认 None
    assert_eq!(property.get_value(Some(&before)), None);
    assert_eq!(property.get_value(Some(&after)), None);

    // 设置 Hold
    property.set_backward_extrapolation_type(ExtrapolationType::Hold);
    property.set_forward_extrapolation_type(ExtrapolationType::Hold);

    assert_eq!(property.get_value(Some(&before)), Some(5.0));
    assert_eq!(property.get_value(Some(&after)), Some(10.0));
}

#[test]
fn equals_works() {
    let times = vec![JulianDate::new(0.0, 0.0, TimeStandard::TAI)];
    let values = vec![1.0];

    let mut left = SampledProperty::<f64>::new();
    left.add_samples(&times, &values);

    let mut right = SampledProperty::<f64>::new();
    right.add_samples(&times, &values);

    assert!(left.equals(Some(&right)));

    right.add_sample(JulianDate::new(1.0, 0.0, TimeStandard::TAI), 2.0);
    assert!(!left.equals(Some(&right)));
}
