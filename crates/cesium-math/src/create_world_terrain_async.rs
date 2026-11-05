//! Helper function matching CesiumJS `Core/createWorldTerrainAsync`.

use crate::cesium_terrain_provider::{CesiumTerrainProvider, CesiumTerrainProviderOptions};
use crate::ellipsoid::Ellipsoid;
use crate::errors::RuntimeError;

/// Options for `create_world_terrain_async`.
#[derive(Debug, Clone, Default)]
pub struct CreateWorldTerrainOptions {
    /// Flag indicating whether the client should request additional lighting / vertex normals information from the server if available.
    pub request_vertex_normals: bool,
    /// Flag indicating whether the client should request per tile water masks from the server if available.
    pub request_water_mask: bool,
    /// Ellipsoid for the terrain. Defaults to WGS84.
    pub ellipsoid: Option<Ellipsoid>,
}

/// Creates a `CesiumTerrainProvider` instance for the Cesium World Terrain (Ion asset ID 1).
pub async fn create_world_terrain_async(
    options: Option<CreateWorldTerrainOptions>,
) -> Result<CesiumTerrainProvider, RuntimeError> {
    let opts = options.unwrap_or_default();
    CesiumTerrainProvider::from_ion_asset_id(
        CesiumTerrainProvider::WORLD_TERRAIN_ASSET_ID,
        Some(CesiumTerrainProviderOptions {
            request_vertex_normals: opts.request_vertex_normals,
            request_water_mask: opts.request_water_mask,
            request_metadata: true,
            credit: None,
            ellipsoid: opts.ellipsoid.or(Some(Ellipsoid::WGS84)),
        }),
    )
    .await
}
