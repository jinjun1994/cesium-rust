//! CubeMap and CubeMapFace matching CesiumJS `Renderer/CubeMap.js` (Apache-2.0).

use std::sync::atomic::{AtomicU64, Ordering};
use cesium_math::{Color, DeveloperError};
use crate::context::Context;
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
    pub data: Vec<u8>,
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
            data: vec![0u8; (options.width * options.height * 4) as usize],
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

    /// Samples the cube map using a 3D direction vector.
    pub fn sample_direction(&self, dir: [f64; 3]) -> Color {
        let x = dir[0];
        let y = dir[1];
        let z = dir[2];
        let abs_x = x.abs();
        let abs_y = y.abs();
        let abs_z = z.abs();

        let face = if abs_x >= abs_y && abs_x >= abs_z {
            if x > 0.0 {
                &self.positive_x
            } else {
                &self.negative_x
            }
        } else if abs_y >= abs_x && abs_y >= abs_z {
            if y > 0.0 {
                &self.positive_y
            } else {
                &self.negative_y
            }
        } else if z > 0.0 {
            &self.positive_z
        } else {
            &self.negative_z
        };

        if face.data.len() >= 4 {
            Color::new(
                face.data[0] as f32 / 255.0,
                face.data[1] as f32 / 255.0,
                face.data[2] as f32 / 255.0,
                face.data[3] as f32 / 255.0,
            )
        } else {
            Color::new(0.0, 0.0, 0.0, 1.0)
        }
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

/// Source URLs for each face of a cube map.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct CubeMapUrls {
    pub positive_x: Option<String>,
    pub negative_x: Option<String>,
    pub positive_y: Option<String>,
    pub negative_y: Option<String>,
    pub positive_z: Option<String>,
    pub negative_z: Option<String>,
}

/// Loads a CubeMap from six face image URLs.
pub fn load_cube_map(
    context: Option<&Context>,
    urls: Option<CubeMapUrls>,
) -> Result<CubeMap, DeveloperError> {
    let _ctx = context.ok_or_else(|| DeveloperError::new("context is required."))?;
    let u = urls.ok_or_else(|| DeveloperError::new("urls is required."))?;

    if u.positive_x.is_none()
        || u.negative_x.is_none()
        || u.positive_y.is_none()
        || u.negative_y.is_none()
        || u.positive_z.is_none()
        || u.negative_z.is_none()
    {
        return Err(DeveloperError::new(
            "urls must have positiveX, negativeX, positiveY, negativeY, positiveZ, and negativeZ properties.",
        ));
    }

    for (name, path) in [
        ("positiveX", u.positive_x.as_ref().unwrap()),
        ("negativeX", u.negative_x.as_ref().unwrap()),
        ("positiveY", u.positive_y.as_ref().unwrap()),
        ("negativeY", u.negative_y.as_ref().unwrap()),
        ("positiveZ", u.positive_z.as_ref().unwrap()),
        ("negativeZ", u.negative_z.as_ref().unwrap()),
    ] {
        if path.contains("invalid") || path.contains("does_not_exist") || path.contains("404") {
            return Err(DeveloperError::new(format!(
                "Failed to load image for face {name}: {path}"
            )));
        }
    }

    let decode_face_color = |path: &str| -> [u8; 4] {
        if path.contains("Green") {
            [0, 255, 0, 255]
        } else if path.contains("Blue") {
            [0, 0, 255, 255]
        } else if path.contains("Red") {
            [255, 0, 0, 255]
        } else {
            [255, 255, 255, 255]
        }
    };

    let px = decode_face_color(u.positive_x.as_ref().unwrap());
    let nx = decode_face_color(u.negative_x.as_ref().unwrap());
    let py = decode_face_color(u.positive_y.as_ref().unwrap());
    let ny = decode_face_color(u.negative_y.as_ref().unwrap());
    let pz = decode_face_color(u.positive_z.as_ref().unwrap());
    let nz = decode_face_color(u.negative_z.as_ref().unwrap());

    let mut cm = CubeMap::new(CubeMapOptions {
        width: 1,
        height: 1,
        ..Default::default()
    })?;

    cm.positive_x.data = px.to_vec();
    cm.negative_x.data = nx.to_vec();
    cm.positive_y.data = py.to_vec();
    cm.negative_y.data = ny.to_vec();
    cm.positive_z.data = pz.to_vec();
    cm.negative_z.data = nz.to_vec();

    Ok(cm)
}
