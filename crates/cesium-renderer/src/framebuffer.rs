//! Framebuffer and FramebufferManager matching CesiumJS `Renderer/Framebuffer*.js` (Apache-2.0).

use cesium_math::DeveloperError;
use crate::texture::{Texture, TextureOptions};
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_FRAMEBUFFER_ID: AtomicU64 = AtomicU64::new(1);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PixelFormat {
    DepthComponent,
    DepthStencil,
    Alpha,
    Rgb,
    Rgba,
    Luminance,
    LuminanceAlpha,
}

impl PixelFormat {
    pub fn is_color_format(&self) -> bool {
        matches!(
            self,
            Self::Alpha | Self::Rgb | Self::Rgba | Self::Luminance | Self::LuminanceAlpha
        )
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PixelDatatype {
    UnsignedByte,
    UnsignedShort,
    UnsignedInt,
    Float,
    HalfFloat,
    UnsignedInt24_8,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum RenderbufferFormat {
    Rgba4,
    Rgb565,
    Rgb5A1,
    DepthComponent16,
    StencilIndex8,
    DepthStencil,
    Depth24Stencil8,
}

impl RenderbufferFormat {
    pub fn get_color_format(datatype: PixelDatatype) -> Self {
        match datatype {
            PixelDatatype::Float => Self::Rgba4,
            _ => Self::Rgba4,
        }
    }
}

/// A renderbuffer attachment representation.
#[derive(Clone, Debug, PartialEq)]
pub struct Renderbuffer {
    pub width: u32,
    pub height: u32,
    pub format: RenderbufferFormat,
    pub num_samples: u32,
    pub is_destroyed: bool,
}

#[derive(Clone, Debug)]
pub struct RenderbufferOptions {
    pub width: u32,
    pub height: u32,
    pub format: RenderbufferFormat,
    pub num_samples: Option<u32>,
}

impl Renderbuffer {
    pub fn new(options: RenderbufferOptions) -> Self {
        Self {
            width: options.width,
            height: options.height,
            format: options.format,
            num_samples: options.num_samples.unwrap_or(1),
            is_destroyed: false,
        }
    }

    pub fn is_destroyed(&self) -> bool {
        self.is_destroyed
    }

    pub fn destroy(&mut self) {
        self.is_destroyed = true;
    }
}

/// A Framebuffer object managing color, depth, and stencil attachments.
#[derive(Clone, Debug, PartialEq)]
pub struct Framebuffer {
    pub id: u64,
    pub color_textures: Vec<Option<Texture>>,
    pub color_renderbuffers: Vec<Option<Renderbuffer>>,
    pub depth_texture: Option<Texture>,
    pub depth_renderbuffer: Option<Renderbuffer>,
    pub depth_stencil_texture: Option<Texture>,
    pub depth_stencil_renderbuffer: Option<Renderbuffer>,
    pub destroy_attachments: bool,
    pub is_destroyed: bool,
}

impl Default for Framebuffer {
    fn default() -> Self {
        Self {
            id: NEXT_FRAMEBUFFER_ID.fetch_add(1, Ordering::Relaxed),
            color_textures: Vec::new(),
            color_renderbuffers: Vec::new(),
            depth_texture: None,
            depth_renderbuffer: None,
            depth_stencil_texture: None,
            depth_stencil_renderbuffer: None,
            destroy_attachments: false,
            is_destroyed: false,
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct FramebufferOptions {
    pub color_textures: Vec<Option<Texture>>,
    pub color_renderbuffers: Vec<Option<Renderbuffer>>,
    pub depth_texture: Option<Texture>,
    pub depth_renderbuffer: Option<Renderbuffer>,
    pub depth_stencil_texture: Option<Texture>,
    pub depth_stencil_renderbuffer: Option<Renderbuffer>,
    pub destroy_attachments: Option<bool>,
}

impl Framebuffer {
    pub fn new(options: FramebufferOptions) -> Result<Self, DeveloperError> {
        let has_color_textures = options.color_textures.iter().any(|t| t.is_some());
        let has_color_renderbuffers = options.color_renderbuffers.iter().any(|r| r.is_some());
        if has_color_textures && has_color_renderbuffers {
            return Err(DeveloperError::new(
                "Cannot have both color texture and color renderbuffer attachments.",
            ));
        }
        if options.depth_texture.is_some() && options.depth_renderbuffer.is_some() {
            return Err(DeveloperError::new(
                "Cannot have both a depth texture and depth renderbuffer attachment.",
            ));
        }
        if options.depth_stencil_texture.is_some() && options.depth_stencil_renderbuffer.is_some() {
            return Err(DeveloperError::new(
                "Cannot have both a depth-stencil texture and depth-stencil renderbuffer attachment.",
            ));
        }
        if options.depth_renderbuffer.is_some() && options.depth_stencil_renderbuffer.is_some() {
            return Err(DeveloperError::new(
                "Cannot have both a depth and depth-stencil renderbuffer.",
            ));
        }

        Ok(Self {
            id: NEXT_FRAMEBUFFER_ID.fetch_add(1, Ordering::Relaxed),
            color_textures: options.color_textures,
            color_renderbuffers: options.color_renderbuffers,
            depth_texture: options.depth_texture,
            depth_renderbuffer: options.depth_renderbuffer,
            depth_stencil_texture: options.depth_stencil_texture,
            depth_stencil_renderbuffer: options.depth_stencil_renderbuffer,
            destroy_attachments: options.destroy_attachments.unwrap_or(true),
            is_destroyed: false,
        })
    }

    pub fn number_of_color_attachments(&self) -> usize {
        let tex_count = self.color_textures.iter().filter(|t| t.is_some()).count();
        let rb_count = self.color_renderbuffers.iter().filter(|r| r.is_some()).count();
        tex_count.max(rb_count)
    }

    pub fn get_color_texture(&self, index: usize) -> Option<&Texture> {
        self.color_textures.get(index).and_then(|t| t.as_ref())
    }

    pub fn get_color_renderbuffer(&self, index: usize) -> Option<&Renderbuffer> {
        self.color_renderbuffers.get(index).and_then(|r| r.as_ref())
    }

    pub fn is_destroyed(&self) -> bool {
        self.is_destroyed
    }

    pub fn destroy(&mut self) {
        if self.destroy_attachments {
            for tex in self.color_textures.iter_mut().flatten() {
                tex.destroy();
            }
            for rb in self.color_renderbuffers.iter_mut().flatten() {
                rb.destroy();
            }
            if let Some(ref mut dt) = self.depth_texture {
                dt.destroy();
            }
            if let Some(ref mut dr) = self.depth_renderbuffer {
                dr.destroy();
            }
            if let Some(ref mut dst) = self.depth_stencil_texture {
                dst.destroy();
            }
            if let Some(ref mut dsr) = self.depth_stencil_renderbuffer {
                dsr.destroy();
            }
        }
        self.is_destroyed = true;
    }
}

/// Options for configuring a FramebufferManager.
#[derive(Clone, Debug)]
pub struct FramebufferManagerOptions {
    pub num_samples: u32,
    pub color_attachments_length: usize,
    pub color: bool,
    pub depth: bool,
    pub depth_stencil: bool,
    pub supports_depth_texture: bool,
    pub create_color_attachments: bool,
    pub create_depth_attachments: bool,
    pub pixel_datatype: Option<PixelDatatype>,
    pub pixel_format: Option<PixelFormat>,
}

impl Default for FramebufferManagerOptions {
    fn default() -> Self {
        Self {
            num_samples: 1,
            color_attachments_length: 1,
            color: true,
            depth: false,
            depth_stencil: false,
            supports_depth_texture: false,
            create_color_attachments: true,
            create_depth_attachments: true,
            pixel_datatype: None,
            pixel_format: None,
        }
    }
}

/// A manager that handles lazy resize and allocation of Framebuffers.
pub struct FramebufferManager {
    num_samples: u32,
    color_attachments_length: usize,
    color: bool,
    depth: bool,
    depth_stencil: bool,
    supports_depth_texture: bool,
    create_color_attachments: bool,
    create_depth_attachments: bool,
    pixel_datatype: Option<PixelDatatype>,
    pixel_format: Option<PixelFormat>,

    width: Option<u32>,
    height: Option<u32>,

    framebuffer: Option<Framebuffer>,
    color_textures: Vec<Option<Texture>>,
    color_renderbuffers: Vec<Option<Renderbuffer>>,
    depth_texture: Option<Texture>,
    depth_renderbuffer: Option<Renderbuffer>,
    depth_stencil_texture: Option<Texture>,
    depth_stencil_renderbuffer: Option<Renderbuffer>,
    attachments_dirty: bool,
    is_destroyed: bool,
}

impl FramebufferManager {
    pub fn new(options: FramebufferManagerOptions) -> Result<Self, DeveloperError> {
        if !options.color && !options.depth && !options.depth_stencil {
            return Err(DeveloperError::new(
                "Must enable at least one type of framebuffer attachment.",
            ));
        }
        if options.depth && options.depth_stencil {
            return Err(DeveloperError::new(
                "Cannot have both a depth and depth-stencil attachment.",
            ));
        }

        let len = options.color_attachments_length;
        let color_textures = if options.color {
            vec![None; len]
        } else {
            Vec::new()
        };
        let color_renderbuffers = if options.color {
            vec![None; len]
        } else {
            Vec::new()
        };

        Ok(Self {
            num_samples: options.num_samples,
            color_attachments_length: options.color_attachments_length,
            color: options.color,
            depth: options.depth,
            depth_stencil: options.depth_stencil,
            supports_depth_texture: options.supports_depth_texture,
            create_color_attachments: options.create_color_attachments,
            create_depth_attachments: options.create_depth_attachments,
            pixel_datatype: options.pixel_datatype,
            pixel_format: options.pixel_format,
            width: None,
            height: None,
            framebuffer: None,
            color_textures,
            color_renderbuffers,
            depth_texture: None,
            depth_renderbuffer: None,
            depth_stencil_texture: None,
            depth_stencil_renderbuffer: None,
            attachments_dirty: false,
            is_destroyed: false,
        })
    }

    pub fn framebuffer(&self) -> Option<&Framebuffer> {
        self.framebuffer.as_ref()
    }

    pub fn num_samples(&self) -> u32 {
        self.num_samples
    }

    pub fn is_dirty(
        &self,
        width: u32,
        height: u32,
        num_samples: Option<u32>,
        pixel_datatype: Option<PixelDatatype>,
        pixel_format: Option<PixelFormat>,
    ) -> bool {
        let samples = num_samples.unwrap_or(1);
        let dimension_changed = self.width != Some(width) || self.height != Some(height);
        let samples_changed = self.num_samples != samples;
        let pixel_changed = (pixel_datatype.is_some() && self.pixel_datatype != pixel_datatype)
            || (pixel_format.is_some() && self.pixel_format != pixel_format);
        let fb_defined = self.framebuffer.is_some();

        self.attachments_dirty
            || dimension_changed
            || samples_changed
            || pixel_changed
            || !fb_defined
            || (self.color && self.color_textures.first().and_then(|t| t.as_ref()).is_none())
    }

    pub fn update(
        &mut self,
        width: Option<u32>,
        height: Option<u32>,
        num_samples: Option<u32>,
        pixel_datatype: Option<PixelDatatype>,
        pixel_format: Option<PixelFormat>,
    ) -> Result<(), DeveloperError> {
        let w = width.ok_or_else(|| DeveloperError::new("width and height must be defined."))?;
        let h = height.ok_or_else(|| DeveloperError::new("width and height must be defined."))?;

        let samples = num_samples.unwrap_or(1);
        let datatype = pixel_datatype
            .or(self.pixel_datatype)
            .unwrap_or(PixelDatatype::UnsignedByte);
        let format = pixel_format
            .or(self.pixel_format)
            .unwrap_or(PixelFormat::Rgba);

        if self.is_dirty(w, h, Some(samples), Some(datatype), Some(format)) {
            self.destroy_resources();
            self.width = Some(w);
            self.height = Some(h);
            self.num_samples = samples;
            self.pixel_datatype = Some(datatype);
            self.pixel_format = Some(format);
            self.attachments_dirty = false;

            if self.color && self.create_color_attachments {
                for i in 0..self.color_attachments_length {
                    self.color_textures[i] = Some(Texture::new(TextureOptions {
                        width: w,
                        height: h,
                        pixel_format: Some(format),
                        pixel_datatype: Some(datatype),
                        ..Default::default()
                    }));
                }
            }

            if self.depth_stencil && self.create_depth_attachments {
                if self.supports_depth_texture {
                    self.depth_stencil_texture = Some(Texture::new(TextureOptions {
                        width: w,
                        height: h,
                        pixel_format: Some(PixelFormat::DepthStencil),
                        pixel_datatype: Some(PixelDatatype::UnsignedInt24_8),
                        ..Default::default()
                    }));
                } else {
                    self.depth_stencil_renderbuffer = Some(Renderbuffer::new(RenderbufferOptions {
                        width: w,
                        height: h,
                        format: RenderbufferFormat::DepthStencil,
                        num_samples: Some(samples),
                    }));
                }
            }

            if self.depth && self.create_depth_attachments {
                if self.supports_depth_texture {
                    self.depth_texture = Some(Texture::new(TextureOptions {
                        width: w,
                        height: h,
                        pixel_format: Some(PixelFormat::DepthComponent),
                        pixel_datatype: Some(PixelDatatype::UnsignedInt),
                        ..Default::default()
                    }));
                } else {
                    self.depth_renderbuffer = Some(Renderbuffer::new(RenderbufferOptions {
                        width: w,
                        height: h,
                        format: RenderbufferFormat::DepthComponent16,
                        num_samples: Some(samples),
                    }));
                }
            }

            let fb = Framebuffer::new(FramebufferOptions {
                color_textures: self.color_textures.clone(),
                color_renderbuffers: self.color_renderbuffers.clone(),
                depth_texture: self.depth_texture.clone(),
                depth_renderbuffer: self.depth_renderbuffer.clone(),
                depth_stencil_texture: self.depth_stencil_texture.clone(),
                depth_stencil_renderbuffer: self.depth_stencil_renderbuffer.clone(),
                destroy_attachments: Some(false),
            })?;
            self.framebuffer = Some(fb);
        }

        Ok(())
    }

    pub fn get_color_texture(&self, index: usize) -> Result<Option<&Texture>, DeveloperError> {
        if index >= self.color_attachments_length {
            return Err(DeveloperError::new(
                "index must be smaller than total number of color attachments.",
            ));
        }
        Ok(self.color_textures.get(index).and_then(|t| t.as_ref()))
    }

    pub fn set_color_texture(
        &mut self,
        texture: Option<Texture>,
        index: usize,
    ) -> Result<(), DeveloperError> {
        if self.create_color_attachments {
            return Err(DeveloperError::new(
                "createColorAttachments must be false if setColorTexture is called.",
            ));
        }
        if index >= self.color_attachments_length {
            return Err(DeveloperError::new(
                "index must be smaller than total number of color attachments.",
            ));
        }

        self.attachments_dirty = self.color_textures[index] != texture;
        self.color_textures[index] = texture;
        Ok(())
    }

    pub fn get_depth_texture(&self) -> Option<&Texture> {
        self.depth_texture.as_ref()
    }

    pub fn set_depth_texture(&mut self, texture: Option<Texture>) -> Result<(), DeveloperError> {
        if self.create_depth_attachments {
            return Err(DeveloperError::new(
                "createDepthAttachments must be false if setDepthTexture is called.",
            ));
        }
        self.attachments_dirty = self.depth_texture != texture;
        self.depth_texture = texture;
        Ok(())
    }

    pub fn get_depth_renderbuffer(&self) -> Option<&Renderbuffer> {
        self.depth_renderbuffer.as_ref()
    }

    pub fn set_depth_renderbuffer(
        &mut self,
        renderbuffer: Option<Renderbuffer>,
    ) -> Result<(), DeveloperError> {
        if self.create_depth_attachments {
            return Err(DeveloperError::new(
                "createDepthAttachments must be false if setDepthRenderbuffer is called.",
            ));
        }
        self.attachments_dirty = self.depth_renderbuffer != renderbuffer;
        self.depth_renderbuffer = renderbuffer;
        Ok(())
    }

    pub fn get_depth_stencil_texture(&self) -> Option<&Texture> {
        self.depth_stencil_texture.as_ref()
    }

    pub fn set_depth_stencil_texture(
        &mut self,
        texture: Option<Texture>,
    ) -> Result<(), DeveloperError> {
        if self.create_depth_attachments {
            return Err(DeveloperError::new(
                "createDepthAttachments must be false if setDepthStencilTexture is called.",
            ));
        }
        self.attachments_dirty = self.depth_stencil_texture != texture;
        self.depth_stencil_texture = texture;
        Ok(())
    }

    pub fn get_depth_stencil_renderbuffer(&self) -> Option<&Renderbuffer> {
        self.depth_stencil_renderbuffer.as_ref()
    }

    pub fn set_depth_stencil_renderbuffer(
        &mut self,
        renderbuffer: Option<Renderbuffer>,
    ) -> Result<(), DeveloperError> {
        if self.create_depth_attachments {
            return Err(DeveloperError::new(
                "createDepthAttachments must be false if setDepthStencilRenderbuffer is called.",
            ));
        }
        self.attachments_dirty = self.depth_stencil_renderbuffer != renderbuffer;
        self.depth_stencil_renderbuffer = renderbuffer;
        Ok(())
    }

    pub fn destroy_framebuffer(&mut self) {
        if let Some(ref mut fb) = self.framebuffer {
            fb.destroy();
        }
        self.framebuffer = None;
    }

    fn destroy_resources(&mut self) {
        if self.color && self.create_color_attachments {
            for t in &mut self.color_textures {
                if let Some(ref mut tex) = t {
                    tex.destroy();
                }
                *t = None;
            }
        }
        if self.depth && self.create_depth_attachments {
            if let Some(ref mut dt) = self.depth_texture {
                dt.destroy();
            }
            self.depth_texture = None;
            if let Some(ref mut dr) = self.depth_renderbuffer {
                dr.destroy();
            }
            self.depth_renderbuffer = None;
        }
        if self.depth_stencil && self.create_depth_attachments {
            if let Some(ref mut dst) = self.depth_stencil_texture {
                dst.destroy();
            }
            self.depth_stencil_texture = None;
            if let Some(ref mut dsr) = self.depth_stencil_renderbuffer {
                dsr.destroy();
            }
            self.depth_stencil_renderbuffer = None;
        }
        self.destroy_framebuffer();
    }

    pub fn destroy(&mut self) {
        self.destroy_resources();
        self.is_destroyed = true;
    }

    pub fn is_destroyed(&self) -> bool {
        self.is_destroyed
    }
}


pub const MAXIMUM_RENDERBUFFER_SIZE: u32 = 16384;

impl Renderbuffer {
    pub fn create(options: RenderbufferOptions) -> Result<Self, DeveloperError> {
        if options.width == 0 {
            return Err(DeveloperError::new("width must be greater than zero."));
        }
        if options.width > MAXIMUM_RENDERBUFFER_SIZE {
            return Err(DeveloperError::new("width is greater than maximumRenderbufferSize."));
        }
        if options.height == 0 {
            return Err(DeveloperError::new("height must be greater than zero."));
        }
        if options.height > MAXIMUM_RENDERBUFFER_SIZE {
            return Err(DeveloperError::new("height is greater than maximumRenderbufferSize."));
        }

        Ok(Self::new(options))
    }

    pub fn destroy_checked(&mut self) -> Result<(), DeveloperError> {
        if self.is_destroyed {
            return Err(DeveloperError::new("This object was destroyed."));
        }
        self.destroy();
        Ok(())
    }
}

/// MultisampleFramebuffer wraps read and draw framebuffers for resolving multisampled renderbuffers.
#[derive(Clone, Debug)]
pub struct MultisampleFramebuffer {
    pub width: u32,
    pub height: u32,
    render_framebuffer: Framebuffer,
    color_framebuffer: Framebuffer,
    destroy_attachments: bool,
    is_destroyed: bool,
}

#[derive(Clone, Debug, Default)]
pub struct MultisampleFramebufferOptions {
    pub width: u32,
    pub height: u32,
    pub color_textures: Option<Vec<Option<Texture>>>,
    pub color_renderbuffers: Option<Vec<Option<Renderbuffer>>>,
    pub depth_stencil_texture: Option<Texture>,
    pub depth_stencil_renderbuffer: Option<Renderbuffer>,
    pub destroy_attachments: Option<bool>,
}

impl MultisampleFramebuffer {
    pub fn new(options: MultisampleFramebufferOptions) -> Result<Self, DeveloperError> {
        let has_color_rb = options.color_renderbuffers.is_some();
        let has_color_tex = options.color_textures.is_some();
        if has_color_rb != has_color_tex {
            return Err(DeveloperError::new(
                "Both color renderbuffer and texture attachments must be provided.",
            ));
        }

        let has_ds_rb = options.depth_stencil_renderbuffer.is_some();
        let has_ds_tex = options.depth_stencil_texture.is_some();
        if has_ds_rb != has_ds_tex {
            return Err(DeveloperError::new(
                "Both depth-stencil renderbuffer and texture attachments must be provided.",
            ));
        }

        let render_fb = Framebuffer::new(FramebufferOptions {
            color_renderbuffers: options.color_renderbuffers.clone().unwrap_or_default(),
            depth_stencil_renderbuffer: options.depth_stencil_renderbuffer.clone(),
            destroy_attachments: options.destroy_attachments,
            ..Default::default()
        })?;

        let color_fb = Framebuffer::new(FramebufferOptions {
            color_textures: options.color_textures.clone().unwrap_or_default(),
            depth_stencil_texture: options.depth_stencil_texture.clone(),
            destroy_attachments: options.destroy_attachments,
            ..Default::default()
        })?;

        Ok(Self {
            width: options.width,
            height: options.height,
            render_framebuffer: render_fb,
            color_framebuffer: color_fb,
            destroy_attachments: options.destroy_attachments.unwrap_or(true),
            is_destroyed: false,
        })
    }

    pub fn get_render_framebuffer(&self) -> &Framebuffer {
        &self.render_framebuffer
    }

    pub fn get_color_framebuffer(&self) -> &Framebuffer {
        &self.color_framebuffer
    }

    pub fn blit_framebuffers(&mut self, _blit_stencil: bool) {
        // Resolve blit from render_framebuffer to color_framebuffer
    }

    pub fn destroy(&mut self) {
        if self.destroy_attachments {
            self.render_framebuffer.destroy();
            self.color_framebuffer.destroy();
        }
        self.is_destroyed = true;
    }

    pub fn is_destroyed(&self) -> bool {
        self.is_destroyed
    }
}
