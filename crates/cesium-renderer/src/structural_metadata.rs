//! StructuralMetadata matching CesiumJS `Scene/StructuralMetadata.js` (Apache-2.0).

use cesium_math::DeveloperError;
use serde_json::Value;

use crate::metadata_schema::MetadataSchema;
use crate::property_attribute::PropertyAttribute;
use crate::property_table::PropertyTable;
use crate::property_texture::PropertyTexture;

/// Options for constructing a [`StructuralMetadata`].
#[derive(Clone, Debug)]
pub struct StructuralMetadataOptions {
    pub schema: MetadataSchema,
    pub property_tables: Option<Vec<PropertyTable>>,
    pub property_textures: Option<Vec<PropertyTexture>>,
    pub property_attributes: Option<Vec<PropertyAttribute>>,
    pub statistics: Option<Value>,
    pub extras: Option<Value>,
    pub extensions: Option<Value>,
}

/// An object containing structural metadata per EXT_structural_metadata.
#[derive(Clone, Debug)]
pub struct StructuralMetadata {
    pub schema: MetadataSchema,
    pub property_tables: Vec<PropertyTable>,
    pub property_textures: Vec<PropertyTexture>,
    pub property_attributes: Vec<PropertyAttribute>,
    pub statistics: Option<Value>,
    pub extras: Option<Value>,
    pub extensions: Option<Value>,
    pub is_destroyed: bool,
}

impl StructuralMetadata {
    /// Creates a new `StructuralMetadata`.
    pub fn new(options: StructuralMetadataOptions) -> Result<Self, DeveloperError> {
        let property_tables = options.property_tables.unwrap_or_default();
        let property_textures = options.property_textures.unwrap_or_default();
        let property_attributes = options.property_attributes.unwrap_or_default();

        Ok(Self {
            schema: options.schema,
            property_tables,
            property_textures,
            property_attributes,
            statistics: options.statistics,
            extras: options.extras,
            extensions: options.extensions,
            is_destroyed: false,
        })
    }

    /// Schema containing classes and enums.
    pub fn schema(&self) -> &MetadataSchema {
        &self.schema
    }

    /// Number of property tables in the metadata.
    pub fn property_table_count(&self) -> usize {
        self.property_tables.len()
    }

    /// The property tables in the metadata.
    pub fn property_tables(&self) -> &[PropertyTable] {
        &self.property_tables
    }

    /// The property textures in the metadata.
    pub fn property_textures(&self) -> &[PropertyTexture] {
        &self.property_textures
    }

    /// The property attributes from the structural metadata extension.
    pub fn property_attributes(&self) -> &[PropertyAttribute] {
        &self.property_attributes
    }

    /// Statistics about the metadata.
    pub fn statistics(&self) -> Option<&Value> {
        self.statistics.as_ref()
    }

    /// Extra user-defined properties.
    pub fn extras(&self) -> Option<&Value> {
        self.extras.as_ref()
    }

    /// An object containing extensions.
    pub fn extensions(&self) -> Option<&Value> {
        self.extensions.as_ref()
    }

    /// Total size in bytes across all property tables.
    pub fn property_tables_byte_length(&self) -> usize {
        self.property_tables
            .iter()
            .map(|table| table.byte_length())
            .sum()
    }

    /// Gets the property table with the given ID.
    pub fn get_property_table(&self, index: usize) -> Result<&PropertyTable, DeveloperError> {
        self.property_tables
            .get(index)
            .ok_or_else(|| DeveloperError::new("propertyTableId is out of bounds."))
    }

    /// Gets the property texture with the given ID.
    pub fn get_property_texture(&self, index: usize) -> Result<&PropertyTexture, DeveloperError> {
        self.property_textures
            .get(index)
            .ok_or_else(|| DeveloperError::new("propertyTextureId is out of bounds."))
    }

    /// Gets the property attribute with the given ID.
    pub fn get_property_attribute(
        &self,
        index: usize,
    ) -> Result<&PropertyAttribute, DeveloperError> {
        self.property_attributes
            .get(index)
            .ok_or_else(|| DeveloperError::new("propertyAttributeId is out of bounds."))
    }

    /// Destroys resources held by this object.
    pub fn destroy(&mut self) {
        self.is_destroyed = true;
        self.property_tables.clear();
        self.property_textures.clear();
        self.property_attributes.clear();
    }
}
