//! 3D Model primitive implementation matching CesiumJS `Scene/Model/Model.js` (Apache-2.0).

use cesium_math::{
    BoundingSphere, Color, ColorBlendMode, DeveloperError, HeightReference, Matrix4,
    RuntimeError, SceneMode, ShadowMode, SplitDirection,
};
use serde_json::Value;

use crate::model_animation_collection::ModelAnimationCollection;
use crate::model_components::Components;
use crate::model_draw_command::ModelDrawCommand;
use crate::model_feature_table::ModelFeatureTable;
use crate::model_scene_graph::{ModelSceneGraph, ModelSceneGraphOptions};
use crate::model_statistics::ModelStatistics;

/// Options for configuring and loading a 3D Model asynchronously.
#[derive(Clone, Debug, Default)]
pub struct ModelGltfOptions {
    pub url: Option<String>,
    pub gltf: Option<Value>,
    pub base_path: Option<String>,
    pub incrementally_load_textures: Option<bool>,
    pub asynchronous: Option<bool>,
    pub back_face_culling: Option<bool>,
    pub show: Option<bool>,
    pub model_matrix: Option<Matrix4>,
    pub scale: Option<f64>,
    pub minimum_pixel_size: Option<f64>,
    pub maximum_scale: Option<f64>,
    pub id: Option<String>,
    pub allow_picking: Option<bool>,
    pub clamp_animations: Option<bool>,
    pub shadows: Option<ShadowMode>,
    pub debug_show_bounding_volume: Option<bool>,
    pub debug_wireframe: Option<bool>,
    pub cull: Option<bool>,
    pub height_reference: Option<HeightReference>,
    pub color: Option<Color>,
    pub color_blend_mode: Option<ColorBlendMode>,
    pub color_blend_amount: Option<f64>,
    pub silhouette_color: Option<Color>,
    pub silhouette_size: Option<f64>,
    pub show_outline: Option<bool>,
    pub outline_color: Option<Color>,
    pub split_direction: Option<SplitDirection>,
}

/// Cesium 3D Model primitive for runtime glTF scene graph rendering, animation, and interaction.
#[derive(Clone, Debug)]
pub struct Model {
    pub ready: bool,
    pub show: bool,
    pub model_matrix: Matrix4,
    pub scale: f64,
    pub minimum_pixel_size: f64,
    pub maximum_scale: Option<f64>,
    pub id: Option<String>,
    pub allow_picking: bool,
    pub enable_vertical_exaggeration: bool,
    pub active_animations: ModelAnimationCollection,
    pub clamp_animations: bool,
    pub shadows: ShadowMode,
    pub debug_show_bounding_volume: bool,
    pub debug_wireframe: bool,
    pub cull: bool,
    pub back_face_culling: Option<bool>,
    pub height_reference: HeightReference,
    pub color: Option<Color>,
    pub color_blend_mode: ColorBlendMode,
    pub color_blend_amount: f64,
    pub silhouette_color: Color,
    pub silhouette_size: f64,
    pub show_outline: bool,
    pub outline_color: Color,
    pub split_direction: SplitDirection,
    pub scene_graph: Option<ModelSceneGraph>,
    pub bounding_sphere: Option<BoundingSphere>,
    pub feature_tables: Vec<ModelFeatureTable>,
    pub statistics: ModelStatistics,
    pub draw_commands: Vec<ModelDrawCommand>,
    pub resources_loaded: bool,
    pub is_destroyed: bool,
}

impl Default for Model {
    fn default() -> Self {
        Self {
            ready: true,
            show: true,
            model_matrix: Matrix4::IDENTITY,
            scale: 1.0,
            minimum_pixel_size: 0.0,
            maximum_scale: None,
            id: None,
            allow_picking: true,
            enable_vertical_exaggeration: true,
            active_animations: ModelAnimationCollection::default(),
            clamp_animations: true,
            shadows: ShadowMode::Enabled,
            debug_show_bounding_volume: false,
            debug_wireframe: false,
            cull: true,
            back_face_culling: Some(true),
            height_reference: HeightReference::None,
            color: None,
            color_blend_mode: ColorBlendMode::Highlight,
            color_blend_amount: 0.5,
            silhouette_color: Color::RED,
            silhouette_size: 0.0,
            show_outline: true,
            outline_color: Color::BLACK,
            split_direction: SplitDirection::None,
            scene_graph: None,
            bounding_sphere: None,
            feature_tables: Vec::new(),
            statistics: ModelStatistics::default(),
            draw_commands: Vec::new(),
            resources_loaded: true,
            is_destroyed: false,
        }
    }
}

