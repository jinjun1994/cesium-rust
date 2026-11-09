use cesium_renderer::DiscardEmptyTileImagePolicy;

#[test]
fn test_does_not_discard_non_empty_image() {
    let green_image: [u8; 4] = [0, 255, 0, 255];
    let policy = DiscardEmptyTileImagePolicy::new();

    assert!(policy.is_ready());
    assert!(!policy.should_discard_image(&green_image));
}

#[test]
fn test_discards_an_empty_image() {
    let empty_image = DiscardEmptyTileImagePolicy::EMPTY_IMAGE;
    let policy = DiscardEmptyTileImagePolicy::new();

    assert!(policy.is_ready());
    assert!(policy.should_discard_image(empty_image));
}
