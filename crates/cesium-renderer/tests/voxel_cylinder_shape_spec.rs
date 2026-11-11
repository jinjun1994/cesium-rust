//! Tests for VoxelCylinderShape matching CesiumJS `Scene/VoxelCylinderShapeSpec.js` (Apache-2.0).

use std::f64::consts::{FRAC_PI_2, FRAC_PI_4, PI};

use cesium_math::{BoundingSphere, Cartesian3, Matrix4, OrientedBoundingBox, Quaternion};
use cesium_renderer::spatial_node::SpatialNode;
use cesium_renderer::voxel_box_shape::VoxelShape;
use cesium_renderer::voxel_cylinder_shape::VoxelCylinderShape;
use glam::{DMat3, DVec3};

#[test]
fn test_constructs() {
    let shape = VoxelCylinderShape::new();
    assert_eq!(shape.shape_transform, Matrix4::IDENTITY);
}

#[test]
fn test_update_works_with_model_matrix() {
    let mut shape = VoxelCylinderShape::new();

    let translation = Cartesian3::new(1.0, 2.0, 3.0);
    let scale = Cartesian3::new(2.0, 3.0, 4.0);
    let angle = FRAC_PI_4;
    let rotation = Quaternion::from_axis_angle(Cartesian3::UNIT_Z, angle);
    let model_matrix = Matrix4::from_translation_quaternion_rotation_scale(
        translation,
        rotation,
        scale,
    );
    let min_bounds = VoxelCylinderShape::DEFAULT_MIN_BOUNDS;
    let max_bounds = VoxelCylinderShape::DEFAULT_MAX_BOUNDS;

    let visible = shape
        .update(Some(model_matrix), Some(min_bounds), Some(max_bounds), None, None)
        .unwrap();

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

    assert!(visible);
    assert_eq!(shape.oriented_bounding_box.center, expected_obb.center);

    for i in 0..3 {
        let diff = (shape.oriented_bounding_box.half_axes.col(i) - expected_obb.half_axes.col(i)).length();
        assert!(diff < 1e-12, "col {i} mismatch: diff = {diff}");
    }

    assert_eq!(shape.bounding_sphere.center, expected_bs.center);
    assert!((shape.bounding_sphere.radius - expected_bs.radius).abs() < 1e-12);
    assert_eq!(shape.bound_transform, model_matrix);
    assert_eq!(shape.shape_transform, model_matrix);
}

#[test]
fn test_update_works_with_non_default_minimum_and_maximum_bounds() {
    let mut shape = VoxelCylinderShape::new();
    let translation = Cartesian3::new(1.0, 2.0, 3.0);
    let scale = Cartesian3::new(2.0, 3.0, 4.0);
    let rotation = Quaternion::IDENTITY;
    let model_matrix = Matrix4::from_translation_quaternion_rotation_scale(
        translation,
        rotation,
        scale,
    );

    let min_radius = 0.25;
    let max_radius = 0.75;
    let min_angle = -PI;
    let max_angle = 0.0;
    let min_height = -0.5;
    let max_height = 0.5;
    let min_bounds = Cartesian3::new(min_radius, min_angle, min_height);
    let max_bounds = Cartesian3::new(max_radius, max_angle, max_height);

    let visible = shape
        .update(Some(model_matrix), Some(min_bounds), Some(max_bounds), None, None)
        .unwrap();

    let expected_min_x = translation.x - max_radius * scale.x;
    let expected_max_x = translation.x + max_radius * scale.x;
    let expected_min_y = translation.y - max_radius * scale.y;
    let expected_max_y = translation.y;
    let expected_min_z = translation.z + min_height * scale.z;
    let expected_max_z = translation.z + max_height * scale.z;

    let expected_scale = Cartesian3::new(
        0.5 * (expected_max_y - expected_min_y),
        0.5 * (expected_max_x - expected_min_x),
        0.5 * (expected_max_z - expected_min_z),
    );
    let expected_translation = DVec3::new(
        0.5 * (expected_max_x + expected_min_x),
        0.5 * (expected_max_y + expected_min_y),
        0.5 * (expected_max_z + expected_min_z),
    );

    let expected_rot = DMat3::from_rotation_z(-FRAC_PI_2);
    let expected_half_axes = expected_rot * DMat3::from_diagonal(DVec3::new(expected_scale.x, expected_scale.y, expected_scale.z));

    assert!(visible);
    assert!((shape.oriented_bounding_box.center - expected_translation).length() < 1e-12);
    for i in 0..3 {
        let diff = (shape.oriented_bounding_box.half_axes.col(i) - expected_half_axes.col(i)).length();
        assert!(diff < 1e-12, "col {i} mismatch: diff = {diff}");
    }
}

#[test]
fn test_update_works_with_bounds_crossing_180th_meridian() {
    let mut shape = VoxelCylinderShape::new();
    let model_matrix = Matrix4::IDENTITY;

    let min_angle = FRAC_PI_2;
    let max_angle = -FRAC_PI_2;
    let min_bounds = Cartesian3::new(0.0, min_angle, -1.0);
    let max_bounds = Cartesian3::new(1.0, max_angle, 1.0);

    let visible = shape
        .update(Some(model_matrix), Some(min_bounds), Some(max_bounds), None, None)
        .unwrap();

    let expected_translation = DVec3::new(-0.5, 0.0, 0.0);
    let expected_rot = DMat3::from_rotation_z(PI);
    let expected_scale = DVec3::new(0.5, 1.0, 1.0);
    let expected_half_axes = expected_rot * DMat3::from_diagonal(expected_scale);

    assert!(visible);
    assert!((shape.oriented_bounding_box.center - expected_translation).length() < 1e-12);
    for i in 0..3 {
        let diff = (shape.oriented_bounding_box.half_axes.col(i) - expected_half_axes.col(i)).length();
        assert!(diff < 1e-12, "col {i} mismatch: diff = {diff}");
    }
}

