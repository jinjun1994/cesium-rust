use cesium_renderer::commands::Pass;
use cesium_renderer::edge_framebuffer::EdgeFramebuffer;
use cesium_renderer::edge_visibility_pipeline_stage::{
    EdgeDisplayMode, EdgeRenderCommand, EdgeRenderingModel, EdgeVisibilityData,
    EdgeVisibilityPrimitive,
};

fn create_test_model(mode: EdgeDisplayMode) -> EdgeRenderingModel {
    let positions = vec![
        0.0, 0.0, 0.0,
        1.0, 0.0, 0.0,
        1.0, 1.0, 0.0,
        0.0, 1.0, 0.0,
    ];
    let indices = vec![0, 1, 2, 0, 2, 3];
    let test_visibility_buffer = vec![18, 2];
    let normals = vec![
        10.0, 10.0, 126.0,
        10.0, 126.0, 10.0,
    ];

    let primitive = EdgeVisibilityPrimitive {
        positions,
        indices,
        edge_visibility: Some(EdgeVisibilityData {
            visibility: Some(test_visibility_buffer),
            silhouette_normals: Some(normals),
            material_color: None,
            line_strings: Vec::new(),
        }),
        vertex_colors: None,
        cumulative_distances: None,
    };

    EdgeRenderingModel::new(primitive, mode)
}

struct MockScene {
    enable_edge_visibility: bool,
    edge_visibility_requested: bool,
    command_list: Vec<EdgeRenderCommand>,
    edge_framebuffer: Option<EdgeFramebuffer>,
}

impl MockScene {
    fn new() -> Self {
        Self {
            enable_edge_visibility: false,
            edge_visibility_requested: false,
            command_list: Vec::new(),
            edge_framebuffer: None,
        }
    }

    fn render(&mut self, model: &mut EdgeRenderingModel, snap_pass: bool) {
        self.command_list.clear();
        self.edge_visibility_requested = false;
        model.update(
            snap_pass,
            &mut self.edge_visibility_requested,
            &mut self.enable_edge_visibility,
            &mut self.command_list,
        );

        if self.edge_visibility_requested && self.edge_framebuffer.is_none() {
            let mut fb = EdgeFramebuffer::new().expect("EdgeFramebuffer create");
            let _ = fb.update(64, 64, false);
            self.edge_framebuffer = Some(fb);
        }
    }
}

#[test]
fn test_surfaces_only_builds_no_edge_geometry_and_does_not_enable_scene_edge_visibility() {
    let mut scene = MockScene::new();
    let mut model = create_test_model(EdgeDisplayMode::SurfacesOnly);

    scene.render(&mut model, false);

    assert!(!model.edge_geometry_needed);
    assert_eq!(model.edge_commands.len(), 0);
    assert!(!scene.edge_visibility_requested);
    assert!(!scene.enable_edge_visibility);
}

#[test]
fn test_builds_edge_geometry_when_edges_become_displayed_and_destroys_it_when_hidden_again() {
    let mut scene = MockScene::new();
    let mut model = create_test_model(EdgeDisplayMode::SurfacesOnly);

    scene.render(&mut model, false);
    assert_eq!(model.edge_commands.len(), 0);

    model.edge_display_mode = EdgeDisplayMode::SurfacesAndEdges;
    scene.render(&mut model, false);
    assert!(!model.edge_commands.is_empty());
    assert!(scene.edge_visibility_requested);
    assert!(scene.enable_edge_visibility);

    // Switching between SurfacesAndEdges and EdgesOnly retains edge geometry
    model.edge_display_mode = EdgeDisplayMode::EdgesOnly;
    scene.render(&mut model, false);
    assert!(!model.edge_commands.is_empty());
    assert!(!scene.edge_visibility_requested);

    // Switching back to SurfacesOnly destroys edge commands on reset
    model.edge_display_mode = EdgeDisplayMode::SurfacesOnly;
    model.reset_draw_commands();
    scene.render(&mut model, false);
    assert_eq!(model.edge_commands.len(), 0);
}

#[test]
fn test_builds_edge_geometry_for_snapping_in_surfaces_only_without_enabling_scene_edge_visibility() {
    let mut scene = MockScene::new();
    let mut model = create_test_model(EdgeDisplayMode::SurfacesOnly);

    scene.render(&mut model, false);
    assert_eq!(model.edge_commands.len(), 0);

    // Simulate snap pass
    scene.render(&mut model, true);
    assert!(model.edge_geometry_needed_for_snapping);
    assert!(!model.edge_commands.is_empty());

    // Edges stay hidden and snap latch survives color render
    scene.render(&mut model, false);
    assert!(!scene.edge_visibility_requested);
    assert!(!scene.enable_edge_visibility);

    let edge_passes: Vec<_> = scene
        .command_list
        .iter()
        .filter(|cmd| {
            cmd.pass == Pass::Cesium3dTileEdges || cmd.pass == Pass::Cesium3dTileEdgesDirect
        })
        .collect();
    assert_eq!(edge_passes.len(), 0);
}

