//! BatchTexture matching CesiumJS `Scene/BatchTexture.js` (Apache-2.0).

use cesium_math::{Color, DeveloperError};

/// Options for constructing a [`BatchTexture`].
#[derive(Clone, Debug)]
pub struct BatchTextureOptions {
    pub features_length: usize,
    pub owner_id: String,
    pub color_table: Option<Vec<Color>>,
}

/// Manages per-feature visibility, color, and translucent counts for 3D Tiles features.
#[derive(Clone, Debug)]
pub struct BatchTexture {
    features_length: usize,
    owner_id: String,
    translucent_features_length: usize,
    batch_values: Option<Vec<u8>>,
    batch_values_dirty: bool,
    has_texture: bool,
    /// Per-feature show flags (stored separately so alpha=0 doesn't lose show vs color).
    shows: Option<Vec<bool>>,
    destroyed: bool,
}

impl BatchTexture {
    pub fn new(options: BatchTextureOptions) -> Result<Self, DeveloperError> {
        if options.features_length == 0 {
            return Err(DeveloperError::new(
                "options.featuresLength must be greater than zero.",
            ));
        }
        if options.owner_id.trim().is_empty() {
            return Err(DeveloperError::new("options.owner is required."));
        }

        Ok(Self {
            features_length: options.features_length,
            owner_id: options.owner_id,
            translucent_features_length: 0,
            batch_values: None,
            batch_values_dirty: false,
            has_texture: false,
            shows: None,
            destroyed: false,
        })
    }

    pub fn features_length(&self) -> usize {
        self.features_length
    }

    pub fn owner_id(&self) -> &str {
        &self.owner_id
    }

    pub fn translucent_features_length(&self) -> usize {
        self.translucent_features_length
    }

    pub fn has_batch_values(&self) -> bool {
        self.batch_values.is_some()
    }

    pub fn has_batch_texture(&self) -> bool {
        self.has_texture
    }

    pub fn is_batch_values_dirty(&self) -> bool {
        self.batch_values_dirty
    }

    pub fn is_destroyed(&self) -> bool {
        self.destroyed
    }

    pub fn destroy(&mut self) {
        self.batch_values = None;
        self.shows = None;
        self.has_texture = false;
        self.destroyed = true;
    }

    fn validate_batch_id(&self, batch_id: usize) -> Result<(), DeveloperError> {
        if batch_id >= self.features_length {
            return Err(DeveloperError::new(format!(
                "batchId ({}) must be between 0 and {}.",
                batch_id,
                self.features_length - 1
            )));
        }
        Ok(())
    }

    fn ensure_allocated(&mut self) {
        if self.batch_values.is_none() {
            self.batch_values = Some(vec![255u8; self.features_length * 4]);
            self.shows = Some(vec![true; self.features_length]);
        }
    }

    pub fn get_show(&self, batch_id: usize) -> Result<bool, DeveloperError> {
        self.validate_batch_id(batch_id)?;
        Ok(self
            .shows
            .as_ref()
            .map(|s| s[batch_id])
            .unwrap_or(true))
    }

    pub fn set_show(&mut self, batch_id: usize, show: bool) -> Result<(), DeveloperError> {
        self.validate_batch_id(batch_id)?;
        if show && self.batch_values.is_none() {
            return Ok(());
        }
        self.ensure_allocated();
        let shows = self.shows.as_mut().unwrap();
        if shows[batch_id] != show {
            shows[batch_id] = show;
            self.batch_values_dirty = true;
        }
        Ok(())
    }

    pub fn set_all_show(&mut self, show: bool) {
        if show && self.batch_values.is_none() {
            return;
        }
        self.ensure_allocated();
        let shows = self.shows.as_mut().unwrap();
        for s in shows.iter_mut() {
            if *s != show {
                *s = show;
                self.batch_values_dirty = true;
            }
        }
    }

    pub fn get_color(&self, batch_id: usize) -> Result<Color, DeveloperError> {
        self.validate_batch_id(batch_id)?;
        if let Some(ref vals) = self.batch_values {
            let base = batch_id * 4;
            Ok(Color::new(
                vals[base] as f32 / 255.0,
                vals[base + 1] as f32 / 255.0,
                vals[base + 2] as f32 / 255.0,
                vals[base + 3] as f32 / 255.0,
            ))
        } else {
            Ok(Color::WHITE)
        }
    }

    pub fn set_color(&mut self, batch_id: usize, color: Color) -> Result<(), DeveloperError> {
        self.validate_batch_id(batch_id)?;
        if color == Color::WHITE && self.batch_values.is_none() {
            return Ok(());
        }
        self.ensure_allocated();
        let vals = self.batch_values.as_mut().unwrap();
        let base = batch_id * 4;
        let r = (color.red.clamp(0.0, 1.0) * 255.0).round() as u8;
        let g = (color.green.clamp(0.0, 1.0) * 255.0).round() as u8;
        let b = (color.blue.clamp(0.0, 1.0) * 255.0).round() as u8;
        let a = (color.alpha.clamp(0.0, 1.0) * 255.0).round() as u8;

        let was_translucent = vals[base + 3] > 0 && vals[base + 3] < 255;
        let is_translucent = a > 0 && a < 255;
        if was_translucent != is_translucent {
            if is_translucent {
                self.translucent_features_length += 1;
            } else {
                self.translucent_features_length = self.translucent_features_length.saturating_sub(1);
            }
        }

        if vals[base] != r || vals[base + 1] != g || vals[base + 2] != b || vals[base + 3] != a {
            vals[base] = r;
            vals[base + 1] = g;
            vals[base + 2] = b;
            vals[base + 3] = a;
            self.batch_values_dirty = true;
        }
        Ok(())
    }

    pub fn set_all_color(&mut self, color: Color) {
        if color == Color::WHITE && self.batch_values.is_none() {
            return;
        }
        for i in 0..self.features_length {
            let _ = self.set_color(i, color);
        }
    }

    /// Uploads dirty batch values into the texture snapshot.
    pub fn update(&mut self) {
        if self.batch_values.is_some() && self.batch_values_dirty {
            self.has_texture = true;
            self.batch_values_dirty = false;
        }
    }
}
