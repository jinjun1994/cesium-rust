use cesium_math::{
    Cartesian3, Cartographic, Color, Ellipsoid, GeoRectangle, GeographicProjection,
    GeographicTilingScheme, Plane, PlaneIntersection, SceneMode, EPSILON1, EPSILON3, EPSILON6,
    EPSILON8,
};
use cesium_renderer::{
    TileBoundingRegion, TileBoundingRegionCamera, TileBoundingRegionFrameState,
    TileBoundingRegionOptions,
};

#[test]
fn test_can_be_instantiated_with_rectangle_and_heights() {
    let rectangle = GeoRectangle::new(0.0, 0.0, 1.0, 1.0);
    let minimum_height = 0.0;
    let maximum_height = 1.0;
    let tbr = TileBoundingRegion::new(TileBoundingRegionOptions {
        rectangle,
        minimum_height: Some(minimum_height),
        maximum_height: Some(maximum_height),
        ellipsoid: None,
        compute_bounding_volumes: None,
    })
    .unwrap();

    assert!(tbr.bounding_volume().is_some());
    assert!(tbr.bounding_sphere().is_some());
    assert_eq!(tbr.rectangle, rectangle);
    assert_eq!(tbr.minimum_height, minimum_height);
    assert_eq!(tbr.maximum_height, maximum_height);
}

#[test]
fn test_can_be_instantiated_with_only_a_rectangle() {
    let rectangle = GeoRectangle::new(0.0, 0.0, 1.0, 1.0);
    let tbr = TileBoundingRegion::new(TileBoundingRegionOptions {
        rectangle,
        minimum_height: None,
        maximum_height: None,
        ellipsoid: None,
        compute_bounding_volumes: None,
    })
    .unwrap();

    assert!(tbr.bounding_volume().is_some());
    assert!(tbr.bounding_sphere().is_some());
    assert_eq!(tbr.rectangle, rectangle);
    assert_eq!(tbr.minimum_height, 0.0);
    assert_eq!(tbr.maximum_height, 0.0);
}

#[test]
fn test_can_be_instantiated_from_a_zero_area_rectangle() {
    let zero_area_rectangle = GeoRectangle::new(0.0, 0.0, 0.0, 0.0);
    let tbr = TileBoundingRegion::new(TileBoundingRegionOptions {
        rectangle: zero_area_rectangle,
        minimum_height: None,
        maximum_height: None,
        ellipsoid: None,
        compute_bounding_volumes: None,
    })
    .unwrap();

    assert!(tbr.bounding_volume().is_some());
    assert!(tbr.bounding_sphere().is_some());
    assert_eq!(tbr.rectangle, zero_area_rectangle);
    assert_eq!(tbr.minimum_height, 0.0);
    assert_eq!(tbr.maximum_height, 0.0);
}

#[test]
fn test_distance_to_camera_inside_and_outside_height() {
    let rectangle = GeoRectangle::new(0.0, 0.0, 1.0, 1.0);
    let tile = TileBoundingRegion::new(TileBoundingRegionOptions {
        rectangle,
        minimum_height: Some(0.0),
        maximum_height: Some(1.0),
        ellipsoid: None,
        compute_bounding_volumes: None,
    })
    .unwrap();

    // Camera inside bounding region
    let camera_inside = TileBoundingRegionCamera::from_cartographic(
        Cartographic::new(EPSILON6, 0.0, 0.0),
        &Ellipsoid::WGS84,
    );
    let frame_state_inside = TileBoundingRegionFrameState {
        camera: camera_inside,
        mode: SceneMode::Scene3D,
        map_projection: None,
    };
    assert_eq!(tile.distance_to_camera(&frame_state_inside), 0.0);

    // Camera outside bounding region (height = 2.0, max_height = 1.0 -> dist = 1.0)
    let camera_outside = TileBoundingRegionCamera::from_cartographic(
        Cartographic::new(0.0, 0.0, 2.0),
        &Ellipsoid::WGS84,
    );
    let frame_state_outside = TileBoundingRegionFrameState {
        camera: camera_outside,
        mode: SceneMode::Scene3D,
        map_projection: None,
    };
    assert!((tile.distance_to_camera(&frame_state_outside) - 1.0).abs() < EPSILON6);
}

