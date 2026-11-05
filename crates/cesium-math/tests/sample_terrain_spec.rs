use cesium_math::{
    sample_terrain, Cartographic, CesiumTerrainLayerJson, CesiumTerrainProvider,
    CesiumTerrainProviderOptions, TileRange,
};
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll, RawWaker, RawWakerVTable, Waker};

fn block_on<F: Future>(fut: F) -> F::Output {
    let mut fut = Box::pin(fut);
    unsafe fn noop_clone(_: *const ()) -> RawWaker {
        RawWaker::new(std::ptr::null(), &VTABLE)
    }
    unsafe fn noop(_: *const ()) {}
    static VTABLE: RawWakerVTable = RawWakerVTable::new(noop_clone, noop, noop, noop);
    let raw_waker = RawWaker::new(std::ptr::null(), &VTABLE);
    let waker = unsafe { Waker::from_raw(raw_waker) };
    let mut cx = Context::from_waker(&waker);
    match Pin::as_mut(&mut fut).poll(&mut cx) {
        Poll::Ready(res) => res,
        Poll::Pending => panic!("Future did not complete immediately"),
    }
}

fn create_sample_terrain_provider() -> CesiumTerrainProvider {
    let mut layer_json = CesiumTerrainLayerJson {
        format: Some("quantized-mesh-1.0".to_string()),
        tiles: Some(vec!["https://terrain.invalid/tiles/{z}/{x}/{y}.terrain".to_string()]),
        projection: Some("EPSG:4326".to_string()),
        scheme: Some("tms".to_string()),
        maxzoom: Some(19),
        attribution: Some("Everest Sample Terrain".to_string()),
        extensions: Some(vec![
            "octvertexnormals".to_string(),
            "watermask".to_string(),
            "metadata".to_string(),
        ]),
        available: None,
        metadata_availability: None,
        parent_url: None,
        version: Some("1.2.0".to_string()),
    };

    // Level 9 availability: range containing 759, 335
    let mut available = vec![Vec::new(); 10];
    available[9] = vec![TileRange {
        start_x: 750,
        start_y: 330,
        end_x: 765,
        end_y: 340,
    }];
    layer_json.available = Some(available);

    let mut provider = CesiumTerrainProvider::from_layer_json(&layer_json, Some(CesiumTerrainProviderOptions::default())).unwrap();
    provider.inject_sample_everest_tile();
    provider
}

#[test]
fn test_queries_heights() {
    let provider = create_sample_terrain_provider();
    let mut positions = vec![
        Cartographic::from_degrees(86.93666235421982, 27.97989963555095, None),
        Cartographic::from_degrees(86.9366623542198, 27.9798996355509, None),
    ];

    block_on(sample_terrain(&provider, 9, &mut positions, false)).unwrap();

    assert!(positions[0].height > 5000.0);
    assert!(positions[0].height < 10000.0);
    assert!(positions[1].height > 5000.0);
    assert!(positions[1].height < 10000.0);
}

#[test]
fn test_sets_height_to_nan_if_terrain_data_not_available_and_reject_is_false() {
    let provider = create_sample_terrain_provider();
    let mut positions = vec![Cartographic::from_degrees(0.0, 0.0, None)];

    block_on(sample_terrain(&provider, 9, &mut positions, false)).unwrap();

    assert!(positions[0].height.is_nan());
    assert!(!positions[0].is_valid_height());
}

#[test]
fn test_rejects_if_terrain_data_not_available_and_reject_on_tile_fail_is_true() {
    let provider = create_sample_terrain_provider();
    let mut positions = vec![Cartographic::from_degrees(0.0, 0.0, None)];

    let result = block_on(sample_terrain(&provider, 9, &mut positions, true));
    assert!(result.is_err());
}

#[test]
fn test_rejects_if_second_position_unavailable_and_reject_is_true() {
    let provider = create_sample_terrain_provider();
    let mut positions = vec![
        Cartographic::from_degrees(86.93666235421982, 27.97989963555095, None),
        Cartographic::from_degrees(0.0, 0.0, None),
    ];

    let result = block_on(sample_terrain(&provider, 9, &mut positions, true));
    assert!(result.is_err());
}

#[test]
fn test_fills_in_what_it_can_when_given_a_mix_of_positions() {
    let provider = create_sample_terrain_provider();
    let mut positions = vec![
        Cartographic::from_degrees(86.93666235421982, 27.97989963555095, None),
        Cartographic::from_degrees(0.0, 89.0, None),
        Cartographic::from_degrees(86.9366623542198, 27.9798996355509, None),
    ];

    block_on(sample_terrain(&provider, 9, &mut positions, false)).unwrap();

    assert!(positions[0].height > 5000.0);
    assert!(positions[0].height < 10000.0);
    assert!(positions[1].height.is_nan());
    assert!(positions[2].height > 5000.0);
    assert!(positions[2].height < 10000.0);
}
