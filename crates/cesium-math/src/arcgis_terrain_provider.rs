//! ArcGIS Tiled Elevation Terrain Provider matching CesiumJS `Core/ArcGISTiledElevationTerrainProvider`.

use serde::{Deserialize, Serialize};

use crate::ellipsoid::Ellipsoid;
use crate::errors::{DeveloperError, RuntimeError};
use crate::heightmap_terrain_data::{
    HeightmapBuffer, HeightmapEncoding, HeightmapStructure, HeightmapTerrainData,
    HeightmapTerrainDataOptions,
};
use crate::terrain_provider::TerrainProvider;
use crate::tile_availability::TileAvailability;
use crate::tiling::TilingScheme;
use crate::{GeographicTilingScheme, GeoRectangle, WebMercatorTilingScheme};

/// Spatial reference information from ArcGIS service metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArcGISMetadataSpatialReference {
    pub wkid: Option<u32>,
    #[serde(rename = "latestWkid")]
    pub latest_wkid: Option<u32>,
}

/// Bounding extent from ArcGIS service metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArcGISMetadataExtent {
    pub xmin: f64,
    pub ymin: f64,
    pub xmax: f64,
    pub ymax: f64,
}

/// Level of detail (LOD) info from ArcGIS service metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArcGISMetadataLod {
    pub level: u32,
    pub resolution: Option<f64>,
    pub scale: Option<f64>,
}

/// Tile format and grid configuration from ArcGIS service metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArcGISMetadataTileInfo {
    pub rows: usize,
    pub cols: usize,
    pub format: Option<String>,
    #[serde(default)]
    pub lods: Vec<ArcGISMetadataLod>,
}

/// Root ArcGIS service metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArcGISMetadata {
    #[serde(rename = "copyrightText")]
    pub copyright_text: Option<String>,
    #[serde(rename = "spatialReference")]
    pub spatial_reference: Option<ArcGISMetadataSpatialReference>,
    pub extent: Option<ArcGISMetadataExtent>,
    #[serde(rename = "tileInfo")]
    pub tile_info: Option<ArcGISMetadataTileInfo>,
    #[serde(default)]
    pub capabilities: Option<String>,
}

/// Tiling scheme used by ArcGIS Tiled Elevation Terrain Provider.
#[derive(Debug, Clone)]
pub enum ArcGISTilingScheme {
    Geographic(GeographicTilingScheme),
    WebMercator(WebMercatorTilingScheme),
}

impl ArcGISTilingScheme {
    pub fn ellipsoid(&self) -> Ellipsoid {
        match self {
            Self::Geographic(g) => g.ellipsoid(),
            Self::WebMercator(w) => w.ellipsoid(),
        }
    }

    pub fn number_of_x_tiles_at_level(&self, level: u32) -> u32 {
        match self {
            Self::Geographic(g) => g.number_of_x_tiles_at_level(level),
            Self::WebMercator(w) => w.number_of_x_tiles_at_level(level),
        }
    }

    pub fn number_of_y_tiles_at_level(&self, level: u32) -> u32 {
        match self {
            Self::Geographic(g) => g.number_of_y_tiles_at_level(level),
            Self::WebMercator(w) => w.number_of_y_tiles_at_level(level),
        }
    }
}

/// Terrain provider that produces terrain geometry from an ArcGIS ImageServer elevation service.
#[derive(Debug, Clone)]
pub struct ArcGISTiledElevationTerrainProvider {
    credit: Option<String>,
    tiling_scheme: ArcGISTilingScheme,
    width: usize,
    height: usize,
    encoding: HeightmapEncoding,
    lod_count: usize,
    has_availability: bool,
    tiles_available: Option<TileAvailability>,
    level_zero_maximum_geometric_error: f64,
}

