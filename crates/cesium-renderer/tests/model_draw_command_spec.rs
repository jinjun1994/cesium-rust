//! Port of CesiumJS `Scene/Model/ModelDrawCommandSpec.js` (Apache-2.0).

use std::f64::consts::PI;
use std::sync::Arc;

use cesium_math::{BoundingSphere, Cartesian3, Color, SceneMode, ShadowMode};
use cesium_renderer::model_draw_command::{
    ModelDrawCommand, ModelDrawCommandContent, ModelDrawCommandFramePasses,
    ModelDrawCommandFrameState, ModelDrawCommandMaterial, ModelDrawCommandModelOptions,
    ModelDrawCommandPrimitive, ModelDrawCommandRuntimePrimitive, ModelDrawCommandTile,
    ModelDrawCommandTileset, ModelPrimitiveRenderResources, StyleCommandsNeeded,
};
use cesium_renderer::render_state::{
    BlendingState, ColorMaskState, CullFace, DepthFunction, DepthTestState, RenderState,
    StencilFunction, StencilOperation,
};
use cesium_renderer::uniform::{UniformMap, UniformValue};
use cesium_renderer::{
    DrawCommand, DrawCommandOptions, Pass, RenderStateHandle, StencilConstants, UniformMapHandle,
};
use glam::{DMat4, DVec3};

fn create_mock_frame_state_3d() -> ModelDrawCommandFrameState {
    ModelDrawCommandFrameState {
        mode: SceneMode::Scene3D,
        ellipsoid_maximum_radius: 6378137.0,
        passes: ModelDrawCommandFramePasses::default(),
        edge_visibility_requested: false,
    }
}

fn create_mock_frame_state_2d() -> ModelDrawCommandFrameState {
    ModelDrawCommandFrameState {
        mode: SceneMode::Scene2D,
        ellipsoid_maximum_radius: 6378137.0,
        passes: ModelDrawCommandFramePasses::default(),
        edge_visibility_requested: false,
    }
}

fn mock_render_resources(
    color: Option<Color>,
    silhouette_color: Option<Color>,
    silhouette_size: Option<f64>,
    skip_level_of_detail: Option<bool>,
    planar_fill_id_pass: Option<bool>,
    bounding_sphere_2d_transform: Option<DMat4>,
) -> ModelPrimitiveRenderResources {
    let mut model_opts = ModelDrawCommandModelOptions {
        color: color.unwrap_or(Color::WHITE),
        silhouette_size: silhouette_size.unwrap_or(0.0),
        silhouette_color: silhouette_color.unwrap_or(Color::RED),
        silhouette_id: 1,
        shadows: ShadowMode::Enabled,
        project_to_2d: false,
        ..Default::default()
    };

    if skip_level_of_detail.unwrap_or(false) {
        model_opts.content = Some(ModelDrawCommandContent {
            tileset: ModelDrawCommandTileset {
                has_mixed_content: true,
                backface_commands: Vec::new(),
            },
            tile: ModelDrawCommandTile {
                final_resolution: false,
                selection_depth: 0,
            },
        });
    }

    if let Some(trans) = bounding_sphere_2d_transform {
        model_opts.bounding_sphere_2d = model_opts
            .bounding_sphere_2d
            .transform(trans)
            .unwrap_or(model_opts.bounding_sphere_2d);
    }

    let has_sil = model_opts.has_silhouette();
    let has_lod = skip_level_of_detail.unwrap_or(false);

    ModelPrimitiveRenderResources {
        model: model_opts,
        runtime_primitive: ModelDrawCommandRuntimePrimitive {
            primitive: ModelDrawCommandPrimitive {
                material: ModelDrawCommandMaterial { double_sided: false },
            },
            bounding_sphere: BoundingSphere::new(Cartesian3::ZERO, 1.0),
        },
        has_silhouette: has_sil,
        has_skip_level_of_detail: has_lod,
        planar_fill_id_pass: planar_fill_id_pass.unwrap_or(false),
        edge_geometry: None,
    }
}

fn create_draw_command(model_matrix: Option<DMat4>, pass: Option<Pass>) -> DrawCommand {
    let mm = model_matrix.unwrap_or(DMat4::IDENTITY);
    let bs = BoundingSphere::new(Cartesian3::ZERO, 1.0)
        .transform(mm)
        .unwrap();

    let rs = RenderState {
        depth_test: DepthTestState {
            enabled: true,
            func: DepthFunction::LEqual,
        },
        ..Default::default()
    };

    DrawCommand::new(DrawCommandOptions {
        model_matrix: Some(mm),
        bounding_volume: Some(bs),
        render_state: Some(RenderStateHandle::State(Arc::new(rs))),
        pass: Some(pass.unwrap_or(Pass::Opaque)),
        uniform_map: Some(UniformMapHandle::from(UniformMap::new())),
        ..Default::default()
    })
}

// In CesiumJS, idlMatrix2D translates [180 deg, 0] to (x=0, y=maxRadius * PI, z=0)
fn get_idl_matrix_2d() -> DMat4 {
    let radius = 6378137.0;
    DMat4::from_translation(DVec3::new(0.0, radius * PI, 0.0))
}

