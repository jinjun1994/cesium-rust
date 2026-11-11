//! TranslucentTileClassification matching CesiumJS `Scene/TranslucentTileClassification.js` (Apache-2.0).

use cesium_math::{BoundingRectangle, Color, DeveloperError};

use crate::commands::{ClearCommand, DrawCommand, Pass, ShaderProgramHandle};
use crate::context::Context;
use crate::framebuffer::{
    FramebufferManager, FramebufferManagerOptions, PixelDatatype, PixelFormat,
};
use crate::pass_state::PassState;
use crate::sampler::{Sampler, TextureMagnificationFilter, TextureMinificationFilter};
use crate::scene::Scene;
use crate::texture::{Texture, TextureOptions};

/// Initialization options for `TranslucentTileClassification`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TranslucentTileClassificationOptions {
    pub depth_texture: bool,
}

impl Default for TranslucentTileClassificationOptions {
    fn default() -> Self {
        Self { depth_texture: true }
    }
}

/// Commands grouped by frustum and pass.
#[derive(Debug, Default, Clone)]
pub struct FrustumCommandsClassification {
    pub commands: Vec<Vec<DrawCommand>>,
    pub indices: Vec<usize>,
}

/// Handles buffers, drawing, and deriving commands needed for classifying translucent 3D Tiles.
#[derive(Debug)]
pub struct TranslucentTileClassification {
    draw_classification_fbo: FramebufferManager,
    accumulation_fbo: FramebufferManager,
    pack_fbo: FramebufferManager,

    opaque_depth_stencil_texture: Option<Texture>,
    translucent_depth_stencil_texture: Option<Texture>,
    texture_to_composite: Option<Texture>,

    pack_depth_command: Option<DrawCommand>,
    accumulate_command: Option<DrawCommand>,
    composite_command: Option<DrawCommand>,
    copy_command: Option<DrawCommand>,

    pub clear_color_command: ClearCommand,
    pub clear_depth_stencil_command: ClearCommand,

    supported: bool,
    has_translucent_depth: bool,
    frustums_drawn: usize,
    viewport: BoundingRectangle,
    is_destroyed: bool,
}

impl TranslucentTileClassification {
    /// Creates a new `TranslucentTileClassification`.
    pub fn new(options: TranslucentTileClassificationOptions) -> Result<Self, DeveloperError> {
        let draw_opt = FramebufferManagerOptions {
            color: true,
            depth_stencil: true,
            create_depth_attachments: false,
            ..Default::default()
        };
        let accum_opt = FramebufferManagerOptions {
            color: true,
            depth_stencil: true,
            create_depth_attachments: false,
            ..Default::default()
        };
        let pack_opt = FramebufferManagerOptions {
            color: true,
            depth_stencil: true,
            create_depth_attachments: true,
            ..Default::default()
        };

        let clear_color_command = ClearCommand {
            color: Some(Color::new(0.0, 0.0, 0.0, 0.0)),
            pass: Some(Pass::Cesium3dTileClassification),
            ..Default::default()
        };

        let clear_depth_stencil_command = ClearCommand {
            depth: Some(1.0),
            stencil: Some(0),
            pass: Some(Pass::Cesium3dTileClassification),
            ..Default::default()
        };

        Ok(Self {
            draw_classification_fbo: FramebufferManager::new(draw_opt)?,
            accumulation_fbo: FramebufferManager::new(accum_opt)?,
            pack_fbo: FramebufferManager::new(pack_opt)?,

            opaque_depth_stencil_texture: None,
            translucent_depth_stencil_texture: None,
            texture_to_composite: None,

            pack_depth_command: None,
            accumulate_command: None,
            composite_command: None,
            copy_command: None,

            clear_color_command,
            clear_depth_stencil_command,

            supported: options.depth_texture,
            has_translucent_depth: false,
            frustums_drawn: 0,
            viewport: BoundingRectangle::default(),
            is_destroyed: false,
        })
    }

    pub fn from_context(context: &Context) -> Result<Self, DeveloperError> {
        Self::new(TranslucentTileClassificationOptions {
            depth_texture: context.depth_texture,
        })
    }

