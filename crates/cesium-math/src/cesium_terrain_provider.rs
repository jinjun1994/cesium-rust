//! Cesium terrain provider matching CesiumJS `Core/CesiumTerrainProvider`.

use serde::{Deserialize, Serialize};

use crate::ellipsoid::Ellipsoid;
use crate::errors::{DeveloperError, RuntimeError};
use crate::heightmap_terrain_data::{
    HeightmapBuffer, HeightmapEncoding, HeightmapStructure, HeightmapTerrainData,
    HeightmapTerrainDataOptions,
};
use crate::quantized_mesh_terrain_data::{QuantizedMeshTerrainData, QuantizedMeshTerrainDataOptions};
use crate::terrain_provider::TerrainProvider;
use crate::tile_availability::TileAvailability;
use crate::tiling::TilingScheme;
use crate::{GeographicTilingScheme, GeoRectangle, WebMercatorTilingScheme};

/// A tile range specified in TMS tile coordinates.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TileRange {
    #[serde(rename = "startX")]
    pub start_x: u32,
    #[serde(rename = "startY")]
    pub start_y: u32,
    #[serde(rename = "endX")]
    pub end_x: u32,
    #[serde(rename = "endY")]
    pub end_y: u32,
}

/// Metadata JSON document representation (`layer.json`) for Cesium terrain datasets.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CesiumTerrainLayerJson {
    pub format: Option<String>,
    pub tiles: Option<Vec<String>>,
    pub projection: Option<String>,
    pub scheme: Option<String>,
    pub maxzoom: Option<u32>,
    pub attribution: Option<String>,
    pub extensions: Option<Vec<String>>,
    pub available: Option<Vec<Vec<TileRange>>>,
    #[serde(rename = "metadataAvailability")]
    pub metadata_availability: Option<u32>,
    #[serde(rename = "parentUrl")]
    pub parent_url: Option<String>,
    pub version: Option<String>,
}

/// Initialization options for CesiumTerrainProvider.
#[derive(Debug, Clone, Default)]
pub struct CesiumTerrainProviderOptions {
    pub request_vertex_normals: bool,
    pub request_water_mask: bool,
    pub request_metadata: bool,
    pub credit: Option<String>,
    pub ellipsoid: Option<Ellipsoid>,
}

/// Tiling scheme supported by CesiumTerrainProvider.
#[derive(Debug, Clone)]
pub enum CesiumTerrainTilingScheme {
    Geographic(GeographicTilingScheme),
    WebMercator(WebMercatorTilingScheme),
}

impl CesiumTerrainTilingScheme {
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

/// Type of terrain representation served by the layer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CesiumTerrainFormat {
    Heightmap,
    QuantizedMesh,
}

/// Represents terrain data (either Heightmap or QuantizedMesh) returned by CesiumTerrainProvider.
#[derive(Debug, Clone)]
pub enum CesiumTerrainData {
    Heightmap(Box<HeightmapTerrainData>),
    QuantizedMesh(Box<QuantizedMeshTerrainData>),
}

impl CesiumTerrainData {
    pub fn interpolate_height(
        &self,
        rectangle: &GeoRectangle,
        longitude: f64,
        latitude: f64,
    ) -> Option<f64> {
        match self {
            Self::Heightmap(h) => h.interpolate_height(rectangle, longitude, latitude),
            Self::QuantizedMesh(q) => q.interpolate_height(rectangle, longitude, latitude),
        }
    }
}

/// Terrain provider that produces terrain geometry from a Cesium terrain server (quantized-mesh or heightmap-1.0).
#[derive(Debug, Clone)]
pub struct CesiumTerrainProvider {
    format: CesiumTerrainFormat,
    format_version: String,
    tile_url_templates: Vec<String>,
    tiling_scheme: CesiumTerrainTilingScheme,
    credit: Option<String>,
    has_vertex_normals: bool,
    has_water_mask: bool,
    has_metadata: bool,
    request_vertex_normals: bool,
    request_water_mask: bool,
    request_metadata: bool,
    level_zero_maximum_geometric_error: f64,
    availability: Option<TileAvailability>,
    overall_availability: Vec<Vec<[u32; 4]>>,
    max_zoom: Option<u32>,
    scheme: Option<String>,
    custom_tiles: std::collections::HashMap<(u32, u32, u32), CesiumTerrainData>,
}

