//! WebGL/WebGPU Rendering Context and Limits matching CesiumJS `Renderer/Context.js` (Apache-2.0).

use crate::automatic_uniforms::UniformState;
use crate::buffer::{Buffer, BufferOptions, BufferUsage};
use crate::commands::ClearCommand;
use crate::framebuffer::Framebuffer;
use crate::pick_id::PickId;
use cesium_math::{BoundingRectangle, Color, DeveloperError};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_CONTEXT_ID: AtomicU64 = AtomicU64::new(1);
static NEXT_PICK_KEY: AtomicU64 = AtomicU64::new(1);

/// ContextLimits constants reflecting platform capabilities.
pub struct ContextLimits;

impl ContextLimits {
    pub const MAXIMUM_COMBINED_TEXTURE_IMAGE_UNITS: u32 = 32;
    pub const MAXIMUM_CUBE_MAP_SIZE: u32 = 2048;
    pub const MAXIMUM_FRAGMENT_UNIFORM_VECTORS: u32 = 224;
    pub const MAXIMUM_TEXTURE_IMAGE_UNITS: u32 = 16;
    pub const MAXIMUM_RENDERBUFFER_SIZE: u32 = 2048;
    pub const MAXIMUM_TEXTURE_SIZE: u32 = 2048;
    pub const MAXIMUM_3D_TEXTURE_SIZE: u32 = 256;
    pub const MAXIMUM_VARYING_VECTORS: u32 = 15;
    pub const MAXIMUM_VERTEX_ATTRIBUTES: u32 = 16;
    pub const MAXIMUM_VERTEX_TEXTURE_IMAGE_UNITS: u32 = 16;
    pub const MAXIMUM_VERTEX_UNIFORM_VECTORS: u32 = 256;
    pub const MINIMUM_ALIASED_LINE_WIDTH: f32 = 1.0;
    pub const MAXIMUM_ALIASED_LINE_WIDTH: f32 = 1.0;
    pub const MINIMUM_ALIASED_POINT_SIZE: f32 = 1.0;
    pub const MAXIMUM_ALIASED_POINT_SIZE: f32 = 1.0;
    pub const MAXIMUM_VIEWPORT_WIDTH: u32 = 8192;
    pub const MAXIMUM_VIEWPORT_HEIGHT: u32 = 8192;
    pub const MAXIMUM_TEXTURE_FILTER_ANISOTROPY: f32 = 16.0;
    pub const MAXIMUM_DRAW_BUFFERS: u32 = 8;
    pub const MAXIMUM_COLOR_ATTACHMENTS: u32 = 8;
}

/// Options for configuring a rendering Context.
#[derive(Clone, Debug, Default)]
pub struct ContextOptions {
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub antialias: Option<bool>,
    pub request_webgl1: Option<bool>,
}

/// Options for reading pixels.
#[derive(Clone, Debug)]
pub struct ReadPixelsOptions {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
    pub framebuffer: Option<Rc<Framebuffer>>,
}

impl Default for ReadPixelsOptions {
    fn default() -> Self {
        Self {
            x: 0,
            y: 0,
            width: 1,
            height: 1,
            framebuffer: None,
        }
    }
}

/// Rendering context managing resources, state, pixel reads, and command execution.
pub struct Context {
    pub id: u64,
    pub drawing_buffer_width: u32,
    pub drawing_buffer_height: u32,
    pub antialias: bool,
    pub webgl2: bool,
    pub stencil_bits: u32,
    pub standard_derivatives: bool,
    pub element_index_uint: bool,
    pub depth_texture: bool,
    pub floating_point_texture: bool,
    pub texture_filter_anisotropic: bool,
    pub vertex_array_object: bool,
    pub fragment_depth: bool,
    pub draw_buffers: bool,
    pub uniform_state: UniformState,
    pub pick_objects: Rc<RefCell<HashMap<u32, String>>>,
    pub color_buffer: Vec<u8>,
    pub framebuffer_colors: HashMap<u64, Vec<u8>>,
    pub is_destroyed: bool,
}

