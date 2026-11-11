//! MetadataEntity matching CesiumJS `Scene/MetadataEntity.js` (Apache-2.0).

use crate::metadata_class::MetadataClass;
use crate::metadata_component_type::MetadataComponentType;
use cesium_math::DeveloperError;
use serde_json::Value;
use std::collections::HashMap;

/// Trait representing an entity containing metadata conforming to a `MetadataClass`.
pub trait MetadataEntityTrait {
    fn class_definition(&self) -> Result<&MetadataClass, DeveloperError>;
    fn has_property(&self, property_id: &str) -> Result<bool, DeveloperError>;
    fn has_property_by_semantic(&self, semantic: &str) -> Result<bool, DeveloperError>;
    fn get_property_ids(&self) -> Result<Vec<String>, DeveloperError>;
    fn get_property(&self, property_id: &str) -> Result<Option<Value>, DeveloperError>;
    fn set_property(&mut self, property_id: &str, value: Value) -> Result<bool, DeveloperError>;
    fn get_property_by_semantic(&self, semantic: &str) -> Result<Option<Value>, DeveloperError>;
    fn set_property_by_semantic(&mut self, semantic: &str, value: Value) -> Result<bool, DeveloperError>;
}

/// Base type for metadata entities that throws DeveloperError when used directly.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct MetadataEntityBase;

impl MetadataEntityTrait for MetadataEntityBase {
    fn class_definition(&self) -> Result<&MetadataClass, DeveloperError> {
        Err(DeveloperError::new(
            "This type describes an interface and is not intended to be instantiated directly.",
        ))
    }

    fn has_property(&self, _property_id: &str) -> Result<bool, DeveloperError> {
        Err(DeveloperError::new(
            "This type describes an interface and is not intended to be instantiated directly.",
        ))
    }

    fn has_property_by_semantic(&self, _semantic: &str) -> Result<bool, DeveloperError> {
        Err(DeveloperError::new(
            "This type describes an interface and is not intended to be instantiated directly.",
        ))
    }

    fn get_property_ids(&self) -> Result<Vec<String>, DeveloperError> {
        Err(DeveloperError::new(
            "This type describes an interface and is not intended to be instantiated directly.",
        ))
    }

    fn get_property(&self, _property_id: &str) -> Result<Option<Value>, DeveloperError> {
        Err(DeveloperError::new(
            "This type describes an interface and is not intended to be instantiated directly.",
        ))
    }

    fn set_property(&mut self, _property_id: &str, _value: Value) -> Result<bool, DeveloperError> {
        Err(DeveloperError::new(
            "This type describes an interface and is not intended to be instantiated directly.",
        ))
    }

    fn get_property_by_semantic(&self, _semantic: &str) -> Result<Option<Value>, DeveloperError> {
        Err(DeveloperError::new(
            "This type describes an interface and is not intended to be instantiated directly.",
        ))
    }

    fn set_property_by_semantic(&mut self, _semantic: &str, _value: Value) -> Result<bool, DeveloperError> {
        Err(DeveloperError::new(
            "This type describes an interface and is not intended to be instantiated directly.",
        ))
    }
}

/// Static helper functions for operating on metadata entity dictionaries and schemas.
pub struct MetadataEntity;

impl MetadataEntity {
    /// Returns whether the entity has the given property ID or a default value for it.
    pub fn has_property(
        property_id: &str,
        properties: &HashMap<String, Value>,
        class_definition: &MetadataClass,
    ) -> bool {
        if properties.contains_key(property_id) {
            return true;
        }

        if let Some(prop_def) = class_definition.properties.get(property_id) {
            if prop_def.default.is_some() {
                return true;
            }
        }

        false
    }

    /// Returns whether the entity has a property matching the given semantic.
    pub fn has_property_by_semantic(
        semantic: &str,
        _properties: &HashMap<String, Value>,
        class_definition: &MetadataClass,
    ) -> bool {
        class_definition.properties_by_semantic.contains_key(semantic)
    }

