//! Built-in Cesium GLSL automatic uniforms matching CesiumJS `Renderer/AutomaticUniforms.js` (Apache-2.0).

use crate::uniform::{UniformMap, UniformType, UniformValue};
use cesium_math::{Cartesian3, Cartesian4, Matrix3, Matrix4};
use std::collections::BTreeMap;
use std::sync::Arc;

/// Dynamic runtime state backing automatic uniforms.
#[derive(Clone, Debug, PartialEq)]
pub struct UniformState {
    pub viewport: Cartesian4,
    pub viewport_orthographic: Matrix4,
    pub viewport_transformation: Matrix4,
    pub model: Matrix4,
    pub view: Matrix4,
    pub projection: Matrix4,
    pub model_view: Matrix4,
    pub model_view_projection: Matrix4,
    pub inverse_model: Matrix4,
    pub inverse_view: Matrix4,
    pub inverse_projection: Matrix4,
    pub inverse_model_view: Matrix4,
    pub normal: Matrix3,
    pub sun_position_wc: Cartesian3,
    pub sun_direction_ec: Cartesian3,
    pub sun_direction_wc: Cartesian3,
    pub frame_number: f64,
}

impl Default for UniformState {
    fn default() -> Self {
        Self {
            viewport: Cartesian4::new(0.0, 0.0, 1024.0, 768.0),
            viewport_orthographic: Matrix4::IDENTITY,
            viewport_transformation: Matrix4::IDENTITY,
            model: Matrix4::IDENTITY,
            view: Matrix4::IDENTITY,
            projection: Matrix4::IDENTITY,
            model_view: Matrix4::IDENTITY,
            model_view_projection: Matrix4::IDENTITY,
            inverse_model: Matrix4::IDENTITY,
            inverse_view: Matrix4::IDENTITY,
            inverse_projection: Matrix4::IDENTITY,
            inverse_model_view: Matrix4::IDENTITY,
            normal: Matrix3::IDENTITY,
            sun_position_wc: Cartesian3::new(0.0, 0.0, 1.0),
            sun_direction_ec: Cartesian3::new(0.0, 0.0, 1.0),
            sun_direction_wc: Cartesian3::new(0.0, 0.0, 1.0),
            frame_number: 0.0,
        }
    }
}

/// Description of an automatic uniform.
#[derive(Clone, Debug)]
pub struct AutomaticUniform {
    pub size: usize,
    pub uniform_type: UniformType,
}

impl AutomaticUniform {
    pub fn new(size: usize, uniform_type: UniformType) -> Self {
        Self { size, uniform_type }
    }

    /// Generates GLSL uniform declaration syntax, e.g. `uniform vec4 czm_viewport;`
    pub fn get_declaration(&self, name: &str) -> String {
        let type_str = match self.uniform_type {
            UniformType::Float => "float",
            UniformType::FloatVec2 => "vec2",
            UniformType::FloatVec3 => "vec3",
            UniformType::FloatVec4 => "vec4",
            UniformType::Int => "int",
            UniformType::IntVec2 => "ivec2",
            UniformType::IntVec3 => "ivec3",
            UniformType::IntVec4 => "ivec4",
            UniformType::Bool => "bool",
            UniformType::BoolVec2 => "bvec2",
            UniformType::BoolVec3 => "bvec3",
            UniformType::BoolVec4 => "bvec4",
            UniformType::FloatMat2 => "mat2",
            UniformType::FloatMat3 => "mat3",
            UniformType::FloatMat4 => "mat4",
            UniformType::Sampler2D => "sampler2D",
            UniformType::SamplerCube => "samplerCube",
        };

        if self.size == 1 {
            format!("uniform {type_str} {name};")
        } else {
            format!("uniform {type_str} {name}[{}];", self.size)
        }
    }
}

/// Dictionary of standard CesiumJS `czm_*` automatic uniform definitions and evaluator.
pub struct AutomaticUniforms;

