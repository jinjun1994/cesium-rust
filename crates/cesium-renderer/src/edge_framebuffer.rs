use crate::commands::ClearCommand;
use crate::framebuffer::{
    Framebuffer, FramebufferManager, FramebufferManagerOptions, PixelDatatype, PixelFormat,
};
use crate::texture::Texture;
use cesium_math::{Color, DeveloperError};

/// Creates and manages framebuffers for edge visibility rendering.
#[derive(Debug)]
pub struct EdgeFramebuffer {
    framebuffer_manager: Option<FramebufferManager>,
    clear_command: Option<ClearCommand>,
    is_destroyed: bool,
    width: u32,
    height: u32,
}

impl Default for EdgeFramebuffer {
    fn default() -> Self {
        Self::new().expect("EdgeFramebuffer default creation should succeed")
    }
}

impl EdgeFramebuffer {
    /// Creates a new `EdgeFramebuffer`.
    pub fn new() -> Result<Self, DeveloperError> {
        let options = FramebufferManagerOptions {
            color_attachments_length: 3,
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

    /// Gets the framebuffer for edge rendering.
    pub fn framebuffer(&self) -> Option<&Framebuffer> {
        self.framebuffer_manager
            .as_ref()
            .and_then(|m| m.framebuffer())
    }

    /// Gets the color texture.
    pub fn color_texture(&self) -> Option<&Texture> {
        self.framebuffer_manager
            .as_ref()
            .and_then(|m| m.get_color_texture(0).ok().flatten())
    }

    /// Gets the ID texture.
    pub fn id_texture(&self) -> Option<&Texture> {
        self.framebuffer_manager
            .as_ref()
            .and_then(|m| m.get_color_texture(1).ok().flatten())
    }

    /// Gets the packed depth texture written during the edge pass.
    pub fn depth_texture(&self) -> Option<&Texture> {
        self.framebuffer_manager
            .as_ref()
            .and_then(|m| m.get_color_texture(2).ok().flatten())
    }

    /// Updates the framebuffer for the given viewport dimensions.
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

        let dirty = manager.is_dirty(width, height, Some(1), Some(datatype), Some(PixelFormat::Rgba));
        if dirty {
            manager.update(
                Some(width),
                Some(height),
                Some(1),
                Some(datatype),
                Some(PixelFormat::Rgba),
            )?;
            self.width = width;
            self.height = height;
        }

        Ok(dirty)
    }

    /// Gets the clear command for this framebuffer.
    pub fn get_clear_command(&self, clear_color: Option<Color>) -> Result<ClearCommand, DeveloperError> {
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

        Ok(cmd)
    }

    /// Returns true if this object was destroyed.
    pub fn is_destroyed(&self) -> bool {
        self.is_destroyed
    }

    /// Destroys the framebuffer resources.
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
