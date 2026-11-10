//! WireframePipelineStage matching CesiumJS `Scene/Model/WireframePipelineStage.js` (Apache-2.0).

use cesium_math::{IndexBuffer, IndexDatatype, PrimitiveType, WireframeIndexGenerator};

use crate::buffer::{Buffer, BufferUsage, IndexBufferOptions};
use crate::model_components::{Indices, Primitive};
use crate::model_statistics::ModelStatistics;
use crate::shader_builder::{ShaderBuilder, ShaderDestination};

/// Standalone model state for WireframePipelineStage.
#[derive(Debug, Default)]
pub struct MockWireframeModel {
    pub pipeline_resources: Vec<Buffer>,
    pub statistics: ModelStatistics,
}

/// Standalone render resources for WireframePipelineStage.
#[derive(Debug, Default)]
pub struct MockWireframeRenderResources {
    pub indices: Option<Indices>,
    pub count: usize,
    pub primitive_type: Option<PrimitiveType>,
    pub wireframe_index_buffer: Option<Buffer>,
    pub model: MockWireframeModel,
    pub shader_builder: ShaderBuilder,
}

/// Standalone frame state for WireframePipelineStage.
#[derive(Clone, Copy, Debug, Default)]
pub struct WireframeFrameState {
    pub webgl2: bool,
}

/// Trait defining the render resources required by [`WireframePipelineStage`].
pub trait WireframeRenderResources {
    fn shader_builder_mut(&mut self) -> &mut ShaderBuilder;
    fn indices(&self) -> Option<&Indices>;
    fn count_mut(&mut self) -> &mut usize;
    fn count(&self) -> usize;
    fn primitive_type_mut(&mut self) -> &mut Option<PrimitiveType>;
    fn primitive_type(&self) -> Option<PrimitiveType>;
    fn set_wireframe_index_buffer(&mut self, buffer: Buffer);
    fn add_pipeline_resource(&mut self, buffer: Buffer);
    fn add_buffer_to_statistics(&mut self, buffer: &Buffer);
}

impl WireframeRenderResources for MockWireframeRenderResources {
    fn shader_builder_mut(&mut self) -> &mut ShaderBuilder {
        &mut self.shader_builder
    }

    fn indices(&self) -> Option<&Indices> {
        self.indices.as_ref()
    }

    fn count_mut(&mut self) -> &mut usize {
        &mut self.count
    }

    fn count(&self) -> usize {
        self.count
    }

    fn primitive_type_mut(&mut self) -> &mut Option<PrimitiveType> {
        &mut self.primitive_type
    }

    fn primitive_type(&self) -> Option<PrimitiveType> {
        self.primitive_type
    }

    fn set_wireframe_index_buffer(&mut self, buffer: Buffer) {
        self.wireframe_index_buffer = Some(buffer);
    }

    fn add_pipeline_resource(&mut self, buffer: Buffer) {
        self.model.pipeline_resources.push(buffer);
    }

    fn add_buffer_to_statistics(&mut self, buffer: &Buffer) {
        self.model.statistics.add_buffer(buffer, false);
    }
}

/// The wireframe pipeline stage generates a new index buffer for rendering the
/// structure of the mesh with gl.LINES.
pub struct WireframePipelineStage;

impl WireframePipelineStage {
    pub const NAME: &'static str = "WireframePipelineStage";