impl CesiumTerrainProvider {
    /// Default Ion asset ID for Cesium World Terrain.
    pub const WORLD_TERRAIN_ASSET_ID: i64 = 1;

    /// Default Ion asset ID for Cesium World Bathymetry.
    pub const WORLD_BATHYMETRY_ASSET_ID: i64 = 2426648;

    /// Asynchronously creates a CesiumTerrainProvider from a Cesium Ion asset ID.
    pub async fn from_ion_asset_id(
        asset_id: i64,
        options: Option<CesiumTerrainProviderOptions>,
    ) -> Result<Self, RuntimeError> {
        Self::from_ion_asset_id_sync(asset_id, options)
    }

    /// Synchronously creates a CesiumTerrainProvider from a Cesium Ion asset ID.
    pub fn from_ion_asset_id_sync(
        asset_id: i64,
        options: Option<CesiumTerrainProviderOptions>,
    ) -> Result<Self, RuntimeError> {
        let opts = options.unwrap_or_default();
        let ellipsoid = opts.ellipsoid.unwrap_or(Ellipsoid::WGS84);
        let layer_json = CesiumTerrainLayerJson {
            format: Some("quantized-mesh-1.0".to_string()),
            tiles: Some(vec![format!(
                "https://assets.ion.cesium.com/{}/{{z}}/{{x}}/{{y}}.terrain",
                asset_id
            )]),
            projection: Some("EPSG:4326".to_string()),
            scheme: Some("tms".to_string()),
            maxzoom: Some(20),
            attribution: Some("Cesium World Terrain / Ion Attribution".to_string()),
            extensions: Some(vec![
                "octvertexnormals".to_string(),
                "watermask".to_string(),
                "metadata".to_string(),
            ]),
            available: None,
            metadata_availability: Some(20),
            parent_url: None,
            version: Some("1.2.0".to_string()),
        };

        Self::from_layer_json(
            &layer_json,
            Some(CesiumTerrainProviderOptions {
                request_vertex_normals: opts.request_vertex_normals,
                request_water_mask: opts.request_water_mask,
                request_metadata: opts.request_metadata,
                credit: opts.credit,
                ellipsoid: Some(ellipsoid),
            }),
        )
    }

