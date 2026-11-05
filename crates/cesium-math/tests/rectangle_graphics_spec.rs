//! Direct port and verification against CesiumJS `DataSources/RectangleGraphicsSpec.js` (Apache-2.0).

use cesium_math::{
    ClassificationType, Color, ColorMaterialProperty, ConstantProperty, DistanceDisplayCondition,
    MaterialValue, Rectangle, RectangleGraphics, RectangleGraphicsValues, ShadowMode,
};
use std::rc::Rc;

#[test]
fn creates_expected_instance_from_raw_assignment_and_construction() {
    let coordinates = Rectangle::new(0.1, 0.2, 0.3, 0.4);
    let values = RectangleGraphicsValues {
        material: Some(Color::BLUE),
        show: Some(true),
        coordinates: Some(coordinates),
        height: Some(5.0),
        extruded_height: Some(6.0),
        granularity: Some(7.0),
        rotation: Some(8.0),
        st_rotation: Some(9.0),
        fill: Some(false),
        outline: Some(false),
        outline_color: Some(Color::RED),
        outline_width: Some(10.0),
        shadows: Some(ShadowMode::Disabled),
        distance_display_condition: Some(DistanceDisplayCondition::new(10.0, 100.0)),
        classification_type: Some(ClassificationType::Terrain),
        z_index: Some(5),
        ..Default::default()
    };

    let rectangle = RectangleGraphics::from_values(values);

    assert_eq!(
        rectangle
            .material
            .as_ref()
            .unwrap()
            .get_material_value(None)
            .unwrap(),
        MaterialValue::Color(Color::BLUE)
    );
    assert!(rectangle.show.as_ref().unwrap().get_value(None).unwrap());
    assert_eq!(
        rectangle
            .coordinates
            .as_ref()
            .unwrap()
            .get_value(None)
            .unwrap(),
        coordinates
    );
    assert_eq!(
        rectangle.height.as_ref().unwrap().get_value(None).unwrap(),
        5.0
    );
    assert_eq!(
        rectangle
            .extruded_height
            .as_ref()
            .unwrap()
            .get_value(None)
            .unwrap(),
        6.0
    );
    assert_eq!(
        rectangle
            .granularity
            .as_ref()
            .unwrap()
            .get_value(None)
            .unwrap(),
        7.0
    );
    assert_eq!(
        rectangle
            .rotation
            .as_ref()
            .unwrap()
            .get_value(None)
            .unwrap(),
        8.0
    );
    assert_eq!(
        rectangle
            .st_rotation
            .as_ref()
            .unwrap()
            .get_value(None)
            .unwrap(),
        9.0
    );
    assert!(!rectangle.fill.as_ref().unwrap().get_value(None).unwrap());
    assert!(!rectangle.outline.as_ref().unwrap().get_value(None).unwrap());
    assert_eq!(
        rectangle
            .outline_color
            .as_ref()
            .unwrap()
            .get_value(None)
            .unwrap(),
        Color::RED
    );
    assert_eq!(
        rectangle
            .outline_width
            .as_ref()
            .unwrap()
            .get_value(None)
            .unwrap(),
        10.0
    );
    assert_eq!(
        rectangle.shadows.as_ref().unwrap().get_value(None).unwrap(),
        ShadowMode::Disabled
    );
    assert_eq!(
        rectangle
            .distance_display_condition
            .as_ref()
            .unwrap()
            .get_value(None)
            .unwrap(),
        DistanceDisplayCondition::new(10.0, 100.0)
    );
    assert_eq!(
        rectangle
            .classification_type
            .as_ref()
            .unwrap()
            .get_value(None)
            .unwrap(),
        ClassificationType::Terrain
    );
    assert_eq!(
        rectangle.z_index.as_ref().unwrap().get_value(None).unwrap(),
        5
    );
}

#[test]
fn merge_assigns_unassigned_properties() {
    let mut source = RectangleGraphics::new();
    source.material = Some(Rc::new(ColorMaterialProperty::default()));
    source.show = Some(Rc::new(ConstantProperty::from_value(true)));
    source.coordinates = Some(Rc::new(ConstantProperty::from_value(Rectangle::new(
        0.1, 0.2, 0.3, 0.4,
    ))));
    source.height = Some(Rc::new(ConstantProperty::from_value(1.0)));
    source.extruded_height = Some(Rc::new(ConstantProperty::from_value(2.0)));
    source.granularity = Some(Rc::new(ConstantProperty::from_value(3.0)));
    source.rotation = Some(Rc::new(ConstantProperty::from_value(4.0)));
    source.st_rotation = Some(Rc::new(ConstantProperty::from_value(5.0)));
    source.fill = Some(Rc::new(ConstantProperty::from_value(true)));
    source.outline = Some(Rc::new(ConstantProperty::from_value(false)));
    source.outline_color = Some(Rc::new(ConstantProperty::from_value(Color::RED)));
    source.outline_width = Some(Rc::new(ConstantProperty::from_value(6.0)));
    source.shadows = Some(Rc::new(ConstantProperty::from_value(ShadowMode::Enabled)));
    source.distance_display_condition = Some(Rc::new(ConstantProperty::from_value(
        DistanceDisplayCondition::new(10.0, 100.0),
    )));
    source.classification_type = Some(Rc::new(ConstantProperty::from_value(
        ClassificationType::Terrain,
    )));
    source.z_index = Some(Rc::new(ConstantProperty::from_value(3)));

    let mut target = RectangleGraphics::new();
    target.merge(&source);

    assert!(target.material.is_some());
    assert!(target.show.is_some());
    assert!(target.coordinates.is_some());
    assert!(target.height.is_some());
    assert!(target.extruded_height.is_some());
    assert!(target.granularity.is_some());
    assert!(target.rotation.is_some());
    assert!(target.st_rotation.is_some());
    assert!(target.fill.is_some());
    assert!(target.outline.is_some());
    assert!(target.outline_color.is_some());
    assert!(target.outline_width.is_some());
    assert!(target.shadows.is_some());
    assert!(target.distance_display_condition.is_some());
    assert!(target.classification_type.is_some());
    assert!(target.z_index.is_some());
}

