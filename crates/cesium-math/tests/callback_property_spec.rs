//! Direct port and verification against CesiumJS `CallbackPropertySpec.js` (Apache-2.0).

use cesium_math::{CallbackProperty, JulianDate};
use std::cell::Cell;
use std::rc::Rc;

#[test]
fn callback_received_proper_parameters_and_returns_result() {
    let time = JulianDate::now();
    let called = Rc::new(Cell::new(0));
    let called_clone = called.clone();

    let callback = Rc::new(move |_time: &JulianDate| {
        called_clone.set(called_clone.get() + 1);
        Some(42)
    });

    let property = CallbackProperty::new(callback, true);
    let result = property.get_value(Some(&time));
    assert_eq!(result, Some(42));
    assert_eq!(called.get(), 1);
}

#[test]
fn get_value_uses_now_if_time_parameter_is_none() {
    let callback = Rc::new(|_time: &JulianDate| Some("time independent result"));
    let property = CallbackProperty::new(callback, true);
    let result = property.get_value(None);
    assert_eq!(result, Some("time independent result"));
}

#[test]
fn is_constant_returns_correct_value_and_set_callback_updates() {
    let cb1 = Rc::new(|_: &JulianDate| Some(1));
    let mut property = CallbackProperty::new(cb1.clone(), true);
    assert!(property.is_constant());

    let cb2 = Rc::new(|_: &JulianDate| Some(2));
    property.set_callback(cb2, false);
    assert!(!property.is_constant());
}

#[test]
fn set_callback_raises_definition_changed_event() {
    let cb = Rc::new(|_: &JulianDate| Some(1));
    let mut property = CallbackProperty::new(cb.clone(), true);
    let called = Rc::new(Cell::new(0));
    let called_clone = called.clone();

    property.definition_changed().add_listener(move |_| {
        called_clone.set(called_clone.get() + 1);
    });

    let new_cb = Rc::new(|_: &JulianDate| Some(2));
    property.set_callback(new_cb, false);
    assert_eq!(called.get(), 1);
}

#[test]
fn equals_works() {
    let callback = Rc::new(|_: &JulianDate| Some(1));
    let left = CallbackProperty::new(callback.clone(), true);
    let mut right = CallbackProperty::new(callback.clone(), true);

    assert!(left.equals(Some(&right)));

    right.set_callback(callback.clone(), false);
    assert!(!left.equals(Some(&right)));

    let diff_cb = Rc::new(|_: &JulianDate| Some(1));
    right.set_callback(diff_cb, true);
    assert!(!left.equals(Some(&right)));
    assert!(!left.equals(None));
}
