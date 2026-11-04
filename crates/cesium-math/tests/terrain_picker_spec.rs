use cesium_math::{
    Cartesian3, GeographicProjection, Matrix4, Ray, SceneMode, TerrainEncoding, TerrainPicker,
};
use glam::DVec3;

fn create_picker_with_triangles(vertices: Vec<f32>, indices: Vec<u32>) -> TerrainPicker {
    TerrainPicker::new(vertices, indices, TerrainEncoding::new_default())
}

fn create_tile_transform() -> Matrix4 {
    let mut mat = Matrix4::from_translation(Cartesian3::new(5.0, 5.0, 5.0));
    mat.set_scale(Cartesian3::new(10.0, 10.0, 20.0));
    mat
}

#[test]
fn test_picks_closer_triangle_when_multiple_triangles_along_ray() {
    let vertices = vec![
        // Tri 0 at Z = 5
        0.0, 0.0, 5.0, 5.0, 0.0, 0.0,
        10.0, 0.0, 5.0, 5.0, 1.0, 0.0,
        0.0, 10.0, 5.0, 5.0, 0.0, 1.0,
        // Tri 1 at Z = 10
        0.0, 0.0, 10.0, 10.0, 0.0, 0.0,
        10.0, 0.0, 10.0, 10.0, 1.0, 0.0,
        0.0, 10.0, 10.0, 10.0, 0.0, 1.0,
    ];
    let indices = vec![0, 1, 2, 3, 4, 5];

    let mut picker = create_picker_with_triangles(vertices, indices);
    let projection = GeographicProjection::default();
    let tile_transform = create_tile_transform();

    // Ray shooting from Z = 20 downwards towards Z = 0
    let ray = Ray::new(
        DVec3::new(2.0, 2.0, 20.0),
        DVec3::new(0.0, 0.0, -1.0),
    )
    .unwrap();

    let hit = picker.ray_intersect(
        &ray,
        &tile_transform,
        false,
        SceneMode::Scene3D,
        &projection,
    );
    assert!(hit.is_some(), "should intersect");
    let pt = hit.unwrap();
    assert!((pt.z - 10.0).abs() < 1e-6);
    assert!((pt.x - 2.0).abs() < 1e-6);
    assert!((pt.y - 2.0).abs() < 1e-6);
}

#[test]
fn test_ignores_triangles_behind_the_ray() {
    let vertices = vec![
        0.0, 0.0, -5.0, -5.0, 0.0, 0.0,
        10.0, 0.0, -5.0, -5.0, 1.0, 0.0,
        0.0, 10.0, -5.0, -5.0, 0.0, 1.0,
    ];
    let indices = vec![0, 1, 2];

    let mut picker = create_picker_with_triangles(vertices, indices);
    let projection = GeographicProjection::default();
    let tile_transform = create_tile_transform();

    // Ray starting at origin and pointing +Z (away from triangle)
    let ray = Ray::new(
        DVec3::new(2.0, 2.0, 0.0),
        DVec3::new(0.0, 0.0, 1.0),
    )
    .unwrap();

    let hit = picker.ray_intersect(
        &ray,
        &tile_transform,
        false,
        SceneMode::Scene3D,
        &projection,
    );
    assert!(hit.is_none(), "triangles behind the ray should be ignored");
}

#[test]
fn test_respects_cull_back_faces() {
    let vertices = vec![
        0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
        10.0, 0.0, 0.0, 0.0, 1.0, 0.0,
        0.0, 10.0, 0.0, 0.0, 0.0, 1.0,
    ];
    let indices = vec![0, 1, 2];

    let mut picker = create_picker_with_triangles(vertices, indices);
    let projection = GeographicProjection::default();
    let tile_transform = create_tile_transform();

    // Ray shooting from below (Z = -10) pointing upwards (+Z): looking at the back face!
    let ray_from_back = Ray::new(
        DVec3::new(2.0, 2.0, -10.0),
        DVec3::new(0.0, 0.0, 1.0),
    )
    .unwrap();

    // When cull_back_faces is true, backface should not register hit
    let hit_culled = picker.ray_intersect(
        &ray_from_back,
        &tile_transform,
        true,
        SceneMode::Scene3D,
        &projection,
    );
    assert!(hit_culled.is_none(), "back face should be culled when cull_back_faces is true");

    // When cull_back_faces is false, backface should register hit
    let hit_unculled = picker.ray_intersect(
        &ray_from_back,
        &tile_transform,
        false,
        SceneMode::Scene3D,
        &projection,
    );
    assert!(hit_unculled.is_some(), "back face should be hit when cull_back_faces is false");
    let pt = hit_unculled.unwrap();
    assert!((pt.z - 0.0).abs() < 1e-6);
}

#[test]
fn test_picks_same_point_repeatedly_given_same_ray() {
    let vertices = vec![
        0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
        10.0, 0.0, 0.0, 0.0, 1.0, 0.0,
        0.0, 10.0, 0.0, 0.0, 0.0, 1.0,
    ];
    let indices = vec![0, 1, 2];

    let mut picker = create_picker_with_triangles(vertices, indices);
    let projection = GeographicProjection::default();
    let tile_transform = create_tile_transform();
    let ray = Ray::new(
        DVec3::new(1.0, 1.0, 5.0),
        DVec3::new(0.0, 0.0, -1.0),
    )
    .unwrap();

    let hit1 = picker.ray_intersect(&ray, &tile_transform, false, SceneMode::Scene3D, &projection);
    let hit2 = picker.ray_intersect(&ray, &tile_transform, false, SceneMode::Scene3D, &projection);
    let hit3 = picker.ray_intersect(&ray, &tile_transform, false, SceneMode::Scene3D, &projection);
    let hit4 = picker.ray_intersect(&ray, &tile_transform, false, SceneMode::Scene3D, &projection);

    assert!(hit1.is_some());
    assert_eq!(hit1, hit2);
    assert_eq!(hit1, hit3);
    assert_eq!(hit1, hit4);
}

#[test]
fn test_quadtree_node_child_addition() {
    let mut node = cesium_math::TerrainPickerNode::new(0, 0, 0);
    assert_eq!(node.children.len(), 0);
    node.add_child(0);
    node.add_child(1);
    node.add_child(2);
    node.add_child(3);
    assert_eq!(node.children.len(), 4);
    assert_eq!(node.children[0].level, 1);
    assert_eq!(node.children[3].level, 1);
}
