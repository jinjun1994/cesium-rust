use std::collections::HashMap;

use cesium_math::{Geometry, GeometryAttributes, GeometryInstance, Matrix4, PrimitiveType, SceneMode};
use cesium_renderer::scene_primitive::{
    Primitive, PrimitiveAppearance, PrimitiveOptions, PrimitivePasses, PrimitiveRenderFrameState,
};

fn create_dummy_instance(id: &str) -> GeometryInstance {
    let geometry = Geometry::new(
        GeometryAttributes::default(),
        vec![0, 1, 2],
        PrimitiveType::Triangles,
        None,
    );
    GeometryInstance {
        geometry,
        model_matrix: Matrix4::IDENTITY,
        id: Some(id.to_string()),
        attributes: HashMap::new(),
    }
}

#[test]
fn test_primitive_default_constructs() {
    let primitive = Primitive::default();
    assert!(primitive.geometry_instances.is_none());
    assert!(primitive.appearance.is_none());
    assert!(primitive.depth_fail_appearance.is_none());
    assert_eq!(primitive.model_matrix, Matrix4::IDENTITY);
    assert!(primitive.show);
    assert!(!primitive.vertex_cache_optimize);
    assert!(!primitive.interleave);
    assert!(primitive.compress_vertices);
    assert!(primitive.release_geometry_instances);
    assert!(primitive.allow_picking);
    assert!(primitive.cull);
    assert!(primitive.asynchronous);
    assert!(!primitive.debug_show_bounding_volume);
    assert!(!primitive.is_destroyed());
}

#[test]
fn test_primitive_constructs_with_options() {
    let instance = create_dummy_instance("rect1");
    let appearance = PrimitiveAppearance::new(true, false);
    let depth_fail = PrimitiveAppearance::new(false, true);
    let model_matrix = Matrix4::from_uniform_scale(5.0);

    let primitive = Primitive::new(PrimitiveOptions {
        geometry_instances: Some(vec![instance]),
        appearance: Some(appearance.clone()),
        depth_fail_appearance: Some(depth_fail.clone()),
        model_matrix: Some(model_matrix),
        show: Some(false),
        vertex_cache_optimize: Some(true),
        interleave: Some(true),
        compress_vertices: Some(false),
        release_geometry_instances: Some(false),
        allow_picking: Some(false),
        cull: Some(false),
        asynchronous: Some(false),
        debug_show_bounding_volume: Some(true),
    });

    assert_eq!(primitive.geometry_instances.as_ref().unwrap().len(), 1);
    assert_eq!(primitive.appearance, Some(appearance));
    assert_eq!(primitive.depth_fail_appearance, Some(depth_fail));
    assert_eq!(primitive.model_matrix, model_matrix);
    assert!(!primitive.show);
    assert!(primitive.vertex_cache_optimize);
    assert!(primitive.interleave);
    assert!(!primitive.compress_vertices);
    assert!(!primitive.release_geometry_instances);
    assert!(!primitive.allow_picking);
    assert!(!primitive.cull);
    assert!(!primitive.asynchronous);
    assert!(primitive.debug_show_bounding_volume);
}

#[test]
fn test_primitive_releases_geometry_instances_when_release_is_true() {
    let instance = create_dummy_instance("rect1");
    let mut primitive = Primitive::new(PrimitiveOptions {
        geometry_instances: Some(vec![instance]),
        release_geometry_instances: Some(true),
        asynchronous: Some(false),
        ..Default::default()
    });

    assert!(primitive.geometry_instances.is_some());
    let mut frame_state = PrimitiveRenderFrameState::default();
    primitive.update(&mut frame_state);
    assert!(primitive.geometry_instances.is_none());
}

#[test]
fn test_primitive_does_not_release_geometry_instances_when_release_is_false() {
    let instance = create_dummy_instance("rect1");
    let mut primitive = Primitive::new(PrimitiveOptions {
        geometry_instances: Some(vec![instance]),
        release_geometry_instances: Some(false),
        asynchronous: Some(false),
        ..Default::default()
    });

    assert!(primitive.geometry_instances.is_some());
    let mut frame_state = PrimitiveRenderFrameState::default();
    primitive.update(&mut frame_state);
    assert!(primitive.geometry_instances.is_some());
}