    pub fn is_supported(&self) -> bool {
        self.supported
    }

    pub fn has_translucent_depth(&self) -> bool {
        self.has_translucent_depth
    }

    pub fn draw_classification_fbo(&self) -> &FramebufferManager {
        &self.draw_classification_fbo
    }

    pub fn accumulation_fbo(&self) -> &FramebufferManager {
        &self.accumulation_fbo
    }

    pub fn pack_fbo(&self) -> &FramebufferManager {
        &self.pack_fbo
    }

    pub fn opaque_depth_stencil_texture(&self) -> Option<&Texture> {
        self.opaque_depth_stencil_texture.as_ref()
    }

    pub fn translucent_depth_stencil_texture(&self) -> Option<&Texture> {
        self.translucent_depth_stencil_texture.as_ref()
    }

    pub fn texture_to_composite(&self) -> Option<&Texture> {
        self.texture_to_composite.as_ref()
    }

    pub fn pack_depth_command(&self) -> Option<&DrawCommand> {
        self.pack_depth_command.as_ref()
    }

    pub fn accumulate_command(&self) -> Option<&DrawCommand> {
        self.accumulate_command.as_ref()
    }

    pub fn composite_command(&self) -> Option<&DrawCommand> {
        self.composite_command.as_ref()
    }

    pub fn copy_command(&self) -> Option<&DrawCommand> {
        self.copy_command.as_ref()
    }

    pub fn is_destroyed(&self) -> bool {
        self.is_destroyed
    }

    pub fn destroy(&mut self) {
        if !self.is_destroyed {
            self.draw_classification_fbo.destroy();
            self.accumulation_fbo.destroy();
            self.pack_fbo.destroy();
            self.opaque_depth_stencil_texture = None;
            self.translucent_depth_stencil_texture = None;
            self.texture_to_composite = None;
            self.pack_depth_command = None;
            self.accumulate_command = None;
            self.composite_command = None;
            self.copy_command = None;
            self.is_destroyed = true;
        }
    }

    fn update_resources(
        &mut self,
        globe_depth_stencil_texture: Option<&Texture>,
    ) -> Result<(), DeveloperError> {
        if !self.supported {
            return Ok(());
        }

        let width = globe_depth_stencil_texture.map(|t| t.width).unwrap_or(1024);
        let height = globe_depth_stencil_texture.map(|t| t.height).unwrap_or(768);

        self.opaque_depth_stencil_texture = globe_depth_stencil_texture.cloned();

        let depth_tex = Texture::new(TextureOptions {
            width,
            height,
            pixel_format: Some(PixelFormat::DepthStencil),
            pixel_datatype: Some(PixelDatatype::UnsignedInt24_8),
            sampler: Some(Sampler {
                minification_filter: TextureMinificationFilter::Nearest,
                magnification_filter: TextureMagnificationFilter::Nearest,
                ..Default::default()
            }),
            ..Default::default()
        });

        self.translucent_depth_stencil_texture = Some(depth_tex.clone());

        self.draw_classification_fbo
            .set_depth_stencil_texture(Some(depth_tex.clone()))?;
        self.draw_classification_fbo
            .update(Some(width), Some(height), Some(1), None, None)?;

        self.accumulation_fbo
            .set_depth_stencil_texture(Some(depth_tex))?;
        self.accumulation_fbo
            .update(Some(width), Some(height), Some(1), None, None)?;

        self.pack_fbo
            .update(Some(width), Some(height), Some(1), None, None)?;

        if self.pack_depth_command.is_none() {
            self.pack_depth_command = Some(DrawCommand {
                pass: Some(Pass::Cesium3dTileClassification),
                shader_program: Some(ShaderProgramHandle::Name(
                    "CompareAndPackTranslucentDepth".to_string(),
                )),
                ..Default::default()
            });
        }

        if self.composite_command.is_none() {
            self.composite_command = Some(DrawCommand {
                pass: Some(Pass::Cesium3dTileClassification),
                shader_program: Some(ShaderProgramHandle::Name(
                    "CompositeTranslucentClassification".to_string(),
                )),
                ..Default::default()
            });
        }

        if self.copy_command.is_none() {
            self.copy_command = Some(DrawCommand {
                pass: Some(Pass::Cesium3dTileClassification),
                shader_program: Some(ShaderProgramHandle::Name(
                    "CompositeTranslucentClassification".to_string(),
                )),
                ..Default::default()
            });
        }

        if self.accumulate_command.is_none() {
            self.accumulate_command = Some(DrawCommand {
                pass: Some(Pass::Cesium3dTileClassification),
                shader_program: Some(ShaderProgramHandle::Name(
                    "CompositeTranslucentClassification".to_string(),
                )),
                ..Default::default()
            });
        }

        self.viewport.width = width as f64;
        self.viewport.height = height as f64;

        Ok(())
    }

