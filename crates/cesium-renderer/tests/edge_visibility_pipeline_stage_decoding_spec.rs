use std::collections::{BTreeSet, HashMap};

use cesium_math::{Cartesian4, PrimitiveType};
use cesium_renderer::edge_visibility_pipeline_stage::{
    extract_visible_edges, EdgeVisibilityData, EdgeVisibilityLineString,
    EdgeVisibilityPipelineStage, EdgeVisibilityPrimitive, EdgeVisibilityRenderResources,
};
use cesium_renderer::shader_builder::ShaderBuilder;

fn create_test_edge_visibility_data(silhouette_normals: Option<Vec<f32>>) -> EdgeVisibilityData {
    // Triangles: [0,1,2, 0,2,3]
    // Edge visibility: [VISIBLE,HIDDEN,SILHOUETTE, HIDDEN,VISIBLE,HIDDEN] = [2,0,1, 0,2,0]
    // Expected bytes: [18, 2] = [00010010, 00000010]
    let test_visibility_buffer = vec![18, 2];
    let normals = silhouette_normals.unwrap_or_else(|| {
        vec![
            10.0, 10.0, 126.0, // normal A for silhouette edge 0 (~+Z, skewed)
            10.0, 126.0, 10.0, // normal B for silhouette edge 0 (~+Y, skewed)
        ]
    });

    EdgeVisibilityData {
        visibility: Some(test_visibility_buffer),
        silhouette_normals: Some(normals),
        material_color: None,
        line_strings: Vec::new(),
    }
}

fn create_test_primitive(silhouette_normals: Option<Vec<f32>>) -> EdgeVisibilityPrimitive {
    // Vertices: (0,0,0), (1,0,0), (1,1,0), (0,1,0)
    // Triangles: [0,1,2], [0,2,3]
    let positions = vec![
        0.0, 0.0, 0.0, // vertex 0
        1.0, 0.0, 0.0, // vertex 1
        1.0, 1.0, 0.0, // vertex 2
        0.0, 1.0, 0.0, // vertex 3
    ];
    let indices = vec![0, 1, 2, 0, 2, 3];

    EdgeVisibilityPrimitive {
        positions,
        indices,
        edge_visibility: Some(create_test_edge_visibility_data(silhouette_normals)),
        vertex_colors: None,
        cumulative_distances: None,
    }
}

struct MockRenderResources {
    shader_builder: ShaderBuilder,
    uniform_map: HashMap<String, Box<dyn Fn() -> serde_json::Value + Send + Sync>>,
}

impl MockRenderResources {
    fn new() -> Self {
        Self {
            shader_builder: ShaderBuilder::default(),
            uniform_map: HashMap::new(),
        }
    }

    fn as_render_resources(&mut self) -> EdgeVisibilityRenderResources<'_> {
        EdgeVisibilityRenderResources {
            shader_builder: &mut self.shader_builder,
            uniform_map: &mut self.uniform_map,
            edge_geometry: None,
        }
    }
}

#[test]
fn test_decodes_edge_visibility_test_data_correctly() {
    let primitive = create_test_primitive(None);
    let mut mock = MockRenderResources::new();
    let mut rr = mock.as_render_resources();

    EdgeVisibilityPipelineStage::process(&mut rr, &primitive);
    let edge_geom = rr.edge_geometry.expect("Edge geometry should be created");

    assert!(edge_geom.index_count > 0);
    assert_eq!(edge_geom.primitive_type, PrimitiveType::Triangles);
}

#[test]
fn test_extracts_correct_edge_visibility_values_from_test_buffer() {
    let test_visibility_buffer = [18u8, 2u8];

    // Edge 0: bits 0-1 of byte 0 (18 = 00010010)
    let edge0 = test_visibility_buffer[0] & 0x3;
    assert_eq!(edge0, 2); // Hard edge

    // Edge 1: bits 2-3 of byte 0
    let edge1 = (test_visibility_buffer[0] >> 2) & 0x3;
    assert_eq!(edge1, 0); // Hidden edge

    // Edge 2: bits 4-5 of byte 0
    let edge2 = (test_visibility_buffer[0] >> 4) & 0x3;
    assert_eq!(edge2, 1); // Silhouette edge

    // Edge 3: bits 6-7 of byte 0
    let edge3 = (test_visibility_buffer[0] >> 6) & 0x3;
    assert_eq!(edge3, 0); // Hidden edge

    // Edge 4: bits 0-1 of byte 1 (2 = 00000010)
    let edge4 = test_visibility_buffer[1] & 0x3;
    assert_eq!(edge4, 2); // Hard edge

    // Edge 5: bits 2-3 of byte 1
    let edge5 = (test_visibility_buffer[1] >> 2) & 0x3;
    assert_eq!(edge5, 0); // Hidden edge
}

