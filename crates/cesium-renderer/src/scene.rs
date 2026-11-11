//! Scene matching CesiumJS `Scene/Scene.js` (Apache-2.0).

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use cesium_math::{Camera, DeveloperError, Ellipsoid, GeographicProjection, SceneMode};
use glam::DVec3;

use crate::commands::{DrawCommand, Pass};
use crate::primitive_collection::ScenePrimitive;
use crate::scene_primitive::{PrimitivePasses, PrimitiveRenderFrameState};
use crate::shared_context::SharedContext;

/// Options for constructing a `Scene`.
#[derive(Clone, Default)]
pub struct SceneOptions {
    /// Canvas name or identifier. Required.
    pub canvas: Option<String>,
    /// Optional shared context across multiple scenes.
    pub shared_context: Option<SharedContext>,
    /// Initial scene mode. Defaults to `Scene3D`.
    pub scene_mode: Option<SceneMode>,
    /// Whether to use logarithmic depth buffer.
    pub log_depth_buffer: Option<bool>,
    /// Explicit request render mode.
    pub request_render_mode: Option<bool>,
    /// Canvas width in pixels. Defaults to 800.
    pub width: Option<u32>,
    /// Canvas height in pixels. Defaults to 600.
    pub height: Option<u32>,
}

impl std::fmt::Debug for SceneOptions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SceneOptions")
            .field("canvas", &self.canvas)
            .field("scene_mode", &self.scene_mode)
            .field("log_depth_buffer", &self.log_depth_buffer)
            .field("request_render_mode", &self.request_render_mode)
            .field("width", &self.width)
            .field("height", &self.height)
            .finish()
    }
}

pub type AfterRenderCallback = Box<dyn FnOnce(&mut Scene) + Send + Sync>;
pub type DrawCommandFilter = Box<dyn Fn(&DrawCommand) -> bool + Send + Sync>;

/// The container for all 3D graphical objects and state in a Cesium virtual scene.
pub struct Scene {
    pub canvas: String,
    pub width: u32,
    pub height: u32,
    pub mode: SceneMode,
    pub morph_time: f64,
    pub background_color: [f32; 4],
    pub vertical_exaggeration: f64,
    pub vertical_exaggeration_relative_height: f64,
    pub use_log_depth: bool,
    pub request_render_mode: bool,
    pub debug_show_bounding_volume: bool,
    pub debug_show_commands: bool,
    pub debug_show_frames_per_second: bool,
    pub camera: Camera,
    pub shared_context: Option<SharedContext>,
    pub map_projection: GeographicProjection,
    primitives: Vec<Arc<dyn ScenePrimitive>>,
    after_render_callbacks: Vec<AfterRenderCallback>,
    command_filter: Option<DrawCommandFilter>,
    is_destroyed: AtomicBool,
}

impl Scene {
    /// Creates a new `Scene` instance. Throws DeveloperError if options or options.canvas is undefined/empty.
    pub fn new(options: Option<SceneOptions>) -> Result<Self, DeveloperError> {
        let opts = options.ok_or_else(|| DeveloperError::new("options is required."))?;
        let canvas = opts
            .canvas
            .ok_or_else(|| DeveloperError::new("options.canvas is required."))?;
        if canvas.is_empty() {
            return Err(DeveloperError::new("options.canvas cannot be empty."));
        }

        let width = opts.width.unwrap_or(800);
        let height = opts.height.unwrap_or(600);
        let mode = opts.scene_mode.unwrap_or(SceneMode::Scene3D);
        let morph_time = mode.get_morph_time().unwrap_or(1.0);
        let use_log_depth = opts.log_depth_buffer.unwrap_or(true);
        let request_render_mode = opts.request_render_mode.unwrap_or(false);

        let eye = DVec3::new(0.0, 0.0, 10_000_000.0);
        let target = DVec3::ZERO;
        let up = DVec3::Y;
        let camera = Camera::look_at(eye, target, up, width, height)
            .map_err(|e| DeveloperError::new(e.to_string()))?;

        Ok(Self {
            canvas,
            width,
            height,
            mode,
            morph_time,
            background_color: [0.0, 0.0, 0.0, 1.0],
            vertical_exaggeration: 1.0,
            vertical_exaggeration_relative_height: 0.0,
            use_log_depth,
            request_render_mode,
            debug_show_bounding_volume: false,
            debug_show_commands: false,
            debug_show_frames_per_second: false,
            camera,
            shared_context: opts.shared_context,
            map_projection: GeographicProjection::new(Ellipsoid::WGS84)
                .map_err(|e| DeveloperError::new(e.to_string()))?,
            primitives: Vec::new(),
            after_render_callbacks: Vec::new(),
            command_filter: None,
            is_destroyed: AtomicBool::new(false),
        })
    }

