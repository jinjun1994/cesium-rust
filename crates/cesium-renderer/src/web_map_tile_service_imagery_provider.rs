//! WebMapTileServiceImageryProvider matching CesiumJS `Scene/WebMapTileServiceImageryProvider.js` (Apache-2.0).

use std::collections::BTreeMap;
use std::sync::atomic::{AtomicUsize, Ordering};

use cesium_math::{
    Credit, DeveloperError, Ellipsoid, GeoRectangle, WebMercatorTilingScheme,
};

use crate::imagery_layer::ImageryProvider;
use crate::quadtree_tile::AnyTilingScheme;

/// Construction options for [`WebMapTileServiceImageryProvider`].
#[derive(Debug, Clone, Default)]
pub struct WebMapTileServiceImageryProviderOptions {
    pub url: Option<String>,
    pub format: Option<String>,
    pub layer: Option<String>,
    pub style: Option<String>,
    pub tile_matrix_set_id: Option<String>,
    pub tile_matrix_labels: Option<Vec<String>>,
    pub clock: Option<cesium_math::JulianDate>,
    pub rectangle: Option<GeoRectangle>,
    pub tiling_scheme: Option<AnyTilingScheme>,
    pub ellipsoid: Option<Ellipsoid>,
    pub tile_width: Option<u32>,
    pub tile_height: Option<u32>,
    pub minimum_level: Option<u32>,
    pub maximum_level: Option<u32>,
    pub credit: Option<Credit>,
    pub subdomains: Option<Vec<String>>,
    pub has_alpha_channel: Option<bool>,
}

/// Provides tiled imagery served by WMTS 1.0.0 compliant servers.
#[derive(Clone, Debug)]
pub struct WebMapTileServiceImageryProvider {
    pub url: String,
    pub format: String,
    pub layer: String,
    pub style: String,
    pub tile_matrix_set_id: String,
    pub tile_matrix_labels: Option<Vec<String>>,
    pub tile_width: u32,
    pub tile_height: u32,
    pub minimum_level: u32,
    pub maximum_level: Option<u32>,
    pub rectangle: GeoRectangle,
    pub tiling_scheme: AnyTilingScheme,
    pub credit: Option<Credit>,
    pub has_alpha_channel: bool,
    pub subdomains: Vec<String>,
    pub use_kvp: bool,
    request_counter: std::sync::Arc<AtomicUsize>,
}

impl WebMapTileServiceImageryProvider {
    /// Creates a new `WebMapTileServiceImageryProvider`.
    pub fn new(options: WebMapTileServiceImageryProviderOptions) -> Result<Self, DeveloperError> {
        let url = options
            .url
            .ok_or_else(|| DeveloperError::new("options.url is required."))?;
        if url.trim().is_empty() {
            return Err(DeveloperError::new("options.url is required."));
        }

        let layer = options
            .layer
            .ok_or_else(|| DeveloperError::new("options.layer is required."))?;

        let style = options
            .style
            .ok_or_else(|| DeveloperError::new("options.style is required."))?;

        let tile_matrix_set_id = options
            .tile_matrix_set_id
            .ok_or_else(|| DeveloperError::new("options.tileMatrixSetID is required."))?;

        let format = options.format.unwrap_or_else(|| "image/jpeg".to_string());
        let ellipsoid = options.ellipsoid.unwrap_or(Ellipsoid::WGS84);
        let tiling_scheme = options.tiling_scheme.unwrap_or_else(|| {
            let wm = WebMercatorTilingScheme::new(ellipsoid, None, 1, 1).unwrap_or_default();
            AnyTilingScheme::WebMercator(wm)
        });

        let tile_width = options.tile_width.unwrap_or(256);
        let tile_height = options.tile_height.unwrap_or(256);
        let minimum_level = options.minimum_level.unwrap_or(0);
        let maximum_level = options.maximum_level;
        let rectangle = options.rectangle.unwrap_or_else(|| tiling_scheme.rectangle());
        let has_alpha_channel = options.has_alpha_channel.unwrap_or(true);

        let subdomains = options
            .subdomains
            .unwrap_or_else(|| vec!["a".to_string(), "b".to_string(), "c".to_string()]);

        let has_brackets = url.contains('{');
        let is_only_s = has_brackets && url.matches('{').count() == 1 && url.contains("{s}");
        let use_kvp = !has_brackets || is_only_s;

        Ok(Self {
            url,
            format,
            layer,
            style,
            tile_matrix_set_id,
            tile_matrix_labels: options.tile_matrix_labels,
            tile_width,
            tile_height,
            minimum_level,
            maximum_level,
            rectangle,
            tiling_scheme,
            credit: options.credit,
            has_alpha_channel,
            subdomains,
            use_kvp,
            request_counter: std::sync::Arc::new(AtomicUsize::new(0)),
        })
    }