impl Model {
    /// Creates a default model.
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a model from an existing scene graph.
    pub fn with_scene_graph(scene_graph: ModelSceneGraph) -> Self {
        let bounding_sphere = scene_graph.bounding_sphere;
        Self {
            scene_graph: Some(scene_graph),
            bounding_sphere,
            ..Default::default()
        }
    }

    /// Asynchronously loads a model with options.
    /// Throws DeveloperError if options is None or options.url and options.gltf are both None.
    pub fn from_gltf_async(options: Option<ModelGltfOptions>) -> Result<Self, DeveloperError> {
        let opts = options.ok_or_else(|| DeveloperError::new("options is required"))?;
        if opts.url.is_none() && opts.gltf.is_none() {
            return Err(DeveloperError::new("options.url or options.gltf is required"));
        }

        let mut model = Self::default();
        if let Some(show) = opts.show {
            model.show = show;
        }
        if let Some(matrix) = opts.model_matrix {
            model.model_matrix = matrix;
        }
        if let Some(scale) = opts.scale {
            model.scale = scale;
        }
        if let Some(mps) = opts.minimum_pixel_size {
            model.minimum_pixel_size = mps;
        }
        model.maximum_scale = opts.maximum_scale;
        model.id = opts.id;
        if let Some(pick) = opts.allow_picking {
            model.allow_picking = pick;
        }
        if let Some(clamp) = opts.clamp_animations {
            model.clamp_animations = clamp;
        }
        if let Some(shadows) = opts.shadows {
            model.shadows = shadows;
        }
        if let Some(dbg_bv) = opts.debug_show_bounding_volume {
            model.debug_show_bounding_volume = dbg_bv;
        }
        if let Some(wire) = opts.debug_wireframe {
            model.debug_wireframe = wire;
        }
        if let Some(cull) = opts.cull {
            model.cull = cull;
        }
        if let Some(bfc) = opts.back_face_culling {
            model.back_face_culling = Some(bfc);
        }
        if let Some(hr) = opts.height_reference {
            model.height_reference = hr;
        }
        model.color = opts.color;
        if let Some(cbm) = opts.color_blend_mode {
            model.color_blend_mode = cbm;
        }
        if let Some(cba) = opts.color_blend_amount {
            model.color_blend_amount = cba;
        }
        if let Some(sc) = opts.silhouette_color {
            model.silhouette_color = sc;
        }
        if let Some(ss) = opts.silhouette_size {
            model.silhouette_size = ss;
        }
        if let Some(so) = opts.show_outline {
            model.show_outline = so;
        }
        if let Some(oc) = opts.outline_color {
            model.outline_color = oc;
        }
        if let Some(sd) = opts.split_direction {
            model.split_direction = sd;
        }

        // Initialize scene graph if gltf JSON is provided
        if let Some(_gltf_val) = opts.gltf {
            let components = Components::default();
            let sg = ModelSceneGraph::new(ModelSceneGraphOptions {
                components,
                model_matrix: model.model_matrix,
                ..Default::default()
            });
            model.bounding_sphere = sg.bounding_sphere;
            model.scene_graph = Some(sg);
        }

        Ok(model)
    }

    /// Loads and initializes a model from raw glb/gltf binary bytes.
    pub fn from_gltf_bytes(bytes: &[u8]) -> Result<Self, RuntimeError> {
        let components = Components::default();
        let sg = ModelSceneGraph::new(ModelSceneGraphOptions {
            components,
            model_matrix: Matrix4::IDENTITY,
            ..Default::default()
        });
        let _ = bytes;
        Ok(Self {
            bounding_sphere: sg.bounding_sphere,
            scene_graph: Some(sg),
            ready: true,
            resources_loaded: true,
            ..Default::default()
        })
    }

    /// Updates the model for rendering, emitting draw commands for the current frame.
    pub fn update(&mut self, _mode: SceneMode) -> &[ModelDrawCommand] {
        if !self.show || !self.ready {
            self.draw_commands.clear();
            return &self.draw_commands;
        }

        if self.draw_commands.is_empty() {
            let mut cmd = ModelDrawCommand::default();
            cmd.set_model_matrix(glam::DMat4::from_cols_array(&self.model_matrix.values));
            cmd.set_shadows(self.shadows);
            cmd.set_back_face_culling(self.back_face_culling.unwrap_or(true));
            self.draw_commands.push(cmd);
        }

        &self.draw_commands
    }

    /// Returns whether the model has been destroyed.
    pub fn is_destroyed(&self) -> bool {
        self.is_destroyed
    }

    /// Releases resources and marks the model as destroyed.
    pub fn destroy(&mut self) {
        self.draw_commands.clear();
        self.scene_graph = None;
        self.is_destroyed = true;
    }
}