#[test]
fn test_processes_triangle_edges_in_correct_order() {
    let primitive = create_test_primitive(None);
    let indices = &primitive.indices;

    assert_eq!(indices.len(), 6);
    // Triangle 0: [0, 1, 2]
    assert_eq!(indices[0], 0);
    assert_eq!(indices[1], 1);
    assert_eq!(indices[2], 2);
    // Triangle 1: [0, 2, 3]
    assert_eq!(indices[3], 0);
    assert_eq!(indices[4], 2);
    assert_eq!(indices[5], 3);
}

#[test]
fn test_filters_edges_based_on_visibility_values() {
    let primitive = create_test_primitive(None);
    let mut mock = MockRenderResources::new();
    let mut rr = mock.as_render_resources();

    EdgeVisibilityPipelineStage::process(&mut rr, &primitive);
    let edge_geom = rr.edge_geometry.expect("Edge geometry should be created");

    // Expected 3 unique visible edges: (0,1), (0,2), (2,3)
    // 3 edges × 6 indices per edge = 18 indices total
    assert_eq!(edge_geom.index_count, 18);
    assert_eq!(edge_geom.index_count % 6, 0);
    assert_eq!(edge_geom.primitive_type, PrimitiveType::Triangles);
}

#[test]
fn test_handles_silhouette_edges_correctly() {
    let primitive = create_test_primitive(None);
    let mut mock = MockRenderResources::new();
    let mut rr = mock.as_render_resources();

    EdgeVisibilityPipelineStage::process(&mut rr, &primitive);

    let vs = rr.shader_builder.create_vertex_shader_source();
    let fs = rr.shader_builder.create_fragment_shader_source();

    let vs_text = vs.sources.join("\n");
    let _fs_text = fs.sources.join("\n");

    assert!(vs.defines.iter().any(|d| d == "HAS_EDGE_VISIBILITY"));
    assert!(fs.defines.iter().any(|d| d == "HAS_EDGE_VISIBILITY_MRT"));
    assert!(vs_text.contains("a_faceNormalA"));
    assert!(vs_text.contains("a_faceNormalB"));
}

#[test]
fn test_creates_proper_edge_geometry_buffers() {
    let primitive = create_test_primitive(None);
    let mut mock = MockRenderResources::new();
    let mut rr = mock.as_render_resources();

    EdgeVisibilityPipelineStage::process(&mut rr, &primitive);
    let edge_geom = rr.edge_geometry.expect("Edge geometry should be created");

    assert!(!edge_geom.positions.is_empty());
    assert!(!edge_geom.indices.is_empty());
    assert!(!edge_geom.face_normals_a.is_empty());
    assert!(!edge_geom.face_normals_b.is_empty());
}

#[test]
fn test_handles_edge_deduplication_correctly() {
    let primitive = create_test_primitive(None);
    let result = extract_visible_edges(&primitive);

    assert_eq!(result.edge_data.len(), 3);
    assert_eq!(result.edge_indices.len(), 6);

    let mut unique_pairs = BTreeSet::new();
    for i in 0..result.edge_data.len() {
        let a = result.edge_indices[i * 2];
        let b = result.edge_indices[i * 2 + 1];
        let key = (a.min(b), a.max(b));
        unique_pairs.insert(key);
    }

    let mut expected_pairs = BTreeSet::new();
    expected_pairs.insert((0, 1));
    expected_pairs.insert((0, 2));
    expected_pairs.insert((2, 3));

    assert_eq!(unique_pairs, expected_pairs);
}

