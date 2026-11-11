//! WebMapServiceImageryProvider matching CesiumJS `Scene/WebMapServiceImageryProvider.js` (Apache-2.0).

use std::collections::BTreeMap;

use cesium_math::{
    Credit, DeveloperError, Ellipsoid, GeoRectangle, GeographicTilingScheme,
};

use crate::imagery_layer::ImageryProvider;
use crate::quadtree_tile::AnyTilingScheme;

const INCLUDES_REVERSE_AXIS: &[u32] = &[3034, 3035, 3042, 3043, 3044];
const EXCLUDES_REVERSE_AXIS: &[u32] = &[4471, 4559];

/// Construction options for [`WebMapServiceImageryProvider`].
#[derive(Debug, Clone, Default)]
pub struct WebMapServiceImageryProviderOptions {
    pub url: Option<String>,
    pub layers: Option<String>,
    pub parameters: Option<BTreeMap<String, String>>,
    pub get_feature_info_parameters: Option<BTreeMap<String, String>>,
    pub get_feature_info_url: Option<String>,
    pub rectangle: Option<GeoRectangle>,
    pub tiling_scheme: Option<AnyTilingScheme>,
    pub ellipsoid: Option<Ellipsoid>,
    pub tile_width: Option<u32>,
    pub tile_height: Option<u32>,
    pub minimum_level: Option<u32>,
    pub maximum_level: Option<u32>,
    pub crs: Option<String>,
    pub srs: Option<String>,
    pub credit: Option<Credit>,
    pub subdomains: Option<Vec<String>>,
    pub has_alpha_channel: Option<bool>,
}

/// Provides tiled imagery hosted by a Web Map Service (WMS) server.
#[derive(Clone, Debug)]
pub struct WebMapServiceImageryProvider {
    pub url: String,
    pub layers: String,
    pub tile_width: u32,
    pub tile_height: u32,
    pub minimum_level: u32,
    pub maximum_level: Option<u32>,
    pub rectangle: GeoRectangle,
    pub tiling_scheme: AnyTilingScheme,
    pub credit: Option<Credit>,
    pub has_alpha_channel: bool,
    pub crs: Option<String>,
    pub srs: Option<String>,
    pub query_parameters: BTreeMap<String, String>,
    pub reverse_axis_order: bool,
}

impl WebMapServiceImageryProvider {
    /// Creates a new `WebMapServiceImageryProvider`.
    pub fn new(options: WebMapServiceImageryProviderOptions) -> Result<Self, DeveloperError> {
        let url = options
            .url
            .ok_or_else(|| DeveloperError::new("options.url is required."))?;
        if url.trim().is_empty() {
            return Err(DeveloperError::new("options.url is required."));
        }

        let layers = options
            .layers
            .ok_or_else(|| DeveloperError::new("options.layers is required."))?;
        if layers.trim().is_empty() {
            return Err(DeveloperError::new("options.layers is required."));
        }

        let ellipsoid = options.ellipsoid.unwrap_or(Ellipsoid::WGS84);
        let tiling_scheme = options.tiling_scheme.unwrap_or_else(|| {
            let scheme = GeographicTilingScheme::new(ellipsoid, GeoRectangle::MAXIMUM, 2, 1)
                .unwrap_or_default();
            AnyTilingScheme::Geographic(scheme)
        });

        let tile_width = options.tile_width.unwrap_or(256);
        let tile_height = options.tile_height.unwrap_or(256);
        let minimum_level = options.minimum_level.unwrap_or(0);
        let maximum_level = options.maximum_level;
        let rectangle = options.rectangle.unwrap_or_else(|| tiling_scheme.rectangle());
        let has_alpha_channel = options.has_alpha_channel.unwrap_or(true);

        let mut query_params = BTreeMap::new();
        query_params.insert("service".to_string(), "WMS".to_string());
        query_params.insert("version".to_string(), "1.1.1".to_string());
        query_params.insert("request".to_string(), "GetMap".to_string());
        query_params.insert("styles".to_string(), "".to_string());
        query_params.insert("format".to_string(), "image/jpeg".to_string());

        if let Some(user_params) = options.parameters {
            for (k, v) in user_params {
                query_params.insert(k.to_lowercase(), v);
            }
        }

        query_params.insert("layers".to_string(), layers.clone());
        query_params.insert("width".to_string(), tile_width.to_string());
        query_params.insert("height".to_string(), tile_height.to_string());

        let version_str = query_params
            .get("version")
            .cloned()
            .unwrap_or_else(|| "1.1.1".to_string());
        let is_version_1_3_or_higher = {
            let mut parts = version_str.split('.');
            let major: u32 = parts.next().and_then(|p| p.parse().ok()).unwrap_or(1);
            let minor: u32 = parts.next().and_then(|p| p.parse().ok()).unwrap_or(1);
            major > 1 || (major == 1 && minor >= 3)
        };

        let is_web_mercator = matches!(tiling_scheme, AnyTilingScheme::WebMercator(_));

        let mut reverse_axis = false;
        let mut final_crs = None;
        let mut final_srs = None;

        if is_version_1_3_or_higher {
            let crs = options.crs.unwrap_or_else(|| {
                if is_web_mercator {
                    "EPSG:3857".to_string()
                } else {
                    "CRS:84".to_string()
                }
            });

            let parts: Vec<&str> = crs.split(':').collect();
            if parts.len() == 2 && parts[0].eq_ignore_ascii_case("EPSG") {
                if let Ok(code) = parts[1].parse::<u32>() {
                    if ((4000..5000).contains(&code) && !EXCLUDES_REVERSE_AXIS.contains(&code))
                        || INCLUDES_REVERSE_AXIS.contains(&code)
                    {
                        reverse_axis = true;
                    }
                }
            }

            query_params.insert("crs".to_string(), crs.clone());
            final_crs = Some(crs);
        } else {
            let srs = options.srs.unwrap_or_else(|| {
                if is_web_mercator {
                    "EPSG:3857".to_string()
                } else {
                    "EPSG:4326".to_string()
                }
            });
            query_params.insert("srs".to_string(), srs.clone());
            final_srs = Some(srs);
        }

        Ok(Self {
            url,
            layers,
            tile_width,
            tile_height,
            minimum_level,
            maximum_level,
            rectangle,
            tiling_scheme,
            credit: options.credit,
            has_alpha_channel,
            crs: final_crs,
            srs: final_srs,
            query_parameters: query_params,
            reverse_axis_order: reverse_axis,
        })
    }

