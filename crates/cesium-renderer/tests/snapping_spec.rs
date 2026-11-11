use cesium_math::{Cartesian2, Cartesian3, EPSILON7};
use cesium_renderer::snapping::{SnapHit, Snapping};

fn surface_hit(x: f64, y: f64, depth: f64, object_id: Option<u64>) -> SnapHit {
    SnapHit::surface(x, y, depth, object_id)
}

fn edge_hit(x: f64, y: f64, depth: f64, object_id: Option<u64>) -> SnapHit {
    SnapHit::edge(x, y, depth, object_id)
}

#[test]
fn test_select_best_hit_lone_surface() {
    let hit = surface_hit(1.0, 2.0, 10.0, None);
    assert_eq!(Snapping::select_best_hit(std::slice::from_ref(&hit)), Some(&hit));
}

#[test]
fn test_select_best_hit_prefers_edge_at_same_depth() {
    let surface = surface_hit(0.0, 0.0, 10.0, None);
    let edge = edge_hit(4.0, 0.0, 10.0, None);
    assert_eq!(Snapping::select_best_hit(&[surface, edge.clone()]), Some(&edge));
}

#[test]
fn test_select_best_hit_edge_closest_to_cursor() {
    let near_edge = edge_hit(1.0, 1.0, 10.0, None);
    let far_edge = edge_hit(5.0, 0.0, 10.0, None);
    assert_eq!(
        Snapping::select_best_hit(&[far_edge, near_edge.clone()]),
        Some(&near_edge)
    );
}

#[test]
fn test_select_best_hit_surface_closest_when_no_edges() {
    let near_surface = surface_hit(0.0, 1.0, 10.0, None);
    let far_surface = surface_hit(3.0, 3.0, 5.0, None);
    assert_eq!(
        Snapping::select_best_hit(&[far_surface, near_surface.clone()]),
        Some(&near_surface)
    );
}

#[test]
fn test_select_best_hit_does_not_snap_through_nearer_surface() {
    let near_surface = surface_hit(0.0, 0.0, 13.7, None);
    let far_edge = edge_hit(0.0, 0.0, 655.0, None);
    assert_eq!(
        Snapping::select_best_hit(&[near_surface.clone(), far_edge]),
        Some(&near_surface)
    );
}

#[test]
fn test_select_best_hit_keeps_edge_on_occluding_surface() {
    let surface = surface_hit(0.0, 0.0, 10.0, None);
    let edge = edge_hit(2.0, 0.0, 10.5, None);
    assert_eq!(Snapping::select_best_hit(&[surface, edge.clone()]), Some(&edge));

    let surface2 = surface_hit(0.0, 0.0, 10.0, None);
    let edge2 = edge_hit(3.0, 0.0, 10.0, None);
    assert_eq!(Snapping::select_best_hit(&[surface2, edge2.clone()]), Some(&edge2));
}

#[test]
fn test_select_best_hit_allows_deep_edge_when_nearer_surface_outside_radius() {
    let off_to_side_surface = surface_hit(10.0, 0.0, 5.0, None);
    let behind_edge = edge_hit(0.0, 0.0, 100.0, None);
    assert_eq!(
        Snapping::select_best_hit(&[off_to_side_surface, behind_edge.clone()]),
        Some(&behind_edge)
    );
}

#[test]
fn test_select_best_hit_fallback_to_closest_surface_when_all_edges_occluded() {
    let near_surface = surface_hit(1.0, 0.0, 10.0, None);
    let other_surface = surface_hit(2.0, 2.0, 12.0, None);
    let occluded_edge = edge_hit(0.0, 0.0, 500.0, None);
    assert_eq!(
        Snapping::select_best_hit(&[occluded_edge, other_surface, near_surface.clone()]),
        Some(&near_surface)
    );
}

