//! GltfStructuralMetadataLoader matching CesiumJS `Scene/GltfStructuralMetadataLoader.js` (Apache-2.0).

use crate::buffer_loader::ResourceLoaderState;
use crate::gltf_buffer_view_loader::{GltfBufferViewLoader, GltfBufferViewLoaderOptions};
use crate::gltf_loader_util::{CreateModelTextureReaderOptions, GltfLoaderUtil, ModelTextureReader};
use crate::gltf_texture_loader::{
    GltfTextureLoader, GltfTextureLoaderOptions, SupportedImageFormats,
};
use cesium_math::{DeveloperError, RuntimeError};
use serde_json::Value;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::future::Future;
use std::sync::{Arc, Mutex};

/// A property table in glTF structural metadata.
#[derive(Debug, Clone, PartialEq)]
pub struct PropertyTable {
    pub id: usize,
    pub name: Option<String>,
    pub class_name: String,
    pub count: usize,
    pub properties: HashMap<String, Vec<Value>>,
}

impl PropertyTable {
    /// Gets a property value by row and property name.
    pub fn get_property(&self, row: usize, property_name: &str) -> Option<Value> {
        self.properties
            .get(property_name)
            .and_then(|col| col.get(row).cloned())
    }
}

/// A property in a property texture.
#[derive(Debug, Clone, PartialEq)]
pub struct PropertyTextureProperty {
    pub texture_reader: ModelTextureReader,
    pub channels: Vec<usize>,
}

/// A property texture in glTF structural metadata.
#[derive(Debug, Clone, PartialEq)]
pub struct PropertyTexture {
    pub id: usize,
    pub name: Option<String>,
    pub class_name: String,
    pub properties: HashMap<String, PropertyTextureProperty>,
}

impl PropertyTexture {
    /// Gets a property texture property by name.
    pub fn get_property(&self, property_name: &str) -> Option<&PropertyTextureProperty> {
        self.properties.get(property_name)
    }
}

/// Structural metadata schema representation.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct MetadataSchema {
    pub id: Option<String>,
    pub classes: BTreeMap<String, Value>,
}

/// Structural metadata object parsed from glTF extension.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct StructuralMetadata {
    pub schema: MetadataSchema,
    pub property_tables: Vec<PropertyTable>,
    pub property_textures: Vec<PropertyTexture>,
}

impl StructuralMetadata {
    /// Gets property table by index.
    pub fn get_property_table(&self, index: usize) -> Option<&PropertyTable> {
        self.property_tables.get(index)
    }

    /// Gets property texture by index.
    pub fn get_property_texture(&self, index: usize) -> Option<&PropertyTexture> {
        self.property_textures.get(index)
    }
}

/// Options for creating a [`GltfStructuralMetadataLoader`].
#[derive(Debug, Clone, Default)]
pub struct GltfStructuralMetadataLoaderOptions {
    pub gltf: Option<Value>,
    pub extension: Option<Value>,
    pub extension_legacy: Option<Value>,
    pub gltf_resource: Option<String>,
    pub base_resource: Option<String>,
    pub supported_image_formats: Option<SupportedImageFormats>,
    pub frame_state: Option<Value>,
    pub cache_key: Option<String>,
    pub asynchronous: Option<bool>,
}

#[derive(Debug)]
struct GltfStructuralMetadataLoaderInner {
    gltf: Option<Value>,
    extension: Option<Value>,
    extension_legacy: Option<Value>,
    gltf_resource: String,
    base_resource: String,
    supported_image_formats: SupportedImageFormats,
    asynchronous: bool,
    cache_key: Option<String>,
    buffer_view_loaders: Vec<GltfBufferViewLoader>,
    buffer_view_ids: Vec<usize>,
    texture_loaders: Vec<GltfTextureLoader>,
    texture_ids: Vec<usize>,
    schema_json: Option<Value>,
    structural_metadata: Option<StructuralMetadata>,
    state: ResourceLoaderState,
    is_destroyed: bool,
}

// SAFETY: StructuralMetadataLoader inner fields are protected by Mutex.
unsafe impl Send for GltfStructuralMetadataLoaderInner {}

/// Loads glTF structural metadata (EXT_structural_metadata / EXT_feature_metadata).
#[derive(Debug, Clone)]
pub struct GltfStructuralMetadataLoader {
    inner: Arc<Mutex<GltfStructuralMetadataLoaderInner>>,
}

