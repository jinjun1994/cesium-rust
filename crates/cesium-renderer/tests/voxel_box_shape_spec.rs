//! Tests for VoxelBoxShape matching CesiumJS `Scene/VoxelBoxShapeSpec.js` (Apache-2.0).

use std::f64::consts::{FRAC_PI_2, FRAC_PI_4};

use cesium_math::{BoundingSphere, Cartesian3, Matrix4, OrientedBoundingBox, Quaternion};
use cesium_renderer::spatial_node::SpatialNode;
use cesium_renderer::voxel_box_shape::{VoxelBoxShape, VoxelShape};
use glam::{DMat3, DVec3};

#[test]
fn test_constructs() {
    let shape = VoxelBoxShape::new();
    assert_eq!(shape.shape_transform, Matrix4::IDENTITY);
}

#[test]
fn test_update_works_with_model_matrix() {
    let mut shape = VoxelBoxShape::new();

    let translation = Cartesian3::new(1.0, 2.0, 3.0);
    let scale = Cartesian3::new(2.0, 3.0, 4.0);
    let angle = FRAC_PI_4;
    let rotation = Quaternion::from_axis_angle(Cartesian3::UNIT_Z, angle);

    let model_matrix = Matrix4::from_translation_quaternion_rotation_scale(
        translation,
        rotation,
        scale,
    );

    let min_bounds = VoxelBoxShape::DEFAULT_MIN_BOUNDS;
    let max_bounds = VoxelBoxShape::DEFAULT_MAX_BOUNDS;

    let expected_center = DVec3::new(translation.x, translation.y, translation.z);
    let expected_half_axes = DMat3::from_cols_array(&[
        scale.x * angle.cos(),
        scale.x * angle.sin(),
        0.0,
        scale.y * (angle + FRAC_PI_2).cos(),
        scale.y * (angle + FRAC_PI_2).sin(),
        0.0,
        0.0,
        0.0,
        scale.z,
    ]);

    let expected_obb = OrientedBoundingBox::new(expected_center, expected_half_axes);
    let expected_bs = BoundingSphere::new(expected_center, scale.magnitude());

    let visible = shape
        .update(Some(model_matrix), Some(min_bounds), Some(max_bounds), None, None)
        .unwrap();

    assert!(visible);
    assert_eq!(shape.oriented_bounding_box.center, expected_obb.center);

    for i in 0..3 {
        let diff = (shape.oriented_bounding_box.half_axes.col(i) - expected_obb.half_axes.col(i)).length();
        assert!(diff < 1e-12, "half axis {i} mismatch: diff = {diff}");
    }

    assert_eq!(shape.bounding_sphere.center, expected_bs.center);
    assert!((shape.bounding_sphere.radius - expected_bs.radius).abs() < 1e-12);
    assert_eq!(shape.bound_transform, model_matrix);
    assert_eq!(shape.shape_transform, model_matrix);
}

#[test]
fn test_update_works_with_non_default_minimum_and_maximum_bounds() {
    let mut shape = VoxelBoxShape::new();
    let translation = Cartesian3::new(1.0, 2.0, 3.0);
    let scale = Cartesian3::new(2.0, 3.0, 4.0);
    let rotation = Quaternion::IDENTITY;
    let model_matrix = Matrix4::from_translation_quaternion_rotation_scale(
        translation,
        rotation,
        scale,
    );
    let min_bounds = Cartesian3::new(-0.75, -0.75, -0.75);
    let max_bounds = Cartesian3::new(-0.25, -0.25, -0.25);

    let visible = shape
        .update(
            Some(model_matrix),
            Some(min_bounds),
            Some(max_bounds),
            Some(min_bounds),
            Some(max_bounds),
        )
        .unwrap();

    let expected_translation = Cartesian3::new(0.0, 0.5, 1.0);
    let expected_scale = Cartesian3::new(0.5, 0.75, 1.0);
    let expected_model_matrix = Matrix4::from_translation_quaternion_rotation_scale(
        expected_translation,
        rotation,
        expected_scale,
    );

    let expected_center = DVec3::new(expected_translation.x, expected_translation.y, expected_translation.z);
    let expected_half_axes = DMat3::from_diagonal(DVec3::new(expected_scale.x, expected_scale.y, expected_scale.z));
    let expected_obb = OrientedBoundingBox::new(expected_center, expected_half_axes);
    let expected_bs = BoundingSphere::new(expected_center, expected_scale.magnitude());

    assert!(visible);
    assert_eq!(shape.oriented_bounding_box, expected_obb);
    assert_eq!(shape.bounding_sphere, expected_bs);
    assert_eq!(shape.bound_transform, expected_model_matrix);
    assert_eq!(shape.shape_transform, model_matrix);
}

