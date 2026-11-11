//! ResourceCacheKey matching CesiumJS `Scene/ResourceCacheKey.js` (Apache-2.0).

use cesium_math::DeveloperError;
use serde_json::Value;

/// Supported image formats for glTF texture loading.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SupportedImageFormats {
    pub webp: bool,
    pub basis: bool,
}

/// Options for computing schema cache key.
#[derive(Clone, Debug, Default)]
pub struct SchemaCacheKeyOptions<'a> {
    pub schema: Option<&'a Value>,
    pub resource: Option<&'a str>,
}

/// Options for vertex buffer cache key.
#[derive(Clone, Debug, Default)]
pub struct VertexBufferCacheKeyOptions<'a> {
    pub gltf: Option<&'a Value>,
    pub gltf_resource: Option<&'a str>,
    pub base_resource: Option<&'a str>,
    pub context_id: Option<&'a str>,
    pub buffer_view_id: Option<usize>,
    pub draco: Option<&'a Value>,
    pub spz: Option<&'a Value>,
    pub attribute_semantic: Option<&'a str>,
    pub dequantize: bool,
    pub load_buffer: bool,
    pub load_typed_array: bool,
}

/// Options for index buffer cache key.
#[derive(Clone, Debug, Default)]
pub struct IndexBufferCacheKeyOptions<'a> {
    pub gltf: Option<&'a Value>,
    pub accessor_id: Option<usize>,
    pub gltf_resource: Option<&'a str>,
    pub base_resource: Option<&'a str>,
    pub context_id: Option<&'a str>,
    pub draco: Option<&'a Value>,
    pub load_buffer: bool,
    pub load_typed_array: bool,
}

/// Options for texture cache key.
#[derive(Clone, Debug, Default)]
pub struct TextureCacheKeyOptions<'a> {
    pub gltf: Option<&'a Value>,
    pub texture_info: Option<&'a Value>,
    pub gltf_resource: Option<&'a str>,
    pub base_resource: Option<&'a str>,
    pub context_id: Option<&'a str>,
    pub supported_image_formats: SupportedImageFormats,
}

/// Utility for generating cache keys matching CesiumJS `ResourceCacheKey`.
pub struct ResourceCacheKey;

impl ResourceCacheKey {
    /// Schema cache key.
    pub fn get_schema_cache_key(options: SchemaCacheKeyOptions) -> Result<String, DeveloperError> {
        if options.schema.is_some() == options.resource.is_some() {
            return Err(DeveloperError::new(
                "One of options.schema and options.resource must be defined.",
            ));
        }

        if let Some(schema) = options.schema {
            let json_str = serde_json::to_string(schema)
                .map_err(|e| DeveloperError::new(e.to_string()))?;
            Ok(format!("embedded-schema:{json_str}"))
        } else if let Some(res) = options.resource {
            Ok(format!("external-schema:{res}"))
        } else {
            Err(DeveloperError::new(
                "One of options.schema and options.resource must be defined.",
            ))
        }
    }

    /// External buffer cache key.
    pub fn get_external_buffer_cache_key(resource: &str) -> String {
        format!("external-buffer:{resource}")
    }

    /// Embedded buffer cache key.
    pub fn get_embedded_buffer_cache_key(parent_resource: &str, buffer_id: usize) -> String {
        format!("embedded-buffer:{parent_resource}-buffer-id-{buffer_id}")
    }

    /// glTF cache key.
    pub fn get_gltf_cache_key(gltf_resource: &str) -> String {
        format!("gltf:{gltf_resource}")
    }

    /// Buffer view cache key.
    pub fn get_buffer_view_cache_key(
        gltf: &Value,
        buffer_view_id: usize,
        gltf_resource: &str,
        base_resource: &str,
    ) -> Result<String, DeveloperError> {
        let buffer_view = gltf
            .get("bufferViews")
            .and_then(|bv| bv.get(buffer_view_id))
            .ok_or_else(|| {
                DeveloperError::new(format!("bufferView ID {buffer_view_id} not found in glTF"))
            })?;

        let meshopt = buffer_view.get("extensions").and_then(|ext| {
            ext.get("EXT_meshopt_compression")
                .or_else(|| ext.get("KHR_meshopt_compression"))
        });

        let (buffer_id, byte_offset, byte_length) = if let Some(m) = meshopt {
            let bid = m.get("buffer").and_then(Value::as_u64).unwrap_or(0) as usize;
            let offset = m.get("byteOffset").and_then(Value::as_u64).unwrap_or(0) as usize;
            let length = m.get("byteLength").and_then(Value::as_u64).unwrap_or(0) as usize;
            (bid, offset, length)
        } else {
            let bid = buffer_view
                .get("buffer")
                .and_then(Value::as_u64)
                .unwrap_or(0) as usize;
            let offset = buffer_view
                .get("byteOffset")
                .and_then(Value::as_u64)
                .unwrap_or(0) as usize;
            let length = buffer_view
                .get("byteLength")
                .and_then(Value::as_u64)
                .unwrap_or(0) as usize;
            (bid, offset, length)
        };

        let buffer_cache_key =
            Self::get_buffer_cache_key(gltf, buffer_id, gltf_resource, base_resource)?;
        let range = format!("{}-{}", byte_offset, byte_offset + byte_length);
        Ok(format!("buffer-view:{buffer_cache_key}-range-{range}"))
    }

