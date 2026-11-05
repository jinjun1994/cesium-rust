use cesium_math::{
    BoundingSphere, Cartesian3, Cesium3DTilesTerrainData, Cesium3DTilesTerrainDataOptions,
    GeographicTilingScheme, GeoRectangle, OrientedBoundingBox,
};

fn create_valid_options() -> Cesium3DTilesTerrainDataOptions {
    Cesium3DTilesTerrainDataOptions {
        positions: Some(vec![
            Cartesian3::new(0.0, 0.0, 0.0),
            Cartesian3::new(1.0, 0.0, 0.0),
            Cartesian3::new(0.0, 1.0, 0.0),
            Cartesian3::new(1.0, 1.0, 0.0),
        ]),
        indices: Some(vec![0, 1, 2, 2, 1, 3]),
        normals: None,
        minimum_height: Some(0.0),
        maximum_height: Some(100.0),
        bounding_sphere: Some(BoundingSphere::new(glam::DVec3::ZERO, 10.0)),
        oriented_bounding_box: Some(OrientedBoundingBox::default()),
        horizon_occlusion_point: Some(Cartesian3::ZERO),
        skirt_height: Some(4.0),
        child_tile_mask: Some(15),
        created_by_upsampling: Some(false),
        water_mask: None,
        credits: None,
        west_indices: Some(vec![0, 2]),
        south_indices: Some(vec![0, 1]),
        east_indices: Some(vec![1, 3]),
        north_indices: Some(vec![2, 3]),
    }
}

#[test]
fn test_requires_mandatory_fields() {
    let mut opts = create_valid_options();
    opts.minimum_height = None;
    assert!(Cesium3DTilesTerrainData::new(opts).is_err());

    let mut opts = create_valid_options();
    opts.maximum_height = None;
    assert!(Cesium3DTilesTerrainData::new(opts).is_err());

    let mut opts = create_valid_options();
    opts.bounding_sphere = None;
    assert!(Cesium3DTilesTerrainData::new(opts).is_err());

    let mut opts = create_valid_options();
    opts.oriented_bounding_box = None;
    assert!(Cesium3DTilesTerrainData::new(opts).is_err());

    let mut opts = create_valid_options();
    opts.horizon_occlusion_point = None;
    assert!(Cesium3DTilesTerrainData::new(opts).is_err());

    let mut opts = create_valid_options();
    opts.skirt_height = None;
    assert!(Cesium3DTilesTerrainData::new(opts).is_err());
}

#[test]
fn test_child_availability_mask_variants() {
    let mut opts = create_valid_options();
    opts.child_tile_mask = None; // Defaults to 15 (all 4 children)
    let data = Cesium3DTilesTerrainData::new(opts).unwrap();
    assert_eq!(data.child_tile_mask(), 15);
    assert!(data.is_child_available(10, 10, 20, 20).unwrap());
    assert!(data.is_child_available(10, 10, 21, 20).unwrap());
    assert!(data.is_child_available(10, 10, 20, 21).unwrap());
    assert!(data.is_child_available(10, 10, 21, 21).unwrap());

    // Only Southwest available (bit 0 = 1)
    let mut opts = create_valid_options();
    opts.child_tile_mask = Some(1);
    let data = Cesium3DTilesTerrainData::new(opts).unwrap();
    assert!(data.is_child_available(10, 10, 20, 20).unwrap());
    assert!(!data.is_child_available(10, 10, 21, 20).unwrap());
    assert!(!data.is_child_available(10, 10, 20, 21).unwrap());
    assert!(!data.is_child_available(10, 10, 21, 21).unwrap());

    // Only Southeast available (bit 1 = 2)
    let mut opts = create_valid_options();
    opts.child_tile_mask = Some(2);
    let data = Cesium3DTilesTerrainData::new(opts).unwrap();
    assert!(!data.is_child_available(10, 10, 20, 20).unwrap());
    assert!(data.is_child_available(10, 10, 21, 20).unwrap());

    // Only Northwest available (bit 2 = 4)
    let mut opts = create_valid_options();
    opts.child_tile_mask = Some(4);
    let data = Cesium3DTilesTerrainData::new(opts).unwrap();
    assert!(data.is_child_available(10, 10, 20, 21).unwrap());

    // Only Northeast available (bit 3 = 8)
    let mut opts = create_valid_options();
    opts.child_tile_mask = Some(8);
    let data = Cesium3DTilesTerrainData::new(opts).unwrap();
    assert!(data.is_child_available(10, 10, 21, 21).unwrap());

    // Invalid child coordinate diff
    assert!(data.is_child_available(10, 10, 25, 25).is_err());
}

#[test]
fn test_create_mesh_and_exaggeration() {
    let opts = create_valid_options();
    let mut data = Cesium3DTilesTerrainData::new(opts).unwrap();
    let tiling_scheme = GeographicTilingScheme::default();

    let mesh = data.create_mesh(&tiling_scheme, 0, 0, 0, Some(2.0)).unwrap();
    assert_eq!(mesh.minimum_height, 0.0);
    assert_eq!(mesh.maximum_height, 200.0); // 100 * 2.0
    assert_eq!(mesh.indices.len(), 6);
    assert_eq!(mesh.vertices.len(), 12);
}

#[test]
fn test_height_interpolation_and_clamping() {
    let opts = create_valid_options();
    let mut data = Cesium3DTilesTerrainData::new(opts).unwrap();
    let rect = GeoRectangle::degrees(0.0, 0.0, 1.0, 1.0).unwrap();

    // Before mesh creation, interpolate_height should fail
    assert!(data.interpolate_height(&rect, 0.5, 0.5).is_err());

    let tiling_scheme = GeographicTilingScheme::default();
    let _ = data.create_mesh(&tiling_scheme, 0, 0, 0, None).unwrap();

    // Center point (u=0.5, v=0.5) => (0 + 100) * 0.5 = 50.0
    let center_lon = (rect.west + rect.east) * 0.5;
    let center_lat = (rect.south + rect.north) * 0.5;
    let h_center = data.interpolate_height(&rect, center_lon, center_lat).unwrap();
    assert!((h_center - 50.0).abs() < 1e-4);

    // Outside coordinates clamp to edge
    let h_clamped = data.interpolate_height(&rect, 2.0, 2.0).unwrap();
    assert!((h_clamped - 100.0).abs() < 1e-4);
}

#[test]
fn test_upsample_tile() {
    let opts = create_valid_options();
    let mut data = Cesium3DTilesTerrainData::new(opts).unwrap();
    let tiling_scheme = GeographicTilingScheme::default();

    // Cannot upsample until mesh is created
    assert!(data.upsample(&tiling_scheme, 0, 0, 0, 0, 0, 1).is_err());

    let _ = data.create_mesh(&tiling_scheme, 0, 0, 0, None).unwrap();

    // Upsample across > 1 levels fails
    assert!(data.upsample(&tiling_scheme, 0, 0, 0, 0, 0, 2).is_err());

    // Single level upsampling succeeds
    let child = data.upsample(&tiling_scheme, 0, 0, 0, 0, 0, 1).unwrap();
    assert!(child.was_created_by_upsampling());
}
