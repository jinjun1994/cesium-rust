use std::collections::{BTreeMap, HashMap};
use std::fmt;
use std::sync::Arc;

use base64::Engine;
use url::Url;

use crate::default_proxy::Proxy;
use crate::request::Request;
use crate::request_error_event::RequestErrorEvent;
use crate::uri_helpers::{
    get_extension_from_uri, is_blob_uri as helper_is_blob_uri,
    is_data_uri as helper_is_data_uri, percent_decode, percent_encode,
};
use crate::{Credit, DeveloperError};

/// Query parameter value which can be either a single string or an array of strings.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum QueryValue {
    Single(String),
    Array(Vec<String>),
}

impl QueryValue {
    pub fn to_vec(&self) -> Vec<String> {
        match self {
            Self::Single(s) => vec![s.clone()],
            Self::Array(v) => v.clone(),
        }
    }
}

impl From<&str> for QueryValue {
    fn from(s: &str) -> Self {
        Self::Single(s.to_string())
    }
}

impl From<String> for QueryValue {
    fn from(s: String) -> Self {
        Self::Single(s)
    }
}

impl From<i32> for QueryValue {
    fn from(v: i32) -> Self {
        Self::Single(v.to_string())
    }
}

impl From<f64> for QueryValue {
    fn from(v: f64) -> Self {
        Self::Single(v.to_string())
    }
}

impl From<Vec<String>> for QueryValue {
    fn from(v: Vec<String>) -> Self {
        Self::Array(v)
    }
}

impl From<Vec<&str>> for QueryValue {
    fn from(v: Vec<&str>) -> Self {
        Self::Array(v.into_iter().map(|s| s.to_string()).collect())
    }
}

impl<const N: usize> From<[&str; N]> for QueryValue {
    fn from(arr: [&str; N]) -> Self {
        Self::Array(arr.iter().map(|s| s.to_string()).collect())
    }
}

impl<const N: usize> From<[i32; N]> for QueryValue {
    fn from(arr: [i32; N]) -> Self {
        Self::Array(arr.iter().map(|v| v.to_string()).collect())
    }
}

pub type RetryCallback = Arc<dyn Fn(&Resource, Option<&RequestErrorEvent>) -> bool + Send + Sync>;

/// Initialization options for the Resource constructor.
#[derive(Clone, Default)]
pub struct ResourceOptions {
    pub url: String,
    pub query_parameters: BTreeMap<String, QueryValue>,
    pub template_values: HashMap<String, String>,
    pub headers: HashMap<String, String>,
    pub proxy: Option<Arc<dyn Proxy>>,
    pub retry_callback: Option<RetryCallback>,
    pub retry_attempts: u32,
    pub request: Option<Request>,
    pub parse_url: bool,
    pub credits: Option<Vec<Credit>>,
}

/// Options for deriving a resource from an existing resource.
#[derive(Clone, Default)]
pub struct DerivedResourceOptions {
    pub url: Option<String>,
    pub query_parameters: Option<BTreeMap<String, QueryValue>>,
    pub template_values: Option<HashMap<String, String>>,
    pub headers: Option<HashMap<String, String>>,
    pub proxy: Option<Arc<dyn Proxy>>,
    pub retry_callback: Option<RetryCallback>,
    pub retry_attempts: Option<u32>,
    pub request: Option<Request>,
    pub preserve_query_parameters: bool,
}

/// A decoded data URI payload.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataUriPayload {
    pub mime_type: String,
    pub is_base64: bool,
    pub data: Vec<u8>,
}

/// A simple Blob representation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Blob {
    pub mime_type: String,
    pub data: Vec<u8>,
}

/// A resource that includes the location and any other parameters needed to retrieve it.
pub struct Resource {
    base_url: String,
    query_parameters: BTreeMap<String, QueryValue>,
    template_values: HashMap<String, String>,
    pub headers: HashMap<String, String>,
    pub proxy: Option<Arc<dyn Proxy>>,
    pub retry_callback: Option<RetryCallback>,
    pub retry_attempts: u32,
    retry_count: u32,
    pub request: Request,
    pub credits: Option<Vec<Credit>>,
}