    /// Returns all property IDs present in properties or having a default value in the class.
    pub fn get_property_ids(
        properties: &HashMap<String, Value>,
        class_definition: &MetadataClass,
    ) -> Vec<String> {
        let mut results = Vec::new();

        for key in properties.keys() {
            results.push(key.clone());
        }

        for (class_prop_id, class_prop) in &class_definition.properties {
            if !properties.contains_key(class_prop_id) && class_prop.default.is_some() {
                results.push(class_prop_id.clone());
            }
        }

        results
    }

    /// Retrieves a property value, taking noData, normalization, offset/scale, and default into account.
    pub fn get_property(
        property_id: &str,
        properties: &HashMap<String, Value>,
        class_definition: &MetadataClass,
    ) -> Result<Option<Value>, DeveloperError> {
        let class_property = class_definition
            .properties
            .get(property_id)
            .ok_or_else(|| {
                DeveloperError::new(format!(
                    "Class definition missing property {property_id}"
                ))
            })?;

        let mut val_opt = properties.get(property_id).cloned();

        // Check noData sentinel
        if let Some(ref val) = val_opt {
            if let Some(ref no_data) = class_property.no_data {
                if val == no_data {
                    val_opt = None;
                }
            }
        }

        // Apply default if undefined
        if val_opt.is_none() {
            if let Some(ref def) = class_property.default {
                return Ok(Some(def.clone()));
            }
            return Ok(None);
        }

        let val = val_opt.unwrap();

        // Apply normalization and value transform
        let transformed = Self::apply_transforms(val, class_property)?;
        Ok(Some(transformed))
    }

    /// Sets the property value, unapplying value transforms and unnormalizing.
    pub fn set_property(
        property_id: &str,
        value: Value,
        properties: &mut HashMap<String, Value>,
        class_definition: &MetadataClass,
    ) -> Result<bool, DeveloperError> {
        if !properties.contains_key(property_id) {
            return Ok(false);
        }

        let stored_val = if let Some(class_property) = class_definition.properties.get(property_id) {
            Self::unapply_transforms(value, class_property)?
        } else {
            value
        };

        properties.insert(property_id.to_string(), stored_val);
        Ok(true)
    }

    /// Retrieves a property value by its semantic.
    pub fn get_property_by_semantic(
        semantic: &str,
        properties: &HashMap<String, Value>,
        class_definition: &MetadataClass,
    ) -> Result<Option<Value>, DeveloperError> {
        if let Some(prop) = class_definition.properties_by_semantic.get(semantic) {
            Self::get_property(&prop.id, properties, class_definition)
        } else {
            Ok(None)
        }
    }

    /// Sets a property value by its semantic.
    pub fn set_property_by_semantic(
        semantic: &str,
        value: Value,
        properties: &mut HashMap<String, Value>,
        class_definition: &MetadataClass,
    ) -> Result<bool, DeveloperError> {
        if let Some(prop) = class_definition.properties_by_semantic.get(semantic) {
            Self::set_property(&prop.id, value, properties, class_definition)
        } else {
            Ok(false)
        }
    }

