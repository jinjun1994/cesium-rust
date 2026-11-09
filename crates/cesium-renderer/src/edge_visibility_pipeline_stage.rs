//! EdgeVisibilityPipelineStage matching CesiumJS `Scene/Model/EdgeVisibilityPipelineStage.js` (Apache-2.0).

use crate::commands::Pass;
use crate::shader_builder::{ShaderBuilder, ShaderDestination};
use cesium_math::{Cartesian4, PrimitiveType};
use std::collections::{BTreeSet, HashMap};

/// 2-bit edge visibility types defined by EXT_mesh_primitive_edge_visibility.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum EdgeVisibilityType {
    Hidden = 0,
    Silhouette = 1,
    Hard = 2,
    RepeatedHard = 3,
}

impl EdgeVisibilityType {
    pub fn from_raw(val: u8) -> Self {
        match val & 0x3 {
            0 => Self::Hidden,
            1 => Self::Silhouette,
            2 => Self::Hard,
            3 => Self::RepeatedHard,
            _ => unreachable!(),
        }
    }
}

/// Mode defining how edges are rendered relative to surface geometry.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum EdgeDisplayMode {
    #[default]
    SurfacesOnly = 0,
    SurfacesAndEdges = 1,
    EdgesOnly = 2,
}

/// Line string representation within edge visibility extension.
#[derive(Clone, Debug, PartialEq)]
pub struct EdgeVisibilityLineString {
    pub indices: Vec<u32>,
    pub restart_index: Option<u32>,
    pub material_color: Option<Cartesian4>,
}

/// Edge visibility extension data on a primitive.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct EdgeVisibilityData {
    pub visibility: Option<Vec<u8>>,
    pub silhouette_normals: Option<Vec<f32>>,
    pub material_color: Option<Cartesian4>,
    pub line_strings: Vec<EdgeVisibilityLineString>,
}

impl EdgeVisibilityData {
    /// Creates silhouette normals from signed 8-bit components.
    pub fn normals_from_i8(raw: &[i8]) -> Vec<f32> {
        raw.iter().map(|&b| b as f32).collect()
    }

    /// Creates silhouette normals from signed 16-bit components.
    pub fn normals_from_i16(raw: &[i16]) -> Vec<f32> {
        raw.iter().map(|&s| s as f32).collect()
    }
}

/// Single edge metadata.
#[derive(Clone, Debug, PartialEq)]
pub struct EdgeDataEntry {
    pub edge_type: EdgeVisibilityType,
    pub silhouette_edge_index: i32,
    pub color: Option<Cartesian4>,
}

/// Result of extracting visible edges.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct VisibleEdgesResult {
    pub edge_indices: Vec<u32>,
    pub edge_data: Vec<EdgeDataEntry>,
    pub silhouette_edge_count: usize,
}

/// Primitive descriptor for EdgeVisibilityPipelineStage.
#[derive(Clone, Debug, Default)]
pub struct EdgeVisibilityPrimitive {
    pub positions: Vec<f32>,
    pub indices: Vec<u32>,
    pub edge_visibility: Option<EdgeVisibilityData>,
    pub vertex_colors: Option<Vec<f32>>,
    pub cumulative_distances: Option<Vec<f32>>,
}

/// Derived edge quad geometry.
#[derive(Clone, Debug, PartialEq)]
pub struct EdgeGeometry {
    pub positions: Vec<f32>,
    pub other_positions: Vec<f32>,
    pub edge_offsets: Vec<f32>,
    pub edge_types: Vec<f32>,
    pub face_normals_a: Vec<f32>,
    pub face_normals_b: Vec<f32>,
    pub edge_colors: Option<Vec<f32>>,
    pub cumulative_distances: Option<Vec<f32>>,
    pub indices: Vec<u32>,
    pub index_count: usize,
    pub primitive_type: PrimitiveType,
    pub pass: Pass,
    pub line_width: Option<f32>,
}

/// Render resources for EdgeVisibilityPipelineStage.
pub struct EdgeVisibilityRenderResources<'a> {
    pub shader_builder: &'a mut ShaderBuilder,
    pub uniform_map: &'a mut HashMap<String, Box<dyn Fn() -> serde_json::Value + Send + Sync>>,
    pub edge_geometry: Option<EdgeGeometry>,
}

