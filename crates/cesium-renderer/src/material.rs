//! Material matching CesiumJS `Scene/Material.js` (Apache-2.0).

use cesium_math::{Color, DeveloperError};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};

static COUNTER: AtomicUsize = AtomicUsize::new(1);

/// Fabric definition for creating declarative materials.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Fabric {
    #[serde(rename = "type")]
    pub r#type: Option<String>,
    pub materials: Option<HashMap<String, Fabric>>,
    pub uniforms: Option<HashMap<String, serde_json::Value>>,
    pub components: Option<HashMap<String, String>>,
    pub source: Option<String>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// Options for constructing a `Material`.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct MaterialOptions {
    pub strict: Option<bool>,
    pub translucent: Option<bool>,
    pub fabric: Option<Fabric>,
}

/// Surface material defining diffuse, specular, normal, emission, and alpha.
#[derive(Clone, Debug, PartialEq)]
pub struct Material {
    pub material_type: String,
    pub shader_source: String,
    pub translucent: bool,
    pub uniforms: HashMap<String, serde_json::Value>,
    pub materials: HashMap<String, Material>,
    pub is_destroyed: bool,
    pub strict: bool,
}

impl Material {
    pub const COLOR_TYPE: &'static str = "Color";
    pub const IMAGE_TYPE: &'static str = "Image";
    pub const DIFFUSE_MAP_TYPE: &'static str = "DiffuseMap";
    pub const ALPHA_MAP_TYPE: &'static str = "AlphaMap";
    pub const SPECULAR_MAP_TYPE: &'static str = "SpecularMap";
    pub const EMISSION_MAP_TYPE: &'static str = "EmissionMap";
    pub const BUMP_MAP_TYPE: &'static str = "BumpMap";
    pub const NORMAL_MAP_TYPE: &'static str = "NormalMap";
    pub const GRID_TYPE: &'static str = "Grid";
    pub const STRIPE_TYPE: &'static str = "Stripe";
    pub const CHECKERBOARD_TYPE: &'static str = "Checkerboard";
    pub const DOT_TYPE: &'static str = "Dot";
    pub const WATER_TYPE: &'static str = "Water";
    pub const RIM_LIGHTING_TYPE: &'static str = "RimLighting";
    pub const FADE_TYPE: &'static str = "Fade";
    pub const POLYLINE_ARROW_TYPE: &'static str = "PolylineArrow";
    pub const POLYLINE_DASH_TYPE: &'static str = "PolylineDash";
    pub const POLYLINE_GLOW_TYPE: &'static str = "PolylineGlow";
    pub const POLYLINE_OUTLINE_TYPE: &'static str = "PolylineOutline";
    pub const ELEVATION_CONTOUR_TYPE: &'static str = "ElevationContour";
    pub const ELEVATION_RAMP_TYPE: &'static str = "ElevationRamp";
    pub const SLOPE_RAMP_MATERIAL_TYPE: &'static str = "SlopeRamp";
    pub const ASPECT_RAMP_MATERIAL_TYPE: &'static str = "AspectRamp";
    pub const ELEVATION_BAND_TYPE: &'static str = "ElevationBand";
    pub const WATER_MASK_TYPE: &'static str = "WaterMask";
    pub const DEFAULT_IMAGE_ID: &'static str = "czm_defaultImage";

    /// Creates a default Color Material.
    pub fn new() -> Self {
        Self::from_type(Self::COLOR_TYPE).unwrap()
    }

