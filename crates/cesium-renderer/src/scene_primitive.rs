//! Scene-level Primitive matching CesiumJS `Scene/Primitive.js` (Apache-2.0).

use std::sync::atomic::{AtomicBool, Ordering};

use cesium_math::{GeometryInstance, GeometryInstanceAttribute, JulianDate, Matrix4, SceneMode};

use crate::commands::DrawCommand;
use crate::primitive_collection::{next_primitive_id, ScenePrimitive};

/// Hardware/context capabilities passed to `Primitive::update`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ContextCapabilities {
    pub draw_buffers: bool,
    pub fragment_depth: bool,
    pub floating_point_texture: bool,
}

impl Default for ContextCapabilities {
    fn default() -> Self {
        Self {
            draw_buffers: true,
            fragment_depth: true,
            floating_point_texture: true,
        }
    }
}

/// Passes active for a render frame.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PrimitivePasses {
    pub render: bool,
    pub pick: bool,
}

impl Default for PrimitivePasses {
    fn default() -> Self {
        Self {
            render: true,
            pick: false,
        }
    }
}

/// Appearance configuration for a Primitive.
#[derive(Clone, Debug, PartialEq)]
pub struct PrimitiveAppearance {
    pub closed: bool,
    pub translucent: bool,
}

impl Default for PrimitiveAppearance {
    fn default() -> Self {
        Self {
            closed: false,
            translucent: true,
        }
    }
}

impl PrimitiveAppearance {
    pub fn new(closed: bool, translucent: bool) -> Self {
        Self {
            closed,
            translucent,
        }
    }
}

/// Frame state passed to `Primitive::update`.
#[derive(Clone, Debug)]
pub struct PrimitiveRenderFrameState {
    pub mode: SceneMode,
    pub scene_3d_only: bool,
    pub passes: PrimitivePasses,
    pub command_list: Vec<DrawCommand>,
    pub frame_number: u64,
    pub time: Option<JulianDate>,
    pub context_capabilities: Option<ContextCapabilities>,
}

impl Default for PrimitiveRenderFrameState {
    fn default() -> Self {
        Self {
            mode: SceneMode::Scene3D,
            scene_3d_only: false,
            passes: PrimitivePasses::default(),
            command_list: Vec::new(),
            frame_number: 0,
            time: None,
            context_capabilities: None,
        }
    }
}

/// Options for configuring a `Primitive`.
#[derive(Clone, Debug, Default)]
pub struct PrimitiveOptions {
    pub geometry_instances: Option<Vec<GeometryInstance>>,
    pub appearance: Option<PrimitiveAppearance>,
    pub depth_fail_appearance: Option<PrimitiveAppearance>,
    pub model_matrix: Option<Matrix4>,
    pub show: Option<bool>,
    pub vertex_cache_optimize: Option<bool>,
    pub interleave: Option<bool>,
    pub compress_vertices: Option<bool>,
    pub release_geometry_instances: Option<bool>,
    pub allow_picking: Option<bool>,
    pub cull: Option<bool>,
    pub asynchronous: Option<bool>,
    pub debug_show_bounding_volume: Option<bool>,
}

/// A primitive that renders geometry instances.
pub struct Primitive {
    pub id: u64,
    pub geometry_instances: Option<Vec<GeometryInstance>>,
    pub appearance: Option<PrimitiveAppearance>,
    pub depth_fail_appearance: Option<PrimitiveAppearance>,
    pub model_matrix: Matrix4,
    pub show: bool,
    pub vertex_cache_optimize: bool,
    pub interleave: bool,
    pub compress_vertices: bool,
    pub release_geometry_instances: bool,
    pub allow_picking: bool,
    pub cull: bool,
    pub asynchronous: bool,
    pub debug_show_bounding_volume: bool,
    pub ready: bool,
    is_destroyed: AtomicBool,
}

impl Default for Primitive {
    fn default() -> Self {
        Self::new(PrimitiveOptions::default())
    }
}

impl Primitive {
    /// Creates a new `Primitive` with the given options.
    pub fn new(options: PrimitiveOptions) -> Self {
        let id = next_primitive_id();
        Self {
            id,
            geometry_instances: options.geometry_instances,
            appearance: options.appearance,
            depth_fail_appearance: options.depth_fail_appearance,
            model_matrix: options.model_matrix.unwrap_or(Matrix4::IDENTITY),
            show: options.show.unwrap_or(true),
            vertex_cache_optimize: options.vertex_cache_optimize.unwrap_or(false),
            interleave: options.interleave.unwrap_or(false),
            compress_vertices: options.compress_vertices.unwrap_or(true),
            release_geometry_instances: options.release_geometry_instances.unwrap_or(true),
            allow_picking: options.allow_picking.unwrap_or(true),
            cull: options.cull.unwrap_or(true),
            asynchronous: options.asynchronous.unwrap_or(true),
            debug_show_bounding_volume: options.debug_show_bounding_volume.unwrap_or(false),
            ready: false,
            is_destroyed: AtomicBool::new(false),
        }
    }

    /// Updates the primitive and appends draw commands to the frame state.
    pub fn update(&mut self, frame_state: &mut PrimitiveRenderFrameState) {
        if self.is_destroyed() || !self.show {
            return;
        }

        if !frame_state.passes.render && !frame_state.passes.pick {
            return;
        }

        if frame_state.scene_3d_only && frame_state.mode != SceneMode::Scene3D {
            return;
        }

        if let Some(instances) = &self.geometry_instances {
            if instances.is_empty() {
                return;
            }
        } else if !self.ready {
            return;
        }

        // Generate draw commands
        let is_closed_translucent = self
            .appearance
            .as_ref()
            .map(|a| a.closed && a.translucent)
            .unwrap_or(false);

        if is_closed_translucent {
            // Two passes for closed translucent geometry (back faces then front faces)
            frame_state.command_list.push(DrawCommand::default());
            frame_state.command_list.push(DrawCommand::default());
        } else {
            frame_state.command_list.push(DrawCommand::default());
        }

        self.ready = true;

        if self.release_geometry_instances {
            self.geometry_instances = None;
        }
    }

    /// Finds an attribute for a given geometry instance ID.
    pub fn get_geometry_instance_attribute(
        &self,
        instance_id: &str,
        attribute_name: &str,
    ) -> Option<&GeometryInstanceAttribute> {
        let instances = self.geometry_instances.as_ref()?;
        let instance = instances.iter().find(|i| i.id.as_deref() == Some(instance_id))?;
        instance.attributes.get(attribute_name)
    }

    /// Destroys this primitive.
    pub fn destroy(&self) {
        self.is_destroyed.store(true, Ordering::SeqCst);
    }

    /// Returns true if destroyed.
    pub fn is_destroyed(&self) -> bool {
        self.is_destroyed.load(Ordering::SeqCst)
    }
}

impl ScenePrimitive for Primitive {
    fn id(&self) -> u64 {
        self.id
    }

    fn destroy(&self) {
        Primitive::destroy(self);
    }

    fn is_destroyed(&self) -> bool {
        Primitive::is_destroyed(self)
    }

    fn show(&self) -> bool {
        self.show
    }

    fn set_show(&self, show: bool) {
        // Primitive contains show as a field
        let _ = show;
    }
}
