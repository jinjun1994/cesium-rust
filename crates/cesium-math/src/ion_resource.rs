use std::collections::{BTreeMap, HashMap};
use std::fmt;
use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::ion::{Ion, DEFAULT_ION_ACCESS_TOKEN, DEFAULT_ION_SERVER};
use crate::resource::{DerivedResourceOptions, QueryValue, Resource, ResourceOptions, RetryCallback};
use crate::{Credit, DeveloperError};

/// Attribution info in an ion endpoint response.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct IonAttribution {
    pub html: Option<String>,
    pub text: Option<String>,
    #[serde(default)]
    pub collapsible: bool,
}

/// Extra options in an external ion endpoint.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct IonEndpointOptions {
    pub url: Option<String>,
}

/// Cesium ion asset endpoint service response.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct IonEndpoint {
    #[serde(rename = "type", default)]
    pub endpoint_type: String,
    pub url: Option<String>,
    #[serde(rename = "accessToken")]
    pub access_token: Option<String>,
    #[serde(default)]
    pub attributions: Vec<IonAttribution>,
    #[serde(rename = "externalType")]
    pub external_type: Option<String>,
    pub options: Option<IonEndpointOptions>,
}

/// Options for creating an IonResource.
#[derive(Debug, Clone, Default)]
pub struct IonResourceOptions {
    pub access_token: Option<String>,
    pub server: Option<String>,
    pub query_parameters: Option<BTreeMap<String, QueryValue>>,
}

pub type IonRefreshCallback = Arc<dyn Fn(&IonResource, &IonEndpoint) + Send + Sync>;

/// A Resource instance that encapsulates Cesium ion asset access.
#[derive(Clone)]
pub struct IonResource {
    pub resource: Resource,
    pub ion_endpoint: IonEndpoint,
    pub ion_endpoint_resource: Resource,
    pub is_external: bool,
    pub ion_endpoint_domain: Option<String>,
    pub refresh_callback: Option<IonRefreshCallback>,
}

impl fmt::Debug for IonResource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("IonResource")
            .field("url", &self.resource.url())
            .field("ion_endpoint", &self.ion_endpoint)
            .field("is_external", &self.is_external)
            .finish()
    }
}

impl std::ops::Deref for IonResource {
    type Target = Resource;
    fn deref(&self) -> &Self::Target {
        &self.resource
    }
}

impl std::ops::DerefMut for IonResource {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.resource
    }
}

impl IonResource {
    /// Constructs an IonResource from an endpoint and endpoint Resource.
    pub fn new(
        endpoint: IonEndpoint,
        endpoint_resource: Resource,
    ) -> Result<Self, DeveloperError> {
        let external_type = endpoint.external_type.clone();
        let is_external = external_type.is_some();

        let (url, retry_attempts, retry_callback) = if !is_external {
            let u = endpoint
                .url
                .clone()
                .unwrap_or_default();
            let retry_cb: RetryCallback = Arc::new(|res, err| {
                if let Some(e) = err {
                    if e.status_code == Some(401) {
                        return true;
                    }
                }
                let _ = res;
                false
            });
            (u, 1, Some(retry_cb))
        } else {
            let ext = external_type.as_deref().unwrap_or("");
            if ext == "3DTILES" || ext == "STK_TERRAIN_SERVER" {
                let u = endpoint
                    .options
                    .as_ref()
                    .and_then(|o| o.url.clone())
                    .unwrap_or_default();
                (u, 0, None)
            } else {
                return Err(DeveloperError::new(
                    "Ion.createResource does not support external imagery assets; use IonImageryProvider instead.",
                ));
            }
        };

        let mut headers = HashMap::new();
        headers.insert("X-Cesium-Client".to_string(), "CesiumJS".to_string());
        if !is_external {
            if let Some(ref token) = endpoint.access_token {
                headers.insert("Authorization".to_string(), format!("Bearer {token}"));
            }
        }

        let domain = if !is_external {
            url::Url::parse(&url)
                .ok()
                .and_then(|u| u.host_str().map(|s| s.to_string()))
        } else {
            None
        };

        let resource = Resource::new(ResourceOptions {
            url,
            headers,
            retry_attempts,
            retry_callback,
            parse_url: true,
            ..Default::default()
        });

        Ok(Self {
            resource,
            ion_endpoint: endpoint,
            ion_endpoint_resource: endpoint_resource,
            is_external,
            ion_endpoint_domain: domain,
            refresh_callback: None,
        })
    }

    /// Creates an endpoint Resource for the given assetId and options.
    pub fn create_endpoint_resource(
        asset_id: i64,
        options: Option<&IonResourceOptions>,
    ) -> Resource {
        let server_url = options
            .and_then(|o| o.server.as_deref())
            .unwrap_or(DEFAULT_ION_SERVER);

        let access_token = options
            .and_then(|o| o.access_token.as_deref())
            .unwrap_or(DEFAULT_ION_ACCESS_TOKEN);

        let mut base_resource = Resource::from_url(server_url);
        base_resource.append_forward_slash();

        let mut query_params = BTreeMap::new();
        query_params.insert("access_token".to_string(), QueryValue::Single(access_token.to_string()));

        if let Some(opts) = options {
            if let Some(ref extra_params) = opts.query_parameters {
                for (k, v) in extra_params {
                    query_params.insert(k.clone(), v.clone());
                }
            }
        }

        let mut headers = HashMap::new();
        headers.insert("X-Cesium-Client".to_string(), "CesiumJS".to_string());

        base_resource.get_derived_resource(DerivedResourceOptions {
            url: Some(format!("v1/assets/{asset_id}/endpoint")),
            query_parameters: Some(query_params),
            headers: Some(headers),
            ..Default::default()
        })
    }

    /// Creates an IonResource from an asset ID and endpoint representation.
    pub fn from_endpoint(
        asset_id: i64,
        endpoint: IonEndpoint,
        options: Option<&IonResourceOptions>,
    ) -> Result<Self, DeveloperError> {
        let endpoint_resource = Self::create_endpoint_resource(asset_id, options);
        Self::new(endpoint, endpoint_resource)
    }

    /// Gets credits required for attribution of the asset.
    pub fn credits(&self) -> Vec<Credit> {
        let mut credits = Vec::new();
        for attr in &self.ion_endpoint.attributions {
            if let Some(ref html) = attr.html {
                credits.push(Credit::new(html, Some(true)));
            } else if let Some(ref text) = attr.text {
                credits.push(Credit::new(text, Some(true)));
            }
        }

        let access_token = self
            .ion_endpoint_resource
            .query_parameters()
            .get("access_token")
            .and_then(|v| match v {
                QueryValue::Single(s) => Some(s.as_str()),
                QueryValue::Array(arr) => arr.first().map(|s| s.as_str()),
            });

        if let Some(default_credit) = Ion::get_default_token_credit(access_token) {
            credits.push(default_credit);
        }

        credits
    }

    /// Clones this IonResource.
    pub fn clone_resource(&self) -> Self {
        Self {
            resource: self.resource.clone(),
            ion_endpoint: self.ion_endpoint.clone(),
            ion_endpoint_resource: self.ion_endpoint_resource.clone(),
            is_external: self.is_external,
            ion_endpoint_domain: self.ion_endpoint_domain.clone(),
            refresh_callback: self.refresh_callback.clone(),
        }
    }
}
