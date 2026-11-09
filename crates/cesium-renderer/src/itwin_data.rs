//! ITwinPlatform and ITwinData matching CesiumJS `Scene/ITwinData.js` and `Core/ITwinPlatform.js` (Apache-2.0).

use std::fmt;
use std::sync::{Arc, Mutex, OnceLock};

use cesium_math::{DeveloperError, Resource, RuntimeError};
use serde::{Deserialize, Serialize};

/// Status states for an iTwin mesh export.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExportStatus {
    NotStarted,
    InProgress,
    Complete,
    Invalid,
}

impl fmt::Display for ExportStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotStarted => write!(f, "NotStarted"),
            Self::InProgress => write!(f, "InProgress"),
            Self::Complete => write!(f, "Complete"),
            Self::Invalid => write!(f, "Invalid"),
        }
    }
}

/// Types of mesh-export exports.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExportType {
    IModel,
    Cesium,
    #[serde(rename = "3DTILES")]
    ThreeDTiles,
}

/// Types of Reality data supported by the iTwin platform.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum RealityDataType {
    Cesium3DTiles,
    #[serde(rename = "PNTS")]
    Pnts,
    RealityMesh3DTiles,
    Terrain3DTiles,
    #[serde(rename = "GS_3DT")]
    GaussianSplat3DTiles,
    #[serde(rename = "KML")]
    Kml,
    #[serde(rename = "GeoJSON")]
    GeoJson,
    Unstructured,
}

impl RealityDataType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Cesium3DTiles => "Cesium3DTiles",
            Self::Pnts => "PNTS",
            Self::RealityMesh3DTiles => "RealityMesh3DTiles",
            Self::Terrain3DTiles => "Terrain3DTiles",
            Self::GaussianSplat3DTiles => "GS_3DT",
            Self::Kml => "KML",
            Self::GeoJson => "GeoJSON",
            Self::Unstructured => "Unstructured",
        }
    }

    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "Cesium3DTiles" => Some(Self::Cesium3DTiles),
            "PNTS" => Some(Self::Pnts),
            "RealityMesh3DTiles" => Some(Self::RealityMesh3DTiles),
            "Terrain3DTiles" => Some(Self::Terrain3DTiles),
            "GS_3DT" | "GaussianSplat3DTiles" => Some(Self::GaussianSplat3DTiles),
            "KML" => Some(Self::Kml),
            "GeoJSON" => Some(Self::GeoJson),
            "Unstructured" => Some(Self::Unstructured),
            _ => None,
        }
    }
}

/// Representation of an iTwin mesh export entry.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ITwinExportRepresentation {
    pub id: String,
    #[serde(rename = "displayName", default)]
    pub display_name: String,
    pub status: ExportStatus,
    #[serde(rename = "lastModified", default)]
    pub last_modified: String,
    pub mesh_href: Option<String>,
}

/// Settings and state for accessing the iTwin platform.
#[derive(Clone, Debug)]
pub struct ITwinPlatformState {
    pub default_access_token: Option<String>,
    pub default_share_key: Option<String>,
    pub api_endpoint: String,
}

impl Default for ITwinPlatformState {
    fn default() -> Self {
        Self {
            default_access_token: None,
            default_share_key: None,
            api_endpoint: "https://api.bentley.com/".to_string(),
        }
    }
}

static PLATFORM_STATE: OnceLock<Arc<Mutex<ITwinPlatformState>>> = OnceLock::new();

fn platform_state() -> Arc<Mutex<ITwinPlatformState>> {
    PLATFORM_STATE
        .get_or_init(|| Arc::new(Mutex::new(ITwinPlatformState::default())))
        .clone()
}

/// Global settings and utilities for ITwinPlatform.
pub struct ITwinPlatform;

impl ITwinPlatform {
    pub fn set_default_access_token(token: Option<String>) {
        if let Ok(mut state) = platform_state().lock() {
            state.default_access_token = token;
        }
    }

    pub fn default_access_token() -> Option<String> {
        platform_state().lock().ok().and_then(|s| s.default_access_token.clone())
    }

    pub fn set_default_share_key(key: Option<String>) {
        if let Ok(mut state) = platform_state().lock() {
            state.default_share_key = key;
        }
    }

