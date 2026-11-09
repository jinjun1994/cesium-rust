use std::collections::HashMap;
use std::sync::{Arc, LazyLock, Mutex};
use cesium_math::{Credit, GeoRectangle, IonEndpoint, RuntimeError};
use crate::imagery_layer::{ImageryLayerFeatureInfo, ImageryProvider};

/// Supported external imagery service types from Cesium ion.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ExternalImageryType {
    ArcGisMapServer,
    Bing,
    GoogleEarth,
    Mapbox,
    SingleTile,
    Tms,
    UrlTemplate,
    Wms,
    Wmts,
    Google2DMaps,
    AzureMaps,
}

impl ExternalImageryType {
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "ARCGIS_MAPSERVER" => Some(Self::ArcGisMapServer),
            "BING" => Some(Self::Bing),
            "GOOGLE_EARTH" => Some(Self::GoogleEarth),
            "MAPBOX" => Some(Self::Mapbox),
            "SINGLE_TILE" => Some(Self::SingleTile),
            "TMS" => Some(Self::Tms),
            "URL_TEMPLATE" => Some(Self::UrlTemplate),
            "WMS" => Some(Self::Wms),
            "WMTS" => Some(Self::Wmts),
            "GOOGLE_2D_MAPS" => Some(Self::Google2DMaps),
            "AZURE_MAPS" => Some(Self::AzureMaps),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::ArcGisMapServer => "ARCGIS_MAPSERVER",
            Self::Bing => "BING",
            Self::GoogleEarth => "GOOGLE_EARTH",
            Self::Mapbox => "MAPBOX",
            Self::SingleTile => "SINGLE_TILE",
            Self::Tms => "TMS",
            Self::UrlTemplate => "URL_TEMPLATE",
            Self::Wms => "WMS",
            Self::Wmts => "WMTS",
            Self::Google2DMaps => "GOOGLE_2D_MAPS",
            Self::AzureMaps => "AZURE_MAPS",
        }
    }
}

/// Generic backing imagery provider initialized from an ion endpoint.
#[derive(Debug, Clone)]
pub struct GenericImageryProvider {
    pub url: String,
    pub external_type: Option<ExternalImageryType>,
    pub rectangle: GeoRectangle,
    pub tile_width: u32,
    pub tile_height: u32,
    pub minimum_level: u32,
    pub maximum_level: Option<u32>,
    pub has_alpha_channel: bool,
    pub tile_credits: Vec<Credit>,
    pub extra_options: Option<serde_json::Value>,
}

impl Default for GenericImageryProvider {
    fn default() -> Self {
        Self {
            url: String::new(),
            external_type: None,
            rectangle: GeoRectangle::MAXIMUM,
            tile_width: 256,
            tile_height: 256,
            minimum_level: 0,
            maximum_level: None,
            has_alpha_channel: true,
            tile_credits: Vec::new(),
            extra_options: None,
        }
    }
}

impl ImageryProvider for GenericImageryProvider {
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

/// Options for configuring an `IonImageryProvider`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct IonImageryProviderOptions {
    pub access_token: Option<String>,
    pub server: Option<String>,
}

static ENDPOINT_CACHE: LazyLock<Mutex<HashMap<String, IonEndpoint>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

/// Provides tiled imagery using the Cesium ion REST API.
#[derive(Clone)]
pub struct IonImageryProvider {
    pub asset_id: i64,
    pub endpoint: IonEndpoint,
    pub external_type: Option<ExternalImageryType>,
    pub tile_credits: Vec<Credit>,
    pub imagery_provider: Arc<dyn ImageryProvider>,
}

impl std::fmt::Debug for IonImageryProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("IonImageryProvider")
            .field("asset_id", &self.asset_id)
            .field("endpoint", &self.endpoint)
            .field("external_type", &self.external_type)
            .field("tile_credits", &self.tile_credits)
            .finish()
    }
}

impl IonImageryProvider {
    /// Clears the global endpoint cache (useful for testing).
    pub fn clear_cache() {
        if let Ok(mut cache) = ENDPOINT_CACHE.lock() {
            cache.clear();
        }
    }

