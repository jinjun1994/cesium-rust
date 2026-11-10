//! GltfLoaderUtil matching CesiumJS `Scene/GltfLoaderUtil.js` (Apache-2.0).

use crate::gltf_texture_loader::SupportedImageFormats;
use crate::sampler::{
    Sampler, TextureMagnificationFilter, TextureMinificationFilter, TextureWrap,
};
use crate::texture::Texture;
use cesium_math::{Cartesian2, DeveloperError, Matrix3};
use serde_json::Value;

/// Constant LOD options matching CesiumJS `EXT_textureInfo_constant_lod`.
#[derive(Debug, Clone, PartialEq)]
pub struct ConstantLod {
    pub min_clamp_distance: f32,
    pub max_clamp_distance: f32,
    pub repetitions: f32,
    pub offset: Cartesian2,
}

/// Options for [`GltfLoaderUtil::get_image_id_from_texture`].
#[derive(Debug, Clone, Default)]
pub struct GetImageIdFromTextureOptions<'a> {
    pub gltf: Option<&'a Value>,
    pub texture_id: Option<usize>,
    pub supported_image_formats: Option<SupportedImageFormats>,
}

/// Options for [`GltfLoaderUtil::create_sampler`].
#[derive(Debug, Clone, Default)]
pub struct CreateSamplerOptions<'a> {
    pub gltf: Option<&'a Value>,
    pub texture_info: Option<&'a Value>,
    pub compressed_texture_no_mipmap: Option<bool>,
}

/// Options for [`GltfLoaderUtil::create_model_texture_reader`].
#[derive(Debug, Clone, Default)]
pub struct CreateModelTextureReaderOptions<'a> {
    pub texture_info: Option<&'a Value>,
    pub channels: Option<String>,
    pub texture: Option<Texture>,
}

/// Texture reader for models matching CesiumJS `ModelComponents.TextureReader`.
#[derive(Debug, Clone, PartialEq)]
pub struct ModelTextureReader {
    pub index: usize,
    pub texture: Option<Texture>,
    pub tex_coord: usize,
    pub scale: Option<f64>,
    pub transform: Option<Matrix3>,
    pub channels: Option<String>,
    pub constant_lod: Option<ConstantLod>,
}

/// glTF loading utilities matching CesiumJS `GltfLoaderUtil`.
pub struct GltfLoaderUtil;

impl GltfLoaderUtil {
    /// Get the image ID referenced by a texture.
    pub fn get_image_id_from_texture(
        options: GetImageIdFromTextureOptions,
    ) -> Result<usize, DeveloperError> {
        let gltf = options
            .gltf
            .ok_or_else(|| DeveloperError::new("options.gltf is required."))?;
        let texture_id = options
            .texture_id
            .ok_or_else(|| DeveloperError::new("options.textureId is required."))?;
        let supported_image_formats = options
            .supported_image_formats
            .ok_or_else(|| DeveloperError::new("options.supportedImageFormats is required."))?;

        let texture = gltf
            .get("textures")
            .and_then(|t| t.get(texture_id))
            .ok_or_else(|| DeveloperError::new(format!("Texture index {texture_id} not found.")))?;

        let extensions = texture.get("extensions");

        if let Some(ext) = extensions {
            if supported_image_formats.webp {
                if let Some(src) = ext
                    .get("EXT_texture_webp")
                    .and_then(|w| w.get("source"))
                    .and_then(Value::as_u64)
                {
                    return Ok(src as usize);
                }
            }

            if supported_image_formats.basis {
                if let Some(src) = ext
                    .get("KHR_texture_basisu")
                    .and_then(|b| b.get("source"))
                    .and_then(Value::as_u64)
                {
                    return Ok(src as usize);
                }
            }
        }

        let source = texture
            .get("source")
            .and_then(Value::as_u64)
            .ok_or_else(|| DeveloperError::new("texture.source is required."))?;

        Ok(source as usize)
    }