/// Pipeline stage parsing EXT_mesh_primitive_edge_visibility.
pub struct EdgeVisibilityPipelineStage;

impl EdgeVisibilityPipelineStage {
    pub const NAME: &'static str = "EdgeVisibilityPipelineStage";

    /// Processes a primitive to derive edge geometry and shader bindings.
    pub fn process(
        render_resources: &mut EdgeVisibilityRenderResources<'_>,
        primitive: &EdgeVisibilityPrimitive,
    ) {
        let Some(ref edge_visibility) = primitive.edge_visibility else {
            return;
        };

        let sb = &mut render_resources.shader_builder;
        sb.add_define("HAS_EDGE_VISIBILITY", None, Some(ShaderDestination::BOTH));
        sb.add_define("HAS_EDGE_VISIBILITY_MRT", None, Some(ShaderDestination::FRAGMENT));

        sb.add_fragment_lines(&[
            "layout(location = 1) out vec4 out_id;",
            "layout(location = 2) out vec4 out_edgeDepth;",
        ]);

        sb.add_uniform("bool", "u_isEdgePass", Some(ShaderDestination::BOTH));
        sb.add_uniform("float", "u_lineWidth", Some(ShaderDestination::VERTEX));

        sb.add_attribute("float", "a_edgeType");
        sb.add_varying("float", "v_edgeType", Some("flat"));

        sb.add_attribute("float", "a_edgeFeatureId");
        sb.add_attribute("vec3", "a_faceNormalA");
        sb.add_attribute("vec3", "a_faceNormalB");
        sb.add_varying("vec3", "v_faceNormalAView", Some("flat"));
        sb.add_varying("vec3", "v_faceNormalBView", Some("flat"));
        sb.add_varying("float", "v_shouldDiscard", Some("flat"));

        sb.add_attribute("vec3", "a_edgeOtherPos");
        sb.add_attribute("float", "a_edgeOffset");
        sb.add_varying("float", "v_edgeOffset", None);

        let edge_result = extract_visible_edges(primitive);
        if edge_result.edge_indices.is_empty() {
            return;
        }

        let has_edge_color_override = edge_result.edge_data.iter().any(|e| e.color.is_some());
        let has_vertex_colors = primitive.vertex_colors.is_some();
        let needs_edge_color = has_edge_color_override || has_vertex_colors;

        if needs_edge_color {
            sb.add_attribute("vec4", "a_edgeColor");
            sb.add_varying("vec4", "v_edgeColor", Some("flat"));
            sb.add_define("HAS_EDGE_COLOR_ATTRIBUTE", None, Some(ShaderDestination::BOTH));
        }

        if primitive.cumulative_distances.is_some() {
            sb.add_attribute("float", "a_edgeCumulativeDistance");
            sb.add_define("HAS_EDGE_CUMULATIVE_DISTANCE", None, Some(ShaderDestination::VERTEX));
        }

        let edge_face_normals = generate_edge_face_normals(&edge_result, edge_visibility);

        let edge_geometry = create_quad_edge_geometry(
            primitive,
            &edge_result,
            &edge_face_normals,
            needs_edge_color,
        );

        render_resources.edge_geometry = Some(edge_geometry);

        render_resources
            .uniform_map
            .insert("u_isEdgePass".to_string(), Box::new(|| serde_json::json!(false)));
        render_resources
            .uniform_map
            .insert("u_lineWidth".to_string(), Box::new(|| serde_json::json!(1.0)));
    }
}