#[test]
fn test_distance_to_camera_directions() {
    let offset = 0.0001;
    let west = -0.001;
    let south = -0.001;
    let east = 0.001;
    let north = 0.001;

    let tile = TileBoundingRegion::new(TileBoundingRegionOptions {
        rectangle: GeoRectangle::new(west, south, east, north),
        minimum_height: Some(0.0),
        maximum_height: Some(10.0),
        ellipsoid: None,
        compute_bounding_volumes: None,
    })
    .unwrap();

    // Inside rectangle, above height
    let camera_above = TileBoundingRegionCamera::from_cartographic(
        Cartographic::new(0.0, 0.0, 20.0),
        &Ellipsoid::WGS84,
    );
    let fs = TileBoundingRegionFrameState {
        camera: camera_above,
        mode: SceneMode::Scene3D,
        map_projection: None,
    };
    assert!((tile.distance_to_camera(&fs) - 10.0).abs() < EPSILON3);

    // Inside rectangle, below height
    let camera_inside = TileBoundingRegionCamera::from_cartographic(
        Cartographic::new(0.0, 0.0, 5.0),
        &Ellipsoid::WGS84,
    );
    let fs = TileBoundingRegionFrameState {
        camera: camera_inside,
        mode: SceneMode::Scene3D,
        map_projection: None,
    };
    assert_eq!(tile.distance_to_camera(&fs), 0.0);

    // From southwest
    let camera_sw = TileBoundingRegionCamera::from_cartographic(
        Cartographic::new(west - offset, south - offset, 0.0),
        &Ellipsoid::WGS84,
    );
    let sw_position: Cartesian3 = Ellipsoid::WGS84
        .cartographic_to_cartesian(Cartographic::new(west, south, 0.0))
        .unwrap()
        .into();
    let expected_distance = camera_sw.position_wc.distance(&sw_position);
    let fs = TileBoundingRegionFrameState {
        camera: camera_sw,
        mode: SceneMode::Scene3D,
        map_projection: None,
    };
    assert!((tile.distance_to_camera(&fs) - expected_distance).abs() < EPSILON1);

    // From northeast
    let camera_ne = TileBoundingRegionCamera::from_cartographic(
        Cartographic::new(east + offset, north + offset, 0.0),
        &Ellipsoid::WGS84,
    );
    let ne_position: Cartesian3 = Ellipsoid::WGS84
        .cartographic_to_cartesian(Cartographic::new(east, north, 0.0))
        .unwrap()
        .into();
    let expected_distance_ne = camera_ne.position_wc.distance(&ne_position);
    let fs = TileBoundingRegionFrameState {
        camera: camera_ne,
        mode: SceneMode::Scene3D,
        map_projection: None,
    };
    assert!((tile.distance_to_camera(&fs) - expected_distance_ne).abs() < EPSILON1);
}

#[test]
fn test_distance_to_camera_northern_and_southern_hemispheres() {
    let ellipsoid = Ellipsoid::WGS84;
    let tiling_scheme =
        GeographicTilingScheme::new(ellipsoid, GeoRectangle::MAXIMUM, 2, 1).unwrap();

    // Tile on the northern hemisphere
    let rectangle = tiling_scheme.tile_xy_to_rectangle(5, 0, 2).unwrap();
    let mut camera_cartographic = Cartographic::new(
        (rectangle.west + rectangle.east) * 0.5,
        rectangle.south,
        0.0,
    );
    camera_cartographic.latitude -= EPSILON8;

    let tile = TileBoundingRegion::new(TileBoundingRegionOptions {
        rectangle,
        minimum_height: Some(0.0),
        maximum_height: Some(10.0),
        ellipsoid: Some(ellipsoid),
        compute_bounding_volumes: None,
    })
    .unwrap();

    let camera = TileBoundingRegionCamera::from_cartographic(camera_cartographic, &ellipsoid);
    let fs = TileBoundingRegionFrameState {
        camera,
        mode: SceneMode::Scene3D,
        map_projection: None,
    };
    assert!(tile.distance_to_camera(&fs) < EPSILON8 * ellipsoid.maximum_radius());

    // Tile on the southern hemisphere
    let rectangle_south = tiling_scheme.tile_xy_to_rectangle(4, 3, 2).unwrap();
    let mut camera_cartographic_south = Cartographic::new(
        (rectangle_south.west + rectangle_south.east) * 0.5,
        rectangle_south.north,
        0.0,
    );
    camera_cartographic_south.latitude += EPSILON8;

    let tile_south = TileBoundingRegion::new(TileBoundingRegionOptions {
        rectangle: rectangle_south,
        minimum_height: Some(0.0),
        maximum_height: Some(10.0),
        ellipsoid: Some(ellipsoid),
        compute_bounding_volumes: None,
    })
    .unwrap();

    let camera_south =
        TileBoundingRegionCamera::from_cartographic(camera_cartographic_south, &ellipsoid);
    let fs_south = TileBoundingRegionFrameState {
        camera: camera_south,
        mode: SceneMode::Scene3D,
        map_projection: None,
    };
    assert!(tile_south.distance_to_camera(&fs_south) < EPSILON8 * ellipsoid.maximum_radius());
}

