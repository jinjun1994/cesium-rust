use cesium_renderer::{FrustumCommands, NUMBER_OF_PASSES};

#[test]
fn test_constructs_without_arguments() {
    let frustum = FrustumCommands::default();
    assert_eq!(frustum.near, 0.0);
    assert_eq!(frustum.far, 0.0);
    assert_eq!(frustum.commands.len(), NUMBER_OF_PASSES);
    assert_eq!(frustum.indices.len(), NUMBER_OF_PASSES);
    for idx in &frustum.indices {
        assert_eq!(*idx, 0);
    }
}

#[test]
fn test_constructs_with_arguments() {
    let near = 1.0;
    let far = 2.0;
    let frustum = FrustumCommands::new(Some(near), Some(far));
    assert_eq!(frustum.near, near);
    assert_eq!(frustum.far, far);
    assert_eq!(frustum.commands.len(), NUMBER_OF_PASSES);
    assert_eq!(frustum.indices.len(), NUMBER_OF_PASSES);
    for idx in &frustum.indices {
        assert_eq!(*idx, 0);
    }
}
