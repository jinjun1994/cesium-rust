//! GoogleStreetViewCubeMapPanoramaProvider matching CesiumJS `Scene/GoogleStreetViewCubeMapPanoramaProvider.js` (Apache-2.0).

use crate::cube_map::CubeMapUrls;
use crate::cube_map_panorama::{CubeMapPanorama, CubeMapPanoramaOptions};
use cesium_math::{Cartographic, DeveloperError, Matrix4};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::future::Future;
use std::sync::Mutex;

pub const DEFAULT_STREET_VIEW_ENDPOINT: &str = "https://maps.googleapis.com/maps/api/streetview";
pub const DEFAULT_TILE_SIZE: u32 = 600;

// Global default API keys matching CesiumJS GoogleMaps
static DEFAULT_STATIC_KEY: Mutex<Option<String>> = Mutex::new(None);
static DEFAULT_KEY: Mutex<Option<String>> = Mutex::new(None);

/// Sets the default Street View Static API key.
pub fn set_default_street_view_static_api_key(key: Option<String>) {
    let mut lock = DEFAULT_STATIC_KEY.lock().unwrap();
    *lock = key;
}

/// Sets the default Google API key.
pub fn set_default_google_api_key(key: Option<String>) {
    let mut lock = DEFAULT_KEY.lock().unwrap();
    *lock = key;
}

fn get_default_key() -> Option<String> {
    if let Some(key) = DEFAULT_STATIC_KEY.lock().unwrap().clone() {
        return Some(key);
    }
    DEFAULT_KEY.lock().unwrap().clone()
}

/// Initialization options for [`GoogleStreetViewCubeMapPanoramaProvider`].
#[derive(Debug, Clone, Default)]
pub struct GoogleStreetViewCubeMapPanoramaProviderOptions {
    /// Google Street View Static API key.
    pub key: Option<String>,
    /// Google Street View API endpoint. Default "https://maps.googleapis.com/maps/api/streetview".
    pub url: Option<String>,
    /// Tile size (square width/height in pixels). Default 600.
    pub tile_size: Option<u32>,
    /// Credit string.
    pub credit: Option<String>,
}

/// Result of nearest panorama ID query.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PanoLocation {
    pub pano_id: String,
    pub latitude: f64,
    pub longitude: f64,
}

/// Face build options for Google Street View static imagery.
#[derive(Debug, Clone)]
pub struct FaceUrlOptions {
    pub heading: i32,
    pub pitch: i32,
    pub tile_size_string: String,
    pub pano_id: String,
    pub signature: Option<String>,
}

/// Options for loading a panorama.
#[derive(Debug, Clone, Default)]
pub struct LoadPanoramaOptions {
    pub cartographic: Option<Cartographic>,
    pub pano_id: Option<String>,
    pub tile_size: Option<u32>,
    pub signature: Option<String>,
    pub credit: Option<String>,
}

/// Provides imagery from Google Street View Static API for display in a cube map panorama.
#[derive(Debug, Clone)]
pub struct GoogleStreetViewCubeMapPanoramaProvider {
    key: String,
    url: String,
    tile_size: u32,
    credit: Option<String>,
}

impl GoogleStreetViewCubeMapPanoramaProvider {
    /// Creates a new `GoogleStreetViewCubeMapPanoramaProvider`.
    pub fn new(options: GoogleStreetViewCubeMapPanoramaProviderOptions) -> Self {
        let key = options
            .key
            .or_else(get_default_key)
            .unwrap_or_default();
        let url = options
            .url
            .unwrap_or_else(|| DEFAULT_STREET_VIEW_ENDPOINT.to_string());
        let tile_size = options.tile_size.unwrap_or(DEFAULT_TILE_SIZE);

        Self {
            key,
            url,
            tile_size,
            credit: options.credit,
        }
    }

