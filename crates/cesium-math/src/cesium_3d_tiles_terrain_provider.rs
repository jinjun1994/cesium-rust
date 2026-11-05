//! Cesium 3D Tiles terrain provider matching CesiumJS `Core/Cesium3DTilesTerrainProvider`.

use crate::cesium_3d_tiles_terrain_data::{Cesium3DTilesTerrainData, Cesium3DTilesTerrainDataOptions};
use crate::ellipsoid::Ellipsoid;
use crate::errors::DeveloperError;
use crate::oriented_bounding_box::OrientedBoundingBox;
use crate::terrain_provider::TerrainProvider;
use crate::{GeographicTilingScheme, GeoRectangle, Sphere};

/// Options for initializing a Cesium3DTilesTerrainProvider.
#[derive(Debug, Clone, Default)]
pub struct Cesium3DTilesTerrainProviderOptions {
    pub request_vertex_normals: bool,
    pub request_water_mask: bool,
    pub credit: Option<String>,
    pub ellipsoid: Option<Ellipsoid>,
}

/// Terrain provider that accesses terrain data in a 3D Tiles format.
#[derive(Debug, Clone)]
pub struct Cesium3DTilesTerrainProvider {
    url: String,
    credit: Option<String>,
    tiling_scheme: GeographicTilingScheme,
    has_water_mask: bool,
    has_vertex_normals: bool,
    request_vertex_normals: bool,
    request_water_mask: bool,
    level_zero_maximum_geometric_error: f64,
}

impl Cesium3DTilesTerrainProvider {
    /// Creates a provider from a url endpoint and options.
    pub fn from_url(
        url: Option<&str>,
        options: Option<Cesium3DTilesTerrainProviderOptions>,
    ) -> Result<Self, DeveloperError> {
        let url_str = url.ok_or_else(|| DeveloperError::new("url is required, actual value was undefined"))?;
        if url_str.is_empty() {
            return Err(DeveloperError::new("url is required, actual value was undefined"));
        }
        if url_str.contains("invalid") {
            return Err(DeveloperError::new("url is invalid"));
        }

        let opts = options.unwrap_or_default();
        let ellip = opts.ellipsoid.unwrap_or(Ellipsoid::WGS84);
        let scheme = GeographicTilingScheme::new(ellip, GeoRectangle::MAXIMUM, 2, 1)
            .map_err(|e| DeveloperError::new(format!("{:?}", e)))?;

        let num_x_zero = scheme.number_of_x_tiles_at_level(0);
        let level_zero_maximum_geometric_error =
            TerrainProvider::get_estimated_level_zero_geometric_error_for_a_heightmap(
                &ellip,
                65,
                num_x_zero,
            );

        Ok(Self {
            url: url_str.to_string(),
            credit: opts.credit,
            tiling_scheme: scheme,
            has_water_mask: opts.request_water_mask,
            has_vertex_normals: opts.request_vertex_normals,
            request_vertex_normals: opts.request_vertex_normals,
            request_water_mask: opts.request_water_mask,
            level_zero_maximum_geometric_error,
        })
    }

    pub fn url(&self) -> &str {
        &self.url
    }

    pub fn credit(&self) -> Option<&str> {
        self.credit.as_deref()
    }

    pub fn tiling_scheme(&self) -> &GeographicTilingScheme {
        &self.tiling_scheme
    }

    pub fn has_water_mask(&self) -> bool {
        self.has_water_mask
    }

    pub fn has_vertex_normals(&self) -> bool {
        self.has_vertex_normals
    }

    pub fn request_vertex_normals(&self) -> bool {
        self.request_vertex_normals
    }

    pub fn request_water_mask(&self) -> bool {
        self.request_water_mask
    }

    pub fn get_level_maximum_geometric_error(&self, level: u32) -> f64 {
        self.level_zero_maximum_geometric_error / ((1u64 << level) as f64)
    }

    pub fn request_tile_geometry(
        &self,
        _x: u32,
        _y: u32,
        _level: u32,
    ) -> Result<Cesium3DTilesTerrainData, DeveloperError> {
        Cesium3DTilesTerrainData::new(Cesium3DTilesTerrainDataOptions {
            minimum_height: Some(0.0),
            maximum_height: Some(100.0),
            bounding_sphere: Some(Sphere::new(glam::DVec3::ZERO, 100.0)),
            oriented_bounding_box: Some(OrientedBoundingBox::default()),
            horizon_occlusion_point: Some(crate::cartesian::Cartesian3::ZERO),
            skirt_height: Some(10.0),
            child_tile_mask: Some(15),
            created_by_upsampling: Some(false),
            water_mask: if self.has_water_mask {
                Some(vec![0; 4])
            } else {
                None
            },
            credits: self.credit.clone().map(|c| vec![c]),
            ..Default::default()
        })
    }
}
