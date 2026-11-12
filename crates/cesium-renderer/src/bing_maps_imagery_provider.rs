//! BingMapsImageryProvider matching CesiumJS `Scene/BingMapsImageryProvider.js` (Apache-2.0).

use cesium_math::{DeveloperError, Ellipsoid, GeoRectangle, RuntimeError, WebMercatorTilingScheme};
use serde_json::Value;

use crate::imagery_layer::ImageryProvider;

/// Bing Maps imagery styles.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum BingMapsStyle {
    #[default]
    Aerial,
    AerialWithLabelsOnDemand,
    RoadOnDemand,
    CanvasDark,
    CanvasLight,
    CanvasGray,
    OrdnanceSurvey,
}

impl BingMapsStyle {
    pub fn as_str(&self) -> &'static str {
        match self {
            BingMapsStyle::Aerial => "Aerial",
            BingMapsStyle::AerialWithLabelsOnDemand => "AerialWithLabelsOnDemand",
            BingMapsStyle::RoadOnDemand => "RoadOnDemand",
            BingMapsStyle::CanvasDark => "CanvasDark",
            BingMapsStyle::CanvasLight => "CanvasLight",
            BingMapsStyle::CanvasGray => "CanvasGray",
            BingMapsStyle::OrdnanceSurvey => "OrdnanceSurvey",
        }
    }
}

/// Initialization options for [`BingMapsImageryProvider`].
#[derive(Clone, Debug, Default)]
pub struct BingMapsImageryProviderOptions {
    pub key: String,
    pub tile_protocol: Option<String>,
    pub map_style: BingMapsStyle,
    pub culture: Option<String>,
    pub map_layer: Option<String>,
    pub ellipsoid: Option<Ellipsoid>,
    pub credit: Option<String>,
}

/// An [`ImageryProvider`] for Bing Maps imagery.
#[derive(Clone, Debug)]
pub struct BingMapsImageryProvider {
    url: String,
    key: String,
    map_style: BingMapsStyle,
    culture: String,
    image_url_template: String,
    subdomains: Vec<String>,
    tile_width: u32,
    tile_height: u32,
    minimum_level: u32,
    maximum_level: u32,
    has_alpha_channel: bool,
    tiling_scheme: WebMercatorTilingScheme,
    rectangle: GeoRectangle,
    ellipsoid: Ellipsoid,
    credit: Option<String>,
}

impl BingMapsImageryProvider {
    /// Converts `(x, y, level)` tile coordinates to a Bing Maps quadkey.
    pub fn tile_xy_to_quad_key(x: u32, y: u32, level: u32) -> String {
        let mut quad_key = String::with_capacity((level + 1) as usize);
        for i in (0..=level).rev() {
            let mask = 1u32 << i;
            let mut digit = b'0';
            if (x & mask) != 0 {
                digit += 1;
            }
            if (y & mask) != 0 {
                digit += 2;
            }
            quad_key.push(digit as char);
        }
        quad_key
    }

    /// Converts a Bing Maps quadkey to `(x, y, level)` tile coordinates.
    pub fn quad_key_to_tile_xy(quad_key: &str) -> Result<(u32, u32, u32), DeveloperError> {
        if quad_key.is_empty() {
            return Err(DeveloperError::new("quadKey is required."));
        }
        let level = (quad_key.len() - 1) as u32;
        let mut x = 0u32;
        let mut y = 0u32;
        for (idx, ch) in quad_key.chars().enumerate() {
            let bit = level - idx as u32;
            let mask = 1u32 << bit;
            match ch {
                '0' => {}
                '1' => x |= mask,
                '2' => y |= mask,
                '3' => {
                    x |= mask;
                    y |= mask;
                }
                _ => {
                    return Err(DeveloperError::new(format!(
                        "Invalid QuadKey digit sequence: {}",
                        quad_key
                    )));
                }
            }
        }
        Ok((x, y, level))
    }

