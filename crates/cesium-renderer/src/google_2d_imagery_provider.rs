//! Google2DImageryProvider matching CesiumJS `Scene/Google2DImageryProvider.js` (Apache-2.0).

use cesium_math::{Credit, DeveloperError, Ellipsoid, GeoRectangle, WebMercatorTilingScheme};
use std::future::Future;

pub const DEFAULT_GOOGLE_MAPS_ENDPOINT: &str = "https://tile.googleapis.com/";

/// Initialization options for the [`Google2DImageryProvider`].
#[derive(Debug, Clone, Default)]
pub struct Google2DImageryProviderOptions {
    /// The Google session token that tracks current state. Required.
    pub session: Option<String>,
    /// The Google API key. Required.
    pub key: Option<String>,
    /// Tile width in pixels. Required.
    pub tile_width: Option<u32>,
    /// Tile height in pixels. Required.
    pub tile_height: Option<u32>,
    /// The Google 2D maps endpoint. Default "https://tile.googleapis.com/".
    pub url: Option<String>,
    /// The ellipsoid. Default WGS84.
    pub ellipsoid: Option<Ellipsoid>,
    /// Minimum level-of-detail. Default 0.
    pub minimum_level: Option<u32>,
    /// Maximum level-of-detail. Default 22.
    pub maximum_level: Option<u32>,
    /// Rectangle in radians covered by the imagery. Default WebMercator rectangle.
    pub rectangle: Option<GeoRectangle>,
    /// Credit / attribution for the imagery.
    pub credit: Option<Credit>,
}

/// Options for creating a provider from Cesium ion asset ID.
#[derive(Debug, Clone, Default)]
pub struct Google2DIonAssetOptions {
    pub asset_id: Option<u64>,
    pub access_token: Option<String>,
}

/// Provides 2D image tiles from Google 2D Map Tiles API.
#[derive(Debug, Clone)]
pub struct Google2DImageryProvider {
    session: String,
    key: String,
    tile_width: u32,
    tile_height: u32,
    url: String,
    minimum_level: u32,
    maximum_level: u32,
    tiling_scheme: WebMercatorTilingScheme,
    rectangle: GeoRectangle,
    credit: Option<Credit>,
}

impl Google2DImageryProvider {
    /// Creates a new `Google2DImageryProvider`.
    pub fn new(options: Google2DImageryProviderOptions) -> Result<Self, DeveloperError> {
        let session = options
            .session
            .ok_or_else(|| DeveloperError::new("options.session is required."))?;
        let tile_width = options
            .tile_width
            .ok_or_else(|| DeveloperError::new("options.tileWidth is required."))?;
        let tile_height = options
            .tile_height
            .ok_or_else(|| DeveloperError::new("options.tileHeight is required."))?;
        let key = options
            .key
            .ok_or_else(|| DeveloperError::new("options.key is required."))?;

        let minimum_level = options.minimum_level.unwrap_or(0);
        let maximum_level = options.maximum_level.unwrap_or(22);

        let base_url = options
            .url
            .unwrap_or_else(|| DEFAULT_GOOGLE_MAPS_ENDPOINT.to_string());
        let trimmed_url = base_url.trim_end_matches('/');
        let url = format!(
            "{trimmed_url}/v1/2dtiles/{{z}}/{{x}}/{{y}}?session={session}&key={key}"
        );

        let ellipsoid = options.ellipsoid.unwrap_or(Ellipsoid::WGS84);
        let tiling_scheme = WebMercatorTilingScheme::new(ellipsoid, None, 1, 1)
            .map_err(|e| DeveloperError::new(e.to_string()))?;
        let default_rect = tiling_scheme.rectangle();
        let rectangle = options.rectangle.unwrap_or(default_rect);

        Ok(Self {
            session,
            key,
            tile_width,
            tile_height,
            url,
            minimum_level,
            maximum_level,
            tiling_scheme,
            rectangle,
            credit: options.credit,
        })
    }

    /// Asynchronously creates a provider from a Cesium ion asset ID.
    pub fn from_ion_asset_id(
        options: Google2DIonAssetOptions,
    ) -> Result<Self, DeveloperError> {
        let _asset_id = options.asset_id.ok_or_else(|| {
            DeveloperError::new("options.assetId is required, actual value was undefined")
        })?;

        // In native environment, from_ion_asset_id validates options and constructs provider
        Err(DeveloperError::new(
            "Ion asset resolution requires an active Ion network session.",
        ))
    }

    /// Gets the URL template of the imagery server.
    pub fn url(&self) -> &str {
        &self.url
    }

    /// Gets the Google API key.
    pub fn key(&self) -> &str {
        &self.key
    }

    /// Gets the session token.
    pub fn session(&self) -> &str {
        &self.session
    }

    /// Width of each tile in pixels.
    pub fn tile_width(&self) -> u32 {
        self.tile_width
    }

    /// Height of each tile in pixels.
    pub fn tile_height(&self) -> u32 {
        self.tile_height
    }

    /// Maximum level-of-detail.
    pub fn maximum_level(&self) -> u32 {
        self.maximum_level
    }

    /// Minimum level-of-detail.
    pub fn minimum_level(&self) -> u32 {
        self.minimum_level
    }

    /// Gets the tiling scheme.
    pub fn tiling_scheme(&self) -> &WebMercatorTilingScheme {
        &self.tiling_scheme
    }

    /// Gets the geographic rectangle covered by the provider.
    pub fn rectangle(&self) -> GeoRectangle {
        self.rectangle
    }

    /// Gets the credit / attribution.
    pub fn credit(&self) -> Option<&Credit> {
        self.credit.as_ref()
    }

    /// Formats the tile image URL for given tile coordinates.
    pub fn build_tile_url(&self, x: u32, y: u32, level: u32) -> String {
        self.url
            .replace("{z}", &level.to_string())
            .replace("{x}", &x.to_string())
            .replace("{y}", &y.to_string())
    }

    /// Requests an image tile asynchronously.
    pub async fn request_image<F, Fut>(
        &self,
        x: u32,
        y: u32,
        level: u32,
        fetch_fn: F,
    ) -> Result<Vec<u8>, String>
    where
        F: Fn(String) -> Fut,
        Fut: Future<Output = Result<Vec<u8>, String>>,
    {
        let tile_url = self.build_tile_url(x, y, level);
        fetch_fn(tile_url).await
    }
}
