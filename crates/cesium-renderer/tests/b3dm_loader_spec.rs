use cesium_renderer::{
    B3dmLoader, B3dmLoaderOptions, B3dmParser, ResourceLoaderState,
};

const TRIANGLE_B3DM: &[u8] = include_bytes!("../../../tests/fixtures/triangle.b3dm");
const BATCH_TABLE_B3DM: &[u8] =
    include_bytes!("../../../tests/fixtures/cesium-batch-table-binary.b3dm");
const DEPRECATED_20_B3DM: &[u8] =
    include_bytes!("../../../tests/fixtures/cesium-batched-deprecated-20.b3dm");

#[test]
fn test_b3dm_parser_valid_and_invalid() {
    let parsed = B3dmParser::parse(TRIANGLE_B3DM, 0).expect("Triangle B3DM should parse");
    assert_eq!(parsed.version, 1);
    assert!(!parsed.glb.is_empty());

    // Invalid magic
    let mut bad_magic = TRIANGLE_B3DM.to_vec();
    bad_magic[0..4].copy_from_slice(b"xxxx");
    assert!(B3dmParser::parse(&bad_magic, 0).is_err());

    // Invalid version
    let mut bad_version = TRIANGLE_B3DM.to_vec();
    bad_version[4..8].copy_from_slice(&2u32.to_le_bytes());
    assert!(B3dmParser::parse(&bad_version, 0).is_err());
}

#[test]
fn test_releases_array_buffer_when_finished_loading() {
    pollster::block_on(async {
        let options = B3dmLoaderOptions {
            b3dm_resource: Some("http://example.com/triangle.b3dm".to_string()),
            base_resource: Some("http://example.com/triangle.b3dm".to_string()),
            array_buffer: Some(TRIANGLE_B3DM.to_vec()),
            byte_offset: Some(0),
            ..Default::default()
        };

        let loader = B3dmLoader::new(options).unwrap();
        assert!(loader.has_array_buffer());

        loader.load().await.expect("Load should succeed");
        assert_eq!(loader.state(), ResourceLoaderState::Processing);

        let processed = loader.process().expect("Process should succeed");
        assert!(processed);
        assert_eq!(loader.state(), ResourceLoaderState::Ready);

        // Components should be present, array buffer released
        assert!(loader.components().is_some());
        assert!(!loader.has_array_buffer());
    });
}

#[test]
fn test_loads_batched_with_batch_table() {
    pollster::block_on(async {
        let options = B3dmLoaderOptions {
            b3dm_resource: Some("http://example.com/batched.b3dm".to_string()),
            base_resource: Some("http://example.com/batched.b3dm".to_string()),
            array_buffer: Some(BATCH_TABLE_B3DM.to_vec()),
            byte_offset: Some(0),
            ..Default::default()
        };

        let loader = B3dmLoader::new(options).unwrap();
        loader.load().await.expect("Load should succeed");
        loader.process().expect("Process should succeed");

        let components = loader.components().expect("Components should exist");
        assert!(components.structural_metadata.is_some());
    });
}

#[test]
fn test_loads_legacy_deprecated_headers() {
    pollster::block_on(async {
        let options = B3dmLoaderOptions {
            b3dm_resource: Some("http://example.com/deprecated20.b3dm".to_string()),
            base_resource: Some("http://example.com/deprecated20.b3dm".to_string()),
            array_buffer: Some(DEPRECATED_20_B3DM.to_vec()),
            byte_offset: Some(0),
            ..Default::default()
        };

        let loader = B3dmLoader::new(options).unwrap();
        loader.load().await.expect("Load should succeed");
        loader.process().expect("Process should succeed");

        assert_eq!(loader.state(), ResourceLoaderState::Ready);
        assert!(loader.components().is_some());
    });
}

#[test]
fn test_constructor_throws_without_resource_or_buffer() {
    assert!(B3dmLoader::new(B3dmLoaderOptions {
        b3dm_resource: None,
        array_buffer: Some(vec![0u8; 12]),
        ..Default::default()
    })
    .is_err());

    assert!(B3dmLoader::new(B3dmLoaderOptions {
        b3dm_resource: Some("http://example.com".to_string()),
        array_buffer: None,
        ..Default::default()
    })
    .is_err());
}