fn create_model_draw_command(
    derive_silhouette: bool,
    derive_2d: bool,
    derive_skip_lod: bool,
) -> ModelDrawCommand {
    let idl_2d = get_idl_matrix_2d();
    let transform_2d = if derive_2d { idl_2d } else { DMat4::IDENTITY };

    let resources = mock_render_resources(
        None,
        None,
        if derive_silhouette { Some(1.0) } else { None },
        Some(derive_skip_lod),
        None,
        Some(transform_2d),
    );

    let command = create_draw_command(Some(transform_2d), Some(Pass::Opaque));
    let mut draw_command = ModelDrawCommand::new(Some(command), Some(resources)).unwrap();

    if derive_2d {
        let frame_state_2d = create_mock_frame_state_2d();
        let mut command_list = Vec::new();
        draw_command.push_commands(&frame_state_2d, &mut command_list);
    }

    draw_command
}

fn compute_expected_2d_matrix(model_matrix: DMat4, max_radius: f64) -> DMat4 {
    let mut result = model_matrix;
    let sign = if model_matrix.w_axis.y > 0.0 {
        1.0
    } else if model_matrix.w_axis.y < 0.0 {
        -1.0
    } else {
        0.0
    };
    result.w_axis.y -= sign * 2.0 * PI * max_radius;
    result
}

fn get_render_state(cmd: &DrawCommand) -> RenderState {
    match &cmd.render_state {
        Some(RenderStateHandle::State(rs)) => (**rs).clone(),
        _ => RenderState::default(),
    }
}

#[test]
fn throws_for_undefined_command() {
    let resources = mock_render_resources(None, None, None, None, None, None);
    let result = ModelDrawCommand::new(None, Some(resources));
    assert!(result.is_err());
}

#[test]
fn throws_for_undefined_primitive_render_resources() {
    let cmd = create_draw_command(None, None);
    let result = ModelDrawCommand::new(Some(cmd), None);
    assert!(result.is_err());
}

#[test]
fn constructs() {
    let resources = mock_render_resources(None, None, None, None, None, None);
    let command = create_draw_command(None, None);
    let draw_command =
        ModelDrawCommand::new(Some(command.clone()), Some(resources.clone())).unwrap();

    assert_eq!(draw_command.command(), &command);
    assert_eq!(
        draw_command.runtime_primitive(),
        &resources.runtime_primitive
    );
    assert_eq!(draw_command.model(), &resources.model);
    assert_eq!(draw_command.model_matrix(), command.model_matrix.unwrap());
    assert_eq!(
        draw_command.bounding_volume(),
        command.bounding_volume.unwrap()
    );

    let original = draw_command.original_command().unwrap();
    assert_eq!(original.command, command);
    assert!(!original.is_2d);
    assert!(original.update_shadows);
    assert!(original.update_back_face_culling);
    assert!(original.update_cull_face);
    assert!(original.update_debug_show_bounding_volume);

    let derived = draw_command.derived_commands();
    assert_eq!(derived.len(), 2);
    assert_eq!(&derived[0], original);

    // Translucent command is derived by default
    assert!(draw_command.translucent_command().is_some());
    assert!(draw_command.skip_lod_backface_command().is_none());
    assert!(draw_command.skip_lod_stencil_command().is_none());
    assert!(draw_command.silhouette_model_command().is_none());
    assert!(draw_command.silhouette_color_command().is_none());

    let translucent = draw_command.translucent_command().unwrap();
    assert_eq!(&derived[1], translucent);
    assert!(!translucent.is_2d);
    assert!(translucent.update_shadows);
    assert!(!translucent.update_back_face_culling);
    assert!(!translucent.update_cull_face);
    assert!(translucent.update_debug_show_bounding_volume);

    let inner = &translucent.command;
    assert_ne!(inner, &command);
    assert_eq!(inner.pass, Some(Pass::Translucent));

    let rs = get_render_state(inner);
    assert!(!rs.cull.enabled);
    assert!(rs.depth_test.enabled);
    assert!(!rs.depth_mask);
    assert_eq!(rs.blending, BlendingState::ALPHA_BLEND);
}

#[test]
fn does_not_derive_translucent_command_if_original_command_is_translucent() {
    let resources = mock_render_resources(None, None, None, None, None, None);
    let command = create_draw_command(None, Some(Pass::Translucent));
    let draw_command = ModelDrawCommand::new(Some(command), Some(resources)).unwrap();

    let original = draw_command.original_command().unwrap();
    assert_eq!(original.command.pass, Some(Pass::Translucent));

    let derived = draw_command.derived_commands();
    assert_eq!(derived.len(), 1);
    assert_eq!(&derived[0], original);

    assert!(draw_command.translucent_command().is_none());
}

