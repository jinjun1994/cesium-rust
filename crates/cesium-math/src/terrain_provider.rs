use std::sync::Arc;

use crate::ellipsoid::Ellipsoid;
use crate::errors::DeveloperError;
use crate::heightmap_terrain_data::{
    get_edge_indices, get_regular_grid_indices, EdgeIndices, HeightmapBuffer, HeightmapTerrainData,
    HeightmapTerrainDataOptions,
};
use crate::tiling::{GeographicTilingScheme, TilingScheme};

/// Default heightmap terrain quality factor used for geometric error calculation.
pub const HEIGHTMAP_TERRAIN_QUALITY: f64 = 0.25;

/// Static utilities and helpers for terrain providers.
pub struct TerrainProvider;

impl TerrainProvider {
    pub const HEIGHTMAP_TERRAIN_QUALITY: f64 = HEIGHTMAP_TERRAIN_QUALITY;

    /// Calculates estimated level zero geometric error for a heightmap.
    pub fn get_estimated_level_zero_geometric_error_for_a_heightmap(
        ellipsoid: &Ellipsoid,
        tile_image_width: usize,
        number_of_tiles_at_level_zero: u32,
    ) -> f64 {
        (ellipsoid.maximum_radius() * 2.0 * std::f64::consts::PI * HEIGHTMAP_TERRAIN_QUALITY)
            / ((tile_image_width as f64) * (number_of_tiles_at_level_zero as f64))
    }

    /// Gets regular grid indices for a given width and height.
    pub fn get_regular_grid_indices(width: usize, height: usize) -> Vec<u32> {
        get_regular_grid_indices(width, height)
    }

    /// Gets regular grid indices and four edge indices.
    pub fn get_regular_grid_indices_and_edge_indices(
        width: usize,
        height: usize,
    ) -> (Vec<u32>, EdgeIndices) {
        (
            get_regular_grid_indices(width, height),
            get_edge_indices(width, height),
        )
    }
}

/// A very simple terrain provider that produces flat geometry by tessellating an ellipsoidal surface.
#[derive(Debug, Clone)]
pub struct EllipsoidTerrainProvider {
    tiling_scheme: GeographicTilingScheme,
    level_zero_maximum_geometric_error: f64,
}

impl Default for EllipsoidTerrainProvider {
    fn default() -> Self {
        Self::new(None)
    }
}

impl EllipsoidTerrainProvider {
    pub fn new(tiling_scheme: Option<GeographicTilingScheme>) -> Self {
        let tiling_scheme = tiling_scheme.unwrap_or_default();
        let ellipsoid = tiling_scheme.ellipsoid();
        let x_tiles = tiling_scheme.number_of_x_tiles_at_level(0);
        let level_zero_maximum_geometric_error =
            TerrainProvider::get_estimated_level_zero_geometric_error_for_a_heightmap(
                &ellipsoid,
                64,
                x_tiles,
            );

        Self {
            tiling_scheme,
            level_zero_maximum_geometric_error,
        }
    }

    pub fn tiling_scheme(&self) -> &GeographicTilingScheme {
        &self.tiling_scheme
    }

    pub fn has_water_mask(&self) -> bool {
        false
    }

    pub fn has_vertex_normals(&self) -> bool {
        false
    }

    pub fn get_level_maximum_geometric_error(&self, level: u32) -> f64 {
        self.level_zero_maximum_geometric_error / ((1 << level) as f64)
    }

    pub fn get_tile_data_available(&self, _x: u32, _y: u32, _level: u32) -> Option<bool> {
        None
    }

    pub fn request_tile_geometry(
        &self,
        _x: u32,
        _y: u32,
        _level: u32,
    ) -> Option<HeightmapTerrainData> {
        let width = 16;
        let height = 16;
        HeightmapTerrainData::new(HeightmapTerrainDataOptions {
            buffer: Some(HeightmapBuffer::Uint8(vec![0; width * height])),
            width: Some(width),
            height: Some(height),
            ..Default::default()
        })
        .ok()
    }
}

