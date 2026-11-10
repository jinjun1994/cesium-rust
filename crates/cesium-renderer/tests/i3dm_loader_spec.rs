use std::fs;
use std::path::PathBuf;

use cesium_math::{Matrix4, EPSILON8};
use cesium_renderer::i3dm_loader::{I3dmLoader, I3dmLoaderOptions};

fn fixture_path(subpath: &str) -> PathBuf {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let path = manifest_dir
        .join("../../../cesium/Specs/Data/Cesium3DTiles/Instanced")
        .join(subpath);
    if path.exists() {
        path
    } else {
        PathBuf::from("../cesium/Specs/Data/Cesium3DTiles/Instanced").join(subpath)
    }
}

async fn load_i3dm(subpath: &str) -> I3dmLoader {
    let path = fixture_path(subpath);
    let bytes = fs::read(&path)
        .unwrap_or_else(|e| panic!("Failed to read fixture {}: {e}", path.display()));

    let mut loader = I3dmLoader::new(I3dmLoaderOptions {
        i3dm_resource: Some(path.to_string_lossy().to_string()),
        base_resource: Some(path.to_string_lossy().to_string()),
        array_buffer: Some(bytes),
        ..Default::default()
    })
    .expect("I3dmLoader creation failed");

    loader.load().await.expect("load() failed");
    let ready = loader.process().expect("process() failed");
    assert!(ready, "loader should be ready after process()");

    loader
}

fn verify_instances(loader: &I3dmLoader, expected_semantics: &[&str], instances_length: usize) {
    let components = loader.components().expect("components must be defined");
    assert!(components.structural_metadata.is_some());

    let mut buffer_count = 0;
    for node in &components.nodes {
        if !node.primitives.is_empty() {
            let instances = node.instances.as_ref().expect("node must have instances");
            assert_eq!(instances.attributes.len(), expected_semantics.len());

            let has_rotation = expected_semantics.contains(&"ROTATION");

            for attribute in &instances.attributes {
                let sem = attribute.semantic.as_deref().unwrap();
                assert!(expected_semantics.contains(&sem));
                assert_eq!(attribute.count, instances_length);

                let is_transform = sem == "TRANSLATION" || sem == "ROTATION" || sem == "SCALE";
                let is_translation = sem == "TRANSLATION";

                if has_rotation && is_transform {
                    assert!(attribute.typed_array.is_some());
                    assert!(attribute.buffer.is_none());
                } else if is_translation {
                    assert!(attribute.typed_array.is_some());
                    assert!(attribute.buffer.is_some());
                    buffer_count += 1;
                } else {
                    assert!(attribute.typed_array.is_none());
                    assert!(attribute.buffer.is_some());
                    buffer_count += 1;
                }
            }
        }
    }

    assert_eq!(loader.buffers().len(), buffer_count);
}

fn generate_instanced_tile_buffer(version: u32, gltf_format: u32) -> Vec<u8> {
    let mut buf = Vec::new();
    buf.extend_from_slice(b"i3dm");
    buf.extend_from_slice(&version.to_le_bytes());

    let ft_json = serde_json::json!({
        "INSTANCES_LENGTH": 1,
        "POSITION": [0.0, 0.0, 0.0]
    });
    let ft_bytes = serde_json::to_vec(&ft_json).unwrap();
    let ft_len = ft_bytes.len() as u32;

    let total_len = 32 + ft_len + 8;
    buf.extend_from_slice(&total_len.to_le_bytes());
    buf.extend_from_slice(&ft_len.to_le_bytes()); // featureTableJsonByteLength
    buf.extend_from_slice(&0u32.to_le_bytes()); // featureTableBinaryByteLength
    buf.extend_from_slice(&0u32.to_le_bytes()); // batchTableJsonByteLength
    buf.extend_from_slice(&0u32.to_le_bytes()); // batchTableBinaryByteLength
    buf.extend_from_slice(&gltf_format.to_le_bytes()); // gltfFormat

    buf.extend_from_slice(&ft_bytes);
    buf.extend_from_slice(b"fakegltf");

    buf
}

