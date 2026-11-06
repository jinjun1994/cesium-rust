//! Direct port and verification against CesiumJS `ClearCommandSpec.js` (Apache-2.0).

use cesium_math::Color;
use cesium_renderer::{ClearCommand, ClearCommandOptions};

#[test]
fn test_constructs_with_defaults() {
    let c = ClearCommand::default();
    assert!(c.color.is_none());
    assert!(c.depth.is_none());
    assert!(c.stencil.is_none());
    assert!(c.render_state.is_none());
    assert!(c.framebuffer.is_none());
}

#[test]
fn test_constructs_with_options() {
    let render_state = "render_state_mock".to_string();
    let framebuffer = "framebuffer_mock".to_string();
    let c = ClearCommand::new(ClearCommandOptions {
        color: Some(Color::new(1.0, 2.0, 3.0, 4.0)),
        depth: Some(1.0),
        stencil: Some(2),
        render_state: Some(render_state.clone()),
        framebuffer: Some(framebuffer.clone()),
        ..Default::default()
    });

    assert_eq!(c.color, Some(Color::new(1.0, 2.0, 3.0, 4.0)));
    assert_eq!(c.depth, Some(1.0));
    assert_eq!(c.stencil, Some(2));
    assert_eq!(c.render_state, Some(render_state));
    assert_eq!(c.framebuffer, Some(framebuffer));
}

#[test]
fn test_clear_command_all_to_have_defaults() {
    assert_eq!(ClearCommand::ALL.color, Some(Color::new(0.0, 0.0, 0.0, 0.0)));
    assert_eq!(ClearCommand::ALL.depth, Some(1.0));
    assert_eq!(ClearCommand::ALL.stencil, Some(0));
    assert!(ClearCommand::ALL.render_state.is_none());
    assert!(ClearCommand::ALL.framebuffer.is_none());
}
