//! MetadataClassProperty matching CesiumJS `Scene/MetadataClassProperty.js` (Apache-2.0).

use crate::metadata_component_type::MetadataComponentType;
use crate::metadata_enum::MetadataEnum;
use crate::metadata_type::MetadataType;
use cesium_math::DeveloperError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Metadata class property defining attribute type, semantics, and value transformations.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MetadataClassProperty {
    pub id: String,
    pub name: Option<String>,
    pub description: Option<String>,
    pub r#type: MetadataType,
    pub component_type: Option<MetadataComponentType>,
    #[serde(skip)]
    pub enum_type: Option<MetadataEnum>,
    pub value_type: Option<MetadataComponentType>,
    pub is_array: bool,
    pub is_variable_length_array: bool,
    pub array_length: Option<usize>,
    pub normalized: bool,
    pub max: Option<serde_json::Value>,
    pub min: Option<serde_json::Value>,
    pub offset: Option<serde_json::Value>,
    pub scale: Option<serde_json::Value>,
    pub no_data: Option<serde_json::Value>,
    pub default: Option<serde_json::Value>,
    pub required: bool,
    pub semantic: Option<String>,
    pub extras: Option<serde_json::Value>,
    pub extensions: Option<serde_json::Value>,
    pub is_legacy_extension: bool,
    pub has_value_transform: bool,
}

impl MetadataClassProperty {
    /// Creates a `MetadataClassProperty` by parsing JSON property options and enum definitions.
    pub fn from_json(
        id: String,
        property_val: &serde_json::Value,
        enums: Option<&HashMap<String, MetadataEnum>>,
    ) -> Result<Self, DeveloperError> {
        let prop = property_val.as_object().ok_or_else(|| {
            DeveloperError::new("options.property must be an object")
        })?;

        let type_str = prop.get("type").and_then(|t| t.as_str()).ok_or_else(|| {
            DeveloperError::new("options.property.type must be a string")
        })?;

        let component_type_str = prop.get("componentType").and_then(|c| c.as_str());

        let is_legacy = Self::detect_legacy(type_str, component_type_str, prop);

        let parsed = Self::parse_type(type_str, component_type_str, prop, enums)?;

        let required = if let Some(legacy) = is_legacy {
            if legacy {
                prop.get("optional")
                    .and_then(|o| o.as_bool())
                    .map(|opt| !opt)
                    .unwrap_or(true)
            } else {
                prop.get("required").and_then(|r| r.as_bool()).unwrap_or(false)
            }
        } else {
            false
        };

        let normalized = parsed
            .component_type
            .map(|ct| ct.is_integer_type())
            .unwrap_or(false)
            && prop.get("normalized").and_then(|n| n.as_bool()).unwrap_or(false);

        let offset = prop.get("offset").cloned();
        let scale = prop.get("scale").cloned();
        let has_value_transform = offset.is_some() || scale.is_some();

        Ok(Self {
            id,
            name: prop.get("name").and_then(|v| v.as_str()).map(|s| s.to_string()),
            description: prop.get("description").and_then(|v| v.as_str()).map(|s| s.to_string()),
            r#type: parsed.r#type,
            component_type: parsed.component_type,
            enum_type: parsed.enum_type,
            value_type: parsed.value_type,
            is_array: parsed.is_array,
            is_variable_length_array: parsed.is_variable_length_array,
            array_length: parsed.array_length,
            normalized,
            max: prop.get("max").cloned(),
            min: prop.get("min").cloned(),
            offset,
            scale,
            no_data: prop.get("noData").cloned(),
            default: prop.get("default").cloned(),
            required,
            semantic: prop.get("semantic").and_then(|v| v.as_str()).map(|s| s.to_string()),
            extras: prop.get("extras").cloned(),
            extensions: prop.get("extensions").cloned(),
            is_legacy_extension: is_legacy.unwrap_or(false),
            has_value_transform,
        })
    }