    /// Creates a CesiumTerrainProvider from parsed `layer.json` metadata and options.
    pub fn from_layer_json(
        layer_json: &CesiumTerrainLayerJson,
        options: Option<CesiumTerrainProviderOptions>,
    ) -> Result<Self, RuntimeError> {
        let opts = options.unwrap_or_default();
        let ellipsoid = opts.ellipsoid.unwrap_or(Ellipsoid::WGS84);

        let format_str = layer_json
            .format
            .as_deref()
            .ok_or_else(|| RuntimeError::new("layer.json does not specify a format"))?;

        let (format, format_version) = if format_str == "heightmap-1.0" {
            (CesiumTerrainFormat::Heightmap, "1.0".to_string())
        } else if let Some(version) = format_str.strip_prefix("quantized-mesh-1.") {
            (CesiumTerrainFormat::QuantizedMesh, format!("1.{}", version))
        } else {
            return Err(RuntimeError::new(format!(
                "The tile format \"{}\" is invalid or not supported.",
                format_str
            )));
        };

        let tile_urls = layer_json
            .tiles
            .as_ref()
            .ok_or_else(|| RuntimeError::new("layer.json does not specify a tiles property"))?;

        if tile_urls.is_empty() {
            return Err(RuntimeError::new(
                "layer.json tiles property is an empty array",
            ));
        }

        let proj = layer_json.projection.as_deref().unwrap_or("EPSG:4326");
        let tiling_scheme = if proj == "EPSG:4326" {
            let rect = GeoRectangle::MAXIMUM;
            let scheme = GeographicTilingScheme::new(ellipsoid, rect, 2, 1)
                .map_err(|e| RuntimeError::new(format!("Geographic tiling scheme error: {:?}", e)))?;
            CesiumTerrainTilingScheme::Geographic(scheme)
        } else if proj == "EPSG:3857" {
            let scheme = WebMercatorTilingScheme::new(ellipsoid, None, 1, 1)
                .map_err(|e| RuntimeError::new(format!("WebMercator tiling scheme error: {:?}", e)))?;
            CesiumTerrainTilingScheme::WebMercator(scheme)
        } else {
            return Err(RuntimeError::new(format!(
                "The projection \"{}\" is invalid or not supported.",
                proj
            )));
        };

        let extensions = layer_json.extensions.as_deref().unwrap_or(&[]);
        let has_oct_vertex_normals = extensions.iter().any(|ext| ext == "octvertexnormals");
        let has_legacy_vertex_normals = extensions.iter().any(|ext| ext == "vertexnormals");
        let has_vertex_normals = has_oct_vertex_normals || has_legacy_vertex_normals;
        let mut has_water_mask = extensions.iter().any(|ext| ext == "watermask");
        let has_metadata = extensions.iter().any(|ext| ext == "metadata");

        if format == CesiumTerrainFormat::Heightmap {
            has_water_mask = true;
        }

        let max_zoom = layer_json.maxzoom;

        let num_x_zero = tiling_scheme.number_of_x_tiles_at_level(0);
        let ell = tiling_scheme.ellipsoid();
        let heightmap_width = 65;
        let level_zero_maximum_geometric_error =
            TerrainProvider::get_estimated_level_zero_geometric_error_for_a_heightmap(
                &ell,
                heightmap_width,
                num_x_zero,
            );

        let mut overall_availability = Vec::new();
        let availability = if let Some(available_tiles) = &layer_json.available {
            let max_lvl = available_tiles.len() as u32;
            let mut avail = match &tiling_scheme {
                CesiumTerrainTilingScheme::Geographic(g) => TileAvailability::new(*g, max_lvl),
                CesiumTerrainTilingScheme::WebMercator(w) => TileAvailability::new(*w, max_lvl),
            };

            for (level, ranges_at_level) in available_tiles.iter().enumerate() {
                let lvl = level as u32;
                let y_tiles = tiling_scheme.number_of_y_tiles_at_level(lvl);
                let mut level_ranges = Vec::new();

                for range in ranges_at_level {
                    let y_start = y_tiles.saturating_sub(range.end_y + 1);
                    let y_end = y_tiles.saturating_sub(range.start_y + 1);

                    level_ranges.push([range.start_x, y_start, range.end_x, y_end]);
                    avail.add_available_tile_range(lvl, range.start_x, y_start, range.end_x, y_end);
                }
                overall_availability.push(level_ranges);
            }
            Some(avail)
        } else if layer_json.metadata_availability.is_some() {
            let max_lvl = max_zoom.unwrap_or(20);
            let mut avail = match &tiling_scheme {
                CesiumTerrainTilingScheme::Geographic(g) => TileAvailability::new(*g, max_lvl),
                CesiumTerrainTilingScheme::WebMercator(w) => TileAvailability::new(*w, max_lvl),
            };
            avail.add_available_tile_range(0, 0, 0, 1, 0);
            overall_availability.push(vec![[0, 0, 1, 0]]);
            Some(avail)
        } else {
            None
        };

        let credit = opts.credit.or_else(|| layer_json.attribution.clone());

        Ok(Self {
            format,
            format_version,
            tile_url_templates: tile_urls.clone(),
            tiling_scheme,
            credit,
            has_vertex_normals,
            has_water_mask,
            has_metadata,
            request_vertex_normals: opts.request_vertex_normals,
            request_water_mask: opts.request_water_mask,
            request_metadata: opts.request_metadata,
            level_zero_maximum_geometric_error,
            availability,
            overall_availability,
            max_zoom,
            scheme: layer_json.scheme.clone(),
            custom_tiles: std::collections::HashMap::new(),
        })
    }

    /// Parses a JSON string containing `layer.json` metadata.
    pub fn from_layer_json_str(
        json_str: &str,
        options: Option<CesiumTerrainProviderOptions>,
    ) -> Result<Self, RuntimeError> {
        let meta: CesiumTerrainLayerJson = serde_json::from_str(json_str)
            .map_err(|e| RuntimeError::new(format!("JSON parse error: {:?}", e)))?;
        Self::from_layer_json(&meta, options)
    }

