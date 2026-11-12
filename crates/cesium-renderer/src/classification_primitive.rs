//! ClassificationPrimitive matching CesiumJS `Scene/ClassificationPrimitive.js` (Apache-2.0).

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};

use cesium_math::{ClassificationType, DeveloperError, GeometryInstance, GeometryInstanceAttribute};

use crate::commands::{DrawCommand, DrawCommandOptions, Pass};
use crate::ground_primitive::GroundPrimitiveAppearance;
use crate::primitive_collection::{next_primitive_id, ScenePrimitive};
use crate::scene_primitive::PrimitiveRenderFrameState;

/// Options for constructing a [`ClassificationPrimitive`].
#[derive(Clone, Debug, Default)]
pub struct ClassificationPrimitiveOptions {
    pub geometry_instances: Option<Vec<GeometryInstance>>,
    pub appearance: Option<GroundPrimitiveAppearance>,
    pub show: Option<bool>,
    pub vertex_cache_optimize: Option<bool>,
    pub interleave: Option<bool>,
    pub compress_vertices: Option<bool>,
    pub release_geometry_instances: Option<bool>,
    pub allow_picking: Option<bool>,
    pub asynchronous: Option<bool>,
    pub classification_type: Option<ClassificationType>,
    pub debug_show_bounding_volume: Option<bool>,
    pub debug_show_shadow_volume: Option<bool>,
}

/// A primitive that highlights volumes enclosing geometry across Terrain and/or 3D Tiles.
#[derive(Debug)]
pub struct ClassificationPrimitive {
    pub id: u64,
    pub geometry_instances: Option<Vec<GeometryInstance>>,
    pub appearance: Option<GroundPrimitiveAppearance>,
    pub show: bool,
    pub vertex_cache_optimize: bool,
    pub interleave: bool,
    pub compress_vertices: bool,
    pub release_geometry_instances: bool,
    pub allow_picking: bool,
    pub asynchronous: bool,
    pub classification_type: ClassificationType,
    pub debug_show_bounding_volume: bool,
    pub debug_show_shadow_volume: bool,
    pub ready: bool,
    is_destroyed: AtomicBool,
    cached_attributes: HashMap<String, HashMap<String, GeometryInstanceAttribute>>,
}

impl Default for ClassificationPrimitive {
    fn default() -> Self {
        Self::new(ClassificationPrimitiveOptions::default())
    }
}

impl ClassificationPrimitive {
    /// Checks whether classification primitives are supported by the context capabilities.
    pub fn is_supported(depth_texture_or_frag_depth: bool) -> bool {
        depth_texture_or_frag_depth
    }

    /// Creates a new `ClassificationPrimitive` with options.
    pub fn new(options: ClassificationPrimitiveOptions) -> Self {
        let mut cached_attributes = HashMap::new();
        if let Some(instances) = &options.geometry_instances {
            for inst in instances {
                if let Some(id) = &inst.id {
                    cached_attributes.insert(id.clone(), inst.attributes.clone());
                }
            }
        }

        Self {
            id: next_primitive_id(),
            geometry_instances: options.geometry_instances,
            appearance: options.appearance,
            show: options.show.unwrap_or(true),
            vertex_cache_optimize: options.vertex_cache_optimize.unwrap_or(false),
            interleave: options.interleave.unwrap_or(false),
            compress_vertices: options.compress_vertices.unwrap_or(true),
            release_geometry_instances: options.release_geometry_instances.unwrap_or(true),
            allow_picking: options.allow_picking.unwrap_or(true),
            asynchronous: options.asynchronous.unwrap_or(true),
            classification_type: options.classification_type.unwrap_or(ClassificationType::Both),
            debug_show_bounding_volume: options.debug_show_bounding_volume.unwrap_or(false),
            debug_show_shadow_volume: options.debug_show_shadow_volume.unwrap_or(false),
            ready: !options.asynchronous.unwrap_or(true),
            is_destroyed: AtomicBool::new(false),
            cached_attributes,
        }
    }

    /// Updates classification primitive and pushes draw commands for matching passes.
    pub fn update(
        &mut self,
        frame_state: &mut PrimitiveRenderFrameState,
    ) -> Result<(), DeveloperError> {
        if self.is_destroyed() {
            return Ok(());
        }

        // Even when show is false, primitive becomes ready during update.
        if !self.ready {
            self.ready = true;
        }

        if !self.show {
            return Ok(());
        }

        if !frame_state.passes.render && !frame_state.passes.pick {
            return Ok(());
        }

        let has_instances = self
            .geometry_instances
            .as_ref()
            .map(|i| !i.is_empty())
            .unwrap_or(false);

        if !has_instances && self.cached_attributes.is_empty() {
            return Ok(());
        }

        let passes = match self.classification_type {
            ClassificationType::Terrain => vec![Pass::TerrainClassification],
            ClassificationType::Cesium3DTile => vec![Pass::Cesium3dTileClassification],
            ClassificationType::Both => vec![
                Pass::TerrainClassification,
                Pass::Cesium3dTileClassification,
            ],
        };

        for pass in passes {
            frame_state.command_list.push(DrawCommand::new(DrawCommandOptions {
                pass: Some(pass),
                owner: Some("ClassificationPrimitive".to_string()),
                pick_only: Some(frame_state.passes.pick && !frame_state.passes.render),
                debug_show_bounding_volume: Some(self.debug_show_bounding_volume),
                ..Default::default()
            }));
        }

        if self.release_geometry_instances {
            self.geometry_instances = None;
        }

        Ok(())
    }

    /// Retrieves an attribute for a given instance ID.
    pub fn get_geometry_instance_attribute(
        &self,
        instance_id: &str,
        attribute_name: &str,
    ) -> Option<&GeometryInstanceAttribute> {
        self.cached_attributes
            .get(instance_id)
            .and_then(|attrs| attrs.get(attribute_name))
    }

    /// Returns true if this primitive has been destroyed.
    pub fn is_destroyed(&self) -> bool {
        self.is_destroyed.load(Ordering::SeqCst)
    }

    /// Destroys this primitive.
    pub fn destroy(&self) {
        self.is_destroyed.store(true, Ordering::SeqCst);
    }
}

impl ScenePrimitive for ClassificationPrimitive {
    fn id(&self) -> u64 {
        self.id
    }

    fn destroy(&self) {
        ClassificationPrimitive::destroy(self);
    }

    fn is_destroyed(&self) -> bool {
        ClassificationPrimitive::is_destroyed(self)
    }

    fn show(&self) -> bool {
        self.show
    }

    fn set_show(&self, show: bool) {
        let _ = show;
    }
}