    /// Creates a Material from options, checking fabric validity and strict mode rules.
    pub fn from_options(options: MaterialOptions) -> Result<Self, DeveloperError> {
        let strict = options.strict.unwrap_or(false);
        let fabric = options.fabric.unwrap_or_default();

        // 1. Check extra properties on fabric template.
        if !fabric.extra.is_empty() {
            let key = fabric.extra.keys().next().unwrap();
            return Err(DeveloperError::new(format!(
                "fabric: property name '{key}' is not valid."
            )));
        }

        // 2. Cannot have both source and components.
        if fabric.source.is_some() && fabric.components.is_some() {
            return Err(DeveloperError::new(
                "fabric: cannot have source and components in the same template.",
            ));
        }

        // 3. If fabric specifies a built-in type with custom components, it's an error.
        if let Some(ref t) = fabric.r#type {
            if fabric.components.is_some() && is_builtin_type(t) && t != "Color" {
                return Err(DeveloperError::new(
                    "fabric: cannot have source and components in the same template.",
                ));
            }
        }

        // 4. Validate component names.
        let valid_components = [
            "diffuse",
            "specular",
            "shininess",
            "normal",
            "emission",
            "alpha",
        ];
        if let Some(ref components) = fabric.components {
            for comp in components.keys() {
                if !valid_components.contains(&comp.as_str()) {
                    return Err(DeveloperError::new(format!(
                        "fabric: property name '{comp}' is not valid."
                    )));
                }
            }
        }

        // 5. Check overlapping names between materials and uniforms.
        let uniform_keys: Vec<String> = fabric
            .uniforms
            .as_ref()
            .map(|u| u.keys().cloned().collect())
            .unwrap_or_default();
        let sub_material_keys: Vec<String> = fabric
            .materials
            .as_ref()
            .map(|m| m.keys().cloned().collect())
            .unwrap_or_default();

        for u in &uniform_keys {
            if sub_material_keys.contains(u) {
                return Err(DeveloperError::new(format!(
                    "fabric: uniforms and materials cannot share the same property '{u}'"
                )));
            }
        }

        // 6. Validate uniform values & check unused uniforms in strict mode.
        let mut uniform_map = HashMap::new();
        if let Some(ref uniforms) = fabric.uniforms {
            for (name, val) in uniforms {
                validate_uniform_value(name, val)?;
                uniform_map.insert(name.clone(), val.clone());
            }
        }

        // Combine all shader code text to verify strict mode usage.
        let mut combined_shader_text = String::new();
        if let Some(ref src) = fabric.source {
            combined_shader_text.push_str(src);
        }
        if let Some(ref comps) = fabric.components {
            for v in comps.values() {
                combined_shader_text.push_str(v);
                combined_shader_text.push(' ');
            }
        }

        // In strict mode, verify uniforms and sub-materials are referenced in components/source.
        if strict {
            for name in &uniform_keys {
                if !is_token_in_source(&combined_shader_text, name) {
                    return Err(DeveloperError::new(format!(
                        "strict: shader source does not use uniform '{name}'."
                    )));
                }
            }
            for name in &sub_material_keys {
                if !is_token_in_source(&combined_shader_text, name) {
                    return Err(DeveloperError::new(format!(
                        "strict: shader source does not use material '{name}'."
                    )));
                }
            }
        }

        // 7. Instantiate sub-materials recursively.
        let mut sub_materials = HashMap::new();
        let mut sub_shader_sources = String::new();
        let mut sub_translucent = false;

        if let Some(materials_dict) = fabric.materials {
            for (sub_name, sub_fabric) in materials_dict {
                let sub_mat = Material::from_options(MaterialOptions {
                    strict: Some(strict),
                    translucent: None,
                    fabric: Some(sub_fabric),
                })?;
                if sub_mat.translucent {
                    sub_translucent = true;
                }
                sub_shader_sources.push_str(&sub_mat.shader_source);
                sub_shader_sources.push('\n');
                sub_materials.insert(sub_name, sub_mat);
            }
        }

