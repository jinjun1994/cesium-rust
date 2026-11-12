use cesium_math::{Cartesian3, Ellipsoid, Matrix4};
use cesium_renderer::cesium_3d_tiles_voxel_provider::Cesium3DTilesVoxelProvider;
use cesium_renderer::metadata_component_type::MetadataComponentType;
use cesium_renderer::metadata_type::MetadataType;
use cesium_renderer::voxel_primitive::VoxelProvider;
use cesium_renderer::voxel_shape_type::VoxelShapeType;
use serde_json::json;

fn voxel_ellipsoid_tileset_json() -> serde_json::Value {
    json!({
        "asset": { "version": "1.1" },
        "schema": {
            "id": "voxel_schema",
            "classes": {
                "voxel": {
                    "properties": {
                        "a": {
                            "type": "SCALAR",
                            "componentType": "FLOAT32"
                        }
                    }
                }
            }
        },
        "statistics": {
            "classes": {
                "voxel": {
                    "properties": {
                        "a": { "min": 0.0, "max": 1.0 }
                    }
                }
            }
        },
        "root": {
            "boundingVolume": {
                "region": [0.0, 0.0, 1.0, 1.0, -1.0, 500000.0]
            },
            "implicitTiling": {
                "subdivisionScheme": "OCTREE",
                "subtreeLevels": 1,
                "availableLevels": 1,
                "subtrees": { "uri": "subtrees/{level}.{x}.{y}.{z}.subtree" }
            },
            "content": {
                "uri": "content/{level}_{x}_{y}_{z}.glb",
                "extensions": {
                    "3DTILES_content_voxels": {
                        "class": "voxel",
                        "dimensions": [2, 2, 2]
                    }
                }
            }
        }
    })
}

fn voxel_box_tileset_json() -> serde_json::Value {
    json!({
        "asset": { "version": "1.1" },
        "schema": {
            "id": "voxel_schema",
            "classes": {
                "voxel": {
                    "properties": {
                        "a": { "type": "SCALAR", "componentType": "FLOAT32" }
                    }
                }
            }
        },
        "root": {
            "boundingVolume": {
                "box": [0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0]
            },
            "implicitTiling": {
                "subdivisionScheme": "OCTREE",
                "subtreeLevels": 1,
                "availableLevels": 1,
                "subtrees": { "uri": "subtrees/{level}.{x}.{y}.{z}.subtree" }
            },
            "content": {
                "uri": "content/{level}_{x}_{y}_{z}.glb",
                "extensions": {
                    "3DTILES_content_voxels": {
                        "class": "voxel",
                        "dimensions": [2, 2, 2]
                    }
                }
            }
        }
    })
}

fn voxel_cylinder_tileset_json() -> serde_json::Value {
    json!({
        "asset": { "version": "1.1" },
        "schema": {
            "id": "voxel_schema",
            "classes": {
                "voxel": {
                    "properties": {
                        "a": { "type": "SCALAR", "componentType": "FLOAT32" }
                    }
                }
            }
        },
        "root": {
            "boundingVolume": {
                "extensions": {
                    "3DTILES_bounding_volume_cylinder": {
                        "minRadius": 0.0,
                        "maxRadius": 1.0,
                        "height": 2.0
                    }
                }
            },
            "implicitTiling": {
                "subdivisionScheme": "OCTREE",
                "subtreeLevels": 1,
                "availableLevels": 1,
                "subtrees": { "uri": "subtrees/{level}.{x}.{y}.{z}.subtree" }
            },
            "content": {
                "uri": "content/{level}_{x}_{y}_{z}.glb",
                "extensions": {
                    "3DTILES_content_voxels": {
                        "class": "voxel",
                        "dimensions": [2, 2, 2]
                    }
                }
            }
        }
    })
}

#[test]
fn conforms_to_voxel_provider_interface() {
    let provider =
        Cesium3DTilesVoxelProvider::from_tileset_json(&voxel_ellipsoid_tileset_json(), None)
            .unwrap();
    let trait_obj: &dyn VoxelProvider = &provider;
    assert_eq!(trait_obj.shape_type(), VoxelShapeType::Ellipsoid);
    assert_eq!(trait_obj.dimensions(), Cartesian3::new(2.0, 2.0, 2.0));
}

