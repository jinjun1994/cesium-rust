use std::sync::Arc;

use cesium_math::{
    Cartesian3, CullingVolume, Matrix4, Plane, EPSILON7,
};
use cesium_renderer::{
    KeyframeLoadState, MetadataComponentType, MockVoxelProvider, PrimitiveRenderFrameState,
    VoxelEllipsoidShape, VoxelPrimitive, VoxelPrimitiveOptions, VoxelShape, VoxelShapeType,
    VoxelTraversal,
};
use glam::DVec3;

fn create_test_traversal() -> (VoxelPrimitive, VoxelTraversal) {
    let provider = Arc::new(MockVoxelProvider {
        shape_type: VoxelShapeType::Ellipsoid,
        dimensions: Cartesian3::new(4.0, 4.0, 4.0),
        tile_count: 8,
        minimum_values: vec![0.0],
        maximum_values: vec![1.0],
        shape_transform: None,
        global_transform: None,
    });
    let primitive = VoxelPrimitive::new(Some(VoxelPrimitiveOptions {
        provider: Some(provider),
        ..Default::default()
    }));
    let traversal = VoxelTraversal::new(&primitive, 3, Some(256));
    (primitive, traversal)
}

#[test]
fn test_voxel_traversal_constructs_with_arguments() {
    let (_, traversal) = create_test_traversal();
    assert_eq!(traversal.megatextures.len(), 1);
    assert_eq!(
        traversal.megatextures[0].component_type,
        MetadataComponentType::Float32
    );
}

#[test]
fn test_voxel_traversal_recomputes_bounding_volume_when_shape_moves() {
    let (_, mut traversal) = create_test_traversal();
    let old_obb = traversal.root_node.oriented_bounding_box;

    let translation = Cartesian3::new(1.0, 1.0, 1.0);
    let mut shape = VoxelEllipsoidShape::default();
    let model_matrix = Matrix4::from_translation(translation);
    shape
        .update(
            Some(model_matrix),
            Some(VoxelEllipsoidShape::DEFAULT_MIN_BOUNDS),
            Some(VoxelEllipsoidShape::DEFAULT_MAX_BOUNDS),
            Some(VoxelEllipsoidShape::DEFAULT_MIN_BOUNDS),
            Some(VoxelEllipsoidShape::DEFAULT_MAX_BOUNDS),
        )
        .unwrap();

    traversal.primitive.model_matrix = model_matrix;
    let mut frame_state = PrimitiveRenderFrameState::default();
    traversal.update(&mut frame_state, 0, true, false);

    let new_obb = shape.compute_oriented_bounding_box_for_tile(0, 0, 0, 0);
    assert_ne!(old_obb, new_obb);
    assert!((new_obb.center.x - translation.x).abs() < 1e-6);
    assert!((new_obb.center.y - translation.y).abs() < 1e-6);
    assert!((new_obb.center.z - translation.z).abs() < 1e-6);
}

#[test]
fn test_voxel_traversal_computes_screen_space_error_for_root_tile() {
    let (_, mut traversal) = create_test_traversal();
    let camera_position = Cartesian3::new(-6378000.0, -6378000.0, -6378000.0);
    let screen_space_error_multiplier = 500.0;

    let root_node = &mut traversal.root_node;
    root_node.compute_screen_space_error(camera_position, screen_space_error_multiplier);

    let dist_sq = root_node
        .oriented_bounding_box
        .distance_squared_to(DVec3::new(
            camera_position.x,
            camera_position.y,
            camera_position.z,
        ))
        .unwrap_or(0.0);
    let distance = dist_sq.sqrt().max(EPSILON7);
    let expected_sse = screen_space_error_multiplier * (root_node.approximate_voxel_size / distance);

    assert!((root_node.screen_space_error - expected_sse).abs() < 1e-7);
}

#[test]
fn test_voxel_traversal_computes_visibility_for_root_tile() {
    let (_, traversal) = create_test_traversal();
    let root_node = &traversal.root_node;

    // A culling volume facing the origin containing root node
    let inside_plane = Plane::new(DVec3::new(0.0, 0.0, 1.0), 1e8).unwrap();
    let culling_volume_inside = CullingVolume::new(vec![inside_plane]);
    let vis_inside = root_node.visibility(
        &culling_volume_inside,
        CullingVolume::MASK_INDETERMINATE,
    );
    assert_eq!(vis_inside, CullingVolume::MASK_INSIDE);

    // A culling volume far away where root node is outside
    let outside_plane = Plane::new(DVec3::new(0.0, 0.0, 1.0), -1e8).unwrap();
    let culling_volume_outside = CullingVolume::new(vec![outside_plane]);
    let vis_outside = root_node.visibility(
        &culling_volume_outside,
        CullingVolume::MASK_INDETERMINATE,
    );
    assert_eq!(vis_outside, CullingVolume::MASK_OUTSIDE);
}

#[test]
fn test_voxel_traversal_destroys() {
    let (_, mut traversal) = create_test_traversal();
    assert!(!traversal.is_destroyed());
    traversal.destroy();
    assert!(traversal.is_destroyed());
    assert_eq!(traversal.texture_memory_byte_length, 0);
}

#[test]
fn test_voxel_traversal_shows_texture_memory_allocation_statistic() {
    let (_, mut traversal) = create_test_traversal();
    assert_eq!(traversal.texture_memory_byte_length, 32);
    traversal.destroy();
    assert_eq!(traversal.texture_memory_byte_length, 0);
}

#[test]
fn test_voxel_traversal_loads_tiles_into_megatexture() {
    let (_, mut traversal) = create_test_traversal();
    let mut frame_state = PrimitiveRenderFrameState::default();
    traversal.update(&mut frame_state, 0, true, false);

    assert_eq!(traversal.megatextures[0].occupied_count, 1);
    assert_eq!(traversal.texture_memory_byte_length, 32);
    assert!(traversal.initial_tiles_loaded);
}

#[test]
fn test_voxel_traversal_tile_failed_event() {
    let (_, mut traversal) = create_test_traversal();
    traversal.calculate_statistics = true;
    let mut frame_state = PrimitiveRenderFrameState::default();
    traversal.update(&mut frame_state, 0, true, false);
    traversal.update(&mut frame_state, 0, true, false);
    traversal.update(&mut frame_state, 0, true, false);

    assert_eq!(traversal.primitive.statistics.visited, 3);
}

#[test]
fn test_voxel_traversal_finds_keyframe_node_with_expected_metadata_values() {
    let (_, mut traversal) = create_test_traversal();
    let mut frame_state = PrimitiveRenderFrameState::default();
    traversal.update(&mut frame_state, 0, true, false);

    let keyframe_node = traversal.find_keyframe_node(0).expect("keyframe node exists");
    assert_eq!(keyframe_node.state, KeyframeLoadState::Loaded);

    let content = keyframe_node.content.as_ref().unwrap();
    let metadata_arr = content["metadata"][0].as_array().unwrap();
    let vals: Vec<f64> = metadata_arr.iter().map(|v| v.as_f64().unwrap()).collect();
    assert_eq!(vals, vec![0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 1.0]);
}
