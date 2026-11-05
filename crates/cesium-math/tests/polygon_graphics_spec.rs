//! Direct port and verification against CesiumJS `DataSources/PolygonGraphicsSpec.js` (Apache-2.0).

use cesium_math::{
    ArcType, Cartesian3, ClassificationType, Color, ColorMaterialProperty, ConstantProperty,
    DistanceDisplayCondition, MaterialValue, PolygonGraphics, PolygonGraphicsValues,
    PolygonHierarchy, ShadowMode,
};
use std::rc::Rc;

#[test]
fn creates_expected_instance_from_raw_assignment_and_construction() {
    let hierarchy = PolygonHierarchy::new(vec![
        Cartesian3::new(1.0, 0.0, 0.0),
        Cartesian3::new(0.0, 1.0, 0.0),
        Cartesian3::new(0.0, 0.0, 1.0),
    ]);
    let values = PolygonGraphicsValues {
        material: Some(Color::BLUE),
        show: Some(true),
        hierarchy: Some(hierarchy.clone()),
        height: Some(2.0),
        extruded_height: Some(3.0),
        granularity: Some(4.0),
        st_rotation: Some(5.0),
        per_position_height: Some(false),
        fill: Some(false),
        outline: Some(false),
        outline_color: Some(Color::RED),
        outline_width: Some(7.0),
        close_top: Some(true),
        close_bottom: Some(true),
        shadows: Some(ShadowMode::Disabled),
        distance_display_condition: Some(DistanceDisplayCondition::new(10.0, 100.0)),
        classification_type: Some(ClassificationType::Terrain),
        arc_type: Some(ArcType::Geodesic),
        z_index: Some(22),
        texture_coordinates: Some(PolygonHierarchy::default()),
        ..Default::default()
    };

    let polygon = PolygonGraphics::from_values(values);

    assert_eq!(
        polygon
            .material
            .as_ref()
            .unwrap()
            .get_material_value(None)
            .unwrap(),
        MaterialValue::Color(Color::BLUE)
    );
    assert!(polygon.show.as_ref().unwrap().get_value(None).unwrap());
    assert_eq!(
        polygon.hierarchy.as_ref().unwrap().get_value(None).unwrap(),
        hierarchy
    );
    assert_eq!(
        polygon.height.as_ref().unwrap().get_value(None).unwrap(),
        2.0
    );
    assert_eq!(
        polygon
            .extruded_height
            .as_ref()
            .unwrap()
            .get_value(None)
            .unwrap(),
        3.0
    );
    assert_eq!(
        polygon.granularity.as_ref().unwrap().get_value(None).unwrap(),
        4.0
    );
    assert_eq!(
        polygon.st_rotation.as_ref().unwrap().get_value(None).unwrap(),
        5.0
    );
    assert!(!polygon
        .per_position_height
        .as_ref()
        .unwrap()
        .get_value(None)
        .unwrap());
    assert!(!polygon.fill.as_ref().unwrap().get_value(None).unwrap());
    assert!(!polygon.outline.as_ref().unwrap().get_value(None).unwrap());
    assert_eq!(
        polygon
            .outline_color
            .as_ref()
            .unwrap()
            .get_value(None)
            .unwrap(),
        Color::RED
    );
    assert_eq!(
        polygon
            .outline_width
            .as_ref()
            .unwrap()
            .get_value(None)
            .unwrap(),
        7.0
    );
    assert!(polygon.close_top.as_ref().unwrap().get_value(None).unwrap());
    assert!(polygon
        .close_bottom
        .as_ref()
        .unwrap()
        .get_value(None)
        .unwrap());
    assert_eq!(
        polygon.shadows.as_ref().unwrap().get_value(None).unwrap(),
        ShadowMode::Disabled
    );
    assert_eq!(
        polygon
            .distance_display_condition
            .as_ref()
            .unwrap()
            .get_value(None)
            .unwrap(),
        DistanceDisplayCondition::new(10.0, 100.0)
    );
    assert_eq!(
        polygon
            .classification_type
            .as_ref()
            .unwrap()
            .get_value(None)
            .unwrap(),
        ClassificationType::Terrain
    );
    assert_eq!(
        polygon.arc_type.as_ref().unwrap().get_value(None).unwrap(),
        ArcType::Geodesic
    );
    assert_eq!(
        polygon.z_index.as_ref().unwrap().get_value(None).unwrap(),
        22
    );
}