#[test]
fn test_generates_edge_color_attribute_for_material_overrides_and_line_strings() {
    let mut primitive = create_test_primitive(None);
    if let Some(ref mut ev) = primitive.edge_visibility {
        ev.material_color = Some(Cartesian4::new(0.2, 0.3, 0.4, 1.0));
        ev.line_strings = vec![EdgeVisibilityLineString {
            indices: vec![0, 1, 65535, 1, 3],
            restart_index: Some(65535),
            material_color: Some(Cartesian4::new(0.9, 0.1, 0.2, 1.0)),
        }];
    }

    let mut mock = MockRenderResources::new();
    let mut rr = mock.as_render_resources();

    EdgeVisibilityPipelineStage::process(&mut rr, &primitive);

    let vs = rr.shader_builder.create_vertex_shader_source();
    let vs_text = vs.sources.join("\n");
    assert!(vs.defines.iter().any(|d| d == "HAS_EDGE_COLOR_ATTRIBUTE"));
    assert!(vs_text.contains("a_edgeColor"));

    let edge_geom = rr.edge_geometry.expect("Edge geometry should exist");
    assert!(edge_geom.edge_colors.is_some());
}

#[test]
fn test_sets_up_uniforms_correctly() {
    let primitive = create_test_primitive(None);
    let mut mock = MockRenderResources::new();
    let mut rr = mock.as_render_resources();

    EdgeVisibilityPipelineStage::process(&mut rr, &primitive);

    let uniform_getter = rr
        .uniform_map
        .get("u_isEdgePass")
        .expect("u_isEdgePass must be set");
    assert_eq!(uniform_getter(), serde_json::json!(false));

    let line_width_getter = rr
        .uniform_map
        .get("u_lineWidth")
        .expect("u_lineWidth must be set");
    assert_eq!(line_width_getter(), serde_json::json!(1.0));
}

#[test]
fn test_validates_primitive_vs_edge_geometry_structure() {
    let primitive = create_test_primitive(None);
    assert_eq!(primitive.indices.len(), 6);

    let mut mock = MockRenderResources::new();
    let mut rr = mock.as_render_resources();

    EdgeVisibilityPipelineStage::process(&mut rr, &primitive);
    let edge_geom = rr.edge_geometry.expect("Edge geometry must exist");

    assert_eq!(edge_geom.primitive_type, PrimitiveType::Triangles);
    // 3 visible edges × 6 indices = 18
    assert_eq!(edge_geom.index_count, 18);
}

#[test]
fn test_validates_edge_geometry_vertex_data_correctness() {
    let primitive = create_test_primitive(None);
    let mut mock = MockRenderResources::new();
    let mut rr = mock.as_render_resources();

    EdgeVisibilityPipelineStage::process(&mut rr, &primitive);
    let edge_geom = rr.edge_geometry.expect("Edge geometry must exist");

    // 3 edges × 4 vertices = 12 vertices
    // Position: 12 × 3 = 36 floats
    assert_eq!(edge_geom.positions.len(), 36);
    // Other position: 12 × 3 = 36 floats
    assert_eq!(edge_geom.other_positions.len(), 36);
    // Edge offset: 12 floats
    assert_eq!(edge_geom.edge_offsets.len(), 12);
    // Edge type: 12 floats
    assert_eq!(edge_geom.edge_types.len(), 12);
    // Face normals A: 12 × 3 = 36 floats
    assert_eq!(edge_geom.face_normals_a.len(), 36);
    // Face normals B: 12 × 3 = 36 floats
    assert_eq!(edge_geom.face_normals_b.len(), 36);
}

