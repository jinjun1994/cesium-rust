//! Direct port and verification against CesiumJS `DataSources/BillboardGraphicsSpec.js` (Apache-2.0).

use cesium_math::{
    BillboardGraphics, BillboardGraphicsValues, BoundingRectangle, Cartesian2, Cartesian3, Color,
    ConstantProperty, DistanceDisplayCondition, HeightReference, HorizontalOrigin, NearFarScalar,
    SplitDirection, VerticalOrigin,
};
use std::rc::Rc;

#[test]
fn creates_expected_instance_from_raw_assignment_and_construction() {
    let values = BillboardGraphicsValues {
        image: Some("0".to_string()),
        rotation: Some(1.0),
        aligned_axis: Some(Cartesian3::UNIT_Z),
        color: Some(Color::RED),
        height_reference: Some(HeightReference::ClampToGround),
        horizontal_origin: Some(HorizontalOrigin::Left),
        vertical_origin: Some(VerticalOrigin::Bottom),
        eye_offset: Some(Cartesian3::new(5.0, 6.0, 7.0)),
        pixel_offset: Some(Cartesian2::new(8.0, 9.0)),
        scale: Some(10.0),
        show: Some(true),
        width: Some(11.0),
        height: Some(12.0),
        scale_by_distance: Some(NearFarScalar::new(13.0, 14.0, 15.0, 16.0)),
        translucency_by_distance: Some(NearFarScalar::new(17.0, 18.0, 19.0, 20.0)),
        pixel_offset_scale_by_distance: Some(NearFarScalar::new(21.0, 22.0, 23.0, 24.0)),
        size_in_meters: Some(true),
        distance_display_condition: Some(DistanceDisplayCondition::new(10.0, 100.0)),
        disable_depth_test_distance: Some(10.0),
        split_direction: Some(SplitDirection::Left),
        ..Default::default()
    };

    let billboard = BillboardGraphics::from_values(values);

    assert_eq!(
        billboard.image.as_ref().unwrap().get_value(None).unwrap(),
        "0"
    );
    assert_eq!(
        billboard.rotation.as_ref().unwrap().get_value(None).unwrap(),
        1.0
    );
    assert_eq!(
        billboard
            .aligned_axis
            .as_ref()
            .unwrap()
            .get_value(None)
            .unwrap(),
        Cartesian3::UNIT_Z
    );
    assert_eq!(
        billboard.color.as_ref().unwrap().get_value(None).unwrap(),
        Color::RED
    );
    assert_eq!(
        billboard
            .height_reference
            .as_ref()
            .unwrap()
            .get_value(None)
            .unwrap(),
        HeightReference::ClampToGround
    );
    assert_eq!(
        billboard
            .horizontal_origin
            .as_ref()
            .unwrap()
            .get_value(None)
            .unwrap(),
        HorizontalOrigin::Left
    );
    assert_eq!(
        billboard
            .vertical_origin
            .as_ref()
            .unwrap()
            .get_value(None)
            .unwrap(),
        VerticalOrigin::Bottom
    );
    assert_eq!(
        billboard.eye_offset.as_ref().unwrap().get_value(None).unwrap(),
        Cartesian3::new(5.0, 6.0, 7.0)
    );
    assert_eq!(
        billboard
            .pixel_offset
            .as_ref()
            .unwrap()
            .get_value(None)
            .unwrap(),
        Cartesian2::new(8.0, 9.0)
    );
    assert_eq!(
        billboard.scale.as_ref().unwrap().get_value(None).unwrap(),
        10.0
    );
    assert!(billboard.show.as_ref().unwrap().get_value(None).unwrap());
    assert_eq!(
        billboard.width.as_ref().unwrap().get_value(None).unwrap(),
        11.0
    );
    assert_eq!(
        billboard.height.as_ref().unwrap().get_value(None).unwrap(),
        12.0
    );
    assert_eq!(
        billboard
            .scale_by_distance
            .as_ref()
            .unwrap()
            .get_value(None)
            .unwrap(),
        NearFarScalar::new(13.0, 14.0, 15.0, 16.0)
    );
    assert_eq!(
        billboard
            .translucency_by_distance
            .as_ref()
            .unwrap()
            .get_value(None)
            .unwrap(),
        NearFarScalar::new(17.0, 18.0, 19.0, 20.0)
    );
    assert_eq!(
        billboard
            .pixel_offset_scale_by_distance
            .as_ref()
            .unwrap()
            .get_value(None)
            .unwrap(),
        NearFarScalar::new(21.0, 22.0, 23.0, 24.0)
    );
    assert!(billboard
        .size_in_meters
        .as_ref()
        .unwrap()
        .get_value(None)
        .unwrap());
    assert_eq!(
        billboard
            .distance_display_condition
            .as_ref()
            .unwrap()
            .get_value(None)
            .unwrap(),
        DistanceDisplayCondition::new(10.0, 100.0)
    );
    assert_eq!(
        billboard
            .disable_depth_test_distance
            .as_ref()
            .unwrap()
            .get_value(None)
            .unwrap(),
        10.0
    );
    assert_eq!(
        billboard
            .split_direction
            .as_ref()
            .unwrap()
            .get_value(None)
            .unwrap(),
        SplitDirection::Left
    );
}