impl Clone for Resource {
    fn clone(&self) -> Self {
        Self {
            base_url: self.base_url.clone(),
            query_parameters: self.query_parameters.clone(),
            template_values: self.template_values.clone(),
            headers: self.headers.clone(),
            proxy: self.proxy.clone(),
            retry_callback: self.retry_callback.clone(),
            retry_attempts: self.retry_attempts,
            retry_count: 0,
            request: self.request.clone(),
            credits: self.credits.clone(),
        }
    }
}

impl fmt::Debug for Resource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Resource")
            .field("url", &self.url())
            .field("base_url", &self.base_url)
            .field("query_parameters", &self.query_parameters)
            .field("template_values", &self.template_values)
            .field("headers", &self.headers)
            .field("retry_attempts", &self.retry_attempts)
            .field("retry_count", &self.retry_count)
            .finish()
    }
}

impl fmt::Display for Resource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.url())
    }
}

impl PartialEq for Resource {
    fn eq(&self, other: &Self) -> bool {
        self.base_url == other.base_url
            && self.query_parameters == other.query_parameters
            && self.template_values == other.template_values
            && self.headers == other.headers
            && self.retry_attempts == other.retry_attempts
            && self.retry_count == other.retry_count
            && self.request == other.request
            && self.credits == other.credits
    }
}

impl Resource {
    /// Creates a new Resource instance.
    pub fn new(options: ResourceOptions) -> Self {
        let mut resource = Self {
            base_url: String::new(),
            query_parameters: BTreeMap::new(),
            template_values: options.template_values,
            headers: options.headers,
            proxy: options.proxy,
            retry_callback: options.retry_callback,
            retry_attempts: options.retry_attempts,
            retry_count: 0,
            request: options.request.unwrap_or_default(),
            credits: options.credits,
        };

        if options.parse_url {
            resource.parse_url(&options.url, false, false, None);
            if !options.query_parameters.is_empty() {
                resource.combine_query_parameters_internal(options.query_parameters, false);
            }
        } else {
            resource.base_url = options.url;
            resource.query_parameters = options.query_parameters;
        }

        resource
    }

    /// Creates a Resource from a URL string with default options (parse_url = true).
    pub fn from_url(url: impl Into<String>) -> Self {
        Self::new(ResourceOptions {
            url: url.into(),
            parse_url: true,
            ..Default::default()
        })
    }

    /// Helper function to create a resource from a string or clone if already a Resource.
    pub fn create_if_needed(resource: Option<&Resource>, url: Option<&str>) -> Option<Self> {
        if let Some(r) = resource {
            return Some(r.get_derived_resource(DerivedResourceOptions {
                request: Some(r.request.clone()),
                ..Default::default()
            }));
        }
        url.map(Self::from_url)
    }

    /// The base URL (without query parameters if parse_url was true).
    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    /// The query parameters for the resource.
    pub fn query_parameters(&self) -> &BTreeMap<String, QueryValue> {
        &self.query_parameters
    }

    /// Mutable query parameters.
    pub fn query_parameters_mut(&mut self) -> &mut BTreeMap<String, QueryValue> {
        &mut self.query_parameters
    }

    /// The template values for the resource.
    pub fn template_values(&self) -> &HashMap<String, String> {
        &self.template_values
    }

    /// Number of times the retryCallback has been called.
    pub fn retry_count(&self) -> u32 {
        self.retry_count
    }

    /// The file extension of the resource.
    pub fn extension(&self) -> String {
        get_extension_from_uri(&self.base_url)
    }

    /// True if the Resource refers to a data URI.
    pub fn is_data_uri(&self) -> bool {
        helper_is_data_uri(&self.base_url)
    }

    /// True if the Resource refers to a blob URI.
    pub fn is_blob_uri(&self) -> bool {
        helper_is_blob_uri(&self.base_url)
    }

    /// Appends a forward slash to the base URL if not present.
    pub fn append_forward_slash(&mut self) {
        if !self.base_url.ends_with('/') {
            self.base_url.push('/');
        }
    }

