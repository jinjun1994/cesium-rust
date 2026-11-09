//! GLSL Uniform representation, type validation and uniform map evaluation matching CesiumJS `Renderer/createUniform.js` (Apache-2.0).

use cesium_math::{Cartesian2, Cartesian3, Cartesian4, Color, DeveloperError, Matrix2, Matrix3, Matrix4};
use std::collections::BTreeMap;
use std::sync::Arc;

/// Supported GLSL uniform data types.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum UniformType {
    Float,
    FloatVec2,
    FloatVec3,
    FloatVec4,
    Int,
    IntVec2,
    IntVec3,
    IntVec4,
    Bool,
    BoolVec2,
    BoolVec3,
    BoolVec4,
    FloatMat2,
    FloatMat3,
    FloatMat4,
    Sampler2D,
    SamplerCube,
}

/// Dynamic value held by a uniform.
#[derive(Clone, Debug, PartialEq)]
pub enum UniformValue {
    Float(f32),
    Vec2([f32; 2]),
    Vec3([f32; 3]),
    Vec4([f32; 4]),
    Int(i32),
    IVec2([i32; 2]),
    IVec3([i32; 3]),
    IVec4([i32; 4]),
    Bool(bool),
    BVec2([bool; 2]),
    BVec3([bool; 3]),
    BVec4([bool; 4]),
    Mat2([f32; 4]),
    Mat3([f32; 9]),
    Mat4([f32; 16]),
    FloatArray(Vec<f32>),
    Vec2Array(Vec<[f32; 2]>),
    Vec3Array(Vec<[f32; 3]>),
    Vec4Array(Vec<[f32; 4]>),
    IntArray(Vec<i32>),
    IVec2Array(Vec<[i32; 2]>),
    IVec3Array(Vec<[i32; 3]>),
    IVec4Array(Vec<[i32; 4]>),
    BoolArray(Vec<bool>),
    BVec2Array(Vec<[bool; 2]>),
    BVec3Array(Vec<[bool; 3]>),
    BVec4Array(Vec<[bool; 4]>),
    Mat2Array(Vec<[f32; 4]>),
    Mat3Array(Vec<[f32; 9]>),
    Mat4Array(Vec<[f32; 16]>),
}

impl From<f32> for UniformValue {
    fn from(val: f32) -> Self {
        UniformValue::Float(val)
    }
}

impl From<f64> for UniformValue {
    fn from(val: f64) -> Self {
        UniformValue::Float(val as f32)
    }
}

impl From<i32> for UniformValue {
    fn from(val: i32) -> Self {
        UniformValue::Int(val)
    }
}

impl From<bool> for UniformValue {
    fn from(val: bool) -> Self {
        UniformValue::Bool(val)
    }
}

impl From<Cartesian2> for UniformValue {
    fn from(val: Cartesian2) -> Self {
        UniformValue::Vec2([val.x as f32, val.y as f32])
    }
}

impl From<[f32; 2]> for UniformValue {
    fn from(val: [f32; 2]) -> Self {
        UniformValue::Vec2(val)
    }
}

impl From<[i32; 2]> for UniformValue {
    fn from(val: [i32; 2]) -> Self {
        UniformValue::IVec2(val)
    }
}

impl From<[bool; 2]> for UniformValue {
    fn from(val: [bool; 2]) -> Self {
        UniformValue::BVec2(val)
    }
}

impl From<Cartesian3> for UniformValue {
    fn from(val: Cartesian3) -> Self {
        UniformValue::Vec3([val.x as f32, val.y as f32, val.z as f32])
    }
}

impl From<[f32; 3]> for UniformValue {
    fn from(val: [f32; 3]) -> Self {
        UniformValue::Vec3(val)
    }
}

impl From<[i32; 3]> for UniformValue {
    fn from(val: [i32; 3]) -> Self {
        UniformValue::IVec3(val)
    }
}

impl From<[bool; 3]> for UniformValue {
    fn from(val: [bool; 3]) -> Self {
        UniformValue::BVec3(val)
    }
}

impl From<Cartesian4> for UniformValue {
    fn from(val: Cartesian4) -> Self {
        UniformValue::Vec4([val.x as f32, val.y as f32, val.z as f32, val.w as f32])
    }
}

impl From<[f32; 4]> for UniformValue {
    fn from(val: [f32; 4]) -> Self {
        UniformValue::Vec4(val)
    }
}

impl From<[i32; 4]> for UniformValue {
    fn from(val: [i32; 4]) -> Self {
        UniformValue::IVec4(val)
    }
}

impl From<[bool; 4]> for UniformValue {
    fn from(val: [bool; 4]) -> Self {
        UniformValue::BVec4(val)
    }
}

impl From<Color> for UniformValue {
    fn from(val: Color) -> Self {
        UniformValue::Vec4([val.red, val.green, val.blue, val.alpha])
    }
}

impl From<Matrix2> for UniformValue {
    fn from(val: Matrix2) -> Self {
        UniformValue::Mat2([
            val[0] as f32,
            val[1] as f32,
            val[2] as f32,
            val[3] as f32,
        ])
    }
}

impl From<Matrix3> for UniformValue {
    fn from(val: Matrix3) -> Self {
        UniformValue::Mat3([
            val[0] as f32,
            val[1] as f32,
            val[2] as f32,
            val[3] as f32,
            val[4] as f32,
            val[5] as f32,
            val[6] as f32,
            val[7] as f32,
            val[8] as f32,
        ])
    }
}