impl GltfStructuralMetadataLoader {
    /// Creates a new `GltfStructuralMetadataLoader`.
    pub fn new(options: GltfStructuralMetadataLoaderOptions) -> Result<Self, DeveloperError> {
        let gltf = options
            .gltf
            .ok_or_else(|| DeveloperError::new("options.gltf is required."))?;
        if options.extension.is_none() && options.extension_legacy.is_none() {
            return Err(DeveloperError::new(
                "One of options.extension or options.extensionLegacy must be specified",
            ));
        }
        let gltf_resource = options
            .gltf_resource
            .ok_or_else(|| DeveloperError::new("options.gltfResource is required."))?;
        let base_resource = options
            .base_resource
            .ok_or_else(|| DeveloperError::new("options.baseResource is required."))?;
        let supported_image_formats = options
            .supported_image_formats
            .ok_or_else(|| DeveloperError::new("options.supportedImageFormats is required."))?;
        let _frame_state = options
            .frame_state
            .ok_or_else(|| DeveloperError::new("options.frameState is required."))?;

        let asynchronous = options.asynchronous.unwrap_or(true);

        Ok(Self {
            inner: Arc::new(Mutex::new(GltfStructuralMetadataLoaderInner {
                gltf: Some(gltf),
                extension: options.extension,
                extension_legacy: options.extension_legacy,
                gltf_resource,
                base_resource,
                supported_image_formats,
                asynchronous,
                cache_key: options.cache_key,
                buffer_view_loaders: Vec::new(),
                buffer_view_ids: Vec::new(),
                texture_loaders: Vec::new(),
                texture_ids: Vec::new(),
                schema_json: None,
                structural_metadata: None,
                state: ResourceLoaderState::Unloaded,
                is_destroyed: false,
            })),
        })
    }

    /// The cache key of the resource.
    pub fn cache_key(&self) -> Option<String> {
        self.inner.lock().unwrap().cache_key.clone()
    }

    /// The loaded structural metadata.
    pub fn structural_metadata(&self) -> Option<StructuralMetadata> {
        self.inner.lock().unwrap().structural_metadata.clone()
    }

    /// Current state.
    pub fn state(&self) -> ResourceLoaderState {
        self.inner.lock().unwrap().state
    }

    /// Whether this loader has been destroyed.
    pub fn is_destroyed(&self) -> bool {
        self.inner.lock().unwrap().is_destroyed
    }

    /// Unloads loaded resources.
    pub fn unload(&self) {
        let mut inner = self.inner.lock().unwrap();
        for loader in &inner.buffer_view_loaders {
            loader.unload();
        }
        for loader in &inner.texture_loaders {
            loader.unload();
        }
        inner.buffer_view_loaders.clear();
        inner.buffer_view_ids.clear();
        inner.texture_loaders.clear();
        inner.texture_ids.clear();
        inner.schema_json = None;
        inner.structural_metadata = None;
    }

    /// Destroys the loader and children.
    pub fn destroy(&self) {
        {
            let mut inner = self.inner.lock().unwrap();
            for loader in &inner.buffer_view_loaders {
                loader.destroy();
            }
            for loader in &inner.texture_loaders {
                loader.destroy();
            }
            inner.buffer_view_loaders.clear();
            inner.buffer_view_ids.clear();
            inner.texture_loaders.clear();
            inner.texture_ids.clear();
            inner.schema_json = None;
            inner.structural_metadata = None;
            inner.is_destroyed = true;
        }
    }

