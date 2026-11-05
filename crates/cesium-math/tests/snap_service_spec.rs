use cesium_math::SnapService;

struct DummySnapService;
impl SnapService for DummySnapService {}

#[test]
fn test_default_snap_aperture() {
    assert_eq!(DummySnapService::DEFAULT_SNAP_APERTURE, None);
}

#[test]
fn test_snap_is_not_implemented_on_base_trait() {
    let service = DummySnapService;
    assert!(service.snap().is_err());
}
