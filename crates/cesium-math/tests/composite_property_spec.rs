//! Direct port and verification against CesiumJS `DataSources/CompositePropertySpec.js` (Apache-2.0).

use cesium_math::{
    Cartesian3, CompositeProperty, ConstantProperty, JulianDate, TimeInterval, TimeStandard,
};
use std::cell::Cell;
use std::rc::Rc;

#[test]
fn default_constructor_has_expected_values() {
    let property: CompositeProperty<Cartesian3> = CompositeProperty::new();
    assert_eq!(property.intervals.len(), 0);
    assert!(property.intervals.is_empty());
    assert_eq!(property.get_value(Some(&JulianDate::now())), None);
    assert!(property.is_constant());
}

#[test]
fn works_without_a_result_parameter() {
    let interval1 = TimeInterval {
        start: JulianDate::new(10.0, 0.0, TimeStandard::TAI),
        stop: JulianDate::new(12.0, 0.0, TimeStandard::TAI),
        data: Some(ConstantProperty::from_value(Cartesian3::new(1.0, 2.0, 3.0)).into()),
        ..Default::default()
    };
    let interval2 = TimeInterval {
        start: JulianDate::new(12.0, 0.0, TimeStandard::TAI),
        stop: JulianDate::new(14.0, 0.0, TimeStandard::TAI),
        is_start_included: false,
        data: Some(ConstantProperty::from_value(Cartesian3::new(4.0, 5.0, 6.0)).into()),
        ..Default::default()
    };

    let mut property = CompositeProperty::new();
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
fn equals_works() {
    let interval1 = TimeInterval {
        start: JulianDate::new(10.0, 0.0, TimeStandard::TAI),
        stop: JulianDate::new(12.0, 0.0, TimeStandard::TAI),
        data: Some(ConstantProperty::from_value(Cartesian3::new(1.0, 2.0, 3.0)).into()),
        ..Default::default()
    };
    let interval2 = TimeInterval {
        start: JulianDate::new(12.0, 0.0, TimeStandard::TAI),
        stop: JulianDate::new(14.0, 0.0, TimeStandard::TAI),
        is_start_included: false,
        data: Some(ConstantProperty::from_value(Cartesian3::new(4.0, 5.0, 6.0)).into()),
        ..Default::default()
    };

    let mut left = CompositeProperty::new();
    left.intervals.add_interval(interval1.clone());
    left.intervals.add_interval(interval2.clone());

    let mut right = CompositeProperty::new();
    right.intervals.add_interval(interval1);
    assert!(!left.equals(Some(&right)));

    right.intervals.add_interval(interval2);
    assert!(left.equals(Some(&right)));
}

#[test]
fn raises_definition_changed_event_in_all_cases() {
    let inner_prop1 = Rc::new(ConstantProperty::from_value(Cartesian3::new(
        1.0, 2.0, 3.0,
    )));
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
        data: Some(ConstantProperty::from_value(Cartesian3::new(4.0, 5.0, 6.0)).into()),
        ..Default::default()
    };

    let mut property = CompositeProperty::new();
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
    inner_prop1.set_value(Some(Cartesian3::ZERO));
    assert_eq!(count.get(), 4);

    property.intervals.remove_all();
    assert_eq!(count.get(), 5);
}

#[test]
fn does_not_raise_definition_changed_for_an_overwritten_interval() {
    let inner_prop1 = Rc::new(ConstantProperty::from_value(Cartesian3::new(
        1.0, 2.0, 3.0,
    )));
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
        data: Some(ConstantProperty::from_value(Cartesian3::new(4.0, 5.0, 6.0)).into()),
        ..Default::default()
    };

    let mut property = CompositeProperty::new();
    let count = Rc::new(Cell::new(0));
    let c = count.clone();
    let _sub = property.definition_changed().add_listener(move |_| {
        c.set(c.get() + 1);
    });

    property.intervals.add_interval(interval1);
    property.intervals.add_interval(interval2);
    assert_eq!(count.get(), 2);

    // interval2 overwrote interval1, so modifying inner_prop1 does not raise definitionChanged
    inner_prop1.set_value(Some(Cartesian3::ZERO));
    assert_eq!(count.get(), 2);
}

#[test]
fn get_value_uses_now_if_time_is_none() {
    let property: CompositeProperty<Cartesian3> = CompositeProperty::new();
    assert_eq!(property.get_value(None), None);
}
