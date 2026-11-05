//! Direct port and verification against CesiumJS `ConstantPropertySpec.js` (Apache-2.0).

use cesium_math::{Cartesian3, ConstantProperty, JulianDate};
use std::cell::Cell;
use std::rc::Rc;

#[test]
fn works_with_basic_types() {
    let time = JulianDate::now();
    let expected = 5;
    let property = ConstantProperty::from_value(expected);
    assert_eq!(property.get_value(Some(&time)), Some(expected));
    assert_eq!(format!("{property}"), expected.to_string());
}

#[test]
fn works_with_objects() {
    let time = JulianDate::now();
    let value = Cartesian3::new(1.0, 2.0, 3.0);
    let property = ConstantProperty::from_value(value);

    let result = property.get_value(Some(&time));
    assert_eq!(result, Some(value));
}

#[test]
fn set_value_raises_definition_changed_event() {
    let mut property = ConstantProperty::new(None);
    let called = Rc::new(Cell::new(0));
    let called_clone = called.clone();

    property.definition_changed().add_listener(move |_| {
        called_clone.set(called_clone.get() + 1);
    });

    property.set_value(Some(5));
    assert_eq!(called.get(), 1);
}

#[test]
fn set_value_does_not_raise_definition_changed_event_with_equal_data() {
    let mut property = ConstantProperty::from_value(Cartesian3::new(0.0, 0.0, 0.0));
    let called = Rc::new(Cell::new(0));
    let called_clone = called.clone();

    property.definition_changed().add_listener(move |_| {
        called_clone.set(called_clone.get() + 1);
    });

    property.set_value(Some(Cartesian3::new(0.0, 0.0, 0.0)));
    assert_eq!(called.get(), 0);
}

#[test]
fn works_with_undefined_value() {
    let property: ConstantProperty<i32> = ConstantProperty::new(None);
    assert_eq!(property.get_value(None), None);
    assert_eq!(format!("{property}"), "undefined");
}

#[test]
fn equals_works_for_object_types() {
    let left = ConstantProperty::from_value(Cartesian3::new(1.0, 2.0, 3.0));
    let right_equal = ConstantProperty::from_value(Cartesian3::new(1.0, 2.0, 3.0));
    let right_diff = ConstantProperty::from_value(Cartesian3::new(1.0, 2.0, 4.0));

    assert!(left.equals(Some(&right_equal)));
    assert!(!left.equals(Some(&right_diff)));
    assert!(!left.equals(None));
}

#[test]
fn equals_works_for_simple_types() {
    let left = ConstantProperty::from_value(1);
    let right_equal = ConstantProperty::from_value(1);
    let right_diff = ConstantProperty::from_value(2);

    assert!(left.equals(Some(&right_equal)));
    assert!(!left.equals(Some(&right_diff)));
}