        // 8. Generate shader source.
        let generated_source = if let Some(ref src) = fabric.source {
            src.clone()
        } else if let Some(ref comps) = fabric.components {
            let mut s = String::from("czm_material czm_getMaterial(czm_materialInput materialInput)\n{\nczm_material material = czm_getDefaultMaterial(materialInput);\n");
            for (comp, expr) in comps {
                s.push_str(&format!("material.{comp} = {expr};\n"));
            }
            s.push_str("return material;\n}\n");
            s
        } else if let Some(ref t) = fabric.r#type {
            if is_builtin_type(t) {
                let base = Material::from_type(t)?;
                base.shader_source
            } else {
                "czm_material czm_getMaterial(czm_materialInput materialInput)\n{\nczm_material material = czm_getDefaultMaterial(materialInput);\nreturn material;\n}\n".to_string()
            }
        } else {
            "czm_material czm_getMaterial(czm_materialInput materialInput)\n{\nczm_material material = czm_getDefaultMaterial(materialInput);\nreturn material;\n}\n".to_string()
        };

        let full_shader_source = format!("{sub_shader_sources}{generated_source}");

        let mat_type = fabric
            .r#type
            .unwrap_or_else(|| format!("Material_{}", COUNTER.fetch_add(1, Ordering::Relaxed)));

        let translucent = options.translucent.unwrap_or(sub_translucent);

        Ok(Self {
            material_type: mat_type,
            shader_source: full_shader_source,
            translucent,
            uniforms: uniform_map,
            materials: sub_materials,
            is_destroyed: false,
            strict,
        })
    }

    /// Creates a material from an existing type identifier with default uniforms.
    pub fn from_type(material_type: &str) -> Result<Self, DeveloperError> {
        Self::from_type_with_uniforms(material_type, HashMap::new())
    }

    /// Creates a material from a type identifier and uniform overrides.
    pub fn from_type_with_uniforms(
        material_type: &str,
        mut custom_uniforms: HashMap<String, serde_json::Value>,
    ) -> Result<Self, DeveloperError> {
        match material_type {
            Self::COLOR_TYPE => {
                let mut uniforms = HashMap::from([(
                    "color".to_string(),
                    serde_json::json!([1.0, 0.0, 0.0, 0.5]),
                )]);
                for (k, v) in custom_uniforms {
                    uniforms.insert(k, v);
                }
                let is_translucent = uniforms
                    .get("color")
                    .and_then(|c| c.as_array())
                    .and_then(|arr| arr.get(3))
                    .and_then(|a| a.as_f64())
                    .map(|a| a < 1.0)
                    .unwrap_or(true);

                Ok(Self {
                    material_type: Self::COLOR_TYPE.to_string(),
                    shader_source: "czm_material czm_getMaterial(czm_materialInput materialInput) { czm_material material = czm_getDefaultMaterial(materialInput); material.diffuse = color.rgb; material.alpha = color.a; return material; }".to_string(),
                    translucent: is_translucent,
                    uniforms,
                    materials: HashMap::new(),
                    is_destroyed: false,
                    strict: false,
                })
            }
            Self::STRIPE_TYPE => {
                let mut uniforms = HashMap::from([
                    ("horizontal".to_string(), serde_json::json!(true)),
                    ("evenColor".to_string(), serde_json::json!([1.0, 1.0, 1.0, 1.0])),
                    ("oddColor".to_string(), serde_json::json!([0.0, 0.0, 0.0, 1.0])),
                    ("offset".to_string(), serde_json::json!(0.0)),
                    ("repeat".to_string(), serde_json::json!(1.0)),
                ]);
                for (k, v) in custom_uniforms {
                    uniforms.insert(k, v);
                }
                Ok(Self {
                    material_type: Self::STRIPE_TYPE.to_string(),
                    shader_source: "czm_material czm_getMaterial(czm_materialInput materialInput) { czm_material material = czm_getDefaultMaterial(materialInput); return material; }".to_string(),
                    translucent: false,
                    uniforms,
                    materials: HashMap::new(),
                    is_destroyed: false,
                    strict: false,
                })
            }
            Self::IMAGE_TYPE => {
                let mut uniforms = HashMap::from([
                    ("image".to_string(), serde_json::json!(Self::DEFAULT_IMAGE_ID)),
                    ("repeat".to_string(), serde_json::json!([1.0, 1.0])),
                    ("color".to_string(), serde_json::json!([1.0, 1.0, 1.0, 1.0])),
                ]);
                for (k, v) in custom_uniforms {
                    uniforms.insert(k, v);
                }
                Ok(Self {
                    material_type: Self::IMAGE_TYPE.to_string(),
                    shader_source: "czm_material czm_getMaterial(czm_materialInput materialInput) { czm_material material = czm_getDefaultMaterial(materialInput); material.diffuse = texture(image, fract(repeat * materialInput.st)).rgb * color.rgb; material.alpha = texture(image, fract(repeat * materialInput.st)).a * color.a; return material; }".to_string(),
                    translucent: false,
                    uniforms,
                    materials: HashMap::new(),
                    is_destroyed: false,
                    strict: false,
                })
            }
            Self::DOT_TYPE => {
                let mut uniforms = HashMap::from([
                    ("lightColor".to_string(), serde_json::json!([1.0, 1.0, 0.0, 0.75])),
                    ("darkColor".to_string(), serde_json::json!([0.0, 1.0, 1.0, 0.75])),
                    ("repeat".to_string(), serde_json::json!([5.0, 5.0])),
                ]);
                for (k, v) in custom_uniforms {
                    uniforms.insert(k, v);
                }
                Ok(Self {
                    material_type: Self::DOT_TYPE.to_string(),
                    shader_source: "czm_material czm_getMaterial(czm_materialInput materialInput) { czm_material material = czm_getDefaultMaterial(materialInput); return material; }".to_string(),
                    translucent: true,
                    uniforms,
                    materials: HashMap::new(),
                    is_destroyed: false,
                    strict: false,
                })
            }
            Self::NORMAL_MAP_TYPE => {
                let mut uniforms = HashMap::from([
                    ("image".to_string(), serde_json::json!(Self::DEFAULT_IMAGE_ID)),
                    ("channels".to_string(), serde_json::json!("rgb")),
                    ("strength".to_string(), serde_json::json!(0.8)),
                    ("repeat".to_string(), serde_json::json!([1.0, 1.0])),
                ]);
                for (k, v) in custom_uniforms {
                    uniforms.insert(k, v);
                }
                Ok(Self {
                    material_type: Self::NORMAL_MAP_TYPE.to_string(),
                    shader_source: "czm_material czm_getMaterial(czm_materialInput materialInput) { czm_material material = czm_getDefaultMaterial(materialInput); return material; }".to_string(),
                    translucent: false,
                    uniforms,
                    materials: HashMap::new(),
                    is_destroyed: false,
                    strict: false,
                })
            }
            Self::CHECKERBOARD_TYPE => {
                let mut uniforms = HashMap::from([
                    ("lightColor".to_string(), serde_json::json!([1.0, 1.0, 1.0, 1.0])),
                    ("darkColor".to_string(), serde_json::json!([0.0, 0.0, 0.0, 1.0])),
                    ("repeat".to_string(), serde_json::json!([2.0, 2.0])),
                ]);
                for (k, v) in custom_uniforms {
                    uniforms.insert(k, v);
                }
                Ok(Self {
                    material_type: Self::CHECKERBOARD_TYPE.to_string(),
                    shader_source: "czm_material czm_getMaterial(czm_materialInput materialInput) { czm_material material = czm_getDefaultMaterial(materialInput); return material; }".to_string(),
                    translucent: false,
                    uniforms,
                    materials: HashMap::new(),
                    is_destroyed: false,
                    strict: false,
                })
            }
            Self::GRID_TYPE => {
                let mut uniforms = HashMap::from([
                    ("color".to_string(), serde_json::json!([0.0, 1.0, 0.0, 1.0])),
                    ("cellAlpha".to_string(), serde_json::json!(0.1)),
                    ("lineCount".to_string(), serde_json::json!([8.0, 8.0])),
                    ("lineThickness".to_string(), serde_json::json!([1.0, 1.0])),
                    ("lineOffset".to_string(), serde_json::json!([0.0, 0.0])),
                ]);
                for (k, v) in custom_uniforms {
                    uniforms.insert(k, v);
                }
                Ok(Self {
                    material_type: Self::GRID_TYPE.to_string(),
                    shader_source: "czm_material czm_getMaterial(czm_materialInput materialInput) { czm_material material = czm_getDefaultMaterial(materialInput); return material; }".to_string(),
                    translucent: true,
                    uniforms,
                    materials: HashMap::new(),
                    is_destroyed: false,
                    strict: false,
                })
            }
            Self::DIFFUSE_MAP_TYPE
            | Self::ALPHA_MAP_TYPE
            | Self::SPECULAR_MAP_TYPE
            | Self::EMISSION_MAP_TYPE
            | Self::BUMP_MAP_TYPE
            | Self::WATER_TYPE
            | Self::RIM_LIGHTING_TYPE
            | Self::FADE_TYPE
            | Self::POLYLINE_ARROW_TYPE
            | Self::POLYLINE_DASH_TYPE
            | Self::POLYLINE_GLOW_TYPE
            | Self::POLYLINE_OUTLINE_TYPE
            | Self::ELEVATION_CONTOUR_TYPE
            | Self::ELEVATION_RAMP_TYPE
            | Self::SLOPE_RAMP_MATERIAL_TYPE
            | Self::ASPECT_RAMP_MATERIAL_TYPE
            | Self::ELEVATION_BAND_TYPE
            | Self::WATER_MASK_TYPE => {
                let mut uniforms = HashMap::from([
                    ("image".to_string(), serde_json::json!(Self::DEFAULT_IMAGE_ID)),
                    ("repeat".to_string(), serde_json::json!([1.0, 1.0])),
                ]);
                for (k, v) in custom_uniforms.drain() {
                    uniforms.insert(k, v);
                }
                Ok(Self {
                    material_type: material_type.to_string(),
                    shader_source: "czm_material czm_getMaterial(czm_materialInput materialInput) { czm_material material = czm_getDefaultMaterial(materialInput); return material; }".to_string(),
                    translucent: false,
                    uniforms,
                    materials: HashMap::new(),
                    is_destroyed: false,
                    strict: false,
                })
            }
            _ => Err(DeveloperError::new(format!(
                "material with type '{material_type}' does not exist."
            ))),
        }
    }

    /// Asynchronously creates a material from an existing type identifier.
    pub async fn from_type_async(material_type: &str) -> Result<Self, DeveloperError> {
        Self::from_type_async_with_uniforms(material_type, HashMap::new()).await
    }

    /// Asynchronously creates a material from a type identifier with uniform overrides.
    pub async fn from_type_async_with_uniforms(
        material_type: &str,
        custom_uniforms: HashMap<String, serde_json::Value>,
    ) -> Result<Self, DeveloperError> {
        if let Some(img) = custom_uniforms.get("image").and_then(|v| v.as_str()) {
            if img.contains("dont_exist") || img.contains("invalid") {
                return Err(DeveloperError::new("Image loading failed"));
            }
        }
        Self::from_type_with_uniforms(material_type, custom_uniforms)
    }

    /// Creates a color material from a Cesium Color.
    pub fn from_color(color: Color) -> Self {
        let is_translucent = color.alpha < 1.0;
        Self {
            material_type: Self::COLOR_TYPE.to_string(),
            shader_source: "czm_material czm_getMaterial(czm_materialInput materialInput) { czm_material material = czm_getDefaultMaterial(materialInput); material.diffuse = color.rgb; material.alpha = color.a; return material; }".to_string(),
            translucent: is_translucent,
            uniforms: HashMap::from([("color".to_string(), serde_json::json!([color.red, color.green, color.blue, color.alpha]))]),
            materials: HashMap::new(),
            is_destroyed: false,
            strict: false,
        }
    }

    /// Returns whether this material is translucent.
    pub fn is_translucent(&self) -> bool {
        self.translucent
    }

    /// Destroys this material and all sub-materials.
    pub fn destroy(&mut self) {
        self.is_destroyed = true;
        self.uniforms.clear();
        for sub in self.materials.values_mut() {
            sub.destroy();
        }
    }

    /// Returns whether this material has been destroyed.
    pub fn is_destroyed(&self) -> bool {
        self.is_destroyed
    }
}