    /// Asynchronously creates a provider from options.
    pub async fn from_url(
        options: GoogleStreetViewCubeMapPanoramaProviderOptions,
    ) -> Result<Self, DeveloperError> {
        let key = options.key.clone().or_else(get_default_key);
        if key.is_none() {
            return Err(DeveloperError::new(
                "options.key, GoogleMaps.defaultStreetViewStaticApiKey or GoogleMaps.defaultApiKey is required.",
            ));
        }

        let mut opts = options;
        opts.key = key;
        Ok(Self::new(opts))
    }

    /// API key used by this provider.
    pub fn key(&self) -> &str {
        &self.key
    }

    /// Base URL endpoint.
    pub fn url(&self) -> &str {
        &self.url
    }

    /// Tile size in pixels.
    pub fn tile_size(&self) -> u32 {
        self.tile_size
    }

    /// Credit string.
    pub fn credit(&self) -> Option<&str> {
        self.credit.as_deref()
    }

    /// Builds a face image URL for given heading, pitch, size, and pano id.
    pub fn build_face_url(&self, options: &FaceUrlOptions) -> String {
        let mut base = self.url.clone();
        let query_sep = if base.contains('?') { '&' } else { '?' };
        base.push(query_sep);

        let mut query = format!(
            "size={}&pano={}&heading={}&pitch={}&key={}",
            options.tile_size_string, options.pano_id, options.heading, options.pitch, self.key
        );
        if let Some(sig) = &options.signature {
            query.push_str(&format!("&signature={sig}"));
        }
        base.push_str(&query);
        base
    }

    /// Gets the nearest panorama ID for a given Cartographic location.
    pub async fn get_nearest_pano_id<F, Fut>(
        &self,
        position: Option<Cartographic>,
        radius: Option<f64>,
        fetch_json: F,
    ) -> Result<PanoLocation, DeveloperError>
    where
        F: FnOnce(String) -> Fut,
        Fut: Future<Output = Result<Value, String>>,
    {
        let pos = position.ok_or_else(|| DeveloperError::new("position is required."))?;
        let lat_deg = pos.latitude.to_degrees();
        let lng_deg = pos.longitude.to_degrees();
        let r = radius.unwrap_or(50.0);

        let mut meta_url = self.url.trim_end_matches('/').to_string();
        if !meta_url.ends_with("/metadata") {
            meta_url.push_str("/metadata");
        }
        let sep = if meta_url.contains('?') { '&' } else { '?' };
        let req_url = format!(
            "{meta_url}{sep}key={}&location={lat_deg},{lng_deg}&radius={r}",
            self.key
        );

        let json = fetch_json(req_url)
            .await
            .map_err(DeveloperError::new)?;

        let status = json
            .get("status")
            .and_then(Value::as_str)
            .unwrap_or("ERROR");
        if status != "OK" {
            return Err(DeveloperError::new(format!(
                "GoogleStreetViewCubeMapPanoramaProvider metadata error: {status}"
            )));
        }

        let pano_id = json
            .get("pano_id")
            .and_then(Value::as_str)
            .ok_or_else(|| DeveloperError::new("Missing pano_id in metadata"))?
            .to_string();

        let loc = json
            .get("location")
            .ok_or_else(|| DeveloperError::new("Missing location in metadata"))?;
        let lat = loc.get("lat").and_then(Value::as_f64).unwrap_or(0.0);
        let lng = loc.get("lng").and_then(Value::as_f64).unwrap_or(0.0);

        Ok(PanoLocation {
            pano_id,
            latitude: lat,
            longitude: lng,
        })
    }

    /// Gets metadata for a specific panorama ID.
    pub async fn get_pano_id_metadata<F, Fut>(
        &self,
        pano_id: &str,
        fetch_json: F,
    ) -> Result<Value, DeveloperError>
    where
        F: FnOnce(String) -> Fut,
        Fut: Future<Output = Result<Value, String>>,
    {
        let mut meta_url = self.url.trim_end_matches('/').to_string();
        if !meta_url.ends_with("/metadata") {
            meta_url.push_str("/metadata");
        }
        let sep = if meta_url.contains('?') { '&' } else { '?' };
        let req_url = format!("{meta_url}{sep}key={}&pano={pano_id}", self.key);

        let json = fetch_json(req_url)
            .await
            .map_err(DeveloperError::new)?;

        let status = json
            .get("status")
            .and_then(Value::as_str)
            .unwrap_or("ERROR");
        if status != "OK" {
            return Err(DeveloperError::new(format!(
                "GoogleStreetViewCubeMapPanoramaProvider metadata error: {status}"
            )));
        }

        Ok(json)
    }