    /// Loads bufferViews, textures, and schema.
    pub async fn load<F, Fut>(&self, fetch_fn: Option<F>) -> Result<(), RuntimeError>
    where
        F: Fn(String) -> Fut + Clone + 'static,
        Fut: Future<Output = Result<Vec<u8>, String>>,
    {
        {
            let mut inner = self.inner.lock().unwrap();
            if inner.is_destroyed {
                return Ok(());
            }
            inner.state = ResourceLoaderState::Loading;
        }

        let (gltf, extension, extension_legacy, gltf_resource, base_resource, supported, async_flag) = {
            let inner = self.inner.lock().unwrap();
            if inner.is_destroyed {
                return Ok(());
            }
            (
                inner.gltf.clone().unwrap(),
                inner.extension.clone(),
                inner.extension_legacy.clone(),
                inner.gltf_resource.clone(),
                inner.base_resource.clone(),
                inner.supported_image_formats,
                inner.asynchronous,
            )
        };

        let ext = extension.as_ref().or(extension_legacy.as_ref()).unwrap();

        // 1. Gather and instantiate BufferViewLoaders
        let mut bv_ids = HashSet::new();
        if let Some(property_tables) = ext.get("propertyTables").and_then(Value::as_array) {
            for pt in property_tables {
                if let Some(props) = pt.get("properties").and_then(Value::as_object) {
                    for prop in props.values() {
                        if let Some(v) = prop.get("values").and_then(Value::as_u64) {
                            bv_ids.insert(v as usize);
                        }
                        if let Some(a) = prop.get("arrayOffsets").and_then(Value::as_u64) {
                            bv_ids.insert(a as usize);
                        }
                        if let Some(s) = prop.get("stringOffsets").and_then(Value::as_u64) {
                            bv_ids.insert(s as usize);
                        }
                    }
                }
            }
        }
        if let Some(feature_tables) = ext.get("featureTables").and_then(Value::as_object) {
            for ft in feature_tables.values() {
                if let Some(props) = ft.get("properties").and_then(Value::as_object) {
                    for prop in props.values() {
                        if let Some(v) = prop.get("bufferView").and_then(Value::as_u64) {
                            bv_ids.insert(v as usize);
                        }
                        if let Some(a) = prop.get("arrayOffsetBufferView").and_then(Value::as_u64) {
                            bv_ids.insert(a as usize);
                        }
                        if let Some(s) = prop.get("stringOffsetBufferView").and_then(Value::as_u64) {
                            bv_ids.insert(s as usize);
                        }
                    }
                }
            }
        }

        let mut bv_loaders = Vec::new();
        let mut bv_id_list = Vec::new();
        for &id in &bv_ids {
            let loader = GltfBufferViewLoader::new(GltfBufferViewLoaderOptions {
                gltf: gltf.clone(),
                buffer_view_id: id,
                gltf_resource: Some(gltf_resource.clone()),
                base_resource: Some(base_resource.clone()),
                buffer_loader: None,
                cache_key: None,
            })
            .map_err(|e| RuntimeError::new(e.message))?;

            bv_loaders.push(loader);
            bv_id_list.push(id);
        }

        // 2. Gather and instantiate TextureLoaders
        let mut tex_infos: BTreeMap<usize, Value> = BTreeMap::new();
        if let Some(property_textures) = ext.get("propertyTextures").and_then(Value::as_array) {
            for pt in property_textures {
                if let Some(props) = pt.get("properties").and_then(Value::as_object) {
                    for prop in props.values() {
                        if let Some(idx) = prop.get("index").and_then(Value::as_u64) {
                            tex_infos.insert(idx as usize, prop.clone());
                        }
                    }
                }
            }
        }

        let mut tex_loaders = Vec::new();
        let mut tex_id_list = Vec::new();
        for (&idx, info) in &tex_infos {
            let loader = GltfTextureLoader::new(GltfTextureLoaderOptions {
                gltf: Some(gltf.clone()),
                texture_info: Some(info.clone()),
                gltf_resource: Some(gltf_resource.clone()),
                base_resource: Some(base_resource.clone()),
                supported_image_formats: Some(supported),
                asynchronous: Some(async_flag),
                ..Default::default()
            })
            .map_err(|e| RuntimeError::new(e.message))?;

            tex_loaders.push(loader);
            tex_id_list.push(idx);
        }

        // Store loaders in inner
        {
            let mut inner = self.inner.lock().unwrap();
            if inner.is_destroyed {
                return Ok(());
            }
            inner.buffer_view_loaders = bv_loaders.clone();
            inner.buffer_view_ids = bv_id_list;
            inner.texture_loaders = tex_loaders.clone();
            inner.texture_ids = tex_id_list;
        }

        // 3. Load BufferViews concurrently
        for loader in bv_loaders {
            if let Err(e) = loader.load(fetch_fn.clone()).await {
                self.unload();
                self.inner.lock().unwrap().state = ResourceLoaderState::Failed;
                return Err(RuntimeError::new(format!(
                    "Failed to load structural metadata\n{}",
                    e.message
                )));
            }
        }

        // 4. Load Textures concurrently
        for loader in tex_loaders {
            if let Err(e) = loader.load(fetch_fn.clone()).await {
                self.unload();
                self.inner.lock().unwrap().state = ResourceLoaderState::Failed;
                return Err(RuntimeError::new(format!(
                    "Failed to load structural metadata\n{}",
                    e.message
                )));
            }
        }

        // 5. Load Schema (embedded or via schemaUri)
        let schema_val = if let Some(schema_uri) = ext.get("schemaUri").and_then(Value::as_str) {
            let fetch = fetch_fn.ok_or_else(|| {
                RuntimeError::new("Failed to load structural metadata\nNo fetcher provided")
            })?;
            let full_url = if schema_uri.starts_with("http://")
                || schema_uri.starts_with("https://")
                || schema_uri.starts_with("data:")
            {
                schema_uri.to_string()
            } else {
                let base = base_resource.trim_end_matches('/');
                format!("{base}/{schema_uri}")
            };

            let bytes = match fetch(full_url.clone()).await {
                Ok(b) => b,
                Err(e) => {
                    self.unload();
                    self.inner.lock().unwrap().state = ResourceLoaderState::Failed;
                    return Err(RuntimeError::new(format!(
                        "Failed to load structural metadata\nFailed to load schema: {full_url}\n{e}"
                    )));
                }
            };

            serde_json::from_slice::<Value>(&bytes).map_err(|e| {
                self.unload();
                self.inner.lock().unwrap().state = ResourceLoaderState::Failed;
                RuntimeError::new(format!("Failed to parse schema: {e}"))
            })?
        } else {
            ext.get("schema").cloned().unwrap_or(Value::Null)
        };

        let mut inner = self.inner.lock().unwrap();
        if inner.is_destroyed {
            return Ok(());
        }

        inner.schema_json = Some(schema_val);
        inner.state = ResourceLoaderState::Loaded;
        Ok(())
    }

