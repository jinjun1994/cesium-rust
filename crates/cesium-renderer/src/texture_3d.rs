//! 3D GPU Texture matching CesiumJS `Renderer/Texture3D.js` (Apache-2.0).

use std::sync::atomic::{AtomicU64, Ordering};
use cesium_math::DeveloperError;
use crate::framebuffer::{PixelDatatype, PixelFormat};
use crate::sampler::Sampler;

static NEXT_TEXTURE3D_ID: AtomicU64 = AtomicU64::new(1);

impl PixelDatatype {
    pub fn size_in_bytes(&self) -> usize {
        match self {
            Self::UnsignedByte => 1,
            Self::UnsignedShort | Self::HalfFloat => 2,
            Self::UnsignedInt | Self::Float | Self::UnsignedInt24_8 => 4,
        }
    }
}

/// A 3D GPU texture for volumetric rendering.
#[derive(Clone, Debug, PartialEq)]
pub struct Texture3D {
    pub id: u64,
    pub width: u32,
    pub height: u32,
    pub depth: u32,
    pub pixel_format: PixelFormat,
    pub pixel_datatype: PixelDatatype,
    pub sampler: Sampler,
    pub is_destroyed: bool,
}

#[derive(Clone, Debug)]
pub struct Texture3DOptions {
    pub width: u32,
    pub height: u32,
    pub depth: u32,
    pub pixel_format: Option<PixelFormat>,
    pub pixel_datatype: Option<PixelDatatype>,
    pub sampler: Option<Sampler>,
}

impl Texture3D {
    pub fn new(options: Texture3DOptions) -> Result<Self, DeveloperError> {
        if options.width == 0 || options.height == 0 || options.depth == 0 {
            return Err(DeveloperError::new(
                "width, height, and depth must be greater than zero.",
            ));
        }

        let id = NEXT_TEXTURE3D_ID.fetch_add(1, Ordering::Relaxed);
        Ok(Self {
            id,
            width: options.width,
            height: options.height,
            depth: options.depth,
            pixel_format: options.pixel_format.unwrap_or(PixelFormat::Rgba),
            pixel_datatype: options.pixel_datatype.unwrap_or(PixelDatatype::UnsignedByte),
            sampler: options.sampler.unwrap_or_default(),
            is_destroyed: false,
        })
    }

    pub fn size_in_bytes(&self) -> usize {
        let volume = (self.width as usize) * (self.height as usize) * (self.depth as usize);
        if self.pixel_format == PixelFormat::DepthStencil
            && self.pixel_datatype == PixelDatatype::UnsignedInt24_8
        {
            // DEPTH_STENCIL packed in a 32-bit uint is 4 bytes per voxel
            volume * 4
        } else if self.pixel_format == PixelFormat::DepthComponent {
            volume * self.pixel_datatype.size_in_bytes()
        } else {
            volume * self.pixel_format.components_length() * self.pixel_datatype.size_in_bytes()
        }
    }

    pub fn is_destroyed(&self) -> bool {
        self.is_destroyed
    }

    pub fn destroy(&mut self) {
        self.is_destroyed = true;
    }
}
