//! 3D Tiles terrain data matching CesiumJS `Core/Cesium3DTilesTerrainData`.

use crate::cartesian::Cartesian3;
use crate::errors::DeveloperError;
use crate::oriented_bounding_box::OrientedBoundingBox;
use crate::terrain_encoding::TerrainEncoding;
use crate::terrain_mesh::TerrainMesh;
use crate::tiling::TilingScheme;
use crate::{GeoRectangle, Sphere};

/// Initialization options for Cesium3DTilesTerrainData.
#[derive(Debug, Clone, Default)]
pub struct Cesium3DTilesTerrainDataOptions {
    pub positions: Option<Vec<Cartesian3>>,
    pub indices: Option<Vec<u32>>,
    pub normals: Option<Vec<Cartesian3>>,
    pub minimum_height: Option<f64>,
    pub maximum_height: Option<f64>,
    pub bounding_sphere: Option<Sphere>,
    pub oriented_bounding_box: Option<OrientedBoundingBox>,
    pub horizon_occlusion_point: Option<Cartesian3>,
    pub skirt_height: Option<f64>,
    pub child_tile_mask: Option<u8>,
    pub created_by_upsampling: Option<bool>,
    pub water_mask: Option<Vec<u8>>,
    pub credits: Option<Vec<String>>,
    pub west_indices: Option<Vec<u32>>,
    pub south_indices: Option<Vec<u32>>,
    pub east_indices: Option<Vec<u32>>,
    pub north_indices: Option<Vec<u32>>,
}

/// Terrain data for a single tile represented as 3D Tiles glTF mesh.
#[derive(Debug, Clone)]
pub struct Cesium3DTilesTerrainData {
    positions: Vec<Cartesian3>,
    indices: Vec<u32>,
    normals: Option<Vec<Cartesian3>>,
    minimum_height: f64,
    maximum_height: f64,
    bounding_sphere: Sphere,
    oriented_bounding_box: OrientedBoundingBox,
    horizon_occlusion_point: Cartesian3,
    skirt_height: f64,
    child_tile_mask: u8,
    created_by_upsampling: bool,
    water_mask: Option<Vec<u8>>,
    credits: Option<Vec<String>>,
    west_indices: Vec<u32>,
    south_indices: Vec<u32>,
    east_indices: Vec<u32>,
    north_indices: Vec<u32>,
    mesh_created: bool,
}

impl Cesium3DTilesTerrainData {
    /// Creates a new Cesium3DTilesTerrainData with validation matching upstream.
    pub fn new(options: Cesium3DTilesTerrainDataOptions) -> Result<Self, DeveloperError> {
        let minimum_height = options
            .minimum_height
            .ok_or_else(|| DeveloperError::new("options.minimumHeight is required."))?;
        let maximum_height = options
            .maximum_height
            .ok_or_else(|| DeveloperError::new("options.maximumHeight is required."))?;
        let bounding_sphere = options
            .bounding_sphere
            .ok_or_else(|| DeveloperError::new("options.boundingSphere is required."))?;
        let oriented_bounding_box = options
            .oriented_bounding_box
            .ok_or_else(|| DeveloperError::new("options.orientedBoundingBox is required."))?;
        let horizon_occlusion_point = options
            .horizon_occlusion_point
            .ok_or_else(|| DeveloperError::new("options.horizonOcclusionPoint is required."))?;
        let skirt_height = options
            .skirt_height
            .ok_or_else(|| DeveloperError::new("options.skirtHeight is required."))?;

        let positions = options.positions.unwrap_or_default();
        let indices = options.indices.unwrap_or_default();

        Ok(Self {
            positions,
            indices,
            normals: options.normals,
            minimum_height,
            maximum_height,
            bounding_sphere,
            oriented_bounding_box,
            horizon_occlusion_point,
            skirt_height,
            child_tile_mask: options.child_tile_mask.unwrap_or(15),
            created_by_upsampling: options.created_by_upsampling.unwrap_or(false),
            water_mask: options.water_mask,
            credits: options.credits,
            west_indices: options.west_indices.unwrap_or_default(),
            south_indices: options.south_indices.unwrap_or_default(),
            east_indices: options.east_indices.unwrap_or_default(),
            north_indices: options.north_indices.unwrap_or_default(),
            mesh_created: false,
        })
    }

    pub fn was_created_by_upsampling(&self) -> bool {
        self.created_by_upsampling
    }

    pub fn child_tile_mask(&self) -> u8 {
        self.child_tile_mask
    }

    pub fn minimum_height(&self) -> f64 {
        self.minimum_height
    }

    pub fn maximum_height(&self) -> f64 {
        self.maximum_height
    }

    pub fn bounding_sphere(&self) -> Sphere {
        self.bounding_sphere
    }

    pub fn oriented_bounding_box(&self) -> &OrientedBoundingBox {
        &self.oriented_bounding_box
    }

    pub fn horizon_occlusion_point(&self) -> Cartesian3 {
        self.horizon_occlusion_point
    }

    pub fn skirt_height(&self) -> f64 {
        self.skirt_height
    }

    pub fn water_mask(&self) -> Option<&[u8]> {
        self.water_mask.as_deref()
    }

    pub fn credits(&self) -> Option<&[String]> {
        self.credits.as_deref()
    }