    fn get_buffer_cache_key(
        gltf: &Value,
        buffer_id: usize,
        gltf_resource: &str,
        base_resource: &str,
    ) -> Result<String, DeveloperError> {
        let buffer = gltf.get("buffers").and_then(|b| b.get(buffer_id));
        if let Some(uri) = buffer.and_then(|b| b.get("uri")).and_then(Value::as_str) {
            let resolved = Self::resolve_uri(base_resource, uri);
            Ok(resolved)
        } else {
            Ok(Self::get_embedded_buffer_cache_key_raw(
                gltf_resource,
                buffer_id,
            ))
        }
    }

    fn get_embedded_buffer_cache_key_raw(parent_resource: &str, buffer_id: usize) -> String {
        format!("{parent_resource}-buffer-id-{buffer_id}")
    }

    fn get_buffer_view_range_cache_key(
        gltf: &Value,
        buffer_view_id: usize,
        gltf_resource: &str,
        base_resource: &str,
    ) -> Result<String, DeveloperError> {
        let bv_key =
            Self::get_buffer_view_cache_key(gltf, buffer_view_id, gltf_resource, base_resource)?;
        let stripped = bv_key.strip_prefix("buffer-view:").unwrap_or(&bv_key);
        Ok(stripped.to_string())
    }

    /// Draco cache key.
    pub fn get_draco_cache_key(
        gltf: &Value,
        draco: &Value,
        gltf_resource: &str,
        base_resource: &str,
    ) -> Result<String, DeveloperError> {
        let buffer_view_id = draco
            .get("bufferView")
            .and_then(Value::as_u64)
            .ok_or_else(|| DeveloperError::new("draco.bufferView is required"))?
            as usize;

        let inner_key = Self::get_buffer_view_range_cache_key(
            gltf,
            buffer_view_id,
            gltf_resource,
            base_resource,
        )?;
        Ok(format!("draco:{inner_key}"))
    }

    /// SPZ cache key.
    pub fn get_spz_cache_key(
        gltf: &Value,
        spz: &Value,
        gltf_resource: &str,
        base_resource: &str,
    ) -> Result<String, DeveloperError> {
        let buffer_view_id = spz
            .get("bufferView")
            .and_then(Value::as_u64)
            .ok_or_else(|| DeveloperError::new("spz.bufferView is required"))?
            as usize;

        let inner_key = Self::get_buffer_view_range_cache_key(
            gltf,
            buffer_view_id,
            gltf_resource,
            base_resource,
        )?;
        Ok(format!("spz:{inner_key}"))
    }

    /// Vertex buffer cache key.
    pub fn get_vertex_buffer_cache_key(
        options: VertexBufferCacheKeyOptions,
    ) -> Result<String, DeveloperError> {
        let gltf = options
            .gltf
            .ok_or_else(|| DeveloperError::new("options.gltf is required"))?;
        let gltf_resource = options
            .gltf_resource
            .ok_or_else(|| DeveloperError::new("options.gltfResource is required"))?;
        let base_resource = options
            .base_resource
            .ok_or_else(|| DeveloperError::new("options.baseResource is required"))?;
        let context_id = options
            .context_id
            .ok_or_else(|| DeveloperError::new("options.frameState is required"))?;

        let has_buffer_view = options.buffer_view_id.is_some();
        let has_draco = options.draco.is_some()
            && options
                .draco
                .and_then(|d| d.get("attributes"))
                .and_then(|a| options.attribute_semantic.and_then(|s| a.get(s)))
                .is_some();
        let has_attribute_semantic = options.attribute_semantic.is_some();
        let has_spz = options.spz.is_some();

        if options.draco.is_some() && !has_attribute_semantic {
            return Err(DeveloperError::new(
                "When options.draco is defined options.attributeSemantic must also be defined.",
            ));
        }

        let source_count = has_buffer_view as usize + has_draco as usize + has_spz as usize;
        if source_count != 1 {
            return Err(DeveloperError::new(
                "Exactly one vertex buffer source must be effective: options.bufferViewId, options.spz, or options.draco for options.attributeSemantic.",
            ));
        }

        if !options.load_buffer && !options.load_typed_array {
            return Err(DeveloperError::new(
                "At least one of loadBuffer and loadTypedArray must be true.",
            ));
        }

        let mut suffix = String::new();
        if options.dequantize {
            suffix.push_str("-dequantize");
        }
        if options.load_buffer {
            suffix.push_str("-buffer");
            suffix.push_str(&format!("-context-{}", context_id));
        }
        if options.load_typed_array {
            suffix.push_str("-typed-array");
        }

        if let Some(spz) = options.spz {
            let spz_key = Self::get_spz_cache_key(gltf, spz, gltf_resource, base_resource)?;
            let inner_key = spz_key.strip_prefix("spz:").unwrap_or(&spz_key);
            let semantic = options.attribute_semantic.unwrap_or("");
            return Ok(format!(
                "vertex-buffer:{inner_key}-spz-{semantic}{suffix}"
            ));
        }

        if let Some(draco) = options.draco {
            let draco_key = Self::get_draco_cache_key(gltf, draco, gltf_resource, base_resource)?;
            let inner_key = draco_key.strip_prefix("draco:").unwrap_or(&draco_key);
            let semantic = options.attribute_semantic.unwrap_or("");
            return Ok(format!(
                "vertex-buffer:{inner_key}-draco-{semantic}{suffix}"
            ));
        }

        let bv_id = options.buffer_view_id.unwrap();
        let bv_key =
            Self::get_buffer_view_cache_key(gltf, bv_id, gltf_resource, base_resource)?;
        let stripped = bv_key.strip_prefix("buffer-view:").unwrap_or(&bv_key);
        Ok(format!("vertex-buffer:{stripped}{suffix}"))
    }

