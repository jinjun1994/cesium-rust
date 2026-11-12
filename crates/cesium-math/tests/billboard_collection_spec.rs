use cesium_math::{
    Billboard, BillboardCollection, Cartesian2, Cartesian3, Color, DistanceDisplayCondition,
    HeightReference, HorizontalOrigin, Matrix4, NearFarScalar, SplitDirection, VerticalOrigin,
};

#[test]
fn test_default_and_explicit_billboard() {
    let b = Billboard::default();
    assert!(b.show);
    assert_eq!(b.position, Cartesian3::ZERO);
    assert_eq!(b.color, Color::WHITE);
    assert_eq!(b.eye_offset, Cartesian3::ZERO);
    assert_eq!(b.pixel_offset, Cartesian2::ZERO);
    assert_eq!(b.scale, 1.0);
    assert_eq!(b.rotation, 0.0);
    assert_eq!(b.aligned_axis, Cartesian3::ZERO);
    assert_eq!(b.horizontal_origin, HorizontalOrigin::Center);
    assert_eq!(b.vertical_origin, VerticalOrigin::Center);
    assert_eq!(b.height_reference, HeightReference::None);
    assert_eq!(b.split_direction, SplitDirection::None);
    assert!(!b.size_in_meters);
    assert!(b.scale_by_distance.is_none());
    assert!(b.translucency_by_distance.is_none());
    assert!(b.pixel_offset_scale_by_distance.is_none());
    assert!(b.distance_display_condition.is_none());
    assert!(b.disable_depth_test_distance.is_none());
    assert!(b.ready);

    let custom = Billboard {
        show: false,
        position: Cartesian3::new(1.0, 2.0, 3.0),
        color: Color::RED,
        scale: 2.5,
        rotation: 0.5,
        horizontal_origin: HorizontalOrigin::Left,
        vertical_origin: VerticalOrigin::Bottom,
        size_in_meters: true,
        width: Some(64.0),
        height: Some(32.0),
        image: Some("icon.png".to_string()),
        ..Default::default()
    };
    assert!(!custom.show);
    assert_eq!(custom.width, Some(64.0));
    assert_eq!(custom.height, Some(32.0));
    assert_eq!(custom.image.as_deref(), Some("icon.png"));
}

#[test]
fn test_billboard_validation_errors() {
    let mut col = BillboardCollection::new();

    // scaleByDistance near == far and near > far
    let b_scale_eq = Billboard {
        scale_by_distance: Some(NearFarScalar::new(10.0, 1.0, 10.0, 0.5)),
        ..Default::default()
    };
    assert!(col.add_checked(b_scale_eq).is_err());

    let b_scale_gt = Billboard {
        scale_by_distance: Some(NearFarScalar::new(20.0, 1.0, 10.0, 0.5)),
        ..Default::default()
    };
    assert!(col.add_checked(b_scale_gt).is_err());

    // translucencyByDistance
    let b_trans = Billboard {
        translucency_by_distance: Some(NearFarScalar::new(10.0, 1.0, 10.0, 0.0)),
        ..Default::default()
    };
    assert!(col.add_checked(b_trans).is_err());

    // pixelOffsetScaleByDistance
    let b_pix = Billboard {
        pixel_offset_scale_by_distance: Some(NearFarScalar::new(50.0, 1.0, 10.0, 0.0)),
        ..Default::default()
    };
    assert!(col.add_checked(b_pix).is_err());

    // distanceDisplayCondition
    let b_ddc = Billboard {
        distance_display_condition: Some(DistanceDisplayCondition::new(100.0, 100.0)),
        ..Default::default()
    };
    assert!(col.add_checked(b_ddc).is_err());

    // disableDepthTestDistance < 0
    let b_depth = Billboard {
        disable_depth_test_distance: Some(-1.0),
        ..Default::default()
    };
    assert!(col.add_checked(b_depth).is_err());

    // valid billboard succeeds
    let valid = Billboard {
        scale_by_distance: Some(NearFarScalar::new(10.0, 1.0, 100.0, 0.5)),
        distance_display_condition: Some(DistanceDisplayCondition::new(0.0, 1000.0)),
        disable_depth_test_distance: Some(0.0),
        ..Default::default()
    };
    assert_eq!(col.add_checked(valid).unwrap(), 0);
}

#[test]
fn test_collection_add_remove_contains_and_destroy() {
    let mut col = BillboardCollection::new();
    assert_eq!(col.len(), 0);
    assert!(col.is_empty());
    assert!(col.show);
    assert_eq!(col.model_matrix, Matrix4::IDENTITY);
    assert!(col.destroy_texture_atlas);
    assert!(!col.is_destroyed());

    let b1 = Billboard {
        id: Some("b1".to_string()),
        ..Default::default()
    };
    let b2 = Billboard {
        id: Some("b2".to_string()),
        ..Default::default()
    };
    let b3 = Billboard {
        id: Some("b3".to_string()),
        ..Default::default()
    };

    col.add(b1.clone());
    col.add(b2.clone());
    assert_eq!(col.len(), 2);
    assert!(col.contains(&b1));
    assert!(col.contains(&b2));
    assert!(!col.contains(&b3));

    // Remove first
    assert!(col.remove_billboard(&b1));
    assert!(!col.contains(&b1));
    assert_eq!(col.len(), 1);
    // Removing twice returns false
    assert!(!col.remove_billboard(&b1));

    // Remove last
    assert!(col.remove_billboard(&b2));
    assert!(col.is_empty());

    col.add(b1);
    col.add(b2);
    col.remove_all();
    assert_eq!(col.len(), 0);

    col.destroy();
    assert!(col.is_destroyed());
}
