//! Tests for ViewportQuad matching CesiumJS `Scene/ViewportQuadSpec.js` (Apache-2.0).

use cesium_math::BoundingRectangle;
use cesium_renderer::commands::{Pass, RenderStateHandle};
use cesium_renderer::material::Material;
use cesium_renderer::scene_primitive::{PrimitivePasses, PrimitiveRenderFrameState};
use cesium_renderer::viewport_quad::ViewportQuad;

#[test]
fn test_constructs_with_a_rectangle() {
    let rectangle = BoundingRectangle::new(1.0, 2.0, 3.0, 4.0);
    let quad = ViewportQuad::new(Some(rectangle), None);
    assert_eq!(quad.rectangle, Some(rectangle));
}

#[test]
fn test_constructs_with_a_material() {
    let material = Material::from_type(Material::STRIPE_TYPE).unwrap();
    let quad = ViewportQuad::new(None, Some(material.clone()));
    assert_eq!(quad.material.as_ref().unwrap().material_type, material.material_type);
}

#[test]
fn test_gets_the_default_color() {
    let quad = ViewportQuad::default();
    let mat = quad.material.as_ref().unwrap();
    let color_val = mat.uniforms.get("color").expect("color uniform should exist");
    assert_eq!(color_val, &serde_json::json!([1.0, 1.0, 1.0, 1.0]));
}

#[test]
fn test_throws_when_rendered_without_a_rectangle() {
    let mut quad = ViewportQuad::default();
    quad.rectangle = None;

    let mut frame_state = PrimitiveRenderFrameState::default();
    let err = quad.update(&mut frame_state).unwrap_err();
    assert!(err.message.contains("this.rectangle must be defined."));
}

#[test]
fn test_throws_when_rendered_without_a_material() {
    let mut quad = ViewportQuad::default();
    quad.material = None;

    let mut frame_state = PrimitiveRenderFrameState::default();
    let err = quad.update(&mut frame_state).unwrap_err();
    assert!(err.message.contains("this.material must be defined."));
}

#[test]
fn test_does_not_render_when_show_is_false() {
    let mut quad = ViewportQuad::default();
    quad.show = false;

    let mut frame_state = PrimitiveRenderFrameState::default();
    quad.update(&mut frame_state).unwrap();
    assert!(frame_state.command_list.is_empty());
}

#[test]
fn test_renders_material() {
    let mut quad = ViewportQuad::new(
        Some(BoundingRectangle::new(0.0, 0.0, 2.0, 2.0)),
        None,
    );

    let mut frame_state = PrimitiveRenderFrameState {
        passes: PrimitivePasses {
            render: true,
            pick: false,
        },
        ..Default::default()
    };

    quad.update(&mut frame_state).unwrap();
    assert_eq!(frame_state.command_list.len(), 1);
    let cmd = &frame_state.command_list[0];
    assert_eq!(cmd.pass, Some(Pass::Overlay));
    assert_eq!(cmd.owner.as_deref(), Some("ViewportQuad"));
}

#[test]
fn test_renders_user_created_texture_or_custom_uniforms() {
    let mut quad = ViewportQuad::default();
    let mut material = Material::from_type(Material::IMAGE_TYPE).unwrap();
    material.uniforms.insert("image".to_string(), serde_json::json!("Red16x16.png"));
    quad.material = Some(material);

    let mut frame_state = PrimitiveRenderFrameState::default();
    quad.update(&mut frame_state).unwrap();
    assert_eq!(frame_state.command_list.len(), 1);
}

#[test]
fn test_updates_rectangle() {
    let mut quad = ViewportQuad::new(
        Some(BoundingRectangle::new(0.0, 0.0, 2.0, 2.0)),
        None,
    );

    let mut frame_state = PrimitiveRenderFrameState::default();
    quad.update(&mut frame_state).unwrap();

    let other_rectangle = BoundingRectangle::new(0.0, 0.0, 4.0, 4.0);
    quad.rectangle = Some(other_rectangle);

    frame_state.command_list.clear();
    quad.update(&mut frame_state).unwrap();
    assert_eq!(frame_state.command_list.len(), 1);

    let cmd = &frame_state.command_list[0];
    if let Some(RenderStateHandle::State(ref rs)) = cmd.render_state {
        assert_eq!(rs.viewport, Some(other_rectangle));
    } else {
        panic!("expected RenderStateHandle::State");
    }
}

#[test]
fn test_is_destroyed_and_destroy() {
    let bound_rectangle = BoundingRectangle::new(0.0, 0.0, 10.0, 10.0);
    let mut quad = ViewportQuad::new(Some(bound_rectangle), None);

    assert!(!quad.is_destroyed());
    quad.destroy();
    assert!(quad.is_destroyed());

    let mut frame_state = PrimitiveRenderFrameState::default();
    assert!(quad.update(&mut frame_state).is_err());
}
