//! ClassificationPipelineStage matching CesiumJS `Scene/Model/ClassificationPipelineStage.js` (Apache-2.0).

use crate::shader_builder::{ShaderBuilder, ShaderDestination};
use cesium_math::{PrimitiveType, RuntimeError};

pub use crate::vertex_attribute_semantic::VertexAttributeSemantic;

/// Attribute descriptor for classification pipeline stage.
#[derive(Clone, Debug, Default)]
pub struct ClassificationStageAttribute {
    pub semantic: Option<String>,
    pub set_index: Option<usize>,
    pub count: usize,
    pub typed_array: Option<Vec<u32>>,
}

/// Indices descriptor for classification pipeline stage.
#[derive(Clone, Debug, Default)]
pub struct ClassificationStageIndices {
    pub count: usize,
    pub typed_array: Option<Vec<u32>>,
}

/// Primitive representation for classification pipeline stage.
#[derive(Clone, Debug, Default)]
pub struct ClassificationStagePrimitive {
    pub primitive_type: Option<PrimitiveType>,
    pub attributes: Vec<ClassificationStageAttribute>,
    pub indices: Option<ClassificationStageIndices>,
}

/// Runtime primitive holding batch calculations.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ClassificationStageRuntimePrimitive {
    pub batch_lengths: Option<Vec<usize>>,
    pub batch_offsets: Option<Vec<usize>>,
}

/// Render resources passed to classification pipeline stage.
pub struct ClassificationStageRenderResources<'a> {
    pub shader_builder: &'a mut ShaderBuilder,
    pub runtime_primitive: &'a mut ClassificationStageRuntimePrimitive,
}

/// The classification pipeline stage is responsible for batching features
/// together to be rendered by the ClassificationModelDrawCommand.
pub struct ClassificationPipelineStage;

impl ClassificationPipelineStage {
    pub const NAME: &'static str = "ClassificationPipelineStage";

    /// Process a primitive to inject classification shader defines and compute batches.
    pub fn process(
        render_resources: &mut ClassificationStageRenderResources<'_>,
        primitive: &mut ClassificationStagePrimitive,
    ) -> Result<(), RuntimeError> {
        render_resources.shader_builder.add_define(
            "HAS_CLASSIFICATION",
            None,
            Some(ShaderDestination::BOTH),
        );

        if render_resources.runtime_primitive.batch_lengths.is_none() {
            create_classification_batches(primitive, render_resources.runtime_primitive)?;
        }

        Ok(())
    }
}

fn create_classification_batches(
    primitive: &mut ClassificationStagePrimitive,
    runtime_primitive: &mut ClassificationStageRuntimePrimitive,
) -> Result<(), RuntimeError> {
    let position_attr = primitive
        .attributes
        .iter()
        .find(|a| a.semantic.as_deref() == Some(VertexAttributeSemantic::POSITION))
        .ok_or_else(|| {
            RuntimeError::new(
                "Primitives must have a position attribute to be used for classification.",
            )
        })?;

    let (has_indices, count, indices_array) = if let Some(ref mut indices) = primitive.indices {
        (true, indices.count, indices.typed_array.take())
    } else {
        (false, position_attr.count, None)
    };

    let feature_id_attr = primitive.attributes.iter_mut().find(|a| {
        a.semantic.as_deref() == Some(VertexAttributeSemantic::FEATURE_ID)
            && a.set_index.unwrap_or(0) == 0
    });

    let Some(feature_id_attr) = feature_id_attr else {
        runtime_primitive.batch_lengths = Some(vec![count]);
        runtime_primitive.batch_offsets = Some(vec![0]);
        return Ok(());
    };

    let Some(feature_ids) = feature_id_attr.typed_array.take() else {
        runtime_primitive.batch_lengths = Some(vec![count]);
        runtime_primitive.batch_offsets = Some(vec![0]);
        return Ok(());
    };

    if count == 0 {
        runtime_primitive.batch_lengths = Some(Vec::new());
        runtime_primitive.batch_offsets = Some(Vec::new());
        return Ok(());
    }

    let mut batch_lengths = Vec::new();
    let mut batch_offsets = vec![0];

    let get_index = |i: usize| -> usize {
        if has_indices {
            indices_array.as_ref().map_or(i, |arr| arr[i] as usize)
        } else {
            i
        }
    };

    let first_index = get_index(0);
    let mut current_batch_id = feature_ids[first_index];
    let mut current_offset = 0;

    for i in 1..count {
        let idx = get_index(i);
        let batch_id = feature_ids[idx];

        if batch_id != current_batch_id {
            let batch_length = i - current_offset;
            let new_offset = i;

            batch_lengths.push(batch_length);
            batch_offsets.push(new_offset);

            current_offset = new_offset;
            current_batch_id = batch_id;
        }
    }

    let final_batch_length = count - current_offset;
    batch_lengths.push(final_batch_length);

    runtime_primitive.batch_lengths = Some(batch_lengths);
    runtime_primitive.batch_offsets = Some(batch_offsets);

    Ok(())
}