    pub fn default_share_key() -> Option<String> {
        platform_state().lock().ok().and_then(|s| s.default_share_key.clone())
    }

    pub fn set_api_endpoint(endpoint: String) {
        if let Ok(mut state) = platform_state().lock() {
            state.api_endpoint = endpoint;
        }
    }

    pub fn api_endpoint() -> String {
        platform_state()
            .lock()
            .map(|s| s.api_endpoint.clone())
            .unwrap_or_else(|_| "https://api.bentley.com/".to_string())
    }

    pub fn get_authorization_header() -> Result<String, DeveloperError> {
        let binding = platform_state();
        let state = binding
            .lock()
            .map_err(|_| DeveloperError::new("Failed to lock ITwinPlatform state"))?;

        if let Some(share_key) = &state.default_share_key {
            Ok(format!("Basic {share_key}"))
        } else if let Some(token) = &state.default_access_token {
            Ok(format!("Bearer {token}"))
        } else {
            Err(DeveloperError::new(
                "Must set ITwinPlatform.defaultAccessToken or ITwinPlatform.defaultShareKey first",
            ))
        }
    }
}

/// Tileset options passed to ITwinData constructors.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ITwinTilesetOptions {
    #[serde(rename = "maximumScreenSpaceError")]
    pub maximum_screen_space_error: Option<f64>,
    #[serde(rename = "cacheBytes")]
    pub cache_bytes: Option<u64>,
    pub show: Option<bool>,
}

fn append_tileset_json_preserving_query(href: &str) -> String {
    if let Some((path, query)) = href.split_once('?') {
        let clean_path = path.trim_end_matches('/');
        format!("{clean_path}/tileset.json?{query}")
    } else {
        let clean_path = href.trim_end_matches('/');
        format!("{clean_path}/tileset.json")
    }
}

/// Methods for loading iTwin platform data into Cesium.
pub struct ITwinData;

impl ITwinData {
    /// Create tileset URL and options from an iModel exports collection.
    pub fn create_tileset_from_exports(
        i_model_id: &str,
        exports: &[ITwinExportRepresentation],
        tileset_options: Option<ITwinTilesetOptions>,
    ) -> Result<Option<(String, Option<ITwinTilesetOptions>)>, RuntimeError> {
        if !exports.is_empty() && exports.iter().all(|e| e.status == ExportStatus::Invalid) {
            return Err(RuntimeError::new(format!(
                "All exports for this iModel are Invalid: {i_model_id}"
            )));
        }

        let complete_export = match exports.iter().find(|e| e.status == ExportStatus::Complete) {
            Some(e) => e,
            None => return Ok(None),
        };

        let mesh_href = match &complete_export.mesh_href {
            Some(href) => href,
            None => return Ok(None),
        };

        let tileset_url = append_tileset_json_preserving_query(mesh_href);
        Ok(Some((tileset_url, tileset_options)))
    }

    /// Resolves tileset access URL and merged options for reality data.
    pub fn create_tileset_for_reality_data(
        i_twin_id: Option<&str>,
        reality_data_id: Option<&str>,
        reality_data_type: Option<&str>,
        root_document: Option<&str>,
        tileset_options: Option<ITwinTilesetOptions>,
        metadata_fallback: Option<(&str, &str)>, // (type, rootDocument)
        access_url_fn: impl FnOnce(&str, &str, &str) -> String,
    ) -> Result<(String, ITwinTilesetOptions), RuntimeError> {
        let twin_id = i_twin_id.ok_or_else(|| {
            RuntimeError::new("Expected iTwinId to be typeof string, actual typeof was undefined")
        })?;
        let r_id = reality_data_id.ok_or_else(|| {
            RuntimeError::new("Expected realityDataId to be typeof string, actual typeof was undefined")
        })?;

        let (final_type_str, final_root_doc) = match (reality_data_type, root_document) {
            (Some(t), Some(r)) => (t, r),
            _ => match metadata_fallback {
                Some((t, r)) => (t, r),
                None => {
                    return Err(RuntimeError::new("Missing type or rootDocument metadata"));
                }
            },
        };

        let parsed_type = RealityDataType::from_name(final_type_str).ok_or_else(|| {
            RuntimeError::new(format!("Reality data type is not a mesh type: {final_type_str}"))
        })?;

        let supported_mesh_types = [
            RealityDataType::Cesium3DTiles,
            RealityDataType::Pnts,
            RealityDataType::RealityMesh3DTiles,
            RealityDataType::Terrain3DTiles,
            RealityDataType::GaussianSplat3DTiles,
        ];

        if !supported_mesh_types.contains(&parsed_type) {
            return Err(RuntimeError::new(format!(
                "Reality data type is not a mesh type: {final_type_str}"
            )));
        }

        let access_url = access_url_fn(twin_id, r_id, final_root_doc);

        let mut internal_options = tileset_options.unwrap_or_default();
        if internal_options.maximum_screen_space_error.is_none() {
            internal_options.maximum_screen_space_error = Some(4.0);
        }

        Ok((access_url, internal_options))
    }