    /// Parses latitude and longitude from metadata into a Cartographic position.
    pub fn parse_metadata(metadata: &Value) -> Result<Cartographic, DeveloperError> {
        let loc = metadata
            .get("location")
            .ok_or_else(|| DeveloperError::new("Missing location in metadata"))?;
        let lat = loc
            .get("lat")
            .and_then(Value::as_f64)
            .ok_or_else(|| DeveloperError::new("Missing lat in location"))?;
        let lng = loc
            .get("lng")
            .and_then(Value::as_f64)
            .ok_or_else(|| DeveloperError::new("Missing lng in location"))?;

        Ok(Cartographic::from_radians(
            lng.to_radians(),
            lat.to_radians(),
        ))
    }

    /// Loads the cube map panorama faces.
    pub async fn load_panorama<F, Fut>(
        &self,
        options: LoadPanoramaOptions,
        fetch_image: F,
    ) -> Result<CubeMapPanorama, DeveloperError>
    where
        F: Fn(String) -> Fut + Clone,
        Fut: Future<Output = Result<Vec<u8>, String>>,
    {
        let _cartographic = options
            .cartographic
            .ok_or_else(|| DeveloperError::new("options.cartographic is required."))?;

        let tile_size = options.tile_size.unwrap_or(self.tile_size);
        let tile_size_string = format!("{tile_size}x{tile_size}");
        let pano_id = options.pano_id.unwrap_or_else(|| "default_pano".to_string());

        let faces = [
            FaceUrlOptions {
                heading: 0,
                pitch: 0,
                tile_size_string: tile_size_string.clone(),
                pano_id: pano_id.clone(),
                signature: options.signature.clone(),
            },
            FaceUrlOptions {
                heading: 180,
                pitch: 0,
                tile_size_string: tile_size_string.clone(),
                pano_id: pano_id.clone(),
                signature: options.signature.clone(),
            },
            FaceUrlOptions {
                heading: -90,
                pitch: -90,
                tile_size_string: tile_size_string.clone(),
                pano_id: pano_id.clone(),
                signature: options.signature.clone(),
            },
            FaceUrlOptions {
                heading: -90,
                pitch: 90,
                tile_size_string: tile_size_string.clone(),
                pano_id: pano_id.clone(),
                signature: options.signature.clone(),
            },
            FaceUrlOptions {
                heading: 270,
                pitch: 0,
                tile_size_string: tile_size_string.clone(),
                pano_id: pano_id.clone(),
                signature: options.signature.clone(),
            },
            FaceUrlOptions {
                heading: 90,
                pitch: 0,
                tile_size_string,
                pano_id,
                signature: options.signature,
            },
        ];

        let mut face_urls = Vec::with_capacity(6);
        for face_opt in &faces {
            let url = self.build_face_url(face_opt);
            fetch_image(url.clone())
                .await
                .map_err(|e| DeveloperError::new(format!("Failed to load face image {url}: {e}")))?;
            face_urls.push(url);
        }

        let sources = CubeMapUrls {
            positive_x: Some(face_urls[0].clone()),
            negative_x: Some(face_urls[1].clone()),
            positive_y: Some(face_urls[2].clone()),
            negative_y: Some(face_urls[3].clone()),
            positive_z: Some(face_urls[4].clone()),
            negative_z: Some(face_urls[5].clone()),
        };

        let credit = options.credit.or_else(|| {
            Some("Google Street View".to_string())
        });

        Ok(CubeMapPanorama::new(CubeMapPanoramaOptions {
            sources: Some(sources),
            transform: Some(Matrix4::IDENTITY),
            show: Some(true),
            credit,
        }))
    }
}
