//! Direct port and verification against CesiumJS `DataSources/CylinderGraphicsSpec.js` (Apache-2.0).

use cesium_math::{
    Color, ColorMaterialProperty, ConstantProperty, CylinderGraphics, CylinderGraphicsValues,
    DistanceDisplayCondition, MaterialValue, ShadowMode,
};
use std::rc::Rc;

#[test]
fn creates_expected_instance_from_raw_assignment_and_construction() {
    let values = CylinderGraphicsValues {
        material: Some(Color::BLUE),
        show: Some(true),
        length: Some(1.0),
        top_radius: Some(2.0),
        bottom_radius: Some(3.0),
        number_of_vertical_lines: Some(4),
        slices: Some(5),
        fill: Some(false),
        outline: Some(false),
        outline_color: Some(Color::RED),
        outline_width: Some(6.0),
        shadows: Some(ShadowMode::Disabled),
        distance_display_condition: Some(DistanceDisplayCondition::new(10.0, 100.0)),
        ..Default::default()
    };

    let cylinder = CylinderGraphics::from_values(values);

    assert_eq!(
        cylinder
            .material
            .as_ref()
            .unwrap()
            .get_material_value(None)
            .unwrap(),
        MaterialValue::Color(Color::BLUE)
    );
    assert!(cylinder.show.as_ref().unwrap().get_value(None).unwrap());
    assert_eq!(
        cylinder.length.as_ref().unwrap().get_value(None).unwrap(),
        1.0
    );
    assert_eq!(
        cylinder
            .top_radius
            .as_ref()
            .unwrap()
            .get_value(None)
            .unwrap(),
        2.0
    );
    assert_eq!(
        cylinder
            .bottom_radius
            .as_ref()
            .unwrap()
            .get_value(None)
            .unwrap(),
        3.0
    );
    assert_eq!(
        cylinder
            .number_of_vertical_lines
            .as_ref()
            .unwrap()
            .get_value(None)
            .unwrap(),
        4
    );
    assert_eq!(
        cylinder.slices.as_ref().unwrap().get_value(None).unwrap(),
        5
    );
    assert!(!cylinder.fill.as_ref().unwrap().get_value(None).unwrap());
    assert!(!cylinder.outline.as_ref().unwrap().get_value(None).unwrap());
    assert_eq!(
        cylinder
            .outline_color
            .as_ref()
            .unwrap()
            .get_value(None)
            .unwrap(),
        Color::RED
    );
    assert_eq!(
        cylinder
            .outline_width
            .as_ref()
            .unwrap()
            .get_value(None)
            .unwrap(),
        6.0
    );
    assert_eq!(
        cylinder.shadows.as_ref().unwrap().get_value(None).unwrap(),
        ShadowMode::Disabled
    );
    assert_eq!(
        cylinder
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
    let mut source = CylinderGraphics::new();
    source.material = Some(Rc::new(ColorMaterialProperty::default()));
    source.length = Some(Rc::new(ConstantProperty::from_value(1.0)));
    source.top_radius = Some(Rc::new(ConstantProperty::from_value(2.0)));
    source.bottom_radius = Some(Rc::new(ConstantProperty::from_value(3.0)));
    source.number_of_vertical_lines = Some(Rc::new(ConstantProperty::from_value(4)));
    source.slices = Some(Rc::new(ConstantProperty::from_value(5)));
    source.fill = Some(Rc::new(ConstantProperty::from_value(true)));
    source.outline = Some(Rc::new(ConstantProperty::from_value(false)));
    source.outline_color = Some(Rc::new(ConstantProperty::from_value(Color::RED)));
    source.outline_width = Some(Rc::new(ConstantProperty::from_value(6.0)));
    source.shadows = Some(Rc::new(ConstantProperty::from_value(ShadowMode::Enabled)));
    source.distance_display_condition = Some(Rc::new(ConstantProperty::from_value(
        DistanceDisplayCondition::new(10.0, 100.0),
    )));

    let mut target = CylinderGraphics::new();
    target.merge(&source);

    assert!(target.material.is_some());
    assert!(target.length.is_some());
    assert!(target.top_radius.is_some());
    assert!(target.bottom_radius.is_some());
    assert!(target.number_of_vertical_lines.is_some());
    assert!(target.slices.is_some());
    assert!(target.fill.is_some());
    assert!(target.outline.is_some());
    assert!(target.outline_color.is_some());
    assert!(target.outline_width.is_some());
    assert!(target.shadows.is_some());
    assert!(target.distance_display_condition.is_some());
}

#[test]
fn merge_does_not_assign_assigned_properties() {
    let mut source = CylinderGraphics::new();
    source.show = Some(Rc::new(ConstantProperty::from_value(false)));

    let mut target = CylinderGraphics::new();
    target.show = Some(Rc::new(ConstantProperty::from_value(true)));
    target.merge(&source);

    assert!(target.show.as_ref().unwrap().get_value(None).unwrap());
}

#[test]
fn clone_works() {
    let mut source = CylinderGraphics::new();
    source.material = Some(Rc::new(ColorMaterialProperty::default()));
    source.length = Some(Rc::new(ConstantProperty::from_value(10.0)));
    source.top_radius = Some(Rc::new(ConstantProperty::from_value(2.0)));
    source.bottom_radius = Some(Rc::new(ConstantProperty::from_value(4.0)));
    source.show = Some(Rc::new(ConstantProperty::from_value(true)));
    source.number_of_vertical_lines = Some(Rc::new(ConstantProperty::from_value(8)));
    source.slices = Some(Rc::new(ConstantProperty::from_value(16)));
    source.fill = Some(Rc::new(ConstantProperty::from_value(true)));
    source.outline = Some(Rc::new(ConstantProperty::from_value(false)));
    source.outline_color = Some(Rc::new(ConstantProperty::from_value(Color::RED)));
    source.outline_width = Some(Rc::new(ConstantProperty::from_value(3.0)));
    source.shadows = Some(Rc::new(ConstantProperty::from_value(ShadowMode::Enabled)));
    source.distance_display_condition = Some(Rc::new(ConstantProperty::from_value(
        DistanceDisplayCondition::new(10.0, 100.0),
    )));

    let cloned = source.clone();
    assert!(cloned.material.is_some());
    assert_eq!(
        cloned.length.as_ref().unwrap().get_value(None).unwrap(),
        10.0
    );
    assert_eq!(
        cloned.top_radius.as_ref().unwrap().get_value(None).unwrap(),
        2.0
    );
    assert_eq!(
        cloned
            .bottom_radius
            .as_ref()
            .unwrap()
            .get_value(None)
            .unwrap(),
        4.0
    );
    assert!(cloned.show.as_ref().unwrap().get_value(None).unwrap());
    assert_eq!(
        cloned
            .number_of_vertical_lines
            .as_ref()
            .unwrap()
            .get_value(None)
            .unwrap(),
        8
    );
    assert_eq!(
        cloned.slices.as_ref().unwrap().get_value(None).unwrap(),
        16
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
}