    /// Index buffer cache key.
    pub fn get_index_buffer_cache_key(
        options: IndexBufferCacheKeyOptions,
    ) -> Result<String, DeveloperError> {
        let gltf = options
            .gltf
            .ok_or_else(|| DeveloperError::new("options.gltf is required"))?;
        let gltf_resource = options
            .gltf_resource
            .ok_or_else(|| DeveloperError::new("options.gltfResource is required"))?;
        let base_resource = options
            .base_resource
            .ok_or_else(|| DeveloperError::new("options.baseResource is required"))?;
        let context_id = options
            .context_id
            .ok_or_else(|| DeveloperError::new("options.frameState is required"))?;

        if !options.load_buffer && !options.load_typed_array {
            return Err(DeveloperError::new(
                "At least one of loadBuffer and loadTypedArray must be true.",
            ));
        }

        let mut suffix = String::new();
        if options.load_buffer {
            suffix.push_str("-buffer");
            suffix.push_str(&format!("-context-{}", context_id));
        }
        if options.load_typed_array {
            suffix.push_str("-typed-array");
        }

        if let Some(draco) = options.draco {
            let draco_key = Self::get_draco_cache_key(gltf, draco, gltf_resource, base_resource)?;
            let inner_key = draco_key.strip_prefix("draco:").unwrap_or(&draco_key);
            return Ok(format!("index-buffer:{inner_key}-draco{suffix}"));
        }

        let accessor_id = options
            .accessor_id
            .ok_or_else(|| DeveloperError::new("options.accessorId is required"))?;
        let accessor = gltf
            .get("accessors")
            .and_then(|a| a.get(accessor_id))
            .ok_or_else(|| {
                DeveloperError::new(format!("accessor ID {accessor_id} not found in glTF"))
            })?;

        let bv_id = accessor
            .get("bufferView")
            .and_then(Value::as_u64)
            .ok_or_else(|| DeveloperError::new("accessor.bufferView is required"))?
            as usize;

        let buffer_view = gltf
            .get("bufferViews")
            .and_then(|bv| bv.get(bv_id))
            .ok_or_else(|| DeveloperError::new("bufferView not found"))?;

        let buffer_id = buffer_view
            .get("buffer")
            .and_then(Value::as_u64)
            .unwrap_or(0) as usize;

        let buffer_cache_key =
            Self::get_buffer_cache_key(gltf, buffer_id, gltf_resource, base_resource)?;

        let bv_byte_offset = buffer_view
            .get("byteOffset")
            .and_then(Value::as_u64)
            .unwrap_or(0) as usize;
        let acc_byte_offset = accessor
            .get("byteOffset")
            .and_then(Value::as_u64)
            .unwrap_or(0) as usize;
        let total_offset = bv_byte_offset + acc_byte_offset;

        let comp_type = accessor.get("componentType").and_then(Value::as_u64).unwrap_or(0);
        let acc_type = accessor.get("type").and_then(Value::as_str).unwrap_or("SCALAR");
        let count = accessor.get("count").and_then(Value::as_u64).unwrap_or(0);

        let accessor_key = format!("{total_offset}-{comp_type}-{acc_type}-{count}");
        Ok(format!(
            "index-buffer:{buffer_cache_key}-accessor-{accessor_key}{suffix}"
        ))
    }