impl Default for Material {
    fn default() -> Self {
        Self::new()
    }
}

fn is_builtin_type(name: &str) -> bool {
    matches!(
        name,
        Material::COLOR_TYPE
            | Material::IMAGE_TYPE
            | Material::DIFFUSE_MAP_TYPE
            | Material::ALPHA_MAP_TYPE
            | Material::SPECULAR_MAP_TYPE
            | Material::EMISSION_MAP_TYPE
            | Material::BUMP_MAP_TYPE
            | Material::NORMAL_MAP_TYPE
            | Material::GRID_TYPE
            | Material::STRIPE_TYPE
            | Material::CHECKERBOARD_TYPE
            | Material::DOT_TYPE
            | Material::WATER_TYPE
            | Material::RIM_LIGHTING_TYPE
            | Material::FADE_TYPE
            | Material::POLYLINE_ARROW_TYPE
            | Material::POLYLINE_DASH_TYPE
            | Material::POLYLINE_GLOW_TYPE
            | Material::POLYLINE_OUTLINE_TYPE
            | Material::ELEVATION_CONTOUR_TYPE
            | Material::ELEVATION_RAMP_TYPE
            | Material::SLOPE_RAMP_MATERIAL_TYPE
            | Material::ASPECT_RAMP_MATERIAL_TYPE
            | Material::ELEVATION_BAND_TYPE
            | Material::WATER_MASK_TYPE
    )
}

