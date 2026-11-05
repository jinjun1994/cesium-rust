use std::cell::Cell;
use std::rc::Rc;

use cesium_math::{CallbackProperty, ConstantProperty, DynProperty, JulianDate, PropertyBag};

#[test]
fn sets_correct_values_when_constructed_with_no_arguments() {
    let time = JulianDate::now();
    let property = PropertyBag::<i32>::default();
    assert!(property.is_constant());
    let value = property.get_value(Some(&time));
    assert!(value.is_empty());
}

#[test]
fn sets_correct_values_when_constructed_with_arguments() {
    let time = JulianDate::now();
    let property = PropertyBag::from_entries(vec![
        (
            "a".to_string(),
            Some(Rc::new(ConstantProperty::from_value(1)) as Rc<dyn DynProperty<i32>>),
        ),
        (
            "b".to_string(),
            Some(Rc::new(ConstantProperty::from_value(2)) as Rc<dyn DynProperty<i32>>),
        ),
    ]);

    assert!(property.property_names().contains(&"a".to_string()));
    assert!(property.has_property("a"));
    assert!(property.property_names().contains(&"b".to_string()));
    assert!(property.has_property("b"));

    let val = property.get_value(Some(&time));
    assert_eq!(val.get("a"), Some(&Some(1)));
    assert_eq!(val.get("b"), Some(&Some(2)));
}

#[test]
fn allows_adding_a_property_without_a_value() {
    let time = JulianDate::now();
    let mut property = PropertyBag::<i32>::default();
    property.add_property("a", None);

    assert_eq!(property.property_names(), &["a".to_string()]);
    assert!(property.has_property("a"));

    let val = property.get_value(Some(&time));
    assert_eq!(val.get("a"), Some(&None));
}

#[test]
fn allows_adding_a_property_with_a_value() {
    let time = JulianDate::now();
    let mut property = PropertyBag::<i32>::default();
    property.add_property("a", Some(Rc::new(ConstantProperty::from_value(1))));

    assert_eq!(property.property_names(), &["a".to_string()]);
    assert!(property.has_property("a"));

    let val = property.get_value(Some(&time));
    assert_eq!(val.get("a"), Some(&Some(1)));
}

#[test]
fn allows_removing_a_property_that_was_previously_added() {
    let time = JulianDate::now();
    let mut property = PropertyBag::<i32>::default();

    property.add_property("a", Some(Rc::new(ConstantProperty::from_value(1))));
    assert!(property.has_property("a"));

    property.remove_property("a");
    assert!(property.property_names().is_empty());
    assert!(!property.has_property("a"));
    assert!(property.get_value(Some(&time)).is_empty());
}

#[test]
#[should_panic(expected = "a is not a registered property.")]
fn throws_when_removing_a_property_that_was_not_added() {
    let mut property = PropertyBag::<i32>::default();
    property.remove_property("a");
}

#[test]
fn raises_definition_changed_event_when_add_property_is_called() {
    let mut property = PropertyBag::<i32>::default();
    let called = Rc::new(Cell::new(false));
    let called_clone = Rc::clone(&called);
    property.definition_changed().add_listener(move |_| {
        called_clone.set(true);
    });

    property.add_property("a", None);
    assert!(called.get());
}

#[test]
fn raises_definition_changed_event_when_properties_are_changed() {
    let mut property = PropertyBag::<i32>::default();
    property.add_property("a", None);

    let called = Rc::new(Cell::new(false));
    let called_clone = Rc::clone(&called);
    property.definition_changed().add_listener(move |_| {
        called_clone.set(true);
    });

    property.set_property_value("a", Some(Rc::new(ConstantProperty::from_value(1))));
    assert!(called.get());
}

#[test]
fn equals_works() {
    let left = PropertyBag::from_entries(vec![(
        "a".to_string(),
        Some(Rc::new(ConstantProperty::from_value(1)) as Rc<dyn DynProperty<i32>>),
    )]);
    let mut right = PropertyBag::from_entries(vec![(
        "a".to_string(),
        Some(Rc::new(ConstantProperty::from_value(1)) as Rc<dyn DynProperty<i32>>),
    )]);

    assert!(left.equals(Some(&right)));

    right.add_property("c", None);
    assert!(!left.equals(Some(&right)));

    right = PropertyBag::from_entries(vec![(
        "a".to_string(),
        Some(Rc::new(ConstantProperty::from_value(2)) as Rc<dyn DynProperty<i32>>),
    )]);
    assert!(!left.equals(Some(&right)));

    right = PropertyBag::from_entries(vec![(
        "b".to_string(),
        Some(Rc::new(ConstantProperty::from_value(1)) as Rc<dyn DynProperty<i32>>),
    )]);
    assert!(!left.equals(Some(&right)));

    right = PropertyBag::default();
    assert!(!left.equals(Some(&right)));

    let empty_left = PropertyBag::<i32>::default();
    let empty_right = PropertyBag::<i32>::default();
    assert!(empty_left.equals(Some(&empty_right)));
}

#[test]
fn returns_true_from_is_constant_only_if_all_members_are_constant() {
    let mut property = PropertyBag::<i32>::default();

    property.add_property("a", Some(Rc::new(ConstantProperty::from_value(2))));
    assert!(property.is_constant());

    let non_constant: Rc<dyn DynProperty<i32>> =
        Rc::new(CallbackProperty::new(Rc::new(|_| Some(1)), false));
    property.add_property("b", Some(non_constant));
    assert!(!property.is_constant());
}
