use cesium_math::{
    Cartesian3, Color, Ellipsoid, Plane, PlaneIntersection, S2Cell, EPSILON7,
};
use cesium_renderer::{
    TileBoundingS2Cell, TileBoundingS2CellCamera, TileBoundingS2CellFrameState,
    TileBoundingS2CellOptions,
};

fn project_point_onto_plane(plane: &Plane, point: Cartesian3) -> Cartesian3 {
    let normal = Cartesian3::from(plane.normal);
    let dist = normal.dot(&point) + plane.distance;
    point - normal * dist
}

#[test]
fn test_can_be_instantiated_with_s2_cell() {
    let t_s2_cell = TileBoundingS2Cell::new(TileBoundingS2CellOptions {
        token: "1".to_string(),
        minimum_height: None,
        maximum_height: None,
        ellipsoid: None,
        compute_bounding_volumes: None,
    })
    .unwrap();

    assert!(t_s2_cell.bounding_sphere().is_some());
    assert_eq!(t_s2_cell.minimum_height, 0.0);
    assert_eq!(t_s2_cell.maximum_height, 0.0);
}

#[test]
fn test_can_be_instantiated_with_s2_cell_and_heights() {
    let t_s2_cell = TileBoundingS2Cell::new(TileBoundingS2CellOptions {
        token: "1".to_string(),
        minimum_height: Some(0.0),
        maximum_height: Some(100000.0),
        ellipsoid: None,
        compute_bounding_volumes: None,
    })
    .unwrap();

    assert!(t_s2_cell.bounding_sphere().is_some());
    assert_eq!(t_s2_cell.minimum_height, 0.0);
    assert_eq!(t_s2_cell.maximum_height, 100000.0);
}

#[test]
fn test_distance_to_camera_inside_bounding_volume() {
    let s2_cell = S2Cell::from_token("1").unwrap();
    let tile_s2_cell = TileBoundingS2Cell::new(TileBoundingS2CellOptions {
        token: "1".to_string(),
        minimum_height: Some(0.0),
        maximum_height: Some(100000.0),
        ellipsoid: None,
        compute_bounding_volumes: None,
    })
    .unwrap();

    let camera = TileBoundingS2CellCamera::new(s2_cell.get_center(Some(&Ellipsoid::WGS84)));
    let fs = TileBoundingS2CellFrameState { camera };
    assert_eq!(tile_s2_cell.distance_to_camera(&fs), 0.0);
}

#[test]
fn test_distance_to_camera_case_1_one_plane() {
    let tile_s2_cell = TileBoundingS2Cell::new(TileBoundingS2CellOptions {
        token: "1".to_string(),
        minimum_height: Some(0.0),
        maximum_height: Some(100000.0),
        ellipsoid: None,
        compute_bounding_volumes: None,
    })
    .unwrap();

    let test_distance = 100.0;

    // Test against top plane
    let mut top_plane = tile_s2_cell.bounding_planes[0];
    top_plane.distance -= test_distance;
    let camera_pos = project_point_onto_plane(&top_plane, tile_s2_cell.center);
    let fs = TileBoundingS2CellFrameState {
        camera: TileBoundingS2CellCamera::new(camera_pos),
    };
    assert!((tile_s2_cell.distance_to_camera(&fs) - test_distance).abs() < EPSILON7);

    // Test against first side plane
    let mut side_plane0 = tile_s2_cell.bounding_planes[2];
    let edge_one = (tile_s2_cell.vertices[0] + tile_s2_cell.vertices[1]) * 0.5;
    let edge_two = (tile_s2_cell.vertices[4] + tile_s2_cell.vertices[5]) * 0.5;
    let face_center = (edge_one + edge_two) * 0.5;

    side_plane0.distance -= test_distance;
    let camera_pos_side = project_point_onto_plane(&side_plane0, face_center);
    let fs_side = TileBoundingS2CellFrameState {
        camera: TileBoundingS2CellCamera::new(camera_pos_side),
    };
    assert!((tile_s2_cell.distance_to_camera(&fs_side) - test_distance).abs() < EPSILON7);
}

