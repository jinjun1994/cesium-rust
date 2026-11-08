//! wgpu 30 renderer: core PBR, sorted alpha, paged dynamic uniforms,
//! robust device state recovery, and multi-tier frame performance telemetry.

pub mod automatic_uniforms;
pub mod buffer;
pub mod capabilities;
pub mod commands;
pub mod compute_command;
pub mod context;
pub mod cube_map;
pub mod demodernize_shader;
pub mod device;
pub mod framebuffer;
pub mod metrics;
pub mod pass_state;
pub mod pick_id;
pub mod picking;
pub mod pipelines;
pub mod render_state;
pub mod sampler;
pub mod shader_builder;
pub mod shader_cache;
pub mod shader_source;
pub mod sync;
pub mod texture;
pub mod texture_3d;
pub mod texture_atlas;
pub mod vertex_array;
mod screen_distance;
pub mod uniform;
pub mod uniforms;

use cesium_formats::Primitive;
use cesium_math::{linear_part, relative_model_view, Camera};
use glam::{DMat4, DVec3};
use std::num::NonZeroU64;
use std::sync::mpsc;
use thiserror::Error;
use wgpu::util::DeviceExt;

pub use automatic_uniforms::{AutomaticUniform, AutomaticUniforms, UniformState};
pub use buffer::{Buffer, BufferOptions, BufferTarget, BufferUsage, IndexBufferOptions};
pub use capabilities::RendererCapabilities;
pub use commands::{
    ClearCommand, ClearCommandOptions, DrawCommand, DrawCommandOptions, Pass, NUMBER_OF_PASSES,
};
pub use compute_command::{ComputeCommand, ComputeCommandOptions};
pub use context::{Context, ContextLimits, ContextOptions, ReadPixelsOptions};
pub use cube_map::{CubeMap, CubeMapFace, CubeMapOptions};
pub use demodernize_shader::demodernize_shader;
pub use device::{DeviceContext, DeviceStatus};
pub use framebuffer::{
    Framebuffer, FramebufferManager, FramebufferManagerOptions, FramebufferOptions,
    MultisampleFramebuffer, MultisampleFramebufferOptions, PixelDatatype, PixelFormat,
    Renderbuffer, RenderbufferFormat, RenderbufferOptions, MAXIMUM_RENDERBUFFER_SIZE,
};
pub use sync::{Sync, SyncStatus};
pub use uniform::{Uniform, UniformGetter, UniformMap, UniformType, UniformValue};
pub use texture::{CachedTextureEntry, Texture, TextureCache, TextureOptions};
pub use texture_3d::{Texture3D, Texture3DOptions};
pub use texture_atlas::{TextureAtlas, TextureAtlasOptions};
pub use vertex_array::{
    VertexArray, VertexArrayFacade, VertexArrayFacadeAttribute,
    VertexArrayFacadeItem, VertexArrayFromGeometryOptions, VertexArrayOptions, VertexAttribute,
    VertexAttributeOptions,
};
pub use metrics::{FrameStats, FrameStatsHistory, FrameStatsSummary, StageTimer};
pub use pass_state::PassState;
pub use pick_id::PickId;
pub use picking::FeatureRef;
pub use render_state::{
    freeze_render_state, BlendEquation, BlendFunction, BlendingState, ColorMaskState, CullFace,
    CullState, DepthFunction, DepthRangeState, DepthTestState, PolygonOffsetState, RenderState,
    SampleCoverageState, ScissorTestState, StencilFunction, StencilOperation, StencilOperationState,
    StencilTestState,
};
pub use sampler::{
    Sampler, SamplerOptions, TextureMagnificationFilter, TextureMinificationFilter, TextureWrap,
};
pub use shader_builder::{
    BuiltShaderProgram, ShaderBuilder, ShaderDestination, ShaderFunction, ShaderStruct,
};
pub use shader_cache::{
    CachedShaderEntry, ShaderCache, ShaderProgram, ShaderProgramOptions, ShaderSourceInput,
    VertexAttributeDesc,
};
pub use shader_source::{ShaderContextInfo, ShaderSource, ShaderSourceOptions};
use picking::{PendingPick, PickRange};
use pipelines::{PipelineCache, PipelineKey};
pub use screen_distance::ScreenDistance;
pub use uniforms::{
    GlobalUniform, MaterialUniform, ObjectUniform, PagedObjectUniforms, DEFAULT_PAGE_CAPACITY,
};

#[derive(Debug, Error)]
#[error("renderer: {0}")]
pub struct RenderError(pub String);

pub fn instance() -> wgpu::Instance {
    let mut desc = wgpu::InstanceDescriptor::new_without_display_handle();
    #[cfg(target_arch = "wasm32")]
    {
        desc.backends = wgpu::Backends::BROWSER_WEBGPU;
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        desc.backends = wgpu::Backends::PRIMARY;
    }
    wgpu::Instance::new(desc)
}

/// Canvas commonly advertises only UNORM; an sRGB view avoids duplicate/manual gamma.
pub fn surface_formats(
    caps: &wgpu::SurfaceCapabilities,
) -> Result<(wgpu::TextureFormat, wgpu::TextureFormat), RenderError> {
    let base = caps
        .formats
        .iter()
        .copied()
        .find(|f| f.is_srgb())
        .or_else(|| {
            caps.formats.iter().copied().find(|f| {
                matches!(
                    f,
                    wgpu::TextureFormat::Rgba8Unorm | wgpu::TextureFormat::Bgra8Unorm
                )
            })
        })
        .ok_or_else(|| RenderError("no compatible 8-bit color surface".into()))?;
    Ok((base, base.add_srgb_suffix()))
}

pub async fn device(
    instance: &wgpu::Instance,
    surface: Option<&wgpu::Surface<'_>>,
) -> Result<(wgpu::Adapter, wgpu::Device, wgpu::Queue), RenderError> {
    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            force_fallback_adapter: false,
            compatible_surface: surface,
            ..Default::default()
        })
        .await
        .map_err(|e| RenderError(format!("adapter: {e}")))?;
    let limits = wgpu::Limits::default();
    if !limits.check_limits(&adapter.limits()) {
        return Err(RenderError("adapter below baseline WebGPU limits".into()));
    }
    let (device, queue) = adapter
        .request_device(&wgpu::DeviceDescriptor {
            label: Some("cesium-rs baseline"),
            required_features: wgpu::Features::empty(),
            required_limits: limits,
            ..Default::default()
        })
        .await
        .map_err(|e| RenderError(format!("device: {e}")))?;
    Ok((adapter, device, queue))
}

/// Color data is encoded as sRGB RGBA8. Dimensions and byte count are validated.
pub struct BaseColorTexture<'a> {
    pub width: u32,
    pub height: u32,
    pub rgba: &'a [u8],
}

fn color_texture(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    data: &BaseColorTexture<'_>,
) -> Result<wgpu::TextureView, RenderError> {
    let bytes = (data.width as usize)
        .checked_mul(data.height as usize)
        .and_then(|n| n.checked_mul(4))
        .ok_or_else(|| RenderError("texture size overflow".into()))?;
    if data.width == 0
        || data.height == 0
        || data.width > device.limits().max_texture_dimension_2d
        || data.height > device.limits().max_texture_dimension_2d
        || data.rgba.len() != bytes
    {
        return Err(RenderError("invalid texture dimensions/bytes".into()));
    }
    let texture = attachment(
        device,
        data.width,
        data.height,
        wgpu::TextureFormat::Rgba8UnormSrgb,
        wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
    );
    queue.write_texture(
        wgpu::TexelCopyTextureInfo {
            texture: &texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        data.rgba,
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(data.width * 4),
            rows_per_image: Some(data.height),
        },
        wgpu::Extent3d {
            width: data.width,
            height: data.height,
            depth_or_array_layers: 1,
        },
    );
    Ok(texture.create_view(&Default::default()))
}

fn material_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
    let mut entries = vec![wgpu::BindGroupLayoutEntry {
        binding: 0,
        visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
        ty: wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Uniform,
            has_dynamic_offset: false,
            min_binding_size: NonZeroU64::new(128),
        },
        count: None,
    }];
    for slot in 0..5 {
        entries.push(wgpu::BindGroupLayoutEntry {
            binding: 1 + slot * 2,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Texture {
                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                view_dimension: wgpu::TextureViewDimension::D2,
                multisampled: false,
            },
            count: None,
        });
        entries.push(wgpu::BindGroupLayoutEntry {
            binding: 2 + slot * 2,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
            count: None,
        });
    }
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("PBR material layout"),
        entries: &entries,
    })
}

fn texture_sampler(device: &wgpu::Device, s: cesium_formats::TextureSampler) -> wgpu::Sampler {
    let wrap = |v| match v {
        33071 => wgpu::AddressMode::ClampToEdge,
        33648 => wgpu::AddressMode::MirrorRepeat,
        _ => wgpu::AddressMode::Repeat,
    };
    let nearest = matches!(s.min, 9728 | 9984 | 9986);
    device.create_sampler(&wgpu::SamplerDescriptor {
        label: Some("glTF sampler"),
        address_mode_u: wrap(s.wrap_s),
        address_mode_v: wrap(s.wrap_t),
        mag_filter: if s.mag == 9728 {
            wgpu::FilterMode::Nearest
        } else {
            wgpu::FilterMode::Linear
        },
        min_filter: if nearest {
            wgpu::FilterMode::Nearest
        } else {
            wgpu::FilterMode::Linear
        },
        mipmap_filter: if matches!(s.min, 9986 | 9987) {
            wgpu::MipmapFilterMode::Linear
        } else {
            wgpu::MipmapFilterMode::Nearest
        },
        lod_max_clamp: if s.min == 9728 || s.min == 9729 {
            0.0
        } else {
            32.0
        },
        ..Default::default()
    })
}

