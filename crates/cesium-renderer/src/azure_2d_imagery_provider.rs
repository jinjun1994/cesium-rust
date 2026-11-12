//! Azure2DImageryProvider matching CesiumJS `Scene/Azure2DImageryProvider.js` (Apache-2.0).

use cesium_math::{DeveloperError, Ellipsoid, GeoRectangle, WebMercatorTilingScheme};

use crate::imagery_layer::ImageryProvider;

/// Initialization options for [`Azure2DImageryProvider`].
#[derive(Clone, Debug, Default)]
pub struct Azure2DImageryProviderOptions {
    pub subscription_key: String,
    pub tileset_id: Option<String>,
    pub url: Option<String>,
    pub minimum_level: Option<u32>,
    pub maximum_level: Option<u32>,
    pub rectangle: Option<GeoRectangle>,
    pub ellipsoid: Option<Ellipsoid>,
    pub credit: Option<String>,
}

/// An [`ImageryProvider`] for Azure Maps 2D tiles.
#[derive(Clone, Debug)]
pub struct Azure2DImageryProvider {
    url: String,
    tiling_scheme: WebMercatorTilingScheme,
    rectangle: GeoRectangle,
    minimum_level: u32,
    maximum_level: u32,
    credit: Option<String>,
    ellipsoid: Ellipsoid,
}

impl Azure2DImageryProvider {
    pub fn new(options: Azure2DImageryProviderOptions) -> Result<Self, DeveloperError> {
        if options.subscription_key.trim().is_empty() {
            return Err(DeveloperError::new(
                "options.subscriptionKey is required, actual value was undefined",
            ));
        }

        let base_url = options
            .url
            .unwrap_or_else(|| "https://atlas.microsoft.com".to_string());
        let base_url = base_url.trim_end_matches('/');
        let tileset_id = options
            .tileset_id
            .unwrap_or_else(|| "microsoft.base.road".to_string());
        let url = format!(
            "{}/map/tile?api-version=2024-04-01&tilesetId={}&subscription-key={}&zoom={{z}}&x={{x}}&y={{y}}",
            base_url, tileset_id, options.subscription_key
        );

        let ellipsoid = options.ellipsoid.unwrap_or(Ellipsoid::WGS84);
        let tiling_scheme = WebMercatorTilingScheme::new(ellipsoid, None, 1, 1)
            .map_err(|e| DeveloperError::new(e.to_string()))?;

        let scheme_rect = tiling_scheme.rectangle();
        let mut rectangle = options.rectangle.unwrap_or(scheme_rect);
        rectangle.west = rectangle.west.max(scheme_rect.west);
        rectangle.south = rectangle.south.max(scheme_rect.south);
        rectangle.east = rectangle.east.min(scheme_rect.east);
        rectangle.north = rectangle.north.min(scheme_rect.north);

        Ok(Self {
            url,
            tiling_scheme,
            rectangle,
            minimum_level: options.minimum_level.unwrap_or(0),
            maximum_level: options.maximum_level.unwrap_or(22),
            credit: options.credit,
            ellipsoid,
        })
    }

    pub fn url(&self) -> &str {
        &self.url
    }

    pub fn tiling_scheme(&self) -> &WebMercatorTilingScheme {
        &self.tiling_scheme
    }

    pub fn credit(&self) -> Option<&str> {
        self.credit.as_deref()
    }

    pub fn build_tile_url(&self, x: u32, y: u32, level: u32) -> String {
        self.url
            .replace("{z}", &level.to_string())
            .replace("{x}", &x.to_string())
            .replace("{y}", &y.to_string())
    }
}

impl ImageryProvider for Azure2DImageryProvider {
    fn rectangle(&self) -> GeoRectangle {
        self.rectangle
    }
    fn tile_width(&self) -> u32 {
        256
    }
    fn tile_height(&self) -> u32 {
        256
    }
    fn minimum_level(&self) -> u32 {
        self.minimum_level
    }
    fn maximum_level(&self) -> Option<u32> {
        Some(self.maximum_level)
    }
    fn has_alpha_channel(&self) -> bool {
        true
    }
    fn is_ready(&self) -> bool {
        true
    }
    fn ellipsoid(&self) -> Ellipsoid {
        self.ellipsoid
    }
}