#[test]
fn test_update_is_invisible_with_zero_scale_for_two_or_more_components() {
    let mut shape = VoxelBoxShape::new();
    let min_bounds = VoxelBoxShape::DEFAULT_MIN_BOUNDS;
    let max_bounds = VoxelBoxShape::DEFAULT_MAX_BOUNDS;
    let translation = Cartesian3::new(1.0, 2.0, 3.0);
    let rotation = Quaternion::IDENTITY;

    // 0 scale for X and Y
    let scale = Cartesian3::new(0.0, 0.0, 2.0);
    let model_matrix = Matrix4::from_translation_quaternion_rotation_scale(translation, rotation, scale);
    let visible = shape.update(Some(model_matrix), Some(min_bounds), Some(max_bounds), None, None).unwrap();
    assert!(!visible);

    // 0 scale for X and Z
    let scale = Cartesian3::new(0.0, 2.0, 0.0);
    let model_matrix = Matrix4::from_translation_quaternion_rotation_scale(translation, rotation, scale);
    let visible = shape.update(Some(model_matrix), Some(min_bounds), Some(max_bounds), None, None).unwrap();
    assert!(!visible);

    // 0 scale for Y and Z
    let scale = Cartesian3::new(2.0, 0.0, 0.0);
    let model_matrix = Matrix4::from_translation_quaternion_rotation_scale(translation, rotation, scale);
    let visible = shape.update(Some(model_matrix), Some(min_bounds), Some(max_bounds), None, None).unwrap();
    assert!(!visible);

    // 0 scale for all
    let scale = Cartesian3::new(0.0, 0.0, 0.0);
    let model_matrix = Matrix4::from_translation_quaternion_rotation_scale(translation, rotation, scale);
    let visible = shape.update(Some(model_matrix), Some(min_bounds), Some(max_bounds), None, None).unwrap();
    assert!(!visible);
}

#[test]
fn test_update_is_visible_with_zero_bounds_for_one_component() {
    let mut shape = VoxelBoxShape::new();
    let translation = Cartesian3::ZERO;
    let rotation = Quaternion::IDENTITY;
    let scale = Cartesian3::new(1.0, 1.0, 1.0);
    let model_matrix = Matrix4::from_translation_quaternion_rotation_scale(translation, rotation, scale);

    let clip_min = Cartesian3::new(-1.0, -1.0, -1.0);
    let clip_max = Cartesian3::new(1.0, 1.0, 1.0);

    // 0 in X bound
    let min_bounds = Cartesian3::new(0.0, -1.0, -1.0);
    let max_bounds = Cartesian3::new(0.0, 1.0, 1.0);
    let visible = shape.update(Some(model_matrix), Some(min_bounds), Some(max_bounds), Some(clip_min), Some(clip_max)).unwrap();
    assert!(visible);
    assert_eq!(shape.bound_transform.get_scale(), Cartesian3::new(0.0, 1.0, 1.0));

    // 0 in Y bound
    let min_bounds = Cartesian3::new(-1.0, 0.0, -1.0);
    let max_bounds = Cartesian3::new(1.0, 0.0, 1.0);
    let visible = shape.update(Some(model_matrix), Some(min_bounds), Some(max_bounds), Some(clip_min), Some(clip_max)).unwrap();
    assert!(visible);
    assert_eq!(shape.bound_transform.get_scale(), Cartesian3::new(1.0, 0.0, 1.0));

    // 0 in Z bound
    let min_bounds = Cartesian3::new(-1.0, -1.0, 0.0);
    let max_bounds = Cartesian3::new(1.0, 1.0, 0.0);
    let visible = shape.update(Some(model_matrix), Some(min_bounds), Some(max_bounds), Some(clip_min), Some(clip_max)).unwrap();
    assert!(visible);
    assert_eq!(shape.bound_transform.get_scale(), Cartesian3::new(1.0, 1.0, 0.0));
}