impl Default for Context {
    fn default() -> Self {
        Self::new(ContextOptions::default()).expect("Valid default Context creation")
    }
}

impl Context {
    /// Creates a new rendering Context.
    pub fn new(options: ContextOptions) -> Result<Self, DeveloperError> {
        let width = options.width.unwrap_or(1024);
        let height = options.height.unwrap_or(768);
        if width == 0 || height == 0 {
            return Err(DeveloperError::new(
                "drawingBufferWidth and drawingBufferHeight must be greater than zero.",
            ));
        }

        let is_webgl1 = options.request_webgl1.unwrap_or(false);
        let webgl2 = !is_webgl1;
        let antialias = options.antialias.unwrap_or(true);
        let id = NEXT_CONTEXT_ID.fetch_add(1, Ordering::Relaxed);

        // Initial default buffer is black transparent (0, 0, 0, 0)
        let total_bytes = (width * height * 4) as usize;
        let color_buffer = vec![0u8; total_bytes];

        Ok(Self {
            id,
            drawing_buffer_width: width,
            drawing_buffer_height: height,
            antialias,
            webgl2,
            stencil_bits: 8,
            standard_derivatives: true,
            element_index_uint: true,
            depth_texture: true,
            floating_point_texture: true,
            texture_filter_anisotropic: true,
            vertex_array_object: true,
            fragment_depth: true,
            draw_buffers: true,
            uniform_state: UniformState::default(),
            pick_objects: Rc::new(RefCell::new(HashMap::new())),
            color_buffer,
            framebuffer_colors: HashMap::new(),
            is_destroyed: false,
        })
    }

    /// Creates a pick ID associated with an object identifier.
    pub fn create_pick_id(&mut self, object: Option<String>) -> Result<PickId<String>, DeveloperError> {
        let object_key = object.ok_or_else(|| DeveloperError::new("object is required."))?;
        let key = NEXT_PICK_KEY.fetch_add(1, Ordering::Relaxed) as u32;

        let r = ((key & 0x000000FF) as f32) / 255.0;
        let g = (((key & 0x0000FF00) >> 8) as f32) / 255.0;
        let b = (((key & 0x00FF0000) >> 16) as f32) / 255.0;
        let a = (((key & 0xFF000000) >> 24) as f32) / 255.0;
        let color = Color::new(r, g, b, a);

        self.pick_objects.borrow_mut().insert(key, object_key);
        Ok(PickId::new(self.pick_objects.clone(), key, color))
    }

    /// Retrieves the object associated with a pick color.
    pub fn get_object_by_pick_color(&self, color: Option<Color>) -> Result<Option<String>, DeveloperError> {
        let col = color.ok_or_else(|| DeveloperError::new("color is required."))?;
        let r = (col.red * 255.0).round() as u32;
        let g = (col.green * 255.0).round() as u32;
        let b = (col.blue * 255.0).round() as u32;
        let a = (col.alpha * 255.0).round() as u32;

        let key = (a << 24) | (b << 16) | (g << 8) | r;
        Ok(self.pick_objects.borrow().get(&key).cloned())
    }