    /// Creates an `IonImageryProvider` from an endpoint response directly.
    pub fn from_endpoint(
        asset_id: i64,
        endpoint: IonEndpoint,
        _options: Option<&IonImageryProviderOptions>,
    ) -> Result<Self, RuntimeError> {
        if asset_id <= 0 {
            return Err(RuntimeError::new("assetId must be positive"));
        }

        if endpoint.endpoint_type != "IMAGERY" {
            return Err(RuntimeError::new(format!(
                "Cesium ion asset {asset_id} is not an imagery asset."
            )));
        }

        let parsed_ext_type = if let Some(ref ext) = endpoint.external_type {
            let ext_type = ExternalImageryType::parse(ext).ok_or_else(|| {
                RuntimeError::new(format!("Unrecognized Cesium ion imagery type: {ext}"))
            })?;
            Some(ext_type)
        } else {
            None
        };

        // Extract credits from attributions
        let mut tile_credits = Vec::new();
        for attr in &endpoint.attributions {
            let text = attr.html.as_deref().or(attr.text.as_deref()).unwrap_or("");
            tile_credits.push(Credit::new(text, Some(!attr.collapsible)));
        }

        let url = endpoint
            .options
            .as_ref()
            .and_then(|o| o.url.clone())
            .or_else(|| endpoint.url.clone())
            .unwrap_or_default();

        let inner = GenericImageryProvider {
            url,
            external_type: parsed_ext_type,
            tile_credits: tile_credits.clone(),
            ..Default::default()
        };

        Ok(Self {
            asset_id,
            endpoint,
            external_type: parsed_ext_type,
            tile_credits,
            imagery_provider: Arc::new(inner),
        })
    }

    /// Resolves an asset endpoint through a cached fetcher function.
    pub fn from_asset_id_with_fetcher<F>(
        asset_id: i64,
        options: Option<&IonImageryProviderOptions>,
        fetcher: F,
    ) -> Result<Self, RuntimeError>
    where
        F: FnOnce(i64, Option<&IonImageryProviderOptions>) -> Result<IonEndpoint, RuntimeError>,
    {
        if asset_id <= 0 {
            return Err(RuntimeError::new("assetId must be positive"));
        }

        let token = options
            .and_then(|o| o.access_token.as_deref())
            .unwrap_or("");
        let server = options.and_then(|o| o.server.as_deref()).unwrap_or("");
        let cache_key = format!("{asset_id}:{token}:{server}");

        let cached_endpoint = {
            let cache = ENDPOINT_CACHE.lock().unwrap();
            cache.get(&cache_key).cloned()
        };

        let endpoint = if let Some(ep) = cached_endpoint {
            ep
        } else {
            let fetched = fetcher(asset_id, options)?;
            let mut cache = ENDPOINT_CACHE.lock().unwrap();
            cache.insert(cache_key, fetched.clone());
            fetched
        };

        Self::from_endpoint(asset_id, endpoint, options)
    }

    /// Gets tile credits, combining server credits and underlying imagery credits.
    pub fn get_tile_credits(&self, _x: u32, _y: u32, _level: u32) -> Vec<Credit> {
        self.tile_credits.clone()
    }
}

impl ImageryProvider for IonImageryProvider {
    fn rectangle(&self) -> GeoRectangle {
        self.imagery_provider.rectangle()
    }
    fn tile_width(&self) -> u32 {
        self.imagery_provider.tile_width()
    }
    fn tile_height(&self) -> u32 {
        self.imagery_provider.tile_height()
    }
    fn minimum_level(&self) -> u32 {
        self.imagery_provider.minimum_level()
    }
    fn maximum_level(&self) -> Option<u32> {
        self.imagery_provider.maximum_level()
    }
    fn has_alpha_channel(&self) -> bool {
        self.imagery_provider.has_alpha_channel()
    }
    fn is_ready(&self) -> bool {
        self.imagery_provider.is_ready()
    }
    fn pick_features(
        &self,
        x: u32,
        y: u32,
        level: u32,
        longitude: f64,
        latitude: f64,
    ) -> Option<Vec<ImageryLayerFeatureInfo>> {
        self.imagery_provider
            .pick_features(x, y, level, longitude, latitude)
    }
}
