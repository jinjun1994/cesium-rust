//! Direct port and verification against CesiumJS `DataSources/CorridorGraphicsSpec.js` (Apache-2.0).

use cesium_math::{
    Cartesian3, ClassificationType, Color, ColorMaterialProperty, ConstantProperty, CornerType,
    CorridorGraphics, CorridorGraphicsValues, DistanceDisplayCondition, MaterialValue, ShadowMode,
};
use std::rc::Rc;

#[test]
fn creates_expected_instance_from_raw_assignment_and_construction() {
    let positions = vec![
        Cartesian3::new(1.0, 2.0, 3.0),
        Cartesian3::new(4.0, 5.0, 6.0),
    ];
    let values = CorridorGraphicsValues {
        material: Some(Color::BLUE),
        positions: Some(positions.clone()),
        show: Some(true),
        height: Some(1.0),
        extruded_height: Some(2.0),
        granularity: Some(3.0),
        width: Some(4.0),
        fill: Some(false),
        outline: Some(false),
        outline_color: Some(Color::RED),
        outline_width: Some(5.0),
        corner_type: Some(CornerType::Beveled),
        shadows: Some(ShadowMode::Disabled),
        distance_display_condition: Some(DistanceDisplayCondition::new(10.0, 100.0)),
        classification_type: Some(ClassificationType::Terrain),
        z_index: Some(3),
    };

    let corridor = CorridorGraphics::from_values(values);

    assert_eq!(
        corridor
            .material
            .as_ref()
            .unwrap()
            .get_material_value(None)
            .unwrap(),
        MaterialValue::Color(Color::BLUE)
    );
    assert_eq!(
        corridor.positions.as_ref().unwrap().get_value(None).unwrap(),
        positions
    );
    assert!(corridor.show.as_ref().unwrap().get_value(None).unwrap());
    assert_eq!(
        corridor.height.as_ref().unwrap().get_value(None).unwrap(),
        1.0
    );
    assert_eq!(
        corridor
            .extruded_height
            .as_ref()
            .unwrap()
            .get_value(None)
            .unwrap(),
        2.0
    );
    assert_eq!(
        corridor
            .granularity
            .as_ref()
            .unwrap()
            .get_value(None)
            .unwrap(),
        3.0
    );
    assert_eq!(
        corridor.width.as_ref().unwrap().get_value(None).unwrap(),
        4.0
    );
    assert!(!corridor.fill.as_ref().unwrap().get_value(None).unwrap());
    assert!(!corridor.outline.as_ref().unwrap().get_value(None).unwrap());
    assert_eq!(
        corridor
            .outline_color
            .as_ref()
            .unwrap()
            .get_value(None)
            .unwrap(),
        Color::RED
    );
    assert_eq!(
        corridor
            .outline_width
            .as_ref()
            .unwrap()
            .get_value(None)
            .unwrap(),
        5.0
    );
    assert_eq!(
        corridor
            .corner_type
            .as_ref()
            .unwrap()
            .get_value(None)
            .unwrap(),
        CornerType::Beveled
    );
    assert_eq!(
        corridor.shadows.as_ref().unwrap().get_value(None).unwrap(),
        ShadowMode::Disabled
    );
    assert_eq!(
        corridor
            .distance_display_condition
            .as_ref()
            .unwrap()
            .get_value(None)
            .unwrap(),
        DistanceDisplayCondition::new(10.0, 100.0)
    );
    assert_eq!(
        corridor
            .classification_type
            .as_ref()
            .unwrap()
            .get_value(None)
            .unwrap(),
        ClassificationType::Terrain
    );
    assert_eq!(
        corridor.z_index.as_ref().unwrap().get_value(None).unwrap(),
        3
    );
}

