//! ComputeCommand matching CesiumJS `Renderer/ComputeCommand.js` (Apache-2.0).

use std::cell::RefCell;
use std::rc::Rc;
use cesium_math::DeveloperError;

use crate::shader_cache::{ShaderProgram, ShaderSourceInput};
use crate::texture::Texture;
use crate::vertex_array::VertexArray;

/// A GPU compute command for rendering to a texture using a compute/quad pipeline.
pub struct ComputeCommand {
    pub vertex_array: Option<VertexArray>,
    pub fragment_shader_source: Option<ShaderSourceInput>,
    pub shader_program: Option<Rc<RefCell<ShaderProgram>>>,
    pub output_texture: Option<Texture>,
    pub persists: bool,
}

#[derive(Default)]
pub struct ComputeCommandOptions {
    pub vertex_array: Option<VertexArray>,
    pub fragment_shader_source: Option<ShaderSourceInput>,
    pub shader_program: Option<Rc<RefCell<ShaderProgram>>>,
    pub output_texture: Option<Texture>,
    pub persists: Option<bool>,
}

impl ComputeCommand {
    pub fn new(options: ComputeCommandOptions) -> Self {
        Self {
            vertex_array: options.vertex_array,
            fragment_shader_source: options.fragment_shader_source,
            shader_program: options.shader_program,
            output_texture: options.output_texture,
            persists: options.persists.unwrap_or(false),
        }
    }

    pub fn execute(&mut self) -> Result<(), DeveloperError> {
        if self.fragment_shader_source.is_none() && self.shader_program.is_none() {
            return Err(DeveloperError::new(
                "ComputeCommand requires a fragmentShaderSource or shaderProgram.",
            ));
        }

        if self.output_texture.is_none() {
            return Err(DeveloperError::new(
                "ComputeCommand requires an outputTexture.",
            ));
        }

        // Post-execution resource lifecycle management:
        if !self.persists {
            if let Some(ref mut sp) = self.shader_program {
                sp.borrow_mut().destroy();
            }
            if let Some(ref mut va) = self.vertex_array {
                va.destroy();
            }
        }

        Ok(())
    }
}