#[test]
fn derives_silhouette_commands_for_opaque_model() {
    let resources = mock_render_resources(None, None, Some(1.0), None, None, None);
    let command = create_draw_command(None, None);
    let draw_command = ModelDrawCommand::new(Some(command.clone()), Some(resources)).unwrap();

    let derived = draw_command.derived_commands();
    assert_eq!(derived.len(), 3);
    assert_eq!(&derived[0], draw_command.original_command().unwrap());
    assert!(!derived[0].is_2d);

    assert!(draw_command.translucent_command().is_none());
    assert!(draw_command.skip_lod_backface_command().is_none());
    assert!(draw_command.silhouette_model_command().is_some());
    assert!(draw_command.silhouette_color_command().is_some());

    // silhouette model command
    let sil_m = draw_command.silhouette_model_command().unwrap();
    assert_eq!(&derived[1], sil_m);
    assert_ne!(&sil_m.command, &command);
    assert!(!sil_m.is_2d);
    assert!(sil_m.update_shadows);
    assert!(sil_m.update_back_face_culling);
    assert!(sil_m.update_cull_face);
    assert!(sil_m.update_debug_show_bounding_volume);

    let rs_m = get_render_state(&sil_m.command);
    assert!(rs_m.stencil_test.enabled);
    assert_eq!(rs_m.stencil_test.front_function, StencilFunction::Always);
    assert_eq!(rs_m.stencil_test.reference, 1);
    assert_eq!(
        rs_m.stencil_test.front_operation.z_pass,
        StencilOperation::Replace
    );
    assert!(rs_m.color_mask.red); // not invisible

    // silhouette color command
    let sil_c = draw_command.silhouette_color_command().unwrap();
    assert_eq!(&derived[2], sil_c);
    assert_ne!(&sil_c.command, &command);
    assert!(!sil_c.is_2d);
    assert!(!sil_c.update_shadows);
    assert!(!sil_c.update_back_face_culling);
    assert!(!sil_c.update_cull_face);
    assert!(!sil_c.update_debug_show_bounding_volume);

    let rs_c = get_render_state(&sil_c.command);
    assert!(rs_c.depth_test.enabled);
    assert!(!rs_c.cull.enabled);
    assert!(rs_c.stencil_test.enabled);
    assert_eq!(rs_c.stencil_test.front_function, StencilFunction::NotEqual);
    assert_eq!(rs_c.stencil_test.reference, 1);
    assert_eq!(
        rs_c.stencil_test.front_operation.z_pass,
        StencilOperation::Keep
    );

    let um = match &sil_c.command.uniform_map {
        Some(UniformMapHandle::UniformMap(m)) => (**m).clone(),
        _ => UniformMap::new(),
    };
    assert_eq!(
        um.evaluate("model_silhouettePass"),
        Some(UniformValue::Bool(true))
    );
    assert!(!sil_c.command.cast_shadows);
    assert!(!sil_c.command.receive_shadows);
}

#[test]
fn derives_silhouette_commands_for_translucent_model() {
    let resources = mock_render_resources(
        Some(Color::new(1.0, 1.0, 1.0, 0.5)),
        None,
        Some(1.0),
        None,
        None,
        None,
    );
    let command = create_draw_command(None, Some(Pass::Translucent));
    let draw_command = ModelDrawCommand::new(Some(command), Some(resources)).unwrap();

    let sil_m = draw_command.silhouette_model_command().unwrap();
    assert!(!sil_m.update_back_face_culling);
    assert!(!sil_m.update_cull_face);

    let sil_c = draw_command.silhouette_color_command().unwrap();
    assert_eq!(sil_c.command.pass, Some(Pass::Translucent));
    let rs_c = get_render_state(&sil_c.command);
    assert!(!rs_c.depth_mask);
    assert_eq!(rs_c.blending, BlendingState::ALPHA_BLEND);
}

#[test]
fn derives_silhouette_commands_for_invisible_model() {
    let resources = mock_render_resources(
        Some(Color::new(1.0, 1.0, 1.0, 0.0)),
        None,
        Some(1.0),
        None,
        None,
        None,
    );
    let command = create_draw_command(None, None);
    let draw_command = ModelDrawCommand::new(Some(command), Some(resources)).unwrap();

    let sil_m = draw_command.silhouette_model_command().unwrap();
    let rs_m = get_render_state(&sil_m.command);
    assert_eq!(
        rs_m.color_mask,
        ColorMaskState {
            red: false,
            green: false,
            blue: false,
            alpha: false,
        }
    );
}

#[test]
fn derives_silhouette_commands_for_translucent_silhouette_color() {
    let resources = mock_render_resources(
        Some(Color::WHITE),
        Some(Color::new(1.0, 1.0, 1.0, 0.5)),
        Some(1.0),
        None,
        None,
        None,
    );
    let command = create_draw_command(None, None);
    let draw_command = ModelDrawCommand::new(Some(command), Some(resources)).unwrap();

    let sil_c = draw_command.silhouette_color_command().unwrap();
    assert_eq!(sil_c.command.pass, Some(Pass::Translucent));
    let rs_c = get_render_state(&sil_c.command);
    assert!(!rs_c.depth_mask);
    assert_eq!(rs_c.blending, BlendingState::ALPHA_BLEND);
}