    /// Morphs the scene to Columbus View.
    pub fn morph_to_columbus_view(&mut self, duration_seconds: f64) {
        self.mode = SceneMode::ColumbusView;
        self.morph_time = if duration_seconds <= 0.0 { 0.0 } else { 0.5 };
    }

    /// Morphs the scene to 2D mode.
    pub fn morph_to_2d(&mut self, duration_seconds: f64) {
        self.mode = SceneMode::Scene2D;
        self.morph_time = if duration_seconds <= 0.0 { 0.0 } else { 0.5 };
    }

    /// Morphs the scene to 3D mode.
    pub fn morph_to_3d(&mut self, duration_seconds: f64) {
        self.mode = SceneMode::Scene3D;
        self.morph_time = if duration_seconds <= 0.0 { 1.0 } else { 0.5 };
    }

    /// Sets vertical exaggeration and its reference height.
    pub fn set_vertical_exaggeration(&mut self, exaggeration: f64, relative_height: f64) {
        self.vertical_exaggeration = exaggeration;
        self.vertical_exaggeration_relative_height = relative_height;
    }

    /// Adds a primitive to the scene.
    pub fn add_primitive(&mut self, primitive: Arc<dyn ScenePrimitive>) {
        self.primitives.push(primitive);
    }

    /// Removes a primitive from the scene.
    pub fn remove_primitive(&mut self, primitive: &Arc<dyn ScenePrimitive>) -> bool {
        if let Some(pos) = self.primitives.iter().position(|p| Arc::ptr_eq(p, primitive)) {
            self.primitives.remove(pos);
            true
        } else {
            false
        }
    }

    /// Returns the number of primitives in the scene.
    pub fn primitive_count(&self) -> usize {
        self.primitives.len()
    }

    /// Schedules a callback function to be executed after the current frame is rendered.
    pub fn add_after_render_callback<F>(&mut self, callback: F)
    where
        F: FnOnce(&mut Scene) + Send + Sync + 'static,
    {
        self.after_render_callbacks.push(Box::new(callback));
    }

    /// Sets an optional debug command filter function.
    pub fn set_debug_command_filter<F>(&mut self, filter: Option<F>)
    where
        F: Fn(&DrawCommand) -> bool + Send + Sync + 'static,
    {
        self.command_filter = filter.map(|f| Box::new(f) as DrawCommandFilter);
    }

    /// Renders a frame of the scene for specs/unit testing. Returns collected filtered commands.
    pub fn render_for_specs(&mut self) -> Result<Vec<DrawCommand>, DeveloperError> {
        if self.is_destroyed.load(Ordering::SeqCst) {
            return Err(DeveloperError::new(
                "This object was destroyed, i.e., destroy() was called.",
            ));
        }

        let mut frame_state = PrimitiveRenderFrameState {
            mode: self.mode,
            scene_3d_only: self.mode == SceneMode::Scene3D,
            passes: PrimitivePasses {
                render: true,
                pick: false,
            },
            command_list: Vec::new(),
            ..Default::default()
        };

        // Collect commands from all primitives
        for primitive in &self.primitives {
            primitive.update();
            primitive.collect_draw_commands(&mut frame_state);
        }

        let mut commands = frame_state.command_list;

        // Apply debug command filter if defined
        if let Some(ref filter) = self.command_filter {
            commands.retain(|cmd| filter(cmd));
        }

        // Sort commands: Opaque before Translucent
        commands.sort_by_key(|cmd| {
            if cmd.pass == Some(Pass::Translucent) {
                1
            } else {
                0
            }
        });

        // Execute afterRender callbacks (drained so callbacks can schedule new callbacks for subsequent frames)
        let callbacks = std::mem::take(&mut self.after_render_callbacks);
        for callback in callbacks {
            callback(self);
        }

        Ok(commands)
    }

    /// Whether this scene has been destroyed.
    pub fn is_destroyed(&self) -> bool {
        self.is_destroyed.load(Ordering::SeqCst)
    }

    /// Destroys the scene and cleans up resources.
    pub fn destroy(&mut self) {
        if !self.is_destroyed.swap(true, Ordering::SeqCst) {
            self.primitives.clear();
            self.after_render_callbacks.clear();
        }
    }
}
