use cesium_math::{
    BoundingRectangle, Cartesian2, Cartesian3, Color, DistanceDisplayCondition, Ellipsoid,
    NearFarScalar, PointPrimitiveCollection, PointPrimitiveOptions, SplitDirection,
};

#[test]
fn test_constructs_a_default_point_primitive() {
    let mut point_primitives = PointPrimitiveCollection::new();
    let idx = point_primitives.add(None).unwrap();
    let p = point_primitives.get(Some(idx)).unwrap();

    assert!(p.show);
    assert_eq!(p.position, Cartesian3::ZERO);
    assert_eq!(p.pixel_size, 10.0);
    assert_eq!(p.color, Color::new(1.0, 1.0, 1.0, 1.0));
    assert_eq!(p.outline_color, Color::new(0.0, 0.0, 0.0, 0.0));
    assert_eq!(p.outline_width, 0.0);
    assert!(p.scale_by_distance.is_none());
    assert!(p.translucency_by_distance.is_none());
    assert!(p.distance_display_condition.is_none());
    assert_eq!(p.disable_depth_test_distance, 0.0);
    assert!(p.id.is_none());
    assert_eq!(p.split_direction, SplitDirection::None);
}

#[test]
fn test_can_add_and_remove_before_render() {
    let mut point_primitives = PointPrimitiveCollection::new();
    let idx = point_primitives.add(None).unwrap();
    assert_eq!(point_primitives.len(), 1);
    let removed = point_primitives.remove_at(idx);
    assert!(removed);
    assert_eq!(point_primitives.len(), 0);
}

#[test]
fn test_explicitly_constructs_a_point_primitive() {
    let mut point_primitives = PointPrimitiveCollection::new();
    let options = PointPrimitiveOptions {
        show: Some(false),
        position: Some(Cartesian3::new(1.0, 2.0, 3.0)),
        pixel_size: Some(2.0),
        color: Some(Color::new(0.1, 0.2, 0.3, 0.4)),
        outline_color: Some(Color::new(0.5, 0.6, 0.7, 0.8)),
        outline_width: Some(4.0),
        scale_by_distance: Some(NearFarScalar::new(1.0, 3.0, 1.0e6, 0.0)),
        translucency_by_distance: Some(NearFarScalar::new(1.0, 1.0, 1.0e6, 0.0)),
        distance_display_condition: Some(DistanceDisplayCondition::new(10.0, 100.0)),
        disable_depth_test_distance: Some(10.0),
        id: Some("custom_id".to_string()),
        split_direction: Some(SplitDirection::Left),
    };

    let idx = point_primitives.add(Some(options)).unwrap();
    let p = point_primitives.get(Some(idx)).unwrap();

    assert!(!p.show);
    assert_eq!(p.position, Cartesian3::new(1.0, 2.0, 3.0));
    assert_eq!(p.pixel_size, 2.0);
    assert_eq!(p.color, Color::new(0.1, 0.2, 0.3, 0.4));
    assert_eq!(p.outline_color, Color::new(0.5, 0.6, 0.7, 0.8));
    assert_eq!(p.outline_width, 4.0);
    assert_eq!(
        p.scale_by_distance,
        Some(NearFarScalar::new(1.0, 3.0, 1.0e6, 0.0))
    );
    assert_eq!(
        p.translucency_by_distance,
        Some(NearFarScalar::new(1.0, 1.0, 1.0e6, 0.0))
    );
    assert_eq!(
        p.distance_display_condition,
        Some(DistanceDisplayCondition::new(10.0, 100.0))
    );
    assert_eq!(p.disable_depth_test_distance, 10.0);
    assert_eq!(p.id.as_deref(), Some("custom_id"));
    assert_eq!(p.split_direction, SplitDirection::Left);
}

#[test]
fn test_sets_point_primitive_properties_and_validations() {
    let mut point_primitives = PointPrimitiveCollection::new();
    let idx = point_primitives.add(None).unwrap();

    {
        let p = point_primitives.get_mut(Some(idx)).unwrap();
        p.show = false;
        p.position = Cartesian3::new(10.0, 20.0, 30.0);
        p.pixel_size = 15.0;
        p.color = Color::RED;
        p.outline_color = Color::BLUE;
        p.outline_width = 3.0;

        assert!(p.set_scale_by_distance(Some(NearFarScalar::new(1.0, 1.0, 100.0, 0.0))).is_ok());
        // scaleByDistance validation (near >= far throws)
        assert!(p.set_scale_by_distance(Some(NearFarScalar::new(10.0, 1.0, 10.0, 0.0))).is_err());
        assert!(p.set_scale_by_distance(Some(NearFarScalar::new(100.0, 1.0, 10.0, 0.0))).is_err());

        // translucencyByDistance validation
        assert!(p.set_translucency_by_distance(Some(NearFarScalar::new(10.0, 1.0, 10.0, 0.0))).is_err());
        assert!(p.set_translucency_by_distance(Some(NearFarScalar::new(100.0, 1.0, 10.0, 0.0))).is_err());

        // distanceDisplayCondition validation
        assert!(p.set_distance_display_condition(Some(DistanceDisplayCondition::new(50.0, 50.0))).is_err());
        assert!(p.set_distance_display_condition(Some(DistanceDisplayCondition::new(60.0, 50.0))).is_err());
        assert!(p.set_distance_display_condition(Some(DistanceDisplayCondition::new(10.0, 50.0))).is_ok());

        // disableDepthTestDistance validation
        assert!(p.set_disable_depth_test_distance(-1.0).is_err());
        assert!(p.set_disable_depth_test_distance(100.0).is_ok());
    }

    let p = point_primitives.get(Some(idx)).unwrap();
    assert!(!p.show);
    assert_eq!(p.position, Cartesian3::new(10.0, 20.0, 30.0));
    assert_eq!(p.pixel_size, 15.0);
    assert_eq!(p.color, Color::RED);
    assert_eq!(p.outline_color, Color::BLUE);
    assert_eq!(p.outline_width, 3.0);
    assert_eq!(p.disable_depth_test_distance, 100.0);
}

