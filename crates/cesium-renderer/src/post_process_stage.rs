use std::collections::HashMap;
use std::rc::Rc;
use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use std::sync::Mutex;

use cesium_math::{BoundingRectangle, Color, DeveloperError};

use crate::context::ContextLimits;
use crate::framebuffer::{PixelDatatype, PixelFormat};
use crate::texture::Texture;

static NEXT_STAGE_ID: AtomicU64 = AtomicU64::new(1);
static OVERRIDE_MAX_TEXTURE_SIZE: AtomicU32 = AtomicU32::new(0);
static LAST_WARNING: Mutex<Option<(String, String)>> = Mutex::new(None);

/// Determines how input texture to a PostProcessStage is sampled.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub enum PostProcessStageSampleMode {
    #[default]
    Nearest = 0,
    Linear = 1,
}

/// Represents dynamic uniform values for post-process stages.
#[derive(Clone, Debug, PartialEq)]
pub enum PostProcessUniformValue {
    Float(f32),
    Int(i32),
    Bool(bool),
    Vec2([f32; 2]),
    Vec3([f32; 3]),
    Vec4([f32; 4]),
    Color(Color),
    TextureUri(String),
    Texture(Rc<Texture>),
}

/// A selected feature representation for per-feature post process stage.
#[derive(Clone, Debug, PartialEq)]
pub struct SelectedFeature {
    pub pick_id_color: Color,
}

impl SelectedFeature {
    pub fn new(pick_id_color: Color) -> Self {
        Self { pick_id_color }
    }
}

/// Options for configuring a PostProcessStage.
#[derive(Clone, Debug)]
pub struct PostProcessStageOptions {
    pub fragment_shader: String,
    pub uniforms: Option<HashMap<String, PostProcessUniformValue>>,
    pub texture_scale: Option<f64>,
    pub force_power_of_two: Option<bool>,
    pub sample_mode: Option<PostProcessStageSampleMode>,
    pub pixel_format: Option<PixelFormat>,
    pub pixel_datatype: Option<PixelDatatype>,
    pub clear_color: Option<Color>,
    pub scissor_rectangle: Option<BoundingRectangle>,
    pub name: Option<String>,
}

impl PostProcessStageOptions {
    pub fn new(fragment_shader: impl Into<String>) -> Self {
        Self {
            fragment_shader: fragment_shader.into(),
            uniforms: None,
            texture_scale: None,
            force_power_of_two: None,
            sample_mode: None,
            pixel_format: None,
            pixel_datatype: None,
            clear_color: None,
            scissor_rectangle: None,
            name: None,
        }
    }
}

#[derive(Clone, Debug)]
pub struct SelectedIdTextureInfo {
    pub width: u32,
    pub height: u32,
}

/// Runs a post-process stage on either the texture rendered by the scene or the output of a previous post-process stage.
#[derive(Clone, Debug)]
pub struct PostProcessStage {
    pub fragment_shader: String,
    pub uniforms: Option<HashMap<String, PostProcessUniformValue>>,
    pub texture_scale: f64,
    pub force_power_of_two: bool,
    pub sample_mode: PostProcessStageSampleMode,
    pub pixel_format: PixelFormat,
    pub pixel_datatype: PixelDatatype,
    pub clear_color: Color,
    pub scissor_rectangle: BoundingRectangle,
    pub name: String,
    pub enabled: bool,
    pub ready: bool,
    pub output_texture: Option<Rc<Texture>>,
    pub selected: Option<Vec<SelectedFeature>>,
    pub selected_id_texture: Option<SelectedIdTextureInfo>,
    is_destroyed: bool,
}