    /// Returns the URL with query string and processed by proxy if requested.
    pub fn get_url_component(&self, query: bool, proxy: bool) -> String {
        if self.is_data_uri() {
            return self.base_url.clone();
        }

        let mut url = self.base_url.clone();

        if query && !self.query_parameters.is_empty() {
            let query_str = self.stringify_query();
            if !query_str.is_empty() {
                if url.contains('?') {
                    url.push('&');
                    url.push_str(&query_str[1..]);
                } else {
                    url.push_str(&query_str);
                }
            }
        }

        // Restore placeholders which might have been encoded
        let mut replaced = url.replace("%7B", "{").replace("%7D", "}");

        // Replace template values
        if !self.template_values.is_empty() {
            for (key, val) in &self.template_values {
                let placeholder = format!("{{{}}}", key);
                if replaced.contains(&placeholder) {
                    replaced = replaced.replace(&placeholder, &percent_encode(val));
                }
            }
        }

        if proxy {
            if let Some(proxy_obj) = &self.proxy {
                return proxy_obj.get_url(&replaced);
            }
        }

        replaced
    }

    /// The full URL to the resource including query string, template replacements, and proxy.
    pub fn url(&self) -> String {
        self.get_url_component(true, true)
    }

    /// Stringifies the query parameters into a `?key1=val1&key2=val2` string.
    fn stringify_query(&self) -> String {
        if self.query_parameters.is_empty() {
            return String::new();
        }

        let mut parts = Vec::new();
        for (key, value) in &self.query_parameters {
            match value {
                QueryValue::Single(val) => {
                    if val.is_empty() {
                        parts.push(percent_encode(key));
                    } else {
                        parts.push(format!("{}={}", percent_encode(key), percent_encode(val)));
                    }
                }
                QueryValue::Array(vals) => {
                    for val in vals {
                        parts.push(format!("{}={}", percent_encode(key), percent_encode(val)));
                    }
                }
            }
        }

        if parts.is_empty() {
            String::new()
        } else {
            format!("?{}", parts.join("&"))
        }
    }

    /// Parses a URL string and merges or replaces parameters.
    pub fn parse_url(
        &mut self,
        url_str: &str,
        merge: bool,
        preserve_query: bool,
        base_url: Option<&str>,
    ) {
        let (raw_base, query_str) = split_url_and_query(url_str);

        let resolved_base = if let Some(base) = base_url {
            resolve_relative_url(base, raw_base)
        } else {
            raw_base.to_string()
        };

        let parsed_query = parse_query_string(query_str);

        if merge {
            self.combine_query_parameters_internal(parsed_query, preserve_query);
        } else {
            self.query_parameters = parsed_query;
        }

        self.base_url = resolved_base;
    }

    /// Derives a resource relative to the current instance.
    pub fn get_derived_resource(&self, options: DerivedResourceOptions) -> Resource {
        let mut resource = self.clone();
        resource.retry_count = 0;

        if let Some(ref derived_url) = options.url {
            resource.parse_url(
                derived_url,
                true,
                options.preserve_query_parameters,
                Some(&self.base_url),
            );
        }

        if let Some(params) = options.query_parameters {
            for (k, v) in params {
                resource.query_parameters.insert(k, v);
            }
        }

        if let Some(tmpl) = options.template_values {
            for (k, v) in tmpl {
                resource.template_values.insert(k, v);
            }
        }

        if let Some(headers) = options.headers {
            for (k, v) in headers {
                resource.headers.insert(k, v);
            }
        }

        if let Some(proxy) = options.proxy {
            resource.proxy = Some(proxy);
        }

        if let Some(req) = options.request {
            resource.request = req;
        }

        if let Some(cb) = options.retry_callback {
            resource.retry_callback = Some(cb);
        }

        if let Some(attempts) = options.retry_attempts {
            resource.retry_attempts = attempts;
        }

        resource
    }

    /// Combines specified query parameters.
    pub fn set_query_parameters(
        &mut self,
        params: impl IntoIterator<Item = (String, QueryValue)>,
        use_as_default: bool,
    ) {
        if use_as_default {
            for (k, v) in params {
                self.query_parameters.entry(k).or_insert(v);
            }
        } else {
            for (k, v) in params {
                self.query_parameters.insert(k, v);
            }
        }
    }