    fn detect_legacy(
        type_str: &str,
        _component_type_str: Option<&str>,
        prop: &serde_json::Map<String, serde_json::Value>,
    ) -> Option<bool> {
        if type_str == "ARRAY" {
            return Some(true);
        }

        if type_str == "SCALAR"
            || type_str.starts_with("VEC")
            || type_str.starts_with("MAT")
        {
            return Some(false);
        }

        if type_str.parse::<MetadataComponentType>().is_ok() {
            return Some(true);
        }

        if prop.contains_key("noData")
            || prop.contains_key("scale")
            || prop.contains_key("offset")
            || prop.contains_key("required")
            || prop.contains_key("count")
            || prop.contains_key("array")
        {
            return Some(false);
        }

        if prop.contains_key("optional") {
            return Some(false);
        }

        None
    }

    fn parse_type(
        type_str: &str,
        component_type_str: Option<&str>,
        prop: &serde_json::Map<String, serde_json::Value>,
        enums: Option<&HashMap<String, MetadataEnum>>,
    ) -> Result<ParsedTypeInfo, DeveloperError> {
        let is_legacy_array = type_str == "ARRAY";
        let is_array;
        let array_length;
        let is_variable_length_array;

        if is_legacy_array {
            is_array = true;
            array_length = prop.get("componentCount").and_then(|c| c.as_u64()).map(|u| u as usize);
            is_variable_length_array = array_length.is_none();
        } else if prop.get("array").and_then(|a| a.as_bool()).unwrap_or(false) {
            is_array = true;
            array_length = prop.get("count").and_then(|c| c.as_u64()).map(|u| u as usize);
            is_variable_length_array = array_length.is_none();
        } else {
            is_array = false;
            array_length = None;
            is_variable_length_array = false;
        }

        let enum_type = prop
            .get("enumType")
            .and_then(|e| e.as_str())
            .and_then(|e_id| enums.and_then(|map| map.get(e_id).cloned()));

        if type_str == "ENUM" {
            let vt = enum_type.as_ref().map(|e| e.value_type);
            return Ok(ParsedTypeInfo {
                r#type: MetadataType::Enum,
                component_type: None,
                enum_type,
                value_type: vt,
                is_array,
                is_variable_length_array,
                array_length,
            });
        }

        if is_legacy_array && component_type_str == Some("ENUM") {
            let vt = enum_type.as_ref().map(|e| e.value_type);
            return Ok(ParsedTypeInfo {
                r#type: MetadataType::Enum,
                component_type: None,
                enum_type,
                value_type: vt,
                is_array,
                is_variable_length_array,
                array_length,
            });
        }

        if let Ok(meta_type) = type_str.parse::<MetadataType>() {
            if meta_type == MetadataType::Scalar
                || meta_type.is_matrix_type()
                || meta_type.is_vector_type()
            {
                let ct = component_type_str
                    .and_then(|s| s.parse::<MetadataComponentType>().ok());
                return Ok(ParsedTypeInfo {
                    r#type: meta_type,
                    component_type: ct,
                    enum_type: None,
                    value_type: ct,
                    is_array,
                    is_variable_length_array,
                    array_length,
                });
            }

            if meta_type == MetadataType::Boolean || meta_type == MetadataType::String {
                return Ok(ParsedTypeInfo {
                    r#type: meta_type,
                    component_type: None,
                    enum_type: None,
                    value_type: None,
                    is_array,
                    is_variable_length_array,
                    array_length,
                });
            }
        }

        if is_legacy_array {
            if let Some(ct_str) = component_type_str {
                if let Ok(meta_type) = ct_str.parse::<MetadataType>() {
                    if meta_type == MetadataType::Boolean || meta_type == MetadataType::String {
                        return Ok(ParsedTypeInfo {
                            r#type: meta_type,
                            component_type: None,
                            enum_type: None,
                            value_type: None,
                            is_array,
                            is_variable_length_array,
                            array_length,
                        });
                    }
                }
            }
        }

        if let Some(ct_str) = component_type_str {
            if let Ok(ct) = ct_str.parse::<MetadataComponentType>() {
                return Ok(ParsedTypeInfo {
                    r#type: MetadataType::Scalar,
                    component_type: Some(ct),
                    enum_type: None,
                    value_type: Some(ct),
                    is_array,
                    is_variable_length_array,
                    array_length,
                });
            }
        }

        if let Ok(ct) = type_str.parse::<MetadataComponentType>() {
            return Ok(ParsedTypeInfo {
                r#type: MetadataType::Scalar,
                component_type: Some(ct),
                enum_type: None,
                value_type: Some(ct),
                is_array,
                is_variable_length_array,
                array_length,
            });
        }

        Err(DeveloperError::new(format!(
            "unknown metadata type {{type: {type_str}, componentType: {component_type_str:?}}}"
        )))
    }