#[test]
fn test_validates_u_is_edge_pass_uniform_and_framebuffer_attachments() {
    let mut scene = MockScene::new();
    let mut model = create_test_model(EdgeDisplayMode::SurfacesAndEdges);

    scene.render(&mut model, false);

    let edge_cmd = scene
        .command_list
        .iter()
        .find(|c| c.pass == Pass::Cesium3dTileEdges);
    let regular_cmd = scene
        .command_list
        .iter()
        .find(|c| c.pass == Pass::Cesium3dTile);

    assert!(edge_cmd.is_some());
    assert!(regular_cmd.is_some());

    assert!(edge_cmd.unwrap().is_edge_pass);
    assert!(!regular_cmd.unwrap().is_edge_pass);

    let efb = scene.edge_framebuffer.as_ref().expect("EdgeFramebuffer exists");
    assert!(efb.framebuffer().is_some());
    assert!(efb.color_texture().is_some());
    assert!(efb.id_texture().is_some());
    assert!(efb.depth_texture().is_some());
}

#[test]
fn test_validates_edge_visibility_shader_code_and_uniforms() {
    let mut scene = MockScene::new();
    let mut model = create_test_model(EdgeDisplayMode::SurfacesAndEdges);

    scene.render(&mut model, false);

    let edge_cmd = scene
        .command_list
        .iter()
        .find(|c| c.pass == Pass::Cesium3dTileEdges)
        .expect("Edge command exists");

    assert!(edge_cmd.vertex_shader_text.contains("HAS_EDGE_VISIBILITY"));
    assert!(edge_cmd.fragment_shader_text.contains("HAS_EDGE_VISIBILITY"));
    assert!(edge_cmd.fragment_shader_text.contains("HAS_EDGE_VISIBILITY_MRT"));
    assert!(edge_cmd.fragment_shader_text.contains("out_id"));
    assert!(edge_cmd.fragment_shader_text.contains("out_edgeDepth"));
    assert!(edge_cmd.is_edge_pass);
}

#[test]
fn test_validates_edge_detection_shader_code_and_texture_sampling() {
    let mut scene = MockScene::new();
    let mut model = create_test_model(EdgeDisplayMode::SurfacesAndEdges);

    scene.render(&mut model, false);

    let regular_cmd = scene
        .command_list
        .iter()
        .find(|c| c.pass == Pass::Cesium3dTile)
        .expect("Regular command exists");

    let fs = &regular_cmd.fragment_shader_text;
    assert!(fs.contains("edgeDetectionStage"));
    assert!(fs.contains("u_isEdgePass"));
    assert!(fs.contains("gl_FragCoord.xy / czm_viewport.zw"));
    assert!(fs.contains("czm_edgeColorTexture"));
    assert!(fs.contains("czm_edgeIdTexture"));
    assert!(fs.contains("czm_globeDepthTexture"));
    assert!(fs.contains("edgeId.r > 0.0"));
    assert!(fs.contains("edgeId.g"));
    assert!(fs.contains("featureIds.featureId_0"));
    assert!(fs.contains("czm_unpackDepth"));
    assert!(fs.contains("geomDepthLinear > globeDepth"));
    assert!(fs.contains("color = edgeColor"));
}

#[test]
fn test_surfaces_only_produces_no_edge_commands() {
    let mut scene = MockScene::new();
    let mut model = create_test_model(EdgeDisplayMode::SurfacesOnly);

    scene.render(&mut model, false);

    let has_edge = scene.command_list.iter().any(|c| {
        c.pass == Pass::Cesium3dTileEdges || c.pass == Pass::Cesium3dTileEdgesDirect
    });
    let has_surface = scene.command_list.iter().any(|c| c.pass == Pass::Cesium3dTile);

    assert!(!has_edge);
    assert!(has_surface);
}

#[test]
fn test_surfaces_and_edges_produces_both_surface_and_mrt_edge_commands() {
    let mut scene = MockScene::new();
    let mut model = create_test_model(EdgeDisplayMode::SurfacesAndEdges);

    scene.render(&mut model, false);

    let has_mrt_edge = scene.command_list.iter().any(|c| c.pass == Pass::Cesium3dTileEdges);
    let has_direct_edge = scene.command_list.iter().any(|c| c.pass == Pass::Cesium3dTileEdgesDirect);
    let has_surface = scene.command_list.iter().any(|c| c.pass == Pass::Cesium3dTile);

    assert!(has_mrt_edge);
    assert!(!has_direct_edge);
    assert!(has_surface);
}

#[test]
fn test_edges_only_produces_only_direct_edge_commands_with_no_surface_commands() {
    let mut scene = MockScene::new();
    let mut model = create_test_model(EdgeDisplayMode::EdgesOnly);

    scene.render(&mut model, false);

    let has_mrt_edge = scene.command_list.iter().any(|c| c.pass == Pass::Cesium3dTileEdges);
    let has_direct_edge = scene.command_list.iter().any(|c| c.pass == Pass::Cesium3dTileEdgesDirect);
    let has_surface = scene.command_list.iter().any(|c| c.pass == Pass::Cesium3dTile);

    assert!(!has_mrt_edge);
    assert!(has_direct_edge);
    assert!(!has_surface);
}

#[test]
fn test_edges_only_does_not_enable_scene_edge_visibility() {
    let mut scene = MockScene::new();
    let mut model = create_test_model(EdgeDisplayMode::EdgesOnly);

    scene.render(&mut model, false);
    assert!(!scene.enable_edge_visibility);
    assert!(!scene.edge_visibility_requested);
}

#[test]
fn test_resets_and_destroys_commands_without_gpu_leaks() {
    let mut scene = MockScene::new();
    let mut model = create_test_model(EdgeDisplayMode::SurfacesAndEdges);

    scene.render(&mut model, false);
    assert_eq!(model.edge_commands.len(), 1);

    model.reset_draw_commands();
    assert_eq!(model.edge_commands.len(), 0);

    model.destroy();
    assert!(model.is_destroyed);
}