#[test]
fn constructs_skip_level_of_detail_commands() {
    let resources = mock_render_resources(None, None, None, Some(true), None, None);
    let command = create_draw_command(None, None);
    let draw_command = ModelDrawCommand::new(Some(command.clone()), Some(resources)).unwrap();

    let derived = draw_command.derived_commands();
    assert_eq!(derived.len(), 4);
    assert_eq!(&derived[0], draw_command.original_command().unwrap());
    assert_eq!(&derived[1], draw_command.translucent_command().unwrap());

    let backface = draw_command.skip_lod_backface_command().unwrap();
    assert_eq!(&derived[2], backface);
    assert!(!backface.is_2d);
    assert!(!backface.update_shadows);
    assert!(!backface.update_back_face_culling);
    assert!(backface.update_cull_face);
    assert!(!backface.update_debug_show_bounding_volume);

    let inner_back = &backface.command;
    assert_ne!(inner_back, &command);
    let rs_back = get_render_state(inner_back);
    assert!(rs_back.cull.enabled);
    assert_eq!(rs_back.cull.face, CullFace::Front);
    assert_eq!(
        rs_back.color_mask,
        ColorMaskState {
            red: false,
            green: false,
            blue: false,
            alpha: false,
        }
    );
    assert!(rs_back.polygon_offset.enabled);
    assert_eq!(rs_back.polygon_offset.factor, 5.0);
    assert_eq!(rs_back.polygon_offset.units, 5.0);

    let um_back = match &inner_back.uniform_map {
        Some(UniformMapHandle::UniformMap(m)) => (**m).clone(),
        _ => UniformMap::new(),
    };
    assert_eq!(
        um_back.evaluate("u_polygonOffset"),
        Some(UniformValue::Vec2([5.0, 5.0]))
    );
    assert!(!inner_back.cast_shadows);
    assert!(!inner_back.receive_shadows);

    let stencil = draw_command.skip_lod_stencil_command().unwrap();
    assert_eq!(&derived[3], stencil);
    assert!(!stencil.is_2d);
    assert!(stencil.update_shadows);
    assert!(stencil.update_back_face_culling);
    assert!(stencil.update_cull_face);
    assert!(stencil.update_debug_show_bounding_volume);

    let inner_sten = &stencil.command;
    let rs_sten = get_render_state(inner_sten);
    assert!(rs_sten.stencil_test.enabled);
    assert_eq!(rs_sten.stencil_test.mask, StencilConstants::SKIP_LOD_MASK);
    assert_eq!(
        rs_sten.stencil_test.reference,
        StencilConstants::CESIUM_3D_TILE_MASK as i32
    );
    assert_eq!(
        rs_sten.stencil_test.front_function,
        StencilFunction::GEqual
    );
    assert_eq!(
        rs_sten.stencil_test.front_operation.z_pass,
        StencilOperation::Replace
    );
    assert_eq!(
        rs_sten.stencil_mask,
        StencilConstants::CESIUM_3D_TILE_MASK | StencilConstants::SKIP_LOD_MASK
    );
}

#[test]
fn does_not_construct_skip_level_of_detail_commands_if_original_command_is_translucent() {
    let resources = mock_render_resources(None, None, None, Some(true), None, None);
    let command = create_draw_command(None, Some(Pass::Translucent));
    let draw_command = ModelDrawCommand::new(Some(command), Some(resources)).unwrap();

    let derived = draw_command.derived_commands();
    assert_eq!(derived.len(), 1);
    assert!(draw_command.translucent_command().is_none());
    assert!(draw_command.skip_lod_backface_command().is_none());
    assert!(draw_command.skip_lod_stencil_command().is_none());
    assert!(draw_command.silhouette_model_command().is_none());
}

#[test]
fn push_commands_pushes_original_command_if_style_commands_needed_is_none() {
    let resources = mock_render_resources(None, None, None, None, None, None);
    let command = create_draw_command(None, None);
    let mut draw_command =
        ModelDrawCommand::new(Some(command.clone()), Some(resources)).unwrap();

    let frame_state = create_mock_frame_state_3d();
    let mut command_list = Vec::new();
    draw_command.push_commands(&frame_state, &mut command_list);

    assert_eq!(command_list.len(), 1);
    assert_eq!(command_list[0], command);
}

#[test]
fn push_commands_pushes_original_command_if_style_is_all_opaque() {
    let resources = mock_render_resources(None, None, None, None, None, None);
    let command = create_draw_command(None, None);
    let mut draw_command =
        ModelDrawCommand::new(Some(command.clone()), Some(resources)).unwrap();
    draw_command.model_mut().style_commands_needed = Some(StyleCommandsNeeded::AllOpaque);

    let frame_state = create_mock_frame_state_3d();
    let mut command_list = Vec::new();
    draw_command.push_commands(&frame_state, &mut command_list);

    assert_eq!(command_list.len(), 1);
    assert_eq!(command_list[0], command);
}

