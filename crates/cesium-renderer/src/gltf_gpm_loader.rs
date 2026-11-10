//! GltfGpmLoader for parsing NGA_gpm_local glTF extension.
//! Matches CesiumJS `Scene/Model/Extensions/Gpm/GltfGpmLoader.js`.

use crate::gltf_loader_util::ModelTextureReader;
use crate::gltf_texture_loader::SupportedImageFormats;
use cesium_math::{Cartesian3, DeveloperError};

/// Storage type for NGA_gpm_local data.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum GpmStorageType {
    Direct,
    Indirect,
}

impl GpmStorageType {
    pub fn from_str_name(name: &str) -> Option<Self> {
        match name {
            "Direct" => Some(Self::Direct),
            "Indirect" => Some(Self::Indirect),
            _ => None,
        }
    }
}

/// Direct anchor point definition.
#[derive(Clone, Debug, PartialEq)]
pub struct AnchorPointDirect {
    pub position: Cartesian3,
    pub adjustment_params: Cartesian3,
}

/// Indirect anchor point definition.
#[derive(Clone, Debug, PartialEq)]
pub struct AnchorPointIndirect {
    pub position: Cartesian3,
    pub adjustment_params: Cartesian3,
    pub covariance_matrix: [f64; 9],
}

/// Spatial parameter decorrelation function (SPDCF) parameters.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Spdcf {
    pub a: f64,
    pub alpha: f64,
    pub beta: f64,
    pub t: f64,
}

/// Intra-tile correlation group definition.
#[derive(Clone, Debug, PartialEq)]
pub struct CorrelationGroup {
    pub group_flags: Vec<bool>,
    pub rotation_thetas: Cartesian3,
    pub params: Vec<Spdcf>,
}

/// Parsed NGA_gpm_local extension data.
#[derive(Clone, Debug, PartialEq)]
pub struct GltfGpmLocal {
    pub storage_type: GpmStorageType,
    pub anchor_points_direct: Vec<AnchorPointDirect>,
    pub covariance_direct: Option<[f64; 9]>,
    pub anchor_points_indirect: Vec<AnchorPointIndirect>,
    pub intra_tile_correlation_groups: Vec<CorrelationGroup>,
}

/// Creates a symmetric 3x3 covariance matrix from the upper triangle components.
/// Array elements: [c0, c1, c2, c3, c4, c5] ->
/// [
///   c0, c1, c3,
///   c1, c2, c4,
///   c3, c4, c5,
/// ]
pub fn create_covariance_matrix_from_upper_triangle(array: &[f64]) -> Result<[f64; 9], DeveloperError> {
    if array.len() < 6 {
        return Err(DeveloperError::new(
            "Upper triangle covariance array must have at least 6 elements",
        ));
    }
    Ok([
        array[0], array[1], array[3],
        array[1], array[2], array[4],
        array[3], array[4], array[5],
    ])
}

/// Loader for glTF `NGA_gpm_local` root extension.
pub struct GltfGpmLoader;

impl GltfGpmLoader {
    /// Loads the GPM data from the given JSON extension object.
    pub fn load(json: &serde_json::Value) -> Result<GltfGpmLocal, DeveloperError> {
        let storage_type_str = json
            .get("storageType")
            .and_then(|v| v.as_str())
            .ok_or_else(|| DeveloperError::new("storageType is required"))?;

        let storage_type = GpmStorageType::from_str_name(storage_type_str).ok_or_else(|| {
            DeveloperError::new(format!(
                "Invalid storage type in NGA_gpm_local - expected 'Direct' or 'Indirect', but found {}",
                storage_type_str
            ))
        })?;

        match storage_type {
            GpmStorageType::Direct => Self::load_direct(json),
            GpmStorageType::Indirect => Self::load_indirect(json),
        }
    }

