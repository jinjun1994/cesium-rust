//! Direct port and verification against CesiumJS `DataSources/PolylineGraphicsSpec.js` (Apache-2.0).

use cesium_math::{
    ArcType, Cartesian3, ClassificationType, Color, ColorMaterialProperty, ConstantProperty,
    DistanceDisplayCondition, MaterialValue, PolylineGraphics, PolylineGraphicsValues, ShadowMode,
};
use std::rc::Rc;

#[test]
fn creates_expected_instance_from_raw_assignment_and_construction() {
    let positions = vec![
        Cartesian3::new(1.0, 2.0, 3.0),
        Cartesian3::new(4.0, 5.0, 6.0),
    ];
    let values = PolylineGraphicsValues {
        material: Some(Color::BLUE),
        depth_fail_material: Some(Color::RED),
        positions: Some(positions.clone()),
        show: Some(true),
        width: Some(1.0),
        clamp_to_ground: Some(true),
        granularity: Some(2.0),
        shadows: Some(ShadowMode::Disabled),
        distance_display_condition: Some(DistanceDisplayCondition::new(10.0, 100.0)),
        classification_type: Some(ClassificationType::Terrain),
        arc_type: Some(ArcType::Geodesic),
        z_index: Some(0),
    };

    let polyline = PolylineGraphics::from_values(values);

    assert_eq!(
        polyline
            .material
            .as_ref()
            .unwrap()
            .get_material_value(None)
            .unwrap(),
        MaterialValue::Color(Color::BLUE)
    );
    assert_eq!(
        polyline
            .depth_fail_material
            .as_ref()
            .unwrap()
            .get_material_value(None)
            .unwrap(),
        MaterialValue::Color(Color::RED)
    );
    assert_eq!(
        polyline.positions.as_ref().unwrap().get_value(None).unwrap(),
        positions
    );
    assert!(polyline.show.as_ref().unwrap().get_value(None).unwrap());
    assert_eq!(
        polyline.width.as_ref().unwrap().get_value(None).unwrap(),
        1.0
    );
    assert!(polyline
        .clamp_to_ground
        .as_ref()
        .unwrap()
        .get_value(None)
        .unwrap());
    assert_eq!(
        polyline.granularity.as_ref().unwrap().get_value(None).unwrap(),
        2.0
    );
    assert_eq!(
        polyline.shadows.as_ref().unwrap().get_value(None).unwrap(),
        ShadowMode::Disabled
    );
    assert_eq!(
        polyline
            .distance_display_condition
            .as_ref()
            .unwrap()
            .get_value(None)
            .unwrap(),
        DistanceDisplayCondition::new(10.0, 100.0)
    );
    assert_eq!(
        polyline
            .classification_type
            .as_ref()
            .unwrap()
            .get_value(None)
            .unwrap(),
        ClassificationType::Terrain
    );
    assert_eq!(
        polyline.arc_type.as_ref().unwrap().get_value(None).unwrap(),
        ArcType::Geodesic
    );
    assert_eq!(
        polyline.z_index.as_ref().unwrap().get_value(None).unwrap(),
        0
    );
}

#[test]
fn merge_assigns_unassigned_properties() {
    let mut source = PolylineGraphics::new();
    source.material = Some(Rc::new(ColorMaterialProperty::default()));
    source.depth_fail_material = Some(Rc::new(ColorMaterialProperty::default()));
    source.positions = Some(Rc::new(ConstantProperty::from_value(vec![
        Cartesian3::ZERO,
    ])));
    source.width = Some(Rc::new(ConstantProperty::from_value(1.0)));
    source.show = Some(Rc::new(ConstantProperty::from_value(true)));
    source.clamp_to_ground = Some(Rc::new(ConstantProperty::from_value(true)));
    source.granularity = Some(Rc::new(ConstantProperty::from_value(2.0)));
    source.shadows = Some(Rc::new(ConstantProperty::from_value(ShadowMode::Enabled)));
    source.distance_display_condition = Some(Rc::new(ConstantProperty::from_value(
        DistanceDisplayCondition::new(10.0, 100.0),
    )));
    source.classification_type = Some(Rc::new(ConstantProperty::from_value(
        ClassificationType::Terrain,
    )));
    source.arc_type = Some(Rc::new(ConstantProperty::from_value(ArcType::Geodesic)));
    source.z_index = Some(Rc::new(ConstantProperty::from_value(3)));

    let mut target = PolylineGraphics::new();
    target.merge(&source);

    assert!(target.material.is_some());
    assert!(target.depth_fail_material.is_some());
    assert!(target.positions.is_some());
    assert!(target.width.is_some());
    assert!(target.show.is_some());
    assert!(target.clamp_to_ground.is_some());
    assert!(target.granularity.is_some());
    assert!(target.shadows.is_some());
    assert!(target.distance_display_condition.is_some());
    assert!(target.classification_type.is_some());
    assert!(target.arc_type.is_some());
    assert!(target.z_index.is_some());
}

#[test]
fn merge_does_not_assign_assigned_properties() {
    let mut source = PolylineGraphics::new();
    source.show = Some(Rc::new(ConstantProperty::from_value(false)));

    let mut target = PolylineGraphics::new();
    target.show = Some(Rc::new(ConstantProperty::from_value(true)));
    target.merge(&source);

    assert!(target.show.as_ref().unwrap().get_value(None).unwrap());
}

#[test]
fn clone_works() {
    let mut source = PolylineGraphics::new();
    source.material = Some(Rc::new(ColorMaterialProperty::default()));
    source.depth_fail_material = Some(Rc::new(ColorMaterialProperty::default()));
    source.positions = Some(Rc::new(ConstantProperty::from_value(vec![
        Cartesian3::new(1.0, 1.0, 1.0),
    ])));
    source.show = Some(Rc::new(ConstantProperty::from_value(true)));
    source.width = Some(Rc::new(ConstantProperty::from_value(3.0)));
    source.clamp_to_ground = Some(Rc::new(ConstantProperty::from_value(false)));
    source.granularity = Some(Rc::new(ConstantProperty::from_value(0.1)));
    source.shadows = Some(Rc::new(ConstantProperty::from_value(ShadowMode::Enabled)));
    source.distance_display_condition = Some(Rc::new(ConstantProperty::from_value(
        DistanceDisplayCondition::new(10.0, 100.0),
    )));
    source.classification_type = Some(Rc::new(ConstantProperty::from_value(
        ClassificationType::Both,
    )));
    source.arc_type = Some(Rc::new(ConstantProperty::from_value(ArcType::Rhumb)));
    source.z_index = Some(Rc::new(ConstantProperty::from_value(5)));

    let cloned = source.clone();
    assert!(cloned.material.is_some());
    assert!(cloned.depth_fail_material.is_some());
    assert_eq!(
        cloned.positions.as_ref().unwrap().get_value(None).unwrap(),
        vec![Cartesian3::new(1.0, 1.0, 1.0)]
    );
    assert!(cloned.show.as_ref().unwrap().get_value(None).unwrap());
    assert_eq!(
        cloned.width.as_ref().unwrap().get_value(None).unwrap(),
        3.0
    );
    assert!(!cloned
        .clamp_to_ground
        .as_ref()
        .unwrap()
        .get_value(None)
        .unwrap());
    assert_eq!(
        cloned.granularity.as_ref().unwrap().get_value(None).unwrap(),
        0.1
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
        cloned.arc_type.as_ref().unwrap().get_value(None).unwrap(),
        ArcType::Rhumb
    );
    assert_eq!(
        cloned.z_index.as_ref().unwrap().get_value(None).unwrap(),
        5
    );
}
