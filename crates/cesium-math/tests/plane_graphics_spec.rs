//! Direct port and verification against CesiumJS `DataSources/PlaneGraphicsSpec.js` (Apache-2.0).

use cesium_math::{
    Cartesian2, Cartesian3, Color, ColorMaterialProperty, ConstantProperty,
    DistanceDisplayCondition, MaterialValue, Plane, PlaneGraphics, PlaneGraphicsValues, ShadowMode,
};
use std::rc::Rc;

#[test]
fn creates_expected_instance_from_raw_assignment_and_construction() {
    let test_plane = Plane::new(Cartesian3::UNIT_X.into(), 0.0).unwrap();
    let values = PlaneGraphicsValues {
        material: Some(Color::BLUE),
        show: Some(true),
        fill: Some(false),
        outline: Some(false),
        outline_color: Some(Color::RED),
        outline_width: Some(1.0),
        plane: Some(test_plane),
        dimensions: Some(Cartesian2::new(2.0, 3.0)),
        shadows: Some(ShadowMode::Disabled),
        distance_display_condition: Some(DistanceDisplayCondition::new(10.0, 100.0)),
    };

    let plane_graphics = PlaneGraphics::from_values(values);

    assert_eq!(
        plane_graphics
            .material
            .as_ref()
            .unwrap()
            .get_material_value(None)
            .unwrap(),
        MaterialValue::Color(Color::BLUE)
    );
    assert!(plane_graphics.show.as_ref().unwrap().get_value(None).unwrap());
    assert!(!plane_graphics.fill.as_ref().unwrap().get_value(None).unwrap());
    assert!(!plane_graphics.outline.as_ref().unwrap().get_value(None).unwrap());
    assert_eq!(
        plane_graphics
            .outline_color
            .as_ref()
            .unwrap()
            .get_value(None)
            .unwrap(),
        Color::RED
    );
    assert_eq!(
        plane_graphics
            .outline_width
            .as_ref()
            .unwrap()
            .get_value(None)
            .unwrap(),
        1.0
    );
    assert_eq!(
        plane_graphics.plane.as_ref().unwrap().get_value(None).unwrap(),
        test_plane
    );
    assert_eq!(
        plane_graphics
            .dimensions
            .as_ref()
            .unwrap()
            .get_value(None)
            .unwrap(),
        Cartesian2::new(2.0, 3.0)
    );
    assert_eq!(
        plane_graphics.shadows.as_ref().unwrap().get_value(None).unwrap(),
        ShadowMode::Disabled
    );
    assert_eq!(
        plane_graphics
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
    let test_plane = Plane::new(Cartesian3::UNIT_Z.into(), 0.0).unwrap();
    let mut source = PlaneGraphics::new();
    source.material = Some(Rc::new(ColorMaterialProperty::default()));
    source.show = Some(Rc::new(ConstantProperty::from_value(true)));
    source.fill = Some(Rc::new(ConstantProperty::from_value(true)));
    source.outline = Some(Rc::new(ConstantProperty::from_value(false)));
    source.outline_color = Some(Rc::new(ConstantProperty::from_value(Color::RED)));
    source.outline_width = Some(Rc::new(ConstantProperty::from_value(2.0)));
    source.plane = Some(Rc::new(ConstantProperty::from_value(test_plane)));
    source.dimensions = Some(Rc::new(ConstantProperty::from_value(Cartesian2::new(
        1.0, 2.0,
    ))));
    source.shadows = Some(Rc::new(ConstantProperty::from_value(ShadowMode::Enabled)));
    source.distance_display_condition = Some(Rc::new(ConstantProperty::from_value(
        DistanceDisplayCondition::new(10.0, 100.0),
    )));

    let mut target = PlaneGraphics::new();
    target.merge(&source);

    assert!(target.material.is_some());
    assert!(target.show.is_some());
    assert!(target.fill.is_some());
    assert!(target.outline.is_some());
    assert!(target.outline_color.is_some());
    assert!(target.outline_width.is_some());
    assert!(target.plane.is_some());
    assert!(target.dimensions.is_some());
    assert!(target.shadows.is_some());
    assert!(target.distance_display_condition.is_some());
}

#[test]
fn merge_does_not_assign_assigned_properties() {
    let mut source = PlaneGraphics::new();
    source.show = Some(Rc::new(ConstantProperty::from_value(false)));

    let mut target = PlaneGraphics::new();
    target.show = Some(Rc::new(ConstantProperty::from_value(true)));
    target.merge(&source);

    assert!(target.show.as_ref().unwrap().get_value(None).unwrap());
}

#[test]
fn clone_works() {
    let test_plane = Plane::new(Cartesian3::UNIT_Y.into(), 5.0).unwrap();
    let mut source = PlaneGraphics::new();
    source.material = Some(Rc::new(ColorMaterialProperty::default()));
    source.show = Some(Rc::new(ConstantProperty::from_value(true)));
    source.fill = Some(Rc::new(ConstantProperty::from_value(false)));
    source.outline = Some(Rc::new(ConstantProperty::from_value(true)));
    source.outline_color = Some(Rc::new(ConstantProperty::from_value(Color::BLUE)));
    source.outline_width = Some(Rc::new(ConstantProperty::from_value(4.0)));
    source.plane = Some(Rc::new(ConstantProperty::from_value(test_plane)));
    source.dimensions = Some(Rc::new(ConstantProperty::from_value(Cartesian2::new(
        10.0, 20.0,
    ))));
    source.shadows = Some(Rc::new(ConstantProperty::from_value(ShadowMode::CastOnly)));
    source.distance_display_condition = Some(Rc::new(ConstantProperty::from_value(
        DistanceDisplayCondition::new(5.0, 50.0),
    )));

    let cloned = source.clone();
    assert!(cloned.material.is_some());
    assert!(cloned.show.as_ref().unwrap().get_value(None).unwrap());
    assert!(!cloned.fill.as_ref().unwrap().get_value(None).unwrap());
    assert!(cloned.outline.as_ref().unwrap().get_value(None).unwrap());
    assert_eq!(
        cloned
            .outline_color
            .as_ref()
            .unwrap()
            .get_value(None)
            .unwrap(),
        Color::BLUE
    );
    assert_eq!(
        cloned
            .outline_width
            .as_ref()
            .unwrap()
            .get_value(None)
            .unwrap(),
        4.0
    );
    assert_eq!(
        cloned.plane.as_ref().unwrap().get_value(None).unwrap(),
        test_plane
    );
    assert_eq!(
        cloned.dimensions.as_ref().unwrap().get_value(None).unwrap(),
        Cartesian2::new(10.0, 20.0)
    );
    assert_eq!(
        cloned.shadows.as_ref().unwrap().get_value(None).unwrap(),
        ShadowMode::CastOnly
    );
    assert_eq!(
        cloned
            .distance_display_condition
            .as_ref()
            .unwrap()
            .get_value(None)
            .unwrap(),
        DistanceDisplayCondition::new(5.0, 50.0)
    );
}
