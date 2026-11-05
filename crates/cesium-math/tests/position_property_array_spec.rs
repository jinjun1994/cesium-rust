use std::cell::Cell;
use std::rc::Rc;

use cesium_math::{
    CallbackPositionProperty, Cartesian3, ConstantPositionProperty, DynPositionProperty,
    JulianDate, PositionPropertyArray, ReferenceFrame,
};

#[test]
fn default_constructor_sets_expected_values() {
    let time = JulianDate::now();
    let property = PositionPropertyArray::default();
    assert!(property.is_constant());
    assert_eq!(property.get_value(Some(&time)), None);
}

#[test]
fn constructor_sets_expected_values() {
    let time = JulianDate::now();
    let expected = vec![Cartesian3::UNIT_X, Cartesian3::UNIT_Z];
    let items: Vec<Rc<dyn DynPositionProperty>> = vec![
        Rc::new(ConstantPositionProperty::new(Some(Cartesian3::UNIT_X), None)),
        Rc::new(ConstantPositionProperty::new(Some(Cartesian3::UNIT_Z), None)),
    ];
    let property = PositionPropertyArray::new(Some(items), None);
    assert_eq!(property.get_value(Some(&time)), Some(expected));
}

#[test]
fn set_value_raises_definition_changed_event() {
    let mut property = PositionPropertyArray::default();
    let called = Rc::new(Cell::new(false));
    let called_clone = Rc::clone(&called);
    property.definition_changed().add_listener(move |_| {
        called_clone.set(true);
    });

    property.set_value(Some(vec![]));
    assert!(called.get());
}

#[test]
fn changing_array_member_raises_definition_changed_event() {
    let mut property = PositionPropertyArray::default();
    let item = Rc::new(ConstantPositionProperty::new(Some(Cartesian3::UNIT_X), None));
    property.set_value(Some(vec![item.clone()]));

    let called = Rc::new(Cell::new(false));
    let called_clone = Rc::clone(&called);
    property.definition_changed().add_listener(move |_| {
        called_clone.set(true);
    });

    item.set_value(Some(Cartesian3::UNIT_Z), None);
    assert!(called.get());
}

#[test]
fn works_with_reference_frame_parameter() {
    let time = JulianDate::now();
    let val0 = ConstantPositionProperty::new(
        Some(Cartesian3::UNIT_X),
        Some(ReferenceFrame::Inertial),
    );
    let val1 = ConstantPositionProperty::new(
        Some(Cartesian3::UNIT_Z),
        Some(ReferenceFrame::Fixed),
    );

    let expected = vec![
        val0.get_value_in_reference_frame(Some(&time), ReferenceFrame::Inertial)
            .unwrap(),
        val1.get_value_in_reference_frame(Some(&time), ReferenceFrame::Inertial)
            .unwrap(),
    ];

    let items: Vec<Rc<dyn DynPositionProperty>> = vec![Rc::new(val0), Rc::new(val1)];
    let property = PositionPropertyArray::new(Some(items), None);
    let result = property
        .get_value_in_reference_frame(Some(&time), ReferenceFrame::Inertial)
        .unwrap();

    assert_eq!(result, expected);
}

#[test]
fn works_with_undefined_value() {
    let time = JulianDate::now();
    let mut property = PositionPropertyArray::default();
    property.set_value(None);
    assert_eq!(property.get_value(Some(&time)), None);
}

#[test]
fn works_with_undefined_property_value() {
    let time = JulianDate::now();
    let mut property = PositionPropertyArray::default();
    let item: Rc<dyn DynPositionProperty> =
        Rc::new(ConstantPositionProperty::new(None, None));
    property.set_value(Some(vec![item]));
    assert_eq!(property.get_value(Some(&time)), Some(vec![]));
}

#[test]
fn works_with_empty_array() {
    let time = JulianDate::now();
    let mut property = PositionPropertyArray::default();
    property.set_value(Some(vec![]));
    assert_eq!(property.get_value(Some(&time)), Some(vec![]));
}

#[test]
fn equals_works() {
    let left = PositionPropertyArray::new(
        Some(vec![Rc::new(ConstantPositionProperty::new(
            Some(Cartesian3::UNIT_X),
            None,
        ))]),
        None,
    );
    let mut right = PositionPropertyArray::new(
        Some(vec![Rc::new(ConstantPositionProperty::new(
            Some(Cartesian3::UNIT_X),
            None,
        ))]),
        None,
    );
    assert!(left.equals(Some(&right)));

    right = PositionPropertyArray::new(
        Some(vec![Rc::new(ConstantPositionProperty::new(
            Some(Cartesian3::UNIT_Z),
            None,
        ))]),
        None,
    );
    assert!(!left.equals(Some(&right)));

    let empty_left = PositionPropertyArray::default();
    let empty_right = PositionPropertyArray::default();
    assert!(empty_left.equals(Some(&empty_right)));
}

#[test]
fn is_constant_is_true_only_if_all_members_are_constant() {
    let mut property = PositionPropertyArray::default();

    property.set_value(Some(vec![Rc::new(ConstantPositionProperty::new(
        Some(Cartesian3::UNIT_X),
        None,
    ))]));
    assert!(property.is_constant());

    let non_constant: Rc<dyn DynPositionProperty> = Rc::new(CallbackPositionProperty::new(
        Rc::new(|_| Some(Cartesian3::UNIT_X)),
        false,
        None,
    ));
    property.set_value(Some(vec![
        Rc::new(ConstantPositionProperty::new(
            Some(Cartesian3::UNIT_X),
            None,
        )),
        non_constant,
    ]));
    assert!(!property.is_constant());
}
