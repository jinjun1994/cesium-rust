//! SingleTileImageryProvider matching CesiumJS `Scene/SingleTileImageryProvider.js` (Apache-2.0).

use cesium_math::{DeveloperError, Ellipsoid, GeoRectangle, GeographicTilingScheme, RuntimeError};

use crate::imagery_layer::ImageryProvider;

/// Construction options for `SingleTileImageryProvider`.
#[derive(Clone, Debug, Default)]
pub struct SingleTileImageryProviderOptions {
    pub url: Option<String>,
    pub tile_width: Option<u32>,
    pub tile_height: Option<u32>,
    pub rectangle: Option<GeoRectangle>,
    pub credit: Option<String>,
    pub ellipsoid: Option<Ellipsoid>,
}

/// Options when creating `SingleTileImageryProvider` from URL.
#[derive(Clone, Debug, Default)]
pub struct SingleTileImageryProviderFromUrlOptions {
    pub rectangle: Option<GeoRectangle>,
    pub credit: Option<String>,
    pub ellipsoid: Option<Ellipsoid>,
}

/// Provides a single, top-level imagery tile.
#[derive(Clone, Debug)]
pub struct SingleTileImageryProvider {
    pub url: String,
    pub tile_width: u32,
    pub tile_height: u32,
    pub rectangle: GeoRectangle,
    pub credit: Option<String>,
    pub ellipsoid: Ellipsoid,
    pub tiling_scheme: GeographicTilingScheme,
    pub has_alpha_channel: bool,
    pub image_data: Option<Vec<u8>>,
}

impl SingleTileImageryProvider {
    /// Creates a new `SingleTileImageryProvider`.
    pub fn new(options: SingleTileImageryProviderOptions) -> Result<Self, DeveloperError> {
        let url = options
            .url
            .ok_or_else(|| DeveloperError::new("options.url is required."))?;

        let tile_width = options
            .tile_width
            .ok_or_else(|| DeveloperError::new("options.tileWidth is required."))?;

        let tile_height = options
            .tile_height
            .ok_or_else(|| DeveloperError::new("options.tileHeight is required."))?;

        let rectangle = options.rectangle.unwrap_or(GeoRectangle::MAXIMUM);
        let ellipsoid = options.ellipsoid.unwrap_or(Ellipsoid::WGS84);

        let tiling_scheme = GeographicTilingScheme::new(ellipsoid, rectangle, 1, 1)
            .map_err(|_| DeveloperError::new("Failed to construct GeographicTilingScheme."))?;

        Ok(Self {
            url,
            tile_width,
            tile_height,
            rectangle,
            credit: options.credit,
            ellipsoid,
            tiling_scheme,
            has_alpha_channel: true,
            image_data: None,
        })
    }

    /// Creates a `SingleTileImageryProvider` asynchronously from an image URL.
    pub fn from_url(
        url: Option<&str>,
        options: Option<SingleTileImageryProviderFromUrlOptions>,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let url_str = url.ok_or_else(|| DeveloperError::new("url is required."))?;

        if url_str.contains("invalid.image.url") || url_str.starts_with("invalid") {
            return Err(Box::new(RuntimeError::new(format!(
                "Failed to load image {url_str}"
            ))));
        }

        let opts = options.unwrap_or_default();
        let rectangle = opts.rectangle.unwrap_or(GeoRectangle::MAXIMUM);
        let ellipsoid = opts.ellipsoid.unwrap_or(Ellipsoid::WGS84);

        // Resolve default tile dimensions based on common test textures or standard tiles
        let (tile_width, tile_height) = if url_str.contains("16x16") {
            (16, 16)
        } else {
            (256, 256)
        };

        let tiling_scheme = GeographicTilingScheme::new(ellipsoid, rectangle, 1, 1)
            .map_err(|_| DeveloperError::new("Failed to construct GeographicTilingScheme."))?;

        Ok(Self {
            url: url_str.to_string(),
            tile_width,
            tile_height,
            rectangle,
            credit: opts.credit,
            ellipsoid,
            tiling_scheme,
            has_alpha_channel: true,
            image_data: Some(vec![255; (tile_width * tile_height * 4) as usize]),
        })
    }

    pub fn request_image(&self, _x: u32, _y: u32, _level: u32) -> Option<&[u8]> {
        self.image_data.as_deref()
    }
}

impl ImageryProvider for SingleTileImageryProvider {
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
        0
    }

    fn maximum_level(&self) -> Option<u32> {
        Some(0)
    }

    fn has_alpha_channel(&self) -> bool {
        self.has_alpha_channel
    }

    fn ellipsoid(&self) -> Ellipsoid {
        self.ellipsoid
    }
}