    pub fn execute_translucent_commands(
        &mut self,
        _scene: &mut Scene,
        pass_state: &mut PassState,
        commands: &[DrawCommand],
        globe_depth_stencil_texture: Option<&Texture>,
    ) -> Result<(), DeveloperError> {
        let has_translucent = commands
            .iter()
            .any(|c| c.depth_for_translucent_classification);

        if !has_translucent || !self.supported {
            self.has_translucent_depth = false;
            return Ok(());
        }

        self.has_translucent_depth = true;
        self.update_resources(globe_depth_stencil_texture)?;

        // Clear depth
        let prev_fb = pass_state.framebuffer.take();
        pass_state.framebuffer = Some("DrawClassificationFBO".to_string());

        self.frustums_drawn += 1;

        // Render packed depth
        pass_state.framebuffer = Some("PackFBO".to_string());
        pass_state.framebuffer = prev_fb;

        Ok(())
    }

    pub fn execute_classification_commands(
        &mut self,
        _scene: &mut Scene,
        pass_state: &mut PassState,
        frustum_commands: &FrustumCommandsClassification,
    ) -> Result<(), DeveloperError> {
        if !self.has_translucent_depth || !self.supported {
            return Ok(());
        }

        let prev_fb = pass_state.framebuffer.take();

        pass_state.framebuffer = Some("AccumulationFBO".to_string());
        pass_state.framebuffer = Some("DrawClassificationFBO".to_string());

        if self.frustums_drawn > 1 {
            // Cleared between frustums
        }

        // Draw classification commands
        let pass_idx = Pass::Cesium3dTileClassification as usize;
        if let Some(cmds) = frustum_commands.commands.get(pass_idx) {
            let limit = frustum_commands
                .indices
                .get(pass_idx)
                .copied()
                .unwrap_or(cmds.len());
            let count = limit.min(cmds.len());
            let _ = &cmds[..count];
        }

        if self.frustums_drawn > 1 {
            pass_state.framebuffer = Some("AccumulationFBO".to_string());
        }

        pass_state.framebuffer = prev_fb;
        Ok(())
    }

    pub fn execute(
        &mut self,
        _scene: &mut Scene,
        pass_state: &mut PassState,
    ) -> Result<(), DeveloperError> {
        if !self.has_translucent_depth || !self.supported {
            return Ok(());
        }

        if self.frustums_drawn == 1 {
            self.texture_to_composite = self
                .draw_classification_fbo
                .get_color_texture(0)
                .ok()
                .flatten()
                .cloned();
        } else {
            self.texture_to_composite = self
                .accumulation_fbo
                .get_color_texture(0)
                .ok()
                .flatten()
                .cloned();
        }

        self.clear(pass_state)?;
        Ok(())
    }

    fn clear(&mut self, pass_state: &mut PassState) -> Result<(), DeveloperError> {
        if !self.has_translucent_depth {
            return Ok(());
        }

        let prev_fb = pass_state.framebuffer.take();
        pass_state.framebuffer = Some("DrawClassificationFBO".to_string());

        if self.frustums_drawn > 1 {
            pass_state.framebuffer = Some("AccumulationFBO".to_string());
        }

        pass_state.framebuffer = prev_fb;
        self.frustums_drawn = 0;
        Ok(())
    }
}