    /// Computes CPU bytes per element.
    pub fn cpu_bytes_per_element(&self) -> usize {
        let comp_count = self.r#type.get_component_count();
        let array_len = if self.is_array {
            self.array_length.unwrap_or(1)
        } else {
            1
        };
        let bytes_per_comp = self
            .value_type
            .map(MetadataComponentType::get_size_in_bytes)
            .unwrap_or(1);
        comp_count * array_len * bytes_per_comp
    }

    /// Computes GPU bytes per element.
    pub fn gpu_bytes_per_element(&self) -> usize {
        let comp_count = self.r#type.get_component_count();
        let array_len = if self.is_array {
            self.array_length.unwrap_or(1)
        } else {
            1
        };
        let bytes_per_comp = self
            .value_type
            .map(|vt| match vt {
                MetadataComponentType::Int64
                | MetadataComponentType::Uint64
                | MetadataComponentType::Float64 => 4,
                other => MetadataComponentType::get_size_in_bytes(other),
            })
            .unwrap_or(1);
        comp_count * array_len * bytes_per_comp
    }

    /// Checks if this property is GPU texture compatible.
    pub fn is_gpu_compatible(&self, channels_length: usize) -> Result<bool, DeveloperError> {
        if channels_length == 0 {
            return Err(DeveloperError::new(
                "channelsLength must be a positive number.",
            ));
        }

        if self.is_variable_length_array {
            return Ok(false);
        }

        if self.r#type == MetadataType::String || self.r#type == MetadataType::Boolean {
            return Ok(false);
        }

        if self.gpu_bytes_per_element() > channels_length {
            return Ok(false);
        }

        Ok(true)
    }

    /// Returns GLSL type name for shader generation.
    pub fn get_glsl_type(&self) -> &'static str {
        let comp_count = self.r#type.get_component_count()
            * if self.is_array {
                self.array_length.unwrap_or(1)
            } else {
                1
            };

        let is_int = self
            .value_type
            .map(|vt| vt.is_integer_type())
            .unwrap_or(false);

        if !is_int || self.normalized {
            match comp_count {
                1 => "float",
                2 => "vec2",
                3 => "vec3",
                4 => "vec4",
                _ => "float",
            }
        } else if self
            .value_type
            .map(|vt| vt.is_unsigned_integer_type())
            .unwrap_or(false)
        {
            match comp_count {
                1 => "uint",
                2 => "uvec2",
                3 => "uvec3",
                4 => "uvec4",
                _ => "uint",
            }
        } else {
            match comp_count {
                1 => "int",
                2 => "ivec2",
                3 => "ivec3",
                4 => "ivec4",
                _ => "int",
            }
        }
    }

    /// Validates whether a value conforms to this property.
    pub fn validate(&self, value: &serde_json::Value) -> Result<(), DeveloperError> {
        if value.is_null() {
            if self.default.is_some() {
                return Ok(());
            }
            if self.required {
                return Err(DeveloperError::new("required property must have a value"));
            }
            return Ok(());
        }

        if self.is_array {
            let arr = value
                .as_array()
                .ok_or_else(|| DeveloperError::new("value must be an array"))?;
            if !self.is_variable_length_array {
                if let Some(len) = self.array_length {
                    if arr.len() != len {
                        return Err(DeveloperError::new(
                            "Array length does not match property.arrayLength",
                        ));
                    }
                }
            }
        }

        Ok(())
    }
}

struct ParsedTypeInfo {
    r#type: MetadataType,
    component_type: Option<MetadataComponentType>,
    enum_type: Option<MetadataEnum>,
    value_type: Option<MetadataComponentType>,
    is_array: bool,
    is_variable_length_array: bool,
    array_length: Option<usize>,
}
