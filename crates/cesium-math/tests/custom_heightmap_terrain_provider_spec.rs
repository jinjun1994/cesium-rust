use std::sync::Arc;

use cesium_math::{
    CustomHeightmapTerrainProvider, CustomHeightmapTerrainProviderOptions, HeightmapBuffer,
    TerrainProvider, TilingScheme, WebMercatorTilingScheme,
};

#[test]
fn test_constructor_validation() {
    let err_cb = CustomHeightmapTerrainProvider::new(CustomHeightmapTerrainProviderOptions {
        width: Some(2),
        height: Some(2),
        ..Default::default()
    });
    assert!(err_cb.is_err());

    let cb = Arc::new(|_x, _y, _level| Some(HeightmapBuffer::Float32(vec![0.0; 4])));

    let err_w = CustomHeightmapTerrainProvider::new(CustomHeightmapTerrainProviderOptions {
        callback: Some(cb.clone()),
        height: Some(2),
        ..Default::default()
    });
    assert!(err_w.is_err());

    let err_h = CustomHeightmapTerrainProvider::new(CustomHeightmapTerrainProviderOptions {
        callback: Some(cb),
        width: Some(2),
        ..Default::default()
    });
    assert!(err_h.is_err());
}

#[test]
fn test_constructs_with_credit_and_dimensions() {
    let cb = Arc::new(|_x, _y, _level| Some(HeightmapBuffer::Float32(vec![0.0; 6])));
    let provider = CustomHeightmapTerrainProvider::new(CustomHeightmapTerrainProviderOptions {
        callback: Some(cb),
        width: Some(2),
        height: Some(3),
        credit: Some("My Credit".to_string()),
        ..Default::default()
    })
    .unwrap();

    assert_eq!(provider.credit(), Some("My Credit"));
    assert_eq!(provider.width(), 2);
    assert_eq!(provider.height(), 3);
    assert!(!provider.has_water_mask());
    assert!(!provider.has_vertex_normals());
    assert!(provider.get_tile_data_available(0, 0, 0).is_none());
}

#[test]
fn test_constructs_with_web_mercator_tiling_scheme() {
    let cb = Arc::new(|_x, _y, _level| Some(HeightmapBuffer::Float32(vec![0.0; 4])));
    let wm = WebMercatorTilingScheme::default();
    let provider = CustomHeightmapTerrainProvider::new_with_tiling_scheme(
        CustomHeightmapTerrainProviderOptions {
            callback: Some(cb),
            width: Some(2),
            height: Some(2),
            tiling_scheme: Some(wm),
            credit: None,
        },
    )
    .unwrap();

    let expected_err = TerrainProvider::get_estimated_level_zero_geometric_error_for_a_heightmap(
        &provider.tiling_scheme().ellipsoid(),
        2,
        provider.tiling_scheme().tile_counts(0).unwrap().0,
    );
    assert!((provider.get_level_maximum_geometric_error(0) - expected_err).abs() < 1e-10);
}

#[test]
fn test_request_tile_geometry_buffers_and_none() {
    // Float32
    let cb_f32 = Arc::new(|_x, _y, _level| Some(HeightmapBuffer::Float32(vec![1.0; 4])));
    let p_f32 = CustomHeightmapTerrainProvider::new(CustomHeightmapTerrainProviderOptions {
        callback: Some(cb_f32),
        width: Some(2),
        height: Some(2),
        ..Default::default()
    })
    .unwrap();
    let res_f32 = p_f32.request_tile_geometry(0, 0, 0);
    assert!(res_f32.is_some());
    assert_eq!(res_f32.unwrap().buffer().unwrap().to_vec_f64(), vec![1.0, 1.0, 1.0, 1.0]);

    // Uint8
    let cb_u8 = Arc::new(|_x, _y, _level| Some(HeightmapBuffer::Uint8(vec![10, 20, 30, 40])));
    let p_u8 = CustomHeightmapTerrainProvider::new(CustomHeightmapTerrainProviderOptions {
        callback: Some(cb_u8),
        width: Some(2),
        height: Some(2),
        ..Default::default()
    })
    .unwrap();
    let res_u8 = p_u8.request_tile_geometry(0, 0, 0);
    assert!(res_u8.is_some());
    assert_eq!(res_u8.unwrap().buffer().unwrap().to_vec_f64(), vec![10.0, 20.0, 30.0, 40.0]);

    // Callback returns None
    let cb_none = Arc::new(|_x, _y, _level| None);
    let p_none = CustomHeightmapTerrainProvider::new(CustomHeightmapTerrainProviderOptions {
        callback: Some(cb_none),
        width: Some(2),
        height: Some(2),
        ..Default::default()
    })
    .unwrap();
    assert!(p_none.request_tile_geometry(0, 0, 0).is_none());
}
