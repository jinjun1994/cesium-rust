//! Creates an IonImageryProvider instance for ion's default global base imagery layer.
//! Adapted from CesiumJS `Scene/createWorldImageryAsync.js` (Apache-2.0).

use cesium_math::{DeveloperError, IonEndpoint, RuntimeError};

use crate::ion_imagery_provider::{
    IonImageryProvider, IonImageryProviderOptions,
};

/// The types of imagery provided by [`create_world_imagery_async`].
/// Values map directly to Cesium ion asset IDs.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
#[repr(i64)]
pub enum IonWorldImageryStyle {
    /// Aerial imagery (asset id 2).
    #[default]
    Aerial = 2,
    /// Aerial imagery with a road overlay (asset id 3).
    AerialWithLabels = 3,
    /// Roads without additional imagery (asset id 4).
    Road = 4,
}

impl IonWorldImageryStyle {
    /// Returns the corresponding Cesium ion asset ID.
    pub const fn asset_id(&self) -> i64 {
        *self as i64
    }

    /// Tries to convert an integer asset ID into an `IonWorldImageryStyle`.
    pub fn from_asset_id(id: i64) -> Result<Self, DeveloperError> {
        match id {
            2 => Ok(Self::Aerial),
            3 => Ok(Self::AerialWithLabels),
            4 => Ok(Self::Road),
            _ => Err(DeveloperError::new(format!(
                "Unsupported IonWorldImageryStyle asset id: {id}"
            ))),
        }
    }
}

/// Options for configuring world imagery.
#[derive(Clone, Debug, Default)]
pub struct CreateWorldImageryOptions {
    /// The style of base imagery. Defaults to `IonWorldImageryStyle::Aerial`.
    pub style: Option<IonWorldImageryStyle>,
    /// Options passed to the underlying `IonImageryProvider`.
    pub provider_options: Option<IonImageryProviderOptions>,
}

/// Creates an [`IonImageryProvider`] instance for ion's default global base imagery layer.
///
/// Uses the default Cesium ion Bing Maps imagery endpoint.
pub fn create_world_imagery_async(
    options: Option<CreateWorldImageryOptions>,
) -> Result<IonImageryProvider, RuntimeError> {
    create_world_imagery_async_with_fetcher(options, |asset_id, opts| {
        let access_token = opts.and_then(|o| o.access_token.clone());
        Ok(IonEndpoint {
            endpoint_type: "IMAGERY".to_string(),
            url: Some(format!("https://api.cesium.com/v1/assets/{asset_id}/endpoint")),
            access_token,
            external_type: Some("BING".to_string()),
            attributions: vec![],
            options: None,
        })
    })
}

/// Creates an [`IonImageryProvider`] instance using a custom endpoint fetcher.
pub fn create_world_imagery_async_with_fetcher<F>(
    options: Option<CreateWorldImageryOptions>,
    fetcher: F,
) -> Result<IonImageryProvider, RuntimeError>
where
    F: FnOnce(i64, Option<&IonImageryProviderOptions>) -> Result<IonEndpoint, RuntimeError>,
{
    let style = options
        .as_ref()
        .and_then(|o| o.style)
        .unwrap_or(IonWorldImageryStyle::Aerial);
    let provider_opts = options.as_ref().and_then(|o| o.provider_options.as_ref());
    IonImageryProvider::from_asset_id_with_fetcher(style.asset_id(), provider_opts, fetcher)
}