#[test]
fn merge_assigns_unassigned_properties() {
    let mut source = PolygonGraphics::new();
    source.material = Some(Rc::new(ColorMaterialProperty::default()));
    source.hierarchy = Some(Rc::new(ConstantProperty::from_value(
        PolygonHierarchy::default(),
    )));
    source.show = Some(Rc::new(ConstantProperty::from_value(true)));
    source.height = Some(Rc::new(ConstantProperty::from_value(1.0)));
    source.extruded_height = Some(Rc::new(ConstantProperty::from_value(2.0)));
    source.granularity = Some(Rc::new(ConstantProperty::from_value(3.0)));
    source.st_rotation = Some(Rc::new(ConstantProperty::from_value(4.0)));
    source.fill = Some(Rc::new(ConstantProperty::from_value(true)));
    source.outline = Some(Rc::new(ConstantProperty::from_value(false)));
    source.outline_color = Some(Rc::new(ConstantProperty::from_value(Color::RED)));
    source.outline_width = Some(Rc::new(ConstantProperty::from_value(5.0)));
    source.per_position_height = Some(Rc::new(ConstantProperty::from_value(true)));
    source.close_top = Some(Rc::new(ConstantProperty::from_value(false)));
    source.close_bottom = Some(Rc::new(ConstantProperty::from_value(false)));
    source.shadows = Some(Rc::new(ConstantProperty::from_value(ShadowMode::Enabled)));
    source.distance_display_condition = Some(Rc::new(ConstantProperty::from_value(
        DistanceDisplayCondition::new(10.0, 100.0),
    )));
    source.classification_type = Some(Rc::new(ConstantProperty::from_value(
        ClassificationType::Terrain,
    )));
    source.arc_type = Some(Rc::new(ConstantProperty::from_value(ArcType::Rhumb)));
    source.z_index = Some(Rc::new(ConstantProperty::from_value(3)));

    let mut target = PolygonGraphics::new();
    target.merge(&source);

    assert!(target.material.is_some());
    assert!(target.hierarchy.is_some());
    assert!(target.show.is_some());
    assert!(target.height.is_some());
    assert!(target.extruded_height.is_some());
    assert!(target.granularity.is_some());
    assert!(target.st_rotation.is_some());
    assert!(target.fill.is_some());
    assert!(target.outline.is_some());
    assert!(target.outline_color.is_some());
    assert!(target.outline_width.is_some());
    assert!(target.per_position_height.is_some());
    assert!(target.close_top.is_some());
    assert!(target.close_bottom.is_some());
    assert!(target.shadows.is_some());
    assert!(target.distance_display_condition.is_some());
    assert!(target.classification_type.is_some());
    assert!(target.arc_type.is_some());
    assert!(target.z_index.is_some());
}

#[test]
fn merge_does_not_assign_assigned_properties() {
    let mut source = PolygonGraphics::new();
    source.show = Some(Rc::new(ConstantProperty::from_value(false)));

    let mut target = PolygonGraphics::new();
    target.show = Some(Rc::new(ConstantProperty::from_value(true)));
    target.merge(&source);

    assert!(target.show.as_ref().unwrap().get_value(None).unwrap());
}

#[test]
fn clone_works() {
    let hierarchy = PolygonHierarchy::new(vec![Cartesian3::new(1.0, 1.0, 1.0)]);
    let mut source = PolygonGraphics::new();
    source.material = Some(Rc::new(ColorMaterialProperty::default()));
    source.hierarchy = Some(Rc::new(ConstantProperty::from_value(hierarchy.clone())));
    source.show = Some(Rc::new(ConstantProperty::from_value(true)));
    source.height = Some(Rc::new(ConstantProperty::from_value(10.0)));
    source.extruded_height = Some(Rc::new(ConstantProperty::from_value(20.0)));
    source.granularity = Some(Rc::new(ConstantProperty::from_value(0.1)));
    source.st_rotation = Some(Rc::new(ConstantProperty::from_value(0.5)));
    source.fill = Some(Rc::new(ConstantProperty::from_value(true)));
    source.outline = Some(Rc::new(ConstantProperty::from_value(false)));
    source.outline_color = Some(Rc::new(ConstantProperty::from_value(Color::RED)));
    source.outline_width = Some(Rc::new(ConstantProperty::from_value(3.0)));
    source.per_position_height = Some(Rc::new(ConstantProperty::from_value(true)));
    source.close_top = Some(Rc::new(ConstantProperty::from_value(false)));
    source.close_bottom = Some(Rc::new(ConstantProperty::from_value(true)));
    source.arc_type = Some(Rc::new(ConstantProperty::from_value(ArcType::None)));
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
        cloned.hierarchy.as_ref().unwrap().get_value(None).unwrap(),
        hierarchy
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
        cloned.st_rotation.as_ref().unwrap().get_value(None).unwrap(),
        0.5
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
    assert!(cloned
        .per_position_height
        .as_ref()
        .unwrap()
        .get_value(None)
        .unwrap());
    assert!(!cloned.close_top.as_ref().unwrap().get_value(None).unwrap());
    assert!(cloned
        .close_bottom
        .as_ref()
        .unwrap()
        .get_value(None)
        .unwrap());
    assert_eq!(
        cloned.arc_type.as_ref().unwrap().get_value(None).unwrap(),
        ArcType::None
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