impl ArcGISTiledElevationTerrainProvider {
    /// Creates a provider by parsing ArcGIS service metadata.
    pub fn from_metadata(
        metadata: &ArcGISMetadata,
        ellipsoid: Option<Ellipsoid>,
    ) -> Result<Self, RuntimeError> {
        let ellip = ellipsoid.unwrap_or(Ellipsoid::WGS84);
        let credit = metadata.copyright_text.clone();

        let spatial_ref = metadata
            .spatial_reference
            .as_ref()
            .ok_or_else(|| RuntimeError::new("spatialReference is required"))?;
        let wkid = spatial_ref.latest_wkid.or(spatial_ref.wkid).unwrap_or(0);

        let extent = metadata
            .extent
            .as_ref()
            .ok_or_else(|| RuntimeError::new("extent is required"))?;

        let tiling_scheme = if wkid == 4326 {
            let rect = GeoRectangle::degrees(
                extent.xmin,
                extent.ymin,
                extent.xmax,
                extent.ymax,
            )
            .map_err(|e| RuntimeError::new(format!("Invalid extent: {:?}", e)))?;
            ArcGISTilingScheme::Geographic(
                GeographicTilingScheme::new(ellip, rect, 2, 1)
                    .map_err(|e| RuntimeError::new(format!("Tiling scheme error: {:?}", e)))?,
            )
        } else if wkid == 3857 || wkid == 102100 {
            let epsg3857_bounds = std::f64::consts::PI * ellip.maximum_radius();
            let mut xmin = extent.xmin;
            let mut ymin = extent.ymin;
            let mut xmax = extent.xmax;
            let mut ymax = extent.ymax;

            if xmax > epsg3857_bounds {
                xmax = epsg3857_bounds;
            }
            if ymax > epsg3857_bounds {
                ymax = epsg3857_bounds;
            }
            if xmin < -epsg3857_bounds {
                xmin = -epsg3857_bounds;
            }
            if ymin < -epsg3857_bounds {
                ymin = -epsg3857_bounds;
            }

            ArcGISTilingScheme::WebMercator(
                WebMercatorTilingScheme::new(
                    ellip,
                    Some((glam::DVec2::new(xmin, ymin), glam::DVec2::new(xmax, ymax))),
                    1,
                    1,
                )
                .map_err(|e| RuntimeError::new(format!("Tiling scheme error: {:?}", e)))?,
            )
        } else {
            return Err(RuntimeError::new("Invalid spatial reference"));
        };

        let tile_info = metadata
            .tile_info
            .as_ref()
            .ok_or_else(|| RuntimeError::new("tileInfo is required"))?;

        let width = tile_info.rows + 1;
        let height = tile_info.cols + 1;
        let encoding = if tile_info.format.as_deref() == Some("LERC") {
            HeightmapEncoding::Lerc
        } else {
            HeightmapEncoding::None
        };
        let lod_count = tile_info.lods.len().saturating_sub(1);

        let has_availability = metadata
            .capabilities
            .as_deref()
            .map(|c| c.contains("Tilemap"))
            .unwrap_or(false);

        let tiles_available = if has_availability {
            let num_x = tiling_scheme.number_of_x_tiles_at_level(0);
            let num_y = tiling_scheme.number_of_y_tiles_at_level(0);
            let mut avail = match &tiling_scheme {
                ArcGISTilingScheme::Geographic(g) => TileAvailability::new(*g, lod_count as u32),
                ArcGISTilingScheme::WebMercator(w) => TileAvailability::new(*w, lod_count as u32),
            };
            avail.add_available_tile_range(0, 0, 0, num_x.saturating_sub(1), num_y.saturating_sub(1));
            Some(avail)
        } else {
            None
        };

        let num_x_zero = tiling_scheme.number_of_x_tiles_at_level(0);
        let ell = tiling_scheme.ellipsoid();
        let level_zero_maximum_geometric_error =
            TerrainProvider::get_estimated_level_zero_geometric_error_for_a_heightmap(
                &ell,
                width,
                num_x_zero,
            );

        Ok(Self {
            credit,
            tiling_scheme,
            width,
            height,
            encoding,
            lod_count,
            has_availability,
            tiles_available,
            level_zero_maximum_geometric_error,
        })
    }

    /// Parses JSON metadata string into provider.
    pub fn from_metadata_json(
        metadata_json: &str,
        ellipsoid: Option<Ellipsoid>,
    ) -> Result<Self, RuntimeError> {
        let meta: ArcGISMetadata = serde_json::from_str(metadata_json)
            .map_err(|e| RuntimeError::new(format!("JSON parse error: {:?}", e)))?;
        Self::from_metadata(&meta, ellipsoid)
    }

    pub fn credit(&self) -> Option<&str> {
        self.credit.as_deref()
    }

    pub fn tiling_scheme(&self) -> &ArcGISTilingScheme {
        &self.tiling_scheme
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn encoding(&self) -> HeightmapEncoding {
        self.encoding
    }

    pub fn lod_count(&self) -> usize {
        self.lod_count
    }

    pub fn has_water_mask(&self) -> bool {
        false
    }

    pub fn has_vertex_normals(&self) -> bool {
        false
    }

    pub fn has_availability(&self) -> bool {
        self.has_availability
    }

    pub fn is_tile_available(&self, level: u32, x: u32, y: u32) -> Option<bool> {
        if self.has_availability {
            self.tiles_available
                .as_ref()
                .map(|avail| avail.is_tile_available(level, x, y))
        } else if (level as usize) <= self.lod_count {
            Some(true)
        } else {
            Some(false)
        }
    }

    pub fn get_level_maximum_geometric_error(&self, level: u32) -> f64 {
        self.level_zero_maximum_geometric_error / ((1u64 << level) as f64)
    }

    pub fn request_tile_geometry(
        &self,
        _x: u32,
        _y: u32,
        _level: u32,
        buffer: Option<HeightmapBuffer>,
    ) -> Result<HeightmapTerrainData, DeveloperError> {
        let buf = buffer.unwrap_or_else(|| {
            HeightmapBuffer::Float32(vec![0.0f32; self.width * self.height])
        });

        HeightmapTerrainData::new(HeightmapTerrainDataOptions {
            buffer: Some(buf),
            width: Some(self.width),
            height: Some(self.height),
            structure: Some(HeightmapStructure::default()),
            encoding: Some(self.encoding),
            child_tile_mask: None,
            water_mask: None,
            created_by_upsampling: Some(false),
        })
    }
}
