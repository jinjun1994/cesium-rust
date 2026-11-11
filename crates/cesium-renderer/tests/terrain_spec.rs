use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use cesium_math::EllipsoidTerrainProvider;
use cesium_renderer::terrain::Terrain;

#[test]
fn test_constructor_throws_without_terrain_promise() {
    let terrain = Terrain::<EllipsoidTerrainProvider>::new(None);
    assert!(terrain.is_err());
    assert!(terrain
        .unwrap_err()
        .message
        .contains("terrainProviderPromise is required."));
}

#[test]
fn test_does_not_throw_on_error() {
    let terrain = Terrain::<EllipsoidTerrainProvider>::new(Some(Err("Fail".to_string())));
    assert!(terrain.is_ok());
    let t = terrain.unwrap();
    assert!(!t.ready());
    assert_eq!(t.error(), Some("Fail"));
}

#[test]
fn test_ready_event_handles_terrain_promise_success() {
    let mut terrain = Terrain::<EllipsoidTerrainProvider>::deferred();
    let provider = EllipsoidTerrainProvider::default();

    let was_called = Arc::new(AtomicBool::new(false));
    let was_called_clone = was_called.clone();

    terrain.ready_event.add_listener(move |_p| {
        was_called_clone.store(true, Ordering::SeqCst);
    });

    terrain.resolve(provider);

    assert!(was_called.load(Ordering::SeqCst));
    assert!(terrain.ready());
    assert!(terrain.provider().is_some());
}

#[test]
fn test_error_event_handles_terrain_promise_failure() {
    let mut terrain = Terrain::<EllipsoidTerrainProvider>::deferred();

    let was_called = Arc::new(AtomicBool::new(false));
    let was_called_clone = was_called.clone();

    terrain.error_event.add_listener(move |err| {
        assert_eq!(err, "Fail");
        was_called_clone.store(true, Ordering::SeqCst);
    });

    terrain.reject("Fail");

    assert!(was_called.load(Ordering::SeqCst));
    assert!(!terrain.ready());
    assert_eq!(terrain.error(), Some("Fail"));
}

#[test]
fn test_from_world_terrain_and_bathymetry_factories() {
    let terrain = Terrain::from_world_terrain();
    assert!(terrain.ready());
    assert!(terrain.provider().is_some());

    let bathymetry = Terrain::from_world_bathymetry();
    assert!(bathymetry.ready());
    assert!(bathymetry.provider().is_some());
}
