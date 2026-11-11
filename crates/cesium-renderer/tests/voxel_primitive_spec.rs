use std::sync::Arc;

use cesium_math::{Cartesian3, Matrix4};
use cesium_renderer::{
    CustomShader, CustomShaderOptions, MockVoxelProvider, PrimitiveRenderFrameState,
    VoxelPrimitive, VoxelPrimitiveOptions, VoxelShapeType,
};

#[test]
fn test_voxel_primitive_constructs_default() {
    let primitive = VoxelPrimitive::default();
    assert_eq!(primitive.model_matrix, Matrix4::IDENTITY);
    assert!(primitive.show);
    assert!(!primitive.ready);
    assert!(!primitive.is_destroyed());
}

#[test]
fn test_voxel_primitive_constructs_with_options() {
    let provider = Arc::new(MockVoxelProvider {
        shape_type: VoxelShapeType::Ellipsoid,
        dimensions: Cartesian3::new(4.0, 4.0, 4.0),
        tile_count: 8,
        minimum_values: vec![0.1, 0.2],
        maximum_values: vec![0.9, 1.0],
        shape_transform: None,
        global_transform: None,
    });

    let mut primitive = VoxelPrimitive::new(Some(VoxelPrimitiveOptions {
        provider: Some(provider.clone()),
        ..Default::default()
    }));

    let mut frame_state = PrimitiveRenderFrameState::default();
    primitive.update(&mut frame_state).unwrap();

    assert!(primitive.ready);
    assert_eq!(primitive.shape_type, provider.shape_type);
    assert_eq!(primitive.dimensions, provider.dimensions);
    assert_eq!(primitive.tile_count, provider.tile_count);
    assert_eq!(primitive.minimum_values, provider.minimum_values);
    assert_eq!(primitive.maximum_values, provider.maximum_values);
}

#[test]
fn test_voxel_primitive_loads_tiles_events() {
    let mut primitive = VoxelPrimitive::default();
    primitive.initial_tiles_loaded_listeners += 1;
    let mut frame_state = PrimitiveRenderFrameState::default();
    primitive.update(&mut frame_state).unwrap();
    assert!(primitive.initial_tiles_loaded);
    assert_eq!(primitive.initial_tiles_loaded_listeners, 1);
}

#[test]
fn test_voxel_primitive_initial_and_all_tiles_loaded_events() {
    let mut primitive = VoxelPrimitive::default();
    primitive.all_tiles_loaded_listeners += 1;
    primitive.initial_tiles_loaded_listeners += 1;
    let mut frame_state = PrimitiveRenderFrameState::default();
    primitive.update(&mut frame_state).unwrap();
    assert_eq!(primitive.all_tiles_loaded_listeners, 1);
    assert_eq!(primitive.initial_tiles_loaded_listeners, 1);
}

#[test]
fn test_voxel_primitive_statistics_updated_when_listeners_assigned() {
    let mut primitive = VoxelPrimitive::default();
    primitive.all_tiles_loaded_listeners += 1;
    primitive.initial_tiles_loaded_listeners += 1;
    let mut frame_state = PrimitiveRenderFrameState::default();
    primitive.update(&mut frame_state).unwrap();

    assert_eq!(primitive.statistics.number_of_tiles_with_content_ready, 1);
    assert_eq!(primitive.statistics.visited, 1);
    assert_eq!(primitive.statistics.textures_byte_length, 32);
}

#[test]
fn test_voxel_primitive_statistics_updated_when_flag_true() {
    let mut primitive = VoxelPrimitive::new(Some(VoxelPrimitiveOptions {
        calculate_statistics: Some(true),
        ..Default::default()
    }));
    let mut frame_state = PrimitiveRenderFrameState::default();
    primitive.update(&mut frame_state).unwrap();

    assert_eq!(primitive.statistics.number_of_tiles_with_content_ready, 1);
    assert_eq!(primitive.statistics.visited, 1);
}

#[test]
fn test_voxel_primitive_statistics_not_updated_when_flag_false() {
    let mut primitive = VoxelPrimitive::new(Some(VoxelPrimitiveOptions {
        calculate_statistics: Some(false),
        ..Default::default()
    }));
    let mut frame_state = PrimitiveRenderFrameState::default();
    primitive.update(&mut frame_state).unwrap();

    assert_eq!(primitive.statistics.number_of_tiles_with_content_ready, 0);
    assert_eq!(primitive.statistics.visited, 0);
}

#[test]
fn test_voxel_primitive_listeners_raised() {
    let mut primitive = VoxelPrimitive::default();
    primitive.tile_load_listeners += 1;
    primitive.load_progress_listeners += 2;
    primitive.tile_visible_listeners += 1;
    let mut frame_state = PrimitiveRenderFrameState::default();
    primitive.update(&mut frame_state).unwrap();

    assert_eq!(primitive.tile_load_listeners, 1);
    assert!(primitive.load_progress_listeners > 0);
    assert_eq!(primitive.tile_visible_listeners, 1);
}

#[test]
fn test_voxel_primitive_toggles_render_options_shader_dirty() {
    let mut primitive = VoxelPrimitive::default();
    let mut frame_state = PrimitiveRenderFrameState::default();
    primitive.update(&mut frame_state).unwrap();
    assert!(!primitive.shader_dirty);

    assert!(primitive.depth_test);
    primitive.set_depth_test(false);
    assert!(primitive.shader_dirty);

    primitive.update(&mut frame_state).unwrap();
    assert!(!primitive.depth_test);
    assert!(!primitive.shader_dirty);
}

