use cesium_math::{Credit, DeveloperError, GeoRectangle};
use crate::imagery_layer::ImageryProvider;

/// Construction options for [`MapboxImageryProvider`].
#[derive(Debug, Clone, Default)]
pub struct MapboxImageryProviderOptions {
    pub url: Option<String>,
    pub map_id: String,
    pub access_token: String,
    pub format: Option<String>,
    pub minimum_level: Option<u32>,
    pub maximum_level: Option<u32>,
    pub rectangle: Option<GeoRectangle>,
    pub credit: Option<Credit>,
}

/// Provides tiled imagery hosted by Mapbox.
#[derive(Debug, Clone)]
pub struct MapboxImageryProvider {
    pub url: String,
    pub map_id: String,
    pub access_token: String,
    pub format: String,
    pub tile_width: u32,
    pub tile_height: u32,
    pub minimum_level: u32,
    pub maximum_level: Option<u32>,
    pub rectangle: GeoRectangle,
    pub credit: Credit,
    pub has_alpha_channel: bool,
}

impl MapboxImageryProvider {
    /// Default attribution credit for Mapbox.
    pub fn default_credit() -> Credit {
        Credit::new(
            "&copy; <a href=\"https://www.mapbox.com/about/maps/\">Mapbox</a> &copy; <a href=\"http://www.openstreetmap.org/copyright\">OpenStreetMap</a> <strong><a href=\"https://www.mapbox.com/map-feedback/\">Improve this map</a></strong>",
            Some(true),
        )
    }

    /// Creates a new Mapbox raster tile provider.
    pub fn new(options: MapboxImageryProviderOptions) -> Result<Self, DeveloperError> {
        if options.map_id.trim().is_empty() {
            return Err(DeveloperError::new("options.mapId is required."));
        }
        if options.access_token.trim().is_empty() {
            return Err(DeveloperError::new("options.accessToken is required."));
        }

        let base_url = options
            .url
            .as_deref()
            .unwrap_or("https://{s}.tiles.mapbox.com/v4/");
        let clean_base = base_url.trim_end_matches('/');

        let raw_format = options.format.as_deref().unwrap_or("png");
        let format = if raw_format.starts_with('.') {
            raw_format.to_string()
        } else {
            format!(".{raw_format}")
        };

        let template_url = format!(
            "{clean_base}/{}/{level}/{x}/{y}{format}?access_token={token}",
            options.map_id,
            level = "{z}",
            x = "{x}",
            y = "{y}",
            token = options.access_token
        );

        let credit = options.credit.unwrap_or_else(Self::default_credit);

        Ok(Self {
            url: template_url,
            map_id: options.map_id,
            access_token: options.access_token,
            format,
            tile_width: 256,
            tile_height: 256,
            minimum_level: options.minimum_level.unwrap_or(0),
            maximum_level: options.maximum_level,
            rectangle: options.rectangle.unwrap_or(GeoRectangle::MAXIMUM),
            credit,
            has_alpha_channel: false,
        })
    }

    /// Formats the tile URL for a given level, x, and y coordinate.
    pub fn request_image_url(&self, x: u32, y: u32, level: u32) -> String {
        self.url
            .replace("{z}", &level.to_string())
            .replace("{x}", &x.to_string())
            .replace("{y}", &y.to_string())
    }
}

impl ImageryProvider for MapboxImageryProvider {
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
