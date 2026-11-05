use std::collections::HashMap;

use serde_json::Value;

use crate::{Cartesian3, Credit, DeveloperError, GeoRectangle};

/// Destination for geocoding results, either an area or a point.
#[derive(Debug, Clone, PartialEq)]
pub enum GeocoderDestination {
    Rectangle(GeoRectangle),
    Cartesian(Cartesian3),
}

/// Result returned from an OpenCage geocoder query.
#[derive(Debug, Clone, PartialEq)]
pub struct OpenCageGeocoderResult {
    pub display_name: String,
    pub destination: GeocoderDestination,
}

/// Provides geocoding via OpenCage server.
#[derive(Debug, Clone)]
pub struct OpenCageGeocoderService {
    url: String,
    api_key: String,
    params: HashMap<String, String>,
    credit: Credit,
}

impl OpenCageGeocoderService {
    pub fn new(
        url: impl Into<String>,
        api_key: impl Into<String>,
        params: Option<HashMap<String, String>>,
    ) -> Result<Self, DeveloperError> {
        let mut url_str = url.into();
        let api_key_str = api_key.into();

        if url_str.trim().is_empty() {
            return Err(DeveloperError::new("url is required."));
        }
        if api_key_str.trim().is_empty() {
            return Err(DeveloperError::new("apiKey is required."));
        }

        if !url_str.ends_with('/') {
            url_str.push('/');
        }

        let credit = Credit::new(
            r#"Geodata copyright <a href="https://www.openstreetmap.org/">OpenStreetMap</a> contributors"#,
            Some(false),
        );

        Ok(Self {
            url: url_str,
            api_key: api_key_str,
            params: params.unwrap_or_default(),
            credit,
        })
    }

    pub fn url(&self) -> &str {
        &self.url
    }

    pub fn api_key(&self) -> &str {
        &self.api_key
    }

    pub fn credit(&self) -> &Credit {
        &self.credit
    }

    pub fn params(&self) -> &HashMap<String, String> {
        &self.params
    }

    /// Constructs the query URL for OpenCage.
    pub fn build_url(&self, query: &str) -> String {
        let mut full_url = format!("{}json?key={}&q={}", self.url, self.api_key, query);
        for (k, v) in &self.params {
            full_url.push('&');
            full_url.push_str(k);
            full_url.push('=');
            full_url.push_str(v);
        }
        full_url
    }

    /// Parses the JSON response received from OpenCage.
    pub fn parse_response(
        &self,
        json_str: &str,
    ) -> Result<Vec<OpenCageGeocoderResult>, DeveloperError> {
        let value: Value = serde_json::from_str(json_str)
            .map_err(|e| DeveloperError::new(format!("Invalid JSON response: {e}")))?;

        let results_arr = match value.get("results").and_then(|v| v.as_array()) {
            Some(arr) => arr,
            None => return Ok(Vec::new()),
        };

        let mut out = Vec::with_capacity(results_arr.len());
        for res in results_arr {
            let display_name = res
                .get("formatted")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();

            let destination = if let Some(bounds) = res.get("bounds") {
                let sw = bounds.get("southwest");
                let ne = bounds.get("northeast");
                let sw_lat = sw.and_then(|p| p.get("lat")).and_then(|v| v.as_f64()).unwrap_or(0.0);
                let sw_lng = sw.and_then(|p| p.get("lng")).and_then(|v| v.as_f64()).unwrap_or(0.0);
                let ne_lat = ne.and_then(|p| p.get("lat")).and_then(|v| v.as_f64()).unwrap_or(0.0);
                let ne_lng = ne.and_then(|p| p.get("lng")).and_then(|v| v.as_f64()).unwrap_or(0.0);

                let rect = GeoRectangle::degrees(sw_lng, sw_lat, ne_lng, ne_lat)
                    .map_err(|e| DeveloperError::new(format!("Invalid bounds rectangle: {e}")))?;
                GeocoderDestination::Rectangle(rect)
            } else if let Some(geom) = res.get("geometry") {
                let lat = geom.get("lat").and_then(|v| v.as_f64()).unwrap_or(0.0);
                let lng = geom.get("lng").and_then(|v| v.as_f64()).unwrap_or(0.0);
                let cart = Cartesian3::from_degrees(lng, lat, Some(0.0), None)
                    .map_err(|e| DeveloperError::new(format!("Invalid cartesian coordinates: {e}")))?;
                GeocoderDestination::Cartesian(cart)
            } else {
                continue;
            };

            out.push(OpenCageGeocoderResult {
                display_name,
                destination,
            });
        }

        Ok(out)
    }
}