    /// Resolves data source URL and type for reality data (GeoJSON or KML).
    pub fn create_data_source_for_reality_data(
        i_twin_id: Option<&str>,
        reality_data_id: Option<&str>,
        reality_data_type: Option<&str>,
        root_document: Option<&str>,
        metadata_fallback: Option<(&str, &str)>,
        access_url_fn: impl FnOnce(&str, &str, &str) -> String,
    ) -> Result<(RealityDataType, String), RuntimeError> {
        let twin_id = i_twin_id.ok_or_else(|| {
            RuntimeError::new("Expected iTwinId to be typeof string, actual typeof was undefined")
        })?;
        let r_id = reality_data_id.ok_or_else(|| {
            RuntimeError::new("Expected realityDataId to be typeof string, actual typeof was undefined")
        })?;

        let (final_type_str, final_root_doc) = match (reality_data_type, root_document) {
            (Some(t), Some(r)) => (t, r),
            _ => match metadata_fallback {
                Some((t, r)) => (t, r),
                None => {
                    return Err(RuntimeError::new("Missing type or rootDocument metadata"));
                }
            },
        };

        let parsed_type = RealityDataType::from_name(final_type_str).ok_or_else(|| {
            RuntimeError::new(format!(
                "Reality data type is not a data source type: {final_type_str}"
            ))
        })?;

        if parsed_type != RealityDataType::GeoJson && parsed_type != RealityDataType::Kml {
            return Err(RuntimeError::new(format!(
                "Reality data type is not a data source type: {final_type_str}"
            )));
        }

        let access_url = access_url_fn(twin_id, r_id, final_root_doc);
        Ok((parsed_type, access_url))
    }

    /// Loads geospatial features URL and resource.
    pub fn load_geospatial_features_resource(
        i_twin_id: Option<&str>,
        collection_id: Option<&str>,
        limit: Option<u32>,
    ) -> Result<Resource, DeveloperError> {
        let twin_id = i_twin_id.ok_or_else(|| {
            DeveloperError::new("Expected iTwinId to be typeof string, actual typeof was undefined")
        })?;
        let coll_id = collection_id.ok_or_else(|| {
            DeveloperError::new("Expected collectionId to be typeof string, actual typeof was undefined")
        })?;

        let page_limit = limit.unwrap_or(10000);
        if page_limit < 1 {
            return Err(DeveloperError::new(format!(
                "Expected limit to be greater than or equal to 1, actual value was {page_limit}"
            )));
        }
        if page_limit > 10000 {
            return Err(DeveloperError::new(format!(
                "Expected limit to be less than or equal to 10000, actual value was {page_limit}"
            )));
        }

        let auth_header = ITwinPlatform::get_authorization_header()?;
        let api_endpoint = ITwinPlatform::api_endpoint();
        let clean_endpoint = api_endpoint.trim_end_matches('/');

        let url = format!(
            "{clean_endpoint}/geospatial-features/itwins/{twin_id}/ogc/collections/{coll_id}/items?limit={page_limit}&client=CesiumJS"
        );

        let mut resource = Resource::from_url(&url);
        resource.headers.insert("Authorization".to_string(), auth_header);
        resource.headers.insert(
            "Accept".to_string(),
            "application/vnd.bentley.itwin-platform.v1+json".to_string(),
        );

        Ok(resource)
    }
}
