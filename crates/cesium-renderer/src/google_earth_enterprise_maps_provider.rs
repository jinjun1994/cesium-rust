//! GoogleEarthEnterpriseMapsProvider matching CesiumJS `Scene/GoogleEarthEnterpriseMapsProvider.js` (Apache-2.0).

use cesium_math::{
    Credit, Ellipsoid, GeoRectangle, GeographicTilingScheme, RuntimeError,
    WebMercatorTilingScheme,
};
use serde_json::Value;
use std::f64::consts::PI;
use std::future::Future;

/// Tiling scheme variants for GoogleEarthEnterpriseMapsProvider.
#[derive(Debug, Clone)]
pub enum GeeMapsTilingScheme {
    Mercator(WebMercatorTilingScheme),
    Geographic(GeographicTilingScheme),
}

impl GeeMapsTilingScheme {
    pub fn rectangle(&self) -> GeoRectangle {
        match self {
            Self::Mercator(m) => m.rectangle(),
            Self::Geographic(g) => g.rectangle(),
        }
    }
}

/// Initialization options for the [`GoogleEarthEnterpriseMapsProvider`].
#[derive(Debug, Clone, Default)]
pub struct GoogleEarthEnterpriseMapsProviderOptions {
    /// URL path of the Google Earth server hosting the imagery. Default "/default_map".
    pub path: Option<String>,
    /// Maximum level-of-detail supported, or None if unlimited.
    pub maximum_level: Option<u32>,
    /// Ellipsoid. Default WGS84.
    pub ellipsoid: Option<Ellipsoid>,
    /// Credit / attribution for the imagery.
    pub credit: Option<Credit>,
}

/// Provides tiled imagery using the Google Earth Maps 2D API.
#[derive(Debug, Clone)]
pub struct GoogleEarthEnterpriseMapsProvider {
    url: String,
    path: String,
    channel: u32,
    version: u32,
    request_type: String,
    tile_width: u32,
    tile_height: u32,
    minimum_level: u32,
    maximum_level: Option<u32>,
    tiling_scheme: GeeMapsTilingScheme,
    rectangle: GeoRectangle,
    credit: Credit,
    has_alpha_channel: bool,
}

impl GoogleEarthEnterpriseMapsProvider {
    /// Creates a provider by requesting and parsing metadata from the server URL.
    pub async fn from_url<F, Fut>(
        url: Option<String>,
        channel: Option<u32>,
        options: Option<GoogleEarthEnterpriseMapsProviderOptions>,
        fetch_metadata: F,
    ) -> Result<Self, RuntimeError>
    where
        F: FnOnce(String) -> Fut,
        Fut: Future<Output = Result<String, String>>,
    {
        let url = url.ok_or_else(|| {
            RuntimeError::new("options.url is required.")
        })?;
        let channel = channel.ok_or_else(|| {
            RuntimeError::new("options.channel is required.")
        })?;

        let options = options.unwrap_or_default();
        let path = options.path.unwrap_or_else(|| "/default_map".to_string());
        let ellipsoid = options.ellipsoid.unwrap_or(Ellipsoid::WGS84);

        // Build metadata request URL
        let mut base_combined = url.clone();
        if !path.is_empty() {
            if !base_combined.ends_with('/') && !path.starts_with('/') {
                base_combined.push('/');
            }
            if base_combined.ends_with('/') && path.starts_with('/') {
                base_combined.push_str(&path[1..]);
            } else {
                base_combined.push_str(&path);
            }
        }
        if !base_combined.ends_with('/') {
            base_combined.push('/');
        }
        let metadata_url = format!("{base_combined}query?request=Json&vars=geeServerDefs&is2d=t");

        let metadata_text = fetch_metadata(metadata_url.clone())
            .await
            .map_err(|err| {
                RuntimeError::new(format!(
                    "An error occurred while accessing {metadata_url}: {err}"
                ))
            })?;

        let parsed_json = parse_loose_json(&metadata_text)
            .map_err(|err| RuntimeError::new(format!("An error occurred while accessing {metadata_url}: {err}")))?;

        let layers = parsed_json
            .get("layers")
            .and_then(Value::as_array)
            .ok_or_else(|| {
                RuntimeError::new(format!("Could not find layer with channel (id) of {channel}."))
            })?;

        let layer = layers
            .iter()
            .find(|l| l.get("id").and_then(Value::as_u64).map(|v| v as u32) == Some(channel))
            .ok_or_else(|| {
                RuntimeError::new(format!("Could not find layer with channel (id) of {channel}."))
            })?;

        let version = layer
            .get("version")
            .and_then(Value::as_u64)
            .map(|v| v as u32)
            .ok_or_else(|| {
                RuntimeError::new(format!("Could not find a version in channel (id) {channel}."))
            })?;

        let proj_str = parsed_json
            .get("projection")
            .and_then(Value::as_str)
            .unwrap_or("mercator");

        let (tiling_scheme, rectangle) = match proj_str {
            "flat" => {
                let rect = GeoRectangle {
                    west: -PI,
                    south: -PI,
                    east: PI,
                    north: PI,
                };
                let scheme = GeographicTilingScheme::new(ellipsoid, rect, 2, 2)
                    .map_err(|e| RuntimeError::new(e.to_string()))?;
                (GeeMapsTilingScheme::Geographic(scheme), rect)
            }
            "mercator" => {
                let scheme = WebMercatorTilingScheme::new(ellipsoid, None, 2, 2)
                    .map_err(|e| RuntimeError::new(e.to_string()))?;
                let rect = scheme.rectangle();
                (GeeMapsTilingScheme::Mercator(scheme), rect)
            }
            other => {
                return Err(RuntimeError::new(format!(
                    "Unsupported projection {other}."
                )));
            }
        };

        let credit = options.credit.unwrap_or_else(|| {
            Credit::new(
                r#"<a href="http://www.google.com/enterprise/mapsearth/products/earthenterprise.html"><img src="Assets/Images/google_earth_credit.png" title="Google Imagery"/></a>"#,
                Some(true),
            )
        });

        Ok(Self {
            url,
            path,
            channel,
            version,
            request_type: "ImageryMaps".to_string(),
            tile_width: 256,
            tile_height: 256,
            minimum_level: 0,
            maximum_level: options.maximum_level,
            tiling_scheme,
            rectangle,
            credit,
            has_alpha_channel: true,
        })
    }

