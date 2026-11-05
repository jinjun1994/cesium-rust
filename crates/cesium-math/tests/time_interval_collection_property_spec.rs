//! Direct port and verification against CesiumJS `TimeIntervalCollectionPropertySpec.js` (Apache-2.0).
#![allow(clippy::field_reassign_with_default)]

use cesium_math::{
    Cartesian3, JulianDate, TimeInterval, TimeIntervalCollectionProperty, TimeStandard,
};
use std::cell::Cell;
use std::rc::Rc;

#[test]
fn default_constructor_has_expected_values() {
    let property: TimeIntervalCollectionProperty<i32> = TimeIntervalCollectionProperty::new();
    assert_eq!(property.intervals.len(), 0);
    assert_eq!(property.get_value(Some(&JulianDate::now())), None);
    assert!(property.is_constant());
}

#[test]
fn works_with_basic_types() {
    let mut interval1 = TimeInterval::default();
    interval1.start = JulianDate::new(2450000.0, 10.0, TimeStandard::TAI);
    interval1.stop = JulianDate::new(2450000.0, 12.0, TimeStandard::TAI);
    interval1.data = Some(5);

    let mut interval2 = TimeInterval::default();
    interval2.start = JulianDate::new(2450000.0, 12.0, TimeStandard::TAI);
    interval2.stop = JulianDate::new(2450000.0, 14.0, TimeStandard::TAI);
    interval2.is_start_included = false;
    interval2.data = Some(6);

    let mut property = TimeIntervalCollectionProperty::new();
    property.intervals.add_interval(interval1);
    property.intervals.add_interval(interval2);

    assert_eq!(property.get_value(Some(&JulianDate::new(2450000.0, 10.0, TimeStandard::TAI))), Some(5));
    assert_eq!(property.get_value(Some(&JulianDate::new(2450000.0, 14.0, TimeStandard::TAI))), Some(6));
    assert!(!property.is_constant());
}

#[test]
fn works_with_clonable_objects() {
    let mut interval1 = TimeInterval::default();
    interval1.start = JulianDate::new(2450000.0, 10.0, TimeStandard::TAI);
    interval1.stop = JulianDate::new(2450000.0, 12.0, TimeStandard::TAI);
    interval1.data = Some(Cartesian3::new(1.0, 2.0, 3.0));

    let mut interval2 = TimeInterval::default();
    interval2.start = JulianDate::new(2450000.0, 12.0, TimeStandard::TAI);
    interval2.stop = JulianDate::new(2450000.0, 14.0, TimeStandard::TAI);
    interval2.is_start_included = false;
    interval2.data = Some(Cartesian3::new(4.0, 5.0, 6.0));

    let mut property = TimeIntervalCollectionProperty::new();
    property.intervals.add_interval(interval1);
    property.intervals.add_interval(interval2);

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
    interval1.data = Some(5);

    let mut left = TimeIntervalCollectionProperty::new();
    left.intervals.add_interval(interval1);

    let mut right = TimeIntervalCollectionProperty::new();
    right.intervals.add_interval(interval1);

    assert!(left.equals(Some(&right)));

    let mut interval2 = TimeInterval::default();
    interval2.start = JulianDate::new(2450000.0, 12.0, TimeStandard::TAI);
    interval2.stop = JulianDate::new(2450000.0, 14.0, TimeStandard::TAI);
    interval2.data = Some(6);
    right.intervals.add_interval(interval2);

    assert!(!left.equals(Some(&right)));
    assert!(!left.equals(None));
}

#[test]
fn raises_definition_changed_event() {
    let property: TimeIntervalCollectionProperty<i32> = TimeIntervalCollectionProperty::new();
    let called = Rc::new(Cell::new(0));
    let called_clone = called.clone();

    property.definition_changed().add_listener(move |_| {
        called_clone.set(called_clone.get() + 1);
    });

    property.raise_definition_changed();
    assert_eq!(called.get(), 1);
}