#[test]
fn test_releases_array_buffer_when_finished_loading() {
    pollster::block_on(async {
        let loader = load_i3dm("InstancedWithBatchTable/instancedWithBatchTable.i3dm").await;
        assert!(loader.components().is_some());
        assert!(!loader.has_array_buffer());
    });
}

#[test]
fn test_loads_instanced_gltf_external_url() {
    pollster::block_on(async {
        let loader = load_i3dm("InstancedGltfExternal/instancedGltfExternal.i3dm").await;
        verify_instances(
            &loader,
            &["TRANSLATION", "ROTATION", "_FEATURE_ID"],
            25,
        );
    });
}

#[test]
fn test_loads_instanced_with_batch_table_url() {
    pollster::block_on(async {
        let loader = load_i3dm("InstancedWithBatchTable/instancedWithBatchTable.i3dm").await;
        verify_instances(
            &loader,
            &["TRANSLATION", "ROTATION", "_FEATURE_ID"],
            25,
        );
    });
}

#[test]
fn test_loads_instanced_with_batch_table_binary_url() {
    pollster::block_on(async {
        let loader = load_i3dm("InstancedWithBatchTableBinary/instancedWithBatchTableBinary.i3dm").await;
        verify_instances(
            &loader,
            &["TRANSLATION", "ROTATION", "_FEATURE_ID"],
            25,
        );
    });
}

#[test]
fn test_loads_instanced_without_batch_table_url() {
    pollster::block_on(async {
        let loader = load_i3dm("InstancedWithoutBatchTable/instancedWithoutBatchTable.i3dm").await;
        verify_instances(
            &loader,
            &["TRANSLATION", "ROTATION", "_FEATURE_ID"],
            25,
        );
    });
}

#[test]
fn test_loads_instanced_orientation_url() {
    pollster::block_on(async {
        let loader = load_i3dm("InstancedOrientation/instancedOrientation.i3dm").await;
        verify_instances(
            &loader,
            &["TRANSLATION", "ROTATION", "_FEATURE_ID"],
            25,
        );
    });
}

#[test]
fn test_loads_instanced_oct32p_orientation_url() {
    pollster::block_on(async {
        let loader = load_i3dm("InstancedOct32POrientation/instancedOct32POrientation.i3dm").await;
        verify_instances(
            &loader,
            &["TRANSLATION", "ROTATION", "_FEATURE_ID"],
            25,
        );
    });
}

#[test]
fn test_loads_instanced_scale_url() {
    pollster::block_on(async {
        let loader = load_i3dm("InstancedScale/instancedScale.i3dm").await;
        verify_instances(
            &loader,
            &["TRANSLATION", "ROTATION", "SCALE", "_FEATURE_ID"],
            25,
        );
    });
}

#[test]
fn test_loads_instanced_scale_non_uniform_url() {
    pollster::block_on(async {
        let loader = load_i3dm("InstancedScaleNonUniform/instancedScaleNonUniform.i3dm").await;
        verify_instances(
            &loader,
            &["TRANSLATION", "ROTATION", "SCALE", "_FEATURE_ID"],
            25,
        );
    });
}

#[test]
fn test_loads_instanced_rtc_url() {
    pollster::block_on(async {
        let loader = load_i3dm("InstancedRTC/instancedRTC.i3dm").await;
        verify_instances(
            &loader,
            &["TRANSLATION", "ROTATION", "_FEATURE_ID"],
            25,
        );

        let transform = loader.components().unwrap().transform;
        let expected = Matrix4::new(
            1.0, 0.0, 0.0, 1215013.8340490046,
            0.0, 1.0, 0.0, -4736316.75897742,
            0.0, 0.0, 1.0, 4081608.4380407534,
            0.0, 0.0, 0.0, 1.0,
        );
        assert!(transform.equals_epsilon(&expected, EPSILON8));
    });
}

