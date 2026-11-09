//! ModelComponents matching CesiumJS `Scene/ModelComponents.js` (Apache-2.0).

use crate::axis::Axis;
use crate::buffer::Buffer;
pub use crate::gltf_loader_util::ModelTextureReader;
use cesium_math::{
    AttributeType, Cartesian3, Cartesian4, ComponentDatatype, IndexDatatype, Matrix4,
    PrimitiveType, Quaternion,
};
use serde_json::Value;
use std::collections::HashMap;

/// The alpha rendering mode of the material.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AlphaMode {
    #[default]
    Opaque,
    Mask,
    Blend,
}

impl AlphaMode {
    pub fn from_str_name(s: &str) -> Option<Self> {
        match s {
            "OPAQUE" => Some(Self::Opaque),
            "MASK" => Some(Self::Mask),
            "BLEND" => Some(Self::Blend),
            _ => None,
        }
    }
}

/// Information about a quantized attribute.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Quantization {
    pub oct_encoded: bool,
    pub oct_encoded_zxy: bool,
    pub normalization_range: Option<Vec<f64>>,
    pub quantized_volume_offset: Option<Vec<f64>>,
    pub quantized_volume_dimensions: Option<Vec<f64>>,
    pub quantized_volume_step_size: Option<Vec<f64>>,
    pub component_datatype: Option<ComponentDatatype>,
    pub attribute_type: Option<AttributeType>,
}

/// A per-vertex or per-instance attribute.
#[derive(Debug, Clone, Default)]
pub struct Attribute {
    pub name: Option<String>,
    pub semantic: Option<String>,
    pub set_index: Option<usize>,
    pub component_datatype: Option<ComponentDatatype>,
    pub attribute_type: Option<AttributeType>,
    pub normalized: bool,
    pub count: usize,
    pub min: Option<Vec<f64>>,
    pub max: Option<Vec<f64>>,
    pub constant: Option<Vec<f64>>,
    pub quantization: Option<Quantization>,
    pub typed_array: Option<Vec<u8>>,
    pub buffer: Option<Buffer>,
    pub byte_offset: usize,
    pub byte_stride: Option<usize>,
}

/// Indices used to select vertices for rendering.
#[derive(Debug, Clone, Default)]
pub struct Indices {
    pub index_datatype: Option<IndexDatatype>,
    pub count: usize,
    pub buffer: Option<Buffer>,
    pub typed_array: Option<Vec<u8>>,
}

/// Feature ID attribute mapping.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct FeatureIdAttribute {
    pub feature_count: usize,
    pub null_feature_id: Option<i64>,
    pub property_table_id: Option<usize>,
    pub set_index: Option<usize>,
    pub label: Option<String>,
    pub positional_label: Option<String>,
}

/// Feature ID texture mapping.
#[derive(Debug, Clone, Default)]
pub struct FeatureIdTexture {
    pub feature_count: usize,
    pub null_feature_id: Option<i64>,
    pub property_table_id: Option<usize>,
    pub texture_reader: Option<ModelTextureReader>,
    pub label: Option<String>,
    pub positional_label: Option<String>,
}

/// Implicit range of feature IDs.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct FeatureIdImplicitRange {
    pub feature_count: usize,
    pub null_feature_id: Option<i64>,
    pub property_table_id: Option<usize>,
    pub offset: usize,
    pub repeat: Option<usize>,
    pub label: Option<String>,
    pub positional_label: Option<String>,
}

/// Feature ID container variant.
#[derive(Debug, Clone)]
pub enum FeatureId {
    Attribute(FeatureIdAttribute),
    Texture(FeatureIdTexture),
    ImplicitRange(FeatureIdImplicitRange),
}

/// Morph target containing attribute displacement data.
#[derive(Debug, Clone, Default)]
pub struct MorphTarget {
    pub attributes: Vec<Attribute>,
}

/// Geometry to be rendered with a material.
#[derive(Debug, Clone, Default)]
pub struct Primitive {
    pub attributes: Vec<Attribute>,
    pub morph_targets: Vec<MorphTarget>,
    pub indices: Option<Indices>,
    pub material: Option<Material>,
    pub primitive_type: Option<PrimitiveType>,
    pub feature_ids: Vec<FeatureId>,
    pub property_texture_ids: Vec<usize>,
    pub property_attribute_ids: Vec<usize>,
    pub outline_coordinates: Option<Attribute>,
}

/// Position and metadata information for instances of a node.
#[derive(Debug, Clone, Default)]
pub struct Instances {
    pub attributes: Vec<Attribute>,
    pub feature_ids: Vec<FeatureId>,
    pub transform_in_world_space: bool,
}

/// Joints and matrices defining a skin.
#[derive(Debug, Clone, Default)]
pub struct Skin {
    pub index: Option<usize>,
    pub joints: Vec<usize>,
    pub inverse_bind_matrices: Vec<Matrix4>,
}

