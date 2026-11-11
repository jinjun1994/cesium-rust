//! buildVoxelDrawCommands matching CesiumJS `Scene/buildVoxelDrawCommands.js` (Apache-2.0).

use crate::commands::{DrawCommand, ShaderProgramHandle};
use crate::shader_builder::ShaderDestination;
use crate::shader_source::ShaderContextInfo;
use crate::voxel_primitive::VoxelPrimitive;
use crate::voxel_render_resources::VoxelRenderResources;

/// Assembles draw commands for rendering and picking a VoxelPrimitive.
pub fn build_voxel_draw_commands(primitive: &mut VoxelPrimitive) {
    let mut render_resources = VoxelRenderResources::new(primitive);

    if render_resources.clipping_planes_length > 0 {
        let function_id = "getClippingPlane";
        let function_signature =
            "vec4 getClippingPlane(highp sampler2D packedPlanes, int planeNumber)";
        let function_body = "return vec4(0.0);";
        render_resources.shader_builder.add_function(
            function_id,
            function_signature,
            ShaderDestination::FRAGMENT,
        );
        render_resources
            .shader_builder
            .add_function_lines(function_id, &[function_body]);
    }

    let context_info = ShaderContextInfo::default();
    let built_program = render_resources
        .shader_builder
        .build_shader_program(&context_info);

    primitive.shader_program = Some(built_program);

    let draw_command = DrawCommand {
        shader_program: Some(ShaderProgramHandle::Name("VoxelDrawCommand".to_string())),
        ..Default::default()
    };
    let draw_command_pick = DrawCommand {
        shader_program: Some(ShaderProgramHandle::Name("VoxelDrawCommandPick".to_string())),
        ..Default::default()
    };

    primitive.draw_command = Some(draw_command);
    primitive.draw_command_pick = Some(draw_command_pick);
}