    /// Gets the server URL.
    pub fn url(&self) -> &str {
        &self.url
    }

    /// Gets the path on the server.
    pub fn path(&self) -> &str {
        &self.path
    }

    /// Gets the channel id.
    pub fn channel(&self) -> u32 {
        self.channel
    }

    /// Gets the version.
    pub fn version(&self) -> u32 {
        self.version
    }

    /// Gets the request type.
    pub fn request_type(&self) -> &str {
        &self.request_type
    }

    /// Width of each tile in pixels.
    pub fn tile_width(&self) -> u32 {
        self.tile_width
    }

    /// Height of each tile in pixels.
    pub fn tile_height(&self) -> u32 {
        self.tile_height
    }

    /// Minimum level-of-detail. Always 0.
    pub fn minimum_level(&self) -> u32 {
        self.minimum_level
    }

    /// Maximum level-of-detail, if specified.
    pub fn maximum_level(&self) -> Option<u32> {
        self.maximum_level
    }

    /// Tiling scheme.
    pub fn tiling_scheme(&self) -> &GeeMapsTilingScheme {
        &self.tiling_scheme
    }

    /// Geographic rectangle bounds.
    pub fn rectangle(&self) -> GeoRectangle {
        self.rectangle
    }

    /// Credit.
    pub fn credit(&self) -> &Credit {
        &self.credit
    }

    /// Whether images have an alpha channel. Always true for Google Earth Maps.
    pub fn has_alpha_channel(&self) -> bool {
        self.has_alpha_channel
    }

    /// Formats tile request URL.
    pub fn build_tile_url(&self, x: u32, y: u32, level: u32) -> String {
        let mut base = self.url.clone();
        if !self.path.is_empty() {
            if !base.ends_with('/') && !self.path.starts_with('/') {
                base.push('/');
            }
            if base.ends_with('/') && self.path.starts_with('/') {
                base.push_str(&self.path[1..]);
            } else {
                base.push_str(&self.path);
            }
        }
        if !base.ends_with('/') {
            base.push('/');
        }
        let z = level + 1;
        format!(
            "{base}query?request={}&channel={}&version={}&x={x}&y={y}&z={z}",
            self.request_type, self.channel, self.version
        )
    }

    /// Requests image for a tile asynchronously.
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

fn parse_loose_json(text: &str) -> Result<Value, String> {
    if let Ok(v) = serde_json::from_str::<Value>(text) {
        return Ok(v);
    }

    // Server might send JSON without quotes around keys, e.g. isAuthenticated: true
    let mut result = String::with_capacity(text.len() + 64);
    let chars: Vec<char> = text.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        result.push(c);
        if c == '{' || c == ',' {
            // Check if next non-whitespace token is an unquoted identifier followed by ':'
            let mut j = i + 1;
            while j < chars.len() && chars[j].is_whitespace() {
                j += 1;
            }
            if j < chars.len() && (chars[j].is_alphanumeric() || chars[j] == '_') {
                let start = j;
                while j < chars.len() && (chars[j].is_alphanumeric() || chars[j] == '_') {
                    j += 1;
                }
                let mut k = j;
                while k < chars.len() && chars[k].is_whitespace() {
                    k += 1;
                }
                if k < chars.len() && chars[k] == ':' {
                    // It is an unquoted key!
                    for ws in &chars[i + 1..start] {
                        result.push(*ws);
                    }
                    result.push('"');
                    for key_char in &chars[start..j] {
                        result.push(*key_char);
                    }
                    result.push('"');
                    i = j - 1;
                }
            }
        }
        i += 1;
    }

    serde_json::from_str::<Value>(&result).map_err(|e| e.to_string())
}