    /// Create a sampler for a texture.
    pub fn create_sampler(options: CreateSamplerOptions) -> Result<Sampler, DeveloperError> {
        let gltf = options
            .gltf
            .ok_or_else(|| DeveloperError::new("options.gltf is required."))?;
        let texture_info = options
            .texture_info
            .ok_or_else(|| DeveloperError::new("options.textureInfo is required."))?;
        let compressed_texture_no_mipmap = options.compressed_texture_no_mipmap.unwrap_or(false);

        let texture_id = texture_info
            .get("index")
            .and_then(Value::as_u64)
            .ok_or_else(|| DeveloperError::new("textureInfo.index is required."))?
            as usize;

        let texture = gltf
            .get("textures")
            .and_then(|t| t.get(texture_id))
            .ok_or_else(|| DeveloperError::new(format!("Texture index {texture_id} not found.")))?;

        let mut wrap_s = TextureWrap::Repeat;
        let mut wrap_t = TextureWrap::Repeat;
        let mut min_filter = TextureMinificationFilter::Linear;
        let mut mag_filter = TextureMagnificationFilter::Linear;

        if let Some(sampler_id) = texture.get("sampler").and_then(Value::as_u64) {
            if let Some(sampler) = gltf.get("samplers").and_then(|s| s.get(sampler_id as usize)) {
                if let Some(ws) = sampler
                    .get("wrapS")
                    .and_then(Value::as_i64)
                    .map(|v| v as u32)
                    .and_then(TextureWrap::from_gl_enum)
                {
                    wrap_s = ws;
                }
                if let Some(wt) = sampler
                    .get("wrapT")
                    .and_then(Value::as_i64)
                    .map(|v| v as u32)
                    .and_then(TextureWrap::from_gl_enum)
                {
                    wrap_t = wt;
                }
                if let Some(min_f) = sampler
                    .get("minFilter")
                    .and_then(Value::as_i64)
                    .map(|v| v as u32)
                    .and_then(TextureMinificationFilter::from_gl_enum)
                {
                    min_filter = min_f;
                }
                if let Some(mag_f) = sampler
                    .get("magFilter")
                    .and_then(Value::as_i64)
                    .map(|v| v as u32)
                    .and_then(TextureMagnificationFilter::from_gl_enum)
                {
                    mag_filter = mag_f;
                }
            }
        }

        if compressed_texture_no_mipmap && min_filter.requires_mipmap() {
            if min_filter == TextureMinificationFilter::NearestMipmapNearest
                || min_filter == TextureMinificationFilter::NearestMipmapLinear
            {
                min_filter = TextureMinificationFilter::Nearest;
            } else {
                min_filter = TextureMinificationFilter::Linear;
            }
        }

        Ok(Sampler {
            wrap_r: TextureWrap::ClampToEdge,
            wrap_s,
            wrap_t,
            minification_filter: min_filter,
            magnification_filter: mag_filter,
            maximum_anisotropy: 1.0,
        })
    }

    /// Create a model texture reader from texture info.
    pub fn create_model_texture_reader(
        options: CreateModelTextureReaderOptions,
    ) -> Result<ModelTextureReader, DeveloperError> {
        let texture_info = options
            .texture_info
            .ok_or_else(|| DeveloperError::new("options.textureInfo is required."))?;

        let index = texture_info
            .get("index")
            .and_then(Value::as_u64)
            .unwrap_or(0) as usize;

        let mut tex_coord = texture_info
            .get("texCoord")
            .and_then(Value::as_u64)
            .unwrap_or(0) as usize;

        let scale = texture_info.get("scale").and_then(Value::as_f64);

        let mut transform = None;
        if let Some(ext) = texture_info
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

            let rotation = ext.get("rotation").and_then(Value::as_f64).unwrap_or(0.0);

            // glTF coordinates start top-left (y-down) while WebGL starts bottom-left (y-up).
            // Rotation is reversed.
            let r = -rotation;
            let cos_r = r.cos();
            let sin_r = r.sin();

            // Row-major constructor for Matrix3:
            // [
            //   cos_r * scale_x, -sin_r * scale_y, offset_x,
            //   sin_r * scale_x,  cos_r * scale_y, offset_y,
            //   0.0,              0.0,             1.0
            // ]
            transform = Some(Matrix3::new(
                cos_r * scale_x,
                -sin_r * scale_y,
                offset_x,
                sin_r * scale_x,
                cos_r * scale_y,
                offset_y,
                0.0,
                0.0,
                1.0,
            ));
        }

        let mut constant_lod = None;
        if let Some(ext) = texture_info.get("extensions") {
            if let Some(lod_ext) = ext.get("EXT_textureInfo_constant_lod") {
                let min_clamp_distance = lod_ext
                    .get("minClampDistance")
                    .and_then(Value::as_f64)
                    .unwrap_or(0.0) as f32;
                let max_clamp_distance = lod_ext
                    .get("maxClampDistance")
                    .and_then(Value::as_f64)
                    .unwrap_or(1000.0) as f32;
                let repetitions = lod_ext
                    .get("repetitions")
                    .and_then(Value::as_f64)
                    .unwrap_or(1.0) as f32;
                let mut offset = Cartesian2::ZERO;
                if let Some(offset_arr) = lod_ext.get("offset").and_then(Value::as_array) {
                    if offset_arr.len() == 2 {
                        offset = Cartesian2::new(
                            offset_arr[0].as_f64().unwrap_or(0.0),
                            offset_arr[1].as_f64().unwrap_or(0.0),
                        );
                    }
                }
                constant_lod = Some(ConstantLod {
                    min_clamp_distance,
                    max_clamp_distance,
                    repetitions,
                    offset,
                });
            }
        }

        Ok(ModelTextureReader {
            index,
            texture: options.texture,
            tex_coord,
            scale,
            transform,
            channels: options.channels,
            constant_lod,
        })
    }
}
