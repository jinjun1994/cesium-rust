//! Direct port and verification against CesiumJS `DataSources/BoxGraphicsSpec.js` (Apache-2.0).

use cesium_math::{
    BoxGraphics, BoxGraphicsValues, Cartesian3, Color, ColorMaterialProperty, ConstantProperty,
    DistanceDisplayCondition, MaterialValue, ShadowMode,
};
use std::rc::Rc;

#[test]
fn creates_expected_instance_from_raw_assignment_and_construction() {
    let values = BoxGraphicsValues {
        material: Some(Color::BLUE),
        show: Some(true),
        fill: Some(false),
        outline: Some(false),
        outline_color: Some(Color::RED),
        outline_width: Some(1.0),
        dimensions: Some(Cartesian3::new(2.0, 3.0, 4.0)),
        shadows: Some(ShadowMode::Disabled),
        distance_display_condition: Some(DistanceDisplayCondition::new(10.0, 100.0)),
        height_reference: None,
    };

    let box_graphics = BoxGraphics::from_values(values);

    assert_eq!(
        box_graphics
            .material
            .as_ref()
            .unwrap()
            .get_material_value(None)
            .unwrap(),
        MaterialValue::Color(Color::BLUE)
    );
    assert!(box_graphics.show.as_ref().unwrap().get_value(None).unwrap());
    assert!(!box_graphics.fill.as_ref().unwrap().get_value(None).unwrap());
    assert!(!box_graphics.outline.as_ref().unwrap().get_value(None).unwrap());
    assert_eq!(
        box_graphics
            .outline_color
            .as_ref()
            .unwrap()
            .get_value(None)
            .unwrap(),
        Color::RED
    );
    assert_eq!(
        box_graphics
            .outline_width
            .as_ref()
            .unwrap()
            .get_value(None)
            .unwrap(),
        1.0
    );
    assert_eq!(
        box_graphics
            .dimensions
            .as_ref()
            .unwrap()
            .get_value(None)
            .unwrap(),
        Cartesian3::new(2.0, 3.0, 4.0)
    );
    assert_eq!(
        box_graphics.shadows.as_ref().unwrap().get_value(None).unwrap(),
        ShadowMode::Disabled
    );
    assert_eq!(
        box_graphics
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
    let mut source = BoxGraphics::new();
    source.material = Some(Rc::new(ColorMaterialProperty::default()));
    source.show = Some(Rc::new(ConstantProperty::from_value(true)));
    source.fill = Some(Rc::new(ConstantProperty::from_value(true)));
    source.outline = Some(Rc::new(ConstantProperty::from_value(false)));
    source.outline_color = Some(Rc::new(ConstantProperty::from_value(Color::RED)));
    source.outline_width = Some(Rc::new(ConstantProperty::from_value(2.0)));
    source.dimensions = Some(Rc::new(ConstantProperty::from_value(Cartesian3::new(
        1.0, 1.0, 1.0,
    ))));
    source.shadows = Some(Rc::new(ConstantProperty::from_value(ShadowMode::Enabled)));
    source.distance_display_condition = Some(Rc::new(ConstantProperty::from_value(
        DistanceDisplayCondition::new(10.0, 100.0),
    )));

    let mut target = BoxGraphics::new();
    target.merge(&source);

    assert!(target.material.is_some());
    assert!(target.show.is_some());
    assert!(target.fill.is_some());
    assert!(target.outline.is_some());
    assert!(target.outline_color.is_some());
    assert!(target.outline_width.is_some());
    assert!(target.dimensions.is_some());
    assert!(target.shadows.is_some());
    assert!(target.distance_display_condition.is_some());

    assert_eq!(
        target.dimensions.as_ref().unwrap().get_value(None).unwrap(),
        Cartesian3::new(1.0, 1.0, 1.0)
    );
    assert_eq!(
        target.shadows.as_ref().unwrap().get_value(None).unwrap(),
        ShadowMode::Enabled
    );
}

#[test]
fn merge_does_not_assign_assigned_properties() {
    let mut source = BoxGraphics::new();
    source.show = Some(Rc::new(ConstantProperty::from_value(false)));
    source.shadows = Some(Rc::new(ConstantProperty::from_value(ShadowMode::Disabled)));

    let mut target = BoxGraphics::new();
    target.show = Some(Rc::new(ConstantProperty::from_value(true)));
    target.shadows = Some(Rc::new(ConstantProperty::from_value(ShadowMode::Enabled)));

    target.merge(&source);

    assert!(target.show.as_ref().unwrap().get_value(None).unwrap());
    assert_eq!(
        target.shadows.as_ref().unwrap().get_value(None).unwrap(),
        ShadowMode::Enabled
    );
}

#[test]
fn clone_works() {
    let mut source = BoxGraphics::new();
    source.show = Some(Rc::new(ConstantProperty::from_value(true)));
    source.dimensions = Some(Rc::new(ConstantProperty::from_value(Cartesian3::new(
        5.0, 6.0, 7.0,
    ))));

    let result = source.clone();
    assert!(result.show.as_ref().unwrap().get_value(None).unwrap());
    assert_eq!(
        result.dimensions.as_ref().unwrap().get_value(None).unwrap(),
        Cartesian3::new(5.0, 6.0, 7.0)
    );
}
