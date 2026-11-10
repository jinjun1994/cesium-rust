//! SceneMode2DPipelineStage matching CesiumJS `Scene/Model/SceneMode2DPipelineStage.js` (Apache-2.0).

use cesium_math::{
    BoundingSphere, Cartesian3, Ellipsoid, GeographicProjection, MapProjection, Matrix4, SceneMode,
};

use crate::buffer::{Buffer, BufferOptions, BufferUsage};
use crate::model_components::{Attribute, Primitive};
use crate::model_statistics::ModelStatistics;
use crate::model_type::ModelType;
use crate::shader_builder::{ShaderBuilder, ShaderDestination};
use crate::vertex_array::VertexAttribute;

/// Frame state requirements for 2D scene mode pipeline processing.
#[derive(Clone, Debug)]
pub struct SceneMode2DFrameState {
    pub mode: SceneMode,
    pub view: Matrix4,
    pub map_projection: GeographicProjection,
}

impl Default for SceneMode2DFrameState {
    fn default() -> Self {
        Self {
            mode: SceneMode::Scene2D,
            view: Matrix4::IDENTITY,
            map_projection: GeographicProjection::new(Ellipsoid::default()).unwrap_or_default(),
        }
    }
}

/// Standalone model state for 2D pipeline stage.
#[derive(Debug, Default)]
pub struct MockSceneMode2DModel {
    pub model_type: ModelType,
    pub statistics: ModelStatistics,
    pub computed_model_matrix: Matrix4,
    pub model_resources: Vec<Buffer>,
}

/// Standalone runtime node state for 2D pipeline stage.
#[derive(Debug, Default)]
pub struct MockSceneMode2DRuntimeNode {
    pub computed_transform: Matrix4,
    pub is_instanced: bool,
}

/// Standalone runtime primitive state for 2D pipeline stage.
#[derive(Debug, Default)]
pub struct MockSceneMode2DRuntimePrimitive {
    pub bounding_sphere_2d: Option<BoundingSphere>,
    pub position_buffer_2d: Option<Buffer>,
}

/// Standalone render resources matching CesiumJS `PrimitiveRenderResources` in 2D mode.
#[derive(Debug, Default)]
pub struct MockSceneMode2DRenderResources {
    pub attributes: Vec<VertexAttribute>,
    pub shader_builder: ShaderBuilder,
    pub attribute_index: u32,
    pub model: MockSceneMode2DModel,
    pub runtime_node: MockSceneMode2DRuntimeNode,
    pub runtime_primitive: MockSceneMode2DRuntimePrimitive,
    pub position_min: Cartesian3,
    pub position_max: Cartesian3,
    pub u_model_view_2d: Option<Matrix4>,
}

/// The scene mode 2D stage generates resources for rendering a primitive in 2D / CV mode.
pub struct SceneMode2DPipelineStage;

impl SceneMode2DPipelineStage {
    pub const NAME: &'static str = "SceneMode2DPipelineStage";

    /// Processes a primitive for 2D rendering.
    pub fn process(
        render_resources: &mut MockSceneMode2DRenderResources,
        primitive: &mut Primitive,
        frame_state: &SceneMode2DFrameState,
    ) {
        let computed_model_matrix = render_resources
            .model
            .computed_model_matrix
            .multiply(&render_resources.runtime_node.computed_transform);

        let bounding_sphere_2d = compute_bounding_sphere_2d(
            render_resources.position_min,
            render_resources.position_max,
            &computed_model_matrix,
            frame_state,
        );

        render_resources.runtime_primitive.bounding_sphere_2d = Some(bounding_sphere_2d);

        // If the model is instanced, 2D projection will be handled in the InstancingPipelineStage.
        if render_resources.runtime_node.is_instanced {
            // Unload original position typedArray as CesiumJS does
            if let Some(pos_attr) = primitive
                .attributes
                .iter_mut()
                .find(|a| a.semantic.as_deref() == Some("POSITION"))
            {
                pos_attr.typed_array = None;
            }
            return;
        }

        let mut created_buffer_2d = None;
        if let Some(pos_attr) = primitive
            .attributes
            .iter_mut()
            .find(|a| a.semantic.as_deref() == Some("POSITION"))
        {
            if let Some(ref typed_array) = pos_attr.typed_array {
                let buffer_2d = create_position_buffer_for_2d(
                    typed_array,
                    pos_attr,
                    &computed_model_matrix,
                    &bounding_sphere_2d,
                    frame_state,
                );

                render_resources.runtime_primitive.position_buffer_2d = Some(buffer_2d.clone());
                render_resources.model.model_resources.push(buffer_2d.clone());
                created_buffer_2d = Some(buffer_2d);

                pos_attr.typed_array = None;
            }
        }

        let _ = created_buffer_2d;

        render_resources.shader_builder.add_define(
            "USE_2D_POSITIONS",
            None,
            Some(ShaderDestination::VERTEX),
        );

        render_resources.shader_builder.add_uniform(
            "mat4",
            "u_modelView2D",
            Some(ShaderDestination::VERTEX),
        );

        let model_matrix_2d = Matrix4::from_translation(bounding_sphere_2d.center.into());
        let u_model_view_2d = frame_state.view.multiply(&model_matrix_2d);
        render_resources.u_model_view_2d = Some(u_model_view_2d);
    }
}