#[test]
fn test_decodes_packed_silhouette_normals_byte_short_float() {
    // 1. Byte (Int8)
    let byte_normals = EdgeVisibilityData::normals_from_i8(&[10, 10, 126, 10, 126, 10]);
    let prim_byte = create_test_primitive(Some(byte_normals));
    let mut mock_byte = MockRenderResources::new();
    let mut rr_byte = mock_byte.as_render_resources();
    EdgeVisibilityPipelineStage::process(&mut rr_byte, &prim_byte);
    let geom_byte = rr_byte.edge_geometry.expect("Byte geom must exist");

    // 2. Short (Int16)
    let short_normals = EdgeVisibilityData::normals_from_i16(&[2580, 2580, 32508, 2580, 32508, 2580]);
    let prim_short = create_test_primitive(Some(short_normals));
    let mut mock_short = MockRenderResources::new();
    let mut rr_short = mock_short.as_render_resources();
    EdgeVisibilityPipelineStage::process(&mut rr_short, &prim_short);
    let geom_short = rr_short.edge_geometry.expect("Short geom must exist");

    // 3. Float32
    let hypot = (10.0f32 * 10.0 + 10.0 * 10.0 + 126.0 * 126.0).sqrt();
    let float_normals = vec![
        10.0 / hypot, 10.0 / hypot, 126.0 / hypot,
        10.0 / hypot, 126.0 / hypot, 10.0 / hypot,
    ];
    let prim_float = create_test_primitive(Some(float_normals));
    let mut mock_float = MockRenderResources::new();
    let mut rr_float = mock_float.as_render_resources();
    EdgeVisibilityPipelineStage::process(&mut rr_float, &prim_float);
    let geom_float = rr_float.edge_geometry.expect("Float geom must exist");

    let small_comp = 10.0 / hypot;
    let large_comp = 126.0 / hypot;

    // Check that all 3 decode identically normalized normals for the silhouette edge
    for geom in &[geom_byte, geom_short, geom_float] {
        let mut silhouette_edge_count = 0;
        for edge in 0..3 {
            let base = edge * 12; // 4 vertices per edge × 3 floats
            let ax = geom.face_normals_a[base];
            let ay = geom.face_normals_a[base + 1];
            let az = geom.face_normals_a[base + 2];
            let bx = geom.face_normals_b[base];
            let by = geom.face_normals_b[base + 1];
            let bz = geom.face_normals_b[base + 2];

            if ax == 0.0 && ay == 0.0 && az == 0.0 {
                continue;
            }
            silhouette_edge_count += 1;

            assert!((ax - small_comp).abs() < 1e-4);
            assert!((ay - small_comp).abs() < 1e-4);
            assert!((az - large_comp).abs() < 1e-4);

            assert!((bx - small_comp).abs() < 1e-4);
            assert!((by - large_comp).abs() < 1e-4);
            assert!((bz - small_comp).abs() < 1e-4);

            let mag_a = (ax * ax + ay * ay + az * az).sqrt();
            let mag_b = (bx * bx + by * by + bz * bz).sqrt();
            assert!((mag_a - 1.0).abs() < 1e-5);
            assert!((mag_b - 1.0).abs() < 1e-5);
        }
        assert_eq!(silhouette_edge_count, 1);
    }
}

#[test]
fn test_does_not_throw_for_degenerate_zero_area_triangles() {
    let positions = vec![
        0.0, 0.0, 0.0, // vertex 0
        1.0, 0.0, 0.0, // vertex 1
        1.0, 0.0, 0.0, // vertex 2 == vertex 1 (degenerate)
    ];
    let indices = vec![0, 1, 2];
    let visibility_buffer = vec![0b00101010u8]; // 3 Hard edges

    let primitive = EdgeVisibilityPrimitive {
        positions,
        indices,
        edge_visibility: Some(EdgeVisibilityData {
            visibility: Some(visibility_buffer),
            silhouette_normals: None,
            material_color: None,
            line_strings: Vec::new(),
        }),
        vertex_colors: None,
        cumulative_distances: None,
    };

    let mut mock = MockRenderResources::new();
    let mut rr = mock.as_render_resources();

    // Must not panic
    EdgeVisibilityPipelineStage::process(&mut rr, &primitive);
    assert!(rr.edge_geometry.is_some());
}

#[test]
fn test_validates_bentley_materials_line_style_support() {
    let primitive = create_test_primitive(None);
    let mut mock = MockRenderResources::new();
    let mut rr = mock.as_render_resources();

    EdgeVisibilityPipelineStage::process(&mut rr, &primitive);
    let edge_geom = rr.edge_geometry.expect("Edge geometry must exist");

    assert_eq!(edge_geom.primitive_type, PrimitiveType::Triangles);
    assert_eq!(edge_geom.index_count, 18);
}
