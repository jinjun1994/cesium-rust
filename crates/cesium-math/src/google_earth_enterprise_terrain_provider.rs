use crate::{
    GeographicTilingScheme, GoogleEarthEnterpriseMetadata, GoogleEarthEnterpriseTerrainData,
    GoogleEarthEnterpriseTerrainDataOptions, RuntimeError,
};

/// Options for GoogleEarthEnterpriseTerrainProvider.
#[derive(Debug, Clone, Default)]
pub struct GoogleEarthEnterpriseTerrainProviderOptions {
    pub credit: Option<String>,
}

/// Provides tiled terrain using the Google Earth Enterprise REST API.
#[derive(Debug, Clone)]
pub struct GoogleEarthEnterpriseTerrainProvider {
    tiling_scheme: GeographicTilingScheme,
    credit: Option<String>,
    level_zero_maximum_geometric_error: f64,
    metadata: GoogleEarthEnterpriseMetadata,
}

impl GoogleEarthEnterpriseTerrainProvider {
    pub fn from_metadata(
        metadata: GoogleEarthEnterpriseMetadata,
        options: Option<GoogleEarthEnterpriseTerrainProviderOptions>,
    ) -> Result<Self, RuntimeError> {
        if !metadata.terrain_present {
            return Err(RuntimeError::new(format!(
                "The server {} doesn't have terrain",
                metadata.url
            )));
        }

        let options = options.unwrap_or_default();

        Ok(Self {
            tiling_scheme: GeographicTilingScheme::default(),
            credit: options.credit,
            level_zero_maximum_geometric_error: 40075.16,
            metadata,
        })
    }

    pub fn from_url(
        url: impl Into<String>,
        options: Option<GoogleEarthEnterpriseTerrainProviderOptions>,
    ) -> Result<Self, RuntimeError> {
        let metadata = GoogleEarthEnterpriseMetadata::from_url(url)
            .map_err(|e| RuntimeError::new(e.message))?;
        Self::from_metadata(metadata, options)
    }

    pub fn metadata(&self) -> &GoogleEarthEnterpriseMetadata {
        &self.metadata
    }

    pub fn metadata_mut(&mut self) -> &mut GoogleEarthEnterpriseMetadata {
        &mut self.metadata
    }

    pub fn tiling_scheme(&self) -> &GeographicTilingScheme {
        &self.tiling_scheme
    }

    pub fn credit(&self) -> Option<&str> {
        self.credit.as_deref()
    }

    pub fn has_water_mask(&self) -> bool {
        false
    }

    pub fn has_vertex_normals(&self) -> bool {
        false
    }

    pub fn get_level_zero_maximum_geometric_error(&self) -> f64 {
        self.level_zero_maximum_geometric_error
    }

    pub fn get_tile_data_available(&self, x: u32, y: u32, level: u32) -> bool {
        let quadkey = GoogleEarthEnterpriseMetadata::tile_xy_to_quad_key(x, y, level);
        if let Some(info) = self.metadata.get_tile_information_from_quad_key(&quadkey) {
            info.has_terrain()
        } else {
            self.metadata.is_valid(&quadkey)
        }
    }

    pub fn request_tile_geometry(
        &self,
        x: u32,
        y: u32,
        level: u32,
    ) -> Result<GoogleEarthEnterpriseTerrainData, RuntimeError> {
        let quadkey = GoogleEarthEnterpriseMetadata::tile_xy_to_quad_key(x, y, level);
        let info = self
            .metadata
            .get_tile_information_from_quad_key(&quadkey)
            .ok_or_else(|| RuntimeError::new("Terrain tile doesn't exist"))?;

        if !info.has_terrain() && !info.ancestor_has_terrain {
            return Err(RuntimeError::new("Terrain tile has no terrain data."));
        }

        let child_mask = info.get_child_bitmask();
        let terrain_data = GoogleEarthEnterpriseTerrainData::new(
            GoogleEarthEnterpriseTerrainDataOptions {
                buffer: vec![0u8; 16], // minimal valid buffer
                negative_altitude_exponent_bias: self.metadata.negative_altitude_exponent_bias,
                negative_elevation_threshold: self.metadata.negative_altitude_threshold,
                child_tile_mask: Some(child_mask),
                created_by_upsampling: Some(false),
                credits: self.credit.clone().map(|c| vec![c]),
            },
        )
        .map_err(|e| RuntimeError::new(e.message))?;

        Ok(terrain_data)
    }
}