    /// Processes resources into ready state.
    pub fn process(&self) -> bool {
        let mut inner = self.inner.lock().unwrap();
        if inner.is_destroyed {
            return false;
        }

        if inner.state == ResourceLoaderState::Ready {
            return true;
        }

        if inner.state != ResourceLoaderState::Loaded {
            return false;
        }

        // Process textures
        let texture_loaders = inner.texture_loaders.clone();
        for tex_loader in &texture_loaders {
            if !tex_loader.process() {
                return false;
            }
        }

        let schema_json = inner.schema_json.clone().unwrap_or_default();
        let mut classes = BTreeMap::new();
        if let Some(cls_obj) = schema_json.get("classes").and_then(Value::as_object) {
            for (k, v) in cls_obj {
                classes.insert(k.clone(), v.clone());
            }
        }

        let schema = MetadataSchema {
            id: schema_json.get("id").and_then(Value::as_str).map(|s| s.to_string()),
            classes,
        };

        // Build Property Tables
        let ext = inner.extension.clone().or_else(|| inner.extension_legacy.clone()).unwrap_or_default();
        let mut property_tables = Vec::new();

        if let Some(pt_array) = ext.get("propertyTables").and_then(Value::as_array) {
            for (id, pt_val) in pt_array.iter().enumerate() {
                let name = pt_val.get("name").and_then(Value::as_str).map(|s| s.to_string());
                let class_name = pt_val.get("class").and_then(Value::as_str).unwrap_or("").to_string();
                let count = pt_val.get("count").and_then(Value::as_u64).unwrap_or(0) as usize;

                let mut properties = HashMap::new();
                if let Some(props_obj) = pt_val.get("properties").and_then(Value::as_object) {
                    for (prop_name, prop_val) in props_obj {
                        let values = if let Some(v_arr) = prop_val.get("values").and_then(Value::as_array) {
                            v_arr.clone()
                        } else if let Some(raw_arr) = prop_val.as_array() {
                            raw_arr.clone()
                        } else {
                            Vec::new()
                        };
                        properties.insert(prop_name.clone(), values);
                    }
                }

                property_tables.push(PropertyTable {
                    id,
                    name,
                    class_name,
                    count,
                    properties,
                });
            }
        }

        // Build Property Textures
        let mut property_textures = Vec::new();
        let mut loaded_textures = HashMap::new();
        for (i, &tex_id) in inner.texture_ids.iter().enumerate() {
            if let Some(loader) = inner.texture_loaders.get(i) {
                if let Some(tex) = loader.texture() {
                    loaded_textures.insert(tex_id, tex);
                }
            }
        }

        if let Some(pt_array) = ext.get("propertyTextures").and_then(Value::as_array) {
            for (id, pt_val) in pt_array.iter().enumerate() {
                let name = pt_val.get("name").and_then(Value::as_str).map(|s| s.to_string());
                let class_name = pt_val.get("class").and_then(Value::as_str).unwrap_or("").to_string();

                let mut properties = HashMap::new();
                if let Some(props_obj) = pt_val.get("properties").and_then(Value::as_object) {
                    for (prop_name, prop_val) in props_obj {
                        let channels: Vec<usize> = prop_val
                            .get("channels")
                            .and_then(Value::as_array)
                            .map(|arr| arr.iter().filter_map(Value::as_u64).map(|v| v as usize).collect())
                            .unwrap_or_default();

                        let tex_idx = prop_val.get("index").and_then(Value::as_u64).unwrap_or(0) as usize;
                        let texture = loaded_textures.get(&tex_idx).cloned();

                        let texture_reader = GltfLoaderUtil::create_model_texture_reader(
                            CreateModelTextureReaderOptions {
                                texture_info: Some(prop_val),
                                channels: None,
                                texture,
                            },
                        )
                        .unwrap();

                        properties.insert(
                            prop_name.clone(),
                            PropertyTextureProperty {
                                texture_reader,
                                channels,
                            },
                        );
                    }
                }

                property_textures.push(PropertyTexture {
                    id,
                    name,
                    class_name,
                    properties,
                });
            }
        }

        inner.structural_metadata = Some(StructuralMetadata {
            schema,
            property_tables,
            property_textures,
        });

        inner.state = ResourceLoaderState::Ready;
        true
    }
}
