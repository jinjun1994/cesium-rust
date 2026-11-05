//! Direct port and verification against CesiumJS `DataSources/LabelGraphicsSpec.js` (Apache-2.0).

use cesium_math::{
    Cartesian2, Cartesian3, Color, ConstantProperty, DistanceDisplayCondition, HeightReference,
    HorizontalOrigin, LabelGraphics, LabelGraphicsValues, LabelStyle, NearFarScalar, VerticalOrigin,
};
use std::rc::Rc;

#[test]
fn creates_expected_instance_from_raw_assignment_and_construction() {
    let values = LabelGraphicsValues {
        text: Some("0".to_string()),
        font: Some("1".to_string()),
        style: Some(LabelStyle::Fill),
        fill_color: Some(Color::RED),
        outline_color: Some(Color::BLUE),
        outline_width: Some(2.0),
        horizontal_origin: Some(HorizontalOrigin::Left),
        vertical_origin: Some(VerticalOrigin::Bottom),
        eye_offset: Some(Cartesian3::new(3.0, 4.0, 5.0)),
        pixel_offset: Some(Cartesian2::new(6.0, 7.0)),
        scale: Some(8.0),
        show: Some(true),
        translucency_by_distance: Some(NearFarScalar::new(9.0, 10.0, 11.0, 12.0)),
        pixel_offset_scale_by_distance: Some(NearFarScalar::new(13.0, 14.0, 15.0, 16.0)),
        scale_by_distance: Some(NearFarScalar::new(17.0, 18.0, 19.0, 20.0)),
        distance_display_condition: Some(DistanceDisplayCondition::new(10.0, 100.0)),
        disable_depth_test_distance: Some(10.0),
        ..Default::default()
    };

    let label = LabelGraphics::from_values(values);

    assert_eq!(
        label.text.as_ref().unwrap().get_value(None).unwrap(),
        "0".to_string()
    );
    assert_eq!(
        label.font.as_ref().unwrap().get_value(None).unwrap(),
        "1".to_string()
    );
    assert_eq!(
        label.style.as_ref().unwrap().get_value(None).unwrap(),
        LabelStyle::Fill
    );
    assert_eq!(
        label.fill_color.as_ref().unwrap().get_value(None).unwrap(),
        Color::RED
    );
    assert_eq!(
        label
            .outline_color
            .as_ref()
            .unwrap()
            .get_value(None)
            .unwrap(),
        Color::BLUE
    );
    assert_eq!(
        label.outline_width.as_ref().unwrap().get_value(None).unwrap(),
        2.0
    );
    assert_eq!(
        label
            .horizontal_origin
            .as_ref()
            .unwrap()
            .get_value(None)
            .unwrap(),
        HorizontalOrigin::Left
    );
    assert_eq!(
        label
            .vertical_origin
            .as_ref()
            .unwrap()
            .get_value(None)
            .unwrap(),
        VerticalOrigin::Bottom
    );
    assert_eq!(
        label.eye_offset.as_ref().unwrap().get_value(None).unwrap(),
        Cartesian3::new(3.0, 4.0, 5.0)
    );
    assert_eq!(
        label
            .pixel_offset
            .as_ref()
            .unwrap()
            .get_value(None)
            .unwrap(),
        Cartesian2::new(6.0, 7.0)
    );
    assert_eq!(
        label.scale.as_ref().unwrap().get_value(None).unwrap(),
        8.0
    );
    assert!(label.show.as_ref().unwrap().get_value(None).unwrap());
    assert_eq!(
        label
            .translucency_by_distance
            .as_ref()
            .unwrap()
            .get_value(None)
            .unwrap(),
        NearFarScalar::new(9.0, 10.0, 11.0, 12.0)
    );
    assert_eq!(
        label
            .pixel_offset_scale_by_distance
            .as_ref()
            .unwrap()
            .get_value(None)
            .unwrap(),
        NearFarScalar::new(13.0, 14.0, 15.0, 16.0)
    );
    assert_eq!(
        label
            .scale_by_distance
            .as_ref()
            .unwrap()
            .get_value(None)
            .unwrap(),
        NearFarScalar::new(17.0, 18.0, 19.0, 20.0)
    );
    assert_eq!(
        label
            .distance_display_condition
            .as_ref()
            .unwrap()
            .get_value(None)
            .unwrap(),
        DistanceDisplayCondition::new(10.0, 100.0)
    );
    assert_eq!(
        label
            .disable_depth_test_distance
            .as_ref()
            .unwrap()
            .get_value(None)
            .unwrap(),
        10.0
    );
}