/// Complete box-filtered mip chain; color maps filter in linear light, scalar maps stay linear.
fn pbr_texture(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    image: &cesium_formats::TextureImage,
    srgb: bool,
) -> Result<wgpu::TextureView, RenderError> {
    let (mut w, mut h) = (image.width, image.height);
    if w == 0
        || h == 0
        || w > device.limits().max_texture_dimension_2d
        || h > device.limits().max_texture_dimension_2d
        || image.rgba.len() != w as usize * h as usize * 4
    {
        return Err(RenderError("invalid PBR image".into()));
    }
    let count = 32 - w.max(h).leading_zeros();
    let tex = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("glTF texture + mips"),
        size: wgpu::Extent3d {
            width: w,
            height: h,
            depth_or_array_layers: 1,
        },
        mip_level_count: count,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: if srgb {
            wgpu::TextureFormat::Rgba8UnormSrgb
        } else {
            wgpu::TextureFormat::Rgba8Unorm
        },
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });
    let mut data = image.rgba.clone();
    for level in 0..count {
        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &tex,
                mip_level: level,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &data,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(w * 4),
                rows_per_image: Some(h),
            },
            wgpu::Extent3d {
                width: w,
                height: h,
                depth_or_array_layers: 1,
            },
        );
        if level + 1 == count {
            break;
        }
        let (nw, nh) = ((w / 2).max(1), (h / 2).max(1));
        let mut next = vec![0; (nw * nh * 4) as usize];
        for y in 0..nh {
            for x in 0..nw {
                for channel in 0..4 {
                    let mut sum = 0.0;
                    let mut n = 0;
                    for sy in y * h / nh..(y + 1) * h / nh {
                        for sx in x * w / nw..(x + 1) * w / nw {
                            let mut v = data[((sy * w + sx) * 4 + channel) as usize] as f32 / 255.0;
                            if srgb && channel < 3 {
                                v = if v <= 0.04045 {
                                    v / 12.92
                                } else {
                                    ((v + 0.055) / 1.055).powf(2.4)
                                };
                            }
                            sum += v;
                            n += 1;
                        }
                    }
                    let mut v = sum / n as f32;
                    if srgb && channel < 3 {
                        v = if v <= 0.0031308 {
                            v * 12.92
                        } else {
                            1.055 * v.powf(1.0 / 2.4) - 0.055
                        };
                    }
                    next[((y * nw + x) * 4 + channel) as usize] = (v * 255.0).round() as u8;
                }
            }
        }
        data = next;
        w = nw;
        h = nh;
    }
    Ok(tex.create_view(&Default::default()))
}

#[derive(Clone, Copy)]
enum UploadMode {
    Triangles,
    Screen,
    WideLine,
    Lines,
    Points,
    Globe,
    Imagery,
}

/// Point diameter and optional geometric-error attenuation, in framebuffer pixels.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PointStyle {
    pub size: f32,
    /// Geometric error already multiplied by its scale, in world units.
    pub geometric_error: Option<f32>,
    pub maximum_attenuation: f32,
    pub normal_shading: bool,
    pub back_face_culling: bool,
}
impl Default for PointStyle {
    fn default() -> Self {
        Self {
            size: 1.0,
            geometric_error: None,
            maximum_attenuation: 16.0,
            normal_shading: true,
            back_face_culling: false,
        }
    }
}
impl PointStyle {
    fn validate(self) -> Result<(), RenderError> {
        if !self.size.is_finite()
            || !(1.0..=4096.0).contains(&self.size)
            || !self.maximum_attenuation.is_finite()
            || !(1.0..=4096.0).contains(&self.maximum_attenuation)
            || self
                .geometric_error
                .is_some_and(|v| !v.is_finite() || v < 0.0)
        {
            return Err(RenderError("invalid point style".into()));
        }
        Ok(())
    }
}

pub struct GpuMesh {
    pub id: u64,
    pub model: DMat4,
    pub bytes: usize,
    pub pickable: bool,
    vertex: wgpu::Buffer,
    index: wgpu::Buffer,
    count: u32,
    material: wgpu::BindGroup,
    variant: u8,
    alpha_blend: bool,
    two_pass: bool,
    line: bool,
    point: bool,
    point_style: PointStyle,
    screen: bool,
    wide_line: bool,
    screen_distance: ScreenDistance,
    screen_offset: [f32; 2],
    screen_rotation: f32,
    center: DVec3,
    feature_count: u32,
    pub z_index: i32,
}

impl GpuMesh {
    /// Returns the active style for a point mesh.
    pub fn point_style(&self) -> Option<PointStyle> {
        self.point.then_some(self.point_style)
    }
}

struct PassOptions<'a> {
    pick: bool,
    scissor: Option<(u32, u32)>,
    overlays: &'a [bool],
}

pub struct Renderer {
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub width: u32,
    pub height: u32,
    pub selected: Option<FeatureRef>,
    pub device_context: DeviceContext,
    pub metrics_history: FrameStatsHistory,
    pub render_dirty: bool,
    format: wgpu::TextureFormat,
    global_buffer: wgpu::Buffer,
    global_group: wgpu::BindGroup,
    object_layout: wgpu::BindGroupLayout,
    paged_uniforms: PagedObjectUniforms,
    material_layout: wgpu::BindGroupLayout,
    white_texture: wgpu::TextureView,
    color_sampler: wgpu::Sampler,
    tile_sampler: wgpu::Sampler,
    pipeline_cache: PipelineCache,
    depth: wgpu::TextureView,
    pick_texture: wgpu::Texture,
    pick_view: wgpu::TextureView,
    pick_depth: wgpu::TextureView,
    pending: Option<PendingPick>,
    next_mesh: u64,
}

fn uniform_layout(device: &wgpu::Device, size: u64, dynamic: bool) -> wgpu::BindGroupLayout {
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("uniform layout"),
        entries: &[wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: dynamic,
                min_binding_size: NonZeroU64::new(size),
            },
            count: None,
        }],
    })
}

fn uniform_group(
    device: &wgpu::Device,
    layout: &wgpu::BindGroupLayout,
    buffer: &wgpu::Buffer,
    size: u64,
) -> wgpu::BindGroup {
    device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("uniform group"),
        layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                buffer,
                offset: 0,
                size: NonZeroU64::new(size),
            }),
        }],
    })
}