pub fn extract_visible_edges(primitive: &EdgeVisibilityPrimitive) -> VisibleEdgesResult {
    let Some(ref edge_visibility) = primitive.edge_visibility else {
        return VisibleEdgesResult::default();
    };

    let mut edge_indices = Vec::new();
    let mut edge_data = Vec::new();
    let mut seen_edge_hashes = BTreeSet::new();
    let mut silhouette_edge_count = 0;
    let global_color = edge_visibility.material_color;

    if let (Some(ref visibility), true) = (&edge_visibility.visibility, !primitive.indices.is_empty()) {
        let indices = &primitive.indices;
        let mut edge_index = 0;
        let total_indices = indices.len();

        let mut i = 0;
        while i + 2 < total_indices {
            let v0 = indices[i];
            let v1 = indices[i + 1];
            let v2 = indices[i + 2];

            let tri_edges = [(v0, v1), (v1, v2), (v2, v0)];
            for (a, b) in tri_edges {
                let byte_index = edge_index / 4;
                let bit_pair_offset = (edge_index % 4) * 2;
                edge_index += 1;

                if byte_index >= visibility.len() {
                    break;
                }

                let byte = visibility[byte_index];
                let raw_type = (byte >> bit_pair_offset) & 0x3;
                let vis_type = EdgeVisibilityType::from_raw(raw_type);

                if vis_type == EdgeVisibilityType::Hidden {
                    continue;
                }

                let small = a.min(b);
                let big = a.max(b);
                let edge_key = (small, big);

                if seen_edge_hashes.contains(&edge_key) {
                    continue;
                }
                seen_edge_hashes.insert(edge_key);

                edge_indices.push(a);
                edge_indices.push(b);

                let silhouette_edge_index = if vis_type == EdgeVisibilityType::Silhouette {
                    let idx = silhouette_edge_count as i32;
                    silhouette_edge_count += 1;
                    idx
                } else {
                    -1
                };

                let normalized_type = if vis_type == EdgeVisibilityType::RepeatedHard {
                    EdgeVisibilityType::Hard
                } else {
                    vis_type
                };

                edge_data.push(EdgeDataEntry {
                    edge_type: normalized_type,
                    silhouette_edge_index,
                    color: global_color,
                });
            }
            i += 3;
        }
    }

    // Process line_strings
    for ls in &edge_visibility.line_strings {
        if ls.indices.len() < 2 {
            continue;
        }
        let line_color = ls.material_color.or(global_color);
        let mut prev: Option<u32> = None;
        for &curr in &ls.indices {
            if let Some(restart) = ls.restart_index {
                if curr == restart {
                    prev = None;
                    continue;
                }
            }

            if let Some(p) = prev {
                if p != curr {
                    let small = p.min(curr);
                    let big = p.max(curr);
                    let edge_key = (small, big);
                    if !seen_edge_hashes.contains(&edge_key) {
                        seen_edge_hashes.insert(edge_key);
                        edge_indices.push(p);
                        edge_indices.push(curr);
                        edge_data.push(EdgeDataEntry {
                            edge_type: EdgeVisibilityType::Hard,
                            silhouette_edge_index: -1,
                            color: line_color,
                        });
                    }
                }
            }
            prev = Some(curr);
        }
    }

    VisibleEdgesResult {
        edge_indices,
        edge_data,
        silhouette_edge_count,
    }
}

fn generate_edge_face_normals(
    result: &VisibleEdgesResult,
    edge_visibility: &EdgeVisibilityData,
) -> Vec<f32> {
    let num_edges = result.edge_data.len();
    let mut edge_face_normals = vec![0.0f32; num_edges * 6];

    let Some(ref raw_normals) = edge_visibility.silhouette_normals else {
        return edge_face_normals;
    };

    let normal_count = raw_normals.len() / 3;
    let mut normalized_floats = vec![0.0f32; raw_normals.len()];

    for i in 0..normal_count {
        let x = raw_normals[i * 3];
        let y = raw_normals[i * 3 + 1];
        let z = raw_normals[i * 3 + 2];
        let mag = (x * x + y * y + z * z).sqrt();
        if mag > 0.0 {
            normalized_floats[i * 3] = x / mag;
            normalized_floats[i * 3 + 1] = y / mag;
            normalized_floats[i * 3 + 2] = z / mag;
        } else {
            normalized_floats[i * 3] = 0.0;
            normalized_floats[i * 3 + 1] = 0.0;
            normalized_floats[i * 3 + 2] = 1.0;
        }
    }

    for (i, edge) in result.edge_data.iter().enumerate() {
        if edge.edge_type != EdgeVisibilityType::Silhouette || edge.silhouette_edge_index < 0 {
            continue;
        }

        let pair_base = edge.silhouette_edge_index as usize * 2;
        if (pair_base + 1) * 3 + 2 >= normalized_floats.len() {
            continue;
        }

        let base_a = pair_base * 3;
        let base_b = (pair_base + 1) * 3;
        let base_idx = i * 6;

        edge_face_normals[base_idx] = normalized_floats[base_a];
        edge_face_normals[base_idx + 1] = normalized_floats[base_a + 1];
        edge_face_normals[base_idx + 2] = normalized_floats[base_a + 2];
        edge_face_normals[base_idx + 3] = normalized_floats[base_b];
        edge_face_normals[base_idx + 4] = normalized_floats[base_b + 1];
        edge_face_normals[base_idx + 5] = normalized_floats[base_b + 2];
    }

    edge_face_normals
}

