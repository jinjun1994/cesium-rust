//! Material matching CesiumJS `Scene/Material.js` (Apache-2.0).

use cesium_math::{Color, DeveloperError};
use std::collections::HashMap;

/// Surface material defining diffuse, specular, normal, emission, and alpha.
#[derive(Clone, Debug, PartialEq)]
pub struct Material {
    pub material_type: String,
    pub shader_source: String,
    pub translucent: bool,
    pub uniforms: HashMap<String, String>,
}

impl Material {
    pub const COLOR_TYPE: &'static str = "Color";

    /// Creates a default Material.
    pub fn new() -> Self {
        Self::from_type(Self::COLOR_TYPE).unwrap()
    }

    /// Creates a material from an existing type identifier.
    pub fn from_type(material_type: &str) -> Result<Self, DeveloperError> {
        match material_type {
            "Color" => Ok(Self {
                material_type: "Color".to_string(),
                shader_source: "czm_material czm_getMaterial(czm_materialInput materialInput) { czm_material material = czm_getDefaultMaterial(materialInput); material.diffuse = color.rgb; material.alpha = color.a; return material; }".to_string(),
                translucent: true,
                uniforms: HashMap::from([("color".to_string(), "vec4(1.0, 0.0, 0.0, 0.5)".to_string())]),
            }),
            _ => Err(DeveloperError::new(format!(
                "material with type '{material_type}' does not exist."
            ))),
        }
    }

    /// Creates a color material from a Cesium Color.
    pub fn from_color(color: Color) -> Self {
        let is_translucent = color.alpha < 1.0;
        Self {
            material_type: "Color".to_string(),
            shader_source: "czm_material czm_getMaterial(czm_materialInput materialInput) { czm_material material = czm_getDefaultMaterial(materialInput); material.diffuse = color.rgb; material.alpha = color.a; return material; }".to_string(),
            translucent: is_translucent,
            uniforms: HashMap::from([("color".to_string(), format!("vec4({}, {}, {}, {})", color.red, color.green, color.blue, color.alpha))]),
        }
    }

    /// Returns whether this material is translucent.
    pub fn is_translucent(&self) -> bool {
        self.translucent
    }
}

impl Default for Material {
    fn default() -> Self {
        Self::new()
    }
}