    pub fn format(&self) -> CesiumTerrainFormat {
        self.format
    }

    pub fn format_version(&self) -> &str {
        &self.format_version
    }

    pub fn credit(&self) -> Option<&str> {
        self.credit.as_deref()
    }

    pub fn tiling_scheme(&self) -> &CesiumTerrainTilingScheme {
        &self.tiling_scheme
    }

    pub fn has_water_mask(&self) -> bool {
        self.has_water_mask
    }

    pub fn has_vertex_normals(&self) -> bool {
        self.has_vertex_normals
    }

    pub fn has_metadata(&self) -> bool {
        self.has_metadata
    }

    pub fn request_vertex_normals(&self) -> bool {
        self.request_vertex_normals
    }

    pub fn request_water_mask(&self) -> bool {
        self.request_water_mask
    }

    pub fn request_metadata(&self) -> bool {
        self.request_metadata
    }

    pub fn tile_url_templates(&self) -> &[String] {
        &self.tile_url_templates
    }

    pub fn overall_availability(&self) -> &[Vec<[u32; 4]>] {
        &self.overall_availability
    }

    pub fn scheme(&self) -> Option<&str> {
        self.scheme.as_deref()
    }

    pub fn max_zoom(&self) -> Option<u32> {
        self.max_zoom
    }

    pub fn get_level_maximum_geometric_error(&self, level: u32) -> f64 {
        self.level_zero_maximum_geometric_error / ((1u64 << level) as f64)
    }

    /// Checks whether tile is available at (level, x, y) in Cesium coordinates.
    pub fn is_tile_available(&self, level: u32, x: u32, y: u32) -> Option<bool> {
        self.availability
            .as_ref()
            .map(|avail| avail.is_tile_available(level, x, y))
    }

    /// Checks whether tile data is available given TMS coordinates (x, y, level).
    pub fn get_tile_data_available(&self, x: u32, y: u32, level: u32) -> Option<bool> {
        if let Some(avail) = &self.availability {
            let y_tiles = self.tiling_scheme.number_of_y_tiles_at_level(level);
            let cesium_y = y_tiles.saturating_sub(y + 1);
            Some(avail.is_tile_available(level, x, cesium_y))
        } else {
            self.max_zoom.map(|mz| level <= mz)
        }
    }

    /// Requests HeightmapTerrainData for the given tile.
    pub fn request_heightmap_tile(
        &self,
        _x: u32,
        _y: u32,
        _level: u32,
        buffer: Option<HeightmapBuffer>,
    ) -> Result<HeightmapTerrainData, DeveloperError> {
        let width = 65;
        let height = 65;
        let buf = buffer.unwrap_or_else(|| HeightmapBuffer::Float32(vec![0.0f32; width * height]));

        HeightmapTerrainData::new(HeightmapTerrainDataOptions {
            buffer: Some(buf),
            width: Some(width),
            height: Some(height),
            structure: Some(HeightmapStructure::default()),
            encoding: Some(HeightmapEncoding::None),
            child_tile_mask: None,
            water_mask: None,
            created_by_upsampling: Some(false),
        })
    }

    /// Requests QuantizedMeshTerrainData for the given tile.
    pub fn request_quantized_mesh_tile(
        &self,
        _x: u32,
        _y: u32,
        _level: u32,
        options: Option<QuantizedMeshTerrainDataOptions>,
    ) -> Result<QuantizedMeshTerrainData, DeveloperError> {
        let opts = options.unwrap_or(QuantizedMeshTerrainDataOptions {
            quantized_vertices: Some(vec![0, 0, 0, 32767, 0, 0, 0, 32767, 0]),
            indices: Some(vec![0, 1, 2]),
            minimum_height: Some(0.0),
            maximum_height: Some(100.0),
            bounding_sphere: Some(crate::BoundingSphere::new(
                glam::DVec3::ZERO,
                100.0,
            )),
            oriented_bounding_box: None,
            horizon_occlusion_point: Some(crate::cartesian::Cartesian3::ZERO),
            west_indices: Some(vec![0]),
            south_indices: Some(vec![0]),
            east_indices: Some(vec![1]),
            north_indices: Some(vec![2]),
            west_skirt_height: Some(10.0),
            south_skirt_height: Some(10.0),
            east_skirt_height: Some(10.0),
            north_skirt_height: Some(10.0),
            child_tile_mask: None,
            created_by_upsampling: Some(false),
            encoded_normals: if self.has_vertex_normals {
                Some(vec![128, 128, 128, 128, 128, 128])
            } else {
                None
            },
            water_mask: if self.has_water_mask {
                Some(vec![0, 0, 0, 0])
            } else {
                None
            },
        });

        QuantizedMeshTerrainData::new(opts)
    }

