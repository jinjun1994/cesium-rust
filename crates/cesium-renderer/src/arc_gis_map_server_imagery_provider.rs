//! ArcGisMapServerImageryProvider matching CesiumJS `Scene/ArcGisMapServerImageryProvider.js` (Apache-2.0).

use cesium_math::{
    Cartographic, DeveloperError, Ellipsoid, GeoRectangle, GeographicTilingScheme, RuntimeError,
    WebMercatorProjection, WebMercatorTilingScheme,
};
use serde_json::Value;

use crate::imagery_layer::ImageryProvider;
use crate::quadtree_tile::AnyTilingScheme;

/// Predefined ArcGIS Online basemap styles.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ArcGisBaseMapType {
    Satellite,
    Hillshade,
    Oceans,
}

impl ArcGisBaseMapType {
    pub fn service_url(&self) -> &'static str {
        match self {
            ArcGisBaseMapType::Satellite => {
                "https://services.arcgisonline.com/ArcGIS/rest/services/World_Imagery/MapServer"
            }
            ArcGisBaseMapType::Hillshade => {
                "https://services.arcgisonline.com/ArcGIS/rest/services/Elevation/World_Hillshade/MapServer"
            }
            ArcGisBaseMapType::Oceans => {
                "https://services.arcgisonline.com/ArcGIS/rest/services/Ocean/World_Ocean_Base/MapServer"
            }
        }
    }
}

/// Initialization options for [`ArcGisMapServerImageryProvider`].
#[derive(Clone, Debug)]
pub struct ArcGisMapServerImageryProviderOptions {
    pub token: Option<String>,
    pub use_pre_cached_tiles_if_available: bool,
    pub layers: Option<String>,
    pub enable_pick_features: bool,
    pub rectangle: Option<GeoRectangle>,
    pub tiling_scheme: Option<AnyTilingScheme>,
    pub ellipsoid: Option<Ellipsoid>,
    pub tile_width: Option<u32>,
    pub tile_height: Option<u32>,
    pub maximum_level: Option<u32>,
    pub credit: Option<String>,
}

impl Default for ArcGisMapServerImageryProviderOptions {
    fn default() -> Self {
        Self {
            token: None,
            use_pre_cached_tiles_if_available: true,
            layers: None,
            enable_pick_features: true,
            rectangle: None,
            tiling_scheme: None,
            ellipsoid: None,
            tile_width: None,
            tile_height: None,
            maximum_level: None,
            credit: None,
        }
    }
}

/// An [`ImageryProvider`] for imagery hosted by an ArcGIS MapServer.
#[derive(Clone, Debug)]
pub struct ArcGisMapServerImageryProvider {
    url: String,
    token: Option<String>,
    layers: Option<String>,
    using_precached_tiles: bool,
    enable_pick_features: bool,
    tile_width: u32,
    tile_height: u32,
    minimum_level: u32,
    maximum_level: Option<u32>,
    tiling_scheme: AnyTilingScheme,
    rectangle: GeoRectangle,
    credit: Option<String>,
    ellipsoid: Ellipsoid,
}

impl ArcGisMapServerImageryProvider {
    /// Creates an [`ArcGisMapServerImageryProvider`] from a MapServer URL with default WebMercator tiling.
    pub fn from_url(
        url: &str,
        options: Option<ArcGisMapServerImageryProviderOptions>,
    ) -> Result<Self, DeveloperError> {
        if url.trim().is_empty() {
            return Err(DeveloperError::new("url is required."));
        }
        let opts = options.unwrap_or_default();
        let clean_url = url.trim_end_matches('/').to_string();
        let ellipsoid = opts.ellipsoid.unwrap_or(Ellipsoid::WGS84);
        let tiling_scheme = opts.tiling_scheme.unwrap_or_else(|| {
            let wm = WebMercatorTilingScheme::new(ellipsoid, None, 1, 1)
                .unwrap_or_else(|_| WebMercatorTilingScheme::default());
            AnyTilingScheme::WebMercator(wm)
        });
        let mut rectangle = opts.rectangle.unwrap_or_else(|| tiling_scheme.rectangle());
        clamp_rectangle(&mut rectangle, &tiling_scheme);

        Ok(Self {
            url: clean_url,
            token: opts.token,
            layers: opts.layers,
            using_precached_tiles: opts.use_pre_cached_tiles_if_available,
            enable_pick_features: opts.enable_pick_features,
            tile_width: opts.tile_width.unwrap_or(256),
            tile_height: opts.tile_height.unwrap_or(256),
            minimum_level: 0,
            maximum_level: opts.maximum_level,
            tiling_scheme,
            rectangle,
            credit: opts.credit,
            ellipsoid,
        })
    }

