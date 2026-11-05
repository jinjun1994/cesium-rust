//! Direct port and verification against CesiumJS `DataSources/EllipseGraphicsSpec.js` (Apache-2.0).

use cesium_math::{
    ClassificationType, Color, ColorMaterialProperty, ConstantProperty, DistanceDisplayCondition,
    EllipseGraphics, EllipseGraphicsValues, MaterialValue, ShadowMode,
};
use std::rc::Rc;

#[test]
fn creates_expected_instance_from_raw_assignment_and_construction() {
    let values = EllipseGraphicsValues {
        material: Some(Color::BLUE),
        show: Some(true),
        semi_minor_axis: Some(1.0),
        semi_major_axis: Some(2.0),
        height: Some(3.0),
        extruded_height: Some(4.0),
        granularity: Some(5.0),
        rotation: Some(6.0),
        st_rotation: Some(7.0),
        number_of_vertical_lines: Some(8),
        fill: Some(false),
        outline: Some(false),
        outline_color: Some(Color::RED),
        outline_width: Some(9.0),
        shadows: Some(ShadowMode::Disabled),
        distance_display_condition: Some(DistanceDisplayCondition::new(10.0, 100.0)),
        classification_type: Some(ClassificationType::Terrain),
        z_index: Some(3),
    };

    let ellipse = EllipseGraphics::from_values(values);

    assert_eq!(
        ellipse
            .material
            .as_ref()
            .unwrap()
            .get_material_value(None)
            .unwrap(),
        MaterialValue::Color(Color::BLUE)
    );
    assert!(ellipse.show.as_ref().unwrap().get_value(None).unwrap());
    assert_eq!(
        ellipse
            .semi_minor_axis
            .as_ref()
            .unwrap()
            .get_value(None)
            .unwrap(),
        1.0
    );
    assert_eq!(
        ellipse
            .semi_major_axis
            .as_ref()
            .unwrap()
            .get_value(None)
            .unwrap(),
        2.0
    );
    assert_eq!(
        ellipse.height.as_ref().unwrap().get_value(None).unwrap(),
        3.0
    );
    assert_eq!(
        ellipse
            .extruded_height
            .as_ref()
            .unwrap()
            .get_value(None)
            .unwrap(),
        4.0
    );
    assert_eq!(
        ellipse
            .granularity
            .as_ref()
            .unwrap()
            .get_value(None)
            .unwrap(),
        5.0
    );
    assert_eq!(
        ellipse.rotation.as_ref().unwrap().get_value(None).unwrap(),
        6.0
    );
    assert_eq!(
        ellipse
            .st_rotation
            .as_ref()
            .unwrap()
            .get_value(None)
            .unwrap(),
        7.0
    );
    assert_eq!(
        ellipse
            .number_of_vertical_lines
            .as_ref()
            .unwrap()
            .get_value(None)
            .unwrap(),
        8
    );
    assert!(!ellipse.fill.as_ref().unwrap().get_value(None).unwrap());
    assert!(!ellipse.outline.as_ref().unwrap().get_value(None).unwrap());
    assert_eq!(
        ellipse
            .outline_color
            .as_ref()
            .unwrap()
            .get_value(None)
            .unwrap(),
        Color::RED
    );
    assert_eq!(
        ellipse
            .outline_width
            .as_ref()
            .unwrap()
            .get_value(None)
            .unwrap(),
        9.0
    );
    assert_eq!(
        ellipse.shadows.as_ref().unwrap().get_value(None).unwrap(),
        ShadowMode::Disabled
    );
    assert_eq!(
        ellipse
            .distance_display_condition
            .as_ref()
            .unwrap()
            .get_value(None)
            .unwrap(),
        DistanceDisplayCondition::new(10.0, 100.0)
    );
    assert_eq!(
        ellipse
            .classification_type
            .as_ref()
            .unwrap()
            .get_value(None)
            .unwrap(),
        ClassificationType::Terrain
    );
    assert_eq!(
        ellipse.z_index.as_ref().unwrap().get_value(None).unwrap(),
        3
    );
}

