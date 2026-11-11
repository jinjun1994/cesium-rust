//! VoxelRenderResources matching CesiumJS `Scene/VoxelRenderResources.js` (Apache-2.0).

use std::collections::HashMap;

use crate::custom_shader::CustomShaderUniformCallback;
use crate::shader_builder::{ShaderBuilder, ShaderDestination};
use crate::voxel_primitive::VoxelPrimitive;
use crate::voxel_shape_type::VoxelShapeType;

/// Render resources assembled for rendering a VoxelPrimitive.
#[derive(Clone)]
pub struct VoxelRenderResources {
    pub shader_builder: ShaderBuilder,
    pub uniform_map: HashMap<String, CustomShaderUniformCallback>,
    pub clipping_planes_length: usize,
    pub render_bound_planes_length: usize,
}

impl VoxelRenderResources {
    /// Constructs a new VoxelRenderResources given a VoxelPrimitive.
    pub fn new(primitive: &VoxelPrimitive) -> Self {
        let mut shader_builder = ShaderBuilder::new();

        // Custom shader uniforms
        let custom_shader = &primitive.custom_shader;
        let uniform_map = custom_shader.uniform_map.clone();

        if let Ok(guard) = custom_shader.uniforms.read() {
            for (name, specifier) in guard.iter() {
                shader_builder.add_uniform(
                    specifier.uniform_type.glsl_type(),
                    name,
                    Some(ShaderDestination::FRAGMENT),
                );
            }
        }

        // Add megatexture sampler
        shader_builder.add_uniform(
            "sampler3D",
            "u_megatextureTextures[METADATA_COUNT]",
            Some(ShaderDestination::FRAGMENT),
        );

        match primitive.shape_type {
            VoxelShapeType::Box => {
                shader_builder.add_define("SHAPE_BOX", None, Some(ShaderDestination::FRAGMENT));
            }
            VoxelShapeType::Cylinder => {
                shader_builder.add_define(
                    "SHAPE_CYLINDER",
                    None,
                    Some(ShaderDestination::FRAGMENT),
                );
            }
            VoxelShapeType::Ellipsoid => {
                shader_builder.add_define(
                    "SHAPE_ELLIPSOID",
                    None,
                    Some(ShaderDestination::FRAGMENT),
                );
            }
        }

        if primitive.depth_test {
            shader_builder.add_define("DEPTH_TEST", None, Some(ShaderDestination::FRAGMENT));
        }

        let mut intersection_count = 1;
        if primitive.depth_test {
            let count_str = intersection_count.to_string();
            shader_builder.add_define(
                "DEPTH_INTERSECTION_INDEX",
                Some(&count_str),
                Some(ShaderDestination::FRAGMENT),
            );
            intersection_count += 1;
        }
        let total_count_str = intersection_count.to_string();
        shader_builder.add_define(
            "INTERSECTION_COUNT",
            Some(&total_count_str),
            Some(ShaderDestination::FRAGMENT),
        );

        let clipping_planes_length = primitive
            .clipping_planes
            .as_ref()
            .map(|c| c.len())
            .unwrap_or(0);

        Self {
            shader_builder,
            uniform_map,
            clipping_planes_length,
            render_bound_planes_length: 0,
        }
    }
}
