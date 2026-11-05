//! Direct port and verification against CesiumJS `ConstantPositionPropertySpec.js` (Apache-2.0).

use cesium_math::{
    Cartesian3, ConstantPositionProperty, JulianDate, PositionProperty, ReferenceFrame,
};
use std::cell::Cell;
use std::rc::Rc;

#[test]
fn constructor_sets_expected_defaults() {
    let property = ConstantPositionProperty::new(None, None);
    assert_eq!(property.reference_frame(), ReferenceFrame::Fixed);

    let property = ConstantPositionProperty::new(
        Some(Cartesian3::new(1.0, 2.0, 3.0)),
        Some(ReferenceFrame::Inertial),
    );
    assert_eq!(property.reference_frame(), ReferenceFrame::Inertial);
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

    let property = ConstantPositionProperty::new(
        Some(value_inertial),
        Some(ReferenceFrame::Inertial),
    );

    let result = property.get_value(Some(&time));
    assert_eq!(result, value_fixed);
}

#[test]
fn get_value_in_reference_frame_works() {
    let time = JulianDate::now();
    let value = Cartesian3::new(1.0, 2.0, 3.0);
    let property = ConstantPositionProperty::new(Some(value), None);

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
fn set_value_raises_definition_changed_event() {
    let mut property = ConstantPositionProperty::new(None, None);
    let called = Rc::new(Cell::new(0));
    let called_clone = called.clone();

    property.definition_changed().add_listener(move |_| {
        called_clone.set(called_clone.get() + 1);
    });

    property.set_value(Some(Cartesian3::new(1.0, 2.0, 3.0)), None);
    assert_eq!(called.get(), 1);

    // Changing frame also triggers
    property.set_value(
        Some(Cartesian3::new(1.0, 2.0, 3.0)),
        Some(ReferenceFrame::Inertial),
    );
    assert_eq!(called.get(), 2);
}

#[test]
fn set_value_does_not_raise_definition_changed_event_with_equal_data() {
    let mut property = ConstantPositionProperty::new(
        Some(Cartesian3::new(1.0, 2.0, 3.0)),
        Some(ReferenceFrame::Fixed),
    );
    let called = Rc::new(Cell::new(0));
    let called_clone = called.clone();

    property.definition_changed().add_listener(move |_| {
        called_clone.set(called_clone.get() + 1);
    });

    property.set_value(
        Some(Cartesian3::new(1.0, 2.0, 3.0)),
        Some(ReferenceFrame::Fixed),
    );
    assert_eq!(called.get(), 0);
}

#[test]
fn equals_works() {
    let left = ConstantPositionProperty::new(
        Some(Cartesian3::new(1.0, 2.0, 3.0)),
        Some(ReferenceFrame::Fixed),
    );
    let right_equal = ConstantPositionProperty::new(
        Some(Cartesian3::new(1.0, 2.0, 3.0)),
        Some(ReferenceFrame::Fixed),
    );
    let right_diff_pos = ConstantPositionProperty::new(
        Some(Cartesian3::new(1.0, 2.0, 4.0)),
        Some(ReferenceFrame::Fixed),
    );
    let right_diff_frame = ConstantPositionProperty::new(
        Some(Cartesian3::new(1.0, 2.0, 3.0)),
        Some(ReferenceFrame::Inertial),
    );

    assert!(left.equals(Some(&right_equal)));
    assert!(!left.equals(Some(&right_diff_pos)));
    assert!(!left.equals(Some(&right_diff_frame)));
    assert!(!left.equals(None));
}