    /// Loads direct storage format.
    pub fn load_direct(json: &serde_json::Value) -> Result<GltfGpmLocal, DeveloperError> {
        let anchor_points_val = json
            .get("anchorPointsDirect")
            .and_then(|v| v.as_array())
            .ok_or_else(|| DeveloperError::new("anchorPointsDirect must be an array"))?;

        let cov_array_val = json
            .get("covarianceDirectUpperTriangle")
            .and_then(|v| v.as_array())
            .ok_or_else(|| {
                DeveloperError::new("covarianceDirectUpperTriangle must be an array")
            })?;

        let mut cov_upper = Vec::with_capacity(cov_array_val.len());
        for val in cov_array_val {
            let num = val
                .as_f64()
                .ok_or_else(|| DeveloperError::new("Covariance elements must be numbers"))?;
            cov_upper.push(num);
        }

        let covariance_direct = create_covariance_matrix_from_upper_triangle(&cov_upper)?;

        let mut anchor_points_direct = Vec::with_capacity(anchor_points_val.len());
        for ap_json in anchor_points_val {
            let pos_arr = ap_json
                .get("position")
                .and_then(|v| v.as_array())
                .ok_or_else(|| DeveloperError::new("position must be an array"))?;
            if pos_arr.len() < 3 {
                return Err(DeveloperError::new("position must have at least 3 elements"));
            }
            let pos = Cartesian3::new(
                pos_arr[0].as_f64().unwrap_or(0.0),
                pos_arr[1].as_f64().unwrap_or(0.0),
                pos_arr[2].as_f64().unwrap_or(0.0),
            );

            let adj_arr = ap_json
                .get("adjustmentParams")
                .and_then(|v| v.as_array())
                .ok_or_else(|| DeveloperError::new("adjustmentParams must be an array"))?;
            if adj_arr.len() < 3 {
                return Err(DeveloperError::new(
                    "adjustmentParams must have at least 3 elements",
                ));
            }
            let adj = Cartesian3::new(
                adj_arr[0].as_f64().unwrap_or(0.0),
                adj_arr[1].as_f64().unwrap_or(0.0),
                adj_arr[2].as_f64().unwrap_or(0.0),
            );

            anchor_points_direct.push(AnchorPointDirect {
                position: pos,
                adjustment_params: adj,
            });
        }

        Ok(GltfGpmLocal {
            storage_type: GpmStorageType::Direct,
            anchor_points_direct,
            covariance_direct: Some(covariance_direct),
            anchor_points_indirect: Vec::new(),
            intra_tile_correlation_groups: Vec::new(),
        })
    }

    /// Loads indirect storage format.
    pub fn load_indirect(json: &serde_json::Value) -> Result<GltfGpmLocal, DeveloperError> {
        let anchor_points_val = json
            .get("anchorPointsIndirect")
            .and_then(|v| v.as_array())
            .ok_or_else(|| DeveloperError::new("anchorPointsIndirect must be an array"))?;

        let corr_groups_val = json
            .get("intraTileCorrelationGroups")
            .and_then(|v| v.as_array())
            .ok_or_else(|| DeveloperError::new("intraTileCorrelationGroups must be an array"))?;

        let mut anchor_points_indirect = Vec::with_capacity(anchor_points_val.len());
        for ap_json in anchor_points_val {
            let pos_arr = ap_json
                .get("position")
                .and_then(|v| v.as_array())
                .ok_or_else(|| DeveloperError::new("position must be an array"))?;
            if pos_arr.len() < 3 {
                return Err(DeveloperError::new("position must have at least 3 elements"));
            }
            let pos = Cartesian3::new(
                pos_arr[0].as_f64().unwrap_or(0.0),
                pos_arr[1].as_f64().unwrap_or(0.0),
                pos_arr[2].as_f64().unwrap_or(0.0),
            );

            let adj_arr = ap_json
                .get("adjustmentParams")
                .and_then(|v| v.as_array())
                .ok_or_else(|| DeveloperError::new("adjustmentParams must be an array"))?;
            if adj_arr.len() < 3 {
                return Err(DeveloperError::new(
                    "adjustmentParams must have at least 3 elements",
                ));
            }
            let adj = Cartesian3::new(
                adj_arr[0].as_f64().unwrap_or(0.0),
                adj_arr[1].as_f64().unwrap_or(0.0),
                adj_arr[2].as_f64().unwrap_or(0.0),
            );

            let cov_arr = ap_json
                .get("covarianceMatrix")
                .and_then(|v| v.as_array())
                .ok_or_else(|| DeveloperError::new("covarianceMatrix must be an array"))?;
            let mut cov_upper = Vec::with_capacity(cov_arr.len());
            for val in cov_arr {
                let num = val.as_f64().ok_or_else(|| {
                    DeveloperError::new("Covariance elements must be numbers")
                })?;
                cov_upper.push(num);
            }
            let cov_mat = create_covariance_matrix_from_upper_triangle(&cov_upper)?;

            anchor_points_indirect.push(AnchorPointIndirect {
                position: pos,
                adjustment_params: adj,
                covariance_matrix: cov_mat,
            });
        }

        let mut intra_tile_correlation_groups = Vec::with_capacity(corr_groups_val.len());
        for cg_json in corr_groups_val {
            let flags_arr = cg_json
                .get("groupFlags")
                .and_then(|v| v.as_array())
                .ok_or_else(|| DeveloperError::new("groupFlags must be an array"))?;
            let mut group_flags = Vec::with_capacity(flags_arr.len());
            for flag in flags_arr {
                group_flags.push(flag.as_bool().unwrap_or(false));
            }

            let thetas_arr = cg_json
                .get("rotationThetas")
                .and_then(|v| v.as_array())
                .ok_or_else(|| DeveloperError::new("rotationThetas must be an array"))?;
            if thetas_arr.len() < 3 {
                return Err(DeveloperError::new(
                    "rotationThetas must have at least 3 elements",
                ));
            }
            let rotation_thetas = Cartesian3::new(
                thetas_arr[0].as_f64().unwrap_or(0.0),
                thetas_arr[1].as_f64().unwrap_or(0.0),
                thetas_arr[2].as_f64().unwrap_or(0.0),
            );

            let params_arr = cg_json
                .get("params")
                .and_then(|v| v.as_array())
                .ok_or_else(|| DeveloperError::new("params must be an array"))?;
            let mut params = Vec::with_capacity(params_arr.len());
            for p_json in params_arr {
                let a = p_json.get("A").and_then(|v| v.as_f64()).unwrap_or(0.0);
                let alpha = p_json.get("alpha").and_then(|v| v.as_f64()).unwrap_or(0.0);
                let beta = p_json.get("beta").and_then(|v| v.as_f64()).unwrap_or(0.0);
                let t = p_json.get("T").and_then(|v| v.as_f64()).unwrap_or(0.0);
                params.push(Spdcf {
                    a,
                    alpha,
                    beta,
                    t,
                });
            }

            intra_tile_correlation_groups.push(CorrelationGroup {
                group_flags,
                rotation_thetas,
                params,
            });
        }

        Ok(GltfGpmLocal {
            storage_type: GpmStorageType::Indirect,
            anchor_points_direct: Vec::new(),
            covariance_direct: None,
            anchor_points_indirect,
            intra_tile_correlation_groups,
        })
    }
}