#[test]
fn test_distance_to_camera_case_2_two_planes() {
    let tile_s2_cell = TileBoundingS2Cell::new(TileBoundingS2CellOptions {
        token: "1".to_string(),
        minimum_height: Some(0.0),
        maximum_height: Some(100000.0),
        ellipsoid: None,
        compute_bounding_volumes: None,
    })
    .unwrap();

    let test_distance = 5.0;

    // Test with top plane and first side plane
    let mut camera_pos = (tile_s2_cell.vertices[0] + tile_s2_cell.vertices[1]) * 0.5;
    camera_pos.z -= test_distance;
    let fs = TileBoundingS2CellFrameState {
        camera: TileBoundingS2CellCamera::new(camera_pos),
    };
    assert!((tile_s2_cell.distance_to_camera(&fs) - test_distance).abs() < EPSILON7);

    // Test with first and second side planes
    let mut camera_pos2 = (tile_s2_cell.vertices[0] + tile_s2_cell.vertices[4]) * 0.5;
    camera_pos2.x -= 1.0;
    camera_pos2.z -= 1.0;
    let fs2 = TileBoundingS2CellFrameState {
        camera: TileBoundingS2CellCamera::new(camera_pos2),
    };
    assert!((tile_s2_cell.distance_to_camera(&fs2) - std::f64::consts::SQRT_2).abs() < EPSILON7);

    // Test with bottom plane and second side plane (obtuse dihedral angle)
    let mut camera_pos3 = (tile_s2_cell.vertices[5] + tile_s2_cell.vertices[6]) * 0.5;
    camera_pos3.x -= 10000.0;
    camera_pos3.y -= 1.0;
    let fs3 = TileBoundingS2CellFrameState {
        camera: TileBoundingS2CellCamera::new(camera_pos3),
    };
    assert!((tile_s2_cell.distance_to_camera(&fs3) - 10000.0).abs() < EPSILON7);
}

#[test]
fn test_distance_to_camera_case_3_three_planes() {
    let tile_s2_cell = TileBoundingS2Cell::new(TileBoundingS2CellOptions {
        token: "1".to_string(),
        minimum_height: Some(0.0),
        maximum_height: Some(100000.0),
        ellipsoid: None,
        compute_bounding_volumes: None,
    })
    .unwrap();

    let mut camera_pos = tile_s2_cell.vertices[2];
    camera_pos.x += 1.0;
    camera_pos.y += 1.0;
    camera_pos.z += 1.0;
    let fs = TileBoundingS2CellFrameState {
        camera: TileBoundingS2CellCamera::new(camera_pos),
    };
    assert!((tile_s2_cell.distance_to_camera(&fs) - 3.0_f64.sqrt()).abs() < EPSILON7);
}

#[test]
fn test_distance_to_camera_case_4_more_than_three_planes() {
    let tile_s2_cell = TileBoundingS2Cell::new(TileBoundingS2CellOptions {
        token: "1".to_string(),
        minimum_height: Some(0.0),
        maximum_height: Some(100000.0),
        ellipsoid: None,
        compute_bounding_volumes: None,
    })
    .unwrap();

    let max_radius = Ellipsoid::WGS84.maximum_radius();
    let camera_pos = Cartesian3::new(-max_radius, 0.0, 0.0);
    let fs = TileBoundingS2CellFrameState {
        camera: TileBoundingS2CellCamera::new(camera_pos),
    };
    let expected = max_radius + tile_s2_cell.bounding_planes[1].distance;
    assert!((tile_s2_cell.distance_to_camera(&fs) - expected).abs() < EPSILON7);
}

#[test]
fn test_create_debug_volume() {
    let tile_s2_cell = TileBoundingS2Cell::new(TileBoundingS2CellOptions {
        token: "1".to_string(),
        minimum_height: Some(0.0),
        maximum_height: Some(100000.0),
        ellipsoid: None,
        compute_bounding_volumes: None,
    })
    .unwrap();

    let debug_volume = tile_s2_cell.create_debug_volume(Color::BLUE).unwrap();
    assert!(debug_volume.geometry_instances.is_some());
    assert_eq!(
        debug_volume.geometry_instances.as_ref().unwrap().len(),
        6
    );
}

#[test]
fn test_intersects_plane() {
    let tile_s2_cell = TileBoundingS2Cell::new(TileBoundingS2CellOptions {
        token: "1".to_string(),
        minimum_height: Some(0.0),
        maximum_height: Some(100000.0),
        ellipsoid: None,
        compute_bounding_volumes: None,
    })
    .unwrap();

    // ORIGIN_ZX_PLANE: normal = Y, distance = 0
    let origin_zx = Plane::new(glam::DVec3::Y, 0.0).unwrap();
    assert_eq!(
        tile_s2_cell.intersect_plane(origin_zx),
        PlaneIntersection::Intersecting
    );

    // outside plane
    let mut outside_plane = Plane::new(glam::DVec3::X, 0.0).unwrap();
    outside_plane.distance -= 2.0 * Ellipsoid::WGS84.maximum_radius();
    assert_eq!(
        tile_s2_cell.intersect_plane(outside_plane),
        PlaneIntersection::Outside
    );

    // ORIGIN_YZ_PLANE: normal = X, distance = 0
    let origin_yz = Plane::new(glam::DVec3::X, 0.0).unwrap();
    assert_eq!(
        tile_s2_cell.intersect_plane(origin_yz),
        PlaneIntersection::Inside
    );
}
