//! Direct port and verification against CesiumJS `DataSources/CompositePositionPropertySpec.js` (Apache-2.0).

use cesium_math::{
    Cartesian3, CompositePositionProperty, ConstantPositionProperty, JulianDate, PositionProperty,
    ReferenceFrame, TimeInterval, TimeStandard,
};
use std::cell::Cell;
use std::rc::Rc;

#[test]
fn default_constructor_has_expected_values() {
    let property = CompositePositionProperty::new(None);
    assert_eq!(property.intervals.len(), 0);
    assert!(property.intervals.is_empty());
    assert_eq!(property.get_value(Some(&JulianDate::now())), None);
    assert_eq!(property.reference_frame, ReferenceFrame::Fixed);
    assert!(property.is_constant());
}

#[test]
fn constructor_sets_expected_values() {
    let property = CompositePositionProperty::new(Some(ReferenceFrame::Inertial));
    assert_eq!(property.reference_frame, ReferenceFrame::Inertial);
}

#[test]
fn can_modify_reference_frame() {
    let mut property = CompositePositionProperty::new(None);
    assert_eq!(property.reference_frame, ReferenceFrame::Fixed);
    property.reference_frame = ReferenceFrame::Inertial;
    assert_eq!(property.reference_frame, ReferenceFrame::Inertial);
}

#[test]
fn works_without_a_result_parameter() {
    let interval1 = TimeInterval {
        start: JulianDate::new(10.0, 0.0, TimeStandard::TAI),
        stop: JulianDate::new(12.0, 0.0, TimeStandard::TAI),
        data: Some(
            ConstantPositionProperty::new(Some(Cartesian3::new(1.0, 2.0, 3.0)), None).into(),
        ),
        ..Default::default()
    };
    let interval2 = TimeInterval {
        start: JulianDate::new(12.0, 0.0, TimeStandard::TAI),
        stop: JulianDate::new(14.0, 0.0, TimeStandard::TAI),
        is_start_included: false,
        data: Some(
            ConstantPositionProperty::new(Some(Cartesian3::new(4.0, 5.0, 6.0)), None).into(),
        ),
        ..Default::default()
    };

    let mut property = CompositePositionProperty::new(None);
    property.intervals.add_interval(interval1);
    property.intervals.add_interval(interval2);
    assert!(!property.is_constant());

    let t1 = JulianDate::new(10.0, 0.0, TimeStandard::TAI);
    let result1 = property.get_value(Some(&t1));
    assert_eq!(result1, Some(Cartesian3::new(1.0, 2.0, 3.0)));

    let t2 = JulianDate::new(14.0, 0.0, TimeStandard::TAI);
    let result2 = property.get_value(Some(&t2));
    assert_eq!(result2, Some(Cartesian3::new(4.0, 5.0, 6.0)));
}

#[test]
fn get_value_returns_in_fixed_frame() {
    let interval1 = TimeInterval {
        start: JulianDate::new(10.0, 0.0, TimeStandard::TAI),
        stop: JulianDate::new(12.0, 0.0, TimeStandard::TAI),
        data: Some(
            ConstantPositionProperty::new(
                Some(Cartesian3::new(1.0, 2.0, 3.0)),
                Some(ReferenceFrame::Inertial),
            )
            .into(),
        ),
        ..Default::default()
    };
    let interval2 = TimeInterval {
        start: JulianDate::new(12.0, 0.0, TimeStandard::TAI),
        stop: JulianDate::new(14.0, 0.0, TimeStandard::TAI),
        is_start_included: false,
        data: Some(
            ConstantPositionProperty::new(
                Some(Cartesian3::new(4.0, 5.0, 6.0)),
                Some(ReferenceFrame::Fixed),
            )
            .into(),
        ),
        ..Default::default()
    };

    let mut property = CompositePositionProperty::new(None);
    property.intervals.add_interval(interval1);
    property.intervals.add_interval(interval2);

    let t1 = JulianDate::new(10.0, 0.0, TimeStandard::TAI);
    let result1 = property.get_value_in_reference_frame(Some(&t1), ReferenceFrame::Inertial);
    assert_eq!(result1, Some(Cartesian3::new(1.0, 2.0, 3.0)));

    let t2 = JulianDate::new(14.0, 0.0, TimeStandard::TAI);
    let result2 = property.get_value_in_reference_frame(Some(&t2), ReferenceFrame::Fixed);
    assert_eq!(result2, Some(Cartesian3::new(4.0, 5.0, 6.0)));

    // In fixed frame, interval1 gets transformed
    let expected_fixed = PositionProperty::convert_to_reference_frame(
        Some(&t1),
        Some(Cartesian3::new(1.0, 2.0, 3.0)),
        ReferenceFrame::Inertial,
        ReferenceFrame::Fixed,
    );
    assert_eq!(property.get_value(Some(&t1)), expected_fixed);
}