fn create_quad_edge_geometry(
    primitive: &EdgeVisibilityPrimitive,
    edge_result: &VisibleEdgesResult,
    edge_face_normals: &[f32],
    needs_edge_color: bool,
) -> EdgeGeometry {
    let num_edges = edge_result.edge_data.len();
    let verts_per_edge = 4;
    let total_verts = num_edges * verts_per_edge;

    let src_pos = &primitive.positions;

    let mut edge_pos = vec![0.0f32; total_verts * 3];
    let mut edge_other_pos = vec![0.0f32; total_verts * 3];
    let mut edge_offset = vec![0.0f32; total_verts];
    let mut edge_type = vec![0.0f32; total_verts];
    let mut face_normals_a = vec![0.0f32; total_verts * 3];
    let mut face_normals_b = vec![0.0f32; total_verts * 3];
    let mut edge_colors = if needs_edge_color {
        Some(vec![0.0f32; total_verts * 4])
    } else {
        None
    };
    let mut edge_cum_dist = if primitive.cumulative_distances.is_some() {
        Some(vec![0.0f32; total_verts])
    } else {
        None
    };

    for i in 0..num_edges {
        let a = edge_result.edge_indices[i * 2] as usize;
        let b = edge_result.edge_indices[i * 2 + 1] as usize;
        let raw_type = edge_result.edge_data[i].edge_type as u8;
        let normalized_type = (raw_type as f32) / 255.0;

        let ax = src_pos[a * 3];
        let ay = src_pos[a * 3 + 1];
        let az = src_pos[a * 3 + 2];
        let bx = src_pos[b * 3];
        let by = src_pos[b * 3 + 1];
        let bz = src_pos[b * 3 + 2];

        let base_v = i * 4;

        // V0: pos A, other B, offset -1.0
        edge_pos[base_v * 3] = ax;
        edge_pos[base_v * 3 + 1] = ay;
        edge_pos[base_v * 3 + 2] = az;
        edge_other_pos[base_v * 3] = bx;
        edge_other_pos[base_v * 3 + 1] = by;
        edge_other_pos[base_v * 3 + 2] = bz;
        edge_offset[base_v] = -1.0;
        edge_type[base_v] = normalized_type;

        // V1: pos A, other B, offset +1.0
        edge_pos[(base_v + 1) * 3] = ax;
        edge_pos[(base_v + 1) * 3 + 1] = ay;
        edge_pos[(base_v + 1) * 3 + 2] = az;
        edge_other_pos[(base_v + 1) * 3] = bx;
        edge_other_pos[(base_v + 1) * 3 + 1] = by;
        edge_other_pos[(base_v + 1) * 3 + 2] = bz;
        edge_offset[base_v + 1] = 1.0;
        edge_type[base_v + 1] = normalized_type;

        // V2: pos B, other A, offset +1.0
        edge_pos[(base_v + 2) * 3] = bx;
        edge_pos[(base_v + 2) * 3 + 1] = by;
        edge_pos[(base_v + 2) * 3 + 2] = bz;
        edge_other_pos[(base_v + 2) * 3] = ax;
        edge_other_pos[(base_v + 2) * 3 + 1] = ay;
        edge_other_pos[(base_v + 2) * 3 + 2] = az;
        edge_offset[base_v + 2] = 1.0;
        edge_type[base_v + 2] = normalized_type;

        // V3: pos B, other A, offset -1.0
        edge_pos[(base_v + 3) * 3] = bx;
        edge_pos[(base_v + 3) * 3 + 1] = by;
        edge_pos[(base_v + 3) * 3 + 2] = bz;
        edge_other_pos[(base_v + 3) * 3] = ax;
        edge_other_pos[(base_v + 3) * 3 + 1] = ay;
        edge_other_pos[(base_v + 3) * 3 + 2] = az;
        edge_offset[base_v + 3] = -1.0;
        edge_type[base_v + 3] = normalized_type;

        if let Some(ref mut colors) = edge_colors {
            if let Some(c) = edge_result.edge_data[i].color {
                for v in 0..4 {
                    let d = (base_v + v) * 4;
                    colors[d] = c.x as f32;
                    colors[d + 1] = c.y as f32;
                    colors[d + 2] = c.z as f32;
                    colors[d + 3] = c.w as f32;
                }
            } else if let Some(ref vc) = primitive.vertex_colors {
                let a_color = if a * 4 + 3 < vc.len() {
                    [vc[a * 4], vc[a * 4 + 1], vc[a * 4 + 2], vc[a * 4 + 3]]
                } else {
                    [0.0, 0.0, 0.0, -1.0]
                };
                for v in 0..4 {
                    let d = (base_v + v) * 4;
                    colors[d] = a_color[0];
                    colors[d + 1] = a_color[1];
                    colors[d + 2] = a_color[2];
                    colors[d + 3] = a_color[3];
                }
            } else {
                for v in 0..4 {
                    let d = (base_v + v) * 4;
                    colors[d] = 0.0;
                    colors[d + 1] = 0.0;
                    colors[d + 2] = 0.0;
                    colors[d + 3] = -1.0;
                }
            }
        }

        if let Some(ref mut cdist) = edge_cum_dist {
            if let Some(ref sdist) = primitive.cumulative_distances {
                let da = if a < sdist.len() { sdist[a] } else { 0.0 };
                let db = if b < sdist.len() { sdist[b] } else { 0.0 };
                cdist[base_v] = da;
                cdist[base_v + 1] = da;
                cdist[base_v + 2] = db;
                cdist[base_v + 3] = db;
            }
        }

        let normal_idx = i * 6;
        let nax = edge_face_normals[normal_idx];
        let nay = edge_face_normals[normal_idx + 1];
        let naz = edge_face_normals[normal_idx + 2];
        let nbx = edge_face_normals[normal_idx + 3];
        let nby = edge_face_normals[normal_idx + 4];
        let nbz = edge_face_normals[normal_idx + 5];

        for v in 0..4 {
            let d = (base_v + v) * 3;
            face_normals_a[d] = nax;
            face_normals_a[d + 1] = nay;
            face_normals_a[d + 2] = naz;
            face_normals_b[d] = nbx;
            face_normals_b[d + 1] = nby;
            face_normals_b[d + 2] = nbz;
        }
    }

    let num_triangles = num_edges * 2;
    let num_indices = num_triangles * 3;
    let mut indices = vec![0u32; num_indices];

    for i in 0..num_edges {
        let base_v = (i * 4) as u32;
        let base_idx = i * 6;

        // Tri 1: (v0, v1, v2)
        indices[base_idx] = base_v;
        indices[base_idx + 1] = base_v + 1;
        indices[base_idx + 2] = base_v + 2;

        // Tri 2: (v0, v2, v3)
        indices[base_idx + 3] = base_v;
        indices[base_idx + 4] = base_v + 2;
        indices[base_idx + 5] = base_v + 3;
    }

    EdgeGeometry {
        positions: edge_pos,
        other_positions: edge_other_pos,
        edge_offsets: edge_offset,
        edge_types: edge_type,
        face_normals_a,
        face_normals_b,
        edge_colors,
        cumulative_distances: edge_cum_dist,
        indices,
        index_count: num_indices,
        primitive_type: PrimitiveType::Triangles,
        pass: Pass::Cesium3dTileEdges,
        line_width: Some(1.0),
    }
}