    /// Creates an [`ArcGisMapServerImageryProvider`] from a URL and parsed MapServer JSON metadata.
    pub fn from_url_with_metadata(
        url: &str,
        metadata: &Value,
        options: Option<ArcGisMapServerImageryProviderOptions>,
    ) -> Result<Self, RuntimeError> {
        if url.trim().is_empty() {
            return Err(RuntimeError::new("url is required."));
        }
        let opts = options.unwrap_or_default();
        let clean_url = url.trim_end_matches('/').to_string();
        let ellipsoid = opts.ellipsoid.unwrap_or(Ellipsoid::WGS84);

        let tile_info = metadata.get("tileInfo").filter(|v| !v.is_null());
        let use_tiles = opts.use_pre_cached_tiles_if_available && tile_info.is_some();

        let (tiling_scheme, tile_width, tile_height, meta_max_level) = if use_tiles {
            let ti = tile_info.unwrap();
            let w = ti.get("cols").and_then(|v| v.as_u64()).unwrap_or(256) as u32;
            let h = ti.get("rows").and_then(|v| v.as_u64()).unwrap_or(256) as u32;
            let wkid = ti
                .get("spatialReference")
                .and_then(|sr| sr.get("wkid"))
                .and_then(|v| v.as_u64())
                .unwrap_or(102100);

            let scheme = match wkid {
                102100 | 3857 => {
                    let wm = WebMercatorTilingScheme::new(ellipsoid, None, 1, 1)
                        .map_err(|e| RuntimeError::new(e.to_string()))?;
                    AnyTilingScheme::WebMercator(wm)
                }
                4326 => {
                    let geo = GeographicTilingScheme::new(ellipsoid, GeoRectangle::MAXIMUM, 2, 1)
                        .map_err(|e| RuntimeError::new(e.to_string()))?;
                    AnyTilingScheme::Geographic(geo)
                }
                other => {
                    return Err(RuntimeError::new(format!(
                        "Tile spatial reference WKID {} is not supported.",
                        other
                    )));
                }
            };

            let max_lvl = ti
                .get("lods")
                .and_then(|v| v.as_array())
                .and_then(|arr| arr.iter().filter_map(|l| l.get("level")?.as_u64()).max())
                .map(|m| m as u32);

            (scheme, w, h, max_lvl)
        } else {
            let scheme = opts.tiling_scheme.unwrap_or_else(|| {
                let geo = GeographicTilingScheme::new(ellipsoid, GeoRectangle::MAXIMUM, 2, 1)
                    .unwrap_or_else(|_| GeographicTilingScheme::default());
                AnyTilingScheme::Geographic(geo)
            });
            (
                scheme,
                opts.tile_width.unwrap_or(256),
                opts.tile_height.unwrap_or(256),
                None,
            )
        };

        let mut rectangle = opts.rectangle.unwrap_or_else(|| tiling_scheme.rectangle());
        if let Some(fe) = metadata.get("fullExtent").filter(|v| v.is_object()) {
            if let (Some(xmin), Some(ymin), Some(xmax), Some(ymax)) = (
                fe.get("xmin").and_then(|v| v.as_f64()),
                fe.get("ymin").and_then(|v| v.as_f64()),
                fe.get("xmax").and_then(|v| v.as_f64()),
                fe.get("ymax").and_then(|v| v.as_f64()),
            ) {
                let fe_wkid = fe
                    .get("spatialReference")
                    .and_then(|sr| sr.get("wkid"))
                    .and_then(|v| v.as_u64())
                    .unwrap_or(4326);
                match fe_wkid {
                    4326 => {
                        rectangle = GeoRectangle::from_degrees(xmin, ymin, xmax, ymax);
                    }
                    102100 | 3857 => {
                        let proj = WebMercatorProjection::new(ellipsoid)
                            .map_err(|e| RuntimeError::new(e.to_string()))?;
                        if let (Some(sw), Some(ne)) = (
                            proj.unproject(cesium_math::Cartesian3::new(xmin, ymin, 0.0).into()),
                            proj.unproject(cesium_math::Cartesian3::new(xmax, ymax, 0.0).into()),
                        ) {
                            rectangle = GeoRectangle::new(
                                sw.longitude,
                                sw.latitude,
                                ne.longitude,
                                ne.latitude,
                            );
                        }
                    }
                    other => {
                        return Err(RuntimeError::new(format!(
                            "fullExtent spatialReference WKID {} is not supported.",
                            other
                        )));
                    }
                }
            }
        }
        clamp_rectangle(&mut rectangle, &tiling_scheme);

        let credit = opts.credit.or_else(|| {
            metadata
                .get("copyrightText")
                .and_then(|v| v.as_str())
                .filter(|s| !s.is_empty())
                .map(|s| s.to_string())
        });

        Ok(Self {
            url: clean_url,
            token: opts.token,
            layers: opts.layers,
            using_precached_tiles: use_tiles,
            enable_pick_features: opts.enable_pick_features,
            tile_width,
            tile_height,
            minimum_level: 0,
            maximum_level: opts.maximum_level.or(meta_max_level),
            tiling_scheme,
            rectangle,
            credit,
            ellipsoid,
        })
    }

