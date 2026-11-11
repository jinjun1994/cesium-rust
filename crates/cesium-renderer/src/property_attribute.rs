//! PropertyAttribute matching CesiumJS `Scene/PropertyAttribute.js` (Apache-2.0).

use std::collections::HashMap;

use cesium_math::DeveloperError;
use serde_json::Value;

use crate::metadata_class::MetadataClass;
use crate::property_attribute_property::PropertyAttributeProperty;

/// An attribute-based structural metadata property set matching EXT_structural_metadata.
#[derive(Clone, Debug, PartialEq)]
pub struct PropertyAttribute {
    pub class: MetadataClass,
    pub properties: HashMap<String, PropertyAttributeProperty>,
    pub extras: Option<Value>,
    pub extensions: Option<Value>,
    pub id: Option<Value>,
    pub name: Option<String>,
}

impl PropertyAttribute {
    /// Creates a new `PropertyAttribute`.
    pub fn new(
        class: Option<&MetadataClass>,
        property_attribute: Option<&Value>,
    ) -> Result<Self, DeveloperError> {
        let class = class.ok_or_else(|| DeveloperError::new("options.class is required."))?;
        let property_attr = property_attribute
            .ok_or_else(|| DeveloperError::new("options.propertyAttribute is required."))?;

        let prop_obj = property_attr.get("properties").and_then(|p| p.as_object());
        let mut properties = HashMap::new();

        if let Some(props) = prop_obj {
            for (prop_id, prop_val) in props {
                if let Some(class_prop) = class.properties.get(prop_id) {
                    let pap = PropertyAttributeProperty::new(Some(prop_val), Some(class_prop))?;
                    properties.insert(prop_id.clone(), pap);
                }
            }
        }

        let extras = property_attr.get("extras").cloned();
        let extensions = property_attr.get("extensions").cloned();
        let id = property_attr.get("id").cloned();
        let name = property_attr
            .get("name")
            .and_then(|n| n.as_str())
            .map(|s| s.to_string());

        Ok(Self {
            class: class.clone(),
            properties,
            extras,
            extensions,
            id,
            name,
        })
    }

    /// Sets the attribute id.
    pub fn with_id(mut self, id: Option<Value>) -> Self {
        self.id = id;
        self
    }

    /// Sets the attribute name.
    pub fn with_name(mut self, name: Option<String>) -> Self {
        self.name = name;
        self
    }

    /// Gets the property attribute property for a given property ID.
    pub fn get_property(
        &self,
        property_id: Option<&str>,
    ) -> Result<Option<&PropertyAttributeProperty>, DeveloperError> {
        let id = property_id
            .ok_or_else(|| DeveloperError::new("propertyId is required."))?;
        Ok(self.properties.get(id))
    }
}
