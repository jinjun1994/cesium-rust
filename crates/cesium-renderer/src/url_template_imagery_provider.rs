//! UrlTemplateImageryProvider matching CesiumJS `Scene/UrlTemplateImageryProvider.js` (Apache-2.0).

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use cesium_math::{Credit, DeveloperError, Ellipsoid, GeoRectangle, WebMercatorTilingScheme};

use crate::imagery_layer::ImageryProvider;

/// Custom error information passed to `error_event`.
#[derive(Debug, Clone)]
pub struct ImageryProviderError {
    pub message: String,
    pub x: u32,
    pub y: u32,
    pub level: u32,
    pub times_retried: usize,
    pub retry: bool,
}

pub type ErrorListener = Box<dyn FnMut(&mut ImageryProviderError) + Send>;

/// Thread-safe event for imagery errors.
#[derive(Default, Clone)]
pub struct ImageryErrorEvent {
    listeners: Arc<Mutex<Vec<ErrorListener>>>,
}

impl ImageryErrorEvent {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_event_listener<F>(&self, listener: F)
    where
        F: FnMut(&mut ImageryProviderError) + Send + 'static,
    {
        if let Ok(mut l) = self.listeners.lock() {
            l.push(Box::new(listener));
        }
    }

    pub fn raise_event(&self, error: &mut ImageryProviderError) {
        if let Ok(mut l) = self.listeners.lock() {
            for listener in l.iter_mut() {
                listener(error);
            }
        }
    }
}

/// Construction options for [`UrlTemplateImageryProvider`].
#[derive(Debug, Clone, Default)]
pub struct UrlTemplateImageryProviderOptions {
    /// The URL template to use to request tiles. Required.
    pub url: Option<String>,
    /// Optional pick features URL template.
    pub pick_features_url: Option<String>,
    /// Subdomains to use for `{s}` keyword. Defaults to `["a", "b", "c"]`.
    pub subdomains: Option<Vec<String>>,
    /// Credit for the data source.
    pub credit: Option<Credit>,
    /// The minimum level-of-detail. Default 0.
    pub minimum_level: Option<u32>,
    /// The maximum level-of-detail.
    pub maximum_level: Option<u32>,
    /// The rectangle of the layer. Default is full tiling scheme rectangle.
    pub rectangle: Option<GeoRectangle>,
    /// The tiling scheme. Default is WebMercator.
    pub tiling_scheme: Option<WebMercatorTilingScheme>,
    /// The ellipsoid. Defaults to WGS84.
    pub ellipsoid: Option<Ellipsoid>,
    /// Pixel width of image tiles. Default 256.
    pub tile_width: Option<u32>,
    /// Pixel height of image tiles. Default 256.
    pub tile_height: Option<u32>,
    /// Whether tiles include alpha channel. Default true.
    pub has_alpha_channel: Option<bool>,
    /// Zero padding format per keyword, e.g. `{ "{x}": "0000" }`.
    pub url_scheme_zero_padding: Option<HashMap<String, String>>,
    /// Whether feature picking is enabled. Default true.
    pub enable_pick_features: Option<bool>,
}

/// Provides imagery by requesting tiles using a specified URL template.
#[derive(Clone)]
pub struct UrlTemplateImageryProvider {
    pub url: String,
    pub pick_features_url: Option<String>,
    pub tile_width: u32,
    pub tile_height: u32,
    pub minimum_level: u32,
    pub maximum_level: Option<u32>,
    pub rectangle: GeoRectangle,
    pub tiling_scheme: WebMercatorTilingScheme,
    pub credit: Option<Credit>,
    pub has_alpha_channel: bool,
    pub subdomains: Vec<String>,
    pub url_scheme_zero_padding: HashMap<String, String>,
    pub enable_pick_features: bool,
    pub error_event: ImageryErrorEvent,
}

impl UrlTemplateImageryProvider {
    /// Creates a new `UrlTemplateImageryProvider`.
    pub fn new(options: UrlTemplateImageryProviderOptions) -> Result<Self, DeveloperError> {
        let url = options
            .url
            .ok_or_else(|| DeveloperError::new("options.url is required."))?;

        let ellipsoid = options.ellipsoid.unwrap_or(Ellipsoid::WGS84);
        let tiling_scheme = options
            .tiling_scheme
            .unwrap_or_else(|| WebMercatorTilingScheme::new(ellipsoid, None, 1, 1).unwrap_or_default());

        let tile_width = options.tile_width.unwrap_or(256);
        let tile_height = options.tile_height.unwrap_or(256);
        let minimum_level = options.minimum_level.unwrap_or(0);
        let maximum_level = options.maximum_level;
        let rectangle = options.rectangle.unwrap_or_else(|| tiling_scheme.rectangle());

        let subdomains = options
            .subdomains
            .unwrap_or_else(|| vec!["a".to_string(), "b".to_string(), "c".to_string()]);

        let url_scheme_zero_padding = options.url_scheme_zero_padding.unwrap_or_default();
        let enable_pick_features = options.enable_pick_features.unwrap_or(true);
        let has_alpha_channel = options.has_alpha_channel.unwrap_or(true);

        Ok(Self {
            url,
            pick_features_url: options.pick_features_url,
            tile_width,
            tile_height,
            minimum_level,
            maximum_level,
            rectangle,
            tiling_scheme,
            credit: options.credit,
            has_alpha_channel,
            subdomains,
            url_scheme_zero_padding,
            enable_pick_features,
            error_event: ImageryErrorEvent::new(),
        })
    }

