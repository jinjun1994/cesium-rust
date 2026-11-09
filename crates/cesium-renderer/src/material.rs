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
    pub is_destroyed: bool,
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

    /// Creates a default Material.
    pub fn new() -> Self {
        Self::from_type(Self::COLOR_TYPE).unwrap()
    }

    /// Creates a material from an existing type identifier with default uniforms.
    pub fn from_type(material_type: &str) -> Result<Self, DeveloperError> {
        Self::from_type_with_uniforms(material_type, HashMap::new())
    }

    /// Creates a material from a type identifier and uniform overrides.
    pub fn from_type_with_uniforms(
        material_type: &str,
        mut custom_uniforms: HashMap<String, String>,
    ) -> Result<Self, DeveloperError> {
        match material_type {
            Self::COLOR_TYPE => {
                let mut uniforms = HashMap::from([(
                    "color".to_string(),
                    "vec4(1.0, 0.0, 0.0, 0.5)".to_string(),
                )]);
                for (k, v) in custom_uniforms {
                    uniforms.insert(k, v);
                }
                Ok(Self {
                    material_type: Self::COLOR_TYPE.to_string(),
                    shader_source: "czm_material czm_getMaterial(czm_materialInput materialInput) { czm_material material = czm_getDefaultMaterial(materialInput); material.diffuse = color.rgb; material.alpha = color.a; return material; }".to_string(),
                    translucent: true,
                    uniforms,
                    is_destroyed: false,
                })
            }
            Self::STRIPE_TYPE => {
                let mut uniforms = HashMap::from([
                    ("horizontal".to_string(), "true".to_string()),
                    ("evenColor".to_string(), "vec4(1.0, 1.0, 1.0, 1.0)".to_string()),
                    ("oddColor".to_string(), "vec4(0.0, 0.0, 0.0, 1.0)".to_string()),
                    ("offset".to_string(), "0.0".to_string()),
                    ("repeat".to_string(), "1.0".to_string()),
                ]);
                for (k, v) in custom_uniforms {
                    uniforms.insert(k, v);
                }
                Ok(Self {
                    material_type: Self::STRIPE_TYPE.to_string(),
                    shader_source: "czm_material czm_getMaterial(czm_materialInput materialInput) { czm_material material = czm_getDefaultMaterial(materialInput); return material; }".to_string(),
                    translucent: false,
                    uniforms,
                    is_destroyed: false,
                })
            }
            Self::IMAGE_TYPE => {
                let mut uniforms = HashMap::from([
                    ("image".to_string(), Self::DEFAULT_IMAGE_ID.to_string()),
                    ("repeat".to_string(), "vec2(1.0, 1.0)".to_string()),
                    ("color".to_string(), "vec4(1.0, 1.0, 1.0, 1.0)".to_string()),
                ]);
                for (k, v) in custom_uniforms {
                    uniforms.insert(k, v);
                }
                Ok(Self {
                    material_type: Self::IMAGE_TYPE.to_string(),
                    shader_source: "czm_material czm_getMaterial(czm_materialInput materialInput) { czm_material material = czm_getDefaultMaterial(materialInput); material.diffuse = texture(image, fract(repeat * materialInput.st)).rgb * color.rgb; material.alpha = texture(image, fract(repeat * materialInput.st)).a * color.a; return material; }".to_string(),
                    translucent: false,
                    uniforms,
                    is_destroyed: false,
                })
            }
            Self::DOT_TYPE => {
                let mut uniforms = HashMap::from([
                    ("lightColor".to_string(), "vec4(1.0, 1.0, 0.0, 0.75)".to_string()),
                    ("darkColor".to_string(), "vec4(0.0, 1.0, 1.0, 0.75)".to_string()),
                    ("repeat".to_string(), "vec2(5.0, 5.0)".to_string()),
                ]);
                for (k, v) in custom_uniforms {
                    uniforms.insert(k, v);
                }
                Ok(Self {
                    material_type: Self::DOT_TYPE.to_string(),
                    shader_source: "czm_material czm_getMaterial(czm_materialInput materialInput) { czm_material material = czm_getDefaultMaterial(materialInput); return material; }".to_string(),
                    translucent: true,
                    uniforms,
                    is_destroyed: false,
                })
            }
            Self::NORMAL_MAP_TYPE => {
                let mut uniforms = HashMap::from([
                    ("image".to_string(), Self::DEFAULT_IMAGE_ID.to_string()),
                    ("channels".to_string(), "\"rgb\"".to_string()),
                    ("strength".to_string(), "0.8".to_string()),
                    ("repeat".to_string(), "vec2(1.0, 1.0)".to_string()),
                ]);
                for (k, v) in custom_uniforms {
                    uniforms.insert(k, v);
                }
                Ok(Self {
                    material_type: Self::NORMAL_MAP_TYPE.to_string(),
                    shader_source: "czm_material czm_getMaterial(czm_materialInput materialInput) { czm_material material = czm_getDefaultMaterial(materialInput); return material; }".to_string(),
                    translucent: false,
                    uniforms,
                    is_destroyed: false,
                })
            }
            Self::CHECKERBOARD_TYPE => {
                let mut uniforms = HashMap::from([
                    ("lightColor".to_string(), "vec4(1.0, 1.0, 1.0, 1.0)".to_string()),
                    ("darkColor".to_string(), "vec4(0.0, 0.0, 0.0, 1.0)".to_string()),
                    ("repeat".to_string(), "vec2(2.0, 2.0)".to_string()),
                ]);
                for (k, v) in custom_uniforms {
                    uniforms.insert(k, v);
                }
                Ok(Self {
                    material_type: Self::CHECKERBOARD_TYPE.to_string(),
                    shader_source: "czm_material czm_getMaterial(czm_materialInput materialInput) { czm_material material = czm_getDefaultMaterial(materialInput); return material; }".to_string(),
                    translucent: false,
                    uniforms,
                    is_destroyed: false,
                })
            }
            Self::GRID_TYPE => {
                let mut uniforms = HashMap::from([
                    ("color".to_string(), "vec4(0.0, 1.0, 0.0, 1.0)".to_string()),
                    ("cellAlpha".to_string(), "0.1".to_string()),
                    ("lineCount".to_string(), "vec2(8.0, 8.0)".to_string()),
                    ("lineThickness".to_string(), "vec2(1.0, 1.0)".to_string()),
                    ("lineOffset".to_string(), "vec2(0.0, 0.0)".to_string()),
                ]);
                for (k, v) in custom_uniforms {
                    uniforms.insert(k, v);
                }
                Ok(Self {
                    material_type: Self::GRID_TYPE.to_string(),
                    shader_source: "czm_material czm_getMaterial(czm_materialInput materialInput) { czm_material material = czm_getDefaultMaterial(materialInput); return material; }".to_string(),
                    translucent: true,
                    uniforms,
                    is_destroyed: false,
                })
            }
            Self::WATER_TYPE
            | Self::DIFFUSE_MAP_TYPE
            | Self::ALPHA_MAP_TYPE
            | Self::SPECULAR_MAP_TYPE
            | Self::EMISSION_MAP_TYPE
            | Self::BUMP_MAP_TYPE
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
                let mut uniforms = HashMap::new();
                for (k, v) in custom_uniforms.drain() {
                    uniforms.insert(k, v);
                }
                Ok(Self {
                    material_type: material_type.to_string(),
                    shader_source: "czm_material czm_getMaterial(czm_materialInput materialInput) { czm_material material = czm_getDefaultMaterial(materialInput); return material; }".to_string(),
                    translucent: false,
                    uniforms,
                    is_destroyed: false,
                })
            }
            _ => Err(DeveloperError::new(format!(
                "material with type '{material_type}' does not exist."
            ))),
        }
    }

    /// Creates a color material from a Cesium Color.
    pub fn from_color(color: Color) -> Self {
        let is_translucent = color.alpha < 1.0;
        Self {
            material_type: Self::COLOR_TYPE.to_string(),
            shader_source: "czm_material czm_getMaterial(czm_materialInput materialInput) { czm_material material = czm_getDefaultMaterial(materialInput); material.diffuse = color.rgb; material.alpha = color.a; return material; }".to_string(),
            translucent: is_translucent,
            uniforms: HashMap::from([("color".to_string(), format!("vec4({}, {}, {}, {})", color.red, color.green, color.blue, color.alpha))]),
            is_destroyed: false,
        }
    }

    /// Returns whether this material is translucent.
    pub fn is_translucent(&self) -> bool {
        self.translucent
    }

    /// Destroys this material and releases resources.
    pub fn destroy(&mut self) {
        self.is_destroyed = true;
        self.uniforms.clear();
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
