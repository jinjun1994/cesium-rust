//! Direct port and verification against CesiumJS `CompositePropertySpec.js` (Apache-2.0).
#![allow(clippy::field_reassign_with_default)]

use cesium_math::{
    Cartesian3, CompositeProperty, ConstantProperty, JulianDate, TimeInterval, TimeStandard,
};
use std::cell::Cell;
use std::rc::Rc;

#[test]
fn default_constructor_has_expected_values() {
    let property: CompositeProperty<Cartesian3> = CompositeProperty::new();
    assert_eq!(property.intervals.len(), 0);
    assert_eq!(property.get_value(Some(&JulianDate::now())), None);
    assert!(property.is_constant());
}

#[test]
fn works_with_intervals_and_evaluates_values() {
    let mut interval1 = TimeInterval::default();
    interval1.start = JulianDate::new(2450000.0, 10.0, TimeStandard::TAI);
    interval1.stop = JulianDate::new(2450000.0, 12.0, TimeStandard::TAI);
    interval1.data = Some(ConstantProperty::from_value(Cartesian3::new(1.0, 2.0, 3.0)));

    let mut interval2 = TimeInterval::default();
    interval2.start = JulianDate::new(2450000.0, 12.0, TimeStandard::TAI);
    interval2.stop = JulianDate::new(2450000.0, 14.0, TimeStandard::TAI);
    interval2.is_start_included = false;
    interval2.data = Some(ConstantProperty::from_value(Cartesian3::new(4.0, 5.0, 6.0)));

    let mut property = CompositeProperty::new();
    property.intervals.add_interval(interval1);
    property.intervals.add_interval(interval2);

    assert!(!property.is_constant());

    let result1 = property.get_value(Some(&JulianDate::new(2450000.0, 10.0, TimeStandard::TAI)));
    assert_eq!(result1, Some(Cartesian3::new(1.0, 2.0, 3.0)));

    let result2 = property.get_value(Some(&JulianDate::new(2450000.0, 14.0, TimeStandard::TAI)));
    assert_eq!(result2, Some(Cartesian3::new(4.0, 5.0, 6.0)));
}

#[test]
fn equals_works() {
    let mut interval1 = TimeInterval::default();
    interval1.start = JulianDate::new(2450000.0, 10.0, TimeStandard::TAI);
    interval1.stop = JulianDate::new(2450000.0, 12.0, TimeStandard::TAI);
    interval1.data = Some(ConstantProperty::from_value(Cartesian3::new(1.0, 2.0, 3.0)));

    let mut interval2 = TimeInterval::default();
    interval2.start = JulianDate::new(2450000.0, 12.0, TimeStandard::TAI);
    interval2.stop = JulianDate::new(2450000.0, 14.0, TimeStandard::TAI);
    interval2.is_start_included = false;
    interval2.data = Some(ConstantProperty::from_value(Cartesian3::new(4.0, 5.0, 6.0)));

    let mut left = CompositeProperty::new();
    left.intervals.add_interval(interval1.clone());
    left.intervals.add_interval(interval2.clone());

    let mut right = CompositeProperty::new();
    right.intervals.add_interval(interval1);
    assert!(!left.equals(Some(&right)));

    right.intervals.add_interval(interval2);
    assert!(left.equals(Some(&right)));
    assert!(!left.equals(None));
}

#[test]
fn definition_changed_event() {
    let property: CompositeProperty<Cartesian3> = CompositeProperty::new();
    let called = Rc::new(Cell::new(0));
    let called_clone = called.clone();

    property.definition_changed().add_listener(move |_| {
        called_clone.set(called_clone.get() + 1);
    });

    property.raise_definition_changed();
    assert_eq!(called.get(), 1);
}