#[test]
fn from_tileset_json_creates_ellipsoid_voxel_provider() {
    let provider =
        Cesium3DTilesVoxelProvider::from_tileset_json(&voxel_ellipsoid_tileset_json(), None)
            .unwrap();

    assert_eq!(provider.global_transform(), Matrix4::IDENTITY);
    let radii = Ellipsoid::WGS84.radii();
    let expected_shape_transform = Matrix4::from_scale(Cartesian3::new(radii.x, radii.y, radii.z));
    assert!(provider
        .shape_transform()
        .equals_epsilon(&expected_shape_transform, 1e-10));
    assert_eq!(provider.shape(), VoxelShapeType::Ellipsoid);
    assert_eq!(provider.min_bounds(), Cartesian3::new(0.0, 0.0, -1.0));
    assert_eq!(provider.max_bounds(), Cartesian3::new(1.0, 1.0, 500000.0));
    assert_eq!(provider.dimensions(), Cartesian3::new(2.0, 2.0, 2.0));
    assert_eq!(provider.padding_before(), Cartesian3::ZERO);
    assert_eq!(provider.padding_after(), Cartesian3::ZERO);
    assert_eq!(provider.names(), &["a".to_string()]);
    assert_eq!(provider.types(), &[MetadataType::Scalar]);
    assert_eq!(
        provider.component_types(),
        &[MetadataComponentType::Float32]
    );
    assert_eq!(provider.minimum_values().unwrap(), &[vec![0.0]]);
    assert_eq!(provider.maximum_values().unwrap(), &[vec![1.0]]);
}

#[test]
fn request_data_works_for_ellipsoid_box_and_cylinder_root_tiles() {
    for json_doc in [
        voxel_ellipsoid_tileset_json(),
        voxel_box_tileset_json(),
        voxel_cylinder_tileset_json(),
    ] {
        let provider =
            Cesium3DTilesVoxelProvider::from_tileset_json(&json_doc, None).unwrap();
        let mut content = provider.request_data(0, 0, 0, 0, None).unwrap();
        assert!(content.update());
        assert!(content.ready);
        assert_eq!(content.metadata.len(), 1);
        let dims = provider.dimensions();
        let voxel_count = (dims.x * dims.y * dims.z) as usize;
        let expected_len = voxel_count * provider.types()[0].get_component_count();
        assert_eq!(content.metadata[0].len(), expected_len);
    }
}

#[test]
fn request_data_loads_multiple_attributes_correctly() {
    let multi_json = json!({
        "asset": { "version": "1.1" },
        "schema": {
            "id": "multi_schema",
            "classes": {
                "voxel": {
                    "properties": {
                        "a": { "type": "SCALAR", "componentType": "FLOAT32" },
                        "b": { "type": "SCALAR", "componentType": "FLOAT32" },
                        "c": { "type": "SCALAR", "componentType": "FLOAT32" }
                    }
                }
            }
        },
        "root": {
            "boundingVolume": {
                "box": [0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0]
            },
            "implicitTiling": {
                "subdivisionScheme": "OCTREE",
                "subtreeLevels": 1,
                "availableLevels": 1,
                "subtrees": { "uri": "subtrees/{level}.{x}.{y}.{z}.subtree" }
            },
            "content": {
                "uri": "content/{level}_{x}_{y}_{z}.glb",
                "extensions": {
                    "3DTILES_content_voxels": {
                        "class": "voxel",
                        "dimensions": [2, 2, 2]
                    }
                }
            }
        }
    });

    let provider = Cesium3DTilesVoxelProvider::from_tileset_json(&multi_json, None).unwrap();
    let buffers = vec![vec![0.0f32; 8], vec![0.5f32; 8], vec![1.0f32; 8]];
    let content = provider
        .request_data(0, 0, 0, 0, Some(&buffers))
        .unwrap();
    assert_eq!(content.metadata.len(), 3);
    assert_eq!(content.metadata[0][0], 0.0);
    assert_eq!(content.metadata[1][0], 0.5);
    assert_eq!(content.metadata[2][0], 1.0);
}
