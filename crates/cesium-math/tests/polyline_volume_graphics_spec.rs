//! Direct port and verification against CesiumJS `DataSources/PolylineVolumeGraphicsSpec.js` (Apache-2.0).

use cesium_math::{
    Cartesian2, Cartesian3, Color, ColorMaterialProperty, ConstantProperty, CornerType,
    DistanceDisplayCondition, MaterialValue, PolylineVolumeGraphics, PolylineVolumeGraphicsValues,
    ShadowMode,
};
use std::rc::Rc;

#[test]
fn creates_expected_instance_from_raw_assignment_and_construction() {
    let positions = vec![
        Cartesian3::new(1.0, 2.0, 3.0),
        Cartesian3::new(4.0, 5.0, 6.0),
    ];
    let shape = vec![Cartesian2::new(-1.0, -1.0), Cartesian2::new(1.0, 1.0)];
    let values = PolylineVolumeGraphicsValues {
        material: Some(Color::BLUE),
        positions: Some(positions.clone()),
        shape: Some(shape.clone()),
        show: Some(true),
        granularity: Some(1.0),
        fill: Some(false),
        outline: Some(false),
        outline_color: Some(Color::RED),
        outline_width: Some(2.0),
        corner_type: Some(CornerType::Beveled),
        shadows: Some(ShadowMode::Disabled),
        distance_display_condition: Some(DistanceDisplayCondition::new(10.0, 100.0)),
    };

    let polyline_volume = PolylineVolumeGraphics::from_values(values);

    assert_eq!(
        polyline_volume
            .material
            .as_ref()
            .unwrap()
            .get_material_value(None)
            .unwrap(),
        MaterialValue::Color(Color::BLUE)
    );
    assert_eq!(
        polyline_volume
            .positions
            .as_ref()
            .unwrap()
            .get_value(None)
            .unwrap(),
        positions
    );
    assert_eq!(
        polyline_volume
            .shape
            .as_ref()
            .unwrap()
            .get_value(None)
            .unwrap(),
        shape
    );
    assert!(polyline_volume
        .show
        .as_ref()
        .unwrap()
        .get_value(None)
        .unwrap());
    assert_eq!(
        polyline_volume
            .granularity
            .as_ref()
            .unwrap()
            .get_value(None)
            .unwrap(),
        1.0
    );
    assert!(!polyline_volume
        .fill
        .as_ref()
        .unwrap()
        .get_value(None)
        .unwrap());
    assert!(!polyline_volume
        .outline
        .as_ref()
        .unwrap()
        .get_value(None)
        .unwrap());
    assert_eq!(
        polyline_volume
            .outline_color
            .as_ref()
            .unwrap()
            .get_value(None)
            .unwrap(),
        Color::RED
    );
    assert_eq!(
        polyline_volume
            .outline_width
            .as_ref()
            .unwrap()
            .get_value(None)
            .unwrap(),
        2.0
    );
    assert_eq!(
        polyline_volume
            .corner_type
            .as_ref()
            .unwrap()
            .get_value(None)
            .unwrap(),
        CornerType::Beveled
    );
    assert_eq!(
        polyline_volume
            .shadows
            .as_ref()
            .unwrap()
            .get_value(None)
            .unwrap(),
        ShadowMode::Disabled
    );
    assert_eq!(
        polyline_volume
            .distance_display_condition
            .as_ref()
            .unwrap()
            .get_value(None)
            .unwrap(),
        DistanceDisplayCondition::new(10.0, 100.0)
    );
}

