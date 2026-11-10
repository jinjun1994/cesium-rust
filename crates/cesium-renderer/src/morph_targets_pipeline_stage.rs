//! MorphTargetsPipelineStage matching CesiumJS `Scene/Model/MorphTargetsPipelineStage.js` (Apache-2.0).

use cesium_math::ComponentDatatype;

use crate::classification_pipeline_stage::VertexAttributeSemantic;
use crate::model_components::Primitive;
use crate::primitive_render_resources::PrimitiveRenderResources;
use crate::shader_builder::{ShaderBuilder, ShaderDestination};
use crate::vertex_array::VertexAttribute;

/// GLSL shader code for the morph targets vertex stage.
pub const MORPH_TARGETS_STAGE_VS: &str = r#"void morphTargetsStage(inout ProcessedAttributes attributes) 
{
    vec3 positionMC = attributes.positionMC;
    attributes.positionMC = getMorphedPosition(positionMC);

    #ifdef HAS_NORMALS
    vec3 normalMC = attributes.normalMC;
    attributes.normalMC = getMorphedNormal(normalMC);
    #endif

    #ifdef HAS_TANGENTS
    vec3 tangentMC = attributes.tangentMC;
    attributes.tangentMC = getMorphedTangent(tangentMC);
    #endif
}"#;

/// Trait abstracting render resources required by [`MorphTargetsPipelineStage`].
pub trait MorphTargetsStageRenderResources {
    fn shader_builder_mut(&mut self) -> &mut ShaderBuilder;
    fn attributes_mut(&mut self) -> &mut Vec<VertexAttribute>;
    fn attribute_index_mut(&mut self) -> &mut u32;
    fn morph_weights(&self) -> &[f64];
    fn set_morph_weights_uniform(&mut self, weights: Vec<f64>);
}

impl<'a, M> MorphTargetsStageRenderResources for PrimitiveRenderResources<'a, M> {
    fn shader_builder_mut(&mut self) -> &mut ShaderBuilder {
        &mut self.shader_builder
    }

    fn attributes_mut(&mut self) -> &mut Vec<VertexAttribute> {
        &mut self.attributes
    }

    fn attribute_index_mut(&mut self) -> &mut u32 {
        &mut self.attribute_index
    }

    fn morph_weights(&self) -> &[f64] {
        &self.runtime_node.morph_weights
    }

    fn set_morph_weights_uniform(&mut self, _weights: Vec<f64>) {}
}

/// Standalone test render resources container for MorphTargetsPipelineStage.
#[derive(Debug, Default)]
pub struct MockMorphTargetsRenderResources {
    pub shader_builder: ShaderBuilder,
    pub attributes: Vec<VertexAttribute>,
    pub attribute_index: u32,
    pub morph_weights: Vec<f64>,
    pub uniform_morph_weights: Option<Vec<f64>>,
}

impl MorphTargetsStageRenderResources for MockMorphTargetsRenderResources {
    fn shader_builder_mut(&mut self) -> &mut ShaderBuilder {
        &mut self.shader_builder
    }

    fn attributes_mut(&mut self) -> &mut Vec<VertexAttribute> {
        &mut self.attributes
    }

    fn attribute_index_mut(&mut self) -> &mut u32 {
        &mut self.attribute_index
    }

    fn morph_weights(&self) -> &[f64] {
        &self.morph_weights
    }

    fn set_morph_weights_uniform(&mut self, weights: Vec<f64>) {
        self.uniform_morph_weights = Some(weights);
    }
}

/// The morph targets pipeline stage processes the morph targets and weights of a primitive.
pub struct MorphTargetsPipelineStage;

impl MorphTargetsPipelineStage {
    pub const NAME: &'static str = "MorphTargetsPipelineStage";

    pub const FUNCTION_ID_GET_MORPHED_POSITION: &'static str = "getMorphedPosition";
    pub const FUNCTION_SIGNATURE_GET_MORPHED_POSITION: &'static str =
        "vec3 getMorphedPosition(in vec3 position)";

    pub const FUNCTION_ID_GET_MORPHED_NORMAL: &'static str = "getMorphedNormal";
    pub const FUNCTION_SIGNATURE_GET_MORPHED_NORMAL: &'static str =
        "vec3 getMorphedNormal(in vec3 normal)";

    pub const FUNCTION_ID_GET_MORPHED_TANGENT: &'static str = "getMorphedTangent";
    pub const FUNCTION_SIGNATURE_GET_MORPHED_TANGENT: &'static str =
        "vec3 getMorphedTangent(in vec3 tangent)";