impl PostProcessStage {
    /// Creates a new PostProcessStage with the specified options.
    pub fn new(options: PostProcessStageOptions) -> Result<Self, DeveloperError> {
        let fragment_shader = options.fragment_shader.trim().to_string();
        if fragment_shader.is_empty() {
            return Err(DeveloperError::new(
                "options.fragmentShader must be provided and non-empty.",
            ));
        }

        let texture_scale = options.texture_scale.unwrap_or(1.0);
        if texture_scale <= 0.0 || texture_scale > 1.0 {
            return Err(DeveloperError::new(
                "options.textureScale must be greater than 0.0 and less than or equal to 1.0.",
            ));
        }

        let pixel_format = options.pixel_format.unwrap_or(PixelFormat::Rgba);
        if !pixel_format.is_color_format() {
            return Err(DeveloperError::new(
                "options.pixelFormat must be a color format.",
            ));
        }

        let force_power_of_two = options.force_power_of_two.unwrap_or(false);
        let sample_mode = options
            .sample_mode
            .unwrap_or(PostProcessStageSampleMode::Nearest);
        let pixel_datatype = options
            .pixel_datatype
            .unwrap_or(PixelDatatype::UnsignedByte);
        let clear_color = options.clear_color.unwrap_or(Color::BLACK);
        let scissor_rectangle = options
            .scissor_rectangle
            .unwrap_or_else(|| BoundingRectangle::new(0.0, 0.0, 0.0, 0.0));

        let name = options.name.unwrap_or_else(|| {
            let id = NEXT_STAGE_ID.fetch_add(1, Ordering::Relaxed);
            format!("stage_{id}")
        });

        Ok(Self {
            fragment_shader,
            uniforms: options.uniforms,
            texture_scale,
            force_power_of_two,
            sample_mode,
            pixel_format,
            pixel_datatype,
            clear_color,
            scissor_rectangle,
            name,
            enabled: true,
            ready: false,
            output_texture: None,
            selected: None,
            selected_id_texture: None,
            is_destroyed: false,
        })
    }

    /// Set an override for maximum texture size for testing.
    pub fn set_maximum_texture_size_override(size: Option<u32>) {
        OVERRIDE_MAX_TEXTURE_SIZE.store(size.unwrap_or(0), Ordering::SeqCst);
    }

    /// Retrieve the last one time warning recorded.
    pub fn last_warning() -> Option<(String, String)> {
        LAST_WARNING.lock().unwrap().clone()
    }

    /// Clear recorded warnings.
    pub fn clear_last_warning() {
        *LAST_WARNING.lock().unwrap() = None;
    }

    /// One-time warning utility.
    pub fn one_time_warning(identifier: &str, message: &str) {
        let mut guard = LAST_WARNING.lock().unwrap();
        *guard = Some((identifier.to_string(), message.to_string()));
    }

    /// Sets the selected features and updates the selected ID texture.
    pub fn set_selected(&mut self, selected: Option<Vec<SelectedFeature>>) {
        if let Some(features) = &selected {
            let max_size = {
                let ov = OVERRIDE_MAX_TEXTURE_SIZE.load(Ordering::SeqCst);
                if ov > 0 {
                    ov
                } else {
                    ContextLimits::MAXIMUM_TEXTURE_SIZE
                }
            };

            let len = features.len() as u32;
            let width = len.min(max_size);
            if len > max_size {
                Self::one_time_warning(
                    "postProcessStageSelectedTextureSize",
                    &format!(
                        "The number of selected feature ids ({len}) exceeds the maximum texture size ({max_size}). Features beyond the limit will not have the post-process effect applied."
                    ),
                );
            }
            self.selected_id_texture = Some(SelectedIdTextureInfo {
                width,
                height: 1,
            });
        } else {
            self.selected_id_texture = None;
        }
        self.selected = selected;
    }

    /// Mark the stage ready.
    pub fn mark_ready(&mut self) {
        self.ready = true;
    }

    /// Destroys the stage.
    pub fn destroy(&mut self) {
        self.is_destroyed = true;
        self.output_texture = None;
        self.selected = None;
        self.selected_id_texture = None;
    }

    /// Returns true if the stage was destroyed.
    pub fn is_destroyed(&self) -> bool {
        self.is_destroyed
    }
}
