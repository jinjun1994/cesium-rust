use serde_json::Value;

use crate::{Credit, DeveloperError, GeoRectangle, RuntimeError};

const API_URL: &str = "https://maps.googleapis.com/maps/api/geocode/json";
const CREDIT_HTML: &str =
    r#"<img alt="Google" src="https://assets.ion.cesium.com/google-credit.png" style="vertical-align:-5px">"#;

/// Options for `GoogleGeocoderService`.
#[derive(Debug, Clone)]
pub struct GoogleGeocoderServiceOptions {
    pub key: String,
}

/// Attribution metadata for Google Geocoder results.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GoogleGeocoderAttribution {
    pub html: String,
    pub collapsible: bool,
}

/// Result returned from a Google Geocoder query.
#[derive(Debug, Clone, PartialEq)]
pub struct GoogleGeocoderResult {
    pub display_name: String,
    pub destination: GeoRectangle,
    pub attribution: GoogleGeocoderAttribution,
}

/// Provides geocoding through Google Maps Geocoding API.
#[derive(Debug, Clone)]
pub struct GoogleGeocoderService {
    key: String,
    resource_url: String,
    credit: Credit,
}

impl GoogleGeocoderService {
    pub fn new(options: GoogleGeocoderServiceOptions) -> Result<Self, DeveloperError> {
        let key = options.key;
        if key.trim().is_empty() {
            return Err(DeveloperError::new("options.key is required."));
        }

        let resource_url = format!("{API_URL}?key={key}");
        let credit = Credit::new(CREDIT_HTML, Some(true));

        Ok(Self {
            key,
            resource_url,
            credit,
        })
    }

    pub fn key(&self) -> &str {
        &self.key
    }

    pub fn resource_url(&self) -> &str {
        &self.resource_url
    }

    pub fn credit(&self) -> &Credit {
        &self.credit
    }

    /// Constructs the query URL for the geocoder.
    pub fn build_url(&self, query: &str) -> String {
        format!("{}&address={}", self.resource_url, query)
    }

    /// Parses the JSON response received from Google Geocoding API.
    pub fn parse_response(&self, json_str: &str) -> Result<Vec<GoogleGeocoderResult>, RuntimeError> {
        let value: Value = serde_json::from_str(json_str)
            .map_err(|e| RuntimeError::new(format!("Invalid JSON response: {e}")))?;

        let status = value
            .get("status")
            .and_then(|v| v.as_str())
            .unwrap_or("UNKNOWN");

        if status == "ZERO_RESULTS" {
            return Ok(Vec::new());
        }

        if status != "OK" {
            let error_msg = value
                .get("error_message")
                .and_then(|v| v.as_str())
                .unwrap_or("Unknown error");
            return Err(RuntimeError::new(format!(
                "GoogleGeocoderService got a bad response {status}: {error_msg}"
            )));
        }

        let results_arr = match value.get("results").and_then(|v| v.as_array()) {
            Some(arr) => arr,
            None => return Ok(Vec::new()),
        };

        let mut out = Vec::with_capacity(results_arr.len());
        for res in results_arr {
            let display_name = res
                .get("formatted_address")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();

            let viewport = res
                .get("geometry")
                .and_then(|g| g.get("viewport"));

            let (sw_lat, sw_lng, ne_lat, ne_lng) = match viewport {
                Some(vp) => {
                    let sw = vp.get("southwest");
                    let ne = vp.get("northeast");
                    let sw_lat = sw.and_then(|p| p.get("lat")).and_then(|v| v.as_f64()).unwrap_or(0.0);
                    let sw_lng = sw.and_then(|p| p.get("lng")).and_then(|v| v.as_f64()).unwrap_or(0.0);
                    let ne_lat = ne.and_then(|p| p.get("lat")).and_then(|v| v.as_f64()).unwrap_or(0.0);
                    let ne_lng = ne.and_then(|p| p.get("lng")).and_then(|v| v.as_f64()).unwrap_or(0.0);
                    (sw_lat, sw_lng, ne_lat, ne_lng)
                }
                None => (0.0, 0.0, 0.0, 0.0),
            };

            let destination = GeoRectangle::degrees(sw_lng, sw_lat, ne_lng, ne_lat)
                .map_err(|e| RuntimeError::new(format!("Invalid viewport rectangle: {e}")))?;

            out.push(GoogleGeocoderResult {
                display_name,
                destination,
                attribution: GoogleGeocoderAttribution {
                    html: CREDIT_HTML.to_string(),
                    collapsible: false,
                },
            });
        }

        Ok(out)
    }
}
