//! Direct port and verification against CesiumJS `DrawCommandSpec.js` (Apache-2.0).

use std::collections::HashMap;

use cesium_math::{BoundingSphere, OrientedBoundingBox, PrimitiveType};
use cesium_renderer::{DrawCommand, DrawCommandOptions, Pass};
use glam::DMat4;

#[test]
fn test_constructs() {
    let c = DrawCommand::default();
    assert!(c.bounding_volume.is_none());
    assert!(c.oriented_bounding_box.is_none());
    assert!(c.cull);
    assert!(c.occlude);
    assert!(c.model_matrix.is_none());
    assert_eq!(c.primitive_type, PrimitiveType::Triangles);
    assert!(c.vertex_array.is_none());
    assert!(c.count.is_none());
    assert_eq!(c.offset, 0);
    assert_eq!(c.instance_count, 0);
    assert!(c.shader_program.is_none());
    assert!(c.uniform_map.is_none());
    assert!(c.render_state.is_none());
    assert!(c.framebuffer.is_none());
    assert!(c.pass.is_none());
    assert!(!c.execute_in_closest_frustum);
    assert!(c.owner.is_none());
    assert!(!c.debug_show_bounding_volume);
    assert_eq!(c.debug_overlapping_frustums, 0);
    assert!(!c.cast_shadows);
    assert!(!c.receive_shadows);
    assert!(c.pick_id.is_none());
    assert!(!c.pick_only);
}

#[test]
fn test_constructs_with_options() {
    let bounding_volume = BoundingSphere::default();
    let oriented_bounding_box = OrientedBoundingBox::default();
    let model_matrix = DMat4::IDENTITY;
    let primitive_type = PrimitiveType::TriangleFan;
    let vertex_array = "va_mock".to_string();
    let shader_program = "shader_mock".to_string();
    let uniform_map = HashMap::from([("u_test".to_string(), "1".to_string())]);
    let render_state = "rs_mock".to_string();
    let framebuffer = "fb_mock".to_string();
    let pass = Pass::Translucent;
    let owner = "owner_mock".to_string();
    let pick_id = "pick_1".to_string();

    let c = DrawCommand::new(DrawCommandOptions {
        bounding_volume: Some(bounding_volume),
        oriented_bounding_box: Some(oriented_bounding_box),
        cull: Some(false),
        occlude: Some(false),
        model_matrix: Some(model_matrix),
        primitive_type: Some(primitive_type),
        vertex_array: Some(vertex_array.clone()),
        count: Some(3),
        offset: Some(3),
        instance_count: Some(2),
        shader_program: Some(shader_program.clone()),
        uniform_map: Some(uniform_map.clone()),
        render_state: Some(render_state.clone()),
        framebuffer: Some(framebuffer.clone()),
        pass: Some(pass),
        execute_in_closest_frustum: Some(true),
        owner: Some(owner.clone()),
        debug_show_bounding_volume: Some(true),
        cast_shadows: Some(true),
        receive_shadows: Some(true),
        pick_id: Some(pick_id.clone()),
        pick_only: Some(true),
        ..Default::default()
    });

    assert_eq!(c.bounding_volume, Some(bounding_volume));
    assert_eq!(c.oriented_bounding_box, Some(oriented_bounding_box));
    assert!(!c.cull);
    assert!(!c.occlude);
    assert_eq!(c.model_matrix, Some(model_matrix));
    assert_eq!(c.primitive_type, primitive_type);
    assert_eq!(c.vertex_array, Some(vertex_array));
    assert_eq!(c.count, Some(3));
    assert_eq!(c.offset, 3);
    assert_eq!(c.instance_count, 2);
    assert_eq!(c.shader_program, Some(shader_program));
    assert_eq!(c.uniform_map, Some(uniform_map));
    assert_eq!(c.render_state, Some(render_state));
    assert_eq!(c.framebuffer, Some(framebuffer));
    assert_eq!(c.pass, Some(pass));
    assert!(c.execute_in_closest_frustum);
    assert_eq!(c.owner, Some(owner));
    assert!(c.debug_show_bounding_volume);
    assert_eq!(c.debug_overlapping_frustums, 0);
    assert!(c.cast_shadows);
    assert!(c.receive_shadows);
    assert_eq!(c.pick_id, Some(pick_id));
    assert!(c.pick_only);
}