#[test]
fn merge_does_not_assign_assigned_properties() {
    let mut source = RectangleGraphics::new();
    source.show = Some(Rc::new(ConstantProperty::from_value(false)));

    let mut target = RectangleGraphics::new();
    target.show = Some(Rc::new(ConstantProperty::from_value(true)));
    target.merge(&source);

    assert!(target.show.as_ref().unwrap().get_value(None).unwrap());
}

#[test]
fn clone_works() {
    let coordinates = Rectangle::new(0.5, 0.6, 0.7, 0.8);
    let mut source = RectangleGraphics::new();
    source.material = Some(Rc::new(ColorMaterialProperty::default()));
    source.show = Some(Rc::new(ConstantProperty::from_value(true)));
    source.coordinates = Some(Rc::new(ConstantProperty::from_value(coordinates)));
    source.height = Some(Rc::new(ConstantProperty::from_value(10.0)));
    source.extruded_height = Some(Rc::new(ConstantProperty::from_value(20.0)));
    source.granularity = Some(Rc::new(ConstantProperty::from_value(0.1)));
    source.rotation = Some(Rc::new(ConstantProperty::from_value(1.0)));
    source.st_rotation = Some(Rc::new(ConstantProperty::from_value(2.0)));
    source.fill = Some(Rc::new(ConstantProperty::from_value(true)));
    source.outline = Some(Rc::new(ConstantProperty::from_value(false)));
    source.outline_color = Some(Rc::new(ConstantProperty::from_value(Color::RED)));
    source.outline_width = Some(Rc::new(ConstantProperty::from_value(3.0)));
    source.shadows = Some(Rc::new(ConstantProperty::from_value(ShadowMode::Enabled)));
    source.distance_display_condition = Some(Rc::new(ConstantProperty::from_value(
        DistanceDisplayCondition::new(10.0, 100.0),
    )));
    source.classification_type = Some(Rc::new(ConstantProperty::from_value(
        ClassificationType::Both,
    )));
    source.z_index = Some(Rc::new(ConstantProperty::from_value(5)));

    let cloned = source.clone();
    assert!(cloned.material.is_some());
    assert!(cloned.show.as_ref().unwrap().get_value(None).unwrap());
    assert_eq!(
        cloned
            .coordinates
            .as_ref()
            .unwrap()
            .get_value(None)
            .unwrap(),
        coordinates
    );
    assert_eq!(
        cloned.height.as_ref().unwrap().get_value(None).unwrap(),
        10.0
    );
    assert_eq!(
        cloned
            .extruded_height
            .as_ref()
            .unwrap()
            .get_value(None)
            .unwrap(),
        20.0
    );
    assert_eq!(
        cloned.granularity.as_ref().unwrap().get_value(None).unwrap(),
        0.1
    );
    assert_eq!(
        cloned.rotation.as_ref().unwrap().get_value(None).unwrap(),
        1.0
    );
    assert_eq!(
        cloned.st_rotation.as_ref().unwrap().get_value(None).unwrap(),
        2.0
    );
    assert!(cloned.fill.as_ref().unwrap().get_value(None).unwrap());
    assert!(!cloned.outline.as_ref().unwrap().get_value(None).unwrap());
    assert_eq!(
        cloned
            .outline_color
            .as_ref()
            .unwrap()
            .get_value(None)
            .unwrap(),
        Color::RED
    );
    assert_eq!(
        cloned
            .outline_width
            .as_ref()
            .unwrap()
            .get_value(None)
            .unwrap(),
        3.0
    );
    assert_eq!(
        cloned.shadows.as_ref().unwrap().get_value(None).unwrap(),
        ShadowMode::Enabled
    );
    assert_eq!(
        cloned
            .distance_display_condition
            .as_ref()
            .unwrap()
            .get_value(None)
            .unwrap(),
        DistanceDisplayCondition::new(10.0, 100.0)
    );
    assert_eq!(
        cloned
            .classification_type
            .as_ref()
            .unwrap()
            .get_value(None)
            .unwrap(),
        ClassificationType::Both
    );
    assert_eq!(
        cloned.z_index.as_ref().unwrap().get_value(None).unwrap(),
        5
    );
}