    fn select_subdomain(&self) -> String {
        if self.subdomains.is_empty() {
            return String::new();
        }
        let count = self.request_counter.fetch_add(1, Ordering::Relaxed);
        self.subdomains[count % self.subdomains.len()].clone()
    }

    /// Formats the tile matrix label for a given level.
    pub fn get_tile_matrix_label(&self, level: u32) -> String {
        if let Some(labels) = &self.tile_matrix_labels {
            if let Some(label) = labels.get(level as usize) {
                return label.clone();
            }
        }
        level.to_string()
    }

    /// Builds the tile request URL for the specified tile coordinate.
    pub fn build_image_url(&self, tile_col: u32, tile_row: u32, level: u32) -> String {
        let tile_matrix = self.get_tile_matrix_label(level);
        let subdomain = self.select_subdomain();

        if self.use_kvp {
            let mut base = self.url.replace("{s}", &subdomain);
            let mut params = BTreeMap::new();
            params.insert("service".to_string(), "WMTS".to_string());
            params.insert("version".to_string(), "1.0.0".to_string());
            params.insert("request".to_string(), "GetTile".to_string());
            params.insert("format".to_string(), self.format.clone());
            params.insert("layer".to_string(), self.layer.clone());
            params.insert("style".to_string(), self.style.clone());
            params.insert("tilecol".to_string(), tile_col.to_string());
            params.insert("tilerow".to_string(), tile_row.to_string());
            params.insert("tilematrixset".to_string(), self.tile_matrix_set_id.clone());
            params.insert("tilematrix".to_string(), tile_matrix);

            let mut separator = if base.contains('?') { "&" } else { "?" };
            for (k, v) in params {
                base.push_str(separator);
                separator = "&";
                base.push_str(&k);
                base.push('=');
                base.push_str(&v);
            }
            base
        } else {
            let mut res = self.url.clone();
            res = res.replace("{s}", &subdomain);
            res = res.replace("{layer}", &self.layer);
            res = res.replace("{Layer}", &self.layer);
            res = res.replace("{style}", &self.style);
            res = res.replace("{Style}", &self.style);
            res = res.replace("{tilematrixset}", &self.tile_matrix_set_id);
            res = res.replace("{TileMatrixSet}", &self.tile_matrix_set_id);
            res = res.replace("{tilematrix}", &tile_matrix);
            res = res.replace("{TileMatrix}", &tile_matrix);
            res = res.replace("{tilerow}", &tile_row.to_string());
            res = res.replace("{TileRow}", &tile_row.to_string());
            res = res.replace("{tilecol}", &tile_col.to_string());
            res = res.replace("{TileCol}", &tile_col.to_string());
            res = res.replace("{format}", &self.format);
            res
        }
    }

    /// Alias for `build_image_url` to match the ImageryProvider contract.
    pub fn request_image_url(&self, tile_col: u32, tile_row: u32, level: u32) -> String {
        self.build_image_url(tile_col, tile_row, level)
    }
}

impl ImageryProvider for WebMapTileServiceImageryProvider {
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

    fn ellipsoid(&self) -> Ellipsoid {
        self.tiling_scheme.ellipsoid()
    }
}
