//! Helper function matching CesiumJS `Core/createWorldBathymetryAsync`.

use crate::cesium_terrain_provider::{CesiumTerrainProvider, CesiumTerrainProviderOptions};
use crate::errors::RuntimeError;

/// Options for `create_world_bathymetry_async`.
#[derive(Debug, Clone, Default)]
pub struct CreateWorldBathymetryOptions {
    /// Flag indicating whether the client should request additional lighting / vertex normals information from the server if available.
    pub request_vertex_normals: bool,
}

/// Creates a `CesiumTerrainProvider` instance for the Cesium World Bathymetry (Ion asset ID 2426648).
pub async fn create_world_bathymetry_async(
    options: Option<CreateWorldBathymetryOptions>,
) -> Result<CesiumTerrainProvider, RuntimeError> {
    let opts = options.unwrap_or_default();
    CesiumTerrainProvider::from_ion_asset_id(
        CesiumTerrainProvider::WORLD_BATHYMETRY_ASSET_ID,
        Some(CesiumTerrainProviderOptions {
            request_vertex_normals: opts.request_vertex_normals,
            request_water_mask: false,
            request_metadata: true,
            credit: None,
            ellipsoid: None,
        }),
    )
    .await
}