pub fn attachment(
    device: &wgpu::Device,
    width: u32,
    height: u32,
    format: wgpu::TextureFormat,
    usage: wgpu::TextureUsages,
) -> wgpu::Texture {
    device.create_texture(&wgpu::TextureDescriptor {
        label: Some("frame attachment"),
        size: wgpu::Extent3d {
            width: width.max(1),
            height: height.max(1),
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format,
        usage,
        view_formats: &[],
    })
}

fn depth(device: &wgpu::Device, w: u32, h: u32) -> wgpu::TextureView {
    attachment(
        device,
        w,
        h,
        wgpu::TextureFormat::Depth32Float,
        wgpu::TextureUsages::RENDER_ATTACHMENT,
    )
    .create_view(&Default::default())
}

impl Renderer {
    pub fn new(
        device: wgpu::Device,
        queue: wgpu::Queue,
        format: wgpu::TextureFormat,
        width: u32,
        height: u32,
    ) -> Result<Self, RenderError> {
        let device_context = DeviceContext::new(&device);

        if !format.is_srgb() {
            return Err(RenderError("baseline requires an sRGB color target".into()));
        }

        let gl = uniform_layout(&device, 96, false);
        let ol = uniform_layout(&device, 144, true);
        let ml = material_layout(&device);

        let white_texture = color_texture(
            &device,
            &queue,
            &BaseColorTexture {
                width: 1,
                height: 1,
                rgba: &[255, 255, 255, 255],
            },
        )?;

        let color_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("equirectangular color sampler"),
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        let tile_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("imagery tile sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        let global_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("global uniforms"),
            size: 96,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let global_group = uniform_group(&device, &gl, &global_buffer, 96);

        // Dynamically paged uniform manager
        let paged_uniforms = PagedObjectUniforms::new(&device, &ol, DEFAULT_PAGE_CAPACITY);

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("mesh and ID shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/mesh.wgsl").into()),
        });

        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("baseline groups"),
            bind_group_layouts: &[Some(&gl), Some(&ol), Some(&ml)],
            immediate_size: 0,
        });

        let pipeline_cache = PipelineCache::build(&device, &shader, &layout, format);

        let dep = depth(&device, width, height);
        let pd = depth(&device, width, height);
        let pt = attachment(
            &device,
            width,
            height,
            wgpu::TextureFormat::R32Uint,
            wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        );
        let pv = pt.create_view(&Default::default());

        Ok(Self {
            device,
            queue,
            width,
            height,
            selected: None,
            device_context,
            metrics_history: FrameStatsHistory::new(120, false),
            render_dirty: true,
            format,
            global_buffer,
            global_group,
            object_layout: ol,
            paged_uniforms,
            material_layout: ml,
            white_texture,
            color_sampler,
            tile_sampler,
            pipeline_cache,
            depth: dep,
            pick_texture: pt,
            pick_view: pv,
            pick_depth: pd,
            pending: None,
            next_mesh: 0,
        })
    }

    pub fn check_health(&mut self) -> Result<(), RenderError> {
        self.device_context
            .poll_health()
            .map_err(|e| RenderError(format!("GPU failure: {e}")))
    }

    pub fn device_status(&self) -> &DeviceStatus {
        &self.device_context.status
    }

    pub fn mark_device_lost(&mut self, reason: &str, message: &str) {
        self.device_context.mark_lost(reason, message);
        self.pending = None;
        self.selected = None;
        self.render_dirty = true;
    }

    pub fn format(&self) -> wgpu::TextureFormat {
        self.format
    }

    /// Discard pending picks after layer removal without recreating attachments.
    pub fn invalidate_pick(&mut self) {
        self.device_context.epoch = self.device_context.epoch.wrapping_add(1);
        self.selected = None;
        self.pending = None;
        self.render_dirty = true;
    }

    pub fn request_render(&mut self) {
        self.render_dirty = true;
    }

    pub fn is_render_dirty(&self) -> bool {
        self.render_dirty
    }

    pub fn mark_render_clean(&mut self) {
        self.render_dirty = false;
    }

    pub fn stats_summary(&self) -> Option<FrameStatsSummary> {
        self.metrics_history.summary()
    }

    pub fn resize(&mut self, w: u32, h: u32) -> Result<(), RenderError> {
        self.check_health()?;
        if w > self.device.limits().max_texture_dimension_2d
            || h > self.device.limits().max_texture_dimension_2d
        {
            return Err(RenderError("viewport exceeds adapter limit".into()));
        }
        self.width = w;
        self.height = h;
        self.device_context.epoch = self.device_context.epoch.wrapping_add(1);
        self.render_dirty = true;

        if w == 0 || h == 0 {
            return Ok(());
        }
        self.depth = depth(&self.device, w, h);
        self.pick_depth = depth(&self.device, w, h);
        self.pick_texture = attachment(
            &self.device,
            w,
            h,
            wgpu::TextureFormat::R32Uint,
            wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        );
        self.pick_view = self.pick_texture.create_view(&Default::default());
        Ok(())
    }

    pub fn upload(
        &mut self,
        p: &Primitive,
        model: DMat4,
        pickable: bool,
    ) -> Result<GpuMesh, RenderError> {
        self.upload_material(p, model, pickable, None, UploadMode::Triangles)
    }

    /// Polyline strips with previous/next ECEF positions and framebuffer-pixel widths.
    pub fn upload_wide_line(
        &mut self,
        p: &Primitive,
        model: DMat4,
        pickable: bool,
    ) -> Result<GpuMesh, RenderError> {
        self.upload_material(p, model, pickable, None, UploadMode::WideLine)
    }

    /// Camera-facing quads whose local XY coordinates are framebuffer pixels.
    pub fn upload_screen(
        &mut self,
        p: &Primitive,
        model: DMat4,
        pickable: bool,
    ) -> Result<GpuMesh, RenderError> {
        self.upload_material(p, model, pickable, None, UploadMode::Screen)
    }

    pub fn configure_screen_distance(
        mesh: &mut GpuMesh,
        options: ScreenDistance,
        pixel_offset: [f32; 2],
        rotation: f32,
    ) -> Result<(), RenderError> {
        if !mesh.screen {
            return Err(RenderError(
                "screen distance options require a screen mesh".into(),
            ));
        }
        options.validate()?;
        if !rotation.is_finite() || rotation.abs() > 10000. {
            return Err(RenderError("screen rotation limit".into()));
        }
        mesh.screen_rotation = rotation;
        if pixel_offset
            .iter()
            .any(|v| !v.is_finite() || v.abs() > 4096.)
        {
            return Err(RenderError("screen offset limit".into()));
        }
        mesh.screen_offset = pixel_offset;
        mesh.screen_distance = options;
        Ok(())
    }

    pub fn upload_lines(
        &mut self,
        p: &Primitive,
        model: DMat4,
        pickable: bool,
    ) -> Result<GpuMesh, RenderError> {
        if !p.indices.len().is_multiple_of(2) {
            return Err(RenderError("line-list indices must form pairs".into()));
        }
        self.upload_material(p, model, pickable, None, UploadMode::Lines)
    }

    /// Upload points once and expand each into a screen-facing instanced quad.
    pub fn upload_points(
        &mut self,
        p: &Primitive,
        model: DMat4,
        pickable: bool,
    ) -> Result<GpuMesh, RenderError> {
        if p.indices
            .iter()
            .enumerate()
            .all(|(i, &index)| index as usize == i)
            && p.indices.len() == p.vertices.len()
        {
            return self.upload_material(p, model, pickable, None, UploadMode::Points);
        }
        let vertices = p
            .indices
            .iter()
            .map(|&i| {
                p.vertices
                    .get(i as usize)
                    .copied()
                    .ok_or_else(|| RenderError("point index out of range".into()))
            })
            .collect::<Result<Vec<_>, _>>()?;
        let mut expanded = p.clone();
        expanded.vertices = vertices;
        self.upload_material(&expanded, model, pickable, None, UploadMode::Points)
    }

    /// Updates a point mesh without re-uploading its vertex buffer.
    pub fn configure_points(mesh: &mut GpuMesh, style: PointStyle) -> Result<(), RenderError> {
        if !mesh.point {
            return Err(RenderError("point style requires a point mesh".into()));
        }
        style.validate()?;
        mesh.point_style = style;
        Ok(())
    }

    pub fn upload_globe(
        &mut self,
        p: &Primitive,
        texture: &BaseColorTexture<'_>,
    ) -> Result<GpuMesh, RenderError> {
        self.upload_material(p, DMat4::IDENTITY, false, Some(texture), UploadMode::Globe)
    }

    /// Terrain retains a double-precision RTC transform while sharing the globe material path.
    pub fn upload_terrain(
        &mut self,
        p: &Primitive,
        texture: &BaseColorTexture<'_>,
    ) -> Result<GpuMesh, RenderError> {
        self.upload_material(p, p.transform, false, Some(texture), UploadMode::Globe)
    }

    pub fn upload_imagery(
        &mut self,
        p: &Primitive,
        texture: &BaseColorTexture<'_>,
    ) -> Result<GpuMesh, RenderError> {
        self.upload_material(
            p,
            DMat4::IDENTITY,
            false,
            Some(texture),
            UploadMode::Imagery,
        )
    }

    /// Geometry for a coverage region, sharing the ancestor's texture and material.
    pub fn upload_shared_globe(
        &mut self,
        p: &Primitive,
        source: &GpuMesh,
    ) -> Result<GpuMesh, RenderError> {
        let mut mesh = self.upload_material(p, DMat4::IDENTITY, false, None, UploadMode::Globe)?;
        mesh.material = source.material.clone();
        mesh.variant = source.variant;
        mesh.alpha_blend = source.alpha_blend;
        mesh.two_pass = source.two_pass;
        Ok(mesh)
    }

    fn upload_material(
        &mut self,
        p: &Primitive,
        model: DMat4,
        pickable: bool,
        texture: Option<&BaseColorTexture<'_>>,
        mode: UploadMode,
    ) -> Result<GpuMesh, RenderError> {
        let earth = matches!(mode, UploadMode::Globe | UploadMode::Imagery);
        let tile = matches!(mode, UploadMode::Imagery);
        let line = matches!(mode, UploadMode::Lines);
        let point = matches!(mode, UploadMode::Points);
        self.check_health()?;
        if p.indices.is_empty() || p.vertices.is_empty() {
            return Err(RenderError("empty primitive".into()));
        }
        if !model.is_finite() || model.determinant().abs() < 1e-20 {
            return Err(RenderError("singular model matrix".into()));
        }
        let vb = bytemuck::cast_slice(&p.vertices);
        let ib = bytemuck::cast_slice(&p.indices);
        if vb.len() as u64 > self.device.limits().max_buffer_size
            || ib.len() as u64 > self.device.limits().max_buffer_size
        {
            return Err(RenderError("mesh exceeds buffer limit".into()));
        }
        let feature_count = p
            .vertices
            .iter()
            .map(|v| v.feature_id)
            .max()
            .unwrap_or(0)
            .checked_add(1)
            .ok_or_else(|| RenderError("feature ID overflow".into()))?;
        let vertex = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("static vertices"),
                contents: vb,
                usage: wgpu::BufferUsages::VERTEX,
            });
        let index = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("static indices"),
                contents: ib,
                usage: wgpu::BufferUsages::INDEX,
            });
        let m = MaterialUniform {
            base_color: p.material.base_color,
            factors: [
                p.material.metallic,
                p.material.roughness,
                if p.material.alpha_blend {
                    -2.0
                } else {
                    p.material.alpha_mask.unwrap_or(-1.0)
                },
                if earth { 1.0 } else { 0.0 },
            ],
            emissive_normal: [
                p.material.emissive[0],
                p.material.emissive[1],
                p.material.emissive[2],
                p.material.normal_scale,
            ],
            params: [
                p.material.occlusion_strength,
                p.material.pattern.as_ref().map_or(0., |p| p.thickness[0]),
                p.material.pattern.as_ref().map_or(0., |p| p.thickness[1]),
                p.material.pattern.as_ref().map_or(1., |p| p.cell_alpha),
            ],
            uv_sets: std::array::from_fn(|i| {
                p.material.textures[i].as_ref().map_or(0, |t| t.tex_coord)
            }),
            extras: [
                p.material.textures[4].as_ref().map_or(0, |t| t.tex_coord),
                u32::from(p.material.textures[2].is_some())
                    | (u32::from(p.material.face_forward) << 1)
                    | (u32::from(p.material.lambert) << 2)
                    | (u32::from(point && p.material.lambert) << 3),
                u32::from(p.material.unlit),
                p.material.pattern.as_ref().map_or(0, |p| p.kind),
            ],
            secondary: p.material.pattern.as_ref().map_or([1.; 4], |p| p.secondary),
            pattern: p.material.pattern.as_ref().map_or([0.; 4], |p| {
                [p.repeat[0], p.repeat[1], p.offset, p.axis as f32]
            }),
        };
        let mb = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("material factors"),
                contents: bytemuck::bytes_of(&m),
                usage: wgpu::BufferUsages::UNIFORM,
            });
        let mut views = Vec::new();
        let mut samplers = Vec::new();
        let mut texture_bytes = 0;
        for slot in 0..5 {
            if let Some(data) = texture.filter(|_| slot == 0) {
                views.push(Some(color_texture(&self.device, &self.queue, data)?));
                texture_bytes += data.rgba.len();
                samplers.push(if tile {
                    self.tile_sampler.clone()
                } else {
                    self.color_sampler.clone()
                });
            } else if let Some(t) = &p.material.textures[slot] {
                views.push(Some(pbr_texture(
                    &self.device,
                    &self.queue,
                    &t.image,
                    slot == 0 || slot == 4,
                )?));
                texture_bytes += t.gpu_bytes();
                samplers.push(texture_sampler(&self.device, t.sampler));
            } else {
                views.push(None);
                samplers.push(self.color_sampler.clone());
            }
        }
        let mut entries = vec![wgpu::BindGroupEntry {
            binding: 0,
            resource: mb.as_entire_binding(),
        }];
        for slot in 0..5 {
            entries.push(wgpu::BindGroupEntry {
                binding: 1 + slot as u32 * 2,
                resource: wgpu::BindingResource::TextureView(
                    views[slot].as_ref().unwrap_or(&self.white_texture),
                ),
            });
            entries.push(wgpu::BindGroupEntry {
                binding: 2 + slot as u32 * 2,
                resource: wgpu::BindingResource::Sampler(&samplers[slot]),
            });
        }
        let material = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("PBR material"),
            layout: &self.material_layout,
            entries: &entries,
        });
        let id = self.next_mesh;
        self.next_mesh = self
            .next_mesh
            .checked_add(1)
            .ok_or_else(|| RenderError("mesh ID overflow".into()))?;
        let variant = if point || p.material.double_sided {
            0
        } else if model.determinant() < 0.0 {
            2
        } else {
            1
        };
        self.render_dirty = true;
        Ok(GpuMesh {
            id,
            model,
            bytes: vb.len() + ib.len() + 128 + texture_bytes,
            pickable,
            vertex,
            index,
            count: p.indices.len() as u32,
            material,
            variant,
            alpha_blend: p.material.alpha_blend,
            two_pass: p.material.two_pass,
            line,
            point,
            point_style: PointStyle::default(),
            screen: matches!(mode, UploadMode::Screen),
            wide_line: matches!(mode, UploadMode::WideLine),
            screen_distance: ScreenDistance::default(),
            screen_offset: [0.; 2],
            screen_rotation: 0.,
            center: if matches!(mode, UploadMode::Screen) {
                DVec3::ZERO
            } else {
                let mut min = DVec3::splat(f64::INFINITY);
                let mut max = DVec3::splat(f64::NEG_INFINITY);
                for v in &p.vertices {
                    let p = DVec3::from_array(v.position.map(f64::from));
                    min = min.min(p);
                    max = max.max(p);
                }
                (min + max) * 0.5
            },
            feature_count,
            z_index: 0,
        })
    }

    /// Encodes and submits without blocking on GPU work or map completion.
    /// Uses paged dynamic uniform buffers and single-call batch uploads per page.
    pub fn render(
        &mut self,
        target: &wgpu::TextureView,
        camera: Camera,
        draws: &[&GpuMesh],
        pick: Option<(u32, u32)>,
    ) -> Result<(), RenderError> {
        let total_timer = StageTimer::start();
        self.check_health()?;
        if self.width == 0 || self.height == 0 {
            return Ok(());
        }
        if camera.width != self.width || camera.height != self.height {
            return Err(RenderError("camera viewport mismatch".into()));
        }

        // Stage 1: Prepare & Sort
        let prepare_timer = StageTimer::start();
        let overlay = |m: &GpuMesh| {
            m.screen
                && m.screen_distance
                    .overlay(m.model.transform_point3(DVec3::ZERO).distance(camera.eye))
        };
        let mut ordered = draws.to_vec();
        ordered.sort_by(|a, b| {
            overlay(a)
                .cmp(&overlay(b))
                .then_with(|| a.alpha_blend.cmp(&b.alpha_blend))
                .then_with(|| a.z_index.cmp(&b.z_index))
                .then_with(|| {
                    if a.alpha_blend {
                        let depth = |m: &GpuMesh| {
                            -(camera.camera_to_ecef.transpose()
                                * (m.model.transform_point3(m.center) - camera.eye))
                                .z
                        };
                        depth(b).total_cmp(&depth(a))
                    } else {
                        std::cmp::Ordering::Equal
                    }
                })
        });
        let draws = ordered.as_slice();
        let overlays = draws.iter().map(|m| overlay(m)).collect::<Vec<_>>();
        let prepare_duration = prepare_timer.elapsed();

        // Stage 2: Uniform batch pack & upload
        let uniform_timer = StageTimer::start();
        // Dynamically ensure capacity without hard 4096 cap
        self.paged_uniforms
            .ensure_capacity(&self.device, &self.object_layout, draws.len());
        self.paged_uniforms.begin_frame();

        let sun = (camera.camera_to_ecef.transpose() * DVec3::new(1.0, 0.5, 0.8).normalize())
            .as_vec3()
            .extend(0.0)
            .to_array();
        let global = GlobalUniform {
            projection: camera
                .projection()
                .map_err(|e| RenderError(e.to_string()))?
                .to_cols_array_2d(),
            sun_direction_eye: sun,
            viewport_exposure: [self.width as f32, self.height as f32, 1.0, 0.0],
        };
        self.queue
            .write_buffer(&self.global_buffer, 0, bytemuck::bytes_of(&global));

        let mut next_id = 1u32;
        let mut ranges = Vec::new();
        for (i, mesh) in draws.iter().enumerate() {
            let mv = relative_model_view(mesh.model, camera.eye, camera.camera_to_ecef);
            let normal = linear_part(mv.as_dmat4()).inverse().transpose();
            let base = if mesh.pickable {
                let b = next_id;
                next_id = next_id
                    .checked_add(mesh.feature_count)
                    .ok_or_else(|| RenderError("frame pick ID overflow".into()))?;
                ranges.push(PickRange {
                    base: b,
                    count: mesh.feature_count,
                    mesh: mesh.id,
                });
                b
            } else {
                0
            };
            let selected = self
                .selected
                .filter(|s| s.mesh == mesh.id)
                .map_or(u32::MAX, |s| s.feature);
            let screen_factors = if mesh.screen {
                mesh.screen_distance.evaluate(
                    mesh.model
                        .transform_point3(DVec3::ZERO)
                        .distance(camera.eye),
                )
            } else {
                [1.; 3]
            };
            let object = ObjectUniform {
                screen_params: if mesh.point {
                    [
                        mesh.point_style.size,
                        mesh.point_style.geometric_error.unwrap_or(-1.0),
                        mesh.point_style.maximum_attenuation,
                        (u32::from(mesh.point_style.normal_shading)
                            | (u32::from(mesh.point_style.back_face_culling) << 1))
                            as f32,
                    ]
                } else {
                    [
                        mesh.screen_offset[0],
                        mesh.screen_offset[1],
                        mesh.screen_rotation.cos(),
                        mesh.screen_rotation.sin(),
                    ]
                },
                model_view_relative: mv.to_cols_array_2d(),
                normal_col0: normal.x_axis.as_vec3().extend(screen_factors[0]).to_array(),
                normal_col1: normal
                    .y_axis
                    .as_vec3()
                    .extend(screen_factors[1] * screen_factors[2])
                    .to_array(),
                normal_col2: normal
                    .z_axis
                    .as_vec3()
                    .extend(if mesh.screen {
                        mesh.screen_distance.offset_scale(
                            mesh.model
                                .transform_point3(DVec3::ZERO)
                                .distance(camera.eye),
                        )
                    } else {
                        1.
                    })
                    .to_array(),
                feature_params: [
                    base,
                    selected,
                    mesh.pickable as u32,
                    if mesh.screen {
                        2
                    } else if mesh.wide_line {
                        3
                    } else {
                        u32::from(mesh.line)
                    },
                ],
            };
            self.paged_uniforms.write_slot(i, &object);
        }
        // Single batch upload per active page instead of per-draw write_buffer calls
        self.paged_uniforms.upload_all(&self.queue);
        let uniform_upload_duration = uniform_timer.elapsed();

        // Stage 3: Pass encoding
        let pass_timer = StageTimer::start();
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("frame"),
            });
        self.pass(
            &mut encoder,
            target,
            &self.depth,
            draws,
            PassOptions {
                pick: false,
                scissor: None,
                overlays: &overlays,
            },
        );
        let position = pick
            .filter(|(x, y)| *x < self.width && *y < self.height)
            .filter(|_| self.pending.is_none());
        let readback = position.map(|(x, y)| {
            self.pass(
                &mut encoder,
                &self.pick_view,
                &self.pick_depth,
                draws,
                PassOptions {
                    pick: true,
                    scissor: Some((x, y)),
                    overlays: &overlays,
                },
            );
            let buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("1x1 ID readback"),
                size: 256,
                usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
            encoder.copy_texture_to_buffer(
                wgpu::TexelCopyTextureInfo {
                    texture: &self.pick_texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d { x, y, z: 0 },
                    aspect: wgpu::TextureAspect::All,
                },
                wgpu::TexelCopyBufferInfo {
                    buffer: &buffer,
                    layout: wgpu::TexelCopyBufferLayout {
                        offset: 0,
                        bytes_per_row: Some(256),
                        rows_per_image: Some(1),
                    },
                },
                wgpu::Extent3d {
                    width: 1,
                    height: 1,
                    depth_or_array_layers: 1,
                },
            );
            buffer
        });
        let pass_encode_duration = pass_timer.elapsed();

        // Stage 4: Submit
        let submit_timer = StageTimer::start();
        self.queue.submit([encoder.finish()]);
        let submit_duration = submit_timer.elapsed();

        if let Some(buffer) = readback {
            let (tx, rx) = mpsc::channel();
            buffer.slice(..).map_async(wgpu::MapMode::Read, move |r| {
                let _ = tx.send(r);
            });
            self.pending = Some(PendingPick {
                buffer,
                receiver: rx,
                ranges,
                epoch: self.device_context.epoch,
            });
        }

        // Metrics recording
        let mut triangles_count = 0u64;
        let mut lines_count = 0u64;
        let mut draw_calls = 0u32;
        let mut mesh_bytes = 0usize;
        for m in draws {
            draw_calls += if m.two_pass && m.alpha_blend && !m.line && !m.point {
                2
            } else {
                1
            };
            if m.line {
                lines_count += (m.count / 2) as u64;
            } else if !m.point {
                triangles_count += (m.count / 3) as u64;
            }
            mesh_bytes += m.bytes;
        }

        let stats = FrameStats {
            prepare_duration,
            uniform_upload_duration,
            pass_encode_duration,
            submit_duration,
            total_cpu_duration: total_timer.elapsed(),
            gpu_time_ms: None,
            draw_calls,
            triangles_drawn: triangles_count,
            lines_drawn: lines_count,
            object_count: draws.len() as u32,
            uniform_pages: self.paged_uniforms.pages.len(),
            resident_mesh_bytes: mesh_bytes,
            resident_texture_bytes: 0,
            uniform_buffer_bytes: self.paged_uniforms.total_gpu_bytes(),
        };
        self.metrics_history.push(stats);
        self.render_dirty = false;

        Ok(())
    }

    fn pass(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        target: &wgpu::TextureView,
        depth: &wgpu::TextureView,
        draws: &[&GpuMesh],
        options: PassOptions<'_>,
    ) {
        let PassOptions {
            pick,
            scissor,
            overlays,
        } = options;
        let clear = if pick {
            wgpu::Color::TRANSPARENT
        } else {
            wgpu::Color {
                r: 0.005,
                g: 0.008,
                b: 0.015,
                a: 1.0,
            }
        };
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some(if pick { "ID pass" } else { "color pass" }),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: target,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(clear),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: depth,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(0.0),
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: None,
            }),
            ..Default::default()
        });
        if let Some((x, y)) = scissor {
            pass.set_scissor_rect(x, y, 1, 1);
        }
        pass.set_bind_group(0, &self.global_group, &[]);
        for (i, m) in draws.iter().enumerate() {
            let key = PipelineKey {
                pick,
                blend: m.alpha_blend,
                line: m.line,
                point: m.point,
                variant: m.variant,
                overlay: overlays[i],
            };
            if let Some(pipeline) = self.pipeline_cache.get(&key) {
                pass.set_pipeline(pipeline);
            }
            let (page_idx, offset) = self.paged_uniforms.slot_location(i);
            let page = &self.paged_uniforms.pages[page_idx];
            pass.set_bind_group(1, &page.bind_group, &[offset]);
            pass.set_bind_group(2, &m.material, &[]);
            pass.set_vertex_buffer(0, m.vertex.slice(..));
            pass.set_index_buffer(m.index.slice(..), wgpu::IndexFormat::Uint32);
            if m.two_pass && m.alpha_blend && !m.line && !m.point {
                let two_pass_key = PipelineKey {
                    pick,
                    blend: true,
                    line: false,
                    point: false,
                    variant: if m.variant == 2 { 4 } else { 3 },
                    overlay: overlays[i],
                };
                if let Some(p2) = self.pipeline_cache.get(&two_pass_key) {
                    pass.set_pipeline(p2);
                }
                pass.draw_indexed(0..m.count, 0, 0..1);
                if let Some(pipeline) = self.pipeline_cache.get(&key) {
                    pass.set_pipeline(pipeline);
                }
            }
            if m.point {
                pass.draw(0..6, 0..m.count);
            } else {
                pass.draw_indexed(0..m.count, 0, 0..1);
            }
        }
    }

    /// None=pending; Some(None)=miss or resize invalidation; Some(Some)=stable ID.
    pub fn poll_pick(&mut self) -> Option<Option<FeatureRef>> {
        #[cfg(not(target_arch = "wasm32"))]
        {
            let _ = self.device.poll(wgpu::PollType::Poll);
        }
        let p = self.pending.take()?;
        let (res, remaining) = p.poll(self.device_context.epoch);
        self.pending = remaining;
        res
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cesium_formats::Vertex;

    #[test]
    fn shader_and_layout() {
        let shader = naga::front::wgsl::parse_str(include_str!("shaders/mesh.wgsl")).unwrap();
        naga::valid::Validator::new(
            naga::valid::ValidationFlags::all(),
            naga::valid::Capabilities::empty(),
        )
        .validate(&shader)
        .unwrap();
        assert_eq!(std::mem::offset_of!(ObjectUniform, feature_params), 112);
        assert_eq!(std::mem::offset_of!(Vertex, feature_id), 32);
    }

    fn wait(r: &Renderer) {
        r.device
            .poll(wgpu::PollType::Wait {
                submission_index: None,
                timeout: Some(std::time::Duration::from_secs(30)),
            })
            .unwrap();
    }

    fn pixel(r: &Renderer, texture: &wgpu::Texture) -> [u8; 4] {
        pixel_at(r, texture, 32, 32)
    }

    fn pixel_at(r: &Renderer, texture: &wgpu::Texture, x: u32, y: u32) -> [u8; 4] {
        let buffer = r.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("test pixel"),
            size: 256,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        let mut encoder = r.device.create_command_encoder(&Default::default());
        encoder.copy_texture_to_buffer(
            wgpu::TexelCopyTextureInfo {
                texture,
                mip_level: 0,
                origin: wgpu::Origin3d { x, y, z: 0 },
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::TexelCopyBufferInfo {
                buffer: &buffer,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(256),
                    rows_per_image: Some(1),
                },
            },
            wgpu::Extent3d {
                width: 1,
                height: 1,
                depth_or_array_layers: 1,
            },
        );
        r.queue.submit([encoder.finish()]);
        let (tx, rx) = mpsc::channel();
        buffer
            .slice(..)
            .map_async(wgpu::MapMode::Read, move |result| {
                tx.send(result).unwrap();
            });
        wait(r);
        rx.recv().unwrap().unwrap();
        let view = buffer.slice(..).get_mapped_range().unwrap();
        let out = [view[0], view[1], view[2], view[3]];
        drop(view);
        buffer.unmap();
        out
    }

    #[test]
    #[ignore = "requires a native GPU or Mesa Vulkan software adapter"]
    fn paged_uniforms_dynamic_allocation_beyond_4096() {
        pollster::block_on(async {
            let inst = instance();
            let (_, device, queue) = device(&inst, None).await.unwrap();
            let mut r =
                Renderer::new(device, queue, wgpu::TextureFormat::Rgba8UnormSrgb, 64, 64).unwrap();
            let mut p = cesium_formats::decode_content(
                include_bytes!("../../../tests/fixtures/triangle.glb"),
                &Default::default(),
            )
            .unwrap()
            .primitives
            .remove(0);
            p.material.unlit = true;
            let mesh = r.upload(&p, DMat4::IDENTITY, false).unwrap();

            // Create 5000 draw references (exceeding original 4096 slots limit)
            let mut draws = Vec::with_capacity(5000);
            for _ in 0..5000 {
                draws.push(&mesh);
            }

            let camera = Camera::look_at(
                DVec3::new(0., 20., 100.),
                DVec3::new(0., 20., 0.),
                DVec3::Y,
                64,
                64,
            )
            .unwrap();
            let target = attachment(
                &r.device,
                64,
                64,
                r.format(),
                wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            );

            // This must succeed without throwing "4096 object slots exceeded"
            r.render(
                &target.create_view(&Default::default()),
                camera,
                &draws,
                None,
            )
            .unwrap();

            assert!(r.paged_uniforms.total_capacity() >= 5000);
            assert!(r.paged_uniforms.pages.len() >= 3);

            let summary = r.stats_summary().unwrap();
            assert_eq!(summary.last_frame.object_count, 5000);
            assert!(summary.last_frame.uniform_pages >= 3);
        });
    }

    #[test]
    #[ignore = "requires a native GPU or Mesa Vulkan software adapter"]
    fn device_lost_and_state_recovery_cycle() {
        pollster::block_on(async {
            let inst = instance();
            let (_, device, queue) = device(&inst, None).await.unwrap();
            let mut r =
                Renderer::new(device, queue, wgpu::TextureFormat::Rgba8UnormSrgb, 64, 64).unwrap();
            assert!(r.check_health().is_ok());
            assert_eq!(*r.device_status(), DeviceStatus::Running);

            // Simulate explicit device lost
            r.mark_device_lost("SyntheticLost", "Test injected loss");
            assert!(!r.check_health().is_ok());
            assert!(matches!(r.device_status(), DeviceStatus::Lost { .. }));

            // Transition: start recreating
            r.device_context.start_recreating();
            assert!(matches!(r.device_status(), DeviceStatus::Recreating));

            // Transition: advance to rehydrating
            r.device_context.advance_to_rehydrating();
            assert!(matches!(r.device_status(), DeviceStatus::Rehydrating));

            // Complete recovery
            r.device_context.complete_recovery();
            assert_eq!(*r.device_status(), DeviceStatus::Running);
            assert!(r.check_health().is_ok());
        });
    }

    #[test]
    #[ignore = "requires a native GPU or Mesa Vulkan software adapter"]
    fn stats_metrics_and_batch_upload_verification() {
        pollster::block_on(async {
            let inst = instance();
            let (_, device, queue) = device(&inst, None).await.unwrap();
            let mut r =
                Renderer::new(device, queue, wgpu::TextureFormat::Rgba8UnormSrgb, 64, 64).unwrap();
            let mut p = cesium_formats::decode_content(
                include_bytes!("../../../tests/fixtures/triangle.glb"),
                &Default::default(),
            )
            .unwrap()
            .primitives
            .remove(0);
            p.material.unlit = true;
            let mesh = r.upload(&p, DMat4::IDENTITY, false).unwrap();

            let camera = Camera::look_at(
                DVec3::new(0., 20., 100.),
                DVec3::new(0., 20., 0.),
                DVec3::Y,
                64,
                64,
            )
            .unwrap();
            let target = attachment(
                &r.device,
                64,
                64,
                r.format(),
                wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            );

            let draws = vec![&mesh; 100];
            r.render(
                &target.create_view(&Default::default()),
                camera,
                &draws,
                None,
            )
            .unwrap();

            let summary = r
                .stats_summary()
                .expect("summary should be available after render");
            assert_eq!(summary.sample_count, 1);
            assert_eq!(summary.last_frame.draw_calls, 100);
            assert_eq!(summary.last_frame.object_count, 100);
            assert_eq!(summary.last_frame.triangles_drawn, 100); // 1 triangle per mesh * 100
            assert!(summary.last_frame.total_cpu_duration.as_nanos() > 0);
            assert!(summary.last_frame.uniform_upload_duration.as_nanos() > 0);
            assert!(summary.last_frame.pass_encode_duration.as_nanos() > 0);
        });
    }

    #[test]
    #[ignore = "requires a native GPU or Mesa Vulkan software adapter"]
    fn point_list_render_and_pick() {
        pollster::block_on(async {
            let inst = instance();
            let (_, device, queue) = device(&inst, None).await.unwrap();
            let mut r =
                Renderer::new(device, queue, wgpu::TextureFormat::Rgba8UnormSrgb, 64, 64).unwrap();
            let mut p = cesium_formats::decode_content(
                include_bytes!("../../../tests/fixtures/triangle.glb"),
                &Default::default(),
            )
            .unwrap()
            .primitives
            .remove(0);
            p.vertices.truncate(1);
            p.vertices[0].position = [0., 20., 0.];
            p.vertices[0].feature_id = 7;
            p.vertices[0].color = [0., 1., 0., 1.];
            p.indices = vec![0];
            p.material = cesium_formats::Material {
                unlit: true,
                ..Default::default()
            };
            let mut point = r.upload_points(&p, DMat4::IDENTITY, true).unwrap();
            let camera = Camera::look_at(
                DVec3::new(0., 20., 100.),
                DVec3::new(0., 20., 0.),
                DVec3::Y,
                64,
                64,
            )
            .unwrap();
            let target = attachment(
                &r.device,
                64,
                64,
                r.format(),
                wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            );
            let mut picked = None;
            for x in [31, 32] {
                for y in [31, 32] {
                    r.render(
                        &target.create_view(&Default::default()),
                        camera,
                        &[&point],
                        Some((x, y)),
                    )
                    .unwrap();
                    wait(&r);
                    if let Some(f) = r.poll_pick().unwrap() {
                        assert_eq!((f.mesh, f.feature), (point.id, 7));
                        picked = Some((x, y));
                    }
                }
            }
            let (x, y) = picked.expect("point was not rasterized or pickable");
            assert_eq!(pixel_at(&r, &target, x, y), [0, 255, 0, 255]);
            Renderer::configure_points(
                &mut point,
                PointStyle {
                    size: 9.0,
                    ..Default::default()
                },
            )
            .unwrap();
            r.render(
                &target.create_view(&Default::default()),
                camera,
                &[&point],
                Some((x + 3, y)),
            )
            .unwrap();
            wait(&r);
            assert_eq!(r.poll_pick().unwrap().unwrap().feature, 7);
            assert_eq!(pixel_at(&r, &target, x + 3, y), [0, 255, 0, 255]);
            assert_ne!(pixel_at(&r, &target, x + 6, y), [0, 255, 0, 255]);
            Renderer::configure_points(
                &mut point,
                PointStyle {
                    geometric_error: Some(16.0),
                    maximum_attenuation: 16.0,
                    ..Default::default()
                },
            )
            .unwrap();
            r.render(
                &target.create_view(&Default::default()),
                camera,
                &[&point],
                Some((x + 3, y)),
            )
            .unwrap();
            wait(&r);
            assert!(r.poll_pick().unwrap().is_some(), "near attenuated point");
            let far_camera = Camera::look_at(
                DVec3::new(0., 20., 200.),
                DVec3::new(0., 20., 0.),
                DVec3::Y,
                64,
                64,
            )
            .unwrap();
            r.render(
                &target.create_view(&Default::default()),
                far_camera,
                &[&point],
                Some((x + 3, y)),
            )
            .unwrap();
            wait(&r);
            assert!(r.poll_pick().unwrap().is_none(), "far point should shrink");
            assert!(Renderer::configure_points(
                &mut point,
                PointStyle {
                    size: f32::NAN,
                    ..Default::default()
                }
            )
            .is_err());
            p.indices = vec![3];
            assert!(r.upload_points(&p, DMat4::IDENTITY, true).is_err());
            p.vertices.push(p.vertices[0]);
            p.vertices[1].feature_id = 9;
            p.vertices[0].position = [1000., 20., 0.];
            p.indices = vec![1];
            let reordered = r.upload_points(&p, DMat4::IDENTITY, true).unwrap();
            r.render(
                &target.create_view(&Default::default()),
                camera,
                &[&reordered],
                Some((x, y)),
            )
            .unwrap();
            wait(&r);
            assert_eq!(r.poll_pick().unwrap().unwrap().feature, 9);
            p.vertices.truncate(1);
            p.vertices[0].position = [0., 20., 0.];
            p.indices = vec![0];
            p.material.lambert = true;
            p.vertices[0].normal = (-DVec3::new(1.0, 0.5, 0.8).normalize())
                .as_vec3()
                .to_array();
            let mut shaded = r.upload_points(&p, DMat4::IDENTITY, true).unwrap();
            r.render(
                &target.create_view(&Default::default()),
                camera,
                &[&shaded],
                None,
            )
            .unwrap();
            wait(&r);
            let color = pixel_at(&r, &target, x, y);
            assert!(
                (168..=171).contains(&color[1]),
                "ambient Lambert color: {color:?}"
            );
            Renderer::configure_points(
                &mut shaded,
                PointStyle {
                    normal_shading: false,
                    ..Default::default()
                },
            )
            .unwrap();
            r.render(
                &target.create_view(&Default::default()),
                camera,
                &[&shaded],
                Some((x, y)),
            )
            .unwrap();
            wait(&r);
            assert!(r.poll_pick().unwrap().is_some());
            assert_eq!(pixel_at(&r, &target, x, y), [0, 255, 0, 255]);
            Renderer::configure_points(
                &mut shaded,
                PointStyle {
                    normal_shading: false,
                    back_face_culling: true,
                    ..Default::default()
                },
            )
            .unwrap();
            r.render(
                &target.create_view(&Default::default()),
                camera,
                &[&shaded],
                Some((x, y)),
            )
            .unwrap();
            wait(&r);
            assert!(
                r.poll_pick().unwrap().is_none(),
                "back-facing point must not be picked"
            );
            assert_ne!(pixel_at(&r, &target, x, y), [0, 255, 0, 255]);
            let opposite = Camera::look_at(
                DVec3::new(0., 20., -100.),
                DVec3::new(0., 20., 0.),
                DVec3::Y,
                64,
                64,
            )
            .unwrap();
            r.render(
                &target.create_view(&Default::default()),
                opposite,
                &[&shaded],
                Some((x, y)),
            )
            .unwrap();
            wait(&r);
            assert_eq!(r.poll_pick().unwrap().unwrap().feature, 7);
            assert_eq!(pixel_at(&r, &target, x, y), [0, 255, 0, 255]);
            p.material.lambert = false;
            let mut without_normals = r.upload_points(&p, DMat4::IDENTITY, true).unwrap();
            Renderer::configure_points(
                &mut without_normals,
                PointStyle {
                    back_face_culling: true,
                    ..Default::default()
                },
            )
            .unwrap();
            r.render(
                &target.create_view(&Default::default()),
                camera,
                &[&without_normals],
                Some((x, y)),
            )
            .unwrap();
            wait(&r);
            assert_eq!(
                r.poll_pick().unwrap().unwrap().feature,
                7,
                "no normals means no back-face culling"
            );
            let projection = camera.projection().unwrap();
            p.vertices[0].position = [(100.0 / projection.x_axis.x) * 1.03, 20.0, 0.0];
            let mut outside = r.upload_points(&p, DMat4::IDENTITY, true).unwrap();
            Renderer::configure_points(
                &mut outside,
                PointStyle {
                    size: 9.0,
                    ..Default::default()
                },
            )
            .unwrap();
            r.render(
                &target.create_view(&Default::default()),
                camera,
                &[&outside],
                Some((63, y)),
            )
            .unwrap();
            wait(&r);
            assert!(
                r.poll_pick().unwrap().is_none(),
                "point center outside frustum is clipped"
            );
            p.vertices[0].position[0] = (100.0 / projection.x_axis.x) * 0.98;
            let mut inside = r.upload_points(&p, DMat4::IDENTITY, true).unwrap();
            Renderer::configure_points(
                &mut inside,
                PointStyle {
                    size: 9.0,
                    ..Default::default()
                },
            )
            .unwrap();
            r.render(
                &target.create_view(&Default::default()),
                camera,
                &[&inside],
                Some((63, y)),
            )
            .unwrap();
            wait(&r);
            assert_eq!(
                r.poll_pick().unwrap().unwrap().feature,
                7,
                "inside point extends to viewport edge"
            );
            assert_eq!(r.stats_summary().unwrap().last_frame.triangles_drawn, 0);
        });
    }

    #[test]
    #[ignore = "requires a native GPU or Mesa Vulkan software adapter"]
    fn one_pixel_lines_pick_and_occlusion() {
        pollster::block_on(async {
            let inst = instance();
            let (_, device, queue) = device(&inst, None).await.unwrap();
            let mut r =
                Renderer::new(device, queue, wgpu::TextureFormat::Rgba8UnormSrgb, 64, 64).unwrap();
            let mut p = cesium_formats::decode_content(
                include_bytes!("../../../tests/fixtures/triangle.glb"),
                &Default::default(),
            )
            .unwrap()
            .primitives
            .remove(0);
            p.material.unlit = true;
            p.material.base_color = [0., 1., 0., 1.];
            p.vertices.truncate(2);
            p.vertices[0].position = [-40., 20., 0.];
            p.vertices[1].position = [40., 20., 0.];
            p.indices = vec![0, 1];
            let line = r.upload_lines(&p, DMat4::IDENTITY, true).unwrap();
            assert!(line.line);
            let camera = Camera::look_at(
                DVec3::new(0., 20., 100.),
                DVec3::new(0., 20., 0.),
                DVec3::Y,
                64,
                64,
            )
            .unwrap();
            let target = attachment(
                &r.device,
                64,
                64,
                r.format(),
                wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            );
            let draw = |r: &mut Renderer, meshes: &[&GpuMesh], y| {
                r.render(
                    &target.create_view(&Default::default()),
                    camera,
                    meshes,
                    Some((32, y)),
                )
                .unwrap();
                wait(r);
                r.poll_pick().unwrap()
            };
            assert!([31, 32]
                .into_iter()
                .any(|y| draw(&mut r, &[&line], y).is_some_and(|f| f.mesh == line.id)));
            let mut p = cesium_formats::decode_content(
                include_bytes!("../../../tests/fixtures/triangle.glb"),
                &Default::default(),
            )
            .unwrap()
            .primitives
            .remove(0);
            p.material.unlit = true;
            p.material.base_color = [1., 0., 0., 1.];
            let front = r
                .upload(&p, DMat4::from_translation(DVec3::new(0., 0., 20.)), true)
                .unwrap();
            for meshes in [[&line, &front], [&front, &line]] {
                assert_eq!(draw(&mut r, &meshes, 32).unwrap().mesh, front.id);
                assert_eq!(pixel(&r, &target), [255, 0, 0, 255]);
            }
        });
    }

    #[test]
    #[ignore = "requires a native GPU or Mesa Vulkan software adapter"]
    fn transparent_order_and_procedural_alpha_pick() {
        pollster::block_on(async {
            let inst = instance();
            let (_, device, queue) = device(&inst, None).await.unwrap();
            let mut r =
                Renderer::new(device, queue, wgpu::TextureFormat::Rgba8UnormSrgb, 64, 64).unwrap();
            let mut p = cesium_formats::decode_content(
                include_bytes!("../../../tests/fixtures/triangle.glb"),
                &Default::default(),
            )
            .unwrap()
            .primitives
            .remove(0);
            p.material.unlit = true;
            p.material.double_sided = true;
            let camera = Camera::look_at(
                DVec3::new(0., 20., 100.),
                DVec3::new(0., 20., 0.),
                DVec3::Y,
                64,
                64,
            )
            .unwrap();
            let target = attachment(
                &r.device,
                64,
                64,
                r.format(),
                wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            );
            let draw = |r: &mut Renderer, meshes: &[&GpuMesh]| {
                r.render(
                    &target.create_view(&Default::default()),
                    camera,
                    meshes,
                    Some((32, 32)),
                )
                .unwrap();
                wait(r);
                (pixel(r, &target), r.poll_pick().unwrap())
            };
            p.material.base_color = [0., 0., 1., 1.];
            let back = r
                .upload(&p, DMat4::from_translation(DVec3::new(0., 0., -20.)), true)
                .unwrap();
            p.material.alpha_blend = true;
            p.material.base_color = [1., 0., 0., 0.5];
            let middle = r.upload(&p, DMat4::IDENTITY, true).unwrap();
            p.material.base_color = [0., 1., 0., 0.5];
            let front = r
                .upload(&p, DMat4::from_translation(DVec3::new(0., 0., 20.)), true)
                .unwrap();
            let (color, pick) = draw(&mut r, &[&front, &back, &middle]);
            assert_eq!(pick.unwrap().mesh, front.id);
            for (actual, expected) in color[..3].iter().zip([137i16, 188, 137]) {
                assert!(
                    (*actual as i16 - expected).abs() < 3,
                    "linear alpha blend must match expected RGB"
                );
            }
            assert_eq!(
                draw(&mut r, &[&middle, &front, &back]).0,
                color,
                "input order must not change nonintersecting transparent composition"
            );
            p.material.base_color = [1., 1., 1., 0.];
            let invisible = r
                .upload(&p, DMat4::from_translation(DVec3::new(0., 0., 40.)), true)
                .unwrap();
            let (same, pick) = draw(&mut r, &[&invisible, &middle, &back, &front]);
            assert_eq!(same, color);
            assert_eq!(pick.unwrap().mesh, front.id);
            p.material.alpha_blend = false;
            p.material.base_color = [1.; 4];
            let occluder = r
                .upload(&p, DMat4::from_translation(DVec3::new(0., 0., 50.)), true)
                .unwrap();
            let (c, pick) = draw(&mut r, &[&front, &middle, &back, &occluder]);
            assert_eq!(c, [255; 4]);
            assert_eq!(pick.unwrap().mesh, occluder.id);
            p.material.alpha_blend = true;
            p.material.base_color = [1., 0., 0., 0.];
            let render_pattern = |r: &mut Renderer, p: &cesium_formats::Primitive| {
                let m = r.upload(p, DMat4::IDENTITY, true).unwrap();
                let (c, pick) = draw(r, &[&m, &back]);
                (c, pick.unwrap().mesh == m.id)
            };
            p.material.pattern = Some(cesium_formats::Pattern {
                kind: 1,
                secondary: [0., 1., 0., 1.],
                repeat: [4., 4.],
                offset: 0.,
                axis: 0,
                thickness: [1.; 2],
                cell_alpha: 0.,
            });
            for v in &mut p.vertices {
                v.uv = [0.1, 0.1];
            }
            assert!(!render_pattern(&mut r, &p).1);
            for v in &mut p.vertices {
                v.uv = [0.1, 0.8];
            }
            let (c, hit) = render_pattern(&mut r, &p);
            assert!(hit && c[1] > 250 && c[0] < 5);
            p.material.pattern.as_mut().unwrap().kind = 2;
            p.material.pattern.as_mut().unwrap().repeat = [2.; 2];
            for v in &mut p.vertices {
                v.uv = [0.1, 0.1];
            }
            assert!(!render_pattern(&mut r, &p).1);
            for v in &mut p.vertices {
                v.uv = [0.6, 0.1];
            }
            assert!(render_pattern(&mut r, &p).1);
            p.material.base_color = [0., 1., 0., 1.];
            p.material.pattern.as_mut().unwrap().kind = 3;
            for v in &mut p.vertices {
                v.uv = [0.25, 0.25];
            }
            assert!(!render_pattern(&mut r, &p).1);
            for v in &mut p.vertices {
                v.uv = [0.5, 0.5];
            }
            assert!(render_pattern(&mut r, &p).1);
            p.material.pattern.as_mut().unwrap().thickness = [0.; 2];
            assert!(!render_pattern(&mut r, &p).1);
            p.material.pattern.as_mut().unwrap().kind = 6;
            p.material.base_color = [1., 1., 0., 1.];
            p.material.pattern.as_mut().unwrap().secondary = [0., 1., 1., 0.];
            for v in &mut p.vertices {
                v.uv = [0.25, 0.25];
            }
            let (dot, hit) = render_pattern(&mut r, &p);
            assert!(hit && dot[0] > 250 && dot[1] > 250 && dot[2] < 5);
            for v in &mut p.vertices {
                v.uv = [0., 0.];
            }
            assert!(
                !render_pattern(&mut r, &p).1,
                "transparent Dot background must pick through"
            );
        });
    }

    #[test]
    #[ignore = "requires a native GPU or Mesa Vulkan software adapter"]
    fn textured_material_channels_and_alpha_pick() {
        pollster::block_on(async {
            let inst = instance();
            let (_, device, queue) = device(&inst, None).await.unwrap();
            let mut r =
                Renderer::new(device, queue, wgpu::TextureFormat::Rgba8UnormSrgb, 64, 64).unwrap();
            let content = cesium_formats::decode_content(
                include_bytes!("../../../tests/fixtures/textured.glb"),
                &Default::default(),
            )
            .unwrap();
            let mut p = content.primitives[0].clone();
            let camera = Camera::look_at(
                DVec3::new(0.0, 20.0, 100.0),
                DVec3::new(0.0, 20.0, 0.0),
                DVec3::Y,
                64,
                64,
            )
            .unwrap();
            let target = attachment(
                &r.device,
                64,
                64,
                r.format(),
                wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            );
            let render = |r: &mut Renderer, p: &cesium_formats::Primitive| {
                let mesh = r.upload(p, DMat4::IDENTITY, true).unwrap();
                assert_eq!(mesh.bytes, p.gpu_bytes());
                r.render(
                    &target.create_view(&Default::default()),
                    camera,
                    &[&mesh],
                    Some((32, 32)),
                )
                .unwrap();
                wait(r);
                let hit = r.poll_pick().unwrap().is_some();
                (pixel(r, &target), hit)
            };
            let image = |rgba: [u8; 4]| {
                std::sync::Arc::new(cesium_formats::TextureImage {
                    width: 1,
                    height: 1,
                    rgba: rgba.to_vec(),
                })
            };
            let (green, hit) = render(&mut r, &p);
            assert!(hit && green[1] > green[0] && green[1] > green[2]);
            p.material.textures[3].as_mut().unwrap().image = image([0, 0, 0, 255]);
            let (occluded, _) = render(&mut r, &p);
            assert_ne!(green, occluded);
            p.material.textures[2].as_mut().unwrap().image = image([255, 128, 128, 255]);
            let (normal, _) = render(&mut r, &p);
            assert_ne!(occluded, normal);
            p.material.textures[1].as_mut().unwrap().image = image([0, 80, 255, 255]);
            let (metal, _) = render(&mut r, &p);
            assert_ne!(normal, metal);
            p.material.emissive = [0.0, 0.0, 1.0];
            p.material.textures[4].as_mut().unwrap().image = image([0, 0, 255, 255]);
            let (emissive, _) = render(&mut r, &p);
            assert!(emissive[2] > metal[2] + 30);
            p.material.textures[0].as_mut().unwrap().image = image([255, 255, 255, 0]);
            let (_, hit) = render(&mut r, &p);
            assert!(!hit); // Alpha from the texture is shared by color and ID passes.
            p.material.alpha_mask = None;
            let (_, hit) = render(&mut r, &p);
            assert!(hit); // OPAQUE ignores texture alpha.
            let mut uv = content.primitives[0].clone();
            for vertex in &mut uv.vertices {
                vertex.uv = [0.25, 0.25];
                vertex.uv1 = [0.75, 0.25];
            }
            uv.material.textures[0].as_mut().unwrap().image =
                std::sync::Arc::new(cesium_formats::TextureImage {
                    width: 2,
                    height: 2,
                    rgba: vec![
                        255, 0, 0, 255, 0, 255, 0, 255, 255, 0, 0, 255, 255, 0, 0, 255,
                    ],
                });
            let (uv1, _) = render(&mut r, &uv);
            assert!(uv1[1] > uv1[0]);
            uv.material.textures[0].as_mut().unwrap().tex_coord = 0;
            let (uv0, _) = render(&mut r, &uv);
            assert!(uv0[0] > uv0[1]);
            for vertex in &mut uv.vertices {
                vertex.color = [0.1, 1.0, 1.0, 1.0];
            }
            let (tinted, _) = render(&mut r, &uv);
            assert!(tinted[0] < uv0[0]);
            r.resize(64, 64).unwrap();
        });
    }

    #[test]
    #[ignore = "requires a native GPU or Mesa Vulkan software adapter"]
    fn offscreen_render_and_pick() {
        pollster::block_on(async {
            let instance = instance();
            let (_, device, queue) = device(&instance, None).await.unwrap();
            let mut r =
                Renderer::new(device, queue, wgpu::TextureFormat::Rgba8UnormSrgb, 64, 64).unwrap();
            let c = cesium_formats::decode_content(
                include_bytes!("../../../tests/fixtures/triangle.glb"),
                &Default::default(),
            )
            .unwrap();
            let mesh = r.upload(&c.primitives[0], DMat4::IDENTITY, true).unwrap();
            let camera = Camera::look_at(
                DVec3::new(0.0, 20.0, 100.0),
                DVec3::new(0.0, 20.0, 0.0),
                DVec3::Y,
                64,
                64,
            )
            .unwrap();
            let target = attachment(
                &r.device,
                64,
                64,
                r.format(),
                wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            );
            r.render(
                &target.create_view(&Default::default()),
                camera,
                &[&mesh],
                Some((32, 32)),
            )
            .unwrap();
            r.device
                .poll(wgpu::PollType::Wait {
                    submission_index: None,
                    timeout: Some(std::time::Duration::from_secs(30)),
                })
                .unwrap();
            assert_eq!(
                r.poll_pick(),
                Some(Some(FeatureRef {
                    mesh: mesh.id,
                    feature: 0
                }))
            );
            r.render(
                &target.create_view(&Default::default()),
                camera,
                &[&mesh],
                Some((0, 0)),
            )
            .unwrap();
            r.device
                .poll(wgpu::PollType::Wait {
                    submission_index: None,
                    timeout: Some(std::time::Duration::from_secs(30)),
                })
                .unwrap();
            assert_eq!(r.poll_pick(), Some(None));
            let orange = pixel(&r, &target);
            assert!(orange[0] > orange[1] && orange[1] > orange[2]);
            r.selected = Some(FeatureRef {
                mesh: mesh.id,
                feature: 0,
            });
            r.render(
                &target.create_view(&Default::default()),
                camera,
                &[&mesh],
                None,
            )
            .unwrap();
            let highlighted = pixel(&r, &target);
            assert_ne!(orange, highlighted);
            r.render(
                &target.create_view(&Default::default()),
                camera,
                &[&mesh],
                Some((32, 32)),
            )
            .unwrap();
            r.resize(64, 64).unwrap();
            wait(&r);
            assert_eq!(r.poll_pick(), Some(None));
            let mut masked = c.primitives[0].clone();
            masked.material.alpha_mask = Some(0.5);
            masked.material.base_color[3] = 0.1;
            let masked = r.upload(&masked, DMat4::IDENTITY, true).unwrap();
            r.render(
                &target.create_view(&Default::default()),
                camera,
                &[&masked],
                Some((32, 32)),
            )
            .unwrap();
            wait(&r);
            assert_eq!(r.poll_pick(), Some(None));
            let mut single = c.primitives[0].clone();
            single.material.double_sided = false;
            let reflected = r
                .upload(&single, DMat4::from_scale(DVec3::new(-1.0, 1.0, 1.0)), true)
                .unwrap();
            r.render(
                &target.create_view(&Default::default()),
                camera,
                &[&reflected],
                Some((32, 32)),
            )
            .unwrap();
            wait(&r);
            assert_eq!(
                r.poll_pick(),
                Some(Some(FeatureRef {
                    mesh: reflected.id,
                    feature: 0
                }))
            );
            r.resize(0, 0).unwrap();
            r.render(
                &target.create_view(&Default::default()),
                camera,
                &[&mesh],
                None,
            )
            .unwrap();
        });
    }
}