#[test]
fn test_shallow_clones() {
    let c = DrawCommand::new(DrawCommandOptions {
        bounding_volume: Some(BoundingSphere::default()),
        oriented_bounding_box: Some(OrientedBoundingBox::default()),
        cull: Some(false),
        occlude: Some(false),
        model_matrix: Some(DMat4::IDENTITY),
        primitive_type: Some(PrimitiveType::TriangleFan),
        vertex_array: Some("va".to_string()),
        count: Some(3),
        offset: Some(3),
        instance_count: Some(2),
        shader_program: Some("sh".to_string()),
        uniform_map: Some(HashMap::new()),
        render_state: Some("rs".to_string()),
        framebuffer: Some("fb".to_string()),
        pass: Some(Pass::Translucent),
        execute_in_closest_frustum: Some(true),
        owner: Some("ow".to_string()),
        debug_show_bounding_volume: Some(true),
        cast_shadows: Some(true),
        receive_shadows: Some(true),
        pick_id: Some("pk".to_string()),
        pick_only: Some(true),
        ..Default::default()
    });

    let clone = DrawCommand::shallow_clone(Some(&c), None).unwrap();

    assert_eq!(clone.bounding_volume, c.bounding_volume);
    assert_eq!(clone.oriented_bounding_box, c.oriented_bounding_box);
    assert_eq!(clone.cull, c.cull);
    assert_eq!(clone.occlude, c.occlude);
    assert_eq!(clone.model_matrix, c.model_matrix);
    assert_eq!(clone.primitive_type, c.primitive_type);
    assert_eq!(clone.vertex_array, c.vertex_array);
    assert_eq!(clone.count, c.count);
    assert_eq!(clone.offset, c.offset);
    assert_eq!(clone.instance_count, c.instance_count);
    assert_eq!(clone.shader_program, c.shader_program);
    assert_eq!(clone.uniform_map, c.uniform_map);
    assert_eq!(clone.render_state, c.render_state);
    assert_eq!(clone.framebuffer, c.framebuffer);
    assert_eq!(clone.pass, c.pass);
    assert_eq!(
        clone.execute_in_closest_frustum,
        c.execute_in_closest_frustum
    );
    assert_eq!(clone.owner, c.owner);
    assert_eq!(
        clone.debug_show_bounding_volume,
        c.debug_show_bounding_volume
    );
    assert_eq!(
        clone.debug_overlapping_frustums,
        c.debug_overlapping_frustums
    );
    assert_eq!(clone.cast_shadows, c.cast_shadows);
    assert_eq!(clone.receive_shadows, c.receive_shadows);
    assert_eq!(clone.pick_id, c.pick_id);
    assert_eq!(clone.pick_only, c.pick_only);
}

#[test]
fn test_shallow_clones_with_result() {
    let c = DrawCommand::new(DrawCommandOptions {
        bounding_volume: Some(BoundingSphere::default()),
        oriented_bounding_box: Some(OrientedBoundingBox::default()),
        cull: Some(false),
        occlude: Some(false),
        model_matrix: Some(DMat4::IDENTITY),
        primitive_type: Some(PrimitiveType::TriangleFan),
        vertex_array: Some("va".to_string()),
        count: Some(3),
        offset: Some(3),
        instance_count: Some(2),
        shader_program: Some("sh".to_string()),
        uniform_map: Some(HashMap::new()),
        render_state: Some("rs".to_string()),
        framebuffer: Some("fb".to_string()),
        pass: Some(Pass::Translucent),
        execute_in_closest_frustum: Some(true),
        owner: Some("ow".to_string()),
        debug_show_bounding_volume: Some(true),
        cast_shadows: Some(true),
        receive_shadows: Some(true),
        pick_id: Some("pk".to_string()),
        pick_only: Some(true),
        ..Default::default()
    });

    let mut result = DrawCommand::default();
    let clone = DrawCommand::shallow_clone(Some(&c), Some(&mut result)).unwrap();

    assert_eq!(result, clone);
    assert_eq!(clone.bounding_volume, c.bounding_volume);
    assert_eq!(clone.oriented_bounding_box, c.oriented_bounding_box);
    assert_eq!(clone.cull, c.cull);
    assert_eq!(clone.occlude, c.occlude);
    assert_eq!(clone.model_matrix, c.model_matrix);
    assert_eq!(clone.primitive_type, c.primitive_type);
    assert_eq!(clone.vertex_array, c.vertex_array);
    assert_eq!(clone.count, c.count);
    assert_eq!(clone.offset, c.offset);
    assert_eq!(clone.instance_count, c.instance_count);
    assert_eq!(clone.shader_program, c.shader_program);
    assert_eq!(clone.uniform_map, c.uniform_map);
    assert_eq!(clone.render_state, c.render_state);
    assert_eq!(clone.framebuffer, c.framebuffer);
    assert_eq!(clone.pass, c.pass);
    assert_eq!(
        clone.execute_in_closest_frustum,
        c.execute_in_closest_frustum
    );
    assert_eq!(clone.owner, c.owner);
    assert_eq!(
        clone.debug_show_bounding_volume,
        c.debug_show_bounding_volume
    );
    assert_eq!(
        clone.debug_overlapping_frustums,
        c.debug_overlapping_frustums
    );
    assert_eq!(clone.cast_shadows, c.cast_shadows);
    assert_eq!(clone.receive_shadows, c.receive_shadows);
    assert_eq!(clone.pick_id, c.pick_id);
    assert_eq!(clone.pick_only, c.pick_only);
}

#[test]
fn test_shallow_clone_returns_none() {
    assert!(DrawCommand::shallow_clone(None, None).is_none());
}