    /// Image cache key.
    pub fn get_image_cache_key(
        gltf: &Value,
        image_id: usize,
        gltf_resource: &str,
        base_resource: &str,
    ) -> Result<String, DeveloperError> {
        let image = gltf
            .get("images")
            .and_then(|imgs| imgs.get(image_id))
            .ok_or_else(|| DeveloperError::new(format!("image ID {image_id} not found in glTF")))?;

        if let Some(uri) = image.get("uri").and_then(Value::as_str) {
            let resolved = Self::resolve_uri(base_resource, uri);
            Ok(format!("image:{resolved}"))
        } else if let Some(bv_id) = image.get("bufferView").and_then(Value::as_u64) {
            let bv_key = Self::get_buffer_view_cache_key(
                gltf,
                bv_id as usize,
                gltf_resource,
                base_resource,
            )?;
            let stripped = bv_key.strip_prefix("buffer-view:").unwrap_or(&bv_key);
            Ok(format!("image:{stripped}"))
        } else {
            Err(DeveloperError::new("Image has neither uri nor bufferView"))
        }
    }

    /// Texture cache key.
    pub fn get_texture_cache_key(
        options: TextureCacheKeyOptions,
    ) -> Result<String, DeveloperError> {
        let gltf = options
            .gltf
            .ok_or_else(|| DeveloperError::new("options.gltf is required"))?;
        let texture_info = options
            .texture_info
            .ok_or_else(|| DeveloperError::new("options.textureInfo is required"))?;
        let gltf_resource = options
            .gltf_resource
            .ok_or_else(|| DeveloperError::new("options.gltfResource is required"))?;
        let base_resource = options
            .base_resource
            .ok_or_else(|| DeveloperError::new("options.baseResource is required"))?;
        let context_id = options
            .context_id
            .ok_or_else(|| DeveloperError::new("options.frameState is required"))?;

        let texture_id = texture_info
            .get("index")
            .and_then(Value::as_u64)
            .unwrap_or(0) as usize;

        let texture = gltf
            .get("textures")
            .and_then(|t| t.get(texture_id))
            .ok_or_else(|| DeveloperError::new(format!("texture ID {texture_id} not found")))?;

        // Image selection
        let mut image_id = None;
        if let Some(ext) = texture.get("extensions") {
            if options.supported_image_formats.webp {
                if let Some(webp) = ext
                    .get("EXT_texture_webp")
                    .and_then(|w| w.get("source"))
                    .and_then(Value::as_u64)
                {
                    image_id = Some(webp as usize);
                }
            } else if options.supported_image_formats.basis {
                if let Some(basisu) = ext
                    .get("KHR_texture_basisu")
                    .and_then(|b| b.get("source"))
                    .and_then(Value::as_u64)
                {
                    image_id = Some(basisu as usize);
                }
            }
        }

        if image_id.is_none() {
            image_id = texture.get("source").and_then(Value::as_u64).map(|id| id as usize);
        }

        let img_id = image_id.ok_or_else(|| DeveloperError::new("Texture source image not found"))?;
        let image_key =
            Self::get_image_cache_key(gltf, img_id, gltf_resource, base_resource)?;
        let stripped_image_key = image_key.strip_prefix("image:").unwrap_or(&image_key);

        let sampler_key = Self::get_sampler_cache_key(gltf, texture);

        Ok(format!(
            "texture:{stripped_image_key}-sampler-{sampler_key}-context-{context_id}"
        ))
    }

    fn get_sampler_cache_key(gltf: &Value, texture: &Value) -> String {
        let sampler_id = texture.get("sampler").and_then(Value::as_u64);
        if let Some(id) = sampler_id {
            if let Some(sampler) = gltf.get("samplers").and_then(|s| s.get(id as usize)) {
                let wrap_s = sampler.get("wrapS").and_then(Value::as_u64).unwrap_or(10497);
                let wrap_t = sampler.get("wrapT").and_then(Value::as_u64).unwrap_or(10497);
                let min_filter = sampler.get("minFilter").and_then(Value::as_u64).unwrap_or(9729);
                let mag_filter = sampler.get("magFilter").and_then(Value::as_u64).unwrap_or(9729);
                return format!("{wrap_s}-{wrap_t}-{min_filter}-{mag_filter}");
            }
        }
        "10497-10497-9729-9729".to_string()
    }

    fn resolve_uri(base: &str, relative: &str) -> String {
        if relative.starts_with("http://")
            || relative.starts_with("https://")
            || relative.starts_with("data:")
            || relative.starts_with('/')
        {
            return relative.to_string();
        }

        if let Some(idx) = base.rfind('/') {
            format!("{}{}", &base[..=idx], relative)
        } else {
            format!("{base}/{relative}")
        }
    }
}