impl AutomaticUniforms {
    /// Returns the schema dictionary of all registered automatic uniforms.
    pub fn definitions() -> BTreeMap<&'static str, AutomaticUniform> {
        let mut defs = BTreeMap::new();
        defs.insert("czm_viewport", AutomaticUniform::new(1, UniformType::FloatVec4));
        defs.insert("czm_viewportOrthographic", AutomaticUniform::new(1, UniformType::FloatMat4));
        defs.insert("czm_viewportTransformation", AutomaticUniform::new(1, UniformType::FloatMat4));
        defs.insert("czm_model", AutomaticUniform::new(1, UniformType::FloatMat4));
        defs.insert("czm_view", AutomaticUniform::new(1, UniformType::FloatMat4));
        defs.insert("czm_projection", AutomaticUniform::new(1, UniformType::FloatMat4));
        defs.insert("czm_modelView", AutomaticUniform::new(1, UniformType::FloatMat4));
        defs.insert("czm_modelViewProjection", AutomaticUniform::new(1, UniformType::FloatMat4));
        defs.insert("czm_inverseModel", AutomaticUniform::new(1, UniformType::FloatMat4));
        defs.insert("czm_inverseView", AutomaticUniform::new(1, UniformType::FloatMat4));
        defs.insert("czm_inverseProjection", AutomaticUniform::new(1, UniformType::FloatMat4));
        defs.insert("czm_inverseModelView", AutomaticUniform::new(1, UniformType::FloatMat4));
        defs.insert("czm_normal", AutomaticUniform::new(1, UniformType::FloatMat3));
        defs.insert("czm_sunPositionWC", AutomaticUniform::new(1, UniformType::FloatVec3));
        defs.insert("czm_sunDirectionEC", AutomaticUniform::new(1, UniformType::FloatVec3));
        defs.insert("czm_sunDirectionWC", AutomaticUniform::new(1, UniformType::FloatVec3));
        defs.insert("czm_frameNumber", AutomaticUniform::new(1, UniformType::Float));
        defs
    }

    /// Evaluates the automatic uniform value from `UniformState`.
    pub fn get_value(name: &str, state: &UniformState) -> Option<UniformValue> {
        match name {
            "czm_viewport" => Some(state.viewport.into()),
            "czm_viewportOrthographic" => Some(state.viewport_orthographic.into()),
            "czm_viewportTransformation" => Some(state.viewport_transformation.into()),
            "czm_model" => Some(state.model.into()),
            "czm_view" => Some(state.view.into()),
            "czm_projection" => Some(state.projection.into()),
            "czm_modelView" => Some(state.model_view.into()),
            "czm_modelViewProjection" => Some(state.model_view_projection.into()),
            "czm_inverseModel" => Some(state.inverse_model.into()),
            "czm_inverseView" => Some(state.inverse_view.into()),
            "czm_inverseProjection" => Some(state.inverse_projection.into()),
            "czm_inverseModelView" => Some(state.inverse_model_view.into()),
            "czm_normal" => Some(state.normal.into()),
            "czm_sunPositionWC" => Some(state.sun_position_wc.into()),
            "czm_sunDirectionEC" => Some(state.sun_direction_ec.into()),
            "czm_sunDirectionWC" => Some(state.sun_direction_wc.into()),
            "czm_frameNumber" => Some(UniformValue::Float(state.frame_number as f32)),
            _ => None,
        }
    }

    /// Creates a dynamic `UniformMap` bound to an `Arc<parking_lot::RwLock<UniformState>>` or thread-safe state.
    pub fn create_uniform_map(state: Arc<std::sync::RwLock<UniformState>>) -> UniformMap {
        let mut map = UniformMap::new();
        for (name, _) in Self::definitions() {
            let state_clone = Arc::clone(&state);
            let name_string = name.to_string();
            map.set_getter(name, move || {
                let st = state_clone.read().unwrap();
                Self::get_value(&name_string, &st).unwrap_or(UniformValue::Float(0.0))
            });
        }
        map
    }
}
