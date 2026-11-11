//! Tests for VoxelEllipsoidShape matching CesiumJS `Scene/VoxelEllipsoidShapeSpec.js` (Apache-2.0).

use std::f64::consts::{FRAC_PI_2, FRAC_PI_4, PI};

use cesium_math::{BoundingSphere, Cartesian3, Matrix4, OrientedBoundingBox, Quaternion};
use cesium_renderer::spatial_node::SpatialNode;
use cesium_renderer::voxel_box_shape::VoxelShape;
use cesium_renderer::voxel_ellipsoid_shape::VoxelEllipsoidShape;
use glam::{DMat3, DVec3};

#[test]
fn test_constructs() {
    let shape = VoxelEllipsoidShape::new();
    assert_eq!(shape.shape_transform, Matrix4::IDENTITY);
}

#[test]
fn test_update_works_with_model_matrix() {
    let mut shape = VoxelEllipsoidShape::new();
    let translation = Cartesian3::new(1.0, 2.0, 3.0);
    let scale = Cartesian3::new(2.0, 2.0, 2.0);
    let angle = FRAC_PI_4;
    let rotation = Quaternion::from_axis_angle(Cartesian3::UNIT_Z, angle);

    let model_matrix = Matrix4::from_translation_quaternion_rotation_scale(
        translation,
        rotation,
        scale,
    );

    let min_bounds = Cartesian3::new(-PI, -FRAC_PI_2, 0.0);
    let max_bounds = Cartesian3::new(PI, FRAC_PI_2, 100000.0);
    let max_height = max_bounds.z;

    // In CesiumJS, Matrix3.fromRowMajorArray:
    // Row 0: 0.0, 0.0, scale.z + maxHeight
    // Row 1: (scale.x + maxHeight) * cos(angle), -(scale.y + maxHeight) * sin(angle), 0.0
    // Row 2: (scale.x + maxHeight) * sin(angle), (scale.y + maxHeight) * cos(angle), 0.0
    let expected_center = DVec3::new(translation.x, translation.y, translation.z);
    let expected_half_axes = DMat3::from_cols(
        DVec3::new(0.0, (scale.x + max_height) * angle.cos(), (scale.x + max_height) * angle.sin()),
        DVec3::new(0.0, -(scale.y + max_height) * angle.sin(), (scale.y + max_height) * angle.cos()),
        DVec3::new(scale.z + max_height, 0.0, 0.0),
    );

    let expected_obb = OrientedBoundingBox::new(expected_center, expected_half_axes);
    let expected_bs = BoundingSphere::from_oriented_bounding_box(&expected_obb).unwrap();

    let visible = shape
        .update(Some(model_matrix), Some(min_bounds), Some(max_bounds), None, None)
        .unwrap();

    assert!(visible);
    assert_eq!(shape.oriented_bounding_box.center, expected_center);

    for i in 0..3 {
        let diff = (shape.oriented_bounding_box.half_axes.col(i) - expected_half_axes.col(i)).length();
        assert!(diff < 1e-8, "col {i} mismatch: {diff}");
    }

    assert_eq!(shape.bounding_sphere.center, expected_bs.center);
    assert!((shape.bounding_sphere.radius - expected_bs.radius).abs() < 1e-8);
    assert_eq!(shape.shape_transform.get_translation(), translation);
}

#[test]
fn test_compute_oriented_bounding_box_for_tile() {
    let mut shape = VoxelEllipsoidShape::new();
    let model_matrix = Matrix4::IDENTITY;

    let min_bounds = Cartesian3::new(-PI, -FRAC_PI_2, -0.5);
    let max_bounds = Cartesian3::new(PI, FRAC_PI_2, 0.0);
    shape
        .update(Some(model_matrix), Some(min_bounds), Some(max_bounds), None, None)
        .unwrap();

    let result = shape.compute_oriented_bounding_box_for_tile(0, 0, 0, 0);
    assert_eq!(result.center, DVec3::ZERO);

    // CesiumJS row-major: Matrix3(0, 0, 1,  1, 0, 0,  0, 1, 0)
    // Row 0: [0, 0, 1], Row 1: [1, 0, 0], Row 2: [0, 1, 0]
    // Col 0: [0, 1, 0], Col 1: [0, 0, 1], Col 2: [1, 0, 0]
    let expected_half_axes = DMat3::from_cols(
        DVec3::new(0.0, 1.0, 0.0),
        DVec3::new(0.0, 0.0, 1.0),
        DVec3::new(1.0, 0.0, 0.0),
    );

    for i in 0..3 {
        let diff = (result.half_axes.col(i) - expected_half_axes.col(i)).length();
        assert!(diff < 1e-12, "col {i} diff: {diff}");
    }
}

#[test]
fn test_compute_oriented_bounding_box_for_tile_throws_with_missing_parameters() {
    let mut shape = VoxelEllipsoidShape::new();
    shape
        .update(
            Some(Matrix4::IDENTITY),
            Some(VoxelEllipsoidShape::DEFAULT_MIN_BOUNDS),
            Some(VoxelEllipsoidShape::DEFAULT_MAX_BOUNDS),
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
    let mut shape = VoxelEllipsoidShape::new();
    let model_matrix = Matrix4::IDENTITY;

    let min_bounds = Cartesian3::new(-PI, -FRAC_PI_2, -1.0);
    let max_bounds = Cartesian3::new(PI, FRAC_PI_2, 0.0);
    shape
        .update(Some(model_matrix), Some(min_bounds), Some(max_bounds), None, None)
        .unwrap();

    let tile_dimensions = Cartesian3::new(16.0, 8.0, 8.0);
    let padded_dimensions = Cartesian3::new(18.0, 10.0, 10.0);
    let spatial_node = SpatialNode::new(0, 0, 0, 0, &shape, padded_dimensions);

    let tile_uv = Cartesian3::new(0.5, 0.5, 0.5);
    let sample_obb = shape.compute_oriented_bounding_box_for_sample(
        &spatial_node,
        tile_dimensions,
        tile_uv,
    );

    let center_lon = PI / 16.0;
    let center_lat = PI / 16.0;
    let center_r = 0.553;
    let expected_center = DVec3::new(
        center_r * center_lon.cos() * center_lat.cos(),
        center_r * center_lon.sin() * center_lat.cos(),
        center_r * center_lat.sin(),
    );

    assert!((sample_obb.center - expected_center).length() < 0.05);
}

#[test]
fn test_compute_oriented_bounding_box_for_sample_throws_with_missing_parameters() {
    let mut shape = VoxelEllipsoidShape::new();
    shape
        .update(
            Some(Matrix4::IDENTITY),
            Some(VoxelEllipsoidShape::DEFAULT_MIN_BOUNDS),
            Some(VoxelEllipsoidShape::DEFAULT_MAX_BOUNDS),
            None,
            None,
        )
        .unwrap();

    assert!(shape.compute_oriented_bounding_box_for_sample_checked(None, Some(Cartesian3::new(16.0, 8.0, 8.0)), Some(Cartesian3::ZERO)).is_err());
    assert!(shape.compute_oriented_bounding_box_for_sample_checked(None, None, Some(Cartesian3::ZERO)).is_err());
}