    /// Creates an [`ArcGisMapServerImageryProvider`] for a standard ArcGIS basemap style.
    pub fn from_basemap_type(
        style: ArcGisBaseMapType,
        token: Option<&str>,
        options: Option<ArcGisMapServerImageryProviderOptions>,
    ) -> Result<Self, DeveloperError> {
        let mut opts = options.unwrap_or_default();
        if let Some(tok) = token {
            opts.token = Some(tok.to_string());
        } else if opts.credit.is_none() {
            opts.credit = Some(
                "Default ArcGIS Basemap (Esri, Maxar, Earthstar Geographics)".to_string(),
            );
        }
        Self::from_url(style.service_url(), Some(opts))
    }

    pub fn url(&self) -> &str {
        &self.url
    }

    pub fn token(&self) -> Option<&str> {
        self.token.as_deref()
    }

    pub fn using_precached_tiles(&self) -> bool {
        self.using_precached_tiles
    }

    pub fn enable_pick_features(&self) -> bool {
        self.enable_pick_features
    }

    pub fn set_enable_pick_features(&mut self, enable: bool) {
        self.enable_pick_features = enable;
    }

    pub fn tiling_scheme(&self) -> &AnyTilingScheme {
        &self.tiling_scheme
    }

    pub fn credit(&self) -> Option<&str> {
        self.credit.as_deref()
    }

