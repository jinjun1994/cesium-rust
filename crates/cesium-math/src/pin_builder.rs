use std::collections::HashMap;
use std::sync::Arc;

use crate::color::Color;
use crate::errors::DeveloperError;

/// A raster image representing a generated map pin.
#[derive(Debug, Clone, PartialEq)]
pub struct PinImage {
    pub width: u32,
    pub height: u32,
    pub data: Vec<u8>,
}

impl PinImage {
    pub fn new(width: u32, height: u32, data: Vec<u8>) -> Self {
        Self { width, height, data }
    }

    pub fn get_pixel(&self, x: u32, y: u32) -> Color {
        if x >= self.width || y >= self.height {
            return Color::BLACK;
        }
        let idx = ((y * self.width + x) * 4) as usize;
        if idx + 3 < self.data.len() {
            Color::from_bytes(
                self.data[idx],
                self.data[idx + 1],
                self.data[idx + 2],
                self.data[idx + 3],
            )
        } else {
            Color::BLACK
        }
    }
}

/// A utility class for generating custom map pins as raster image elements.
#[derive(Debug, Default)]
pub struct PinBuilder {
    cache: HashMap<String, Arc<PinImage>>,
}

impl PinBuilder {
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
        }
    }

    pub fn from_color(
        &mut self,
        color: Option<Color>,
        size: Option<u32>,
    ) -> Result<Arc<PinImage>, DeveloperError> {
        let color = color.ok_or_else(|| DeveloperError::new("color is required"))?;
        let size = size.ok_or_else(|| DeveloperError::new("size is required"))?;

        let key = format!("color_{}_{}", color.to_rgba(), size);
        if let Some(img) = self.cache.get(&key) {
            return Ok(img.clone());
        }

        let img = Arc::new(generate_pin(None, color, size));
        self.cache.insert(key, img.clone());
        Ok(img)
    }

    pub fn from_text(
        &mut self,
        text: Option<&str>,
        color: Option<Color>,
        size: Option<u32>,
    ) -> Result<Arc<PinImage>, DeveloperError> {
        let text = text.ok_or_else(|| DeveloperError::new("text is required"))?;
        let color = color.ok_or_else(|| DeveloperError::new("color is required"))?;
        let size = size.ok_or_else(|| DeveloperError::new("size is required"))?;

        let key = format!("text_{}_{}_{}", text, color.to_rgba(), size);
        if let Some(img) = self.cache.get(&key) {
            return Ok(img.clone());
        }

        let img = Arc::new(generate_pin(Some(text), color, size));
        self.cache.insert(key, img.clone());
        Ok(img)
    }

    pub fn from_url(
        &mut self,
        url: Option<&str>,
        color: Option<Color>,
        size: Option<u32>,
    ) -> Result<Arc<PinImage>, DeveloperError> {
        let url = url.ok_or_else(|| DeveloperError::new("url is required"))?;
        let color = color.ok_or_else(|| DeveloperError::new("color is required"))?;
        let size = size.ok_or_else(|| DeveloperError::new("size is required"))?;

        let key = format!("url_{}_{}_{}", url, color.to_rgba(), size);
        if let Some(img) = self.cache.get(&key) {
            return Ok(img.clone());
        }

        let img = Arc::new(generate_pin(Some("icon"), color, size));
        self.cache.insert(key, img.clone());
        Ok(img)
    }

    pub fn from_maki_icon_id(
        &mut self,
        id: Option<&str>,
        color: Option<Color>,
        size: Option<u32>,
    ) -> Result<Arc<PinImage>, DeveloperError> {
        let id = id.ok_or_else(|| DeveloperError::new("id is required"))?;
        let color = color.ok_or_else(|| DeveloperError::new("color is required"))?;
        let size = size.ok_or_else(|| DeveloperError::new("size is required"))?;

        let key = format!("maki_{}_{}_{}", id, color.to_rgba(), size);
        if let Some(img) = self.cache.get(&key) {
            return Ok(img.clone());
        }

        let img = Arc::new(generate_pin(Some(id), color, size));
        self.cache.insert(key, img.clone());
        Ok(img)
    }
}

fn generate_pin(symbol: Option<&str>, color: Color, size: u32) -> PinImage {
    let width = size;
    let height = size;
    let mut data = vec![0u8; (width * height * 4) as usize];

    let bytes = color.to_bytes();

    // Fill pin background area
    for y in 0..height {
        for x in 0..width {
            let idx = ((y * width + x) * 4) as usize;
            data[idx] = bytes[0];
            data[idx + 1] = bytes[1];
            data[idx + 2] = bytes[2];
            data[idx + 3] = bytes[3];
        }
    }

    // If symbol exists, stamp white icon in central box
    if symbol.is_some() {
        let icon_box_size = (size as f64 / 2.5) as u32;
        let start_x = (size.saturating_sub(icon_box_size)) / 2;
        let start_y = (size as f64 * 0.15) as u32;
        let end_x = (start_x + icon_box_size).min(width);
        let end_y = (start_y + icon_box_size).min(height);

        for y in start_y..end_y {
            for x in start_x..end_x {
                let idx = ((y * width + x) * 4) as usize;
                data[idx] = 255;
                data[idx + 1] = 255;
                data[idx + 2] = 255;
                data[idx + 3] = 255;
            }
        }
    }

    PinImage::new(width, height, data)
}
