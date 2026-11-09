//! GltfGpmLoader for parsing NGA_gpm_local glTF extension.
//! Matches CesiumJS `Scene/Model/Extensions/Gpm/GltfGpmLoader.js`.

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