#[test]
fn test_loads_instanced_zero_rtc_url() {
    pollster::block_on(async {
        let loader = load_i3dm("InstancedZeroRTC/instancedZeroRTC.i3dm").await;
        verify_instances(
            &loader,
            &["TRANSLATION", "ROTATION", "_FEATURE_ID"],
            25,
        );

        let transform = loader.components().unwrap().transform;
        let expected = Matrix4::new(
            1.0, 0.0, 0.0, 1215013.8125,
            0.0, 1.0, 0.0, -4736316.75,
            0.0, 0.0, 1.0, 4081608.5,
            0.0, 0.0, 0.0, 1.0,
        );
        assert!(transform.equals_epsilon(&expected, EPSILON8));
    });
}

#[test]
fn test_loads_instanced_quantized_url() {
    pollster::block_on(async {
        let loader = load_i3dm("InstancedQuantized/instancedQuantized.i3dm").await;
        verify_instances(
            &loader,
            &["TRANSLATION", "ROTATION", "_FEATURE_ID"],
            25,
        );

        let transform = loader.components().unwrap().transform;
        let expected = Matrix4::new(
            1.0, 0.0, 0.0, 1215013.8125,
            0.0, 1.0, 0.0, -4736316.75,
            0.0, 0.0, 1.0, 4081608.5,
            0.0, 0.0, 0.0, 1.0,
        );
        assert!(transform.equals_epsilon(&expected, EPSILON8));
    });
}

#[test]
fn test_loads_instanced_quantized_oct32p_orientation_url() {
    pollster::block_on(async {
        let loader = load_i3dm("InstancedQuantizedOct32POrientation/instancedQuantizedOct32POrientation.i3dm").await;
        verify_instances(
            &loader,
            &["TRANSLATION", "ROTATION", "_FEATURE_ID"],
            25,
        );
    });
}

#[test]
fn test_loads_instanced_with_transform_url() {
    pollster::block_on(async {
        let loader = load_i3dm("InstancedWithTransform/instancedWithTransform.i3dm").await;
        verify_instances(
            &loader,
            &["TRANSLATION", "_FEATURE_ID"],
            25,
        );
    });
}

#[test]
fn test_loads_instanced_with_batch_ids_url() {
    pollster::block_on(async {
        let loader = load_i3dm("InstancedWithBatchIds/instancedWithBatchIds.i3dm").await;
        verify_instances(
            &loader,
            &["TRANSLATION", "ROTATION", "_FEATURE_ID"],
            25,
        );
    });
}

#[test]
fn test_loads_instanced_textured_url() {
    pollster::block_on(async {
        let loader = load_i3dm("InstancedTextured/instancedTextured.i3dm").await;
        verify_instances(
            &loader,
            &["TRANSLATION", "ROTATION", "_FEATURE_ID"],
            25,
        );
    });
}

#[test]
fn test_destroys_buffers_when_unloaded() {
    pollster::block_on(async {
        let mut loader = load_i3dm("InstancedGltfExternal/instancedGltfExternal.i3dm").await;
        let buffers = loader.buffers();
        assert_eq!(buffers.len(), 1);

        let buffer = buffers[0].clone();
        loader.destroy();
        assert!(buffer.is_destroyed());
    });
}

#[test]
fn test_throws_with_invalid_format() {
    pollster::block_on(async {
        let bytes = generate_instanced_tile_buffer(1, 2);
        let mut loader = I3dmLoader::new(I3dmLoaderOptions {
            i3dm_resource: Some("http://example.com/test.i3dm".to_string()),
            array_buffer: Some(bytes),
            ..Default::default()
        })
        .unwrap();

        let res = loader.load().await;
        assert!(res.is_err());
    });
}

#[test]
fn test_throws_with_invalid_version() {
    pollster::block_on(async {
        let bytes = generate_instanced_tile_buffer(2, 1);
        let mut loader = I3dmLoader::new(I3dmLoaderOptions {
            i3dm_resource: Some("http://example.com/test.i3dm".to_string()),
            array_buffer: Some(bytes),
            ..Default::default()
        })
        .unwrap();

        let res = loader.load().await;
        assert!(res.is_err());
    });
}
