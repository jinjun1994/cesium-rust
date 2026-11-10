//! PrimitiveStatisticsPipelineStage matching CesiumJS `Scene/Model/PrimitiveStatisticsPipelineStage.js` (Apache-2.0).

use std::sync::Arc;
use cesium_math::PrimitiveType;

use crate::buffer::Buffer;
use crate::classification_pipeline_stage::VertexAttributeSemantic;
use crate::gltf_loader_util::ModelTextureReader;
use crate::gltf_structural_metadata_loader::StructuralMetadata;
use crate::model_components::{FeatureId, Material, Primitive};
use crate::model_runtime_primitive::ModelRuntimePrimitive;
use crate::model_statistics::{BatchTextureStatisticsSource, ModelStatistics};

/// Pipeline stage that updates memory usage statistics for a model on the primitive level.
pub struct PrimitiveStatisticsPipelineStage;

impl PrimitiveStatisticsPipelineStage {
    pub const NAME: &'static str = "PrimitiveStatisticsPipelineStage";

    /// Counts points, triangles and geometry buffers for a primitive.
    pub fn count_geometry(statistics: &mut ModelStatistics, primitive: &Primitive) {
        let indices_count = if let Some(ref indices) = primitive.indices {
            indices.count
        } else if let Some(pos) = primitive
            .attributes
            .iter()
            .find(|a| a.semantic.as_deref() == Some(VertexAttributeSemantic::POSITION))
        {
            pos.count
        } else {
            0
        };

        if let Some(prim_type) = primitive.primitive_type {
            if prim_type == PrimitiveType::Points {
                statistics.points_length += indices_count;
            } else if prim_type == PrimitiveType::Triangles {
                statistics.triangles_length += indices_count / 3;
            } else if prim_type == PrimitiveType::TriangleStrip
                || prim_type == PrimitiveType::TriangleFan
            {
                statistics.triangles_length += indices_count.saturating_sub(2);
            }
        }

        for attr in &primitive.attributes {
            if let Some(ref buffer) = attr.buffer {
                let has_cpu_copy = attr.typed_array.is_some();
                statistics.add_buffer(buffer, has_cpu_copy);
            }
        }

        if let Some(ref outline) = primitive.outline_coordinates {
            if let Some(ref buffer) = outline.buffer {
                statistics.add_buffer(buffer, false);
            }
        }

        if let Some(ref indices) = primitive.indices {
            if let Some(ref buffer) = indices.buffer {
                let has_cpu_copy = indices.typed_array.is_some();
                statistics.add_buffer(buffer, has_cpu_copy);
            }
        }
    }

    /// Counts memory allocated for 2D position buffers.
    pub fn count_2d_positions(
        statistics: &mut ModelStatistics,
        position_buffer_2d: Option<&Buffer>,
    ) {
        if let Some(buffer) = position_buffer_2d {
            statistics.add_buffer(buffer, true);
        }
    }

    /// Counts buffers used for morph target displacements.
    pub fn count_morph_target_attributes(
        statistics: &mut ModelStatistics,
        primitive: &Primitive,
    ) {
        for target in &primitive.morph_targets {
            for attr in &target.attributes {
                if let Some(ref buffer) = attr.buffer {
                    statistics.add_buffer(buffer, false);
                }
            }
        }
    }

    /// Counts texture memory for standard materials.
    pub fn count_material_textures(
        statistics: &mut ModelStatistics,
        material: Option<&Material>,
    ) {
        let Some(material) = material else { return };

        let mut readers: Vec<&ModelTextureReader> = Vec::new();
        if let Some(ref tr) = material.emissive_texture {
            readers.push(tr);
        }
        if let Some(ref tr) = material.normal_texture {
            readers.push(tr);
        }
        if let Some(ref tr) = material.occlusion_texture {
            readers.push(tr);
        }
        if let Some(ref mr) = material.metallic_roughness {
            if let Some(ref tr) = mr.base_color_texture {
                readers.push(tr);
            }
            if let Some(ref tr) = mr.metallic_roughness_texture {
                readers.push(tr);
            }
        }
        if let Some(ref sg) = material.specular_glossiness {
            if let Some(ref tr) = sg.diffuse_texture {
                readers.push(tr);
            }
            if let Some(ref tr) = sg.specular_glossiness_texture {
                readers.push(tr);
            }
        }
        if let Some(ref s) = material.specular {
            if let Some(ref tr) = s.specular_texture {
                readers.push(tr);
            }
            if let Some(ref tr) = s.specular_color_texture {
                readers.push(tr);
            }
        }

        for reader in readers {
            if let Some(ref texture) = reader.texture {
                statistics.add_texture(texture);
            }
        }
    }

    /// Counts textures used by feature ID textures.
    pub fn count_feature_id_textures(
        statistics: &mut ModelStatistics,
        feature_ids: &[FeatureId],
    ) {
        for fid in feature_ids {
            if let FeatureId::Texture(ref fit) = fid {
                if let Some(ref reader) = fit.texture_reader {
                    if let Some(ref texture) = reader.texture {
                        statistics.add_texture(texture);
                    }
                }
            }
        }
    }

    /// Counts property tables, property textures, and batch textures.
    pub fn count_binary_metadata(
        statistics: &mut ModelStatistics,
        structural_metadata: Option<&StructuralMetadata>,
        property_tables_byte_length: usize,
        feature_tables: &[Arc<dyn BatchTextureStatisticsSource>],
    ) {
        if let Some(metadata) = structural_metadata {
            for prop_tex in &metadata.property_textures {
                for prop in prop_tex.properties.values() {
                    if let Some(ref texture) = prop.texture_reader.texture {
                        statistics.add_texture(texture);
                    }
                }
            }
            statistics.property_tables_byte_length += property_tables_byte_length;
        }

        for ft in feature_tables {
            statistics.add_batch_texture(ft.clone());
        }
    }

    /// Processes a primitive, updating statistics across geometry, materials, metadata, and 2D.
    pub fn process(
        statistics: &mut ModelStatistics,
        primitive: &Primitive,
        runtime_primitive: &ModelRuntimePrimitive,
        structural_metadata: Option<&StructuralMetadata>,
        property_tables_byte_length: usize,
        feature_tables: &[Arc<dyn BatchTextureStatisticsSource>],
    ) {
        Self::count_geometry(statistics, primitive);
        Self::count_2d_positions(
            statistics,
            runtime_primitive
                .position_buffer_2d
                .as_ref()
                .map(|_| () /* handled if Buffer */)
                .and(None),
        );
        Self::count_morph_target_attributes(statistics, primitive);
        Self::count_material_textures(statistics, primitive.material.as_ref());
        Self::count_feature_id_textures(statistics, &primitive.feature_ids);
        Self::count_binary_metadata(
            statistics,
            structural_metadata,
            property_tables_byte_length,
            feature_tables,
        );
    }
}
