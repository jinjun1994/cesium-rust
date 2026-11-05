use cesium_math::{
    sample_terrain_most_detailed, Cartographic, CesiumTerrainLayerJson, CesiumTerrainProvider,
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

fn create_provider_without_availability() -> CesiumTerrainProvider {
    let layer_json = CesiumTerrainLayerJson {
        format: Some("quantized-mesh-1.0".to_string()),
        tiles: Some(vec!["https://terrain.invalid/tiles/{z}/{x}/{y}.terrain".to_string()]),
        projection: Some("EPSG:4326".to_string()),
        scheme: Some("tms".to_string()),
        maxzoom: Some(19),
        attribution: None,
        extensions: None,
        available: None,
        metadata_availability: None,
        parent_url: None,
        version: Some("1.2.0".to_string()),
    };
    CesiumTerrainProvider::from_layer_json(&layer_json, None).unwrap()
}

fn create_sample_provider_with_availability() -> CesiumTerrainProvider {
    let mut layer_json = CesiumTerrainLayerJson {
        format: Some("quantized-mesh-1.0".to_string()),
        tiles: Some(vec!["https://terrain.invalid/tiles/{z}/{x}/{y}.terrain".to_string()]),
        projection: Some("EPSG:4326".to_string()),
        scheme: Some("tms".to_string()),
        maxzoom: Some(19),
        attribution: Some("Everest Detailed Sample".to_string()),
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

    let mut available = vec![Vec::new(); 10];
    available[0] = vec![TileRange {
        start_x: 0,
        start_y: 0,
        end_x: 1,
        end_y: 0,
    }];
    available[9] = vec![TileRange {
        start_x: 750,
        start_y: 330,
        end_x: 765,
        end_y: 340,
    }];
    layer_json.available = Some(available);

    let mut provider = CesiumTerrainProvider::from_layer_json(
        &layer_json,
        Some(CesiumTerrainProviderOptions::default()),
    )
    .unwrap();
    provider.inject_sample_everest_tile();
    provider
}

#[test]
fn test_throws_if_terrain_provider_has_no_availability() {
    let provider = create_provider_without_availability();
    let mut positions = vec![Cartographic::from_degrees(86.9366, 27.9798, None)];

    let result = block_on(sample_terrain_most_detailed(&provider, &mut positions, false));
    assert!(result.is_err());
    let err_str = format!("{:?}", result.err().unwrap());
    assert!(err_str.contains("sampleTerrainMostDetailed requires a terrain provider that has tile availability"));
}

#[test]
fn test_queries_heights_at_maximum_available_level() {
    let provider = create_sample_provider_with_availability();
    let mut positions = vec![
        Cartographic::from_degrees(86.93666235421982, 27.97989963555095, None),
        Cartographic::from_degrees(86.9366623542198, 27.9798996355509, None),
    ];

    block_on(sample_terrain_most_detailed(&provider, &mut positions, false)).unwrap();

    assert!(positions[0].height > 5000.0);
    assert!(positions[0].height < 10000.0);
    assert!(positions[1].height > 5000.0);
    assert!(positions[1].height < 10000.0);
}