#[test]
fn merge_assigns_unassigned_properties() {
    let mut source = PolylineVolumeGraphics::new();
    source.material = Some(Rc::new(ColorMaterialProperty::default()));
    source.positions = Some(Rc::new(ConstantProperty::from_value(vec![
        Cartesian3::ZERO,
    ])));
    source.show = Some(Rc::new(ConstantProperty::from_value(true)));
    source.shape = Some(Rc::new(ConstantProperty::from_value(vec![
        Cartesian2::ZERO,
    ])));
    source.granularity = Some(Rc::new(ConstantProperty::from_value(1.0)));
    source.fill = Some(Rc::new(ConstantProperty::from_value(true)));
    source.outline = Some(Rc::new(ConstantProperty::from_value(false)));
    source.outline_color = Some(Rc::new(ConstantProperty::from_value(Color::RED)));
    source.outline_width = Some(Rc::new(ConstantProperty::from_value(2.0)));
    source.corner_type = Some(Rc::new(ConstantProperty::from_value(CornerType::Mitered)));
    source.shadows = Some(Rc::new(ConstantProperty::from_value(ShadowMode::Enabled)));
    source.distance_display_condition = Some(Rc::new(ConstantProperty::from_value(
        DistanceDisplayCondition::new(10.0, 100.0),
    )));

    let mut target = PolylineVolumeGraphics::new();
    target.merge(&source);

    assert!(target.material.is_some());
    assert!(target.positions.is_some());
    assert!(target.shape.is_some());
    assert!(target.show.is_some());
    assert!(target.granularity.is_some());
    assert!(target.fill.is_some());
    assert!(target.outline.is_some());
    assert!(target.outline_color.is_some());
    assert!(target.outline_width.is_some());
    assert!(target.corner_type.is_some());
    assert!(target.shadows.is_some());
    assert!(target.distance_display_condition.is_some());
}

#[test]
fn merge_does_not_assign_assigned_properties() {
    let mut source = PolylineVolumeGraphics::new();
    source.show = Some(Rc::new(ConstantProperty::from_value(false)));

    let mut target = PolylineVolumeGraphics::new();
    target.show = Some(Rc::new(ConstantProperty::from_value(true)));
    target.merge(&source);

    assert!(target.show.as_ref().unwrap().get_value(None).unwrap());
}

#[test]
fn clone_works() {
    let mut source = PolylineVolumeGraphics::new();
    source.material = Some(Rc::new(ColorMaterialProperty::default()));
    source.positions = Some(Rc::new(ConstantProperty::from_value(vec![
        Cartesian3::new(1.0, 1.0, 1.0),
    ])));
    source.shape = Some(Rc::new(ConstantProperty::from_value(vec![
        Cartesian2::new(2.0, 2.0),
    ])));
    source.show = Some(Rc::new(ConstantProperty::from_value(true)));
    source.granularity = Some(Rc::new(ConstantProperty::from_value(0.1)));
    source.fill = Some(Rc::new(ConstantProperty::from_value(true)));
    source.outline = Some(Rc::new(ConstantProperty::from_value(false)));
    source.outline_color = Some(Rc::new(ConstantProperty::from_value(Color::RED)));
    source.outline_width = Some(Rc::new(ConstantProperty::from_value(3.0)));
    source.corner_type = Some(Rc::new(ConstantProperty::from_value(CornerType::Rounded)));
    source.shadows = Some(Rc::new(ConstantProperty::from_value(ShadowMode::Enabled)));
    source.distance_display_condition = Some(Rc::new(ConstantProperty::from_value(
        DistanceDisplayCondition::new(10.0, 100.0),
    )));

    let cloned = source.clone();
    assert!(cloned.material.is_some());
    assert_eq!(
        cloned.positions.as_ref().unwrap().get_value(None).unwrap(),
        vec![Cartesian3::new(1.0, 1.0, 1.0)]
    );
    assert_eq!(
        cloned.shape.as_ref().unwrap().get_value(None).unwrap(),
        vec![Cartesian2::new(2.0, 2.0)]
    );
    assert!(cloned.show.as_ref().unwrap().get_value(None).unwrap());
    assert_eq!(
        cloned.granularity.as_ref().unwrap().get_value(None).unwrap(),
        0.1
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
}
