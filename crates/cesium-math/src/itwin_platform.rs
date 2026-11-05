use serde_json::Value;

use crate::{DeveloperError, RuntimeError};

pub const DEFAULT_ITWIN_API_ENDPOINT: &str = "https://api.bentley.com/";

/// Status states for a mesh-export export.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ITwinExportStatus {
    NotStarted,
    InProgress,
    Complete,
    Invalid,
}

impl ITwinExportStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::NotStarted => "NotStarted",
            Self::InProgress => "InProgress",
            Self::Complete => "Complete",
            Self::Invalid => "Invalid",
        }
    }
}

/// Types of mesh-export exports.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ITwinExportType {
    IModel,
    Cesium,
    ThreeDTiles,
}

impl ITwinExportType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::IModel => "IMODEL",
            Self::Cesium => "CESIUM",
            Self::ThreeDTiles => "3DTILES",
        }
    }
}

/// Types of Reality data supported by iTwin platform.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ITwinRealityDataType {
    Cesium3DTiles,
    Pnts,
    RealityMesh3DTiles,
    Terrain3DTiles,
    GaussianSplat3DTiles,
    Kml,
    GeoJson,
    Unstructured,
}

impl ITwinRealityDataType {
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
}

/// Helper methods and configurations for accessing the Bentley iTwin Platform.
pub struct ITwinPlatform;

impl ITwinPlatform {
    /// Computes the Authorization HTTP header value.
    /// If share_key is provided, Basic auth is used; otherwise Bearer auth with access_token.
    pub fn get_authorization_header(
        access_token: Option<&str>,
        share_key: Option<&str>,
    ) -> Result<String, DeveloperError> {
        if let Some(key) = share_key {
            let trimmed = key.trim();
            if !trimmed.is_empty() {
                return Ok(format!("Basic {trimmed}"));
            }
        }

        if let Some(token) = access_token {
            let trimmed = token.trim();
            if !trimmed.is_empty() {
                return Ok(format!("Bearer {trimmed}"));
            }
        }

        Err(DeveloperError::new(
            "Must set ITwinPlatform.defaultAccessToken or ITwinPlatform.defaultShareKey first",
        ))
    }

    /// Combines a container URL with a rootDocument relative path.
    pub fn combine_reality_data_url(
        container_url: &str,
        root_document: &str,
    ) -> Result<String, RuntimeError> {
        let trimmed_url = container_url.trim();
        let trimmed_doc = root_document.trim();

        if trimmed_url.is_empty() {
            return Err(RuntimeError::new("containerUrl is required"));
        }
        if trimmed_doc.is_empty() {
            return Err(RuntimeError::new("rootDocument is required"));
        }

        let (base_and_path, query_part) = match trimmed_url.split_once('?') {
            Some((bp, q)) => (bp, Some(q)),
            None => (trimmed_url, None),
        };

        let mut out = base_and_path.trim_end_matches('/').to_string();
        out.push('/');
        out.push_str(trimmed_doc.trim_start_matches('/'));

        if let Some(q) = query_part {
            out.push('?');
            out.push_str(q);
        }

        Ok(out)
    }

    /// Formats an error from status code and JSON error payload.
    pub fn map_http_error(
        status_code: u16,
        response_body: &str,
        context_id: Option<&str>,
    ) -> RuntimeError {
        let parsed: Value = serde_json::from_str(response_body).unwrap_or(Value::Null);

        match status_code {
            401 => {
                let code = parsed
                    .get("error")
                    .and_then(|e| e.get("details"))
                    .and_then(|d| d.as_array())
                    .and_then(|arr| arr.first())
                    .and_then(|item| item.get("code"))
                    .and_then(|c| c.as_str())
                    .unwrap_or("");
                RuntimeError::new(format!(
                    "Unauthorized, bad token, wrong scopes or headers bad. {code}"
                ))
            }
            403 => RuntimeError::new("Not allowed, forbidden"),
            404 => RuntimeError::new(format!(
                "Reality data not found: {}",
                context_id.unwrap_or("unknown")
            )),
            422 => {
                let code = parsed
                    .get("error")
                    .and_then(|e| e.get("code"))
                    .and_then(|c| c.as_str())
                    .unwrap_or("");
                let msg = parsed
                    .get("error")
                    .and_then(|e| e.get("message"))
                    .and_then(|m| m.as_str())
                    .unwrap_or("");
                RuntimeError::new(format!("Unprocessable Entity:{code} {msg}"))
            }
            429 => RuntimeError::new("Too many requests"),
            code => RuntimeError::new(format!("Unknown request failure {code}")),
        }
    }
}
