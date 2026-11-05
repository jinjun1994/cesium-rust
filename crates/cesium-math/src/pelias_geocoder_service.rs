use std::collections::HashMap;

use serde_json::Value;

use crate::{Cartesian3, Credit, DeveloperError, GeoRectangle, GeocoderDestination};

/// The type of geocode query to perform.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum GeocodeType {
    #[default]
    Search,
    Autocomplete,
}

impl GeocodeType {
    pub fn endpoint(&self) -> &'static str {
        match self {
            Self::Search => "search",
            Self::Autocomplete => "autocomplete",
        }
    }
}

/// Attribution metadata for Pelias results.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PeliasAttribution {
    pub html: String,
    pub collapsible: bool,
}

/// Result returned from a Pelias geocoder query.
#[derive(Debug, Clone, PartialEq)]
pub struct PeliasGeocoderResult {
    pub display_name: String,
    pub destination: GeocoderDestination,
    pub attributions: Vec<PeliasAttribution>,
}

/// Provides geocoding via a Pelias server.
#[derive(Debug, Clone)]
pub struct PeliasGeocoderService {
    url: String,
    query_parameters: HashMap<String, String>,
}

impl PeliasGeocoderService {
    pub fn new(url: impl Into<String>) -> Result<Self, DeveloperError> {
        let mut url_str = url.into();
        if url_str.trim().is_empty() {
            return Err(DeveloperError::new("url is required."));
        }

        if !url_str.ends_with('/') {
            url_str.push('/');
        }

        Ok(Self {
            url: url_str,
            query_parameters: HashMap::new(),
        })
    }

    pub fn url(&self) -> &str {
        &self.url
    }

    pub fn query_parameters(&self) -> &HashMap<String, String> {
        &self.query_parameters
    }

    pub fn query_parameters_mut(&mut self) -> &mut HashMap<String, String> {
        &mut self.query_parameters
    }

    pub fn credit(&self) -> Option<&Credit> {
        None
    }

    /// Constructs the query URL for the specified GeocodeType.
    pub fn build_url(&self, query: &str, geocode_type: GeocodeType) -> String {
        let mut full_url = format!("{}{}", self.url, geocode_type.endpoint());
        full_url.push_str("?text=");
        full_url.push_str(query);

        for (k, v) in &self.query_parameters {
            full_url.push('&');
            full_url.push_str(k);
            full_url.push('=');
            full_url.push_str(v);
        }

        full_url
    }

    /// Parses the JSON response received from Pelias.
    pub fn parse_response(
        &self,
        json_str: &str,
    ) -> Result<Vec<PeliasGeocoderResult>, DeveloperError> {
        let value: Value = serde_json::from_str(json_str)
            .map_err(|e| DeveloperError::new(format!("Invalid JSON response: {e}")))?;

        let mut top_attributions = Vec::new();
        if let Some(attr_arr) = value.get("attributions").and_then(|v| v.as_array()) {
            for a in attr_arr {
                let html = a.get("html").and_then(|v| v.as_str()).unwrap_or("").to_string();
                let collapsible = a.get("collapsible").and_then(|v| v.as_bool()).unwrap_or(false);
                top_attributions.push(PeliasAttribution { html, collapsible });
            }
        }

        let features = match value.get("features").and_then(|v| v.as_array()) {
            Some(f) => f,
            None => return Ok(Vec::new()),
        };

        let mut out = Vec::with_capacity(features.len());
        for feat in features {
            let display_name = feat
                .get("properties")
                .and_then(|p| p.get("label"))
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();

            let destination = if let Some(bbox) = feat.get("bbox").and_then(|v| v.as_array()) {
                if bbox.len() == 4 {
                    let w = bbox[0].as_f64().unwrap_or(0.0);
                    let s = bbox[1].as_f64().unwrap_or(0.0);
                    let e = bbox[2].as_f64().unwrap_or(0.0);
                    let n = bbox[3].as_f64().unwrap_or(0.0);
                    let rect = GeoRectangle::degrees(w, s, e, n)
                        .map_err(|err| DeveloperError::new(format!("Invalid bbox: {err}")))?;
                    GeocoderDestination::Rectangle(rect)
                } else {
                    continue;
                }
            } else if let Some(geom) = feat.get("geometry") {
                let coords = geom.get("coordinates").and_then(|v| v.as_array());
                if let Some(c) = coords {
                    if c.len() >= 2 {
                        let lon = c[0].as_f64().unwrap_or(0.0);
                        let lat = c[1].as_f64().unwrap_or(0.0);
                        let cart = Cartesian3::from_degrees(lon, lat, Some(0.0), None)
                            .map_err(|err| DeveloperError::new(format!("Invalid coordinates: {err}")))?;
                        GeocoderDestination::Cartesian(cart)
                    } else {
                        continue;
                    }
                } else {
                    continue;
                }
            } else {
                continue;
            };

            out.push(PeliasGeocoderResult {
                display_name,
                destination,
                attributions: top_attributions.clone(),
            });
        }

        Ok(out)
    }
}
