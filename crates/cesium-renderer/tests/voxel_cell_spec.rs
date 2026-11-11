//! Tests for VoxelCell matching CesiumJS `Scene/VoxelCellSpec.js` (Apache-2.0).

use std::collections::HashMap;

use cesium_math::{Cartesian3, Matrix4};
use cesium_renderer::spatial_node::SpatialNode;
use cesium_renderer::voxel_box_shape::VoxelBoxShape;
use cesium_renderer::voxel_cell::VoxelCell;
use glam::{DMat3, DVec3};

#[test]
fn test_constructs() {
    let tile_index = 0;
    let sample_index = 0;
    let primitive_id = 42;
    let cell = VoxelCell::new(primitive_id, tile_index, sample_index);

    assert_eq!(cell.primitive_id, primitive_id);
    assert_eq!(cell.tile_index, tile_index);
    assert_eq!(cell.sample_index, sample_index);
}

#[test]
fn test_constructs_using_factory_function() {
    let mut shape = VoxelBoxShape::new();
    shape
        .update(
            Some(Matrix4::IDENTITY),
            Some(VoxelBoxShape::DEFAULT_MIN_BOUNDS),
            Some(VoxelBoxShape::DEFAULT_MAX_BOUNDS),
            None,
            None,
        )
        .unwrap();

    let padded_dimensions = Cartesian3::new(2.0, 2.0, 2.0);
    let spatial_node = SpatialNode::new(0, 0, 0, 0, &shape, padded_dimensions);

    let mut metadata = HashMap::new();
    metadata.insert("a".to_string(), vec![0.0]);

    let cell = VoxelCell::from_keyframe_node_details(
        Some(100),
        Some(0),
        Some(0),
        Some(&shape),
        Some(&spatial_node),
        Some(Cartesian3::new(2.0, 2.0, 2.0)),
        Some(Cartesian3::ZERO),
        Some(metadata),
    )
    .unwrap();

    assert_eq!(cell.primitive_id, 100);
    assert_eq!(cell.tile_index, 0);
    assert_eq!(cell.sample_index, 0);
}

#[test]
fn test_from_keyframe_node_throws_with_missing_parameters() {
    let mut shape = VoxelBoxShape::new();
    shape
        .update(
            Some(Matrix4::IDENTITY),
            Some(VoxelBoxShape::DEFAULT_MIN_BOUNDS),
            Some(VoxelBoxShape::DEFAULT_MAX_BOUNDS),
            None,
            None,
        )
        .unwrap();

    let spatial_node = SpatialNode::new(0, 0, 0, 0, &shape, Cartesian3::new(2.0, 2.0, 2.0));

    assert!(VoxelCell::from_keyframe_node_details(
        None,
        Some(0),
        Some(0),
        Some(&shape),
        Some(&spatial_node),
        None,
        None,
        None,
    )
    .is_err());

    assert!(VoxelCell::from_keyframe_node_details(
        Some(1),
        Some(0),
        None,
        Some(&shape),
        Some(&spatial_node),
        None,
        None,
        None,
    )
    .is_err());

    assert!(VoxelCell::from_keyframe_node_details(
        None,
        Some(0),
        Some(0),
        None::<&VoxelBoxShape>,
        Some(&spatial_node),
        None,
        None,
        None,
    )
    .is_err());
}

#[test]
fn test_reports_sample_metadata_names_and_properties() {
    let mut shape = VoxelBoxShape::new();
    shape
        .update(
            Some(Matrix4::IDENTITY),
            Some(VoxelBoxShape::DEFAULT_MIN_BOUNDS),
            Some(VoxelBoxShape::DEFAULT_MAX_BOUNDS),
            None,
            None,
        )
        .unwrap();

    let spatial_node = SpatialNode::new(0, 0, 0, 0, &shape, Cartesian3::new(2.0, 2.0, 2.0));

    let mut metadata = HashMap::new();
    metadata.insert("a".to_string(), vec![42.0]);

    let cell = VoxelCell::from_keyframe_node_details(
        Some(1),
        Some(0),
        Some(0),
        Some(&shape),
        Some(&spatial_node),
        Some(Cartesian3::new(2.0, 2.0, 2.0)),
        Some(Cartesian3::ZERO),
        Some(metadata),
    )
    .unwrap();

    assert_eq!(cell.get_names(), vec!["a"]);
    assert!(cell.has_property("a"));
    assert!(!cell.has_property("b"));
    assert_eq!(cell.get_property("a"), Some([42.0].as_slice()));
    assert_eq!(cell.get_property("b"), None);
}

#[test]
fn test_computes_bounding_box_for_the_sample() {
    let mut shape = VoxelBoxShape::new();
    shape
        .update(
            Some(Matrix4::IDENTITY),
            Some(VoxelBoxShape::DEFAULT_MIN_BOUNDS),
            Some(VoxelBoxShape::DEFAULT_MAX_BOUNDS),
            None,
            None,
        )
        .unwrap();

    let tile_dimensions = Cartesian3::new(2.0, 2.0, 2.0);
    let padded_dimensions = Cartesian3::new(2.0, 2.0, 2.0);
    let spatial_node = SpatialNode::new(0, 0, 0, 0, &shape, padded_dimensions);

    let cell = VoxelCell::from_keyframe_node_details(
        Some(1),
        Some(0),
        Some(7),
        Some(&shape),
        Some(&spatial_node),
        Some(tile_dimensions),
        Some(Cartesian3::ZERO),
        None,
    )
    .unwrap();

    let expected_center = DVec3::new(0.5, 0.5, 0.5);
    let expected_half_axes = DMat3::from_diagonal(DVec3::splat(0.5));

    assert_eq!(cell.oriented_bounding_box.center, expected_center);
    assert_eq!(cell.oriented_bounding_box.half_axes, expected_half_axes);
}
