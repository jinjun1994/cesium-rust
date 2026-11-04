use crate::DeveloperError;
use std::collections::BTreeMap;
use std::fmt::Write;
use url::Url;

/// Given a relative Uri and a base Uri, returns the absolute Uri of the relative Uri.
pub fn get_absolute_uri(relative: &str, base: Option<&str>) -> Result<String, DeveloperError> {
    if relative.is_empty() {
        return Ok(String::new());
    }
    if let Ok(parsed) = Url::parse(relative) {
        if !parsed.scheme().is_empty() {
            return Ok(parsed.to_string());
        }
    }

    if let Some(base_str) = base {
        let base_url = Url::parse(base_str)
            .map_err(|_| DeveloperError::new("Invalid base URI"))?;
        let joined = base_url
            .join(relative)
            .map_err(|_| DeveloperError::new("Failed to resolve relative URI"))?;
        Ok(joined.to_string())
    } else {
        Ok(relative.to_string())
    }
}

/// Given a URI, returns the base path of the URI.
pub fn get_base_uri(uri: &str, include_query: bool) -> String {
    let clean_uri = if let Some(hash_idx) = uri.find('#') {
        &uri[..hash_idx]
    } else {
        uri
    };
    let without_query = if let Some(q_idx) = clean_uri.find('?') {
        &clean_uri[..q_idx]
    } else {
        clean_uri
    };
    let base_path = if let Some(last_slash) = without_query.rfind('/') {
        &without_query[..=last_slash]
    } else {
        ""
    };

    if !include_query {
        base_path.to_string()
    } else {
        let mut result = base_path.to_string();
        if let Some(q_idx) = uri.find('?') {
            let query_part = if let Some(h_idx) = uri.find('#') {
                if h_idx > q_idx {
                    &uri[q_idx..h_idx]
                } else {
                    &uri[q_idx..]
                }
            } else {
                &uri[q_idx..]
            };
            result.push_str(query_part);
        }
        if let Some(h_idx) = uri.find('#') {
            result.push_str(&uri[h_idx..]);
        }
        result
    }
}

/// Given a URI, returns the last segment of the URI, removing any path or query information.
pub fn get_filename_from_uri(uri: &str) -> String {
    let without_hash = match uri.find('#') {
        Some(idx) => &uri[..idx],
        None => uri,
    };
    let without_query = match without_hash.find('?') {
        Some(idx) => &without_hash[..idx],
        None => without_hash,
    };
    match without_query.rfind('/') {
        Some(idx) => without_query[idx + 1..].to_string(),
        None => without_query.to_string(),
    }
}

/// Given a URI, returns the file extension of the URI.
pub fn get_extension_from_uri(uri: &str) -> String {
    let filename = get_filename_from_uri(uri);
    match filename.rfind('.') {
        Some(idx) => filename[idx + 1..].to_string(),
        None => String::new(),
    }
}

pub fn percent_encode(s: &str) -> String {
    let mut encoded = String::new();
    for b in s.bytes() {
        if b.is_ascii_alphanumeric() || b == b'-' || b == b'_' || b == b'.' || b == b'~' {
            encoded.push(b as char);
        } else {
            let _ = write!(encoded, "%{:02X}", b);
        }
    }
    encoded
}

pub fn percent_decode(s: &str) -> String {
    let bytes = s.as_bytes();
    let mut decoded = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'+' {
            decoded.push(b' ');
            i += 1;
        } else if bytes[i] == b'%' && i + 2 < bytes.len() {
            if let Ok(val) =
                u8::from_str_radix(std::str::from_utf8(&bytes[i + 1..i + 3]).unwrap_or(""), 16)
            {
                decoded.push(val);
                i += 3;
            } else {
                decoded.push(bytes[i]);
                i += 1;
            }
        } else {
            decoded.push(bytes[i]);
            i += 1;
        }
    }
    String::from_utf8_lossy(&decoded).into_owned()
}

/// Converts a map or list of key-value pairs into a URL query string.
pub fn object_to_query(pairs: &[(&str, &[&str])]) -> String {
    let mut result = String::new();
    let mut first = true;
    for &(key, values) in pairs {
        let enc_key = percent_encode(key);
        for &val in values {
            if !first {
                result.push('&');
            }
            first = false;
            result.push_str(&enc_key);
            result.push('=');
            result.push_str(&percent_encode(val));
        }
    }
    result
}

/// Decodes a URL query string into a BTreeMap of keys to list of string values.
pub fn query_to_object(query: &str) -> BTreeMap<String, Vec<String>> {
    let mut map: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let clean = query.strip_prefix('?').unwrap_or(query);
    if clean.is_empty() {
        return map;
    }

    for segment in clean.split(['&', ';']) {
        if segment.is_empty() {
            continue;
        }
        let (k, v) = match segment.split_once('=') {
            Some((key, val)) => (percent_decode(key), percent_decode(val)),
            None => (percent_decode(segment), String::new()),
        };
        map.entry(k).or_default().push(v);
    }

    map
}

/// Determines if the specified URI is a blob URI (`^blob:` case-insensitive).
pub fn is_blob_uri(uri: &str) -> bool {
    if uri.len() < 5 {
        return false;
    }
    uri[..5].eq_ignore_ascii_case("blob:")
}

/// Determines if the specified URI is a data URI (`^data:` case-insensitive).
pub fn is_data_uri(uri: &str) -> bool {
    if uri.len() < 5 {
        return false;
    }
    uri[..5].eq_ignore_ascii_case("data:")
}

use std::sync::RwLock;

static PAGE_LOCATION: RwLock<Option<String>> = RwLock::new(None);

/// Sets the current page location used for origin comparison in `is_cross_origin_url`.
pub fn set_page_location(url: &str) {
    if let Ok(mut lock) = PAGE_LOCATION.write() {
        *lock = Some(url.to_string());
    }
}

/// Gets the current page location.
pub fn get_page_location() -> String {
    PAGE_LOCATION
        .read()
        .ok()
        .and_then(|lock| lock.clone())
        .unwrap_or_else(|| "http://localhost:8080/index.html".to_string())
}

/// Determines whether a URL is considered cross-origin to the current page.
pub fn is_cross_origin_url(url: &str) -> bool {
    let target_url = match Url::parse(url) {
        Ok(u) => u,
        Err(_) => return false, // Relative URLs are not cross-origin
    };

    let page_str = get_page_location();
    let page_url = match Url::parse(&page_str) {
        Ok(u) => u,
        Err(_) => return true,
    };

    target_url.scheme() != page_url.scheme()
        || target_url.host_str() != page_url.host_str()
        || target_url.port_or_known_default() != page_url.port_or_known_default()
}
