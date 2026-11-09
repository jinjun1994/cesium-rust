use cesium_math::Cartesian3;
use cesium_renderer::gltf_gpm_loader::{GltfGpmLoader, GpmStorageType};
use serde_json::json;

#[test]
fn test_load_throws_with_invalid_storage_type() {
    let gltf_gpm_local_json = json!({
        "storageType": "INVALID"
    });

    let result = GltfGpmLoader::load(&gltf_gpm_local_json);
    assert!(result.is_err());
}

#[test]
fn test_load_throws_for_storage_type_direct_without_anchor_points_direct() {
    let gltf_gpm_local_json = json!({
        "storageType": "Direct"
    });

    let result = GltfGpmLoader::load(&gltf_gpm_local_json);
    assert!(result.is_err());
}

#[test]
fn test_load_throws_for_storage_type_direct_without_covariance_direct_upper_triangle() {
    let gltf_gpm_local_json = json!({
        "storageType": "Direct",
        "anchorPointsDirect": [
            {
                "position": [1.0, 2.0, 3.0],
                "adjustmentParams": [0.1, 0.2, 0.3]
            }
        ]
    });

    let result = GltfGpmLoader::load(&gltf_gpm_local_json);
    assert!(result.is_err());
}

#[test]
fn test_load_returns_result_for_valid_json_for_storage_type_direct() {
    let gltf_gpm_local_json = json!({
        "storageType": "Direct",
        "anchorPointsDirect": [
            {
                "position": [1.0, 2.0, 3.0],
                "adjustmentParams": [0.1, 0.2, 0.3]
            }
        ],
        "covarianceDirectUpperTriangle": [0.1, 0.2, 0.3, 0.4, 0.5, 0.6]
    });

    let result = GltfGpmLoader::load(&gltf_gpm_local_json).expect("Should parse direct GPM");
    assert_eq!(result.storage_type, GpmStorageType::Direct);
    assert_eq!(result.anchor_points_direct.len(), 1);

    let actual_anchor_point = &result.anchor_points_direct[0];
    let expected_position = Cartesian3::new(1.0, 2.0, 3.0);
    assert_eq!(actual_anchor_point.position, expected_position);

    let expected_adjustment_params = Cartesian3::new(0.1, 0.2, 0.3);
    assert_eq!(actual_anchor_point.adjustment_params, expected_adjustment_params);

    let expected_covariance = [0.1, 0.2, 0.4, 0.2, 0.3, 0.5, 0.4, 0.5, 0.6];
    let actual_cov = result.covariance_direct.expect("Covariance direct must exist");
    for (a, b) in actual_cov.iter().zip(expected_covariance.iter()) {
        assert!((a - b).abs() < 1e-6);
    }
}

#[test]
fn test_load_throws_for_storage_type_indirect_without_anchor_points_indirect() {
    let gltf_gpm_local_json = json!({
        "storageType": "Indirect"
    });

    let result = GltfGpmLoader::load(&gltf_gpm_local_json);
    assert!(result.is_err());
}

#[test]
fn test_load_throws_for_storage_type_indirect_without_intra_tile_correlation_groups() {
    let gltf_gpm_local_json = json!({
        "storageType": "Indirect",
        "anchorPointsIndirect": [
            {
                "position": [1.0, 2.0, 3.0],
                "adjustmentParams": [0.1, 0.2, 0.3]
            }
        ]
    });

    let result = GltfGpmLoader::load(&gltf_gpm_local_json);
    assert!(result.is_err());
}

#[test]
fn test_load_returns_result_for_valid_json_for_storage_type_indirect() {
    let gltf_gpm_local_json = json!({
        "storageType": "Indirect",
        "anchorPointsIndirect": [
            {
                "position": [1.0, 2.0, 3.0],
                "adjustmentParams": [0.1, 0.2, 0.3],
                "covarianceMatrix": [0.1, 0.2, 0.3, 0.4, 0.5, 0.6]
            }
        ],
        "intraTileCorrelationGroups": [
            {
                "groupFlags": [true, true, true],
                "rotationThetas": [0.1, 0.2, 0.3],
                "params": [
                    {
                        "A": 0.1,
                        "alpha": 0.2,
                        "beta": 0.3,
                        "T": 0.4
                    }
                ]
            }
        ]
    });

    let result = GltfGpmLoader::load(&gltf_gpm_local_json).expect("Should parse indirect GPM");
    assert_eq!(result.storage_type, GpmStorageType::Indirect);
    assert_eq!(result.anchor_points_indirect.len(), 1);

    let actual_anchor_point = &result.anchor_points_indirect[0];
    let expected_position = Cartesian3::new(1.0, 2.0, 3.0);
    assert_eq!(actual_anchor_point.position, expected_position);

    let expected_adjustment_params = Cartesian3::new(0.1, 0.2, 0.3);
    assert_eq!(actual_anchor_point.adjustment_params, expected_adjustment_params);

    let expected_cov = [0.1, 0.2, 0.4, 0.2, 0.3, 0.5, 0.4, 0.5, 0.6];
    for (a, b) in actual_anchor_point.covariance_matrix.iter().zip(expected_cov.iter()) {
        assert!((a - b).abs() < 1e-6);
    }

    assert_eq!(result.intra_tile_correlation_groups.len(), 1);
    let correlation_group = &result.intra_tile_correlation_groups[0];
    assert_eq!(correlation_group.group_flags, vec![true, true, true]);

    let expected_rotation_thetas = Cartesian3::new(0.1, 0.2, 0.3);
    assert_eq!(correlation_group.rotation_thetas, expected_rotation_thetas);

    assert_eq!(correlation_group.params.len(), 1);
    let param = correlation_group.params[0];
    assert!((param.a - 0.1).abs() < 1e-6);
    assert!((param.alpha - 0.2).abs() < 1e-6);
    assert!((param.beta - 0.3).abs() < 1e-6);
    assert!((param.t - 0.4).abs() < 1e-6);
}