#[test]
fn merge_assigns_unassigned_properties() {
    let mut source = BillboardGraphics::new();
    source.image = Some(Rc::new(ConstantProperty::from_value("image.png".to_string())));
    source.image_sub_region = Some(Rc::new(ConstantProperty::from_value(
        BoundingRectangle::new(0.0, 0.0, 10.0, 10.0),
    )));
    source.rotation = Some(Rc::new(ConstantProperty::from_value(5.0)));
    source.aligned_axis = Some(Rc::new(ConstantProperty::from_value(Cartesian3::ZERO)));
    source.color = Some(Rc::new(ConstantProperty::from_value(Color::BLACK)));
    source.height_reference = Some(Rc::new(ConstantProperty::from_value(
        HeightReference::ClampToGround,
    )));
    source.horizontal_origin = Some(Rc::new(ConstantProperty::from_value(HorizontalOrigin::Left)));
    source.vertical_origin = Some(Rc::new(ConstantProperty::from_value(VerticalOrigin::Bottom)));
    source.eye_offset = Some(Rc::new(ConstantProperty::from_value(Cartesian3::UNIT_Y)));
    source.pixel_offset = Some(Rc::new(ConstantProperty::from_value(Cartesian2::UNIT_X)));
    source.scale = Some(Rc::new(ConstantProperty::from_value(1.0)));
    source.show = Some(Rc::new(ConstantProperty::from_value(false)));
    source.width = Some(Rc::new(ConstantProperty::from_value(24.0)));
    source.height = Some(Rc::new(ConstantProperty::from_value(36.0)));
    source.scale_by_distance = Some(Rc::new(ConstantProperty::from_value(NearFarScalar::default())));
    source.translucency_by_distance =
        Some(Rc::new(ConstantProperty::from_value(NearFarScalar::default())));
    source.pixel_offset_scale_by_distance = Some(Rc::new(ConstantProperty::from_value(
        NearFarScalar::new(1.0, 0.0, 3.0e9, 0.0),
    )));
    source.size_in_meters = Some(Rc::new(ConstantProperty::from_value(true)));
    source.distance_display_condition = Some(Rc::new(ConstantProperty::from_value(
        DistanceDisplayCondition::new(10.0, 100.0),
    )));
    source.disable_depth_test_distance = Some(Rc::new(ConstantProperty::from_value(10.0)));
    source.split_direction = Some(Rc::new(ConstantProperty::from_value(SplitDirection::Left)));

    let mut target = BillboardGraphics::new();
    target.merge(&source);

    assert!(target.image.is_some());
    assert!(target.image_sub_region.is_some());
    assert!(target.rotation.is_some());
    assert!(target.aligned_axis.is_some());
    assert!(target.color.is_some());
    assert!(target.height_reference.is_some());
    assert!(target.horizontal_origin.is_some());
    assert!(target.vertical_origin.is_some());
    assert!(target.eye_offset.is_some());
    assert!(target.pixel_offset.is_some());
    assert!(target.scale.is_some());
    assert!(target.show.is_some());
    assert!(target.width.is_some());
    assert!(target.height.is_some());
    assert!(target.scale_by_distance.is_some());
    assert!(target.translucency_by_distance.is_some());
    assert!(target.pixel_offset_scale_by_distance.is_some());
    assert!(target.size_in_meters.is_some());
    assert!(target.distance_display_condition.is_some());
    assert!(target.disable_depth_test_distance.is_some());
    assert!(target.split_direction.is_some());

    assert_eq!(
        target.image.as_ref().unwrap().get_value(None).unwrap(),
        "image.png"
    );
    assert_eq!(
        target.height_reference.as_ref().unwrap().get_value(None).unwrap(),
        HeightReference::ClampToGround
    );
    assert_eq!(
        target.horizontal_origin.as_ref().unwrap().get_value(None).unwrap(),
        HorizontalOrigin::Left
    );
}

#[test]
fn merge_does_not_assign_assigned_properties() {
    let mut source = BillboardGraphics::new();
    source.image = Some(Rc::new(ConstantProperty::from_value("source.png".to_string())));
    source.scale = Some(Rc::new(ConstantProperty::from_value(1.0)));
    source.color = Some(Rc::new(ConstantProperty::from_value(Color::WHITE)));

    let mut target = BillboardGraphics::new();
    target.image = Some(Rc::new(ConstantProperty::from_value("target.png".to_string())));
    target.scale = Some(Rc::new(ConstantProperty::from_value(2.0)));
    target.color = Some(Rc::new(ConstantProperty::from_value(Color::RED)));

    target.merge(&source);

    assert_eq!(
        target.image.as_ref().unwrap().get_value(None).unwrap(),
        "target.png"
    );
    assert_eq!(target.scale.as_ref().unwrap().get_value(None).unwrap(), 2.0);
    assert_eq!(target.color.as_ref().unwrap().get_value(None).unwrap(), Color::RED);
}

#[test]
fn clone_works() {
    let mut source = BillboardGraphics::new();
    source.image = Some(Rc::new(ConstantProperty::from_value("test.png".to_string())));
    source.scale = Some(Rc::new(ConstantProperty::from_value(3.0)));
    source.horizontal_origin = Some(Rc::new(ConstantProperty::from_value(HorizontalOrigin::Right)));

    let result = source.clone();
    assert_eq!(
        result.image.as_ref().unwrap().get_value(None).unwrap(),
        "test.png"
    );
    assert_eq!(result.scale.as_ref().unwrap().get_value(None).unwrap(), 3.0);
    assert_eq!(
        result.horizontal_origin.as_ref().unwrap().get_value(None).unwrap(),
        HorizontalOrigin::Right
    );
}
