use cesium_math::{
    decode_google_earth_enterprise_data, GoogleEarthEnterpriseMetadata,
    GoogleEarthEnterpriseTileInformation,
};

#[test]
fn test_tile_xy_to_quad_key() {
    assert_eq!(GoogleEarthEnterpriseMetadata::tile_xy_to_quad_key(1, 0, 0), "2");
    assert_eq!(GoogleEarthEnterpriseMetadata::tile_xy_to_quad_key(1, 2, 1), "02");
    assert_eq!(GoogleEarthEnterpriseMetadata::tile_xy_to_quad_key(3, 5, 2), "021");
    assert_eq!(GoogleEarthEnterpriseMetadata::tile_xy_to_quad_key(4, 7, 2), "100");
}

#[test]
fn test_quad_key_to_tile_xy() {
    assert_eq!(
        GoogleEarthEnterpriseMetadata::quad_key_to_tile_xy("2").unwrap(),
        (1, 0, 0)
    );
    assert_eq!(
        GoogleEarthEnterpriseMetadata::quad_key_to_tile_xy("02").unwrap(),
        (1, 2, 1)
    );
    assert_eq!(
        GoogleEarthEnterpriseMetadata::quad_key_to_tile_xy("021").unwrap(),
        (3, 5, 2)
    );
    assert_eq!(
        GoogleEarthEnterpriseMetadata::quad_key_to_tile_xy("100").unwrap(),
        (4, 7, 2)
    );
}

#[test]
fn test_tile_xy_and_quad_key_roundtrip() {
    for level in 0..6 {
        let max_coord = 1 << level;
        for x in (0..max_coord).step_by(3) {
            for y in (0..max_coord).step_by(3) {
                let quad = GoogleEarthEnterpriseMetadata::tile_xy_to_quad_key(x, y, level);
                let (rx, ry, rlevel) =
                    GoogleEarthEnterpriseMetadata::quad_key_to_tile_xy(&quad).unwrap();
                assert_eq!(rx, x);
                assert_eq!(ry, y);
                assert_eq!(rlevel, level);
            }
        }
    }
}

#[test]
fn test_decode_google_earth_enterprise_data() {
    // Linear congruential generator for reproducible pseudo-random bytes
    let mut seed = 123123u64;
    let mut next_rand = || {
        seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        ((seed >> 33) & 0xFF) as u8
    };

    let mut key = vec![0u8; 1024];
    for b in &mut key {
        *b = next_rand();
    }

    let mut data = vec![0u8; 1025];
    for b in &mut data {
        *b = next_rand();
    }

    let original_data = data.clone();

    // First decode: data is scrambled (not equal to original)
    decode_google_earth_enterprise_data(&key, &mut data).unwrap();
    assert_ne!(data, original_data);

    // Second decode: symmetric decryption restores original data
    decode_google_earth_enterprise_data(&key, &mut data).unwrap();
    assert_eq!(data, original_data);
}

#[test]
fn test_decode_requires_key_length_multiple_of_4() {
    let mut data = vec![1, 2, 3];
    assert!(decode_google_earth_enterprise_data(&[], &mut data).is_err());
    assert!(decode_google_earth_enterprise_data(&[1], &mut data).is_err());
    assert!(decode_google_earth_enterprise_data(&[1, 2], &mut data).is_err());
    assert!(decode_google_earth_enterprise_data(&[1, 2, 3], &mut data).is_err());
    assert!(decode_google_earth_enterprise_data(&[1, 2, 3, 4], &mut data).is_ok());
}

#[test]
fn test_tile_information_queries() {
    let mut tile_info = GoogleEarthEnterpriseTileInformation::new(0xff, 1, 1, 1, 10, 20);
    assert!(tile_info.has_subtree());
    assert!(tile_info.has_imagery());
    assert!(tile_info.has_terrain());
    assert!(tile_info.has_children());
    assert!(tile_info.has_child(0));
    assert!(tile_info.has_child(1));
    assert!(tile_info.has_child(2));
    assert!(tile_info.has_child(3));
    assert_eq!(tile_info.get_child_bitmask(), 0x0f);
    assert_eq!(tile_info.imagery_provider, 10);
    assert_eq!(tile_info.terrain_provider, 20);

    // Parent relationship: parent has terrain, setting parent propagates ancestor_has_terrain
    let dummy_root = GoogleEarthEnterpriseTileInformation::new(0x00, 1, 1, 1, 0, 0);
    tile_info.set_parent(&dummy_root);
    assert!(tile_info.ancestor_has_terrain);

    let mut child = GoogleEarthEnterpriseTileInformation::new(0x00, 1, 1, 1, 0, 0);
    assert!(!child.ancestor_has_terrain);
    child.set_parent(&tile_info);
    assert!(child.ancestor_has_terrain);
}

#[test]
fn test_metadata_populate_subtree_and_defaults() {
    let mut metadata = GoogleEarthEnterpriseMetadata::from_url("http://test.server").unwrap();
    assert!(metadata.imagery_present);
    assert_eq!(metadata.proto_imagery, None);
    assert!(metadata.terrain_present);
    assert_eq!(metadata.negative_altitude_exponent_bias, 32);
    assert!((metadata.negative_altitude_threshold - 1e-12).abs() < 1e-18);

    let (x, y, level) = GoogleEarthEnterpriseMetadata::quad_key_to_tile_xy("0123").unwrap();
    metadata.populate_subtree(x, y, level);

    let tile_info = metadata.tile_info_map();
    assert!(tile_info.get("0").is_some());
    assert!(tile_info.get("01").is_some());
    assert!(tile_info.get("012").is_some());
    assert!(tile_info.get("0123").is_some());
    assert!(metadata.is_valid("0123"));
}