#[test]
fn test_select_best_hit_closest_edge_when_no_surfaces() {
    let near_edge = edge_hit(1.0, 0.0, 400.0, None);
    let deep_edge = edge_hit(0.0, 2.0, 800.0, None);
    assert_eq!(
        Snapping::select_best_hit(&[deep_edge, near_edge.clone()]),
        Some(&near_edge)
    );
}

#[test]
fn test_nearest_surface_hit() {
    let obj_id = 42;
    assert_eq!(Snapping::nearest_surface_hit(&[], obj_id), None);

    let edge = edge_hit(0.0, 0.0, 10.0, Some(obj_id));
    assert_eq!(Snapping::nearest_surface_hit(&[edge], obj_id), None);

    let other_surface = surface_hit(0.0, 0.0, 10.0, Some(999));
    assert_eq!(Snapping::nearest_surface_hit(&[other_surface], obj_id), None);

    let near_surface = surface_hit(1.0, 1.0, 10.0, Some(obj_id));
    let far_surface = surface_hit(3.0, 0.0, 5.0, Some(obj_id));
    assert_eq!(
        Snapping::nearest_surface_hit(&[far_surface, near_surface.clone()], obj_id),
        Some(&near_surface)
    );

    let edge = edge_hit(0.0, 0.0, 10.0, Some(obj_id));
    let other_obj_surface = surface_hit(1.0, 0.0, 10.0, Some(123));
    let same_surface = surface_hit(2.0, 2.0, 10.0, Some(obj_id));
    assert_eq!(
        Snapping::nearest_surface_hit(&[edge, other_obj_surface, same_surface.clone()], obj_id),
        Some(&same_surface)
    );
}

#[test]
fn test_snap_hit_to_world_axis_aligned() {
    let cam_dir = Cartesian3::new(0.0, 0.0, -1.0);
    let ray_origin = Cartesian3::ZERO;
    let ray_dir = Cartesian3::new(0.0, 0.0, -1.0);
    let hit = edge_hit(0.0, 0.0, 100.0, None);

    let world_pos = Snapping::snap_hit_to_world(cam_dir, ray_origin, Some(ray_dir), &hit)
        .expect("should unproject");
    assert!((world_pos.x - 0.0).abs() < EPSILON7);
    assert!((world_pos.y - 0.0).abs() < EPSILON7);
    assert!((world_pos.z - (-100.0)).abs() < EPSILON7);
}

#[test]
fn test_snap_hit_to_world_off_axis() {
    let cos = 0.5;
    let sin = (1.0f64 - cos * cos).sqrt();
    let ray_dir = Cartesian3::new(sin, 0.0, -cos);
    let cam_dir = Cartesian3::new(0.0, 0.0, -1.0);
    let hit = edge_hit(0.0, 0.0, 100.0, None);

    let world_pos = Snapping::snap_hit_to_world(cam_dir, Cartesian3::ZERO, Some(ray_dir), &hit)
        .expect("should unproject off-axis");
    let expected = ray_dir * 200.0;
    assert!((world_pos.x - expected.x).abs() < EPSILON7);
    assert!((world_pos.y - expected.y).abs() < EPSILON7);
    assert!((world_pos.z - expected.z).abs() < EPSILON7);
}

#[test]
fn test_snap_hit_to_world_invalid_cases() {
    let cam_dir = Cartesian3::new(0.0, 0.0, -1.0);
    let hit = edge_hit(0.0, 0.0, 100.0, None);

    // None ray direction
    assert_eq!(
        Snapping::snap_hit_to_world(cam_dir, Cartesian3::ZERO, None, &hit),
        None
    );

    // Ray pointing away from view direction
    let away_dir = Cartesian3::new(0.0, 0.0, 1.0);
    assert_eq!(
        Snapping::snap_hit_to_world(cam_dir, Cartesian3::ZERO, Some(away_dir), &hit),
        None
    );
}

#[test]
fn test_validate_window_position() {
    assert!(Snapping::validate_window_position(None).is_err());
    assert!(Snapping::validate_window_position(Some(Cartesian2::new(10.0, 20.0))).is_ok());
}
