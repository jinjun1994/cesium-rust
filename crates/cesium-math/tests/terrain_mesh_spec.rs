use cesium_math::{
    Cartesian3, GeoRectangle, GeographicProjection, Ray, SceneMode, Sphere, TerrainEncoding,
    TerrainMesh,
};

fn create_test_mesh() -> TerrainMesh {
    let center = Cartesian3::new(0.0, 0.0, 0.0);
    TerrainMesh::new(
        center,
        vec![0.0; 6], // Vertices - one vertex: X,Y,Z,H,U,V
        vec![0, 0, 0], // Indices - one triangle
        0,             // index_count_without_skirts
        1,             // vertex_count_without_skirts
        0.0,           // minimum_height
        0.0,           // maximum_height
        GeoRectangle::radians(0.0, 0.0, 1.0, 1.0).unwrap(),
        Sphere::new(center, 1.0),
        Cartesian3::new(0.0, 0.0, 0.0), // occludee_point_in_scaled_space
        Some(6),                        // vertex_stride
        None,                           // oriented_bounding_box
        TerrainEncoding::new_default(),
        vec![],
        vec![],
        vec![],
        vec![],
    )
}

#[test]
fn test_uses_3d_transform_when_picking_in_3d_mode() {
    let mut mesh = create_test_mesh();
    let projection = GeographicProjection::default();
    let ray = Ray::new(
        Cartesian3::new(1.0, 2.0, 3.0).into(),
        Cartesian3::new(4.0, 5.0, 6.0).into(),
    )
    .unwrap();

    let expected_transform = mesh.get_transform(SceneMode::Scene3D, &projection);
    mesh.pick(&ray, false, SceneMode::Scene3D, &projection);

    assert_eq!(mesh.last_pick_scene_mode, Some(SceneMode::Scene3D));
    assert_eq!(mesh.transform, expected_transform);
}

#[test]
fn test_uses_2d_transform_when_picking_in_2d_mode() {
    let mut mesh = create_test_mesh();
    let projection = GeographicProjection::default();
    let ray = Ray::new(
        Cartesian3::new(1.0, 2.0, 3.0).into(),
        Cartesian3::new(4.0, 5.0, 6.0).into(),
    )
    .unwrap();

    let expected_transform = mesh.get_transform(SceneMode::Scene2D, &projection);
    mesh.pick(&ray, false, SceneMode::Scene2D, &projection);

    assert_eq!(mesh.last_pick_scene_mode, Some(SceneMode::Scene2D));
    assert_eq!(mesh.transform, expected_transform);
}

#[test]
fn test_recomputes_the_transform_after_scene_mode_changes() {
    let mut mesh = create_test_mesh();
    let projection = GeographicProjection::default();
    let ray = Ray::new(
        Cartesian3::new(1.0, 2.0, 3.0).into(),
        Cartesian3::new(4.0, 5.0, 6.0).into(),
    )
    .unwrap();

    let expected_3d = mesh.get_transform(SceneMode::Scene3D, &projection);
    mesh.pick(&ray, false, SceneMode::Scene3D, &projection);
    assert_eq!(mesh.transform, expected_3d);

    mesh.update_scene_mode(SceneMode::Scene2D);
    assert!(mesh.terrain_picker.needs_rebuild);
    assert_eq!(mesh.last_pick_scene_mode, None);

    let expected_2d = mesh.get_transform(SceneMode::Scene2D, &projection);
    mesh.pick(&ray, false, SceneMode::Scene2D, &projection);
    assert_eq!(mesh.transform, expected_2d);
    assert_ne!(expected_3d, expected_2d);
}

#[test]
fn test_recomputes_the_transform_after_exaggeration_changes() {
    let mut mesh = create_test_mesh();
    let projection = GeographicProjection::default();
    let ray = Ray::new(
        Cartesian3::new(1.0, 2.0, 3.0).into(),
        Cartesian3::new(4.0, 5.0, 6.0).into(),
    )
    .unwrap();

    let expected_3d = mesh.get_transform(SceneMode::Scene3D, &projection);
    mesh.pick(&ray, false, SceneMode::Scene3D, &projection);
    assert_eq!(mesh.transform, expected_3d);

    mesh.update_exaggeration(2.0, 1.0);
    assert_eq!(mesh.encoding.exaggeration, 2.0);
    assert_eq!(mesh.encoding.exaggeration_relative_height, 1.0);
    assert!(mesh.terrain_picker.needs_rebuild);
    assert_eq!(mesh.last_pick_scene_mode, None);

    let expected_2d = mesh.get_transform(SceneMode::Scene2D, &projection);
    mesh.pick(&ray, false, SceneMode::Scene2D, &projection);
    assert_eq!(mesh.transform, expected_2d);
}

#[test]
fn test_recomputes_the_transform_after_a_pick_in_a_different_scene_mode() {
    let mut mesh = create_test_mesh();
    let projection = GeographicProjection::default();
    let ray = Ray::new(
        Cartesian3::new(1.0, 2.0, 3.0).into(),
        Cartesian3::new(4.0, 5.0, 6.0).into(),
    )
    .unwrap();

    let expected_3d = mesh.get_transform(SceneMode::Scene3D, &projection);
    mesh.pick(&ray, false, SceneMode::Scene3D, &projection);
    assert_eq!(mesh.transform, expected_3d);

    let expected_2d = mesh.get_transform(SceneMode::Scene2D, &projection);
    mesh.pick(&ray, false, SceneMode::Scene2D, &projection);
    assert_eq!(mesh.transform, expected_2d);
}

#[test]
fn test_actual_triangle_pick_intersection() {
    let center = Cartesian3::new(0.0, 0.0, 0.0);
    // Vertices of a triangle in the XY plane: (0, 0, 0), (10, 0, 0), (0, 10, 0)
    // Stride = 6 (X, Y, Z, H, U, V)
    let vertices = vec![
        0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
        10.0, 0.0, 0.0, 0.0, 1.0, 0.0,
        0.0, 10.0, 0.0, 0.0, 0.0, 1.0,
    ];
    let indices = vec![0, 1, 2];

    let obb = cesium_math::OrientedBoundingBox::new(
        glam::DVec3::new(5.0, 5.0, 0.0),
        glam::DMat3::from_diagonal(glam::DVec3::new(10.0, 10.0, 10.0)),
    );

    let mut mesh = TerrainMesh::new(
        center,
        vertices,
        indices,
        3,
        3,
        0.0,
        0.0,
        GeoRectangle::radians(0.0, 0.0, 0.001, 0.001).unwrap(),
        Sphere::new(center, 20.0),
        Cartesian3::ZERO,
        Some(6),
        Some(obb),
        TerrainEncoding::new_default(),
        vec![],
        vec![],
        vec![],
        vec![],
    );

    let projection = GeographicProjection::default();
    // A ray shooting from (2, 2, 10) downwards (0, 0, -1) to hit (2, 2, 0)
    let ray = Ray::new(
        Cartesian3::new(2.0, 2.0, 10.0).into(),
        Cartesian3::new(0.0, 0.0, -1.0).into(),
    )
    .unwrap();

    let hit = mesh.pick(&ray, false, SceneMode::Scene3D, &projection);
    assert!(hit.is_some(), "ray should intersect triangle");
    let hit_pt = hit.unwrap();
    assert!((hit_pt.x - 2.0).abs() < 1e-6);
    assert!((hit_pt.y - 2.0).abs() < 1e-6);
    assert!((hit_pt.z - 0.0).abs() < 1e-6);
}