#[test]
fn test_update_is_invisible_with_zero_bounds_for_two_or_more_components() {
    let mut shape = VoxelBoxShape::new();
    let model_matrix = Matrix4::IDENTITY;

    // 0 in X and Y
    let min_bounds = Cartesian3::new(0.0, 0.0, -1.0);
    let max_bounds = Cartesian3::new(0.0, 0.0, 1.0);
    assert!(!shape.update(Some(model_matrix), Some(min_bounds), Some(max_bounds), None, None).unwrap());

    // 0 in X, Y, Z
    let min_bounds = Cartesian3::ZERO;
    let max_bounds = Cartesian3::ZERO;
    assert!(!shape.update(Some(model_matrix), Some(min_bounds), Some(max_bounds), None, None).unwrap());
}

#[test]
fn test_update_is_invisible_when_minimum_bounds_exceed_maximum_bounds() {
    let mut shape = VoxelBoxShape::new();
    let model_matrix = Matrix4::IDENTITY;

    let clip_min = Cartesian3::new(-1.0, -1.0, -1.0);
    let clip_max = Cartesian3::new(2.0, 2.0, 2.0);

    // Exceeds X
    let min_bounds = Cartesian3::new(1.0, -1.0, -1.0);
    let max_bounds = Cartesian3::new(0.9, 1.0, 1.0);
    assert!(!shape.update(Some(model_matrix), Some(min_bounds), Some(max_bounds), Some(clip_min), Some(clip_max)).unwrap());

    // Exceeds Y
    let min_bounds = Cartesian3::new(-1.0, 1.0, -1.0);
    let max_bounds = Cartesian3::new(1.0, 0.9, 1.0);
    assert!(!shape.update(Some(model_matrix), Some(min_bounds), Some(max_bounds), Some(clip_min), Some(clip_max)).unwrap());

    // Exceeds Z
    let min_bounds = Cartesian3::new(-1.0, -1.0, 1.0);
    let max_bounds = Cartesian3::new(1.0, 1.0, 0.9);
    assert!(!shape.update(Some(model_matrix), Some(min_bounds), Some(max_bounds), Some(clip_min), Some(clip_max)).unwrap());
}

#[test]
fn test_update_throws_with_missing_parameters() {
    let mut shape = VoxelBoxShape::new();
    let min_bounds = VoxelBoxShape::DEFAULT_MIN_BOUNDS;
    let max_bounds = VoxelBoxShape::DEFAULT_MAX_BOUNDS;

    assert!(shape.update(None, Some(min_bounds), Some(max_bounds), None, None).is_err());
    assert!(shape.update(Some(Matrix4::IDENTITY), None, Some(max_bounds), None, None).is_err());
    assert!(shape.update(Some(Matrix4::IDENTITY), Some(min_bounds), None, None, None).is_err());
}

#[test]
fn test_compute_oriented_bounding_box_for_tile_works_for_root_tile() {
    let mut shape = VoxelBoxShape::new();
    shape.update(
        Some(Matrix4::IDENTITY),
        Some(VoxelBoxShape::DEFAULT_MIN_BOUNDS),
        Some(VoxelBoxShape::DEFAULT_MAX_BOUNDS),
        None,
        None,
    ).unwrap();

    let obb = shape.compute_oriented_bounding_box_for_tile(0, 0, 0, 0);
    assert_eq!(obb, shape.oriented_bounding_box);
}

