//! OpenStreetMapImageryProvider matching CesiumJS `Scene/OpenStreetMapImageryProvider.js` (Apache-2.0).

use cesium_math::{
    Cartographic, Credit, DeveloperError, Ellipsoid, GeoRectangle, WebMercatorTilingScheme,
};

use crate::imagery_layer::ImageryProvider;

/// Construction options for [`OpenStreetMapImageryProvider`].
#[derive(Debug, Clone, Default)]
pub struct OpenStreetMapImageryProviderOptions {
    /// The OpenStreetMap server URL. Default is `"https://tile.openstreetmap.org/"`.
    pub url: Option<String>,
    /// The file extension for images on the server. Default is `"png"`.
    pub file_extension: Option<String>,
    /// When true, request tiles at 2x resolution for retina displays. Default is `false`.
    pub retina_tiles: Option<bool>,
    /// The rectangle of the layer. Default is the full tiling scheme rectangle.
    pub rectangle: Option<GeoRectangle>,
    /// The minimum level-of-detail supported by the provider. Default is `0`.
    pub minimum_level: Option<u32>,
    /// The maximum level-of-detail supported by the provider, or None if no limit.
    pub maximum_level: Option<u32>,
    /// The ellipsoid. If not specified, WGS84 is used.
    pub ellipsoid: Option<Ellipsoid>,
    /// A credit for the data source.
    pub credit: Option<Credit>,
}

/// Provides tiled imagery hosted by OpenStreetMap or another Slippy tile provider.
#[derive(Debug, Clone)]
pub struct OpenStreetMapImageryProvider {
    pub url: String,
    pub template_url: String,
    pub file_extension: String,
    pub retina_tiles: bool,
    pub tile_width: u32,
    pub tile_height: u32,
    pub minimum_level: u32,
    pub maximum_level: Option<u32>,
    pub rectangle: GeoRectangle,
    pub tiling_scheme: WebMercatorTilingScheme,
    pub credit: Credit,
    pub has_alpha_channel: bool,
}

impl OpenStreetMapImageryProvider {
    /// Default attribution credit for OpenStreetMap.
    pub fn default_credit() -> Credit {
        Credit::new("MapQuest, Open Street Map and contributors, CC-BY-SA", None)
    }

    /// Creates a new OpenStreetMap imagery provider.
    pub fn new(options: OpenStreetMapImageryProviderOptions) -> Result<Self, DeveloperError> {
        let raw_url = options
            .url
            .as_deref()
            .unwrap_or("https://tile.openstreetmap.org/");

        let mut base_url = raw_url.trim().to_string();
        if !base_url.ends_with('/') {
            base_url.push('/');
        }

        let file_extension = options.file_extension.as_deref().unwrap_or("png");
        let retina_tiles = options.retina_tiles.unwrap_or(false);
        let retina_suffix = if retina_tiles { "@2x" } else { "" };

        let template_url = format!("{base_url}{{z}}/{{x}}/{{y}}{retina_suffix}.{file_extension}");

        let ellipsoid = options.ellipsoid.unwrap_or(Ellipsoid::WGS84);
        let tiling_scheme = WebMercatorTilingScheme::new(ellipsoid, None, 1, 1).unwrap_or_default();

        let tile_width = 256;
        let tile_height = 256;

        let minimum_level = options.minimum_level.unwrap_or(0);
        let maximum_level = options.maximum_level;

        let rectangle = options.rectangle.unwrap_or_else(|| tiling_scheme.rectangle());

        // Check the number of tiles at the minimum level. If it's more than four, throw an error.
        let sw_pos = Cartographic::new(rectangle.west, rectangle.south, 0.0);
        let ne_pos = Cartographic::new(rectangle.east, rectangle.north, 0.0);

        if let (Some(sw_tile), Some(ne_tile)) = (
            tiling_scheme.position_to_tile(sw_pos, minimum_level),
            tiling_scheme.position_to_tile(ne_pos, minimum_level),
        ) {
            let x_count = (ne_tile.0 as i64 - sw_tile.0 as i64).abs() + 1;
            let y_count = (ne_tile.1 as i64 - sw_tile.1 as i64).abs() + 1;
            let tile_count = x_count * y_count;

            if tile_count > 4 {
                return Err(DeveloperError::new(format!(
                    "The rectangle and minimumLevel indicate that there are {tile_count} tiles at the minimum level. Imagery providers with more than four tiles at the minimum level are not supported."
                )));
            }
        }

        let credit = options.credit.unwrap_or_else(Self::default_credit);

        Ok(Self {
            url: base_url,
            template_url,
            file_extension: file_extension.to_string(),
            retina_tiles,
            tile_width,
            tile_height,
            minimum_level,
            maximum_level,
            rectangle,
            tiling_scheme,
            credit,
            has_alpha_channel: false,
        })
    }

    /// Formats the tile image request URL for a given (x, y, level).
    pub fn request_image_url(&self, x: u32, y: u32, level: u32) -> String {
        self.template_url
            .replace("{z}", &level.to_string())
            .replace("{x}", &x.to_string())
            .replace("{y}", &y.to_string())
    }

    pub fn tiling_scheme(&self) -> &WebMercatorTilingScheme {
        &self.tiling_scheme
    }

    pub fn credit(&self) -> &Credit {
        &self.credit
    }
}

impl ImageryProvider for OpenStreetMapImageryProvider {
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
