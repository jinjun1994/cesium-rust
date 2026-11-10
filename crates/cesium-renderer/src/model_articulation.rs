//! ModelArticulation matching CesiumJS `Scene/Model/ModelArticulation.js` (Apache-2.0).

use std::cell::RefCell;
use std::rc::Rc;

use cesium_math::{DeveloperError, Matrix4};
use crate::model_animation_channel::ModelRuntimeNode;
use crate::model_articulation_stage::{ArticulationStageConfig, ModelArticulationStage};

/// Configuration for an articulation defined in a 3D model.
#[derive(Debug, Clone, PartialEq)]
pub struct ArticulationConfig {
    pub name: String,
    pub stages: Vec<ArticulationStageConfig>,
}

impl ArticulationConfig {
    pub fn new(name: impl Into<String>, stages: Vec<ArticulationStageConfig>) -> Self {
        Self {
            name: name.into(),
            stages,
        }
    }
}

/// Dummy or scene graph reference context for `ModelArticulation`.
#[derive(Clone, Debug, PartialEq, Default)]
pub struct ModelArticulationSceneGraphContext;

/// An in-memory representation of an articulation that affects nodes in the ModelSceneGraph.
#[derive(Clone, Debug, PartialEq)]
pub struct ModelArticulation {
    pub articulation: ArticulationConfig,
    pub scene_graph: ModelArticulationSceneGraphContext,
    pub name: String,
    pub runtime_stages: Vec<ModelArticulationStage>,
    pub runtime_nodes: Vec<Rc<RefCell<ModelRuntimeNode>>>,
    pub dirty_flag: Rc<RefCell<bool>>,
}

impl ModelArticulation {
    /// Creates a new `ModelArticulation`.
    pub fn new(
        articulation: Option<ArticulationConfig>,
        scene_graph: Option<ModelArticulationSceneGraphContext>,
    ) -> Result<Self, DeveloperError> {
        let articulation =
            articulation.ok_or_else(|| DeveloperError::new("options.articulation is required."))?;
        let scene_graph =
            scene_graph.ok_or_else(|| DeveloperError::new("options.sceneGraph is required."))?;

        let name = articulation.name.clone();
        let dirty_flag = Rc::new(RefCell::new(true));

        let mut runtime_stages = Vec::with_capacity(articulation.stages.len());
        for stage in &articulation.stages {
            let runtime_stage =
                ModelArticulationStage::new(Some(stage.clone()), Some(dirty_flag.clone()))?;
            runtime_stages.push(runtime_stage);
        }

        Ok(Self {
            articulation,
            scene_graph,
            name,
            runtime_stages,
            runtime_nodes: Vec::new(),
            dirty_flag,
        })
    }

    /// The name of this articulation.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Whether this articulation has modified stage values that need to be applied.
    pub fn is_dirty(&self) -> bool {
        *self.dirty_flag.borrow()
    }

    /// Sets the dirty flag.
    pub fn set_dirty(&mut self, dirty: bool) {
        *self.dirty_flag.borrow_mut() = dirty;
    }

    /// Sets the current value of a named articulation stage.
    pub fn set_articulation_stage(&mut self, stage_name: &str, value: f64) {
        if let Some(stage) = self.runtime_stages.iter_mut().find(|s| s.name == stage_name) {
            stage.set_current_value(value);
        }
    }

    /// Retrieves an articulation stage by name.
    pub fn get_stage(&self, stage_name: &str) -> Option<&ModelArticulationStage> {
        self.runtime_stages.iter().find(|s| s.name == stage_name)
    }

    /// Applies the chain of articulation stages to the transform of each participating node.
    pub fn apply(&mut self) {
        if !self.is_dirty() {
            return;
        }
        self.set_dirty(false);

        let mut articulation_matrix = Matrix4::IDENTITY;
        for stage in &self.runtime_stages {
            articulation_matrix = stage.apply_stage_to_matrix(&articulation_matrix);
        }

        for node in &self.runtime_nodes {
            let mut borrowed = node.borrow_mut();
            let transform = borrowed
                .original_transform
                .multiply_transformation(&articulation_matrix);
            borrowed.transform = transform;
        }
    }
}