/// Metadata traits for a PPE texture.
#[derive(Clone, Debug, PartialEq)]
pub struct PpeMetadata {
    pub min: f64,
    pub max: f64,
    pub source: String,
}

/// A single PPE texture descriptor.
#[derive(Clone, Debug, PartialEq)]
pub struct PpeTexture {
    pub traits: PpeMetadata,
    pub index: usize,
    pub no_data: u32,
    pub offset: f64,
    pub scale: f64,
    pub tex_coord: usize,
}

/// Parsed mesh primitive GPM local extension data.
#[derive(Clone, Debug, PartialEq, Default)]
pub struct MeshPrimitiveGpmLocal {
    pub ppe_textures: Vec<PpeTexture>,
}

impl MeshPrimitiveGpmLocal {
    pub fn new(ppe_textures: Vec<PpeTexture>) -> Self {
        Self { ppe_textures }
    }
}

/// Options for configuring GltfMeshPrimitiveGpmLoader.
#[derive(Clone, Debug, Default)]
pub struct GltfMeshPrimitiveGpmLoaderOptions {
    pub gltf: Option<serde_json::Value>,
    pub extension: Option<serde_json::Value>,
    pub gltf_resource: Option<String>,
    pub base_resource: Option<String>,
    pub supported_image_formats: Option<SupportedImageFormats>,
    pub frame_state: Option<serde_json::Value>,
}

/// Loader for `NGA_gpm_local` extension on a mesh primitive.
pub struct GltfMeshPrimitiveGpmLoader {
    pub gltf: serde_json::Value,
    extension: Option<serde_json::Value>,
    pub mesh_primitive_gpm_local: Option<MeshPrimitiveGpmLocal>,
    pub structural_metadata: Option<crate::gltf_structural_metadata_loader::StructuralMetadata>,
    is_loaded: bool,
    is_destroyed: bool,
    pub texture_destroy_count: usize,
}

impl GltfMeshPrimitiveGpmLoader {
    /// Creates a new `GltfMeshPrimitiveGpmLoader`.
    pub fn new(options: GltfMeshPrimitiveGpmLoaderOptions) -> Result<Self, DeveloperError> {
        let gltf = options
            .gltf
            .ok_or_else(|| DeveloperError::new("options.gltf is required"))?;
        if options.gltf_resource.is_none() {
            return Err(DeveloperError::new("options.gltfResource is required"));
        }
        if options.base_resource.is_none() {
            return Err(DeveloperError::new("options.baseResource is required"));
        }
        if options.supported_image_formats.is_none() {
            return Err(DeveloperError::new(
                "options.supportedImageFormats is required",
            ));
        }
        if options.frame_state.is_none() {
            return Err(DeveloperError::new("options.frameState is required"));
        }

        Ok(Self {
            gltf,
            extension: options.extension,
            mesh_primitive_gpm_local: None,
            structural_metadata: None,
            is_loaded: false,
            is_destroyed: false,
            texture_destroy_count: 0,
        })
    }