    /// Processes a primitive's morph targets and weights, injecting attribute declarations and functions into the shaders.
    pub fn process<R: MorphTargetsStageRenderResources>(
        render_resources: &mut R,
        primitive: &Primitive,
    ) {
        let shader_builder = render_resources.shader_builder_mut();

        shader_builder.add_define(
            "HAS_MORPH_TARGETS",
            None,
            Some(ShaderDestination::VERTEX),
        );

        // Add function declarations with initial assignment lines
        shader_builder.add_function(
            Self::FUNCTION_ID_GET_MORPHED_POSITION,
            Self::FUNCTION_SIGNATURE_GET_MORPHED_POSITION,
            ShaderDestination::VERTEX,
        );
        shader_builder.add_function_lines(
            Self::FUNCTION_ID_GET_MORPHED_POSITION,
            &["vec3 morphedPosition = position;"],
        );

        shader_builder.add_function(
            Self::FUNCTION_ID_GET_MORPHED_NORMAL,
            Self::FUNCTION_SIGNATURE_GET_MORPHED_NORMAL,
            ShaderDestination::VERTEX,
        );
        shader_builder.add_function_lines(
            Self::FUNCTION_ID_GET_MORPHED_NORMAL,
            &["vec3 morphedNormal = normal;"],
        );

        shader_builder.add_function(
            Self::FUNCTION_ID_GET_MORPHED_TANGENT,
            Self::FUNCTION_SIGNATURE_GET_MORPHED_TANGENT,
            ShaderDestination::VERTEX,
        );
        shader_builder.add_function_lines(
            Self::FUNCTION_ID_GET_MORPHED_TANGENT,
            &["vec3 morphedTangent = tangent;"],
        );

        for (i, target) in primitive.morph_targets.iter().enumerate() {
            for attribute in &target.attributes {
                let semantic = attribute.semantic.as_deref().unwrap_or("");

                let (attribute_string, function_id) = match semantic {
                    s if s == VertexAttributeSemantic::POSITION => {
                        ("Position", Self::FUNCTION_ID_GET_MORPHED_POSITION)
                    }
                    s if s == VertexAttributeSemantic::NORMAL => {
                        ("Normal", Self::FUNCTION_ID_GET_MORPHED_NORMAL)
                    }
                    s if s == VertexAttributeSemantic::TANGENT => {
                        ("Tangent", Self::FUNCTION_ID_GET_MORPHED_TANGENT)
                    }
                    _ => continue,
                };

                let current_index = *render_resources.attribute_index_mut();
                *render_resources.attribute_index_mut() += 1;

                let components = attribute
                    .attribute_type
                    .map_or(3, |t| t.get_number_of_components() as u32);
                let datatype = attribute
                    .component_datatype
                    .unwrap_or(ComponentDatatype::Float);
                let stride = attribute.byte_stride.unwrap_or(12);

                let vertex_attr = VertexAttribute {
                    index: current_index,
                    enabled: true,
                    vertex_buffer: attribute.buffer.clone(),
                    components_per_attribute: components,
                    component_datatype: datatype,
                    offset_in_bytes: attribute.byte_offset,
                    stride_in_bytes: stride,
                    normalize: attribute.normalized,
                    instance_divisor: 0,
                };
                render_resources.attributes_mut().push(vertex_attr);

                let attribute_name = format!("a_target{attribute_string}_{i}");
                let line = format!(
                    "morphed{attribute_string} += u_morphWeights[{i}] * a_target{attribute_string}_{i};"
                );

                let shader_builder = render_resources.shader_builder_mut();
                shader_builder.add_attribute("vec3", &attribute_name);
                shader_builder.add_function_lines(function_id, &[&line]);
            }
        }

        let weights = render_resources.morph_weights().to_vec();
        let weights_length = weights.len();
        let uniform_name = format!("u_morphWeights[{weights_length}]");

        let shader_builder = render_resources.shader_builder_mut();

        // Add return statements
        shader_builder.add_function_lines(
            Self::FUNCTION_ID_GET_MORPHED_POSITION,
            &["return morphedPosition;"],
        );
        shader_builder.add_function_lines(
            Self::FUNCTION_ID_GET_MORPHED_NORMAL,
            &["return morphedNormal;"],
        );
        shader_builder.add_function_lines(
            Self::FUNCTION_ID_GET_MORPHED_TANGENT,
            &["return morphedTangent;"],
        );

        shader_builder.add_uniform("float", &uniform_name, Some(ShaderDestination::VERTEX));
        shader_builder.add_vertex_lines(&[MORPH_TARGETS_STAGE_VS]);

        render_resources.set_morph_weights_uniform(weights);
    }
}
