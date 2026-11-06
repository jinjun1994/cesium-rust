//! Direct port and verification against CesiumJS `PassStateSpec.js` (Apache-2.0).

use cesium_renderer::PassState;

#[test]
fn test_creates_a_pass_state() {
    let context = "context_mock";
    let pass_state = PassState::new(context);

    assert_eq!(pass_state.context.as_deref(), Some(context));
    assert!(pass_state.framebuffer.is_none());
    assert!(pass_state.blending_enabled.is_none());
    assert!(pass_state.scissor_test.is_none());
    assert!(pass_state.viewport.is_none());
}