#[test]
fn merge_assigns_unassigned_properties() {
    let mut source = EllipseGraphics::new();
    source.material = Some(Rc::new(ColorMaterialProperty::default()));
    source.semi_minor_axis = Some(Rc::new(ConstantProperty::from_value(1.0)));
    source.semi_major_axis = Some(Rc::new(ConstantProperty::from_value(2.0)));
    source.show = Some(Rc::new(ConstantProperty::from_value(true)));
    source.height = Some(Rc::new(ConstantProperty::from_value(3.0)));
    source.extruded_height = Some(Rc::new(ConstantProperty::from_value(4.0)));
    source.granularity = Some(Rc::new(ConstantProperty::from_value(5.0)));
    source.rotation = Some(Rc::new(ConstantProperty::from_value(6.0)));
    source.st_rotation = Some(Rc::new(ConstantProperty::from_value(7.0)));
    source.fill = Some(Rc::new(ConstantProperty::from_value(true)));
    source.outline = Some(Rc::new(ConstantProperty::from_value(false)));
    source.outline_color = Some(Rc::new(ConstantProperty::from_value(Color::RED)));
    source.outline_width = Some(Rc::new(ConstantProperty::from_value(9.0)));
    source.number_of_vertical_lines = Some(Rc::new(ConstantProperty::from_value(8)));
    source.shadows = Some(Rc::new(ConstantProperty::from_value(ShadowMode::Enabled)));
    source.distance_display_condition = Some(Rc::new(ConstantProperty::from_value(
        DistanceDisplayCondition::new(10.0, 100.0),
    )));
    source.classification_type = Some(Rc::new(ConstantProperty::from_value(
        ClassificationType::Terrain,
    )));
    source.z_index = Some(Rc::new(ConstantProperty::from_value(3)));

    let mut target = EllipseGraphics::new();
    target.merge(&source);

    assert!(target.material.is_some());
    assert!(target.semi_minor_axis.is_some());
    assert!(target.semi_major_axis.is_some());
    assert!(target.show.is_some());
    assert!(target.height.is_some());
    assert!(target.extruded_height.is_some());
    assert!(target.granularity.is_some());
    assert!(target.rotation.is_some());
    assert!(target.st_rotation.is_some());
    assert!(target.fill.is_some());
    assert!(target.outline.is_some());
    assert!(target.outline_color.is_some());
    assert!(target.outline_width.is_some());
    assert!(target.number_of_vertical_lines.is_some());
    assert!(target.shadows.is_some());
    assert!(target.distance_display_condition.is_some());
    assert!(target.classification_type.is_some());
    assert!(target.z_index.is_some());
}

#[test]
fn merge_does_not_assign_assigned_properties() {
    let mut source = EllipseGraphics::new();
    source.show = Some(Rc::new(ConstantProperty::from_value(false)));

    let mut target = EllipseGraphics::new();
    target.show = Some(Rc::new(ConstantProperty::from_value(true)));
    target.merge(&source);

    assert!(target.show.as_ref().unwrap().get_value(None).unwrap());
}

#[test]
fn clone_works() {
    let mut source = EllipseGraphics::new();
    source.material = Some(Rc::new(ColorMaterialProperty::default()));
    source.semi_minor_axis = Some(Rc::new(ConstantProperty::from_value(10.0)));
    source.semi_major_axis = Some(Rc::new(ConstantProperty::from_value(20.0)));
    source.show = Some(Rc::new(ConstantProperty::from_value(true)));
    source.height = Some(Rc::new(ConstantProperty::from_value(30.0)));
    source.extruded_height = Some(Rc::new(ConstantProperty::from_value(40.0)));
    source.granularity = Some(Rc::new(ConstantProperty::from_value(0.1)));
    source.rotation = Some(Rc::new(ConstantProperty::from_value(1.0)));
    source.st_rotation = Some(Rc::new(ConstantProperty::from_value(2.0)));
    source.fill = Some(Rc::new(ConstantProperty::from_value(true)));
    source.outline = Some(Rc::new(ConstantProperty::from_value(false)));
    source.outline_color = Some(Rc::new(ConstantProperty::from_value(Color::RED)));
    source.outline_width = Some(Rc::new(ConstantProperty::from_value(3.0)));
    source.number_of_vertical_lines = Some(Rc::new(ConstantProperty::from_value(16)));
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
    assert_eq!(
        cloned
            .semi_minor_axis
            .as_ref()
            .unwrap()
            .get_value(None)
            .unwrap(),
        10.0
    );
    assert_eq!(
        cloned
            .semi_major_axis
            .as_ref()
            .unwrap()
            .get_value(None)
            .unwrap(),
        20.0
    );
    assert!(cloned.show.as_ref().unwrap().get_value(None).unwrap());
    assert_eq!(
        cloned.height.as_ref().unwrap().get_value(None).unwrap(),
        30.0
    );
    assert_eq!(
        cloned
            .extruded_height
            .as_ref()
            .unwrap()
            .get_value(None)
            .unwrap(),
        40.0
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
        cloned
            .number_of_vertical_lines
            .as_ref()
            .unwrap()
            .get_value(None)
            .unwrap(),
        16
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