impl From<Matrix4> for UniformValue {
    fn from(val: Matrix4) -> Self {
        UniformValue::Mat4([
            val[0] as f32,
            val[1] as f32,
            val[2] as f32,
            val[3] as f32,
            val[4] as f32,
            val[5] as f32,
            val[6] as f32,
            val[7] as f32,
            val[8] as f32,
            val[9] as f32,
            val[10] as f32,
            val[11] as f32,
            val[12] as f32,
            val[13] as f32,
            val[14] as f32,
            val[15] as f32,
        ])
    }
}

/// Representation of a single shader uniform variable.
#[derive(Clone, Debug)]
pub struct Uniform {
    pub name: String,
    pub uniform_type: UniformType,
    pub location: i32,
    pub value: Option<UniformValue>,
}

impl Uniform {
    pub fn new(name: impl Into<String>, uniform_type: UniformType, location: i32) -> Self {
        Self {
            name: name.into(),
            uniform_type,
            location,
            value: None,
        }
    }

    /// Sets the value of the uniform after validating type compatibility.
    pub fn set(&mut self, value: UniformValue) -> Result<(), DeveloperError> {
        let is_valid = match (self.uniform_type, &value) {
            (UniformType::Float, UniformValue::Float(_)) => true,
            (UniformType::FloatVec2, UniformValue::Vec2(_)) => true,
            (UniformType::FloatVec3, UniformValue::Vec3(_)) => true,
            (UniformType::FloatVec4, UniformValue::Vec4(_)) => true,
            (UniformType::Int, UniformValue::Int(_)) => true,
            (UniformType::IntVec2, UniformValue::IVec2(_)) => true,
            (UniformType::IntVec3, UniformValue::IVec3(_)) => true,
            (UniformType::IntVec4, UniformValue::IVec4(_)) => true,
            (UniformType::Bool, UniformValue::Bool(_)) => true,
            (UniformType::BoolVec2, UniformValue::BVec2(_)) => true,
            (UniformType::BoolVec3, UniformValue::BVec3(_)) => true,
            (UniformType::BoolVec4, UniformValue::BVec4(_)) => true,
            (UniformType::FloatMat2, UniformValue::Mat2(_)) => true,
            (UniformType::FloatMat3, UniformValue::Mat3(_)) => true,
            (UniformType::FloatMat4, UniformValue::Mat4(_)) => true,
            // Arrays
            (UniformType::Float, UniformValue::FloatArray(_)) => true,
            (UniformType::FloatVec2, UniformValue::Vec2Array(_)) => true,
            (UniformType::FloatVec3, UniformValue::Vec3Array(_)) => true,
            (UniformType::FloatVec4, UniformValue::Vec4Array(_)) => true,
            (UniformType::Int, UniformValue::IntArray(_)) => true,
            (UniformType::IntVec2, UniformValue::IVec2Array(_)) => true,
            (UniformType::IntVec3, UniformValue::IVec3Array(_)) => true,
            (UniformType::IntVec4, UniformValue::IVec4Array(_)) => true,
            (UniformType::Bool, UniformValue::BoolArray(_)) => true,
            (UniformType::BoolVec2, UniformValue::BVec2Array(_)) => true,
            (UniformType::BoolVec3, UniformValue::BVec3Array(_)) => true,
            (UniformType::BoolVec4, UniformValue::BVec4Array(_)) => true,
            (UniformType::FloatMat2, UniformValue::Mat2Array(_)) => true,
            (UniformType::FloatMat3, UniformValue::Mat3Array(_)) => true,
            (UniformType::FloatMat4, UniformValue::Mat4Array(_)) => true,
            _ => false,
        };

        if !is_valid {
            return Err(DeveloperError::new(format!(
                "Invalid uniform value {:?} for uniform type {:?}",
                value, self.uniform_type
            )));
        }

        self.value = Some(value);
        Ok(())
    }
}

pub type UniformGetter = Arc<dyn Fn() -> UniformValue + Send + Sync>;

/// Dynamic uniform dictionary mapping GLSL uniform names to value getters.
#[derive(Default, Clone)]
pub struct UniformMap {
    pub map: BTreeMap<String, UniformGetter>,
}

impl std::fmt::Debug for UniformMap {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("UniformMap")
            .field("keys", &self.map.keys().collect::<Vec<_>>())
            .finish()
    }
}

impl PartialEq for UniformMap {
    fn eq(&self, other: &Self) -> bool {
        self.map.keys().collect::<Vec<_>>() == other.map.keys().collect::<Vec<_>>()
    }
}

impl UniformMap {
    pub fn new() -> Self {
        Self {
            map: BTreeMap::new(),
        }
    }

    /// Sets a constant value for the uniform name.
    pub fn set_value(&mut self, name: impl Into<String>, value: impl Into<UniformValue>) {
        let val = value.into();
        self.map.insert(name.into(), Arc::new(move || val.clone()));
    }

    /// Sets a dynamic callback getter for the uniform name.
    pub fn set_getter<F>(&mut self, name: impl Into<String>, getter: F)
    where
        F: Fn() -> UniformValue + Send + Sync + 'static,
    {
        self.map.insert(name.into(), Arc::new(getter));
    }

    /// Evaluates the uniform value by name.
    pub fn evaluate(&self, name: &str) -> Option<UniformValue> {
        self.map.get(name).map(|f| f())
    }

    /// Number of registered uniforms in the map.
    pub fn len(&self) -> usize {
        self.map.len()
    }

    /// Whether the map is empty.
    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }
}