#[test]
fn push_commands_pushes_translucent_command_if_style_is_all_translucent() {
    let resources = mock_render_resources(None, None, None, None, None, None);
    let command = create_draw_command(None, None);
    let mut draw_command = ModelDrawCommand::new(Some(command), Some(resources)).unwrap();
    draw_command.model_mut().style_commands_needed = Some(StyleCommandsNeeded::AllTranslucent);

    let frame_state = create_mock_frame_state_3d();
    let mut command_list = Vec::new();
    draw_command.push_commands(&frame_state, &mut command_list);

    assert_eq!(command_list.len(), 1);
    assert_eq!(
        command_list[0],
        draw_command.translucent_command().unwrap().command
    );
}

#[test]
fn push_commands_pushes_both_commands_if_style_is_opaque_and_translucent() {
    let resources = mock_render_resources(None, None, None, None, None, None);
    let command = create_draw_command(None, None);
    let mut draw_command = ModelDrawCommand::new(Some(command), Some(resources)).unwrap();
    draw_command.model_mut().style_commands_needed =
        Some(StyleCommandsNeeded::OpaqueAndTranslucent);

    let frame_state = create_mock_frame_state_3d();
    let mut command_list = Vec::new();
    draw_command.push_commands(&frame_state, &mut command_list);

    assert_eq!(command_list.len(), 2);
    assert_eq!(
        command_list[0],
        draw_command.translucent_command().unwrap().command
    );
    assert_eq!(
        command_list[1],
        draw_command.original_command().unwrap().command
    );
}

#[test]
fn push_commands_pushes_silhouette_model_command() {
    let resources = mock_render_resources(None, None, Some(1.0), None, None, None);
    let command = create_draw_command(None, None);
    let mut draw_command = ModelDrawCommand::new(Some(command), Some(resources)).unwrap();

    let frame_state = create_mock_frame_state_3d();
    let mut command_list = Vec::new();
    draw_command.push_commands(&frame_state, &mut command_list);

    assert_eq!(command_list.len(), 1);
    assert_eq!(
        command_list[0],
        draw_command.silhouette_model_command().unwrap().command
    );
}

#[test]
fn push_commands_pushes_skip_level_of_detail_commands() {
    let resources = mock_render_resources(None, None, None, Some(true), None, None);
    let command = create_draw_command(None, None);
    let mut draw_command = ModelDrawCommand::new(Some(command), Some(resources)).unwrap();

    let frame_state = create_mock_frame_state_3d();
    let mut command_list = Vec::new();
    draw_command.push_commands(&frame_state, &mut command_list);

    assert_eq!(command_list.len(), 1);
    assert_eq!(
        command_list[0],
        draw_command.skip_lod_stencil_command().unwrap().command
    );

    let backface_commands = &draw_command
        .model()
        .content
        .as_ref()
        .unwrap()
        .tileset
        .backface_commands;
    assert_eq!(backface_commands.len(), 1);
    assert_eq!(
        backface_commands[0],
        draw_command.skip_lod_backface_command().unwrap().command
    );
}

#[test]
fn push_commands_does_not_push_skip_lod_backface_command_if_tile_is_at_final_resolution() {
    let resources = mock_render_resources(None, None, None, Some(true), None, None);
    let command = create_draw_command(None, None);
    let mut draw_command = ModelDrawCommand::new(Some(command), Some(resources)).unwrap();

    draw_command
        .model_mut()
        .content
        .as_mut()
        .unwrap()
        .tile
        .final_resolution = true;

    let frame_state = create_mock_frame_state_3d();
    let mut command_list = Vec::new();
    draw_command.push_commands(&frame_state, &mut command_list);

    assert_eq!(command_list.len(), 1);
    assert_eq!(
        command_list[0],
        draw_command.skip_lod_stencil_command().unwrap().command
    );

    let backface_commands = &draw_command
        .model()
        .content
        .as_ref()
        .unwrap()
        .tileset
        .backface_commands;
    assert_eq!(backface_commands.len(), 0);
}

#[test]
fn push_commands_derives_2d_command_if_model_is_near_idl() {
    let idl_2d = get_idl_matrix_2d();
    let resources = mock_render_resources(None, None, None, None, None, Some(idl_2d));
    let command = create_draw_command(Some(idl_2d), None);
    let mut draw_command = ModelDrawCommand::new(Some(command), Some(resources)).unwrap();

    let frame_state_2d = create_mock_frame_state_2d();
    assert!(draw_command
        .original_command()
        .unwrap()
        .derived_command_2d
        .is_none());

    let mut command_list = Vec::new();
    draw_command.push_commands(&frame_state_2d, &mut command_list);

    let orig = draw_command.original_command().unwrap();
    let orig_2d = orig.derived_command_2d.as_ref().unwrap();
    assert!(orig_2d.is_2d);

    assert_eq!(
        orig.command.model_matrix.unwrap(),
        draw_command.model_matrix()
    );
    assert_eq!(
        orig_2d.command.model_matrix.unwrap(),
        draw_command.model_matrix_2d()
    );

    assert_eq!(command_list.len(), 2);
    assert_eq!(command_list[0], orig.command);
    assert_eq!(command_list[1], orig_2d.command);
}

