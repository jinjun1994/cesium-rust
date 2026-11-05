use cesium_math::{
    create_world_terrain_async, CesiumTerrainFormat, CesiumTerrainProvider,
    CesiumTerrainTilingScheme, CreateWorldTerrainOptions, Ellipsoid,
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
    let provider = block_on(create_world_terrain_async(None)).unwrap();

    assert!(!provider.request_vertex_normals());
    assert!(!provider.request_water_mask());
    assert_eq!(provider.format(), CesiumTerrainFormat::QuantizedMesh);
    assert!(matches!(
        provider.tiling_scheme(),
        CesiumTerrainTilingScheme::Geographic(_)
    ));
    assert_eq!(
        provider.tiling_scheme().ellipsoid(),
        Ellipsoid::WGS84
    );
    assert!(provider.tile_url_templates().iter().any(|t| t.contains("/1/")));
}

#[test]
fn test_creates_world_terrain_with_custom_flags_and_ellipsoid() {
    let custom_ellipsoid = Ellipsoid::new(glam::DVec3::splat(6378137.0)).unwrap();
    let options = CreateWorldTerrainOptions {
        request_vertex_normals: true,
        request_water_mask: true,
        ellipsoid: Some(custom_ellipsoid),
    };

    let provider = block_on(create_world_terrain_async(Some(options))).unwrap();

    assert!(provider.request_vertex_normals());
    assert!(provider.request_water_mask());
    assert_eq!(
        provider.tiling_scheme().ellipsoid(),
        custom_ellipsoid
    );
    assert_eq!(provider.max_zoom(), Some(20));
}

#[test]
fn test_from_ion_asset_id_sync_equivalent() {
    let provider = CesiumTerrainProvider::from_ion_asset_id_sync(
        CesiumTerrainProvider::WORLD_TERRAIN_ASSET_ID,
        None,
    )
    .unwrap();

    assert!(!provider.request_vertex_normals());
    assert!(!provider.request_water_mask());
    assert!(provider.has_vertex_normals());
    assert!(provider.has_water_mask());
}
