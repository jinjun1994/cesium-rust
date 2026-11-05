use std::cell::Cell;
use std::rc::Rc;

use cesium_math::{
    Cartesian3, ConstantProperty, JulianDate, NodeTransformationProperty, Quaternion, TimeInterval,
    TimeIntervalCollectionProperty, TimeStandard, TranslationRotationScale,
};

#[test]
fn default_constructor_sets_expected_values() {
    let property = NodeTransformationProperty::new();
    assert!(property.is_constant());
    assert!(property.translation().is_none());
    assert!(property.rotation().is_none());
    assert!(property.scale().is_none());

    let result = property.get_value(None).unwrap();
    assert_eq!(result.translation, Cartesian3::ZERO);
    assert_eq!(result.rotation, Quaternion::IDENTITY);
    assert_eq!(result.scale, Cartesian3::new(1.0, 1.0, 1.0));
}

#[test]
fn constructor_sets_options_and_allows_raw_assignment() {
    let translation = Cartesian3::UNIT_Y;
    let rotation = Quaternion::new(0.5, 0.5, 0.5, 0.5);
    let scale = Cartesian3::UNIT_X;

    let property =
        NodeTransformationProperty::from_values(Some(translation), Some(rotation), Some(scale));

    assert!(property.translation().is_some());
    assert!(property.rotation().is_some());
    assert!(property.scale().is_some());

    assert_eq!(
        property.translation().unwrap().get_value(None).unwrap(),
        translation
    );
    assert_eq!(
        property.rotation().unwrap().get_value(None).unwrap(),
        rotation
    );
    assert_eq!(property.scale().unwrap().get_value(None).unwrap(), scale);

    let result = property.get_value(None).unwrap();
    assert_eq!(result.translation, translation);
    assert_eq!(result.rotation, rotation);
    assert_eq!(result.scale, scale);
}

#[test]
fn works_with_constant_values() {
    let property = NodeTransformationProperty::new();
    property.set_translation_value(Cartesian3::UNIT_Y);
    property.set_rotation_value(Quaternion::new(0.5, 0.5, 0.5, 0.5));
    property.set_scale_value(Cartesian3::UNIT_X);

    let now = JulianDate::now();
    let result = property.get_value(Some(&now)).unwrap();
    assert_eq!(result.translation, Cartesian3::UNIT_Y);
    assert_eq!(result.rotation, Quaternion::new(0.5, 0.5, 0.5, 0.5));
    assert_eq!(result.scale, Cartesian3::UNIT_X);
}

#[test]
fn works_with_dynamic_values() {
    let property = NodeTransformationProperty::new();

    let mut trans_prop = TimeIntervalCollectionProperty::<Cartesian3>::new();
    let mut rot_prop = TimeIntervalCollectionProperty::<Quaternion>::new();
    let mut scale_prop = TimeIntervalCollectionProperty::<Cartesian3>::new();

    let start = JulianDate::new(1.0, 0.0, TimeStandard::TAI);
    let stop = JulianDate::new(2.0, 0.0, TimeStandard::TAI);

    let t_interval = TimeInterval {
        start,
        stop,
        data: Some(Cartesian3::UNIT_Y),
        ..Default::default()
    };
    trans_prop.intervals.add_interval(t_interval);

    let r_interval = TimeInterval {
        start,
        stop,
        data: Some(Quaternion::new(0.5, 0.5, 0.5, 0.5)),
        ..Default::default()
    };
    rot_prop.intervals.add_interval(r_interval);

    let s_interval = TimeInterval {
        start,
        stop,
        data: Some(Cartesian3::UNIT_X),
        ..Default::default()
    };
    scale_prop.intervals.add_interval(s_interval);

    property.set_translation(Some(Rc::new(trans_prop)));
    property.set_rotation(Some(Rc::new(rot_prop)));
    property.set_scale(Some(Rc::new(scale_prop)));

    assert!(!property.is_constant());

    let result = property.get_value(Some(&start)).unwrap();
    assert_eq!(result.translation, Cartesian3::UNIT_Y);
    assert_eq!(result.rotation, Quaternion::new(0.5, 0.5, 0.5, 0.5));
    assert_eq!(result.scale, Cartesian3::UNIT_X);
}

#[test]
fn works_with_a_result_parameter() {
    let property = NodeTransformationProperty::new();
    property.set_translation_value(Cartesian3::UNIT_Y);
    property.set_rotation_value(Quaternion::new(0.5, 0.5, 0.5, 0.5));
    property.set_scale_value(Cartesian3::UNIT_X);

    let mut result = TranslationRotationScale::default();
    let now = JulianDate::now();
    property.get_value_to(Some(&now), &mut result);

    assert_eq!(result.translation, Cartesian3::UNIT_Y);
    assert_eq!(result.rotation, Quaternion::new(0.5, 0.5, 0.5, 0.5));
    assert_eq!(result.scale, Cartesian3::UNIT_X);
}

#[test]
fn equals_works() {
    let left = NodeTransformationProperty::new();
    left.set_translation_value(Cartesian3::UNIT_Y);
    left.set_rotation_value(Quaternion::new(0.5, 0.5, 0.5, 0.5));
    left.set_scale_value(Cartesian3::UNIT_X);

    let right = NodeTransformationProperty::new();
    right.set_translation_value(Cartesian3::UNIT_Y);
    right.set_rotation_value(Quaternion::new(0.5, 0.5, 0.5, 0.5));
    right.set_scale_value(Cartesian3::UNIT_X);

    assert!(left.equals(Some(&right)));

    right.set_scale_value(Cartesian3::ZERO);
    assert!(!left.equals(Some(&right)));

    right.set_scale_value(Cartesian3::UNIT_X);
    right.set_translation_value(Cartesian3::ZERO);
    assert!(!left.equals(Some(&right)));

    right.set_translation_value(Cartesian3::UNIT_Y);
    right.set_rotation_value(Quaternion::ZERO);
    assert!(!left.equals(Some(&right)));
}

#[test]
fn raises_definition_changed_when_a_property_is_assigned_or_modified() {
    let property = NodeTransformationProperty::new();
    let count = Rc::new(Cell::new(0));

    let c = count.clone();
    let _sub = property.definition_changed().add_listener(move |_| {
        c.set(c.get() + 1);
    });

    // 1. Assign new translation
    let t_prop = Rc::new(ConstantProperty::from_value(Cartesian3::UNIT_X));
    property.set_translation(Some(t_prop.clone()));
    assert_eq!(count.get(), 1);

    // 2. Modifying translation's inner value triggers parent's definition_changed
    t_prop.set_value(Some(Cartesian3::ZERO));
    assert_eq!(count.get(), 2);

    // 3. Re-assigning the same pointer doesn't trigger
    property.set_translation(Some(t_prop));
    assert_eq!(count.get(), 2);

    // 4. Assign new rotation
    let r_prop = Rc::new(ConstantProperty::from_value(Quaternion::new(
        0.5, 0.5, 0.5, 0.5,
    )));
    property.set_rotation(Some(r_prop.clone()));
    assert_eq!(count.get(), 3);

    // Modifying rotation triggers parent
    r_prop.set_value(Some(Quaternion::ZERO));
    assert_eq!(count.get(), 4);

    // 5. Assign new scale
    let s_prop = Rc::new(ConstantProperty::from_value(Cartesian3::UNIT_X));
    property.set_scale(Some(s_prop.clone()));
    assert_eq!(count.get(), 5);

    // Modifying scale triggers parent
    s_prop.set_value(Some(Cartesian3::ZERO));
    assert_eq!(count.get(), 6);
}
