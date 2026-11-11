use std::sync::Arc;

use cesium_math::{Cartesian3, Plane};
use cesium_renderer::{
    build_voxel_draw_commands, ClippingPlane, ClippingPlaneCollection,
    ClippingPlaneCollectionOptions, MockVoxelProvider, PrimitiveRenderFrameState, VoxelPrimitive,
    VoxelPrimitiveOptions, VoxelShapeType,
};

#[test]
fn test_sets_up_basic_voxel_draw_commands() {
    let provider = Arc::new(MockVoxelProvider {
        shape_type: VoxelShapeType::Ellipsoid,
        ..Default::default()
    });
    let mut primitive = VoxelPrimitive::new(Some(VoxelPrimitiveOptions {
        provider: Some(provider),
        ..Default::default()
    }));

    let mut frame_state = PrimitiveRenderFrameState::default();
    primitive.update(&mut frame_state).unwrap();

    build_voxel_draw_commands(&mut primitive);
    assert!(primitive.draw_command.is_some());
    assert!(primitive.draw_command_pick.is_some());
}

#[test]
fn test_adds_clipping_function_for_primitive_with_clipping_planes() {
    let provider = Arc::new(MockVoxelProvider {
        shape_type: VoxelShapeType::Ellipsoid,
        ..Default::default()
    });
    let mut primitive = VoxelPrimitive::new(Some(VoxelPrimitiveOptions {
        provider: Some(provider),
        ..Default::default()
    }));

    let planes = vec![
        ClippingPlane::from_plane(&Plane::new(Cartesian3::UNIT_X.into(), 1.0).unwrap()),
        ClippingPlane::from_plane(&Plane::new(Cartesian3::UNIT_Y.into(), 2.0).unwrap()),
    ];

    primitive.clipping_planes = Some(ClippingPlaneCollection::new(Some(
        ClippingPlaneCollectionOptions {
            planes: Some(planes),
            ..Default::default()
        },
    )));

    let mut frame_state = PrimitiveRenderFrameState::default();
    primitive.update(&mut frame_state).unwrap();

    build_voxel_draw_commands(&mut primitive);

    let shader_prog = primitive
        .shader_program
        .as_ref()
        .expect("has shader program");

    let clipping_sig = "vec4 getClippingPlane(highp sampler2D packedPlanes, int planeNumber)";
    assert!(
        shader_prog.fragment_shader_text.contains(clipping_sig),
        "fragment shader should contain clipping plane function signature"
    );
}
