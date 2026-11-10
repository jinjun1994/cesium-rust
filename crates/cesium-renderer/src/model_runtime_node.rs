//! ModelRuntimeNode and ModelRuntimePrimitive matching CesiumJS `Scene/Model/ModelRuntimeNode.js` and `ModelRuntimePrimitive.js` (Apache-2.0).

use cesium_math::{BoundingSphere, Cartesian3, Matrix4, Quaternion};

use crate::model_draw_command::ModelDrawCommand;

/// Runtime primitive belonging to a `ModelRuntimeNode`.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ModelRuntimePrimitive {
    pub draw_command: Option<ModelDrawCommand>,
    pub bounding_sphere: Option<BoundingSphere>,
    pub bounding_sphere_2d: Option<BoundingSphere>,
}

impl ModelRuntimePrimitive {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_command(draw_command: ModelDrawCommand, bounding_sphere: Option<BoundingSphere>) -> Self {
        Self {
            draw_command: Some(draw_command),
            bounding_sphere,
            bounding_sphere_2d: None,
        }
    }
}

/// Options to instantiate a [`ModelRuntimeNode`].
#[derive(Clone, Debug)]
pub struct ModelRuntimeNodeOptions {
    pub name: String,
    pub id: usize,
    pub transform: Matrix4,
    pub transform_to_root: Matrix4,
    pub children: Vec<usize>,
    pub show: Option<bool>,
}

/// An in-memory representation of a node in a model scene graph.
#[derive(Clone, Debug, PartialEq)]
pub struct ModelRuntimeNode {
    pub name: String,
    pub id: usize,
    pub translation: Cartesian3,
    pub rotation: Quaternion,
    pub scale: Cartesian3,
    pub morph_weights: Vec<f64>,
    pub original_transform: Matrix4,
    pub transform: Matrix4,
    pub transform_to_root: Matrix4,
    pub computed_transform: Matrix4,
    pub transform_dirty: bool,
    pub show: bool,
    pub user_animated: bool,
    pub children: Vec<usize>,
    pub runtime_primitives: Vec<ModelRuntimePrimitive>,
}

impl Default for ModelRuntimeNode {
    fn default() -> Self {
        Self {
            name: String::new(),
            id: 0,
            translation: Cartesian3::ZERO,
            rotation: Quaternion::IDENTITY,
            scale: Cartesian3::new(1.0, 1.0, 1.0),
            morph_weights: Vec::new(),
            original_transform: Matrix4::IDENTITY,
            transform: Matrix4::IDENTITY,
            transform_to_root: Matrix4::IDENTITY,
            computed_transform: Matrix4::IDENTITY,
            transform_dirty: false,
            show: true,
            user_animated: false,
            children: Vec::new(),
            runtime_primitives: Vec::new(),
        }
    }
}

impl ModelRuntimeNode {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_options(options: ModelRuntimeNodeOptions) -> Self {
        let original_transform = options.transform;
        let transform = options.transform;
        let transform_to_root = options.transform_to_root;
        let computed_transform = Matrix4::IDENTITY;

        Self {
            name: options.name,
            id: options.id,
            translation: Cartesian3::ZERO,
            rotation: Quaternion::IDENTITY,
            scale: Cartesian3::new(1.0, 1.0, 1.0),
            morph_weights: Vec::new(),
            original_transform,
            transform,
            transform_to_root,
            computed_transform,
            transform_dirty: false,
            show: options.show.unwrap_or(true),
            user_animated: false,
            children: options.children,
            runtime_primitives: Vec::new(),
        }
    }

    pub fn with_transform(transform: Matrix4) -> Self {
        Self {
            original_transform: transform,
            transform,
            ..Default::default()
        }
    }

    pub fn computed_transform(&self) -> Matrix4 {
        Matrix4::from_translation_quaternion_rotation_scale(
            self.translation,
            self.rotation,
            self.scale,
        )
    }

    /// Sets the node transform and marks it dirty.
    pub fn set_transform(&mut self, transform: Matrix4) {
        self.transform = transform;
        self.transform_dirty = true;
    }

    /// Recalculates `computed_transform = root_matrix * transform_to_root * transform`.
    pub fn update_computed_transform(&mut self, root_matrix: &Matrix4) {
        let combined = root_matrix.multiply(&self.transform_to_root);
        self.computed_transform = combined.multiply(&self.transform);
    }
}