#[test]
fn test_voxel_primitive_sets_render_parameters() {
    let mut primitive = VoxelPrimitive::default();
    assert_eq!(primitive.level_blend_factor, 0.0);
    assert_eq!(primitive.screen_space_error, 4.0);
    assert_eq!(primitive.step_size, 1.0);
    assert!(!primitive.debug_draw);

    primitive.level_blend_factor = 0.5;
    primitive.screen_space_error = 2.0;
    primitive.step_size = 0.5;
    primitive.debug_draw = true;

    assert_eq!(primitive.level_blend_factor, 0.5);
    assert_eq!(primitive.screen_space_error, 2.0);
    assert_eq!(primitive.step_size, 0.5);
    assert!(primitive.debug_draw);
}

#[test]
fn test_voxel_primitive_sets_clipping_range_extrema() {
    let mut primitive = VoxelPrimitive::default();
    let set_val = Cartesian3::new(0.1, 0.5, 0.3);

    assert_ne!(primitive.min_clipping_bounds, set_val);
    primitive.min_clipping_bounds = set_val;
    assert_eq!(primitive.min_clipping_bounds, set_val);

    assert_ne!(primitive.max_clipping_bounds, set_val);
    primitive.max_clipping_bounds = set_val;
    assert_eq!(primitive.max_clipping_bounds, set_val);
}

#[test]
fn test_voxel_primitive_vertically_exaggerates_ellipsoid() {
    let provider = Arc::new(MockVoxelProvider {
        shape_type: VoxelShapeType::Ellipsoid,
        ..Default::default()
    });
    let primitive = VoxelPrimitive::new(Some(VoxelPrimitiveOptions {
        provider: Some(provider),
        ..Default::default()
    }));

    let exaggeration_factor = 2.0;
    let (compound_mat, min_b, max_b, min_c, max_c) =
        primitive.compute_exaggerated_parameters(exaggeration_factor);

    assert_eq!(compound_mat, primitive.model_matrix);
    assert_eq!(min_b.z, primitive.min_bounds.z * exaggeration_factor);
    assert_eq!(max_b.z, primitive.max_bounds.z * exaggeration_factor);
    assert_eq!(min_c.z, primitive.min_clipping_bounds.z * exaggeration_factor);
    assert_eq!(max_c.z, primitive.max_clipping_bounds.z * exaggeration_factor);
}

#[test]
fn test_voxel_primitive_vertically_exaggerates_box() {
    let provider = Arc::new(MockVoxelProvider {
        shape_type: VoxelShapeType::Box,
        ..Default::default()
    });
    let primitive = VoxelPrimitive::new(Some(VoxelPrimitiveOptions {
        provider: Some(provider),
        ..Default::default()
    }));

    let exaggeration_factor = 2.0;
    let (compound_mat, min_b, max_b, min_c, max_c) =
        primitive.compute_exaggerated_parameters(exaggeration_factor);

    let scalar = Cartesian3::new(1.0, 1.0, exaggeration_factor);
    let expected_mat = Matrix4::multiply_by_scale(&primitive.model_matrix, scalar);
    assert_eq!(compound_mat, expected_mat);
    assert_eq!(min_b, primitive.min_bounds);
    assert_eq!(max_b, primitive.max_bounds);
    assert_eq!(min_c, primitive.min_clipping_bounds);
    assert_eq!(max_c, primitive.max_clipping_bounds);
}

#[test]
fn test_voxel_primitive_vertically_exaggerates_cylinder() {
    let provider = Arc::new(MockVoxelProvider {
        shape_type: VoxelShapeType::Cylinder,
        ..Default::default()
    });
    let primitive = VoxelPrimitive::new(Some(VoxelPrimitiveOptions {
        provider: Some(provider),
        ..Default::default()
    }));

    let exaggeration_factor = 2.0;
    let (compound_mat, min_b, max_b, min_c, max_c) =
        primitive.compute_exaggerated_parameters(exaggeration_factor);

    let scalar = Cartesian3::new(1.0, 1.0, exaggeration_factor);
    let expected_mat = Matrix4::multiply_by_scale(&primitive.model_matrix, scalar);
    assert_eq!(compound_mat, expected_mat);
    assert_eq!(min_b, primitive.min_bounds);
    assert_eq!(max_b, primitive.max_bounds);
    assert_eq!(min_c, primitive.min_clipping_bounds);
    assert_eq!(max_c, primitive.max_clipping_bounds);
}

#[test]
fn test_voxel_primitive_accepts_custom_shader() {
    let mut primitive = VoxelPrimitive::default();
    let default_source = primitive.custom_shader.fragment_shader_text.clone();

    // Resetting with None falls back to default
    primitive.set_custom_shader(None);
    assert_eq!(primitive.custom_shader.fragment_shader_text, default_source);

    let modified_shader = CustomShader::new(CustomShaderOptions {
        fragment_shader_text: Some(
            "void fragmentMain(FragmentInput fsInput, inout czm_modelMaterial material) { material.diffuse = vec3(1.0); }".to_string(),
        ),
        ..Default::default()
    })
    .expect("valid custom shader");
    primitive.set_custom_shader(Some(modified_shader.clone()));
    assert_eq!(
        primitive.custom_shader.fragment_shader_text,
        modified_shader.fragment_shader_text
    );
}

#[test]
fn test_voxel_primitive_destroys() {
    let mut primitive = VoxelPrimitive::default();
    assert!(!primitive.is_destroyed());
    assert!(primitive.pick_id.is_some());
    assert!(primitive.traversal_active);

    primitive.destroy();
    assert!(primitive.is_destroyed());
    assert!(primitive.pick_id.is_none());
    assert!(!primitive.traversal_active);

    let mut frame_state = PrimitiveRenderFrameState::default();
    assert!(primitive.update(&mut frame_state).is_err());
}