#[test]
fn equals_works() {
    let interval1 = TimeInterval {
        start: JulianDate::new(10.0, 0.0, TimeStandard::TAI),
        stop: JulianDate::new(12.0, 0.0, TimeStandard::TAI),
        data: Some(
            ConstantPositionProperty::new(Some(Cartesian3::new(1.0, 2.0, 3.0)), None).into(),
        ),
        ..Default::default()
    };
    let interval2 = TimeInterval {
        start: JulianDate::new(12.0, 0.0, TimeStandard::TAI),
        stop: JulianDate::new(14.0, 0.0, TimeStandard::TAI),
        is_start_included: false,
        data: Some(
            ConstantPositionProperty::new(Some(Cartesian3::new(4.0, 5.0, 6.0)), None).into(),
        ),
        ..Default::default()
    };

    let mut left = CompositePositionProperty::new(None);
    left.intervals.add_interval(interval1.clone());
    left.intervals.add_interval(interval2.clone());

    let mut right = CompositePositionProperty::new(None);
    right.intervals.add_interval(interval1);
    assert!(!left.equals(Some(&right)));

    right.intervals.add_interval(interval2);
    assert!(left.equals(Some(&right)));

    right.reference_frame = ReferenceFrame::Inertial;
    assert!(!left.equals(Some(&right)));
}

#[test]
fn raises_definition_changed_event_in_all_cases() {
    let inner_prop1 = Rc::new(ConstantPositionProperty::new(
        Some(Cartesian3::new(1.0, 2.0, 3.0)),
        None,
    ));
    let interval1 = TimeInterval {
        start: JulianDate::new(10.0, 0.0, TimeStandard::TAI),
        stop: JulianDate::new(12.0, 0.0, TimeStandard::TAI),
        data: Some(inner_prop1.clone().into()),
        ..Default::default()
    };
    let interval2 = TimeInterval {
        start: JulianDate::new(12.0, 0.0, TimeStandard::TAI),
        stop: JulianDate::new(14.0, 0.0, TimeStandard::TAI),
        is_start_included: false,
        data: Some(
            ConstantPositionProperty::new(Some(Cartesian3::new(4.0, 5.0, 6.0)), None).into(),
        ),
        ..Default::default()
    };

    let mut property = CompositePositionProperty::new(None);
    let count = Rc::new(Cell::new(0));
    let c = count.clone();
    let _sub = property.definition_changed().add_listener(move |_| {
        c.set(c.get() + 1);
    });

    property.intervals.add_interval(interval1);
    assert_eq!(count.get(), 1);

    property.intervals.add_interval(interval2.clone());
    assert_eq!(count.get(), 2);

    property.intervals.remove_interval(&interval2);
    assert_eq!(count.get(), 3);

    // Modifying child property's value raises parent definitionChanged
    inner_prop1.set_value(Some(Cartesian3::ZERO), None);
    assert_eq!(count.get(), 4);

    property.intervals.remove_all();
    assert_eq!(count.get(), 5);
}

#[test]
fn does_not_raise_definition_changed_for_an_overwritten_interval() {
    let inner_prop1 = Rc::new(ConstantPositionProperty::new(
        Some(Cartesian3::new(1.0, 2.0, 3.0)),
        None,
    ));
    let interval1 = TimeInterval {
        start: JulianDate::new(11.0, 0.0, TimeStandard::TAI),
        stop: JulianDate::new(13.0, 0.0, TimeStandard::TAI),
        data: Some(inner_prop1.clone().into()),
        ..Default::default()
    };
    let interval2 = TimeInterval {
        start: JulianDate::new(10.0, 0.0, TimeStandard::TAI),
        stop: JulianDate::new(14.0, 0.0, TimeStandard::TAI),
        is_start_included: false,
        data: Some(
            ConstantPositionProperty::new(Some(Cartesian3::new(4.0, 5.0, 6.0)), None).into(),
        ),
        ..Default::default()
    };

    let mut property = CompositePositionProperty::new(None);
    let count = Rc::new(Cell::new(0));
    let c = count.clone();
    let _sub = property.definition_changed().add_listener(move |_| {
        c.set(c.get() + 1);
    });

    property.intervals.add_interval(interval1);
    property.intervals.add_interval(interval2);
    assert_eq!(count.get(), 2);

    // interval2 overwrote interval1, so modifying inner_prop1 does not raise definitionChanged
    inner_prop1.set_value(Some(Cartesian3::ZERO), None);
    assert_eq!(count.get(), 2);
}