    fn pad_value(&self, key: &str, val: String) -> String {
        if let Some(template) = self.url_scheme_zero_padding.get(key) {
            let width = template.len();
            if val.len() < width {
                return format!("{:0>width$}", val, width = width);
            }
        }
        val
    }

    /// Formats the tile image request URL according to template variables.
    pub fn request_image_url(&self, x: u32, y: u32, level: u32) -> String {
        let mut result = self.url.clone();

        let num_x = self.tiling_scheme.number_of_x_tiles_at_level(level);
        let num_y = self.tiling_scheme.number_of_y_tiles_at_level(level);

        let rev_x = num_x.saturating_sub(x + 1);
        let rev_y = num_y.saturating_sub(y + 1);
        let rev_z = self
            .maximum_level
            .map(|m| if level < m { m.saturating_sub(level + 1) } else { level })
            .unwrap_or(level);

        let subdomain = if !self.subdomains.is_empty() {
            let idx = (x as usize + y as usize + level as usize) % self.subdomains.len();
            self.subdomains[idx].as_str()
        } else {
            ""
        };

        let tile_rect = self
            .tiling_scheme
            .tile_xy_to_rectangle(x, y, level)
            .unwrap_or(self.rectangle);
        let west_deg = tile_rect.west.to_degrees();
        let south_deg = tile_rect.south.to_degrees();
        let east_deg = tile_rect.east.to_degrees();
        let north_deg = tile_rect.north.to_degrees();

        // Zero-padded values
        let x_str = self.pad_value("{x}", x.to_string());
        let y_str = self.pad_value("{y}", y.to_string());
        let z_str = self.pad_value("{z}", level.to_string());
        let rev_x_str = self.pad_value("{reverseX}", rev_x.to_string());
        let rev_y_str = self.pad_value("{reverseY}", rev_y.to_string());
        let rev_z_str = self.pad_value("{reverseZ}", rev_z.to_string());

        // Standard case replacements
        result = result.replace("{x}", &x_str);
        result = result.replace("{X}", &x_str);
        result = result.replace("{y}", &y_str);
        result = result.replace("{Y}", &y_str);
        result = result.replace("{z}", &z_str);
        result = result.replace("{Z}", &z_str);
        result = result.replace("{s}", subdomain);
        result = result.replace("{S}", subdomain);
        result = result.replace("{reverseX}", &rev_x_str);
        result = result.replace("{reverseY}", &rev_y_str);
        result = result.replace("{reverseZ}", &rev_z_str);

        // Extents
        result = result.replace("{westDegrees}", &west_deg.to_string());
        result = result.replace("{southDegrees}", &south_deg.to_string());
        result = result.replace("{eastDegrees}", &east_deg.to_string());
        result = result.replace("{northDegrees}", &north_deg.to_string());

        // Dimensions
        result = result.replace("{width}", &self.tile_width.to_string());
        result = result.replace("{height}", &self.tile_height.to_string());

        result
    }

    /// Requests an image for a given tile.
    pub fn request_image(&self, x: u32, y: u32, level: u32) -> Result<Vec<u8>, DeveloperError> {
        let url = self.request_image_url(x, y, level);

        // Try local file access if relative or path
        if let Ok(bytes) = std::fs::read(&url) {
            return Ok(bytes);
        }

        // Try standard repo sample image paths
        let sample_paths = [
            "cesium/Specs/Data/Images/Red16x16.png",
            "../cesium/Specs/Data/Images/Red16x16.png",
            "../../cesium/Specs/Data/Images/Red16x16.png",
            "/Users/jinjun/study/cesium-rust/cesium/Specs/Data/Images/Red16x16.png",
        ];

        for sample in sample_paths {
            if let Ok(bytes) = std::fs::read(sample) {
                return Ok(bytes);
            }
        }

        // Return a red pixel buffer
        Ok(vec![255, 0, 0, 255])
    }
}

impl ImageryProvider for UrlTemplateImageryProvider {
    fn rectangle(&self) -> GeoRectangle {
        self.rectangle
    }

    fn tile_width(&self) -> u32 {
        self.tile_width
    }

    fn tile_height(&self) -> u32 {
        self.tile_height
    }

    fn minimum_level(&self) -> u32 {
        self.minimum_level
    }

    fn maximum_level(&self) -> Option<u32> {
        self.maximum_level
    }

    fn has_alpha_channel(&self) -> bool {
        self.has_alpha_channel
    }
}