#[test]
fn push_commands_derives_2d_translucent_command() {
    let idl_2d = get_idl_matrix_2d();
    let resources = mock_render_resources(None, None, None, None, None, Some(idl_2d));
    let command = create_draw_command(Some(idl_2d), None);
    let mut draw_command = ModelDrawCommand::new(Some(command), Some(resources)).unwrap();
    draw_command.model_mut().style_commands_needed =
        Some(StyleCommandsNeeded::OpaqueAndTranslucent);

    let frame_state_2d = create_mock_frame_state_2d();
    let mut command_list = Vec::new();
    draw_command.push_commands(&frame_state_2d, &mut command_list);

    let orig = draw_command.original_command().unwrap();
    let trans = draw_command.translucent_command().unwrap();

    let orig_2d = orig.derived_command_2d.as_ref().unwrap();
    let trans_2d = trans.derived_command_2d.as_ref().unwrap();

    assert!(orig_2d.is_2d);
    assert!(trans_2d.is_2d);

    assert_eq!(
        trans.command.model_matrix.unwrap(),
        draw_command.model_matrix()
    );
    assert_eq!(
        trans_2d.command.model_matrix.unwrap(),
        draw_command.model_matrix_2d()
    );

    assert_eq!(command_list.len(), 4);
    assert_eq!(command_list[0], trans.command);
    assert_eq!(command_list[1], trans_2d.command);
    assert_eq!(command_list[2], orig.command);
    assert_eq!(command_list[3], orig_2d.command);
}

#[test]
fn push_commands_derives_2d_silhouette_commands() {
    let idl_2d = get_idl_matrix_2d();
    let resources = mock_render_resources(None, None, Some(1.0), None, None, Some(idl_2d));
    let command = create_draw_command(Some(idl_2d), None);
    let mut draw_command = ModelDrawCommand::new(Some(command), Some(resources)).unwrap();

    let frame_state_2d = create_mock_frame_state_2d();
    let mut command_list = Vec::new();
    draw_command.push_commands(&frame_state_2d, &mut command_list);

    let sil_m = draw_command.silhouette_model_command().unwrap();
    let sil_c = draw_command.silhouette_color_command().unwrap();

    let sil_m_2d = sil_m.derived_command_2d.as_ref().unwrap();
    let sil_c_2d = sil_c.derived_command_2d.as_ref().unwrap();

    assert!(sil_m_2d.is_2d);
    assert!(sil_c_2d.is_2d);

    assert_eq!(
        sil_m.command.model_matrix.unwrap(),
        draw_command.model_matrix()
    );
    assert_eq!(
        sil_m_2d.command.model_matrix.unwrap(),
        draw_command.model_matrix_2d()
    );

    assert_eq!(command_list.len(), 2);
    assert_eq!(command_list[0], sil_m.command);
    assert_eq!(command_list[1], sil_m_2d.command);
}

#[test]
fn push_commands_derives_2d_skip_level_of_detail_commands() {
    let idl_2d = get_idl_matrix_2d();
    let resources = mock_render_resources(None, None, None, Some(true), None, Some(idl_2d));
    let command = create_draw_command(Some(idl_2d), None);
    let mut draw_command = ModelDrawCommand::new(Some(command), Some(resources)).unwrap();

    let frame_state_2d = create_mock_frame_state_2d();
    let mut command_list = Vec::new();
    draw_command.push_commands(&frame_state_2d, &mut command_list);

    let back = draw_command.skip_lod_backface_command().unwrap();
    let sten = draw_command.skip_lod_stencil_command().unwrap();

    let back_2d = back.derived_command_2d.as_ref().unwrap();
    let sten_2d = sten.derived_command_2d.as_ref().unwrap();

    assert!(back_2d.is_2d);
    assert!(sten_2d.is_2d);

    assert_eq!(command_list.len(), 2);
    assert_eq!(command_list[0], sten.command);
    assert_eq!(command_list[1], sten_2d.command);

    let backface_commands = &draw_command
        .model()
        .content
        .as_ref()
        .unwrap()
        .tileset
        .backface_commands;
    assert_eq!(backface_commands.len(), 2);
    assert_eq!(backface_commands[0], back.command);
    assert_eq!(backface_commands[1], back_2d.command);
}

#[test]
fn push_commands_does_not_derive_2d_commands_if_model_is_not_near_idl() {
    // Model far away from IDL
    let model_matrix_2d = DMat4::from_translation(DVec3::new(100.0, 250.0, 0.0));
    let resources = mock_render_resources(None, None, None, None, None, Some(model_matrix_2d));
    let command = create_draw_command(Some(model_matrix_2d), None);
    let mut draw_command = ModelDrawCommand::new(Some(command), Some(resources)).unwrap();

    let frame_state_2d = create_mock_frame_state_2d();
    let mut command_list = Vec::new();
    draw_command.push_commands(&frame_state_2d, &mut command_list);

    assert!(draw_command
        .original_command()
        .unwrap()
        .derived_command_2d
        .is_none());
    assert_eq!(command_list.len(), 1);
    assert_eq!(
        command_list[0],
        draw_command.original_command().unwrap().command
    );
}