    /// Appends query parameters. If key already exists, values are concatenated.
    pub fn append_query_parameters(
        &mut self,
        params: impl IntoIterator<Item = (String, QueryValue)>,
    ) {
        let param_map: BTreeMap<String, QueryValue> = params.into_iter().collect();
        // In CesiumJS: combineQueryParameters(params, this._queryParameters, true)
        // Values from params come first, existing values are appended.
        let mut result = param_map;
        for (k, existing_val) in &self.query_parameters {
            if let Some(new_val) = result.get_mut(k) {
                let mut combined = new_val.to_vec();
                combined.extend(existing_val.to_vec());
                *new_val = QueryValue::Array(combined);
            } else {
                result.insert(k.clone(), existing_val.clone());
            }
        }
        self.query_parameters = result;
    }

    /// Combines specified template values.
    pub fn set_template_values(
        &mut self,
        values: impl IntoIterator<Item = (String, String)>,
        use_as_default: bool,
    ) {
        if use_as_default {
            for (k, v) in values {
                self.template_values.entry(k).or_insert(v);
            }
        } else {
            for (k, v) in values {
                self.template_values.insert(k, v);
            }
        }
    }

    fn combine_query_parameters_internal(
        &mut self,
        incoming: BTreeMap<String, QueryValue>,
        preserve_query: bool,
    ) {
        if !preserve_query {
            // Keys in incoming take precedence
            for (k, v) in incoming {
                self.query_parameters.insert(k, v);
            }
        } else {
            // Incoming comes first, then existing
            let mut result = incoming;
            for (k, existing_val) in &self.query_parameters {
                if let Some(new_val) = result.get_mut(k) {
                    let mut combined = new_val.to_vec();
                    combined.extend(existing_val.to_vec());
                    *new_val = QueryValue::Array(combined);
                } else {
                    result.insert(k.clone(), existing_val.clone());
                }
            }
            self.query_parameters = result;
        }
    }

    /// Called when a resource fails to load. Calls retry_callback until retry_attempts reached.
    pub fn retry_on_error(&mut self, error: Option<&RequestErrorEvent>) -> bool {
        if self.retry_count >= self.retry_attempts {
            return false;
        }
        if let Some(ref cb) = self.retry_callback {
            self.retry_count += 1;
            cb(self, error)
        } else {
            false
        }
    }

    /// Decodes a data URI into bytes and metadata.
    pub fn decode_data_uri(&self) -> Result<DataUriPayload, DeveloperError> {
        decode_data_uri_str(&self.base_url)
    }

    /// Fetches text from a data URI or in-memory resource.
    pub fn fetch_text(&self) -> Result<String, DeveloperError> {
        let payload = self.decode_data_uri()?;
        String::from_utf8(payload.data)
            .map_err(|e| DeveloperError::new(format!("Invalid UTF-8 in data URI: {e}")))
    }

    /// Fetches JSON from a data URI.
    pub fn fetch_json<T: serde::de::DeserializeOwned>(&self) -> Result<T, DeveloperError> {
        let text = self.fetch_text()?;
        serde_json::from_str(&text)
            .map_err(|e| DeveloperError::new(format!("Failed to parse JSON from data URI: {e}")))
    }

    /// Fetches raw bytes (ArrayBuffer equivalent) from a data URI.
    pub fn fetch_array_buffer(&self) -> Result<Vec<u8>, DeveloperError> {
        let payload = self.decode_data_uri()?;
        Ok(payload.data)
    }

    /// Fetches Blob from a data URI.
    pub fn fetch_blob(&self) -> Result<Blob, DeveloperError> {
        let payload = self.decode_data_uri()?;
        Ok(Blob {
            mime_type: payload.mime_type,
            data: payload.data,
        })
    }
}

/// Helper function to split a URL into base and query string parts.
fn split_url_and_query(url: &str) -> (&str, &str) {
    let clean = if let Some(idx) = url.find('#') {
        &url[..idx]
    } else {
        url
    };

    if let Some(idx) = clean.find('?') {
        (&clean[..idx], &clean[idx + 1..])
    } else {
        (clean, "")
    }
}

