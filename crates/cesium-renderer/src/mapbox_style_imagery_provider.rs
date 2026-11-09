use cesium_math::{Credit, DeveloperError, GeoRectangle};
use crate::imagery_layer::ImageryProvider;

/// Construction options for [`MapboxStyleImageryProvider`].
#[derive(Debug, Clone, Default)]
pub struct MapboxStyleImageryProviderOptions {
    pub url: Option<String>,
    pub username: Option<String>,
    pub style_id: String,
    pub access_token: String,
    pub tilesize: Option<u32>,
    pub scale_factor: bool,
    pub minimum_level: Option<u32>,
    pub maximum_level: Option<u32>,
    pub rectangle: Option<GeoRectangle>,
    pub credit: Option<Credit>,
}

/// Provides tiled imagery styled via Mapbox Styles API.
#[derive(Debug, Clone)]
pub struct MapboxStyleImageryProvider {
    pub url: String,
    pub username: String,
    pub style_id: String,
    pub access_token: String,
    pub tilesize: u32,
    pub scale_factor: bool,
    pub tile_width: u32,
    pub tile_height: u32,
    pub minimum_level: u32,
    pub maximum_level: Option<u32>,
    pub rectangle: GeoRectangle,
    pub credit: Credit,
    pub has_alpha_channel: bool,
}

impl MapboxStyleImageryProvider {
    /// Default attribution credit for Mapbox.
    pub fn default_credit() -> Credit {
        Credit::new(
            "&copy; <a href=\"https://www.mapbox.com/about/maps/\">Mapbox</a> &copy; <a href=\"http://www.openstreetmap.org/copyright\">OpenStreetMap</a> <strong><a href=\"https://www.mapbox.com/map-feedback/\">Improve this map</a></strong>",
            Some(true),
        )
    }

    /// Creates a new Mapbox style tile provider.
    pub fn new(options: MapboxStyleImageryProviderOptions) -> Result<Self, DeveloperError> {
        if options.style_id.trim().is_empty() {
            return Err(DeveloperError::new("options.styleId is required."));
        }
        if options.access_token.trim().is_empty() {
            return Err(DeveloperError::new("options.accessToken is required."));
        }

        let base_url = options
            .url
            .as_deref()
            .unwrap_or("https://api.mapbox.com/styles/v1/");
        let clean_base = base_url.trim_end_matches('/');

        let username = options.username.unwrap_or_else(|| "mapbox".to_string());
        let tilesize = options.tilesize.unwrap_or(512);
        let scale_suffix = if options.scale_factor { "@2x" } else { "" };

        let template_url = format!(
            "{clean_base}/{username}/{style_id}/tiles/{tilesize}/{level}/{x}/{y}{scale_suffix}?access_token={token}",
            style_id = options.style_id,
            level = "{z}",
            x = "{x}",
            y = "{y}",
            token = options.access_token
        );

        let credit = options.credit.unwrap_or_else(Self::default_credit);

        Ok(Self {
            url: template_url,
            username,
            style_id: options.style_id,
            access_token: options.access_token,
            tilesize,
            scale_factor: options.scale_factor,
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

impl ImageryProvider for MapboxStyleImageryProvider {
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