#[test]
fn push_commands_updates_model_matrix_for_2d_commands() {
    let idl_2d = get_idl_matrix_2d();
    let resources = mock_render_resources(None, None, None, None, None, Some(idl_2d));
    let command = create_draw_command(Some(idl_2d), None);
    let mut draw_command = ModelDrawCommand::new(Some(command), Some(resources)).unwrap();

    let frame_state_2d = create_mock_frame_state_2d();
    let mut command_list = Vec::new();
    draw_command.push_commands(&frame_state_2d, &mut command_list);

    let expected_mm =
        compute_expected_2d_matrix(idl_2d, frame_state_2d.ellipsoid_maximum_radius);
    let orig = draw_command.original_command().unwrap();
    let orig_2d = orig.derived_command_2d.as_ref().unwrap();

    assert_eq!(orig.command.model_matrix.unwrap(), idl_2d);
    assert_eq!(orig_2d.command.model_matrix.unwrap(), expected_mm);
}

#[test]
fn push_silhouette_commands_pushes_silhouette_pass_commands() {
    let mut draw_command = create_model_draw_command(true, false, false);

    let frame_state = create_mock_frame_state_3d();
    let mut command_list = Vec::new();
    draw_command.push_commands(&frame_state, &mut command_list);
    assert_eq!(command_list.len(), 1);
    assert_eq!(
        command_list[0],
        draw_command.silhouette_model_command().unwrap().command
    );

    let mut sil_commands = Vec::new();
    draw_command.push_silhouette_commands(&frame_state, &mut sil_commands);
    assert_eq!(sil_commands.len(), 1);
    assert_eq!(
        sil_commands[0],
        draw_command.silhouette_color_command().unwrap().command
    );
}

#[test]
fn push_silhouette_commands_pushes_2d_silhouette_pass_commands() {
    let mut draw_command = create_model_draw_command(true, true, false);

    let frame_state_2d = create_mock_frame_state_2d();
    let mut command_list = Vec::new();
    draw_command.push_commands(&frame_state_2d, &mut command_list);
    assert_eq!(command_list.len(), 2);

    let mut sil_commands = Vec::new();
    draw_command.push_silhouette_commands(&frame_state_2d, &mut sil_commands);
    assert_eq!(sil_commands.len(), 2);
    assert_eq!(
        sil_commands[0],
        draw_command.silhouette_color_command().unwrap().command
    );
    assert_eq!(
        sil_commands[1],
        draw_command
            .silhouette_color_command()
            .unwrap()
            .derived_command_2d
            .as_ref()
            .unwrap()
            .command
    );
}

#[test]
fn updates_model_matrix() {
    let mut draw_command = create_model_draw_command(false, false, false);
    let new_mm = DMat4::from_translation(DVec3::new(10.0, 20.0, 30.0));

    draw_command.set_model_matrix(new_mm);
    assert_eq!(draw_command.model_matrix(), new_mm);
    assert_eq!(
        draw_command.bounding_volume().center,
        DVec3::new(10.0, 20.0, 30.0)
    );

    for cmd in draw_command.derived_commands() {
        assert_eq!(cmd.command.model_matrix.unwrap(), new_mm);
        assert_eq!(
            cmd.command.bounding_volume.unwrap().center,
            DVec3::new(10.0, 20.0, 30.0)
        );
    }
}

#[test]
fn updates_shadows() {
    let mut draw_command = create_model_draw_command(false, false, false);
    assert_eq!(draw_command.derived_commands().len(), 2);

    for cmd in draw_command.derived_commands() {
        assert!(!cmd.command.cast_shadows);
        assert!(!cmd.command.receive_shadows);
    }

    draw_command.set_shadows(ShadowMode::Enabled);
    for cmd in draw_command.derived_commands() {
        assert!(cmd.command.cast_shadows);
        assert!(cmd.command.receive_shadows);
    }
}

#[test]
fn does_not_update_shadows_for_silhouette_color_command() {
    let mut draw_command = create_model_draw_command(true, false, false);
    assert_eq!(draw_command.derived_commands().len(), 3);

    draw_command.set_shadows(ShadowMode::Enabled);
    for cmd in draw_command.derived_commands() {
        if !cmd.update_shadows {
            assert_eq!(cmd, draw_command.silhouette_color_command().unwrap());
        }
        assert_eq!(cmd.command.cast_shadows, cmd.update_shadows);
        assert_eq!(cmd.command.receive_shadows, cmd.update_shadows);
    }
}

#[test]
fn does_not_update_shadows_for_skip_lod_backface_command() {
    let mut draw_command = create_model_draw_command(false, false, true);
    assert_eq!(draw_command.derived_commands().len(), 4);

    draw_command.set_shadows(ShadowMode::Enabled);
    for cmd in draw_command.derived_commands() {
        if !cmd.update_shadows {
            assert_eq!(cmd, draw_command.skip_lod_backface_command().unwrap());
        }
        assert_eq!(cmd.command.cast_shadows, cmd.update_shadows);
        assert_eq!(cmd.command.receive_shadows, cmd.update_shadows);
    }
}

