//! PropertyTextureProperty matching CesiumJS `Scene/PropertyTextureProperty.js` (Apache-2.0).

use std::collections::HashMap;
use std::rc::Rc;

use cesium_math::{DeveloperError, Matrix3};
use serde_json::Value;

use crate::gltf_loader_util::ModelTextureReader;
use crate::metadata_class_property::MetadataClassProperty;
use crate::metadata_type::MetadataType;
use crate::texture::Texture;

/// A property in a PropertyTexture, mapping to a texture reader and class property.
#[derive(Clone, Debug, PartialEq)]
pub struct PropertyTextureProperty {
    pub texture_reader: ModelTextureReader,
    pub class_property: MetadataClassProperty,
    pub has_value_transform: bool,
    pub offset: Value,
    pub scale: Value,
    pub min: Option<Value>,
    pub max: Option<Value>,
    pub extras: Option<Value>,
    pub extensions: Option<Value>,
}

impl PropertyTextureProperty {
    /// Creates a new `PropertyTextureProperty`.
    pub fn new(
        property: Option<&Value>,
        class_property: Option<&MetadataClassProperty>,
        textures: Option<&HashMap<usize, Rc<Texture>>>,
    ) -> Result<Self, DeveloperError> {
        let property = property.ok_or_else(|| DeveloperError::new("options.property is required."))?;
        let class_prop =
            class_property.ok_or_else(|| DeveloperError::new("options.classProperty is required."))?;
        let textures = textures.ok_or_else(|| DeveloperError::new("options.textures is required."))?;

        let channels_str = if let Some(arr) = property.get("channels").and_then(|c| c.as_array()) {
            arr.iter()
                .filter_map(|ch| ch.as_u64())
                .filter_map(|idx| match idx {
                    0 => Some('r'),
                    1 => Some('g'),
                    2 => Some('b'),
                    3 => Some('a'),
                    _ => None,
                })
                .collect::<String>()
        } else {
            "r".to_string()
        };

        let texture_index = property
            .get("index")
            .and_then(Value::as_u64)
            .unwrap_or(0) as usize;

        let mut tex_coord = property
            .get("texCoord")
            .and_then(Value::as_u64)
            .unwrap_or(0) as usize;

        let scale_tex = property.get("scale").and_then(Value::as_f64);

        let mut transform = None;
        if let Some(ext) = property
            .get("extensions")
            .and_then(|e| e.get("KHR_texture_transform"))
        {
            if let Some(tc) = ext.get("texCoord").and_then(Value::as_u64) {
                tex_coord = tc as usize;
            }

            let offset_x = ext
                .get("offset")
                .and_then(|arr| arr.get(0))
                .and_then(Value::as_f64)
                .unwrap_or(0.0);
            let offset_y = ext
                .get("offset")
                .and_then(|arr| arr.get(1))
                .and_then(Value::as_f64)
                .unwrap_or(0.0);

            let scale_x = ext
                .get("scale")
                .and_then(|arr| arr.get(0))
                .and_then(Value::as_f64)
                .unwrap_or(1.0);
            let scale_y = ext
                .get("scale")
                .and_then(|arr| arr.get(1))
                .and_then(Value::as_f64)
                .unwrap_or(1.0);

            // Matrix3 column-major constructor:
            // [scale_x, 0, offset_x]
            // [0, scale_y, offset_y]
            // [0, 0, 1]
            transform = Some(Matrix3::new(
                scale_x, 0.0, 0.0,
                0.0, scale_y, 0.0,
                offset_x, offset_y, 1.0,
            ));
        }

        let texture = textures.get(&texture_index).map(|t| (**t).clone());

        let texture_reader = ModelTextureReader {
            index: texture_index,
            texture,
            tex_coord,
            scale: scale_tex,
            transform,
            channels: Some(channels_str),
            constant_lod: None,
        };

        let prop_offset = property.get("offset");
        let prop_scale = property.get("scale");

        let class_offset = class_prop.offset.as_ref();
        let class_scale = class_prop.scale.as_ref();

        let has_value_transform = prop_offset.is_some()
            || prop_scale.is_some()
            || class_offset.is_some()
            || class_scale.is_some();

        let default_offset = Self::default_offset(class_prop.r#type);
        let default_scale = Self::default_scale(class_prop.r#type);

        let offset = prop_offset
            .cloned()
            .or_else(|| class_offset.cloned())
            .unwrap_or(default_offset);

        let scale = prop_scale
            .cloned()
            .or_else(|| class_scale.cloned())
            .unwrap_or(default_scale);

        let min = property.get("min").cloned().or_else(|| class_prop.min.clone());
        let max = property.get("max").cloned().or_else(|| class_prop.max.clone());
        let extras = property.get("extras").cloned();
        let extensions = property.get("extensions").cloned();

        Ok(Self {
            texture_reader,
            class_property: class_prop.clone(),
            has_value_transform,
            offset,
            scale,
            min,
            max,
            extras,
            extensions,
        })
    }

    fn default_offset(meta_type: MetadataType) -> Value {
        match meta_type {
            MetadataType::Scalar => Value::from(0.0),
            MetadataType::Vec2 => serde_json::json!([0.0, 0.0]),
            MetadataType::Vec3 => serde_json::json!([0.0, 0.0, 0.0]),
            MetadataType::Vec4 => serde_json::json!([0.0, 0.0, 0.0, 0.0]),
            MetadataType::Mat2 => serde_json::json!([0.0, 0.0, 0.0, 0.0]),
            MetadataType::Mat3 => serde_json::json!([0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]),
            MetadataType::Mat4 => serde_json::json!([
                0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0
            ]),
            _ => Value::from(0.0),
        }
    }

    fn default_scale(meta_type: MetadataType) -> Value {
        match meta_type {
            MetadataType::Scalar => Value::from(1.0),
            MetadataType::Vec2 => serde_json::json!([1.0, 1.0]),
            MetadataType::Vec3 => serde_json::json!([1.0, 1.0, 1.0]),
            MetadataType::Vec4 => serde_json::json!([1.0, 1.0, 1.0, 1.0]),
            MetadataType::Mat2 => serde_json::json!([1.0, 1.0, 1.0, 1.0]),
            MetadataType::Mat3 => serde_json::json!([1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0]),
            MetadataType::Mat4 => serde_json::json!([
                1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0
            ]),
            _ => Value::from(1.0),
        }
    }
}
