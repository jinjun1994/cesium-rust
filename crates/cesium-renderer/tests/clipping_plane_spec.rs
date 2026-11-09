use cesium_math::{Cartesian3, Matrix3, Matrix4, Plane, EPSILON10};
use cesium_renderer::ClippingPlane;
use glam::{DMat4, DVec3};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

#[test]
fn test_constructs() {
    let normal = Cartesian3::UNIT_X;
    let distance = 1.0;
    let clipping_plane = ClippingPlane::new(normal, distance);
    assert_eq!(clipping_plane.normal(), normal);
    assert_eq!(clipping_plane.distance(), distance);
}

#[test]
fn test_runs_on_change_callback_when_changed() {
    let normal = Cartesian3::UNIT_X;
    let distance = 1.0;
    let change_count = Arc::new(AtomicUsize::new(0));

    let mut clipping_plane = ClippingPlane::new(normal, distance);
    clipping_plane.index = 5;

    let count_clone = Arc::clone(&change_count);
    clipping_plane.on_change_callback = Some(Box::new(move |index| {
        assert_eq!(index, 5);
        count_clone.fetch_add(1, Ordering::SeqCst);
    }));

    // Distance change
    clipping_plane.set_distance(distance + 0.1);
    assert_eq!(change_count.load(Ordering::SeqCst), 1);

    // Distance non-change
    let cur_dist = clipping_plane.distance();
    clipping_plane.set_distance(cur_dist);
    assert_eq!(change_count.load(Ordering::SeqCst), 1);

    // Normal change
    clipping_plane.set_normal(Cartesian3::UNIT_Z);
    assert_eq!(change_count.load(Ordering::SeqCst), 2);

    // Normal non-change
    clipping_plane.set_normal(Cartesian3::UNIT_Z);
    assert_eq!(change_count.load(Ordering::SeqCst), 2);

    // Normal member change
    let cur_x = clipping_plane.normal().x;
    clipping_plane.set_normal_x(cur_x + 1.0);
    assert_eq!(change_count.load(Ordering::SeqCst), 3);
}

#[test]
fn test_can_be_instantiated_from_a_plane() {
    let plane = Plane::new(DVec3::X, 1.0).unwrap();
    let clipping_plane = ClippingPlane::from_plane(&plane);
    assert_eq!(clipping_plane.normal(), Cartesian3::UNIT_X);
    assert_eq!(clipping_plane.distance(), plane.distance);

    let mut scratch_clipping_plane = ClippingPlane::new(Cartesian3::UNIT_Y, 0.0);
    ClippingPlane::from_plane_to(&plane, &mut scratch_clipping_plane);
    assert_eq!(scratch_clipping_plane.normal(), Cartesian3::UNIT_X);
    assert_eq!(scratch_clipping_plane.distance(), plane.distance);
}

#[test]
fn test_clones() {
    let clipping_plane = ClippingPlane::new(Cartesian3::UNIT_X, 1.0);
    let clone_clipping_plane = clipping_plane.clone();
    assert_eq!(clipping_plane.normal(), clone_clipping_plane.normal());
    assert_eq!(clipping_plane.distance(), clone_clipping_plane.distance());

    let mut scratch_clipping_plane = ClippingPlane::new(Cartesian3::UNIT_Y, 0.0);
    clipping_plane.clone_to(&mut scratch_clipping_plane);
    assert_eq!(clipping_plane.normal(), scratch_clipping_plane.normal());
    assert_eq!(clipping_plane.distance(), scratch_clipping_plane.distance());
}

#[test]
fn test_works_with_plane_math() {
    let normal = Cartesian3::new(1.0, 2.0, 3.0).normalize();
    let clipping_plane = ClippingPlane::new(normal, 12.34);

    let scale = Matrix4::from_uniform_scale(2.0);
    let rot = Matrix4::from_rotation_translation(
        &Matrix3::from_rotation_y(std::f64::consts::PI),
        Cartesian3::ZERO,
    );
    let transform = scale.multiply(&rot);

    let plane = clipping_plane.to_plane().unwrap();
    let transformed_plane = plane.transform(DMat4::from(transform)).unwrap();

    assert_eq!(transformed_plane.distance, clipping_plane.distance() * 2.0);
    assert!((transformed_plane.normal.x - (-clipping_plane.normal().x)).abs() <= EPSILON10);
    assert_eq!(transformed_plane.normal.y, clipping_plane.normal().y);
    assert_eq!(transformed_plane.normal.z, -clipping_plane.normal().z);
}