#[test]
fn merge_assigns_unassigned_properties() {
    let mut source = CorridorGraphics::new();
    source.material = Some(Rc::new(ColorMaterialProperty::default()));
    source.positions = Some(Rc::new(ConstantProperty::from_value(vec![
        Cartesian3::ZERO,
    ])));
    source.show = Some(Rc::new(ConstantProperty::from_value(true)));
    source.height = Some(Rc::new(ConstantProperty::from_value(1.0)));
    source.extruded_height = Some(Rc::new(ConstantProperty::from_value(2.0)));
    source.granularity = Some(Rc::new(ConstantProperty::from_value(3.0)));
    source.width = Some(Rc::new(ConstantProperty::from_value(4.0)));
    source.fill = Some(Rc::new(ConstantProperty::from_value(true)));
    source.outline = Some(Rc::new(ConstantProperty::from_value(false)));
    source.outline_color = Some(Rc::new(ConstantProperty::from_value(Color::RED)));
    source.outline_width = Some(Rc::new(ConstantProperty::from_value(5.0)));
    source.corner_type = Some(Rc::new(ConstantProperty::from_value(CornerType::Mitered)));
    source.shadows = Some(Rc::new(ConstantProperty::from_value(ShadowMode::Enabled)));
    source.distance_display_condition = Some(Rc::new(ConstantProperty::from_value(
        DistanceDisplayCondition::new(10.0, 100.0),
    )));
    source.classification_type = Some(Rc::new(ConstantProperty::from_value(
        ClassificationType::Terrain,
    )));
    source.z_index = Some(Rc::new(ConstantProperty::from_value(3)));

    let mut target = CorridorGraphics::new();
    target.merge(&source);

    assert!(target.material.is_some());
    assert!(target.positions.is_some());
    assert!(target.show.is_some());
    assert!(target.height.is_some());
    assert!(target.extruded_height.is_some());
    assert!(target.granularity.is_some());
    assert!(target.width.is_some());
    assert!(target.fill.is_some());
    assert!(target.outline.is_some());
    assert!(target.outline_color.is_some());
    assert!(target.outline_width.is_some());
    assert!(target.corner_type.is_some());
    assert!(target.shadows.is_some());
    assert!(target.distance_display_condition.is_some());
    assert!(target.classification_type.is_some());
    assert!(target.z_index.is_some());
}

#[test]
fn merge_does_not_assign_assigned_properties() {
    let mut source = CorridorGraphics::new();
    source.show = Some(Rc::new(ConstantProperty::from_value(false)));

    let mut target = CorridorGraphics::new();
    target.show = Some(Rc::new(ConstantProperty::from_value(true)));
    target.merge(&source);

    assert!(target.show.as_ref().unwrap().get_value(None).unwrap());
}

#[test]
fn clone_works() {
    let mut source = CorridorGraphics::new();
    source.material = Some(Rc::new(ColorMaterialProperty::default()));
    source.positions = Some(Rc::new(ConstantProperty::from_value(vec![
        Cartesian3::new(1.0, 1.0, 1.0),
    ])));
    source.show = Some(Rc::new(ConstantProperty::from_value(true)));
    source.height = Some(Rc::new(ConstantProperty::from_value(10.0)));
    source.extruded_height = Some(Rc::new(ConstantProperty::from_value(20.0)));
    source.granularity = Some(Rc::new(ConstantProperty::from_value(0.1)));
    source.width = Some(Rc::new(ConstantProperty::from_value(5.0)));
    source.fill = Some(Rc::new(ConstantProperty::from_value(true)));
    source.outline = Some(Rc::new(ConstantProperty::from_value(false)));
    source.outline_color = Some(Rc::new(ConstantProperty::from_value(Color::RED)));
    source.outline_width = Some(Rc::new(ConstantProperty::from_value(3.0)));
    source.corner_type = Some(Rc::new(ConstantProperty::from_value(CornerType::Rounded)));
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
        cloned.positions.as_ref().unwrap().get_value(None).unwrap(),
        vec![Cartesian3::new(1.0, 1.0, 1.0)]
    );
    assert!(cloned.show.as_ref().unwrap().get_value(None).unwrap());
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
        cloned.width.as_ref().unwrap().get_value(None).unwrap(),
        5.0
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
            .corner_type
            .as_ref()
            .unwrap()
            .get_value(None)
            .unwrap(),
        CornerType::Rounded
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
