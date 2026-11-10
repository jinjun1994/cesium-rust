//! ModelType matching CesiumJS `Scene/Model/ModelType.js` (Apache-2.0).

use cesium_math::DeveloperError;
use std::str::FromStr;

/// An enum to distinguish the different uses for Model,
/// which include individual glTF models, and various 3D Tiles formats.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub enum ModelType {
    /// An individual glTF model.
    #[default]
    Gltf,
    /// A glTF model used as tile content in a 3D Tileset via 3DTILES_content_gltf.
    TileGltf,
    /// A 3D Tiles 1.0 Batched 3D Model.
    TileB3dm,
    /// A 3D Tiles 1.0 Instanced 3D Model.
    TileI3dm,
    /// A 3D Tiles 1.0 Point Cloud.
    TilePnts,
    /// GeoJSON content for MAXAR_content_geojson extension.
    TileGeoJson,
}

impl ModelType {
    /// Checks if a model is used for 3D Tiles.
    pub fn is_3d_tiles(model_type: Self) -> bool {
        matches!(
            model_type,
            Self::TileGltf
                | Self::TileB3dm
                | Self::TileI3dm
                | Self::TilePnts
                | Self::TileGeoJson
        )
    }

    /// Checks if a string representation of modelType is 3D Tiles.
    pub fn is_3d_tiles_str(model_type_str: Option<&str>) -> Result<bool, DeveloperError> {
        let s = model_type_str.ok_or_else(|| DeveloperError::new("modelType is required."))?;
        let parsed = ModelType::from_str(s)?;
        Ok(Self::is_3d_tiles(parsed))
    }
}

impl FromStr for ModelType {
    type Err = DeveloperError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "GLTF" => Ok(Self::Gltf),
            "TILE_GLTF" => Ok(Self::TileGltf),
            "B3DM" | "TILE_B3DM" => Ok(Self::TileB3dm),
            "I3DM" | "TILE_I3DM" => Ok(Self::TileI3dm),
            "PNTS" | "TILE_PNTS" => Ok(Self::TilePnts),
            "TILE_GEOJSON" => Ok(Self::TileGeoJson),
            _ => Err(DeveloperError::new(format!(
                "modelType is not a valid value: {s}"
            ))),
        }
    }
}
