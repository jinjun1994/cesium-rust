//! GoogleEarthEnterpriseImageryProvider matching CesiumJS `Scene/GoogleEarthEnterpriseImageryProvider.js` (Apache-2.0).

use cesium_math::{
    Credit, Ellipsoid, GeoRectangle, GeographicTilingScheme,
    GoogleEarthEnterpriseMetadata, RuntimeError,
};
use std::f64::consts::PI;
use std::future::Future;

/// Initialization options for the [`GoogleEarthEnterpriseImageryProvider`].
#[derive(Debug, Clone, Default)]
pub struct GoogleEarthEnterpriseImageryProviderOptions {
    /// The ellipsoid. Default WGS84.
    pub ellipsoid: Option<Ellipsoid>,
    /// Credit / attribution for the imagery.
    pub credit: Option<Credit>,
}

/// Provides tiled imagery using the Google Earth Enterprise REST API.
#[derive(Debug, Clone)]
pub struct GoogleEarthEnterpriseImageryProvider {
    metadata: GoogleEarthEnterpriseMetadata,
    tile_width: u32,
    tile_height: u32,
    maximum_level: u32,
    minimum_level: u32,
    tiling_scheme: GeographicTilingScheme,
    rectangle: GeoRectangle,
    credit: Option<Credit>,
    has_alpha_channel: bool,
}

impl GoogleEarthEnterpriseImageryProvider {
    /// Creates a provider from a metadata instance.
    pub fn from_metadata(
        metadata: Option<GoogleEarthEnterpriseMetadata>,
        options: Option<GoogleEarthEnterpriseImageryProviderOptions>,
    ) -> Result<Self, RuntimeError> {
        let metadata = metadata
            .ok_or_else(|| RuntimeError::new("metadata is required."))?;

        if !metadata.imagery_present {
            let mut url = metadata.url.clone();
            if !url.ends_with('/') {
                url.push('/');
            }
            return Err(RuntimeError::new(format!(
                "The server {url} doesn't have imagery"
            )));
        }

        let options = options.unwrap_or_default();
        let ellipsoid = options.ellipsoid.unwrap_or(Ellipsoid::WGS84);
        let rectangle = GeoRectangle {
            west: -PI,
            south: -PI,
            east: PI,
            north: PI,
        };
        let tiling_scheme = GeographicTilingScheme::new(ellipsoid, rectangle, 2, 2)
            .map_err(|e| RuntimeError::new(e.to_string()))?;

        Ok(Self {
            metadata,
            tile_width: 256,
            tile_height: 256,
            maximum_level: 23,
            minimum_level: 0,
            tiling_scheme,
            rectangle,
            credit: options.credit,
            has_alpha_channel: false,
        })
    }

    /// Creates a new provider from a server URL.
    pub fn from_url(
        url: &str,
        options: Option<GoogleEarthEnterpriseImageryProviderOptions>,
    ) -> Result<Self, RuntimeError> {
        let metadata = GoogleEarthEnterpriseMetadata::from_url(url)
            .map_err(|e| RuntimeError::new(e.message))?;
        Self::from_metadata(Some(metadata), options)
    }

    /// Gets the server URL hosting the imagery.
    pub fn url(&self) -> &str {
        &self.metadata.url
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

    /// Tiling scheme used by this provider.
    pub fn tiling_scheme(&self) -> &GeographicTilingScheme {
        &self.tiling_scheme
    }

    /// The rectangle covered by this provider.
    pub fn rectangle(&self) -> GeoRectangle {
        self.rectangle
    }

    /// The credit to display.
    pub fn credit(&self) -> Option<&Credit> {
        self.credit.as_ref()
    }

    /// Whether images include an alpha channel. Always false for Google Earth Enterprise imagery.
    pub fn has_alpha_channel(&self) -> bool {
        self.has_alpha_channel
    }

    /// Formats the tile image URL for given tile coordinates.
    pub fn build_tile_url(&self, x: u32, y: u32, level: u32) -> String {
        let quadkey = GoogleEarthEnterpriseMetadata::tile_xy_to_quad_key(x, y, level);
        let version = self
            .metadata
            .get_tile_information(x, y, level)
            .map(|info| info.imagery_version)
            .unwrap_or(1);

        let mut base_url = self.metadata.url.clone();
        if !base_url.ends_with('/') {
            base_url.push('/');
        }
        format!("{base_url}flatfile?f1-0{quadkey}-i.{version}")
    }

    /// Requests the tile image asynchronously.
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
