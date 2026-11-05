use std::str::FromStr;

use crate::{
    Credit, DeveloperError, GeocodeType, PeliasGeocoderResult, PeliasGeocoderService,
};

pub use crate::ion::{DEFAULT_ION_ACCESS_TOKEN, DEFAULT_ION_SERVER};

/// Underlying geocoding services that can be used via Cesium ion.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum IonGeocodeProviderType {
    #[default]
    Default,
    Google,
    Bing,
}

impl IonGeocodeProviderType {
    pub fn to_query_parameter(self) -> Option<&'static str> {
        match self {
            Self::Google => Some("google"),
            Self::Bing => Some("bing"),
            Self::Default => None,
        }
    }
}

impl FromStr for IonGeocodeProviderType {
    type Err = DeveloperError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_uppercase().as_str() {
            "DEFAULT" => Ok(Self::Default),
            "GOOGLE" => Ok(Self::Google),
            "BING" => Ok(Self::Bing),
            _ => Err(DeveloperError::new(format!(
                "Invalid geocodeProviderType: \"{s}\""
            ))),
        }
    }
}

/// Options for `IonGeocoderService`.
#[derive(Debug, Clone, Default)]
pub struct IonGeocoderServiceOptions {
    pub access_token: Option<String>,
    pub server: Option<String>,
    pub geocode_provider_type: Option<IonGeocodeProviderType>,
}

/// Provides geocoding through Cesium ion.
#[derive(Debug, Clone)]
pub struct IonGeocoderService {
    access_token: String,
    server: String,
    pelias: PeliasGeocoderService,
}

impl IonGeocoderService {
    pub fn new(options: Option<IonGeocoderServiceOptions>) -> Result<Self, DeveloperError> {
        let opts = options.unwrap_or_default();
        let access_token = opts
            .access_token
            .unwrap_or_else(|| DEFAULT_ION_ACCESS_TOKEN.to_string());

        let mut server = opts
            .server
            .unwrap_or_else(|| DEFAULT_ION_SERVER.to_string());
        if !server.ends_with('/') {
            server.push('/');
        }

        let endpoint = format!("{server}v1/geocode");
        let mut pelias = PeliasGeocoderService::new(endpoint)?;

        if !access_token.is_empty() {
            pelias
                .query_parameters_mut()
                .insert("access_token".to_string(), access_token.clone());
        }

        let provider_type = opts.geocode_provider_type.unwrap_or(IonGeocodeProviderType::Default);

        let mut service = Self {
            access_token,
            server,
            pelias,
        };

        service.set_geocode_provider_type(provider_type)?;
        Ok(service)
    }

    pub fn access_token(&self) -> &str {
        &self.access_token
    }

    pub fn server(&self) -> &str {
        &self.server
    }

    pub fn pelias(&self) -> &PeliasGeocoderService {
        &self.pelias
    }

    pub fn credit(&self) -> Option<&Credit> {
        None
    }

    pub fn geocode_provider_type(&self) -> IonGeocodeProviderType {
        match self.pelias.query_parameters().get("geocoder").map(|s| s.as_str()) {
            Some("google") => IonGeocodeProviderType::Google,
            Some("bing") => IonGeocodeProviderType::Bing,
            _ => IonGeocodeProviderType::Default,
        }
    }

    pub fn set_geocode_provider_type(
        &mut self,
        provider_type: IonGeocodeProviderType,
    ) -> Result<(), DeveloperError> {
        match provider_type.to_query_parameter() {
            Some(param) => {
                self.pelias
                    .query_parameters_mut()
                    .insert("geocoder".to_string(), param.to_string());
            }
            None => {
                self.pelias
                    .query_parameters_mut()
                    .remove("geocoder");
            }
        }
        Ok(())
    }

    pub fn build_url(&self, query: &str, geocode_type: GeocodeType) -> String {
        self.pelias.build_url(query, geocode_type)
    }

    pub fn parse_response(
        &self,
        json_str: &str,
    ) -> Result<Vec<PeliasGeocoderResult>, DeveloperError> {
        self.pelias.parse_response(json_str)
    }
}
