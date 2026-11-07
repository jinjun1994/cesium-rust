//! TextureAtlas matching CesiumJS `Scene/TextureAtlas.js` (Apache-2.0).

use std::collections::HashMap;
use cesium_math::{BoundingRectangle, Cartesian2, DeveloperError};
use crate::framebuffer::PixelFormat;
use crate::texture::{Texture, TextureOptions};

/// A dynamic texture atlas for packing images into a single GPU texture.
pub struct TextureAtlas {
    pub border_width_in_pixels: u32,
    pub initial_size: Cartesian2,
    pub pixel_format: PixelFormat,
    pub texture: Option<Texture>,
    pub texture_coordinates_by_index: Vec<BoundingRectangle>,
    image_indices_by_id: HashMap<String, usize>,
    image_dimensions_by_id: HashMap<String, (u32, u32)>,
    is_destroyed: bool,
}

#[derive(Clone, Debug)]
pub struct TextureAtlasOptions {
    pub border_width_in_pixels: Option<i32>,
    pub initial_size: Option<Cartesian2>,
    pub pixel_format: Option<PixelFormat>,
}

impl Default for TextureAtlasOptions {
    fn default() -> Self {
        Self {
            border_width_in_pixels: Some(1),
            initial_size: Some(Cartesian2::new(16.0, 16.0)),
            pixel_format: Some(PixelFormat::Rgba),
        }
    }
}

impl Default for TextureAtlas {
    fn default() -> Self {
        Self::new(TextureAtlasOptions::default()).unwrap()
    }
}

impl TextureAtlas {
    pub fn new(options: TextureAtlasOptions) -> Result<Self, DeveloperError> {
        let border_width = options.border_width_in_pixels.unwrap_or(1);
        if border_width < 0 {
            return Err(DeveloperError::new(
                "borderWidthInPixels must be greater than or equal to zero.",
            ));
        }

        let initial_size = options
            .initial_size
            .unwrap_or_else(|| Cartesian2::new(16.0, 16.0));
        if initial_size.x < 1.0 || initial_size.y < 1.0 {
            return Err(DeveloperError::new(
                "initialSize must be greater than or equal to [1, 1].",
            ));
        }

        let pixel_format = options.pixel_format.unwrap_or(PixelFormat::Rgba);

        let texture = Texture::new(TextureOptions {
            width: initial_size.x as u32,
            height: initial_size.y as u32,
            pixel_format: Some(pixel_format),
            ..Default::default()
        });

        Ok(Self {
            border_width_in_pixels: border_width as u32,
            initial_size,
            pixel_format,
            texture: Some(texture),
            texture_coordinates_by_index: Vec::new(),
            image_indices_by_id: HashMap::new(),
            image_dimensions_by_id: HashMap::new(),
            is_destroyed: false,
        })
    }

    pub fn number_of_images(&self) -> usize {
        self.image_indices_by_id.len()
    }

    pub fn add_image(
        &mut self,
        id: Option<&str>,
        width: u32,
        height: u32,
    ) -> Result<usize, DeveloperError> {
        let id_str = id.ok_or_else(|| DeveloperError::new("id is required."))?;
        if width == 0 || height == 0 {
            return Err(DeveloperError::new("image width and height must be > 0."));
        }

        if let Some(&index) = self.image_indices_by_id.get(id_str) {
            return Ok(index);
        }

        let index = self.texture_coordinates_by_index.len();
        self.image_indices_by_id.insert(id_str.to_string(), index);
        self.image_dimensions_by_id
            .insert(id_str.to_string(), (width, height));

        // Simplified coordinate allocation normalized [0, 1]
        let tex_coord = BoundingRectangle::new(0.0, 0.0, 1.0, 1.0);
        self.texture_coordinates_by_index.push(tex_coord);

        Ok(index)
    }

    pub fn add_image_sub_region(
        &mut self,
        id: Option<&str>,
        sub_region: Option<BoundingRectangle>,
    ) -> Result<usize, DeveloperError> {
        let id_str = id.ok_or_else(|| DeveloperError::new("id is required."))?;
        let region = sub_region.ok_or_else(|| DeveloperError::new("subRegion is required."))?;

        let _base_index = self
            .image_indices_by_id
            .get(id_str)
            .ok_or_else(|| DeveloperError::new("An image with id was not in atlas."))?;

        let sub_index = self.texture_coordinates_by_index.len();
        self.texture_coordinates_by_index.push(region);
        Ok(sub_index)
    }

    pub fn is_destroyed(&self) -> bool {
        self.is_destroyed
    }

    pub fn destroy(&mut self) {
        if let Some(ref mut tex) = self.texture {
            tex.destroy();
        }
        self.texture = None;
        self.is_destroyed = true;
    }
}