    /// Executes a clear command against the current context or target framebuffer.
    pub fn clear(&mut self, command: &ClearCommand) -> Result<(), DeveloperError> {
        if self.is_destroyed {
            return Err(DeveloperError::new(
                "This object was destroyed, i.e., destroy() was called.",
            ));
        }

        let clear_color = command.color.unwrap_or(Color::new(0.0, 0.0, 0.0, 1.0));
        let clear_r = (clear_color.red * 255.0).clamp(0.0, 255.0) as u8;
        let clear_g = (clear_color.green * 255.0).clamp(0.0, 255.0) as u8;
        let clear_b = (clear_color.blue * 255.0).clamp(0.0, 255.0) as u8;
        let clear_a = (clear_color.alpha * 255.0).clamp(0.0, 255.0) as u8;

        let mut mask_r = true;
        let mut mask_g = true;
        let mut mask_b = true;
        let mut mask_a = true;

        let mut scissor_rect: Option<BoundingRectangle> = None;

        if let Some(rs) = &command.render_state {
            mask_r = rs.color_mask.red;
            mask_g = rs.color_mask.green;
            mask_b = rs.color_mask.blue;
            mask_a = rs.color_mask.alpha;

            if rs.scissor_test.enabled {
                scissor_rect = Some(rs.scissor_test.rectangle);
            }
        }

        let width = self.drawing_buffer_width;
        let height = self.drawing_buffer_height;

        for y in 0..height {
            for x in 0..width {
                if let Some(rect) = scissor_rect {
                    let rx = rect.x as u32;
                    let ry = rect.y as u32;
                    let rw = rect.width as u32;
                    let rh = rect.height as u32;
                    let inside = x >= rx && x < rx + rw && y >= ry && y < ry + rh;
                    if !inside {
                        continue;
                    }
                }

                let idx = ((y * width + x) * 4) as usize;
                if idx + 4 <= self.color_buffer.len() {
                    if mask_r {
                        self.color_buffer[idx] = clear_r;
                    }
                    if mask_g {
                        self.color_buffer[idx + 1] = clear_g;
                    }
                    if mask_b {
                        self.color_buffer[idx + 2] = clear_b;
                    }
                    if mask_a {
                        self.color_buffer[idx + 3] = clear_a;
                    }
                }
            }
        }

        // If target framebuffer has color textures, clear the framebuffer color record
        if let Some(fb) = &command.framebuffer {
            let width = fb.color_textures.first().and_then(|t| t.as_ref()).map(|t| t.width).unwrap_or(1);
            let height = fb.color_textures.first().and_then(|t| t.as_ref()).map(|t| t.height).unwrap_or(1);
            let total = (width * height * 4) as usize;
            let mut fb_data = vec![0u8; total];
            for chunk in fb_data.as_chunks_mut::<4>().0 {
                chunk[0] = clear_r;
                chunk[1] = clear_g;
                chunk[2] = clear_b;
                chunk[3] = clear_a;
            }
            self.framebuffer_colors.insert(fb.id, fb_data);
        }

        Ok(())
    }

    /// Reads RGBA pixel data from the framebuffer.
    pub fn read_pixels(&self, options: Option<ReadPixelsOptions>) -> Result<Vec<u8>, DeveloperError> {
        if self.is_destroyed {
            return Err(DeveloperError::new(
                "This object was destroyed, i.e., destroy() was called.",
            ));
        }

        let opts = options.unwrap_or_default();
        if opts.width <= 0 || opts.height <= 0 {
            return Err(DeveloperError::new(
                "width and height must be greater than zero.",
            ));
        }

        if let Some(fb) = opts.framebuffer {
            if let Some(data) = self.framebuffer_colors.get(&fb.id) {
                return Ok(data.clone());
            }
        }

        let w = opts.width as usize;
        let h = opts.height as usize;
        let count = w * h * 4;

        if count <= self.color_buffer.len() {
            Ok(self.color_buffer[0..count].to_vec())
        } else {
            Ok(vec![0u8; count])
        }
    }

    /// Reads pixels asynchronously into a PixelBuffer (PBO).
    pub fn read_pixels_to_pbo(&self, options: Option<ReadPixelsOptions>) -> Result<Buffer, DeveloperError> {
        if !self.webgl2 {
            return Err(DeveloperError::new(
                "A WebGL 2 context is required to read pixels using a PBO.",
            ));
        }

        let pixels = self.read_pixels(options)?;
        Buffer::create_pixel_buffer(BufferOptions {
            size_in_bytes: 0,
            usage: Some(BufferUsage::StaticDraw),
            typed_array: Some(pixels),
        })
    }

    pub fn is_destroyed(&self) -> bool {
        self.is_destroyed
    }

    pub fn destroy(&mut self) -> Result<(), DeveloperError> {
        if self.is_destroyed {
            return Err(DeveloperError::new(
                "This object was destroyed, i.e., destroy() was called.",
            ));
        }
        self.is_destroyed = true;
        self.pick_objects.borrow_mut().clear();
        self.color_buffer.clear();
        self.color_buffer.shrink_to_fit();
        self.framebuffer_colors.clear();
        Ok(())
    }
}