    /// Creates a [`BingMapsImageryProvider`] from a base URL and options.
    pub fn from_url(
        url: &str,
        options: BingMapsImageryProviderOptions,
    ) -> Result<Self, DeveloperError> {
        if url.trim().is_empty() {
            return Err(DeveloperError::new("url is required."));
        }
        if options.key.trim().is_empty() {
            return Err(DeveloperError::new("options.key is required."));
        }

        let ellipsoid = options.ellipsoid.unwrap_or(Ellipsoid::WGS84);
        let tiling_scheme = WebMercatorTilingScheme::new(ellipsoid, None, 1, 1)
            .map_err(|e| DeveloperError::new(e.to_string()))?;
        let rectangle = tiling_scheme.rectangle();
        let has_alpha_channel = options.map_layer.is_some();

        Ok(Self {
            url: url.to_string(),
            key: options.key,
            map_style: options.map_style,
            culture: options.culture.unwrap_or_default(),
            image_url_template:
                "https://ecn.{subdomain}.tiles.virtualearth.net/tiles/a{quadkey}.jpeg?g=1&mkt={culture}"
                    .to_string(),
            subdomains: vec![
                "t0".to_string(),
                "t1".to_string(),
                "t2".to_string(),
                "t3".to_string(),
            ],
            tile_width: 256,
            tile_height: 256,
            minimum_level: 0,
            maximum_level: 21,
            has_alpha_channel,
            tiling_scheme,
            rectangle,
            ellipsoid,
            credit: options.credit,
        })
    }

    /// Creates a [`BingMapsImageryProvider`] from a URL and Bing Maps REST metadata JSON.
    pub fn from_url_with_metadata(
        url: &str,
        metadata: &Value,
        options: BingMapsImageryProviderOptions,
    ) -> Result<Self, RuntimeError> {
        let mut provider = Self::from_url(url, options)
            .map_err(|e| RuntimeError::new(e.to_string()))?;

        let resource_sets = metadata
            .get("resourceSets")
            .and_then(|v| v.as_array())
            .ok_or_else(|| RuntimeError::new("Invalid Bing Maps metadata: missing resourceSets."))?;
        if resource_sets.len() != 1 {
            return Err(RuntimeError::new(
                "Bing Maps metadata must contain exactly one resourceSet.",
            ));
        }

        let resources = resource_sets[0]
            .get("resources")
            .and_then(|v| v.as_array())
            .ok_or_else(|| RuntimeError::new("Invalid Bing Maps metadata: missing resources."))?;
        if resources.len() != 1 {
            return Err(RuntimeError::new(
                "Bing Maps metadata must contain exactly one resource.",
            ));
        }

        let res = &resources[0];
        if let Some(tmpl) = res.get("imageUrl").and_then(|v| v.as_str()) {
            provider.image_url_template = tmpl.to_string();
        }
        if let Some(subs) = res.get("imageUrlSubdomains").and_then(|v| v.as_array()) {
            let parsed: Vec<String> = subs
                .iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect();
            if !parsed.is_empty() {
                provider.subdomains = parsed;
            }
        }
        if let Some(w) = res.get("imageWidth").and_then(|v| v.as_u64()) {
            provider.tile_width = w as u32;
        }
        if let Some(h) = res.get("imageHeight").and_then(|v| v.as_u64()) {
            provider.tile_height = h as u32;
        }
        if let Some(zmin) = res.get("zoomMin").and_then(|v| v.as_u64()) {
            provider.minimum_level = zmin.saturating_sub(1) as u32;
        }
        if let Some(zmax) = res.get("zoomMax").and_then(|v| v.as_u64()) {
            provider.maximum_level = zmax.saturating_sub(1) as u32;
        }

        Ok(provider)
    }

    pub fn url(&self) -> &str {
        &self.url
    }

    pub fn key(&self) -> &str {
        &self.key
    }

    pub fn map_style(&self) -> BingMapsStyle {
        self.map_style
    }

    pub fn culture(&self) -> &str {
        &self.culture
    }

    pub fn tiling_scheme(&self) -> &WebMercatorTilingScheme {
        &self.tiling_scheme
    }

    pub fn credit(&self) -> Option<&str> {
        self.credit.as_deref()
    }

    pub fn build_tile_url(&self, x: u32, y: u32, level: u32) -> String {
        let quadkey = Self::tile_xy_to_quad_key(x, y, level);
        let sub_idx = ((x + y + level) as usize) % self.subdomains.len().max(1);
        let subdomain = self
            .subdomains
            .get(sub_idx)
            .map(|s| s.as_str())
            .unwrap_or("t0");
        self.image_url_template
            .replace("{quadkey}", &quadkey)
            .replace("{subdomain}", subdomain)
            .replace("{culture}", &self.culture)
    }
}

impl ImageryProvider for BingMapsImageryProvider {
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
        Some(self.maximum_level)
    }
    fn has_alpha_channel(&self) -> bool {
        self.has_alpha_channel
    }
    fn is_ready(&self) -> bool {
        true
    }
    fn ellipsoid(&self) -> Ellipsoid {
        self.ellipsoid
    }
}