    fn apply_transforms(
        val: Value,
        prop_def: &crate::metadata_class_property::MetadataClassProperty,
    ) -> Result<Value, DeveloperError> {
        match val {
            Value::Number(n) => {
                let mut num = n.as_f64().unwrap_or(0.0);
                if prop_def.normalized {
                    if let Some(ct) = prop_def.value_type {
                        num = MetadataComponentType::normalize(num, ct)?;
                    }
                }
                if prop_def.has_value_transform {
                    let offset = prop_def
                        .offset
                        .as_ref()
                        .and_then(|o| o.as_f64())
                        .unwrap_or(0.0);
                    let scale = prop_def
                        .scale
                        .as_ref()
                        .and_then(|s| s.as_f64())
                        .unwrap_or(1.0);
                    num = MetadataComponentType::apply_value_transform(num, offset, scale);
                }
                Ok(serde_json::json!(num))
            }
            Value::Array(arr) => {
                if !prop_def.normalized && !prop_def.has_value_transform {
                    return Ok(Value::Array(arr));
                }
                let mut transformed_arr = Vec::new();
                for (i, elem) in arr.into_iter().enumerate() {
                    let mut num = elem.as_f64().unwrap_or(0.0);
                    if prop_def.normalized {
                        if let Some(ct) = prop_def.value_type {
                            num = MetadataComponentType::normalize(num, ct)?;
                        }
                    }
                    if prop_def.has_value_transform {
                        let offset = prop_def
                            .offset
                            .as_ref()
                            .and_then(|o| {
                                if let Some(sub_arr) = o.as_array() {
                                    sub_arr.get(i).and_then(|v| v.as_f64())
                                } else {
                                    o.as_f64()
                                }
                            })
                            .unwrap_or(0.0);

                        let scale = prop_def
                            .scale
                            .as_ref()
                            .and_then(|s| {
                                if let Some(sub_arr) = s.as_array() {
                                    sub_arr.get(i).and_then(|v| v.as_f64())
                                } else {
                                    s.as_f64()
                                }
                            })
                            .unwrap_or(1.0);

                        num = MetadataComponentType::apply_value_transform(num, offset, scale);
                    }
                    transformed_arr.push(serde_json::json!(num));
                }
                Ok(Value::Array(transformed_arr))
            }
            other => Ok(other),
        }
    }

    fn unapply_transforms(
        val: Value,
        prop_def: &crate::metadata_class_property::MetadataClassProperty,
    ) -> Result<Value, DeveloperError> {
        match val {
            Value::Number(n) => {
                let mut num = n.as_f64().unwrap_or(0.0);
                if prop_def.has_value_transform {
                    let offset = prop_def
                        .offset
                        .as_ref()
                        .and_then(|o| o.as_f64())
                        .unwrap_or(0.0);
                    let scale = prop_def
                        .scale
                        .as_ref()
                        .and_then(|s| s.as_f64())
                        .unwrap_or(1.0);
                    num = MetadataComponentType::unapply_value_transform(num, offset, scale);
                }
                if prop_def.normalized {
                    if let Some(ct) = prop_def.value_type {
                        num = MetadataComponentType::unnormalize(num, ct)?;
                    }
                }
                Ok(serde_json::json!(num))
            }
            Value::Array(arr) => {
                if !prop_def.normalized && !prop_def.has_value_transform {
                    return Ok(Value::Array(arr));
                }
                let mut transformed_arr = Vec::new();
                for (i, elem) in arr.into_iter().enumerate() {
                    let mut num = elem.as_f64().unwrap_or(0.0);
                    if prop_def.has_value_transform {
                        let offset = prop_def
                            .offset
                            .as_ref()
                            .and_then(|o| {
                                if let Some(sub_arr) = o.as_array() {
                                    sub_arr.get(i).and_then(|v| v.as_f64())
                                } else {
                                    o.as_f64()
                                }
                            })
                            .unwrap_or(0.0);

                        let scale = prop_def
                            .scale
                            .as_ref()
                            .and_then(|s| {
                                if let Some(sub_arr) = s.as_array() {
                                    sub_arr.get(i).and_then(|v| v.as_f64())
                                } else {
                                    s.as_f64()
                                }
                            })
                            .unwrap_or(1.0);

                        num = MetadataComponentType::unapply_value_transform(num, offset, scale);
                    }
                    if prop_def.normalized {
                        if let Some(ct) = prop_def.value_type {
                            num = MetadataComponentType::unnormalize(num, ct)?;
                        }
                    }
                    transformed_arr.push(serde_json::json!(num));
                }
                Ok(Value::Array(transformed_arr))
            }
            other => Ok(other),
        }
    }
}
