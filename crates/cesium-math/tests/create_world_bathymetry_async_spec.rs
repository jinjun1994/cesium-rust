use cesium_math::{
    create_world_bathymetry_async, CesiumTerrainFormat, CesiumTerrainProvider,
    CesiumTerrainTilingScheme, CreateWorldBathymetryOptions, Ellipsoid,
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

#[test]
fn test_resolves_to_cesium_terrain_provider_instance_with_default_parameters() {
    let provider = block_on(create_world_bathymetry_async(None)).unwrap();

    assert!(!provider.request_vertex_normals());
    assert_eq!(provider.format(), CesiumTerrainFormat::QuantizedMesh);
    assert!(matches!(
        provider.tiling_scheme(),
        CesiumTerrainTilingScheme::Geographic(_)
    ));
    assert_eq!(
        provider.tiling_scheme().ellipsoid(),
        Ellipsoid::WGS84
    );
    assert!(provider
        .tile_url_templates()
        .iter()
        .any(|t| t.contains("/2426648/")));
}

#[test]
fn test_creates_world_bathymetry_with_vertex_normals() {
    let options = CreateWorldBathymetryOptions {
        request_vertex_normals: true,
    };

    let provider = block_on(create_world_bathymetry_async(Some(options))).unwrap();

    assert!(provider.request_vertex_normals());
    assert_eq!(provider.max_zoom(), Some(20));
}

#[test]
fn test_from_ion_asset_id_sync_equivalent() {
    let provider = CesiumTerrainProvider::from_ion_asset_id_sync(
        CesiumTerrainProvider::WORLD_BATHYMETRY_ASSET_ID,
        None,
    )
    .unwrap();

    assert!(!provider.request_vertex_normals());
    assert!(provider.has_vertex_normals());
}