#[test]
fn test_compute_oriented_bounding_box_for_tile() {
    let mut shape = VoxelCylinderShape::new();
    let translation = Cartesian3::new(1.0, 2.0, 3.0);
    let scale = Cartesian3::new(2.0, 3.0, 4.0);
    let rotation = Quaternion::IDENTITY;
    let model_matrix = Matrix4::from_translation_quaternion_rotation_scale(
        translation,
        rotation,
        scale,
    );

    let min_radius = 0.25;
    let max_radius = 0.75;
    let min_angle = -PI;
    let max_angle = 0.0;
    let min_height = -0.5;
    let max_height = 0.5;
    let min_bounds = Cartesian3::new(min_radius, min_angle, min_height);
    let max_bounds = Cartesian3::new(max_radius, max_angle, max_height);
    shape
        .update(Some(model_matrix), Some(min_bounds), Some(max_bounds), None, None)
        .unwrap();

    let result = shape.compute_oriented_bounding_box_for_tile(0, 0, 0, 0);
    assert_eq!(result.center.x, 1.0);
    assert_eq!(result.center.y, 0.875);
    assert_eq!(result.center.z, 3.0);

    // CesiumJS Matrix3 constructor takes row-major elements:
    // new Matrix3(0, 1.5, 0,  -1.125, 0, 0,  0, 0, 2)
    // In column-major, column 0 is [0.0, -1.125, 0.0], column 1 is [1.5, 0.0, 0.0]
    let expected_half_axes = DMat3::from_cols(
        DVec3::new(0.0, -1.125, 0.0),
        DVec3::new(1.5, 0.0, 0.0),
        DVec3::new(0.0, 0.0, 2.0),
    );

    for i in 0..3 {
        let diff = (result.half_axes.col(i) - expected_half_axes.col(i)).length();
        assert!(diff < 1e-12, "half axis {i} diff: {diff}");
    }
}

#[test]
fn test_compute_oriented_bounding_box_for_tile_throws_with_missing_parameters() {
    let mut shape = VoxelCylinderShape::new();
    shape
        .update(
            Some(Matrix4::IDENTITY),
            Some(VoxelCylinderShape::DEFAULT_MIN_BOUNDS),
            Some(VoxelCylinderShape::DEFAULT_MAX_BOUNDS),
            None,
            None,
        )
        .unwrap();

    assert!(shape.compute_oriented_bounding_box_for_tile_checked(None, Some(0), Some(0), Some(0)).is_err());
    assert!(shape.compute_oriented_bounding_box_for_tile_checked(Some(0), None, Some(0), Some(0)).is_err());
    assert!(shape.compute_oriented_bounding_box_for_tile_checked(Some(0), Some(0), None, Some(0)).is_err());
    assert!(shape.compute_oriented_bounding_box_for_tile_checked(Some(0), Some(0), Some(0), None).is_err());
}

#[test]
fn test_compute_oriented_bounding_box_for_sample() {
    let mut shape = VoxelCylinderShape::new();
    shape
        .update(
            Some(Matrix4::IDENTITY),
            Some(VoxelCylinderShape::DEFAULT_MIN_BOUNDS),
            Some(VoxelCylinderShape::DEFAULT_MAX_BOUNDS),
            None,
            None,
        )
        .unwrap();

    let tile_dimensions = Cartesian3::new(8.0, 8.0, 8.0);
    let padded_dimensions = Cartesian3::new(10.0, 10.0, 10.0);
    let spatial_node = SpatialNode::new(0, 0, 0, 0, &shape, padded_dimensions);

    let tile_uv = Cartesian3::new(0.5, 0.5, 0.5);
    let sample_obb = shape.compute_oriented_bounding_box_for_sample(
        &spatial_node,
        tile_dimensions,
        tile_uv,
    );

    let center_angle = PI / 8.0;
    let center_radius = 0.5434699;
    let expected_center = DVec3::new(
        center_radius * center_angle.cos(),
        center_radius * center_angle.sin(),
        0.125,
    );

    assert!((sample_obb.center - expected_center).length() < 1e-6);

    // CesiumJS Matrix3 constructor takes row-major elements:
    // Row 0: 0.075324, -0.091529, 0.0
    // Row 1: 0.0312, 0.22097, 0.0
    // Row 2: 0.0, 0.0, 0.125
    let expected_half_axes = DMat3::from_cols(
        DVec3::new(0.075324, 0.0312, 0.0),
        DVec3::new(-0.091529, 0.22097, 0.0),
        DVec3::new(0.0, 0.0, 0.125),
    );

    for i in 0..3 {
        let diff = (sample_obb.half_axes.col(i) - expected_half_axes.col(i)).length();
        assert!(diff < 1e-4, "col {i} mismatch: {diff}");
    }
}

#[test]
fn test_compute_oriented_bounding_box_for_sample_throws_with_missing_parameters() {
    let mut shape = VoxelCylinderShape::new();
    shape
        .update(
            Some(Matrix4::IDENTITY),
            Some(VoxelCylinderShape::DEFAULT_MIN_BOUNDS),
            Some(VoxelCylinderShape::DEFAULT_MAX_BOUNDS),
            None,
            None,
        )
        .unwrap();

    assert!(shape.compute_oriented_bounding_box_for_sample_checked(None, Some(Cartesian3::new(8.0, 8.0, 8.0)), Some(Cartesian3::ZERO)).is_err());
    assert!(shape.compute_oriented_bounding_box_for_sample_checked(None, None, Some(Cartesian3::ZERO)).is_err());
}
