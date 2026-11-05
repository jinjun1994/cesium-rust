use std::collections::HashMap;

/// Trait to allow flexible inputs (None, (), &str, Option<&str>) to `parse_response_headers`.
pub trait IntoHeaderStr<'a> {
    fn into_header_str(self) -> Option<&'a str>;
}

impl<'a> IntoHeaderStr<'a> for Option<&'a str> {
    fn into_header_str(self) -> Option<&'a str> {
        self
    }
}

impl<'a> IntoHeaderStr<'a> for &'a str {
    fn into_header_str(self) -> Option<&'a str> {
        Some(self)
    }
}

impl<'a> IntoHeaderStr<'a> for &'a String {
    fn into_header_str(self) -> Option<&'a str> {
        Some(self.as_str())
    }
}

impl IntoHeaderStr<'_> for () {
    fn into_header_str(self) -> Option<&'static str> {
        None
    }
}

/// Parses raw HTTP response headers into a dictionary of key-value pairs matching CesiumJS `Core/parseResponseHeaders`.
pub fn parse_response_headers<'a>(input: impl IntoHeaderStr<'a>) -> HashMap<String, String> {
    let Some(raw_headers) = input.into_header_str() else {
        return HashMap::new();
    };
    if raw_headers.is_empty() {
        return HashMap::new();
    }
    let mut map = HashMap::new();
    for line in raw_headers.split("\r\n") {
        if line.is_empty() {
            continue;
        }
        if let Some(index) = line.find(": ") {
            if index > 0 {
                let key = &line[..index];
                let val = &line[index + 2..];
                map.insert(key.to_string(), val.to_string());
            }
        } else if let Some((key, val)) = line.split_once(':') {
            let key = key.trim();
            if !key.is_empty() {
                map.insert(key.to_string(), val.trim_start().to_string());
            }
        }
    }
    map
}

/// An event that is raised when a request encounters an error.
#[derive(Debug, Clone, PartialEq)]
pub struct RequestErrorEvent {
    pub status_code: Option<u16>,
    pub response: Option<String>,
    pub response_headers: HashMap<String, String>,
}

impl RequestErrorEvent {
    pub fn new(
        status_code: Option<u16>,
        response: Option<String>,
        headers: Option<HashMap<String, String>>,
    ) -> Self {
        Self {
            status_code,
            response,
            response_headers: headers.unwrap_or_default(),
        }
    }

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
