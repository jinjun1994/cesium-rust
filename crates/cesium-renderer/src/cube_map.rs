//! CubeMap and CubeMapFace matching CesiumJS `Renderer/CubeMap.js` (Apache-2.0).

use std::sync::atomic::{AtomicU64, Ordering};
use cesium_math::DeveloperError;
use crate::framebuffer::{PixelDatatype, PixelFormat};
use crate::sampler::Sampler;

static NEXT_CUBEMAP_ID: AtomicU64 = AtomicU64::new(1);

/// A single face of a CubeMap (+X, -X, +Y, -Y, +Z, -Z).
#[derive(Clone, Debug, PartialEq)]
pub struct CubeMapFace {
    pub pixel_format: PixelFormat,
    pub pixel_datatype: PixelDatatype,
    pub width: u32,
    pub height: u32,
}

/// A 6-sided GPU cube texture.
#[derive(Clone, Debug, PartialEq)]
pub struct CubeMap {
    pub id: u64,
    pub width: u32,
    pub height: u32,
    pub pixel_format: PixelFormat,
    pub pixel_datatype: PixelDatatype,
    pub sampler: Sampler,
    pub flip_y: bool,

    pub positive_x: CubeMapFace,
    pub negative_x: CubeMapFace,
    pub positive_y: CubeMapFace,
    pub negative_y: CubeMapFace,
    pub positive_z: CubeMapFace,
    pub negative_z: CubeMapFace,

    pub is_destroyed: bool,
}

#[derive(Clone, Debug)]
pub struct CubeMapOptions {
    pub width: u32,
    pub height: u32,
    pub pixel_format: Option<PixelFormat>,
    pub pixel_datatype: Option<PixelDatatype>,
    pub sampler: Option<Sampler>,
    pub flip_y: Option<bool>,
}

impl Default for CubeMapOptions {
    fn default() -> Self {
        Self {
            width: 16,
            height: 16,
            pixel_format: Some(PixelFormat::Rgba),
            pixel_datatype: Some(PixelDatatype::UnsignedByte),
            sampler: None,
            flip_y: Some(false),
        }
    }
}

impl CubeMap {
    pub fn new(options: CubeMapOptions) -> Result<Self, DeveloperError> {
        if options.width == 0 || options.height == 0 {
            return Err(DeveloperError::new(
                "width and height must be greater than zero.",
            ));
        }

        let pixel_format = options.pixel_format.unwrap_or(PixelFormat::Rgba);
        let pixel_datatype = options.pixel_datatype.unwrap_or(PixelDatatype::UnsignedByte);

        let make_face = || CubeMapFace {
            pixel_format,
            pixel_datatype,
            width: options.width,
            height: options.height,
        };

        let id = NEXT_CUBEMAP_ID.fetch_add(1, Ordering::Relaxed);
        Ok(Self {
            id,
            width: options.width,
            height: options.height,
            pixel_format,
            pixel_datatype,
            sampler: options.sampler.unwrap_or_default(),
            flip_y: options.flip_y.unwrap_or(false),
            positive_x: make_face(),
            negative_x: make_face(),
            positive_y: make_face(),
            negative_y: make_face(),
            positive_z: make_face(),
            negative_z: make_face(),
            is_destroyed: false,
        })
    }

    pub fn size_in_bytes(&self) -> usize {
        (self.width as usize)
            * (self.height as usize)
            * self.pixel_format.components_length()
            * self.pixel_datatype.size_in_bytes()
            * 6
    }

    pub fn is_destroyed(&self) -> bool {
        self.is_destroyed
    }

    pub fn destroy(&mut self) {
        self.is_destroyed = true;
    }
}