/// Fragment shader snippet for edge detection in surface pass.
pub const EDGE_DETECTION_STAGE_FS: &str = r#"
void edgeDetectionStage(inout vec4 color, inout FeatureIds featureIds) {
    if (u_isEdgePass) {
        return;
    }

    vec2 screenCoord = gl_FragCoord.xy / czm_viewport.zw;

    vec4 edgeColor = texture(czm_edgeColorTexture, screenCoord);
    vec4 edgeId = texture(czm_edgeIdTexture, screenCoord);

    // Packed window-space depth from edge pass (0..1)
    float edgeDepthWin = czm_unpackDepth(texture(czm_edgeDepthTexture, screenCoord));

    // Near / far for current frustum
    float n = czm_currentFrustum.x;
    float f = czm_currentFrustum.y;

    // geometry depth in eye coordinate
    vec4 geomEC = czm_windowToEyeCoordinates(gl_FragCoord);
    float geomDepthLinear = -geomEC.z;

    // Convert edge depth to linear depth
    float z_ndc_edge = edgeDepthWin * 2.0 - 1.0;
    float edgeDepthLinear = (2.0 * n * f) / (f + n - z_ndc_edge * (f - n));

    float d = abs(edgeDepthLinear - geomDepthLinear);

    // Adaptive epsilon using linear depth fwidth for robustness
    float pixelStepLinear = fwidth(geomDepthLinear);
    float rel = geomDepthLinear * 0.0005;
    float eps = max(n * 1e-4, max(pixelStepLinear * 1.5, rel));

    // If Edge isn't behind any geometry and the pixel has edge data
    if (d < eps && edgeId.r > 0.0) {
#ifdef HAS_EDGE_FEATURE_ID
        float edgeFeatureId    = edgeId.g;
        float currentFeatureId = float(featureIds.featureId_0);
#endif
        float globeDepth = czm_unpackDepth(texture(czm_globeDepthTexture, screenCoord));
        // Background / sky / globe: always show edge
        bool isBackground = geomDepthLinear > globeDepth;
        bool drawEdge = isBackground;

#ifdef HAS_EDGE_FEATURE_ID
        bool hasEdgeFeature = edgeFeatureId > 0.0;
        bool hasCurrentFeature = currentFeatureId > 0.0;
        bool featuresMatch = edgeFeatureId == currentFeatureId;

        drawEdge = drawEdge || !hasEdgeFeature || !hasCurrentFeature || featuresMatch;
#else
        drawEdge = true;
#endif

        if (drawEdge) {
            color = edgeColor;
        }
    }
}
"#;

