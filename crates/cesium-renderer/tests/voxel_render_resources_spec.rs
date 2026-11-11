use cesium_renderer::{
    PrimitiveRenderFrameState, VoxelPrimitive, VoxelRenderResources,
};

#[test]
fn test_voxel_render_resources_constructs() {
    let mut primitive = VoxelPrimitive {
        show: false,
        ..Default::default()
    };
    let mut frame_state = PrimitiveRenderFrameState::default();
    primitive.update(&mut frame_state).unwrap();

    let render_resources = VoxelRenderResources::new(&primitive);
    assert!(!render_resources.shader_builder.fragment_uniform_lines().is_empty());
    assert!(!render_resources.shader_builder.fragment_define_lines().is_empty());
    assert_eq!(render_resources.clipping_planes_length, 0);
    assert_eq!(render_resources.render_bound_planes_length, 0);
}