    /// Formats the bounding box parameter for a given tile.
    pub fn compute_tile_bbox(&self, x: u32, y: u32, level: u32) -> String {
        let tile_rect = self
            .tiling_scheme
            .tile_xy_to_rectangle(x, y, level)
            .unwrap_or(GeoRectangle::MAXIMUM);

        let fmt_num = |val: f64| -> String {
            if (val - val.round()).abs() < 1e-9 {
                format!("{}", val.round() as i64)
            } else {
                format!("{val}")
            }
        };

        match &self.tiling_scheme {
            AnyTilingScheme::Geographic(_) => {
                let west = fmt_num(tile_rect.west.to_degrees());
                let south = fmt_num(tile_rect.south.to_degrees());
                let east = fmt_num(tile_rect.east.to_degrees());
                let north = fmt_num(tile_rect.north.to_degrees());

                if self.reverse_axis_order {
                    format!("{south},{west},{north},{east}")
                } else {
                    format!("{west},{south},{east},{north}")
                }
            }
            AnyTilingScheme::WebMercator(wm) => {
                let proj = wm.projection();
                let (west_proj, south_proj, east_proj, north_proj) = {
                    let sw = proj
                        .project(cesium_math::Cartographic::new(
                            tile_rect.west,
                            tile_rect.south,
                            0.0,
                        ))
                        .unwrap_or(glam::DVec3::ZERO);
                    let ne = proj
                        .project(cesium_math::Cartographic::new(
                            tile_rect.east,
                            tile_rect.north,
                            0.0,
                        ))
                        .unwrap_or(glam::DVec3::ZERO);
                    (fmt_num(sw.x), fmt_num(sw.y), fmt_num(ne.x), fmt_num(ne.y))
                };

                if self.reverse_axis_order {
                    format!("{south_proj},{west_proj},{north_proj},{east_proj}")
                } else {
                    format!("{west_proj},{south_proj},{east_proj},{north_proj}")
                }
            }
        }
    }

    /// Builds the tile request URL for the specified tile coordinate.
    pub fn build_image_url(&self, x: u32, y: u32, level: u32) -> String {
        let bbox = self.compute_tile_bbox(x, y, level);
        let mut separator = if self.url.contains('?') { "&" } else { "?" };
        let mut res = self.url.clone();

        for (k, v) in &self.query_parameters {
            res.push_str(separator);
            separator = "&";
            res.push_str(k);
            res.push('=');
            res.push_str(v);
        }

        res.push_str(separator);
        res.push_str("bbox=");
        res.push_str(&bbox);

        res
    }

    /// Alias for `build_image_url` to match the ImageryProvider contract.
    pub fn request_image_url(&self, x: u32, y: u32, level: u32) -> String {
        self.build_image_url(x, y, level)
    }
}

impl ImageryProvider for WebMapServiceImageryProvider {
    fn rectangle(&self) -> GeoRectangle {
        self.rectangle
    }

    fn tile_width(&self) -> u32 {
        self.tile_width
    }

    fn tile_height(&self) -> u32 {
        self.tile_height
    }

    fn minimum_level(&self) -> u32 {
        self.minimum_level
    }

    fn maximum_level(&self) -> Option<u32> {
        self.maximum_level
    }

    fn has_alpha_channel(&self) -> bool {
        self.has_alpha_channel
    }

    fn ellipsoid(&self) -> Ellipsoid {
        self.tiling_scheme.ellipsoid()
    }
}
