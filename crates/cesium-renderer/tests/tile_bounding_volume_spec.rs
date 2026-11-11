use cesium_renderer::TileBoundingVolume;

#[test]
fn test_throws_developer_error_on_direct_instantiation_calls() {
    let bounding_volume = TileBoundingVolume::new();

    assert!(bounding_volume.create_debug_volume().is_err());
    assert!(bounding_volume.distance_to_camera().is_err());
    assert!(bounding_volume.intersect_plane().is_err());
}