    /// Process a primitive to configure wireframe rendering.
    pub fn process<R: WireframeRenderResources>(
        render_resources: &mut R,
        primitive: &Primitive,
        _frame_state: &WireframeFrameState,
    ) {
        let shader_builder = render_resources.shader_builder_mut();
        shader_builder.add_define(
            "HAS_WIREFRAME",
            None,
            Some(ShaderDestination::FRAGMENT),
        );

        let vertex_count = primitive
            .attributes
            .iter()
            .find(|a| a.semantic.as_deref() == Some("POSITION"))
            .map(|a| a.count)
            .unwrap_or(0);

        let original_indices = if let Some(ind) = render_resources.indices() {
            parse_indices(ind)
        } else {
            Vec::new()
        };

        let prim_type = primitive
            .primitive_type
            .or_else(|| render_resources.primitive_type())
            .unwrap_or(PrimitiveType::Triangles);

        let wireframe_opt = if original_indices.is_empty() {
            WireframeIndexGenerator::create_wireframe_indices(prim_type, vertex_count, None)
        } else {
            WireframeIndexGenerator::create_wireframe_indices(
                prim_type,
                vertex_count,
                Some(&original_indices),
            )
        };

        if let Some(wireframe) = wireframe_opt {
            let (bytes, index_datatype) = encode_index_buffer(&wireframe);

            let buffer = Buffer::create_index_buffer(IndexBufferOptions {
                size_in_bytes: 0,
                usage: Some(BufferUsage::StaticDraw),
                typed_array: Some(bytes),
                index_datatype,
            })
            .expect("Failed to create wireframe index buffer");

            render_resources.add_pipeline_resource(buffer.clone());
            render_resources.add_buffer_to_statistics(&buffer);
            render_resources.set_wireframe_index_buffer(buffer);

            let original_prim_type = render_resources
                .primitive_type()
                .unwrap_or(PrimitiveType::Triangles);
            let original_count = render_resources.count();

            *render_resources.primitive_type_mut() = Some(PrimitiveType::Lines);
            *render_resources.count_mut() =
                WireframeIndexGenerator::get_wireframe_indices_count(original_prim_type, original_count);
        }
    }
}

fn parse_indices(indices: &Indices) -> Vec<u32> {
    let raw_bytes = if let Some(ref bytes) = indices.typed_array {
        bytes.as_slice()
    } else if let Some(ref buf) = indices.buffer {
        buf.data.as_slice()
    } else {
        return Vec::new();
    };

    match indices.index_datatype {
        Some(IndexDatatype::UnsignedByte) => raw_bytes.iter().map(|&b| b as u32).collect(),
        Some(IndexDatatype::UnsignedShort) => raw_bytes
            .as_chunks::<2>()
            .0
            .iter()
            .map(|chunk| u16::from_le_bytes(*chunk) as u32)
            .collect(),
        Some(IndexDatatype::UnsignedInt) => raw_bytes
            .as_chunks::<4>()
            .0
            .iter()
            .map(|chunk| u32::from_le_bytes(*chunk))
            .collect(),
        None => {
            if indices.count > 0 && raw_bytes.len() == indices.count {
                raw_bytes.iter().map(|&b| b as u32).collect()
            } else if indices.count > 0 && raw_bytes.len() == indices.count * 2 {
                raw_bytes
                    .as_chunks::<2>()
                    .0
                    .iter()
                    .map(|chunk| u16::from_le_bytes(*chunk) as u32)
                    .collect()
            } else if indices.count > 0 && raw_bytes.len() == indices.count * 4 {
                raw_bytes
                    .as_chunks::<4>()
                    .0
                    .iter()
                    .map(|chunk| u32::from_le_bytes(*chunk))
                    .collect()
            } else {
                raw_bytes.iter().map(|&b| b as u32).collect()
            }
        }
    }
}

fn encode_index_buffer(buffer: &IndexBuffer) -> (Vec<u8>, IndexDatatype) {
    match buffer {
        IndexBuffer::U16(v) => {
            let mut out = Vec::with_capacity(v.len() * 2);
            for &val in v {
                out.extend_from_slice(&val.to_le_bytes());
            }
            (out, IndexDatatype::UnsignedShort)
        }
        IndexBuffer::U32(v) => {
            let mut out = Vec::with_capacity(v.len() * 4);
            for &val in v {
                out.extend_from_slice(&val.to_le_bytes());
            }
            (out, IndexDatatype::UnsignedInt)
        }
    }
}