    /// Loads and parses the mesh primitive GPM extension into structural metadata.
    pub fn load(&mut self) -> Result<(), DeveloperError> {
        if self.is_destroyed {
            return Ok(());
        }

        let Some(ref extension) = self.extension else {
            self.is_loaded = true;
            return Ok(());
        };

        let mut ppe_textures = Vec::new();
        if let Some(ppe_textures_json) = extension.get("ppeTextures").and_then(|v| v.as_array()) {
            for tex_json in ppe_textures_json {
                let traits_json = tex_json.get("traits");
                let traits = PpeMetadata {
                    min: traits_json.and_then(|t| t.get("min")).and_then(|v| v.as_f64()).unwrap_or(0.0),
                    max: traits_json.and_then(|t| t.get("max")).and_then(|v| v.as_f64()).unwrap_or(0.0),
                    source: traits_json
                        .and_then(|t| t.get("source"))
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                };

                let index = tex_json.get("index").and_then(|v| v.as_u64()).unwrap_or(0) as usize;
                let no_data = tex_json.get("noData").and_then(|v| v.as_u64()).unwrap_or(255) as u32;
                let offset = tex_json.get("offset").and_then(|v| v.as_f64()).unwrap_or(0.0);
                let scale = tex_json.get("scale").and_then(|v| v.as_f64()).unwrap_or(1.0);
                let tex_coord = tex_json.get("texCoord").and_then(|v| v.as_u64()).unwrap_or(0) as usize;

                ppe_textures.push(PpeTexture {
                    traits,
                    index,
                    no_data,
                    offset,
                    scale,
                    tex_coord,
                });
            }
        }

        let gpm_local = MeshPrimitiveGpmLocal::new(ppe_textures);

        // Convert to StructuralMetadata
        let mut property_textures = Vec::new();
        let mut schema_classes = std::collections::BTreeMap::new();

        for (i, ppe_tex) in gpm_local.ppe_textures.iter().enumerate() {
            let class_id = format!("ppeTexture_{}", i);
            let prop_name = ppe_tex.traits.source.clone();

            let mut class_props = serde_json::Map::new();
            class_props.insert(
                prop_name.clone(),
                serde_json::json!({
                    "type": "SCALAR",
                    "componentType": "FLOAT32",
                    "min": ppe_tex.traits.min,
                    "max": ppe_tex.traits.max,
                    "offset": ppe_tex.offset,
                    "scale": ppe_tex.scale,
                    "noData": ppe_tex.no_data,
                }),
            );
            schema_classes.insert(
                class_id.clone(),
                serde_json::Value::Object(class_props),
            );

            let mut properties = std::collections::HashMap::new();
            properties.insert(
                prop_name,
                crate::gltf_structural_metadata_loader::PropertyTextureProperty {
                    texture_reader: ModelTextureReader {
                        index: ppe_tex.index,
                        texture: None,
                        tex_coord: ppe_tex.tex_coord,
                        scale: Some(ppe_tex.scale),
                        transform: None,
                        channels: Some("r".to_string()),
                        constant_lod: None,
                    },
                    channels: vec![0],
                },
            );

            property_textures.push(crate::gltf_structural_metadata_loader::PropertyTexture {
                id: i,
                name: Some(class_id.clone()),
                class_name: class_id,
                properties,
            });
        }

        let structural_metadata = crate::gltf_structural_metadata_loader::StructuralMetadata {
            schema: crate::gltf_structural_metadata_loader::MetadataSchema {
                id: Some("PPE_TEXTURE_SCHEMA_0".to_string()),
                classes: schema_classes,
            },
            property_tables: Vec::new(),
            property_textures,
        };

        self.mesh_primitive_gpm_local = Some(gpm_local);
        self.structural_metadata = Some(structural_metadata);
        self.is_loaded = true;

        Ok(())
    }

    /// Process loader progress.
    pub fn process(&mut self) -> Result<bool, DeveloperError> {
        if self.is_destroyed {
            return Ok(true);
        }
        Ok(self.is_loaded)
    }

    /// Destroys the loader and releases resources.
    pub fn destroy(&mut self) {
        if !self.is_destroyed {
            self.structural_metadata = None;
            self.texture_destroy_count += 1;
            self.is_destroyed = true;
        }
    }

    /// Returns whether this loader has been destroyed.
    pub fn is_destroyed(&self) -> bool {
        self.is_destroyed
    }
}
