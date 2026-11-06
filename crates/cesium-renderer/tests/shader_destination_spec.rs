//! Direct port and verification against CesiumJS `ShaderDestinationSpec.js` (Apache-2.0).

use cesium_renderer::ShaderDestination;

#[test]
fn test_includes_vertex_shader_throws_for_undefined_destination() {
    assert!(ShaderDestination::includes_vertex_shader(None).is_err());
}

#[test]
fn test_includes_vertex_shader_works() {
    assert!(ShaderDestination::includes_vertex_shader(Some(ShaderDestination::VERTEX)).unwrap());
    assert!(!ShaderDestination::includes_vertex_shader(Some(ShaderDestination::FRAGMENT)).unwrap());
    assert!(ShaderDestination::includes_vertex_shader(Some(ShaderDestination::BOTH)).unwrap());
}

#[test]
fn test_includes_fragment_shader_throws_for_undefined_destination() {
    assert!(ShaderDestination::includes_fragment_shader(None).is_err());
}

#[test]
fn test_includes_fragment_shader_works() {
    assert!(!ShaderDestination::includes_fragment_shader(Some(ShaderDestination::VERTEX)).unwrap());
    assert!(ShaderDestination::includes_fragment_shader(Some(ShaderDestination::FRAGMENT)).unwrap());
    assert!(ShaderDestination::includes_fragment_shader(Some(ShaderDestination::BOTH)).unwrap());
}