/// Parses a query string into a map of QueryValue.
fn parse_query_string(query_str: &str) -> BTreeMap<String, QueryValue> {
    let mut map: BTreeMap<String, Vec<String>> = BTreeMap::new();
    if query_str.is_empty() {
        return BTreeMap::new();
    }

    for pair in query_str.split('&') {
        if pair.is_empty() {
            continue;
        }
        let (key_enc, val_enc) = if let Some(eq_idx) = pair.find('=') {
            (&pair[..eq_idx], &pair[eq_idx + 1..])
        } else {
            (pair, "")
        };

        let key = percent_decode(key_enc);
        let val = percent_decode(val_enc);
        map.entry(key).or_default().push(val);
    }

    let mut result = BTreeMap::new();
    for (k, v) in map {
        if v.len() == 1 {
            result.insert(k, QueryValue::Single(v.into_iter().next().unwrap()));
        } else {
            result.insert(k, QueryValue::Array(v));
        }
    }
    result
}

/// Resolves a relative URL against a base URL.
fn resolve_relative_url(base: &str, relative: &str) -> String {
    if relative.is_empty() {
        return base.to_string();
    }

    if let Ok(parsed_rel) = Url::parse(relative) {
        if !parsed_rel.scheme().is_empty() {
            return relative.to_string();
        }
    }

    if let Ok(base_parsed) = Url::parse(base) {
        if let Ok(joined) = base_parsed.join(relative) {
            return joined.to_string();
        }
    }

    // Fallback manual resolution for non-standard or relative base
    if base.ends_with('/') {
        format!("{}{}", base, relative.trim_start_matches('/'))
    } else if let Some(last_slash) = base.rfind('/') {
        format!("{}/{}", &base[..last_slash], relative.trim_start_matches('/'))
    } else {
        relative.to_string()
    }
}

/// Decodes a data URI string.
pub fn decode_data_uri_str(uri: &str) -> Result<DataUriPayload, DeveloperError> {
    if !helper_is_data_uri(uri) {
        return Err(DeveloperError::new("Not a data URI"));
    }

    let content = &uri[5..]; // skip "data:"
    let comma_idx = content
        .find(',')
        .ok_or_else(|| DeveloperError::new("Malformed data URI: missing comma"))?;

    let meta = &content[..comma_idx];
    let data_str = &content[comma_idx + 1..];

    let is_base64 = meta.ends_with(";base64") || meta == ";base64";
    let mime_type = if is_base64 {
        meta.trim_end_matches(";base64").to_string()
    } else {
        meta.to_string()
    };
    let mime_type = if mime_type.is_empty() {
        "text/plain;charset=US-ASCII".to_string()
    } else {
        mime_type
    };

    let data = if is_base64 {
        // First percent-decode only %XX hex sequences so '+' in base64 is preserved
        let unescaped_bytes = decode_uri_escaped_bytes_keep_plus(data_str);
        // Remove whitespace
        let cleaned: Vec<u8> = unescaped_bytes
            .into_iter()
            .filter(|b| !b.is_ascii_whitespace())
            .collect();
        base64::engine::general_purpose::STANDARD
            .decode(&cleaned)
            .map_err(|e| DeveloperError::new(format!("Base64 decode error: {e}")))?
    } else {
        percent_decode(data_str).into_bytes()
    };

    Ok(DataUriPayload {
        mime_type,
        is_base64,
        data,
    })
}

/// Percent-decodes %XX sequences without converting '+' to space.
fn decode_uri_escaped_bytes_keep_plus(s: &str) -> Vec<u8> {
    let bytes = s.as_bytes();
    let mut decoded = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            if let Ok(val) =
                u8::from_str_radix(std::str::from_utf8(&bytes[i + 1..i + 3]).unwrap_or(""), 16)
            {
                decoded.push(val);
                i += 3;
                continue;
            }
        }
        decoded.push(bytes[i]);
        i += 1;
    }
    decoded
}
