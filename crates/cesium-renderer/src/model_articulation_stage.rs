//! ModelArticulationStage matching CesiumJS `Scene/Model/ModelArticulationStage.js` (Apache-2.0).

use std::cell::RefCell;
use std::rc::Rc;

use cesium_math::{
    clamp, ArticulationStageType, Cartesian3, DeveloperError, Matrix3, Matrix4, EPSILON16,
};

/// Configuration options for an articulation stage from 3D model metadata.
#[derive(Debug, Clone, PartialEq)]
pub struct ArticulationStageConfig {
    pub name: String,
    pub stage_type: ArticulationStageType,
    pub minimum_value: f64,
    pub maximum_value: f64,
    pub initial_value: f64,
}

impl ArticulationStageConfig {
    pub fn new(
        name: impl Into<String>,
        stage_type: ArticulationStageType,
        minimum_value: f64,
        maximum_value: f64,
        initial_value: f64,
    ) -> Self {
        Self {
            name: name.into(),
            stage_type,
            minimum_value,
            maximum_value,
            initial_value,
        }
    }
}

/// An in-memory representation of an articulation stage belonging to a `ModelArticulation`.
#[derive(Clone, Debug, PartialEq)]
pub struct ModelArticulationStage {
    pub stage: ArticulationStageConfig,
    pub name: String,
    pub stage_type: ArticulationStageType,
    pub minimum_value: f64,
    pub maximum_value: f64,
    pub current_value: f64,
    pub dirty_flag: Option<Rc<RefCell<bool>>>,
}

impl ModelArticulationStage {
    /// Creates a new `ModelArticulationStage`.
    pub fn new(
        stage: Option<ArticulationStageConfig>,
        dirty_flag: Option<Rc<RefCell<bool>>>,
    ) -> Result<Self, DeveloperError> {
        let stage = stage.ok_or_else(|| DeveloperError::new("options.stage is required."))?;
        let dirty_flag = dirty_flag
            .ok_or_else(|| DeveloperError::new("options.runtimeArticulation is required."))?;

        let name = stage.name.clone();
        let stage_type = stage.stage_type;
        let minimum_value = stage.minimum_value;
        let maximum_value = stage.maximum_value;
        let current_value = stage.initial_value;

        Ok(Self {
            stage,
            name,
            stage_type,
            minimum_value,
            maximum_value,
            current_value,
            dirty_flag: Some(dirty_flag),
        })
    }

    /// The name of this articulation stage.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// The motion type of this articulation stage.
    pub fn stage_type(&self) -> ArticulationStageType {
        self.stage_type
    }

    /// The minimum permissible value.
    pub fn minimum_value(&self) -> f64 {
        self.minimum_value
    }

    /// The maximum permissible value.
    pub fn maximum_value(&self) -> f64 {
        self.maximum_value
    }

    /// The current stage value.
    pub fn current_value(&self) -> f64 {
        self.current_value
    }

    /// Sets the current stage value, clamping within [minimum_value, maximum_value],
    /// and marks the owning runtime articulation dirty if changed.
    pub fn set_current_value(&mut self, value: f64) {
        let clamped = clamp(value, self.minimum_value, self.maximum_value);
        if (self.current_value - clamped).abs() > EPSILON16 {
            self.current_value = clamped;
            if let Some(flag) = &self.dirty_flag {
                *flag.borrow_mut() = true;
            }
        }
    }

    /// Modifies a Matrix4 by applying the transformation corresponding to this stage.
    pub fn apply_stage_to_matrix(&self, result: &Matrix4) -> Matrix4 {
        let value = self.current_value;
        match self.stage_type {
            ArticulationStageType::XRotate => {
                let rotation = Matrix3::from_rotation_x(value.to_radians());
                result.multiply_by_matrix3(&rotation)
            }
            ArticulationStageType::YRotate => {
                let rotation = Matrix3::from_rotation_y(value.to_radians());
                result.multiply_by_matrix3(&rotation)
            }
            ArticulationStageType::ZRotate => {
                let rotation = Matrix3::from_rotation_z(value.to_radians());
                result.multiply_by_matrix3(&rotation)
            }
            ArticulationStageType::XTranslate => {
                result.multiply_by_translation(Cartesian3::new(value, 0.0, 0.0))
            }
            ArticulationStageType::YTranslate => {
                result.multiply_by_translation(Cartesian3::new(0.0, value, 0.0))
            }
            ArticulationStageType::ZTranslate => {
                result.multiply_by_translation(Cartesian3::new(0.0, 0.0, value))
            }
            ArticulationStageType::XScale => {
                result.multiply_by_scale(Cartesian3::new(value, 1.0, 1.0))
            }
            ArticulationStageType::YScale => {
                result.multiply_by_scale(Cartesian3::new(1.0, value, 1.0))
            }
            ArticulationStageType::ZScale => {
                result.multiply_by_scale(Cartesian3::new(1.0, 1.0, value))
            }
            ArticulationStageType::UniformScale => result.multiply_by_uniform_scale(value),
        }
    }
}
