//! Direct port and verification against CesiumJS `DataSources/WallGraphicsSpec.js` (Apache-2.0).

use cesium_math::{
    Cartesian3, Color, ColorMaterialProperty, ConstantProperty, DistanceDisplayCondition,
    MaterialValue, ShadowMode, WallGraphics, WallGraphicsValues,
};
use std::rc::Rc;

#[test]
fn creates_expected_instance_from_raw_assignment_and_construction() {
    let positions = vec![
        Cartesian3::new(1.0, 2.0, 3.0),
        Cartesian3::new(4.0, 5.0, 6.0),
    ];
    let min_heights = vec![3.0, 4.0, 5.0];
    let max_heights = vec![6.0, 7.0, 8.0];
    let values = WallGraphicsValues {
        material: Some(Color::BLUE),
        positions: Some(positions.clone()),
        show: Some(true),
        granularity: Some(1.0),
        fill: Some(false),
        outline: Some(false),
        outline_color: Some(Color::RED),
        outline_width: Some(2.0),
        minimum_heights: Some(min_heights.clone()),
        maximum_heights: Some(max_heights.clone()),
        shadows: Some(ShadowMode::Disabled),
        distance_display_condition: Some(DistanceDisplayCondition::new(10.0, 100.0)),
    };

    let wall = WallGraphics::from_values(values);

    assert_eq!(
        wall.material
            .as_ref()
            .unwrap()
            .get_material_value(None)
            .unwrap(),
        MaterialValue::Color(Color::BLUE)
    );
    assert_eq!(
        wall.positions.as_ref().unwrap().get_value(None).unwrap(),
        positions
    );
    assert!(wall.show.as_ref().unwrap().get_value(None).unwrap());
    assert_eq!(
        wall.granularity.as_ref().unwrap().get_value(None).unwrap(),
        1.0
    );
    assert!(!wall.fill.as_ref().unwrap().get_value(None).unwrap());
    assert!(!wall.outline.as_ref().unwrap().get_value(None).unwrap());
    assert_eq!(
        wall.outline_color.as_ref().unwrap().get_value(None).unwrap(),
        Color::RED
    );
    assert_eq!(
        wall.outline_width.as_ref().unwrap().get_value(None).unwrap(),
        2.0
    );
    assert_eq!(
        wall.minimum_heights
            .as_ref()
            .unwrap()
            .get_value(None)
            .unwrap(),
        min_heights
    );
    assert_eq!(
        wall.maximum_heights
            .as_ref()
            .unwrap()
            .get_value(None)
            .unwrap(),
        max_heights
    );
    assert_eq!(
        wall.shadows.as_ref().unwrap().get_value(None).unwrap(),
        ShadowMode::Disabled
    );
    assert_eq!(
        wall.distance_display_condition
            .as_ref()
            .unwrap()
            .get_value(None)
            .unwrap(),
        DistanceDisplayCondition::new(10.0, 100.0)
    );
}

#[test]
fn merge_assigns_unassigned_properties() {
    let mut source = WallGraphics::new();
    source.material = Some(Rc::new(ColorMaterialProperty::default()));
    source.positions = Some(Rc::new(ConstantProperty::from_value(vec![
        Cartesian3::ZERO,
    ])));
    source.show = Some(Rc::new(ConstantProperty::from_value(true)));
    source.granularity = Some(Rc::new(ConstantProperty::from_value(1.0)));
    source.fill = Some(Rc::new(ConstantProperty::from_value(true)));
    source.outline = Some(Rc::new(ConstantProperty::from_value(false)));
    source.outline_color = Some(Rc::new(ConstantProperty::from_value(Color::RED)));
    source.outline_width = Some(Rc::new(ConstantProperty::from_value(2.0)));
    source.minimum_heights = Some(Rc::new(ConstantProperty::from_value(vec![1.0, 2.0])));
    source.maximum_heights = Some(Rc::new(ConstantProperty::from_value(vec![3.0, 4.0])));
    source.shadows = Some(Rc::new(ConstantProperty::from_value(ShadowMode::Enabled)));
    source.distance_display_condition = Some(Rc::new(ConstantProperty::from_value(
        DistanceDisplayCondition::new(10.0, 100.0),
    )));

    let mut target = WallGraphics::new();
    target.merge(&source);

    assert!(target.material.is_some());
    assert!(target.positions.is_some());
    assert!(target.show.is_some());
    assert!(target.granularity.is_some());
    assert!(target.fill.is_some());
    assert!(target.outline.is_some());
    assert!(target.outline_color.is_some());
    assert!(target.outline_width.is_some());
    assert!(target.minimum_heights.is_some());
    assert!(target.maximum_heights.is_some());
    assert!(target.shadows.is_some());
    assert!(target.distance_display_condition.is_some());
}

#[test]
fn merge_does_not_assign_assigned_properties() {
    let mut source = WallGraphics::new();
    source.show = Some(Rc::new(ConstantProperty::from_value(false)));

    let mut target = WallGraphics::new();
    target.show = Some(Rc::new(ConstantProperty::from_value(true)));
    target.merge(&source);

    assert!(target.show.as_ref().unwrap().get_value(None).unwrap());
}

#[test]
fn clone_works() {
    let mut source = WallGraphics::new();
    source.material = Some(Rc::new(ColorMaterialProperty::default()));
    source.positions = Some(Rc::new(ConstantProperty::from_value(vec![
        Cartesian3::new(1.0, 1.0, 1.0),
    ])));
    source.show = Some(Rc::new(ConstantProperty::from_value(true)));
    source.granularity = Some(Rc::new(ConstantProperty::from_value(0.1)));
    source.fill = Some(Rc::new(ConstantProperty::from_value(true)));
    source.outline = Some(Rc::new(ConstantProperty::from_value(false)));
    source.outline_color = Some(Rc::new(ConstantProperty::from_value(Color::RED)));
    source.outline_width = Some(Rc::new(ConstantProperty::from_value(3.0)));
    source.minimum_heights = Some(Rc::new(ConstantProperty::from_value(vec![10.0])));
    source.maximum_heights = Some(Rc::new(ConstantProperty::from_value(vec![20.0])));
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
            .minimum_heights
            .as_ref()
            .unwrap()
            .get_value(None)
            .unwrap(),
        vec![10.0]
    );
    assert_eq!(
        cloned
            .maximum_heights
            .as_ref()
            .unwrap()
            .get_value(None)
            .unwrap(),
        vec![20.0]
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