/// Stage injecting fragment shader logic for edge detection and screen-space composition.
pub struct EdgeDetectionPipelineStage;

impl EdgeDetectionPipelineStage {
    pub const NAME: &'static str = "EdgeDetectionPipelineStage";

    pub fn process(render_resources: &mut EdgeVisibilityRenderResources<'_>) {
        render_resources
            .shader_builder
            .add_fragment_lines(&[EDGE_DETECTION_STAGE_FS]);
    }
}

/// Edge draw command representation holding shader text, pass and uniforms.
#[derive(Clone, Debug)]
pub struct EdgeRenderCommand {
    pub pass: Pass,
    pub vertex_shader_text: String,
    pub fragment_shader_text: String,
    pub is_edge_pass: bool,
    pub is_destroyed: bool,
}

impl EdgeRenderCommand {
    pub fn destroy(&mut self) {
        self.is_destroyed = true;
    }
}

/// Simulated model managing multi-pass edge commands and snapping.
#[derive(Clone, Debug)]
pub struct EdgeRenderingModel {
    pub edge_display_mode: EdgeDisplayMode,
    pub edge_geometry_needed: bool,
    pub edge_geometry_needed_for_snapping: bool,
    pub primitive: EdgeVisibilityPrimitive,
    pub edge_commands: Vec<EdgeRenderCommand>,
    pub surface_commands: Vec<EdgeRenderCommand>,
    pub is_destroyed: bool,
}

impl EdgeRenderingModel {
    pub fn new(primitive: EdgeVisibilityPrimitive, mode: EdgeDisplayMode) -> Self {
        Self {
            edge_display_mode: mode,
            edge_geometry_needed: false,
            edge_geometry_needed_for_snapping: false,
            primitive,
            edge_commands: Vec::new(),
            surface_commands: Vec::new(),
            is_destroyed: false,
        }
    }