#[test]
fn test_primitive_does_not_render_when_geometry_instances_is_empty() {
    let mut primitive = Primitive::new(PrimitiveOptions {
        geometry_instances: Some(Vec::new()),
        asynchronous: Some(false),
        ..Default::default()
    });

    let mut frame_state = PrimitiveRenderFrameState::default();
    primitive.update(&mut frame_state);
    assert_eq!(frame_state.command_list.len(), 0);
}

#[test]
fn test_primitive_does_not_render_when_show_is_false() {
    let instance = create_dummy_instance("rect1");
    let mut primitive = Primitive::new(PrimitiveOptions {
        geometry_instances: Some(vec![instance]),
        release_geometry_instances: Some(false),
        asynchronous: Some(false),
        ..Default::default()
    });

    let mut frame_state = PrimitiveRenderFrameState::default();
    primitive.update(&mut frame_state);
    assert!(!frame_state.command_list.is_empty());

    frame_state.command_list.clear();
    primitive.show = false;
    primitive.update(&mut frame_state);
    assert_eq!(frame_state.command_list.len(), 0);
}

#[test]
fn test_primitive_does_not_render_other_than_for_render_or_pick_pass() {
    let instance = create_dummy_instance("rect1");
    let mut primitive = Primitive::new(PrimitiveOptions {
        geometry_instances: Some(vec![instance]),
        asynchronous: Some(false),
        ..Default::default()
    });

    let mut frame_state = PrimitiveRenderFrameState {
        passes: PrimitivePasses {
            render: false,
            pick: false,
        },
        ..Default::default()
    };

    primitive.update(&mut frame_state);
    assert_eq!(frame_state.command_list.len(), 0);
}

#[test]
fn test_primitive_does_not_render_when_scene_3d_only_and_mode_is_not_3d() {
    let instance = create_dummy_instance("rect1");
    let mut primitive = Primitive::new(PrimitiveOptions {
        geometry_instances: Some(vec![instance]),
        release_geometry_instances: Some(false),
        asynchronous: Some(false),
        ..Default::default()
    });

    let mut frame_state = PrimitiveRenderFrameState {
        scene_3d_only: true,
        mode: SceneMode::Scene2D,
        ..Default::default()
    };
    primitive.update(&mut frame_state);
    assert_eq!(frame_state.command_list.len(), 0);

    let mut frame_state_cv = PrimitiveRenderFrameState {
        scene_3d_only: true,
        mode: SceneMode::ColumbusView,
        ..Default::default()
    };
    primitive.update(&mut frame_state_cv);
    assert_eq!(frame_state_cv.command_list.len(), 0);
}

#[test]
fn test_primitive_renders_in_two_passes_for_closed_translucent_geometry() {
    let instance = create_dummy_instance("box");
    let mut primitive = Primitive::new(PrimitiveOptions {
        geometry_instances: Some(vec![instance]),
        appearance: Some(PrimitiveAppearance::new(true, true)),
        asynchronous: Some(false),
        ..Default::default()
    });

    let mut frame_state = PrimitiveRenderFrameState {
        scene_3d_only: true,
        mode: SceneMode::Scene3D,
        ..Default::default()
    };
    primitive.update(&mut frame_state);
    assert_eq!(frame_state.command_list.len(), 2);
}

#[test]
fn test_primitive_renders_in_columbus_view_and_2d_when_scene_3d_only_is_false() {
    let instance = create_dummy_instance("rect1");
    let mut primitive = Primitive::new(PrimitiveOptions {
        geometry_instances: Some(vec![instance]),
        release_geometry_instances: Some(false),
        asynchronous: Some(false),
        ..Default::default()
    });

    let mut frame_state_cv = PrimitiveRenderFrameState {
        scene_3d_only: false,
        mode: SceneMode::ColumbusView,
        ..Default::default()
    };
    primitive.update(&mut frame_state_cv);
    assert_eq!(frame_state_cv.command_list.len(), 1);

    let mut frame_state_2d = PrimitiveRenderFrameState {
        scene_3d_only: false,
        mode: SceneMode::Scene2D,
        ..Default::default()
    };
    primitive.update(&mut frame_state_2d);
    assert_eq!(frame_state_2d.command_list.len(), 1);
}

#[test]
fn test_primitive_destroys() {
    let primitive = Primitive::default();
    assert!(!primitive.is_destroyed());
    primitive.destroy();
    assert!(primitive.is_destroyed());
}