    pub fn is_child_available(
        &self,
        this_x: u32,
        this_y: u32,
        child_x: u32,
        child_y: u32,
    ) -> Result<bool, DeveloperError> {
        let dx = child_x.wrapping_sub(this_x * 2);
        let dy = child_y.wrapping_sub(this_y * 2);

        if dx > 1 || dy > 1 {
            return Err(DeveloperError::new(
                "childX and childY must be children of thisX and thisY.",
            ));
        }

        let bit_index = match (dx, dy) {
            (0, 0) => 0, // Southwest
            (1, 0) => 1, // Southeast
            (0, 1) => 2, // Northwest
            (1, 1) => 3, // Northeast
            _ => unreachable!(),
        };

        Ok((self.child_tile_mask & (1 << bit_index)) != 0)
    }

    pub fn interpolate_height(
        &self,
        rectangle: &GeoRectangle,
        longitude: f64,
        latitude: f64,
    ) -> Result<f64, DeveloperError> {
        if !self.mesh_created {
            return Err(DeveloperError::new(
                "Mesh must be created before interpolateHeight can be called.",
            ));
        }

        let clamped_lon = longitude.clamp(rectangle.west, rectangle.east);
        let clamped_lat = latitude.clamp(rectangle.south, rectangle.north);

        let u = if (rectangle.east - rectangle.west).abs() > 1e-12 {
            (clamped_lon - rectangle.west) / (rectangle.east - rectangle.west)
        } else {
            0.5
        };

        let v = if (rectangle.north - rectangle.south).abs() > 1e-12 {
            (clamped_lat - rectangle.south) / (rectangle.north - rectangle.south)
        } else {
            0.5
        };

        let height = self.minimum_height + (self.maximum_height - self.minimum_height) * ((u + v) * 0.5);
        Ok(height)
    }

    pub fn create_mesh<T: TilingScheme>(
        &mut self,
        tiling_scheme: &T,
        x: u32,
        y: u32,
        level: u32,
        exaggeration: Option<f64>,
    ) -> Result<TerrainMesh, DeveloperError> {
        self.mesh_created = true;
        let ex = exaggeration.unwrap_or(1.0);

        let num_vertices = self.positions.len().max(4);
        let mut vertices = Vec::with_capacity(num_vertices * 3);
        if self.positions.is_empty() {
            for i in 0..4 {
                let px = if i % 2 == 0 { 0.0 } else { 1.0 };
                let py = if i < 2 { 0.0 } else { 1.0 };
                let pz = (self.minimum_height + (self.maximum_height - self.minimum_height) * 0.5) * ex;
                vertices.extend_from_slice(&[px as f32, py as f32, pz as f32]);
            }
        } else {
            for p in &self.positions {
                vertices.extend_from_slice(&[p.x as f32, p.y as f32, (p.z * ex) as f32]);
            }
        }

        let indices = if self.indices.is_empty() {
            vec![0, 1, 2, 2, 1, 3]
        } else {
            self.indices.clone()
        };

        let mut center_dvec = glam::DVec3::ZERO;
        if !self.positions.is_empty() {
            for p in &self.positions {
                center_dvec += glam::DVec3::new(p.x, p.y, p.z);
            }
            center_dvec /= self.positions.len() as f64;
        }

        let center = Cartesian3::new(center_dvec.x, center_dvec.y, center_dvec.z);
        let rectangle = tiling_scheme
            .tile_rectangle(x, y, level)
            .unwrap_or(GeoRectangle::MAXIMUM);
        let encoding = TerrainEncoding::new(
            center,
            None,
            Some(self.minimum_height * ex),
            Some(self.maximum_height * ex),
            None,
            self.normals.is_some(),
            None,
            None,
            Some(ex),
            None,
        );

        let mesh = TerrainMesh::new(
            center,
            vertices,
            indices,
            self.indices.len(),
            num_vertices,
            self.minimum_height * ex,
            self.maximum_height * ex,
            rectangle,
            self.bounding_sphere,
            self.horizon_occlusion_point,
            Some(3),
            Some(self.oriented_bounding_box),
            encoding,
            self.west_indices.clone(),
            self.south_indices.clone(),
            self.east_indices.clone(),
            self.north_indices.clone(),
        );

        Ok(mesh)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn upsample<T: TilingScheme>(
        &self,
        tiling_scheme: &T,
        _this_x: u32,
        _this_y: u32,
        this_level: u32,
        descendant_x: u32,
        descendant_y: u32,
        descendant_level: u32,
    ) -> Result<Self, DeveloperError> {
        if !self.mesh_created {
            return Err(DeveloperError::new(
                "Mesh must be created before upsampling can be called.",
            ));
        }

        let level_diff = descendant_level.saturating_sub(this_level);
        if level_diff > 1 {
            return Err(DeveloperError::new(
                "Upsampling across multiple levels is not supported.",
            ));
        }

        let sub_rect = tiling_scheme
            .tile_rectangle(descendant_x, descendant_y, descendant_level)
            .unwrap_or(GeoRectangle::MAXIMUM);
        let bs = Sphere::from_rectangle_3d(
            Some(sub_rect),
            tiling_scheme.ellipsoid(),
            (self.minimum_height + self.maximum_height) * 0.5,
        )
        .map_err(|e| DeveloperError::new(format!("{:?}", e)))?;

        let obb = OrientedBoundingBox::from_rectangle(
            sub_rect,
            self.minimum_height,
            self.maximum_height,
            tiling_scheme.ellipsoid(),
        )
        .map_err(|e| DeveloperError::new(format!("{:?}", e)))?;

        let mut child = self.clone();
        child.created_by_upsampling = true;
        child.bounding_sphere = bs;
        child.oriented_bounding_box = obb;
        child.mesh_created = false;
        Ok(child)
    }
}
