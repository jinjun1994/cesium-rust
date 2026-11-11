//! PropertyTexture matching CesiumJS `Scene/PropertyTexture.js` (Apache-2.0).

use std::collections::HashMap;
use std::rc::Rc;

use cesium_math::DeveloperError;
use serde_json::Value;

use crate::metadata_class::MetadataClass;
use crate::property_texture_property::PropertyTextureProperty;
use crate::texture::Texture;

/// A texture-based structural metadata property set matching EXT_structural_metadata.
#[derive(Clone, Debug, PartialEq)]
pub struct PropertyTexture {
    pub name: Option<String>,
    pub id: Option<Value>,
    pub class: MetadataClass,
    pub properties: HashMap<String, PropertyTextureProperty>,
    pub extras: Option<Value>,
    pub extensions: Option<Value>,
}

impl PropertyTexture {
    /// Creates a new `PropertyTexture`.
    pub fn new(
        property_texture: Option<&Value>,
        class: Option<&MetadataClass>,
        textures: Option<&HashMap<usize, Rc<Texture>>>,
        name: Option<String>,
        id: Option<Value>,
    ) -> Result<Self, DeveloperError> {
        let property_texture =
            property_texture.ok_or_else(|| DeveloperError::new("options.propertyTexture is required."))?;
        let class = class.ok_or_else(|| DeveloperError::new("options.class is required."))?;
        let textures = textures.ok_or_else(|| DeveloperError::new("options.textures is required."))?;

        let mut properties = HashMap::new();
        if let Some(props) = property_texture.get("properties").and_then(|p| p.as_object()) {
            for (property_id, prop_val) in props {
                if let Some(class_prop) = class.properties.get(property_id) {
                    let ptp = PropertyTextureProperty::new(
                        Some(prop_val),
                        Some(class_prop),
                        Some(textures),
                    )?;
                    properties.insert(property_id.clone(), ptp);
                }
            }
        }

        let extras = property_texture.get("extras").cloned();
        let extensions = property_texture.get("extensions").cloned();

        Ok(Self {
            name,
            id,
            class: class.clone(),
            properties,
            extras,
            extensions,
        })
    }

    /// Gets the property with the given property ID.
    pub fn get_property(
        &self,
        property_id: Option<&str>,
    ) -> Result<Option<&PropertyTextureProperty>, DeveloperError> {
        let id = property_id.ok_or_else(|| DeveloperError::new("propertyId is required."))?;
        Ok(self.properties.get(id))
    }
}