/// Geometry request callback type for `CustomHeightmapTerrainProvider`.
pub type CustomHeightmapGeometryCallback =
    Arc<dyn Fn(u32, u32, u32) -> Option<HeightmapBuffer> + Send + Sync>;

/// Construction options for `CustomHeightmapTerrainProvider`.
#[derive(Clone)]
pub struct CustomHeightmapTerrainProviderOptions<T: TilingScheme = GeographicTilingScheme> {
    pub callback: Option<CustomHeightmapGeometryCallback>,
    pub width: Option<usize>,
    pub height: Option<usize>,
    pub tiling_scheme: Option<T>,
    pub credit: Option<String>,
}

impl<T: TilingScheme> Default for CustomHeightmapTerrainProviderOptions<T> {
    fn default() -> Self {
        Self {
            callback: None,
            width: None,
            height: None,
            tiling_scheme: None,
            credit: None,
        }
    }
}

/// A simple terrain provider that gets height values from a callback function.
#[derive(Clone)]
pub struct CustomHeightmapTerrainProvider<T: TilingScheme = GeographicTilingScheme> {
    callback: CustomHeightmapGeometryCallback,
    width: usize,
    height: usize,
    tiling_scheme: T,
    credit: Option<String>,
    level_zero_maximum_geometric_error: f64,
}

impl CustomHeightmapTerrainProvider<GeographicTilingScheme> {
    pub fn new(
        options: CustomHeightmapTerrainProviderOptions<GeographicTilingScheme>,
    ) -> Result<Self, DeveloperError> {
        Self::new_with_tiling_scheme(options)
    }
}

impl<T: TilingScheme + Default> CustomHeightmapTerrainProvider<T> {
    pub fn new_with_tiling_scheme(
        options: CustomHeightmapTerrainProviderOptions<T>,
    ) -> Result<Self, DeveloperError> {
        let callback = options
            .callback
            .ok_or_else(|| DeveloperError::new("options.callback is required."))?;
        let width = options
            .width
            .ok_or_else(|| DeveloperError::new("options.width is required."))?;
        let height = options
            .height
            .ok_or_else(|| DeveloperError::new("options.height is required."))?;

        let tiling_scheme = options.tiling_scheme.unwrap_or_default();
        let max_dim = width.max(height);
        let ellipsoid = tiling_scheme.ellipsoid();
        let x_tiles = tiling_scheme
            .tile_counts(0)
            .map(|(x, _)| x)
            .unwrap_or(2);
        let level_zero_maximum_geometric_error =
            TerrainProvider::get_estimated_level_zero_geometric_error_for_a_heightmap(
                &ellipsoid,
                max_dim,
                x_tiles,
            );

        Ok(Self {
            callback,
            width,
            height,
            tiling_scheme,
            credit: options.credit,
            level_zero_maximum_geometric_error,
        })
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn credit(&self) -> Option<&str> {
        self.credit.as_deref()
    }

    pub fn tiling_scheme(&self) -> &T {
        &self.tiling_scheme
    }

    pub fn has_water_mask(&self) -> bool {
        false
    }

    pub fn has_vertex_normals(&self) -> bool {
        false
    }

    pub fn get_level_maximum_geometric_error(&self, level: u32) -> f64 {
        self.level_zero_maximum_geometric_error / ((1 << level) as f64)
    }

    pub fn get_tile_data_available(&self, _x: u32, _y: u32, _level: u32) -> Option<bool> {
        None
    }

    pub fn request_tile_geometry(
        &self,
        x: u32,
        y: u32,
        level: u32,
    ) -> Option<HeightmapTerrainData> {
        let buffer = (self.callback)(x, y, level)?;
        HeightmapTerrainData::new(HeightmapTerrainDataOptions {
            buffer: Some(buffer),
            width: Some(self.width),
            height: Some(self.height),
            ..Default::default()
        })
        .ok()
    }
}