/// A node in the node hierarchy.
#[derive(Debug, Clone, Default)]
pub struct Node {
    pub name: Option<String>,
    pub index: Option<usize>,
    pub children: Vec<usize>,
    pub primitives: Vec<Primitive>,
    pub instances: Option<Instances>,
    pub skin: Option<usize>,
    pub matrix: Option<Matrix4>,
    pub translation: Option<Cartesian3>,
    pub rotation: Option<Quaternion>,
    pub scale: Option<Cartesian3>,
    pub morph_weights: Vec<f64>,
    pub articulation_name: Option<String>,
}

/// A scene containing root node indices.
#[derive(Debug, Clone, Default)]
pub struct Scene {
    pub nodes: Vec<usize>,
}

/// Material properties for PBR metallic roughness.
#[derive(Debug, Clone)]
pub struct MetallicRoughness {
    pub base_color_texture: Option<ModelTextureReader>,
    pub metallic_roughness_texture: Option<ModelTextureReader>,
    pub base_color_factor: Cartesian4,
    pub metallic_factor: f32,
    pub roughness_factor: f32,
}

impl Default for MetallicRoughness {
    fn default() -> Self {
        Self {
            base_color_texture: None,
            metallic_roughness_texture: None,
            base_color_factor: Cartesian4::new(1.0, 1.0, 1.0, 1.0),
            metallic_factor: 1.0,
            roughness_factor: 1.0,
        }
    }
}

/// Material properties for PBR specular glossiness.
#[derive(Debug, Clone)]
pub struct SpecularGlossiness {
    pub diffuse_texture: Option<ModelTextureReader>,
    pub specular_glossiness_texture: Option<ModelTextureReader>,
    pub diffuse_factor: Cartesian4,
    pub specular_factor: Cartesian3,
    pub glossiness_factor: f32,
}

impl Default for SpecularGlossiness {
    fn default() -> Self {
        Self {
            diffuse_texture: None,
            specular_glossiness_texture: None,
            diffuse_factor: Cartesian4::new(1.0, 1.0, 1.0, 1.0),
            specular_factor: Cartesian3::new(1.0, 1.0, 1.0),
            glossiness_factor: 1.0,
        }
    }
}

/// Material appearance of a primitive.
#[derive(Debug, Clone)]
pub struct Material {
    pub metallic_roughness: Option<MetallicRoughness>,
    pub specular_glossiness: Option<SpecularGlossiness>,
    pub emissive_texture: Option<ModelTextureReader>,
    pub normal_texture: Option<ModelTextureReader>,
    pub occlusion_texture: Option<ModelTextureReader>,
    pub emissive_factor: Cartesian3,
    pub alpha_mode: AlphaMode,
    pub alpha_cutoff: f32,
    pub double_sided: bool,
    pub unlit: bool,
    pub point_diameter: Option<f32>,
}

impl Default for Material {
    fn default() -> Self {
        Self {
            metallic_roughness: Some(MetallicRoughness::default()),
            specular_glossiness: None,
            emissive_texture: None,
            normal_texture: None,
            occlusion_texture: None,
            emissive_factor: Cartesian3::ZERO,
            alpha_mode: AlphaMode::Opaque,
            alpha_cutoff: 0.5,
            double_sided: false,
            unlit: false,
            point_diameter: None,
        }
    }
}

/// Asset information in glTF.
#[derive(Debug, Clone)]
pub struct Asset {
    pub version: String,
    pub generator: Option<String>,
    pub copyright: Option<String>,
    pub min_version: Option<String>,
    pub credits: Vec<String>,
}

impl Default for Asset {
    fn default() -> Self {
        Self {
            version: "2.0".to_string(),
            generator: None,
            copyright: None,
            min_version: None,
            credits: Vec::new(),
        }
    }
}

/// The complete components that make up a model.
#[derive(Debug, Clone)]
pub struct Components {
    pub asset: Asset,
    pub scene: Option<Scene>,
    pub nodes: Vec<Node>,
    pub skins: Vec<Skin>,
    pub animations: Vec<Value>,
    pub articulations: Vec<Value>,
    pub structural_metadata: Option<Value>,
    pub up_axis: Option<Axis>,
    pub forward_axis: Option<Axis>,
    pub transform: Matrix4,
    pub extensions: HashMap<String, Value>,
}

impl Default for Components {
    fn default() -> Self {
        Self {
            asset: Asset::default(),
            scene: None,
            nodes: Vec::new(),
            skins: Vec::new(),
            animations: Vec::new(),
            articulations: Vec::new(),
            structural_metadata: None,
            up_axis: None,
            forward_axis: None,
            transform: Matrix4::IDENTITY,
            extensions: HashMap::new(),
        }
    }
}