#[test]
fn merge_assigns_unassigned_properties() {
    let mut source = LabelGraphics::new();
    source.text = Some(Rc::new(ConstantProperty::from_value("text".to_string())));
    source.font = Some(Rc::new(ConstantProperty::from_value("10px".to_string())));
    source.style = Some(Rc::new(ConstantProperty::from_value(LabelStyle::Fill)));
    source.fill_color = Some(Rc::new(ConstantProperty::from_value(Color::RED)));
    source.outline_color = Some(Rc::new(ConstantProperty::from_value(Color::BLUE)));
    source.outline_width = Some(Rc::new(ConstantProperty::from_value(1.0)));
    source.horizontal_origin = Some(Rc::new(ConstantProperty::from_value(
        HorizontalOrigin::Left,
    )));
    source.vertical_origin = Some(Rc::new(ConstantProperty::from_value(
        VerticalOrigin::Bottom,
    )));
    source.eye_offset = Some(Rc::new(ConstantProperty::from_value(Cartesian3::ZERO)));
    source.pixel_offset = Some(Rc::new(ConstantProperty::from_value(Cartesian2::ZERO)));
    source.scale = Some(Rc::new(ConstantProperty::from_value(1.0)));
    source.show = Some(Rc::new(ConstantProperty::from_value(true)));
    source.show_background = Some(Rc::new(ConstantProperty::from_value(true)));
    source.background_color = Some(Rc::new(ConstantProperty::from_value(Color::BLACK)));
    source.background_padding = Some(Rc::new(ConstantProperty::from_value(Cartesian2::new(
        1.0, 2.0,
    ))));
    source.height_reference = Some(Rc::new(ConstantProperty::from_value(
        HeightReference::ClampToGround,
    )));
    source.translucency_by_distance = Some(Rc::new(ConstantProperty::from_value(
        NearFarScalar::new(1.0, 1.0, 2.0, 2.0),
    )));
    source.pixel_offset_scale_by_distance = Some(Rc::new(ConstantProperty::from_value(
        NearFarScalar::new(1.0, 1.0, 2.0, 2.0),
    )));
    source.scale_by_distance = Some(Rc::new(ConstantProperty::from_value(
        NearFarScalar::new(1.0, 1.0, 2.0, 2.0),
    )));
    source.distance_display_condition = Some(Rc::new(ConstantProperty::from_value(
        DistanceDisplayCondition::new(10.0, 100.0),
    )));
    source.disable_depth_test_distance = Some(Rc::new(ConstantProperty::from_value(10.0)));

    let mut target = LabelGraphics::new();
    target.merge(&source);

    assert!(target.text.is_some());
    assert!(target.font.is_some());
    assert!(target.style.is_some());
    assert!(target.fill_color.is_some());
    assert!(target.outline_color.is_some());
    assert!(target.outline_width.is_some());
    assert!(target.horizontal_origin.is_some());
    assert!(target.vertical_origin.is_some());
    assert!(target.eye_offset.is_some());
    assert!(target.pixel_offset.is_some());
    assert!(target.scale.is_some());
    assert!(target.show.is_some());
    assert!(target.show_background.is_some());
    assert!(target.background_color.is_some());
    assert!(target.background_padding.is_some());
    assert!(target.height_reference.is_some());
    assert!(target.translucency_by_distance.is_some());
    assert!(target.pixel_offset_scale_by_distance.is_some());
    assert!(target.scale_by_distance.is_some());
    assert!(target.distance_display_condition.is_some());
    assert!(target.disable_depth_test_distance.is_some());
}

#[test]
fn merge_does_not_assign_assigned_properties() {
    let mut source = LabelGraphics::new();
    source.show = Some(Rc::new(ConstantProperty::from_value(false)));

    let mut target = LabelGraphics::new();
    target.show = Some(Rc::new(ConstantProperty::from_value(true)));
    target.merge(&source);

    assert!(target.show.as_ref().unwrap().get_value(None).unwrap());
}

#[test]
fn clone_works() {
    let mut source = LabelGraphics::new();
    source.text = Some(Rc::new(ConstantProperty::from_value("text".to_string())));
    source.font = Some(Rc::new(ConstantProperty::from_value("12px".to_string())));
    source.style = Some(Rc::new(ConstantProperty::from_value(
        LabelStyle::FillAndOutline,
    )));
    source.scale = Some(Rc::new(ConstantProperty::from_value(2.0)));
    source.show = Some(Rc::new(ConstantProperty::from_value(true)));

    let cloned = source.clone();
    assert_eq!(
        cloned.text.as_ref().unwrap().get_value(None).unwrap(),
        "text".to_string()
    );
    assert_eq!(
        cloned.font.as_ref().unwrap().get_value(None).unwrap(),
        "12px".to_string()
    );
    assert_eq!(
        cloned.style.as_ref().unwrap().get_value(None).unwrap(),
        LabelStyle::FillAndOutline
    );
    assert_eq!(
        cloned.scale.as_ref().unwrap().get_value(None).unwrap(),
        2.0
    );
    assert!(cloned.show.as_ref().unwrap().get_value(None).unwrap());
}