#[test]
fn test_collection_add_remove_and_contains() {
    let mut point_primitives = PointPrimitiveCollection::new();
    assert_eq!(point_primitives.len(), 0);
    assert!(point_primitives.is_empty());

    let i1 = point_primitives.add(None).unwrap();
    let p1_clone = point_primitives.get(Some(i1)).unwrap().clone();
    let i2 = point_primitives.add(None).unwrap();
    let p2_clone = point_primitives.get(Some(i2)).unwrap().clone();
    let i3 = point_primitives.add(None).unwrap();
    let p3_clone = point_primitives.get(Some(i3)).unwrap().clone();

    assert_eq!(point_primitives.len(), 3);
    assert!(point_primitives.contains_point(Some(&p1_clone)));
    assert!(point_primitives.contains_point(Some(&p2_clone)));
    assert!(point_primitives.contains_point(Some(&p3_clone)));

    // Remove first
    assert!(point_primitives.remove_point(Some(&p1_clone)));
    assert_eq!(point_primitives.len(), 2);
    assert!(!point_primitives.contains_point(Some(&p1_clone)));

    // Remove same twice returns false
    assert!(!point_primitives.remove_point(Some(&p1_clone)));

    // Remove last
    assert!(point_primitives.remove_point(Some(&p3_clone)));
    assert_eq!(point_primitives.len(), 1);

    // Remove undefined returns false
    assert!(!point_primitives.remove_point(None));
    assert!(!point_primitives.contains_point(None));

    // Remove all
    point_primitives.remove_all();
    assert_eq!(point_primitives.len(), 0);
    assert!(!point_primitives.contains_point(Some(&p2_clone)));
}

#[test]
fn test_get_and_validations() {
    let mut point_primitives = PointPrimitiveCollection::new();
    let idx = point_primitives.add(None).unwrap();

    // Access without index throws
    assert!(point_primitives.get(None).is_err());

    // Access out of bounds throws
    assert!(point_primitives.get(Some(5)).is_err());

    // Access valid index
    let p = point_primitives.get(Some(idx)).unwrap();
    assert_eq!(p.pixel_size, 10.0);
}

#[test]
fn test_screen_space_position_and_bounding_box() {
    let mut point_primitives = PointPrimitiveCollection::new();
    let idx = point_primitives
        .add(Some(PointPrimitiveOptions {
            position: Some(Cartesian3::new(10.0, 5.0, 0.0)),
            pixel_size: Some(20.0),
            outline_width: Some(2.0),
            ..Default::default()
        }))
        .unwrap();

    let p = point_primitives.get(Some(idx)).unwrap();
    let camera_pos = Cartesian3::new(0.0, 0.0, 0.0);
    let screen_pos = p
        .compute_screen_space_position(Some(camera_pos), 800.0, 600.0, None)
        .expect("screen space position");
    assert_eq!(screen_pos, Cartesian2::new(410.0, 295.0));

    // With result scratch
    let mut result = Cartesian2::ZERO;
    let _ = p.compute_screen_space_position(Some(camera_pos), 800.0, 600.0, Some(&mut result));
    assert_eq!(result, Cartesian2::new(410.0, 295.0));

    // Throws without camera/scene
    assert!(p
        .compute_screen_space_position(None, 800.0, 600.0, None)
        .is_err());

    // Compute bounding box (total size = 20 + 2*2 = 24)
    let bbox = p.compute_screen_space_bounding_box(screen_pos, None);
    assert_eq!(
        bbox,
        BoundingRectangle::new(410.0 - 12.0, 295.0 - 12.0, 24.0, 24.0)
    );

    // Throws when not in a collection
    let mut orphan = p.clone();
    orphan.collection_id = None;
    assert!(orphan
        .compute_screen_space_position(Some(camera_pos), 800.0, 600.0, None)
        .is_err());
}

#[test]
fn test_bounding_spheres_3d_2d_and_columbus_view() {
    let mut point_primitives = PointPrimitiveCollection::new();
    let ellipsoid = Ellipsoid::WGS84;

    let p1 = Cartesian3::from_degrees(-50.0, -50.0, Some(0.0), Some(&ellipsoid)).unwrap();
    let p2 = Cartesian3::from_degrees(-50.0, 50.0, Some(0.0), Some(&ellipsoid)).unwrap();

    point_primitives
        .add(Some(PointPrimitiveOptions {
            position: Some(p1),
            ..Default::default()
        }))
        .unwrap();

    point_primitives
        .add(Some(PointPrimitiveOptions {
            position: Some(p2),
            ..Default::default()
        }))
        .unwrap();

    // 3D bounding sphere
    let bs3d = point_primitives.compute_bounding_sphere_3d().expect("3D sphere");
    assert!(bs3d.radius > 0.0);

    // 2D bounding sphere
    let bs2d = point_primitives
        .compute_bounding_sphere_2d(&ellipsoid)
        .expect("2D sphere");
    assert_eq!(bs2d.center.x, 0.0);
    assert!(bs2d.radius > 0.0);

    // Columbus view bounding sphere
    let bscv = point_primitives
        .compute_bounding_sphere_columbus_view(&ellipsoid)
        .expect("CV sphere");
    assert!(bscv.radius > 0.0);

    // Destruction
    point_primitives.destroy();
    assert!(point_primitives.is_destroyed());
    assert!(point_primitives.is_empty());
}
