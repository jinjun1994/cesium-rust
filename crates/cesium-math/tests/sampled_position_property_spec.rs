use std::cell::Cell;
use std::rc::Rc;

use cesium_math::{
    Cartesian3, ExtrapolationType, InterpolationAlgorithm, JulianDate, ReferenceFrame,
    SampledPositionProperty, TimeInterval, TimeStandard,
};

#[test]
fn constructor_sets_expected_defaults() {
    let property = SampledPositionProperty::default();
    assert_eq!(property.interpolation_degree(), 1);
    assert_eq!(
        property.interpolation_algorithm(),
        InterpolationAlgorithm::Linear
    );
    assert_eq!(property.reference_frame(), ReferenceFrame::Fixed);
    assert!(property.is_constant());
    assert_eq!(
        property.forward_extrapolation_type(),
        ExtrapolationType::None
    );
    assert_eq!(
        property.backward_extrapolation_type(),
        ExtrapolationType::None
    );
}

#[test]
fn can_specify_reference_frame() {
    let property = SampledPositionProperty::new(Some(ReferenceFrame::Inertial));
    assert_eq!(property.reference_frame(), ReferenceFrame::Inertial);
}

#[test]
fn is_constant_works() {
    let mut property = SampledPositionProperty::default();
    assert!(property.is_constant());
    property.add_sample(
        JulianDate::new(0.0, 0.0, TimeStandard::TAI),
        Cartesian3::UNIT_X,
    );
    assert!(!property.is_constant());
}

#[test]
fn add_samples_packed_array_works() {
    let data = vec![0.0, 7.0, 0.0, 0.0, 1.0, 8.0, 0.0, 0.0, 2.0, 9.0, 0.0, 0.0];
    let epoch = JulianDate::new(0.0, 0.0, TimeStandard::TAI);

    let mut property = SampledPositionProperty::default();
    let called = Rc::new(Cell::new(false));
    let called_clone = Rc::clone(&called);
    property.definition_changed().add_listener(move |_| {
        called_clone.set(true);
    });

    property.add_samples_packed_array(&data, &epoch);
    assert!(called.get());

    let val0 = property.get_value(Some(&epoch)).unwrap();
    assert_eq!(val0, Cartesian3::new(7.0, 0.0, 0.0));

    let half_sec = JulianDate::new(0.0, 0.5, TimeStandard::TAI);
    let val_half = property.get_value(Some(&half_sec)).unwrap();
    assert_eq!(val_half, Cartesian3::new(7.5, 0.0, 0.0));
}

#[test]
fn add_sample_works() {
    let times = [
        JulianDate::new(0.0, 0.0, TimeStandard::TAI),
        JulianDate::new(1.0, 0.0, TimeStandard::TAI),
        JulianDate::new(2.0, 0.0, TimeStandard::TAI),
    ];
    let positions = [
        Cartesian3::new(7.0, 0.0, 0.0),
        Cartesian3::new(8.0, 0.0, 0.0),
        Cartesian3::new(9.0, 0.0, 0.0),
    ];

    let mut property = SampledPositionProperty::default();
    let called = Rc::new(Cell::new(false));
    let called_clone = Rc::clone(&called);
    property.definition_changed().add_listener(move |_| {
        called_clone.set(true);
    });

    property.add_sample(times[0], positions[0]);
    assert!(called.get());

    property.add_sample(times[1], positions[1]);
    property.add_sample(times[2], positions[2]);

    assert_eq!(property.get_value(Some(&times[0])), Some(positions[0]));
    assert_eq!(property.get_value(Some(&times[1])), Some(positions[1]));
    assert_eq!(property.get_value(Some(&times[2])), Some(positions[2]));

    let mid = JulianDate::new(0.5, 0.0, TimeStandard::TAI);
    assert_eq!(
        property.get_value(Some(&mid)),
        Some(Cartesian3::new(7.5, 0.0, 0.0))
    );
}

#[test]
fn works_with_reference_frame_parameter() {
    let time = JulianDate::new(0.0, 0.0, TimeStandard::TAI);
    let pos_fixed = Cartesian3::new(1000.0, 2000.0, 3000.0);

    let mut property = SampledPositionProperty::new(Some(ReferenceFrame::Fixed));
    property.add_sample(time, pos_fixed);

    let val_fixed = property
        .get_value_in_reference_frame(Some(&time), ReferenceFrame::Fixed)
        .unwrap();
    assert_eq!(val_fixed, pos_fixed);

    // 查询在 Inertial 坐标系下的值
    let val_inertial = property
        .get_value_in_reference_frame(Some(&time), ReferenceFrame::Inertial)
        .unwrap();
    assert!(val_inertial.magnitude() > 0.0);
}

#[test]
fn can_remove_a_sample_at_a_date() {
    let times = vec![
        JulianDate::new(0.0, 0.0, TimeStandard::TAI),
        JulianDate::new(1.0, 0.0, TimeStandard::TAI),
        JulianDate::new(2.0, 0.0, TimeStandard::TAI),
    ];
    let positions = vec![
        Cartesian3::new(1.0, 0.0, 0.0),
        Cartesian3::new(8.0, 0.0, 0.0),
        Cartesian3::new(3.0, 0.0, 0.0),
    ];

    let mut property = SampledPositionProperty::default();
    property.add_samples(&times, &positions);

    assert!(property.remove_sample(&times[1]));
    assert_eq!(property.times().len(), 2);

    let val_mid = property.get_value(Some(&times[1])).unwrap();
    assert_eq!(val_mid, Cartesian3::new(2.0, 0.0, 0.0));
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
    let positions = vec![
        Cartesian3::new(1.0, 0.0, 0.0),
        Cartesian3::new(8.0, 0.0, 0.0),
        Cartesian3::new(13.0, 0.0, 0.0),
        Cartesian3::new(1.0, 0.0, 0.0),
        Cartesian3::new(3.0, 0.0, 0.0),
    ];

    let mut property = SampledPositionProperty::default();
    property.add_samples(&times, &positions);

    let interval = TimeInterval::new(times[1], times[3], true, true);
    assert!(property.remove_samples(&interval));

    assert_eq!(property.times().len(), 2);
    let val_mid = property.get_value(Some(&times[2])).unwrap();
    assert_eq!(val_mid, Cartesian3::new(2.0, 0.0, 0.0));
}

#[test]
fn equals_works() {
    let time = JulianDate::new(0.0, 0.0, TimeStandard::TAI);
    let mut left = SampledPositionProperty::default();
    left.add_sample(time, Cartesian3::UNIT_X);

    let mut right = SampledPositionProperty::default();
    right.add_sample(time, Cartesian3::UNIT_X);

    assert!(left.equals(Some(&right)));

    right.add_sample(
        JulianDate::new(1.0, 0.0, TimeStandard::TAI),
        Cartesian3::UNIT_Y,
    );
    assert!(!left.equals(Some(&right)));
}