fn compute_bounding_sphere_2d(
    position_min: Cartesian3,
    position_max: Cartesian3,
    model_matrix: &Matrix4,
    frame_state: &SceneMode2DFrameState,
) -> BoundingSphere {
    let transformed_min = model_matrix.multiply_by_point(position_min);
    let projected_min = compute_actual_ellipsoid_position(frame_state, transformed_min);

    let transformed_max = model_matrix.multiply_by_point(position_max);
    let projected_max = compute_actual_ellipsoid_position(frame_state, transformed_max);

    BoundingSphere::from_corner_points(projected_min.into(), projected_max.into())
}

fn create_position_buffer_for_2d(
    typed_array: &[u8],
    attr: &Attribute,
    model_matrix: &Matrix4,
    bounding_sphere_2d: &BoundingSphere,
    frame_state: &SceneMode2DFrameState,
) -> Buffer {
    let mut frame_state_cv = frame_state.clone();
    frame_state_cv.mode = SceneMode::ColumbusView;

    let reference_point = bounding_sphere_2d.center;

    // Decode positions to f32
    let mut positions = if let Some(ref quant) = attr.quantization {
        dequantize_positions(typed_array, quant)
    } else {
        bytes_to_f32_vec(typed_array)
    };

    let start_index = attr.byte_offset / 4;
    let stride = attr.byte_stride.map(|s| s / 4).unwrap_or(3);
    let length = positions.len();

    let mut i = start_index;
    while i + 2 < length {
        let initial_position = Cartesian3::new(
            positions[i] as f64,
            positions[i + 1] as f64,
            positions[i + 2] as f64,
        );

        if !initial_position.x.is_nan()
            && !initial_position.y.is_nan()
            && !initial_position.z.is_nan()
        {
            let transformed_position = model_matrix.multiply_by_point(initial_position);
            let projected_position =
                compute_actual_ellipsoid_position(&frame_state_cv, transformed_position);
            let rel = projected_position - reference_point.into();

            positions[i] = rel.x as f32;
            positions[i + 1] = rel.y as f32;
            positions[i + 2] = rel.z as f32;
        }

        i += stride;
    }

    let bytes: Vec<u8> = positions.iter().flat_map(|f| f.to_le_bytes()).collect();

    Buffer::create_vertex_buffer(BufferOptions {
        size_in_bytes: 0,
        usage: Some(BufferUsage::StaticDraw),
        typed_array: Some(bytes),
    })
    .unwrap()
}

fn dequantize_positions(
    typed_array: &[u8],
    quantization: &crate::model_components::Quantization,
) -> Vec<f32> {
    let raw_floats = bytes_to_f32_vec(typed_array);
    let offset = quantization
        .quantized_volume_offset
        .as_deref()
        .unwrap_or(&[0.0, 0.0, 0.0]);
    let step_size = quantization
        .quantized_volume_step_size
        .as_deref()
        .unwrap_or(&[1.0, 1.0, 1.0]);

    let mut out = Vec::with_capacity(raw_floats.len());
    for chunk in raw_floats.as_chunks::<3>().0 {
        let x = chunk[0] as f64 * step_size.first().copied().unwrap_or(1.0)
            + offset.first().copied().unwrap_or(0.0);
        let y = chunk[1] as f64 * step_size.get(1).copied().unwrap_or(1.0)
            + offset.get(1).copied().unwrap_or(0.0);
        let z = chunk[2] as f64 * step_size.get(2).copied().unwrap_or(1.0)
            + offset.get(2).copied().unwrap_or(0.0);

        out.push(x as f32);
        out.push(y as f32);
        out.push(z as f32);
    }
    out
}

pub fn compute_actual_ellipsoid_position(
    frame_state: &SceneMode2DFrameState,
    position: Cartesian3,
) -> Cartesian3 {
    let mode = frame_state.mode;
    if mode == SceneMode::Scene3D {
        return position;
    }

    let projection = &frame_state.map_projection;
    let dvec = glam::DVec3::new(position.x, position.y, position.z);
    let carto = match projection.ellipsoid().cartesian_to_cartographic(dvec) {
        Some(c) => c,
        None => return position,
    };
    let projected = match projection.project(carto) {
        Some(p) => p,
        None => return position,
    };

    match mode {
        SceneMode::ColumbusView => Cartesian3::new(projected.z, projected.x, projected.y),
        SceneMode::Scene2D => Cartesian3::new(0.0, projected.x, projected.y),
        _ => position,
    }
}

fn bytes_to_f32_vec(bytes: &[u8]) -> Vec<f32> {
    bytes
        .as_chunks::<4>()
        .0
        .iter()
        .map(|chunk| f32::from_le_bytes(*chunk))
        .collect()
}