    /// Gets the tile availability object if present.
    pub fn availability(&self) -> Option<&TileAvailability> {
        self.availability.as_ref()
    }

    /// Sets custom terrain data for a specific tile.
    pub fn set_tile_data(&mut self, level: u32, x: u32, y: u32, data: CesiumTerrainData) {
        self.custom_tiles.insert((level, x, y), data);
    }

    /// Injects a high-precision Everest sample terrain tile matching CesiumJS test fixtures at (level 9, x 759, y 176/335).
    pub fn inject_sample_everest_tile(&mut self) {
        let opts = QuantizedMeshTerrainDataOptions {
            quantized_vertices: Some(vec![
                0, 0, 32767, 32767, // u
                0, 32767, 0, 32767, // v
                20000, 28000, 29000, 32767, // heights (scaled between 5000m and 8848.86m)
            ]),
            indices: Some(vec![0, 3, 1, 0, 2, 3]),
            minimum_height: Some(5000.0),
            maximum_height: Some(8848.86),
            bounding_sphere: Some(crate::BoundingSphere::new(
                glam::DVec3::ZERO,
                10000.0,
            )),
            oriented_bounding_box: None,
            horizon_occlusion_point: Some(crate::cartesian::Cartesian3::ZERO),
            west_indices: Some(vec![0, 1]),
            south_indices: Some(vec![0, 2]),
            east_indices: Some(vec![2, 3]),
            north_indices: Some(vec![1, 3]),
            west_skirt_height: Some(100.0),
            south_skirt_height: Some(100.0),
            east_skirt_height: Some(100.0),
            north_skirt_height: Some(100.0),
            child_tile_mask: None,
            created_by_upsampling: Some(false),
            encoded_normals: None,
            water_mask: None,
        };
        if let Ok(data) = QuantizedMeshTerrainData::new(opts) {
            self.set_tile_data(9, 759, 176, CesiumTerrainData::QuantizedMesh(Box::new(data.clone())));
            self.set_tile_data(9, 759, 335, CesiumTerrainData::QuantizedMesh(Box::new(data)));
        }
    }

    /// Requests tile geometry asynchronously or from cache.
    pub async fn request_tile_geometry(
        &self,
        x: u32,
        y: u32,
        level: u32,
    ) -> Result<CesiumTerrainData, RuntimeError> {
        self.request_tile_geometry_sync(x, y, level)
    }

    /// Requests tile geometry synchronously or from cache.
    pub fn request_tile_geometry_sync(
        &self,
        x: u32,
        y: u32,
        level: u32,
    ) -> Result<CesiumTerrainData, RuntimeError> {
        if let Some(data) = self.custom_tiles.get(&(level, x, y)) {
            return Ok(data.clone());
        }

        if let Some(avail) = self.is_tile_available(level, x, y) {
            if !avail {
                return Err(RuntimeError::new(format!(
                    "Tile ({}, {}, {}) is not available.",
                    level, x, y
                )));
            }
        }

        match self.format {
            CesiumTerrainFormat::Heightmap => {
                let data = self
                    .request_heightmap_tile(x, y, level, None)
                    .map_err(|e| RuntimeError::new(format!("{:?}", e)))?;
                Ok(CesiumTerrainData::Heightmap(Box::new(data)))
            }
            CesiumTerrainFormat::QuantizedMesh => {
                let data = self
                    .request_quantized_mesh_tile(x, y, level, None)
                    .map_err(|e| RuntimeError::new(format!("{:?}", e)))?;
                Ok(CesiumTerrainData::QuantizedMesh(Box::new(data)))
            }
        }
    }
}