#[test]
fn test_distance_to_camera_in_2d() {
    let offset = 0.0001;
    let west = -0.001;
    let south = -0.001;
    let east = 0.001;
    let north = 0.001;

    let tile = TileBoundingRegion::new(TileBoundingRegionOptions {
        rectangle: GeoRectangle::new(west, south, east, north),
        minimum_height: Some(0.0),
        maximum_height: Some(10.0),
        ellipsoid: None,
        compute_bounding_volumes: None,
    })
    .unwrap();

    let projection = GeographicProjection::new(Ellipsoid::WGS84).unwrap();

    // Inside rectangle
    let camera_inside = TileBoundingRegionCamera {
        position_wc: Cartesian3::new(Ellipsoid::WGS84.radii().x, 0.0, 0.0),
        position_cartographic: Cartographic::new(0.0, 0.0, 0.0),
    };
    let fs_inside = TileBoundingRegionFrameState {
        camera: camera_inside,
        mode: SceneMode::Scene2D,
        map_projection: Some(&projection),
    };
    assert_eq!(
        tile.distance_to_camera(&fs_inside),
        Ellipsoid::WGS84.radii().x
    );

    // From southwest
    let pos_3d = Cartographic::new(west - offset, south - offset, 0.0);
    let pos_2d = projection.project_2d(pos_3d).unwrap();
    let sw_2d = projection
        .project_2d(Cartographic::new(west, south, 0.0))
        .unwrap();
    let distance_2d = sw_2d.distance(pos_2d);
    let height = Ellipsoid::WGS84.radii().x;
    let expected_distance = (distance_2d * distance_2d + height * height).sqrt();

    let camera_sw = TileBoundingRegionCamera {
        position_wc: Cartesian3::new(height, pos_2d.x, pos_2d.y),
        position_cartographic: pos_3d,
    };
    let fs_sw = TileBoundingRegionFrameState {
        camera: camera_sw,
        mode: SceneMode::Scene2D,
        map_projection: Some(&projection),
    };
    assert!((tile.distance_to_camera(&fs_sw) - expected_distance).abs() < 10.0);
}

#[test]
fn test_create_debug_volume() {
    let rectangle = GeoRectangle::new(0.0, 0.0, 1.0, 1.0);
    let tile = TileBoundingRegion::new(TileBoundingRegionOptions {
        rectangle,
        minimum_height: Some(0.0),
        maximum_height: Some(1.0),
        ellipsoid: None,
        compute_bounding_volumes: None,
    })
    .unwrap();

    let debug_volume = tile.create_debug_volume(Color::BLUE).unwrap();
    assert!(debug_volume.geometry_instances.is_some());
}

#[test]
fn test_intersects_plane() {
    let rectangle = GeoRectangle::new(0.0, 0.0, 1.0, 1.0);
    let tile = TileBoundingRegion::new(TileBoundingRegionOptions {
        rectangle,
        minimum_height: Some(0.0),
        maximum_height: Some(1.0),
        ellipsoid: None,
        compute_bounding_volumes: None,
    })
    .unwrap();

    let normal = Cartesian3::new(1.0, 0.0, 0.0);
    let center = Ellipsoid::WGS84
        .cartographic_to_cartesian(Cartographic::new(0.5, 0.5, 0.5))
        .unwrap();
    let plane = Plane::from_point_normal(center, normal.into()).unwrap();
    assert_eq!(
        tile.intersect_plane(plane).unwrap(),
        PlaneIntersection::Intersecting
    );
}

#[test]
fn test_uncomputed_bounding_volumes() {
    let rectangle = GeoRectangle::new(0.0, 0.0, 1.0, 1.0);
    let mut tile = TileBoundingRegion::new(TileBoundingRegionOptions {
        rectangle,
        minimum_height: Some(0.0),
        maximum_height: Some(1.0),
        ellipsoid: None,
        compute_bounding_volumes: Some(false),
    })
    .unwrap();

    assert!(tile.bounding_volume().is_none());
    assert!(tile.bounding_sphere().is_none());
    let plane = Plane::from_point_normal(Cartesian3::ZERO.into(), Cartesian3::UNIT_Z.into()).unwrap();
    assert!(tile.intersect_plane(plane).is_err());

    tile.compute_bounding_volumes(None);
    assert!(tile.bounding_volume().is_some());
    assert!(tile.bounding_sphere().is_some());
    assert!(tile.intersect_plane(plane).is_ok());
}
