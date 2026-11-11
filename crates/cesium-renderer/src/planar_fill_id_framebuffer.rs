use crate::commands::ClearCommand;
use crate::framebuffer::{
    Framebuffer, FramebufferManager, FramebufferManagerOptions, PixelDatatype, PixelFormat,
};
use crate::texture::Texture;
use cesium_math::{Color, DeveloperError};

/// Creates and manages a framebuffer for the planar fill feature-ID pre-pass.
///
/// During this pass, non-behind planar fill geometry writes its per-fragment
/// feature ID into a screen-sized texture. Later, primitives with
/// `behind: true` (BENTLEY_materials_planar_fill) sample this texture to check
/// whether the pixel they are about to overwrite belongs to the same logical object
/// (same feature ID). Only then do they apply an additional depth offset, pushing
/// the fill behind its own object's edges and non-behind fills — but NOT behind
/// other objects.
///
/// Layout:
/// - Color attachment 0: R32F (single RED channel): encoded feature ID.
/// - Depth-stencil attachment: shared with the main scene for correct depth testing.
#[derive(Debug)]
pub struct PlanarFillIdFramebuffer {
    framebuffer_manager: Option<FramebufferManager>,
    clear_command: Option<ClearCommand>,
    is_destroyed: bool,
    width: u32,
    height: u32,
}

impl Default for PlanarFillIdFramebuffer {
    fn default() -> Self {
        Self::new().expect("PlanarFillIdFramebuffer default creation should succeed")
    }
}

impl PlanarFillIdFramebuffer {
    /// Creates a new `PlanarFillIdFramebuffer`.
    pub fn new() -> Result<Self, DeveloperError> {
        let options = FramebufferManagerOptions {
            color_attachments_length: 1,
            color: true,
            depth_stencil: true,
            supports_depth_texture: true,
            create_color_attachments: true,
            create_depth_attachments: true,
            ..Default::default()
        };

        let framebuffer_manager = FramebufferManager::new(options)?;
        let clear_command = ClearCommand {
            color: Some(Color::new(0.0, 0.0, 0.0, 0.0)),
            depth: Some(1.0),
            stencil: Some(0),
            ..Default::default()
        };

        Ok(Self {
            framebuffer_manager: Some(framebuffer_manager),
            clear_command: Some(clear_command),
            is_destroyed: false,
            width: 0,
            height: 0,
        })
    }

    /// Gets the underlying framebuffer.
    pub fn framebuffer(&self) -> Option<&Framebuffer> {
        self.framebuffer_manager
            .as_ref()
            .and_then(|m| m.framebuffer())
    }

    /// Gets the feature-ID texture (color attachment 0).
    pub fn id_texture(&self) -> Option<&Texture> {
        self.framebuffer_manager
            .as_ref()
            .and_then(|m| m.get_color_texture(0).ok().flatten())
    }

    /// Gets the depth-stencil texture.
    pub fn depth_stencil_texture(&self) -> Option<&Texture> {
        self.framebuffer_manager
            .as_ref()
            .and_then(|m| m.get_depth_stencil_texture())
    }

    /// Updates (creates / resizes) the framebuffer to match the viewport dimensions.
    pub fn update(
        &mut self,
        width: u32,
        height: u32,
        hdr: bool,
    ) -> Result<bool, DeveloperError> {
        if self.is_destroyed {
            return Err(DeveloperError::new(
                "This object was destroyed, i.e., destroy() was called.",
            ));
        }

        if width == 0 || height == 0 {
            return Err(DeveloperError::new(
                "width and height must be greater than zero.",
            ));
        }

        let datatype = if hdr {
            PixelDatatype::Float
        } else {
            PixelDatatype::UnsignedByte
        };

        let manager = self
            .framebuffer_manager
            .as_mut()
            .ok_or_else(|| DeveloperError::new("FramebufferManager not initialized"))?;

        let dirty = manager.is_dirty(
            width,
            height,
            Some(1),
            Some(datatype),
            Some(PixelFormat::Red),
        );

        if dirty {
            manager.update(
                Some(width),
                Some(height),
                Some(1),
                Some(datatype),
                Some(PixelFormat::Red),
            )?;
            self.width = width;
            self.height = height;
        }

        Ok(dirty)
    }

    /// Returns a ClearCommand that targets this framebuffer.
    pub fn get_clear_command(
        &self,
        clear_color: Option<Color>,
    ) -> Result<ClearCommand, DeveloperError> {
        if self.is_destroyed {
            return Err(DeveloperError::new(
                "This object was destroyed, i.e., destroy() was called.",
            ));
        }

        let mut cmd = self
            .clear_command
            .clone()
            .ok_or_else(|| DeveloperError::new("ClearCommand not initialized"))?;

        if let Some(c) = clear_color {
            cmd.color = Some(c);
        }

        if let Some(fb) = self.framebuffer() {
            cmd.framebuffer = Some(std::rc::Rc::new(fb.clone()));
        }

        Ok(cmd)
    }

    /// Releases GPU resources without destroying this object; a later call to `update` recreates them.
    pub fn release_resources(&mut self) {
        if let Some(ref mut m) = self.framebuffer_manager {
            m.destroy();
        }
        self.width = 0;
        self.height = 0;
    }

    /// Returns true if this object was destroyed.
    pub fn is_destroyed(&self) -> bool {
        self.is_destroyed
    }

    /// Destroys WebGL/WebGPU resources.
    pub fn destroy(&mut self) {
        if !self.is_destroyed {
            if let Some(mut m) = self.framebuffer_manager.take() {
                m.destroy();
            }
            self.clear_command = None;
            self.is_destroyed = true;
        }
    }
}
