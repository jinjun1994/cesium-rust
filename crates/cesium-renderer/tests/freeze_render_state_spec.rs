//! Direct port and verification against CesiumJS `freezeRenderStateSpec.js` (Apache-2.0).

use std::sync::Arc;

use cesium_renderer::{freeze_render_state, RenderState};

#[test]
fn test_freeze_render_state_works_as_expected() {
    let fresh = RenderState::default();
    let frozen = freeze_render_state(Some(fresh));
    assert_eq!(*frozen, RenderState::default());

    // Immutable Arc instance matches caching deduplication
    let frozen2 = freeze_render_state(None);
    assert!(Arc::ptr_eq(&frozen, &frozen2));
}