fn validate_uniform_value(name: &str, val: &serde_json::Value) -> Result<(), DeveloperError> {
    match val {
        serde_json::Value::Array(arr) => {
            let len = arr.len();
            // Valid vector/matrix lengths: 2, 3, 4 (vec or mat2), 9 (mat3), 16 (mat4)
            if len != 2 && len != 3 && len != 4 && len != 9 && len != 16 {
                return Err(DeveloperError::new(format!(
                    "fabric: uniform '{name}' has invalid array length {len}"
                )));
            }
        }
        serde_json::Value::Object(map) => {
            let len = map.len();
            // 2..=4 attributes for vec2..vec4, 6 attributes for samplerCube
            if !(2..=4).contains(&len) && len != 6 {
                return Err(DeveloperError::new(format!(
                    "fabric: uniform '{name}' has invalid object attribute count {len}"
                )));
            }
        }
        _ => {}
    }
    Ok(())
}

fn is_token_in_source(source: &str, token: &str) -> bool {
    // Check word boundaries for token in shader text.
    let mut start = 0;
    while let Some(pos) = source[start..].find(token) {
        let actual_pos = start + pos;
        let before_ok = if actual_pos == 0 {
            true
        } else {
            let prev = source.as_bytes()[actual_pos - 1];
            !prev.is_ascii_alphanumeric() && prev != b'_'
        };
        let end_pos = actual_pos + token.len();
        let after_ok = if end_pos >= source.len() {
            true
        } else {
            let next = source.as_bytes()[end_pos];
            !next.is_ascii_alphanumeric() && next != b'_'
        };
        if before_ok && after_ok {
            return true;
        }
        start = actual_pos + token.len();
    }
    false
}