#[test]
fn does_not_update_shadows_for_2d_commands() {
    let mut draw_command = create_model_draw_command(false, true, false);
    assert_eq!(draw_command.derived_commands().len(), 4);

    draw_command.set_shadows(ShadowMode::Enabled);
    for cmd in draw_command.derived_commands() {
        if !cmd.update_shadows {
            assert!(cmd.is_2d);
        }
        assert_eq!(cmd.command.cast_shadows, cmd.update_shadows);
        assert_eq!(cmd.command.receive_shadows, cmd.update_shadows);
    }
}

#[test]
fn updates_back_face_culling_for_opaque_command() {
    let mut draw_command = create_model_draw_command(false, false, false);
    assert_eq!(draw_command.derived_commands().len(), 2);

    for cmd in draw_command.derived_commands() {
        assert!(!get_render_state(&cmd.command).cull.enabled);
    }

    draw_command.set_back_face_culling(true);
    for cmd in draw_command.derived_commands() {
        let rs = get_render_state(&cmd.command);
        if !cmd.update_back_face_culling {
            assert_eq!(cmd, draw_command.translucent_command().unwrap());
        }
        assert_eq!(rs.cull.enabled, cmd.update_back_face_culling);
    }
}

#[test]
fn does_not_update_back_face_culling_for_silhouette_pass_command() {
    let mut draw_command = create_model_draw_command(true, false, false);
    draw_command.set_back_face_culling(true);

    for cmd in draw_command.derived_commands() {
        let rs = get_render_state(&cmd.command);
        if !cmd.update_back_face_culling {
            assert_eq!(cmd, draw_command.silhouette_color_command().unwrap());
        }
        assert_eq!(rs.cull.enabled, cmd.update_back_face_culling);
    }
}

#[test]
fn updates_cull_face() {
    let mut draw_command = create_model_draw_command(false, false, false);
    draw_command.set_cull_face(CullFace::Front);

    for cmd in draw_command.derived_commands() {
        let rs = get_render_state(&cmd.command);
        if cmd.update_cull_face {
            assert_eq!(rs.cull.face, CullFace::Front);
        }
    }
}

#[test]
fn does_not_update_cull_face_for_silhouette_pass_command() {
    let mut draw_command = create_model_draw_command(true, false, false);
    draw_command.set_cull_face(CullFace::Front);

    for cmd in draw_command.derived_commands() {
        let rs = get_render_state(&cmd.command);
        if !cmd.update_cull_face {
            assert_eq!(cmd, draw_command.silhouette_color_command().unwrap());
        }
        if cmd.update_cull_face {
            assert_eq!(rs.cull.face, CullFace::Front);
        }
    }
}

#[test]
fn updates_debug_show_bounding_volume() {
    let mut draw_command = create_model_draw_command(false, false, false);
    draw_command.set_debug_show_bounding_volume(true);

    for cmd in draw_command.derived_commands() {
        assert!(cmd.command.debug_show_bounding_volume);
    }
}

#[test]
fn creates_planar_fill_id_command_when_planar_fill_id_pass_is_true() {
    let resources = mock_render_resources(None, None, None, None, Some(true), None);
    let command = create_draw_command(None, None);
    let draw_command = ModelDrawCommand::new(Some(command), Some(resources)).unwrap();

    assert!(draw_command.needs_planar_fill_id_command());
    assert!(draw_command.planar_fill_id_command().is_some());

    let planar = &draw_command.planar_fill_id_command().unwrap().command;
    assert_eq!(planar.pass, Some(Pass::Cesium3dTilePlanarFillId));
    assert!(!planar.cast_shadows);
    assert!(!planar.receive_shadows);

    let rs = get_render_state(planar);
    assert!(!rs.blending.enabled);
}

#[test]
fn does_not_create_planar_fill_id_command_when_planar_fill_id_pass_is_false() {
    let resources = mock_render_resources(None, None, None, None, Some(false), None);
    let command = create_draw_command(None, None);
    let draw_command = ModelDrawCommand::new(Some(command), Some(resources)).unwrap();

    assert!(!draw_command.needs_planar_fill_id_command());
    assert!(draw_command.planar_fill_id_command().is_none());
}

#[test]
fn push_planar_fill_id_commands_pushes_command_to_list() {
    let resources = mock_render_resources(None, None, None, None, Some(true), None);
    let command = create_draw_command(None, None);
    let draw_command = ModelDrawCommand::new(Some(command), Some(resources)).unwrap();

    let frame_state = create_mock_frame_state_3d();
    let mut command_list = Vec::new();
    draw_command.push_planar_fill_id_commands(&frame_state, &mut command_list);

    assert_eq!(command_list.len(), 1);
    assert_eq!(
        command_list[0].pass,
        Some(Pass::Cesium3dTilePlanarFillId)
    );
}

#[test]
fn push_planar_fill_id_commands_returns_early_when_no_planar_fill_id_command() {
    let resources = mock_render_resources(None, None, None, None, Some(false), None);
    let command = create_draw_command(None, None);
    let draw_command = ModelDrawCommand::new(Some(command), Some(resources)).unwrap();

    let frame_state = create_mock_frame_state_3d();
    let mut command_list = Vec::new();
    draw_command.push_planar_fill_id_commands(&frame_state, &mut command_list);

    assert_eq!(command_list.len(), 0);
}
