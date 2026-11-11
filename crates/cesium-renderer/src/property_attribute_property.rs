//! PropertyAttributeProperty matching CesiumJS `Scene/PropertyAttributeProperty.js` (Apache-2.0).

use cesium_math::DeveloperError;
use serde_json::Value;

use crate::metadata_class_property::MetadataClassProperty;
use crate::metadata_type::MetadataType;

/// A property in a PropertyAttribute, mapping to a vertex attribute and class property.
#[derive(Clone, Debug, PartialEq)]
pub struct PropertyAttributeProperty {
    pub attribute: String,
    pub class_property: MetadataClassProperty,
    pub has_value_transform: bool,
    pub offset: Value,
    pub scale: Value,
    pub extras: Option<Value>,
    pub extensions: Option<Value>,
}

impl PropertyAttributeProperty {
    /// Creates a new `PropertyAttributeProperty`.
    pub fn new(
        property: Option<&Value>,
        class_property: Option<&MetadataClassProperty>,
    ) -> Result<Self, DeveloperError> {
        let property = property.ok_or_else(|| {
            DeveloperError::new("options.property is required.")
        })?;
        let class_prop = class_property.ok_or_else(|| {
            DeveloperError::new("options.classProperty is required.")
        })?;

        let attribute = property
            .get("attribute")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

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

        let extras = property.get("extras").cloned();
        let extensions = property.get("extensions").cloned();

        Ok(Self {
            attribute,
            class_property: class_prop.clone(),
            has_value_transform,
            offset,
            scale,
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
