use crate::uri_helpers::percent_encode;

/// A proxy trait for resource requests.
pub trait Proxy: Send + Sync {
    /// Get the final URL to use to request a given resource.
    fn get_url(&self, resource_url: &str) -> String;
}

/// A simple proxy that appends the desired resource as the sole query parameter
/// to the given proxy URL.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DefaultProxy {
    pub proxy: String,
}

impl DefaultProxy {
    pub fn new(proxy: impl Into<String>) -> Self {
        Self {
            proxy: proxy.into(),
        }
    }
}

impl Proxy for DefaultProxy {
    fn get_url(&self, resource_url: &str) -> String {
        let prefix = if self.proxy.contains('?') { "" } else { "?" };
        format!("{}{}{}", self.proxy, prefix, percent_encode(resource_url))
    }
}