    pub fn reset_draw_commands(&mut self) {
        for cmd in &mut self.edge_commands {
            cmd.destroy();
        }
        self.edge_commands.clear();
        self.surface_commands.clear();
        self.edge_geometry_needed = false;
    }

    pub fn destroy(&mut self) {
        self.reset_draw_commands();
        self.is_destroyed = true;
    }

    pub fn update(
        &mut self,
        snap_pass: bool,
        edge_visibility_requested: &mut bool,
        enable_edge_visibility: &mut bool,
        command_list: &mut Vec<EdgeRenderCommand>,
    ) {
        if snap_pass {
            self.edge_geometry_needed_for_snapping = true;
        }

        let needs_edges = self.edge_display_mode != EdgeDisplayMode::SurfacesOnly
            || self.edge_geometry_needed_for_snapping;

        self.edge_geometry_needed = needs_edges;

        if needs_edges && self.edge_commands.is_empty() {
            let mut sb = ShaderBuilder::default();
            let mut um = HashMap::new();
            let mut rr = EdgeVisibilityRenderResources {
                shader_builder: &mut sb,
                uniform_map: &mut um,
                edge_geometry: None,
            };
            EdgeVisibilityPipelineStage::process(&mut rr, &self.primitive);

            let pass = match self.edge_display_mode {
                EdgeDisplayMode::EdgesOnly => Pass::Cesium3dTileEdgesDirect,
                _ => Pass::Cesium3dTileEdges,
            };

            let vs_source = rr.shader_builder.create_vertex_shader_source();
            let mut vs_parts = vs_source
                .defines
                .iter()
                .map(|d| format!("#define {}\n", d))
                .collect::<Vec<_>>();
            vs_parts.extend(vs_source.sources);
            let vs = vs_parts.join("\n");

            let fs_source = rr.shader_builder.create_fragment_shader_source();
            let mut fs_parts = fs_source
                .defines
                .iter()
                .map(|d| format!("#define {}\n", d))
                .collect::<Vec<_>>();
            fs_parts.extend(fs_source.sources);
            fs_parts.push("layout(location = 1) out vec4 out_id;".to_string());
            fs_parts.push("layout(location = 2) out vec4 out_edgeDepth;".to_string());
            let fs = fs_parts.join("\n");

            let edge_cmd = EdgeRenderCommand {
                pass,
                vertex_shader_text: vs,
                fragment_shader_text: fs,
                is_edge_pass: true,
                is_destroyed: false,
            };
            self.edge_commands.push(edge_cmd);
        }

        if self.surface_commands.is_empty() && self.edge_display_mode != EdgeDisplayMode::EdgesOnly {
            let mut sb = ShaderBuilder::default();
            let mut um = HashMap::new();
            let mut rr = EdgeVisibilityRenderResources {
                shader_builder: &mut sb,
                uniform_map: &mut um,
                edge_geometry: None,
            };
            EdgeDetectionPipelineStage::process(&mut rr);

            let vs = rr.shader_builder.create_vertex_shader_source().sources.join("\n");
            let fs = rr.shader_builder.create_fragment_shader_source().sources.join("\n");

            let surf_cmd = EdgeRenderCommand {
                pass: Pass::Cesium3dTile,
                vertex_shader_text: vs,
                fragment_shader_text: fs,
                is_edge_pass: false,
                is_destroyed: false,
            };
            self.surface_commands.push(surf_cmd);
        }

        // Update frame states and collect commands
        match self.edge_display_mode {
            EdgeDisplayMode::SurfacesOnly => {
                *edge_visibility_requested = false;
                for cmd in &self.surface_commands {
                    command_list.push(cmd.clone());
                }
            }
            EdgeDisplayMode::SurfacesAndEdges => {
                *edge_visibility_requested = true;
                *enable_edge_visibility = true;
                for cmd in &self.surface_commands {
                    command_list.push(cmd.clone());
                }
                for cmd in &self.edge_commands {
                    command_list.push(cmd.clone());
                }
            }
            EdgeDisplayMode::EdgesOnly => {
                *edge_visibility_requested = false;
                for cmd in &self.edge_commands {
                    command_list.push(cmd.clone());
                }
            }
        }
    }
}