#[test]
fn test_compute_oriented_bounding_box_for_tile_works_for_children_of_root_tile() {
    let mut shape = VoxelBoxShape::new();
    shape.update(
        Some(Matrix4::IDENTITY),
        Some(VoxelBoxShape::DEFAULT_MIN_BOUNDS),
        Some(VoxelBoxShape::DEFAULT_MAX_BOUNDS),
        None,
        None,
    ).unwrap();

    let expected_scale = DVec3::splat(0.5);

    // Child (0, 0, 0)
    let obb = shape.compute_oriented_bounding_box_for_tile(1, 0, 0, 0);
    assert_eq!(obb.center, DVec3::new(-0.5, -0.5, -0.5));
    assert_eq!(obb.half_axes, DMat3::from_diagonal(expected_scale));

    // Child (1, 0, 0)
    let obb = shape.compute_oriented_bounding_box_for_tile(1, 1, 0, 0);
    assert_eq!(obb.center, DVec3::new(0.5, -0.5, -0.5));
    assert_eq!(obb.half_axes, DMat3::from_diagonal(expected_scale));

    // Child (0, 1, 0)
    let obb = shape.compute_oriented_bounding_box_for_tile(1, 0, 1, 0);
    assert_eq!(obb.center, DVec3::new(-0.5, 0.5, -0.5));
    assert_eq!(obb.half_axes, DMat3::from_diagonal(expected_scale));

    // Child (0, 0, 1)
    let obb = shape.compute_oriented_bounding_box_for_tile(1, 0, 0, 1);
    assert_eq!(obb.center, DVec3::new(-0.5, -0.5, 0.5));
    assert_eq!(obb.half_axes, DMat3::from_diagonal(expected_scale));

    // Child (1, 1, 1)
    let obb = shape.compute_oriented_bounding_box_for_tile(1, 1, 1, 1);
    assert_eq!(obb.center, DVec3::new(0.5, 0.5, 0.5));
    assert_eq!(obb.half_axes, DMat3::from_diagonal(expected_scale));
}

#[test]
fn test_compute_oriented_bounding_box_for_sample_gives_expected_result() {
    let mut shape = VoxelBoxShape::new();
    shape.update(
        Some(Matrix4::IDENTITY),
        Some(VoxelBoxShape::DEFAULT_MIN_BOUNDS),
        Some(VoxelBoxShape::DEFAULT_MAX_BOUNDS),
        None,
        None,
    ).unwrap();

    let spatial_node = SpatialNode::new(
        0,
        0,
        0,
        0,
        &shape,
        Cartesian3::new(10.0, 10.0, 10.0),
    );

    let tile_dimensions = Cartesian3::new(8.0, 8.0, 8.0);
    let tile_uv = Cartesian3::new(0.5, 0.5, 0.5);

    let sample_obb = shape.compute_oriented_bounding_box_for_sample(
        &spatial_node,
        tile_dimensions,
        tile_uv,
    );

    let expected_center = DVec3::new(0.125, 0.125, 0.125);
    let expected_half_axes = DMat3::from_diagonal(DVec3::new(0.125, 0.125, 0.125));

    assert_eq!(sample_obb.center, expected_center);
    assert_eq!(sample_obb.half_axes, expected_half_axes);
}

#[test]
fn test_compute_oriented_bounding_box_throws_with_missing_parameters() {
    let mut shape = VoxelBoxShape::new();
    shape.update(
        Some(Matrix4::IDENTITY),
        Some(VoxelBoxShape::DEFAULT_MIN_BOUNDS),
        Some(VoxelBoxShape::DEFAULT_MAX_BOUNDS),
        None,
        None,
    ).unwrap();

    assert!(shape.compute_oriented_bounding_box_for_tile_checked(None, Some(0), Some(0), Some(0)).is_err());
    assert!(shape.compute_oriented_bounding_box_for_sample_checked(None, Some(Cartesian3::new(1.0, 1.0, 1.0)), Some(Cartesian3::ZERO)).is_err());
}