    /// Builds the image request URL for tile `(x, y, level)`.
    pub fn build_tile_url(&self, x: u32, y: u32, level: u32) -> String {
        let mut url = if self.using_precached_tiles {
            format!("{}/tile/{}/{}/{}", self.url, level, y, x)
        } else {
            let rect = self
                .tiling_scheme
                .tile_xy_to_rectangle(x, y, level)
                .unwrap_or_else(|| self.tiling_scheme.rectangle());
            let (west, south, east, north, sr) = match &self.tiling_scheme {
                AnyTilingScheme::WebMercator(_) => {
                    let proj = WebMercatorProjection::new(self.ellipsoid)
                        .unwrap_or_else(|_| WebMercatorProjection::default());
                    let sw = proj
                        .project(Cartographic::new(rect.west, rect.south, 0.0))
                        .unwrap_or_default();
                    let ne = proj
                        .project(Cartographic::new(rect.east, rect.north, 0.0))
                        .unwrap_or_default();
                    (sw.x, sw.y, ne.x, ne.y, 102100)
                }
                AnyTilingScheme::Geographic(_) => (
                    rect.west.to_degrees(),
                    rect.south.to_degrees(),
                    rect.east.to_degrees(),
                    rect.north.to_degrees(),
                    4326,
                ),
            };
            let mut export_url = format!(
                "{}/export?bbox={}%2C{}%2C{}%2C{}&bboxSR={}&size={}%2C{}&imageSR={}&format=png32&transparent=true&f=image",
                self.url,
                west,
                south,
                east,
                north,
                sr,
                self.tile_width,
                self.tile_height,
                sr
            );
            if let Some(ref layers) = self.layers {
                export_url.push_str(&format!("&layers=show%3A{}", layers));
            }
            export_url
        };

        if let Some(ref token) = self.token {
            let sep = if url.contains('?') { '&' } else { '?' };
            url.push_str(&format!("{}token={}", sep, token));
        }
        url
    }

    /// Builds the ArcGIS `/identify` URL for picking features at `(longitude, latitude)` radians.
    pub fn build_pick_features_url(
        &self,
        x: u32,
        y: u32,
        level: u32,
        longitude: f64,
        latitude: f64,
    ) -> Option<String> {
        if !self.enable_pick_features {
            return None;
        }
        let rect = self
            .tiling_scheme
            .tile_xy_to_rectangle(x, y, level)
            .unwrap_or_else(|| self.tiling_scheme.rectangle());
        let (geom_x, geom_y, w, s, e, n, sr) = match &self.tiling_scheme {
            AnyTilingScheme::WebMercator(_) => {
                let proj = WebMercatorProjection::new(self.ellipsoid)
                    .unwrap_or_else(|_| WebMercatorProjection::default());
                let p = proj
                    .project(Cartographic::new(longitude, latitude, 0.0))
                    .unwrap_or_default();
                let sw = proj
                    .project(Cartographic::new(rect.west, rect.south, 0.0))
                    .unwrap_or_default();
                let ne = proj
                    .project(Cartographic::new(rect.east, rect.north, 0.0))
                    .unwrap_or_default();
                (p.x, p.y, sw.x, sw.y, ne.x, ne.y, 102100)
            }
            AnyTilingScheme::Geographic(_) => (
                longitude.to_degrees(),
                latitude.to_degrees(),
                rect.west.to_degrees(),
                rect.south.to_degrees(),
                rect.east.to_degrees(),
                rect.north.to_degrees(),
                4326,
            ),
        };
        let layers_param = self
            .layers
            .as_deref()
            .map(|l| format!("visible:{}", l))
            .unwrap_or_else(|| "visible".to_string());
        let mut url = format!(
            "{}/identify?f=json&tolerance=2&geometryType=esriGeometryPoint&geometry={}%2C{}&sr={}&layers={}&mapExtent={}%2C{}%2C{}%2C{}&imageDisplay={}%2C{}%2C96",
            self.url,
            geom_x,
            geom_y,
            sr,
            layers_param,
            w,
            s,
            e,
            n,
            self.tile_width,
            self.tile_height
        );
        if let Some(ref token) = self.token {
            url.push_str(&format!("&token={}", token));
        }
        Some(url)
    }
}

impl ImageryProvider for ArcGisMapServerImageryProvider {
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
        true
    }
    fn is_ready(&self) -> bool {
        true
    }
    fn ellipsoid(&self) -> Ellipsoid {
        self.ellipsoid
    }
}

fn clamp_rectangle(rect: &mut GeoRectangle, scheme: &AnyTilingScheme) {
    let scheme_rect = scheme.rectangle();
    rect.west = rect.west.max(scheme_rect.west);
    rect.south = rect.south.max(scheme_rect.south);
    rect.east = rect.east.min(scheme_rect.east);
    rect.north = rect.north.min(scheme_rect.north);
}
