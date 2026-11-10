//! TextureUniform matching CesiumJS `Scene/Model/TextureUniform.js` (Apache-2.0).

use cesium_math::DeveloperError;

use crate::framebuffer::{PixelDatatype, PixelFormat};
use crate::sampler::{
    Sampler, TextureMagnificationFilter, TextureMinificationFilter, TextureWrap,
};

/// Options for constructing a [`TextureUniform`].
#[derive(Clone, Debug, Default)]
pub struct TextureUniformOptions {
    pub typed_array: Option<Vec<u8>>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub url: Option<String>,
    pub repeat: Option<bool>,
    pub pixel_format: Option<PixelFormat>,
    pub pixel_datatype: Option<PixelDatatype>,
    pub minification_filter: Option<TextureMinificationFilter>,
    pub magnification_filter: Option<TextureMagnificationFilter>,
    pub maximum_anisotropy: Option<f64>,
}

/// A struct that serves as a value of a sampler2D-valued uniform.
#[derive(Clone, Debug, PartialEq)]
pub struct TextureUniform {
    pub typed_array: Option<Vec<u8>>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub url: Option<String>,
    pub pixel_format: PixelFormat,
    pub pixel_datatype: PixelDatatype,
    pub sampler: Sampler,
}

impl TextureUniform {
    /// Creates a new TextureUniform from options, validating required fields.
    pub fn new(options: TextureUniformOptions) -> Result<Self, DeveloperError> {
        let has_typed_array = options.typed_array.is_some();
        let has_url = options.url.is_some();

        if has_typed_array == has_url {
            return Err(DeveloperError::new(
                "exactly one of options.typedArray, options.url must be defined",
            ));
        }

        if has_typed_array && (options.width.is_none() || options.height.is_none()) {
            return Err(DeveloperError::new(
                "options.width and options.height are required when options.typedArray is defined",
            ));
        }

        let pixel_format = options.pixel_format.unwrap_or(PixelFormat::Rgba);
        let pixel_datatype = options.pixel_datatype.unwrap_or(PixelDatatype::UnsignedByte);

        let repeat = options.repeat.unwrap_or(true);
        let wrap = if repeat {
            TextureWrap::Repeat
        } else {
            TextureWrap::ClampToEdge
        };

        let sampler = Sampler {
            wrap_s: wrap,
            wrap_t: wrap,
            wrap_r: TextureWrap::ClampToEdge,
            minification_filter: options
                .minification_filter
                .unwrap_or(TextureMinificationFilter::Linear),
            magnification_filter: options
                .magnification_filter
                .unwrap_or(TextureMagnificationFilter::Linear),
            maximum_anisotropy: options.maximum_anisotropy.unwrap_or(1.0),
        };

        Ok(Self {
            typed_array: options.typed_array,
            width: options.width,
            height: options.height,
            url: options.url,
            pixel_format,
            pixel_datatype,
            sampler,
        })
    }

    /// Convenience constructor from URL.
    pub fn from_url(url: impl Into<String>) -> Self {
        Self::new(TextureUniformOptions {
            url: Some(url.into()),
            ..Default::default()
        })
        .expect("Valid URL constructor")
    }

    /// Convenience constructor from raw bytes.
    pub fn from_bytes(bytes: Vec<u8>, width: u32, height: u32) -> Self {
        Self::new(TextureUniformOptions {
            typed_array: Some(bytes),
            width: Some(width),
            height: Some(height),
            ..Default::default()
        })
        .expect("Valid bytes constructor")
    }
}
