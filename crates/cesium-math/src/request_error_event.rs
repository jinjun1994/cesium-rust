use std::collections::HashMap;

/// An event that is raised when a request encounters an error.
#[derive(Debug, Clone, PartialEq)]
pub struct RequestErrorEvent {
    pub status_code: Option<u16>,
    pub response: Option<String>,
    pub response_headers: HashMap<String, String>,
}

pub fn parse_response_headers(raw_headers: &str) -> HashMap<String, String> {
    let mut map = HashMap::new();
    for line in raw_headers.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        if let Some((key, val)) = line.split_once(':') {
            map.insert(key.trim().to_string(), val.trim().to_string());
        }
    }
    map
}

impl RequestErrorEvent {
    pub fn from_raw_headers(
        status_code: Option<u16>,
        response: Option<String>,
        raw_headers: &str,
    ) -> Self {
        Self {
            status_code,
            response,
            response_headers: parse_response_headers(raw_headers),
        }
    }

    pub fn from_headers_map(
        status_code: Option<u16>,
        response: Option<String>,
        headers: HashMap<String, String>,
    ) -> Self {
        Self {
            status_code,
            response,
            response_headers: headers,
        }
    }
}
