//! Direct port and verification against CesiumJS `CallbackPositionPropertySpec.js` (Apache-2.0).

use cesium_math::{
    CallbackPositionProperty, Cartesian3, JulianDate, PositionProperty, ReferenceFrame,
};
use std::cell::Cell;
use std::rc::Rc;

#[test]
fn constructor_sets_expected_defaults() {
    let callback = Rc::new(|_: &JulianDate| Some(Cartesian3::ZERO));
    let property = CallbackPositionProperty::new(callback.clone(), true, None);
    assert_eq!(property.reference_frame(), ReferenceFrame::Fixed);

    let property = CallbackPositionProperty::new(
        callback,
        true,
        Some(ReferenceFrame::Inertial),
    );
    assert_eq!(property.reference_frame(), ReferenceFrame::Inertial);
}

#[test]
fn callback_received_proper_parameters_and_returns_result() {
    let time = JulianDate::now();
    let value = Cartesian3::new(1.0, 2.0, 3.0);
    let callback = Rc::new(move |_time: &JulianDate| Some(value));
    let property = CallbackPositionProperty::new(callback, true, None);

    let result = property.get_value(Some(&time));
    assert_eq!(result, Some(value));
}

#[test]
fn get_value_returns_in_fixed_frame() {
    let time = JulianDate::now();
    let value_inertial = Cartesian3::new(1.0, 2.0, 3.0);
    let value_fixed = PositionProperty::convert_to_reference_frame(
        Some(&time),
        Some(value_inertial),
        ReferenceFrame::Inertial,
        ReferenceFrame::Fixed,
    );

    let callback = Rc::new(move |_time: &JulianDate| Some(value_inertial));
    let property = CallbackPositionProperty::new(
        callback,
        true,
        Some(ReferenceFrame::Inertial),
    );

    let result = property.get_value(Some(&time));
    assert_eq!(result, value_fixed);
}

#[test]
fn get_value_in_reference_frame_works() {
    let time = JulianDate::now();
    let value = Cartesian3::new(1.0, 2.0, 3.0);
    let callback = Rc::new(move |_time: &JulianDate| Some(value));
    let property = CallbackPositionProperty::new(callback, true, None);

    let result = property.get_value_in_reference_frame(Some(&time), ReferenceFrame::Fixed);
    assert_eq!(result, Some(value));

    let result_inertial = property.get_value_in_reference_frame(Some(&time), ReferenceFrame::Inertial);
    let expected = PositionProperty::convert_to_reference_frame(
        Some(&time),
        Some(value),
        ReferenceFrame::Fixed,
        ReferenceFrame::Inertial,
    );
    assert_eq!(result_inertial, expected);
}

#[test]
fn is_constant_returns_correct_value_and_set_callback_raises_event() {
    let callback = Rc::new(|_: &JulianDate| Some(Cartesian3::ZERO));
    let mut property = CallbackPositionProperty::new(callback.clone(), true, None);
    assert!(property.is_constant());

    let called = Rc::new(Cell::new(0));
    let called_clone = called.clone();
    property.definition_changed().add_listener(move |_| {
        called_clone.set(called_clone.get() + 1);
    });

    let new_cb = Rc::new(|_: &JulianDate| Some(Cartesian3::UNIT_X));
    property.set_callback(new_cb, false);
    assert!(!property.is_constant());
    assert_eq!(called.get(), 1);
}

#[test]
fn equals_works() {
    let callback = Rc::new(|_: &JulianDate| Some(Cartesian3::ZERO));
    let left = CallbackPositionProperty::new(callback.clone(), true, None);
    let mut right = CallbackPositionProperty::new(callback.clone(), true, None);

    assert!(left.equals(Some(&right)));

    right.set_callback(callback.clone(), false);
    assert!(!left.equals(Some(&right)));

    let diff_cb = Rc::new(|_: &JulianDate| Some(Cartesian3::ZERO));
    right.set_callback(diff_cb, true);
    assert!(!left.equals(Some(&right)));

    let inertial = CallbackPositionProperty::new(
        callback,
        true,
        Some(ReferenceFrame::Inertial),
    );
    assert!(!left.equals(Some(&inertial)));
    assert!(!left.equals(None));
}
