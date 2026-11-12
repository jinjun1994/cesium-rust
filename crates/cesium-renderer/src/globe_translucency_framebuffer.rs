//! GlobeTranslucencyFramebuffer matching CesiumJS `Scene/GlobeTranslucencyFramebuffer.js`.

use std::rc::Rc;

use cesium_math::{BoundingRectangle, Color, DeveloperError};

use crate::commands::{ClearCommand, DrawCommand, FramebufferHandle};
use crate::framebuffer::{
    Framebuffer, FramebufferManager, FramebufferManagerOptions, PixelDatatype, PixelFormat,
};
use crate::pass_state::PassState;
use crate::render_state::RenderState;
use crate::texture::Texture;

/// Manages offscreen framebuffers and textures for globe translucency rendering.
#[derive(Debug)]
pub struct GlobeTranslucencyFramebuffer {
    framebuffer: FramebufferManager,
    packed_depth_framebuffer: FramebufferManager,
    render_state: Option<RenderState>,
    packed_depth_command: Option<DrawCommand>,
    clear_command: Option<ClearCommand>,
    viewport: BoundingRectangle,
    use_scissor_test: bool,
    scissor_rectangle: Option<BoundingRectangle>,
    use_hdr: Option<bool>,
    is_destroyed: bool,
}

impl Default for GlobeTranslucencyFramebuffer {
    fn default() -> Self {
        Self::new()
    }
}

impl GlobeTranslucencyFramebuffer {
    /// Creates a new `GlobeTranslucencyFramebuffer`.
    pub fn new() -> Self {
        let framebuffer = FramebufferManager::new(FramebufferManagerOptions {
            color_attachments_length: 1,
            color: true,
            depth_stencil: true,
            supports_depth_texture: true,
            create_color_attachments: true,
            create_depth_attachments: true,
            ..Default::default()
        })
        .expect("GlobeTranslucencyFramebuffer main FramebufferManager creation should succeed");

        let packed_depth_framebuffer = FramebufferManager::new(FramebufferManagerOptions {
            color_attachments_length: 1,
            color: true,
            depth_stencil: false,
            supports_depth_texture: false,
            create_color_attachments: true,
            create_depth_attachments: false,
            ..Default::default()
        })
        .expect("GlobeTranslucencyFramebuffer packed depth FramebufferManager creation should succeed");

        Self {
            framebuffer,
            packed_depth_framebuffer,
            render_state: None,
            packed_depth_command: None,
            clear_command: None,
            viewport: BoundingRectangle::default(),
            use_scissor_test: false,
            scissor_rectangle: None,
            use_hdr: None,
            is_destroyed: false,
        }
    }

    pub fn classification_texture(&self) -> Option<&Texture> {
        self.framebuffer.get_color_texture(0).ok().flatten()
    }

    pub fn classification_framebuffer(&self) -> Option<&Framebuffer> {
        self.framebuffer.framebuffer()
    }

    pub fn packed_depth_framebuffer(&self) -> Option<&Framebuffer> {
        self.packed_depth_framebuffer.framebuffer()
    }

    pub fn depth_stencil_texture(&self) -> Option<&Texture> {
        self.framebuffer.get_depth_stencil_texture()
    }

    pub fn depth_stencil_renderbuffer(&self) -> Option<&crate::framebuffer::Renderbuffer> {
        self.framebuffer.get_depth_stencil_renderbuffer()
    }

    pub fn packed_depth_texture(&self) -> Option<&Texture> {
        self.packed_depth_framebuffer
            .get_color_texture(0)
            .ok()
            .flatten()
    }

    pub fn clear_command(&self) -> Option<&ClearCommand> {
        self.clear_command.as_ref()
    }

    pub fn packed_depth_command(&self) -> Option<&DrawCommand> {
        self.packed_depth_command.as_ref()
    }

    pub fn render_state(&self) -> Option<&RenderState> {
        self.render_state.as_ref()
    }

    pub fn use_scissor_test(&self) -> bool {
        self.use_scissor_test
    }

    pub fn scissor_rectangle(&self) -> Option<&BoundingRectangle> {
        self.scissor_rectangle.as_ref()
    }

    /// Updates resources if viewport or HDR setting changed, and prepares clear/packed-depth commands.
    pub fn update_and_clear(
        &mut self,
        hdr: bool,
        viewport: &BoundingRectangle,
        half_floating_point_texture: bool,
        pass_state: &PassState,
    ) -> Result<(), DeveloperError> {
        if self.is_destroyed {
            return Err(DeveloperError::new(
                "This object was destroyed, i.e., destroy() was called.",
            ));
        }
        let width = viewport.width.max(1.0) as u32;
        let height = viewport.height.max(1.0) as u32;

        let pixel_datatype = if hdr {
            if half_floating_point_texture {
                PixelDatatype::HalfFloat
            } else {
                PixelDatatype::Float
            }
        } else {
            PixelDatatype::UnsignedByte
        };

        let dirty = self.use_hdr != Some(hdr)
            || self.framebuffer.is_dirty(
                width,
                height,
                Some(1),
                Some(pixel_datatype),
                Some(PixelFormat::Rgba),
            );

        if dirty {
            self.framebuffer.update(
                Some(width),
                Some(height),
                Some(1),
                Some(pixel_datatype),
                Some(PixelFormat::Rgba),
            )?;
            self.packed_depth_framebuffer.update(
                Some(width),
                Some(height),
                Some(1),
                Some(PixelDatatype::UnsignedByte),
                Some(PixelFormat::Rgba),
            )?;
            self.use_hdr = Some(hdr);
        }

        self.viewport.width = viewport.width;
        self.viewport.height = viewport.height;

        let pass_viewport = pass_state.viewport.unwrap_or(*viewport);
        self.use_scissor_test = self.viewport != pass_viewport;
        if self
            .scissor_rectangle
            .map(|r| r != pass_viewport)
            .unwrap_or(true)
        {
            self.scissor_rectangle = Some(pass_viewport);
        }

        if self.render_state.is_none() || dirty {
            self.render_state = Some(RenderState {
                viewport: Some(self.viewport),
                ..Default::default()
            });
            self.clear_command = Some(ClearCommand {
                color: Some(Color::new(0.0, 0.0, 0.0, 0.0)),
                depth: Some(1.0),
                stencil: Some(0),
                framebuffer: self
                    .framebuffer
                    .framebuffer()
                    .map(|fb| Rc::new(fb.clone())),
                ..Default::default()
            });
            self.packed_depth_command = Some(DrawCommand {
                framebuffer: self
                    .packed_depth_framebuffer
                    .framebuffer()
                    .map(|fb| FramebufferHandle::from(fb.clone())),
                ..Default::default()
            });
        }

        Ok(())
    }

    pub fn is_destroyed(&self) -> bool {
        self.is_destroyed
    }

    pub fn destroy(&mut self) {
        if !self.is_destroyed {
            self.framebuffer.destroy();
            self.packed_depth_framebuffer.destroy();
            self.render_state = None;
            self.clear_command = None;
            self.packed_depth_command = None;
            self.is_destroyed = true;
        }
    }
}
