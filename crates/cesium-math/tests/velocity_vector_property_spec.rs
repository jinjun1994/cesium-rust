use std::cell::Cell;
use std::rc::Rc;

use cesium_math::{
    Cartesian3, ConstantPositionProperty, DynPositionProperty, JulianDate,
    SampledPositionProperty, TimeStandard, VelocityVectorProperty,
};

#[test]
fn can_default_construct() {
    let time = JulianDate::now();
    let property = VelocityVectorProperty::default();
    assert!(property.is_constant());
    assert!(property.position().is_none());
    assert_eq!(property.get_value(Some(&time)), None);
    assert!(property.normalize());
}

#[test]
fn can_construct_with_arguments() {
    let time = JulianDate::now();
    let position: Rc<dyn DynPositionProperty> = Rc::new(SampledPositionProperty::default());
    let property = VelocityVectorProperty::new(Some(position.clone()), Some(false));
    assert!(property.is_constant());
    assert!(property.position().is_some());
    assert_eq!(property.get_value(Some(&time)), Some(Cartesian3::ZERO));
    assert!(!property.normalize());
}

#[test]
fn raises_definition_changed_event_when_position_is_set() {
    let mut property = VelocityVectorProperty::default();
    let called = Rc::new(Cell::new(false));
    let called_clone = Rc::clone(&called);
    property.definition_changed().add_listener(move |_| {
        called_clone.set(true);
    });

    let position: Rc<dyn DynPositionProperty> = Rc::new(SampledPositionProperty::default());
    property.set_position(Some(position));
    assert!(called.get());
}

#[test]
fn raises_definition_changed_event_when_normalize_changes() {
    let position: Rc<dyn DynPositionProperty> = Rc::new(SampledPositionProperty::default());
    let mut property = VelocityVectorProperty::new(Some(position), None);

    let count = Rc::new(Cell::new(0));
    let count_clone = Rc::clone(&count);
    property.definition_changed().add_listener(move |_| {
        count_clone.set(count_clone.get() + 1);
    });

    property.set_normalize(true);
    assert_eq!(count.get(), 0);

    property.set_normalize(false);
    assert_eq!(count.get(), 1);
}

#[test]
fn subscribes_and_unsubscribes_to_position_definition_changed_and_propagates_up() {
    let position = Rc::new(SampledPositionProperty::default());
    let mut property = VelocityVectorProperty::new(Some(position.clone()), None);

    let count = Rc::new(Cell::new(0));
    let count_clone = Rc::clone(&count);
    property.definition_changed().add_listener(move |_| {
        count_clone.set(count_clone.get() + 1);
    });

    position.definition_changed().raise_event(&());
    assert_eq!(count.get(), 1);

    property.set_position(None);
    count.set(0);

    position.definition_changed().raise_event(&());
    assert_eq!(count.get(), 0);
}

#[test]
fn does_not_raise_definition_changed_event_when_position_is_set_to_the_same_instance() {
    let position: Rc<dyn DynPositionProperty> = Rc::new(SampledPositionProperty::default());
    let mut property = VelocityVectorProperty::new(Some(position.clone()), None);

    let count = Rc::new(Cell::new(0));
    let count_clone = Rc::clone(&count);
    property.definition_changed().add_listener(move |_| {
        count_clone.set(count_clone.get() + 1);
    });

    property.set_position(Some(position));
    assert_eq!(count.get(), 0);
}

#[test]
fn produces_correct_normalized_value() {
    let times = [
        JulianDate::new(0.0, 0.0, TimeStandard::TAI),
        JulianDate::new(0.0, 1.0 / 60.0, TimeStandard::TAI),
    ];
    let values = [
        Cartesian3::new(0.0, 0.0, 0.0),
        Cartesian3::new(20.0, 0.0, 0.0),
    ];

    let mut position = SampledPositionProperty::default();
    position.add_samples(&times, &values);

    let property = VelocityVectorProperty::new(Some(Rc::new(position)), None);
    let expected = Cartesian3::new(1.0, 0.0, 0.0);

    assert_eq!(property.get_value(Some(&times[0])), Some(expected));
    assert_eq!(property.get_value(Some(&times[1])), Some(expected));
}

#[test]
fn produces_correct_unnormalized_value() {
    let times = [
        JulianDate::new(0.0, 0.0, TimeStandard::TAI),
        JulianDate::new(0.0, 1.0, TimeStandard::TAI),
    ];
    let values = [
        Cartesian3::new(0.0, 0.0, 0.0),
        Cartesian3::new(20.0, 0.0, 0.0),
    ];

    let mut position = SampledPositionProperty::default();
    position.add_samples(&times, &values);

    let property = VelocityVectorProperty::new(Some(Rc::new(position)), Some(false));
    let expected = Cartesian3::new(20.0, 0.0, 0.0);

    let val0 = property.get_value(Some(&times[0])).unwrap();
    let val1 = property.get_value(Some(&times[1])).unwrap();
    assert!((val0.x - expected.x).abs() < 1e-10);
    assert!((val1.x - expected.x).abs() < 1e-10);
}

#[test]
fn produces_normalized_value_of_none_with_constant_position() {
    let position = ConstantPositionProperty::new(Some(Cartesian3::new(1.0, 2.0, 3.0)), None);
    let property = VelocityVectorProperty::new(Some(Rc::new(position)), Some(true));
    assert_eq!(property.get_value(Some(&JulianDate::now())), None);
}

#[test]
fn produces_unnormalized_value_of_zero_with_constant_position() {
    let position = ConstantPositionProperty::new(Some(Cartesian3::new(1.0, 2.0, 3.0)), None);
    let property = VelocityVectorProperty::new(Some(Rc::new(position)), Some(false));
    assert_eq!(
        property.get_value(Some(&JulianDate::now())),
        Some(Cartesian3::ZERO)
    );
}

#[test]
fn equals_works() {
    let position = Rc::new(SampledPositionProperty::default());

    let mut left = VelocityVectorProperty::default();
    let mut right = VelocityVectorProperty::default();

    assert!(left.equals(Some(&right)));

    left.set_position(Some(position.clone()));
    assert!(!left.equals(Some(&right)));

    right.set_position(Some(position));
    assert!(left.equals(Some(&right)));
}
