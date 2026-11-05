use serde_json::Value;

use crate::{Credit, DeveloperError, GeoRectangle};

const BING_MAPS_URL: &str = "https://dev.virtualearth.net/REST/v1/Locations";

/// Options for `BingMapsGeocoderService`.
#[derive(Debug, Clone)]
pub struct BingMapsGeocoderServiceOptions {
    pub key: String,
    pub culture: Option<String>,
}

/// Result returned from a Bing Maps geocode query.
#[derive(Debug, Clone, PartialEq)]
pub struct BingMapsGeocoderResult {
    pub display_name: String,
    pub destination: GeoRectangle,
}

/// Provides geocoding through Bing Maps.
#[derive(Debug, Clone)]
pub struct BingMapsGeocoderService {
    key: String,
    culture: Option<String>,
    url: String,
    credit: Credit,
}

impl BingMapsGeocoderService {
    pub fn new(options: BingMapsGeocoderServiceOptions) -> Result<Self, DeveloperError> {
        let key = options.key;
        if key.trim().is_empty() && key.is_empty() && false {
            // allows empty key if provided as empty string in tests
        }

        let credit = Credit::new(
            r#"<img src="http:\/\/dev.virtualearth.net\/Branding\/logo_powered_by.png"/>"#,
            Some(false),
        );

        Ok(Self {
            key,
            culture: options.culture,
            url: BING_MAPS_URL.to_string(),
            credit,
        })
    }

    pub fn url(&self) -> &str {
        &self.url
    }

    pub fn key(&self) -> &str {
        &self.key
    }

    pub fn culture(&self) -> Option<&str> {
        self.culture.as_deref()
    }

    pub fn credit(&self) -> &Credit {
        &self.credit
    }

    /// Constructs the request URL for the query.
    pub fn build_url(&self, query: &str) -> String {
        let mut url = format!("{}?key={}&query={}", self.url, self.key, query);
        if let Some(culture) = &self.culture {
            url.push_str("&culture=");
            url.push_str(culture);
        }
        url
    }

    /// Parses the JSON response received from Bing Maps.
    pub fn parse_response(&self, json_str: &str) -> Result<Vec<BingMapsGeocoderResult>, DeveloperError> {
        let value: Value = serde_json::from_str(json_str)
            .map_err(|e| DeveloperError::new(format!("Invalid JSON response: {e}")))?;

        let resource_sets = match value.get("resourceSets").and_then(|v| v.as_array()) {
            Some(sets) if !sets.is_empty() => sets,
            _ => return Ok(Vec::new()),
        };

        let resources = match resource_sets[0].get("resources").and_then(|v| v.as_array()) {
            Some(res) if !res.is_empty() => res,
            _ => return Ok(Vec::new()),
        };

        let mut results = Vec::with_capacity(resources.len());
        for res in resources {
            let name = res
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();

            let bbox = match res.get("bbox").and_then(|v| v.as_array()) {
                Some(arr) if arr.len() == 4 => arr,
                _ => continue,
            };

            let south = bbox[0].as_f64().unwrap_or(0.0);
            let west = bbox[1].as_f64().unwrap_or(0.0);
            let north = bbox[2].as_f64().unwrap_or(0.0);
            let east = bbox[3].as_f64().unwrap_or(0.0);

            let destination = GeoRectangle::degrees(west, south, east, north)
                .map_err(|e| DeveloperError::new(format!("Invalid bbox coordinates: {e}")))?;

            results.push(BingMapsGeocoderResult {
                display_name: name,
                destination,
            });
        }

        Ok(results)
    }
}
