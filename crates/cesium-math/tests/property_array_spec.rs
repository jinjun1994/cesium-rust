use std::cell::Cell;
use std::rc::Rc;

use cesium_math::{
    CallbackProperty, ConstantProperty, DynProperty, JulianDate, PropertyArray,
};

#[test]
fn default_constructor_sets_expected_values() {
    let time = JulianDate::now();
    let property = PropertyArray::<i32>::default();
    assert!(property.is_constant());
    assert_eq!(property.get_value(Some(&time)), None);
}

#[test]
fn constructor_sets_expected_values() {
    let time = JulianDate::now();
    let items: Vec<Rc<dyn DynProperty<i32>>> = vec![
        Rc::new(ConstantProperty::from_value(1)),
        Rc::new(ConstantProperty::from_value(2)),
    ];
    let property = PropertyArray::new(Some(items));
    assert_eq!(property.get_value(Some(&time)), Some(vec![1, 2]));
}

#[test]
fn set_value_raises_definition_changed_event() {
    let mut property = PropertyArray::<i32>::default();
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
    let mut property = PropertyArray::<i32>::default();
    let item = Rc::new(ConstantProperty::from_value(1));
    property.set_value(Some(vec![item.clone()]));

    let called = Rc::new(Cell::new(false));
    let called_clone = Rc::clone(&called);
    property.definition_changed().add_listener(move |_| {
        called_clone.set(true);
    });

    item.set_value(Some(2));
    assert!(called.get());
}

#[test]
fn works_with_undefined_value() {
    let time = JulianDate::now();
    let mut property = PropertyArray::<i32>::default();
    property.set_value(None);
    assert_eq!(property.get_value(Some(&time)), None);
}

#[test]
fn ignores_undefined_property_values() {
    let time = JulianDate::now();
    let mut property = PropertyArray::<i32>::default();
    let item: Rc<dyn DynProperty<i32>> = Rc::new(ConstantProperty::new(None));
    property.set_value(Some(vec![item]));
    assert_eq!(property.get_value(Some(&time)), Some(vec![]));
}

#[test]
fn works_with_empty_array() {
    let time = JulianDate::now();
    let mut property = PropertyArray::<i32>::default();
    property.set_value(Some(vec![]));
    assert_eq!(property.get_value(Some(&time)), Some(vec![]));
}

#[test]
fn equals_works() {
    let left = PropertyArray::new(Some(vec![Rc::new(ConstantProperty::from_value(1))]));
    let mut right = PropertyArray::new(Some(vec![Rc::new(ConstantProperty::from_value(1))]));
    assert!(left.equals(Some(&right)));

    right = PropertyArray::new(Some(vec![Rc::new(ConstantProperty::from_value(2))]));
    assert!(!left.equals(Some(&right)));

    let empty_left = PropertyArray::<i32>::default();
    let empty_right = PropertyArray::<i32>::default();
    assert!(empty_left.equals(Some(&empty_right)));
}

#[test]
fn is_constant_is_true_only_if_all_members_are_constant() {
    let mut property = PropertyArray::<i32>::default();

    property.set_value(Some(vec![Rc::new(ConstantProperty::from_value(2))]));
    assert!(property.is_constant());

    let non_constant: Rc<dyn DynProperty<i32>> =
        Rc::new(CallbackProperty::new(Rc::new(|_| Some(1)), false));
    property.set_value(Some(vec![
        Rc::new(ConstantProperty::from_value(2)),
        non_constant,
    ]));
    assert!(!property.is_constant());
}
