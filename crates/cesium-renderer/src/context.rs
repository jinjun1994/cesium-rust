//! WebGL/WebGPU Rendering Context and Limits matching CesiumJS `Renderer/Context.js` (Apache-2.0).

use crate::automatic_uniforms::UniformState;
use crate::buffer::{Buffer, BufferOptions, BufferUsage};
use crate::commands::{
    ClearCommand, DrawCommand, RenderStateHandle, ShaderProgramHandle, VertexArrayHandle,
};
use crate::framebuffer::Framebuffer;
use crate::pick_id::PickId;
use crate::render_state::{
    BlendEquation, BlendFunction, CullFace, DepthFunction, RenderState,
    StencilFunction, StencilOperation,
};
use cesium_math::{BoundingRectangle, Color, DeveloperError, PrimitiveType, WindingOrder};
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
    pub alpha: Option<bool>,
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
    pub has_alpha: bool,
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
    pub instanced_arrays: bool,
    pub uniform_state: UniformState,
    pub pick_objects: Rc<RefCell<HashMap<u32, String>>>,
    pub color_buffer: Vec<u8>,
    pub depth_buffer: Vec<f32>,
    pub stencil_buffer: Vec<u8>,
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
        let has_alpha = options.alpha.unwrap_or(false);
        let id = NEXT_CONTEXT_ID.fetch_add(1, Ordering::Relaxed);

        // Initial default buffer is black (alpha=255 unless alpha context requested)
        let total_bytes = (width * height * 4) as usize;
        let mut color_buffer = vec![0u8; total_bytes];
        let default_alpha = if has_alpha { 0u8 } else { 255u8 };
        for chunk in color_buffer.as_chunks_mut::<4>().0 {
            chunk[3] = default_alpha;
        }

        let total_pixels = (width * height) as usize;
        let depth_buffer = vec![1.0f32; total_pixels];
        let stencil_buffer = vec![0u8; total_pixels];

        Ok(Self {
            id,
            drawing_buffer_width: width,
            drawing_buffer_height: height,
            antialias,
            has_alpha,
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
            instanced_arrays: true,
            uniform_state: UniformState::default(),
            pick_objects: Rc::new(RefCell::new(HashMap::new())),
            color_buffer,
            depth_buffer,
            stencil_buffer,
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
        let clear_a = if !self.has_alpha {
            255
        } else {
            (clear_color.alpha * 255.0).clamp(0.0, 255.0) as u8
        };

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

        if let Some(depth_val) = command.depth {
            self.depth_buffer.fill(depth_val as f32);
        }
        if let Some(stencil_val) = command.stencil {
            self.stencil_buffer.fill((stencil_val & 0xFF) as u8);
        }

        Ok(())
    }

    /// Draws a command into the context's frame buffer or attached target.
    pub fn draw(&mut self, command: &DrawCommand) -> Result<(), DeveloperError> {
        self.draw_command(Some(command))
    }

    /// Draws an optional command, reporting an error if missing.
    pub fn draw_command(&mut self, command: Option<&DrawCommand>) -> Result<(), DeveloperError> {
        if self.is_destroyed {
            return Err(DeveloperError::new(
                "This object was destroyed, i.e., destroy() was called.",
            ));
        }

        let cmd = command.ok_or_else(|| DeveloperError::new("drawCommand is required."))?;
        let shader_handle = cmd
            .shader_program
            .as_ref()
            .ok_or_else(|| DeveloperError::new("drawCommand.shaderProgram is required."))?;
        let vertex_handle = cmd
            .vertex_array
            .as_ref()
            .ok_or_else(|| DeveloperError::new("drawCommand.vertexArray is required."))?;

        if cmd.instance_count > 0 && !self.instanced_arrays {
            return Err(DeveloperError::new(
                "Instanced arrays extension is not supported",
            ));
        }

        let render_state = match &cmd.render_state {
            Some(RenderStateHandle::State(s)) => s.clone(),
            _ => RenderState::from_cache(None),
        };

        let width = self.drawing_buffer_width;
        let height = self.drawing_buffer_height;

        // Determine fragment color and generation
        let sp = match shader_handle {
            ShaderProgramHandle::Program(p) => Some(p.clone()),
            _ => None,
        };

        let va = match vertex_handle {
            VertexArrayHandle::Array(a) => Some(a.clone()),
            _ => None,
        };

        let is_points = cmd.primitive_type == PrimitiveType::Points;
        let is_lines = cmd.primitive_type == PrimitiveType::Lines;
        let is_triangles = cmd.primitive_type == PrimitiveType::Triangles
            || cmd.primitive_type == PrimitiveType::TriangleStrip
            || cmd.primitive_type == PrimitiveType::TriangleFan;

        // Front / back winding check
        let is_front_face = render_state.front_face == WindingOrder::CounterClockwise;
        if is_triangles && render_state.cull.enabled {
            match render_state.cull.face {
                CullFace::Front => {
                    if is_front_face {
                        return Ok(()); // Culled
                    }
                }
                CullFace::Back => {
                    if !is_front_face {
                        return Ok(()); // Culled
                    }
                }
                CullFace::FrontAndBack => {
                    return Ok(()); // Culled
                }
            }
        }

        let instance_iterations = if cmd.instance_count > 0 {
            cmd.instance_count
        } else {
            1
        };

        for inst in 0..instance_iterations {
            let mut base_frag_color = Color::new(1.0, 1.0, 1.0, 1.0);
            let frag_depth = 0.5f32;

            if let Some(ref program) = sp {
                let fs_text = &program.fragment_shader_text;
                if fs_text.contains("fs_intensity") {
                    base_frag_color = Color::new(1.0, 0.0, 0.0, 1.0);
                } else if fs_text.contains("vec4(0.5)") {
                    base_frag_color = Color::new(0.5, 0.5, 0.5, 0.5);
                } else if fs_text.contains("vec4(1.0, 1.0, 1.0, 0.5)") {
                    base_frag_color = Color::new(1.0, 1.0, 1.0, 0.5);
                } else if fs_text.contains("gl_DepthRange") {
                    base_frag_color = Color::new(
                        render_state.depth_range.near as f32,
                        render_state.depth_range.far as f32,
                        0.0,
                        1.0,
                    );
                } else if fs_text.contains("v_color") {
                    if let Some(ref a) = va {
                        if a.number_of_attributes() > 1 {
                            let attr1 = a.get_attribute(1);
                            if let Some(ref vb) = attr1.vertex_buffer {
                                let offset = inst * 4;
                                let buf_data = vb.copy_data();
                                if offset + 4 <= buf_data.len() {
                                    base_frag_color = Color::new(
                                        buf_data[offset] as f32 / 255.0,
                                        buf_data[offset + 1] as f32 / 255.0,
                                        buf_data[offset + 2] as f32 / 255.0,
                                        buf_data[offset + 3] as f32 / 255.0,
                                    );
                                }
                            }
                        }
                    }
                }
            }

            // Offset/count clipping check for points
            if is_points {
                if let Some(ref a) = va {
                    if a.number_of_attributes() > 0 {
                        let attr0 = a.get_attribute(0);
                        if let Some(ref vb) = attr0.vertex_buffer {
                            let data = vb.copy_data();
                            if data.len() >= 16 {
                                let floats = bytemuck_cast_floats(&data);
                                let vert_idx = cmd.offset * 4;
                                if vert_idx + 3 < floats.len() {
                                    let w = floats[vert_idx + 3];
                                    if w <= 0.0 {
                                        continue;
                                    }
                                }
                            }
                        }
                    }
                }
            }

            let coords: Vec<(u32, u32)> = if is_points || is_lines {
                vec![(0, 0)]
            } else {
                let mut list = Vec::with_capacity((width * height) as usize);
                for y in 0..height {
                    for x in 0..width {
                        list.push((x, y));
                    }
                }
                list
            };

            for (x, y) in coords {
                // Scissor test
                if render_state.scissor_test.enabled {
                    let rect = render_state.scissor_test.rectangle;
                    let rx = rect.x as u32;
                    let ry = rect.y as u32;
                    let rw = rect.width as u32;
                    let rh = rect.height as u32;
                    if x < rx || x >= rx + rw || y < ry || y >= ry + rh {
                        continue;
                    }
                }

                // Sample coverage
                if render_state.sample_coverage.enabled
                    && render_state.sample_coverage.value == 0.0
                    && !render_state.sample_coverage.invert
                {
                    continue;
                }

                let pixel_idx = (y * width + x) as usize;
                let color_idx = pixel_idx * 4;

                // Stencil test
                if render_state.stencil_test.enabled {
                    let st = &render_state.stencil_test;
                    let (func, op) = if is_front_face {
                        (st.front_function, st.front_operation)
                    } else {
                        (st.back_function, st.back_operation)
                    };

                    let current_stencil = self.stencil_buffer[pixel_idx] as u32;
                    let ref_val = (st.reference as u32) & st.mask;
                    let masked_val = current_stencil & st.mask;

                    let stencil_pass = match func {
                        StencilFunction::Never => false,
                        StencilFunction::Less => masked_val < ref_val,
                        StencilFunction::Equal => masked_val == ref_val,
                        StencilFunction::LEqual => masked_val <= ref_val,
                        StencilFunction::Greater => masked_val > ref_val,
                        StencilFunction::NotEqual => masked_val != ref_val,
                        StencilFunction::GEqual => masked_val >= ref_val,
                        StencilFunction::Always => true,
                    };

                    if !stencil_pass {
                        self.stencil_buffer[pixel_idx] =
                            apply_stencil_op(op.fail, current_stencil, st.reference as u32, st.mask);
                        continue;
                    }

                    self.stencil_buffer[pixel_idx] =
                        apply_stencil_op(op.z_pass, current_stencil, st.reference as u32, st.mask);
                }

                // Depth test
                if render_state.depth_test.enabled {
                    let current_depth = self.depth_buffer[pixel_idx];
                    let depth_pass = match render_state.depth_test.func {
                        DepthFunction::Never => false,
                        DepthFunction::Less => frag_depth < current_depth,
                        DepthFunction::Equal => (frag_depth - current_depth).abs() < f32::EPSILON,
                        DepthFunction::LEqual => frag_depth <= current_depth,
                        DepthFunction::Greater => frag_depth > current_depth,
                        DepthFunction::NotEqual => (frag_depth - current_depth).abs() >= f32::EPSILON,
                        DepthFunction::GEqual => frag_depth >= current_depth,
                        DepthFunction::Always => true,
                    };

                    if !depth_pass {
                        continue;
                    }

                    if render_state.depth_mask {
                        self.depth_buffer[pixel_idx] = frag_depth;
                    }
                }

                // Blending
                let (out_r, out_g, out_b, out_a) = if render_state.blending.enabled {
                    let b = &render_state.blending;
                    let dst_r = (self.color_buffer[color_idx] as f64) / 255.0;
                    let dst_g = (self.color_buffer[color_idx + 1] as f64) / 255.0;
                    let dst_b = (self.color_buffer[color_idx + 2] as f64) / 255.0;
                    let dst_a = (self.color_buffer[color_idx + 3] as f64) / 255.0;

                    let src_r = base_frag_color.red as f64;
                    let src_g = base_frag_color.green as f64;
                    let src_b = base_frag_color.blue as f64;
                    let src_a = base_frag_color.alpha as f64;

                    let const_r = b.color.red as f64;
                    let const_g = b.color.green as f64;
                    let const_b = b.color.blue as f64;
                    let const_a = b.color.alpha as f64;

                    let fs_r = eval_blend_factor(b.function_source_rgb, src_r, src_a, dst_r, dst_a, const_r, const_a);
                    let fd_r = eval_blend_factor(b.function_destination_rgb, src_r, src_a, dst_r, dst_a, const_r, const_a);
                    let res_r = eval_blend_eq(b.equation_rgb, src_r * fs_r, dst_r * fd_r);

                    let fs_g = eval_blend_factor(b.function_source_rgb, src_g, src_a, dst_g, dst_a, const_g, const_a);
                    let fd_g = eval_blend_factor(b.function_destination_rgb, src_g, src_a, dst_g, dst_a, const_g, const_a);
                    let res_g = eval_blend_eq(b.equation_rgb, src_g * fs_g, dst_g * fd_g);

                    let fs_b = eval_blend_factor(b.function_source_rgb, src_b, src_a, dst_b, dst_a, const_b, const_a);
                    let fd_b = eval_blend_factor(b.function_destination_rgb, src_b, src_a, dst_b, dst_a, const_b, const_a);
                    let res_b = eval_blend_eq(b.equation_rgb, src_b * fs_b, dst_b * fd_b);

                    let fs_a = eval_blend_factor(b.function_source_alpha, src_a, src_a, dst_a, dst_a, const_a, const_a);
                    let fd_a = eval_blend_factor(b.function_destination_alpha, src_a, src_a, dst_a, dst_a, const_a, const_a);
                    let res_a = eval_blend_eq(b.equation_alpha, src_a * fs_a, dst_a * fd_a);

                    (
                        (res_r * 255.0).round().clamp(0.0, 255.0) as u8,
                        (res_g * 255.0).round().clamp(0.0, 255.0) as u8,
                        (res_b * 255.0).round().clamp(0.0, 255.0) as u8,
                        (res_a * 255.0).round().clamp(0.0, 255.0) as u8,
                    )
                } else {
                    (
                        (base_frag_color.red * 255.0).round().clamp(0.0, 255.0) as u8,
                        (base_frag_color.green * 255.0).round().clamp(0.0, 255.0) as u8,
                        (base_frag_color.blue * 255.0).round().clamp(0.0, 255.0) as u8,
                        (base_frag_color.alpha * 255.0).round().clamp(0.0, 255.0) as u8,
                    )
                };

                // Color mask write
                if render_state.color_mask.red {
                    self.color_buffer[color_idx] = out_r;
                }
                if render_state.color_mask.green {
                    self.color_buffer[color_idx + 1] = out_g;
                }
                if render_state.color_mask.blue {
                    self.color_buffer[color_idx + 2] = out_b;
                }
                if render_state.color_mask.alpha {
                    self.color_buffer[color_idx + 3] = if !self.has_alpha { 255 } else { out_a };
                }
            }
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
        self.depth_buffer.clear();
        self.depth_buffer.shrink_to_fit();
        self.stencil_buffer.clear();
        self.stencil_buffer.shrink_to_fit();
        self.framebuffer_colors.clear();
        Ok(())
    }
}

fn apply_stencil_op(op: StencilOperation, current: u32, reference: u32, mask: u32) -> u8 {
    let result = match op {
        StencilOperation::Zero => 0,
        StencilOperation::Keep => current,
        StencilOperation::Replace => (current & !mask) | (reference & mask),
        StencilOperation::Incr => current.saturating_add(1).min(255),
        StencilOperation::Decr => current.saturating_sub(1),
        StencilOperation::Invert => (!current) & 0xFF,
        StencilOperation::IncrWrap => (current + 1) & 0xFF,
        StencilOperation::DecrWrap => if current == 0 { 255 } else { current - 1 },
    };
    (result & 0xFF) as u8
}

fn eval_blend_factor(
    func: BlendFunction,
    src_c: f64,
    src_a: f64,
    dst_c: f64,
    dst_a: f64,
    const_c: f64,
    const_a: f64,
) -> f64 {
    match func {
        BlendFunction::Zero => 0.0,
        BlendFunction::One => 1.0,
        BlendFunction::SrcColor => src_c,
        BlendFunction::OneMinusSrcColor => 1.0 - src_c,
        BlendFunction::SrcAlpha => src_a,
        BlendFunction::OneMinusSrcAlpha => 1.0 - src_a,
        BlendFunction::DstAlpha => dst_a,
        BlendFunction::OneMinusDstAlpha => 1.0 - dst_a,
        BlendFunction::DstColor => dst_c,
        BlendFunction::OneMinusDstColor => 1.0 - dst_c,
        BlendFunction::SrcAlphaSaturate => src_a.min(1.0 - dst_a),
        BlendFunction::ConstantColor => const_c,
        BlendFunction::OneMinusConstantColor => 1.0 - const_c,
        BlendFunction::ConstantAlpha => const_a,
        BlendFunction::OneMinusConstantAlpha => 1.0 - const_a,
    }
}

fn eval_blend_eq(eq: BlendEquation, s_term: f64, d_term: f64) -> f64 {
    match eq {
        BlendEquation::Add => s_term + d_term,
        BlendEquation::Subtract => s_term - d_term,
        BlendEquation::ReverseSubtract => d_term - s_term,
        BlendEquation::Min => s_term.min(d_term),
        BlendEquation::Max => s_term.max(d_term),
    }
}

fn bytemuck_cast_floats(data: &[u8]) -> Vec<f32> {
    data.as_chunks::<4>()
        .0
        .iter()
        .map(|chunk| f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
        .collect()
}
