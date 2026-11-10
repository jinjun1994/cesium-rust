//! SkinningPipelineStage matching CesiumJS `Scene/Model/SkinningPipelineStage.js` (Apache-2.0).

use cesium_math::Matrix4;

use crate::model_components::Primitive;
use crate::shader_builder::{ShaderBuilder, ShaderDestination};

/// Vertex shader code for the skinning stage.
pub const SKINNING_STAGE_VS: &str = r#"void skinningStage(inout ProcessedAttributes attributes) 
{
    mat4 skinningMatrix = getSkinningMatrix();
    mat3 skinningMatrixMat3 = mat3(skinningMatrix);

    vec4 positionMC = vec4(attributes.positionMC, 1.0);
    attributes.positionMC = vec3(skinningMatrix * positionMC);

    #ifdef HAS_NORMALS
    vec3 normalMC = attributes.normalMC;
    attributes.normalMC = skinningMatrixMat3 * normalMC;
    #endif

    #ifdef HAS_TANGENTS
    vec3 tangentMC = attributes.tangentMC;
    attributes.tangentMC = skinningMatrixMat3 * tangentMC;
    #endif
}"#;

/// Standalone runtime node state for skinning.
#[derive(Debug, Default, Clone)]
pub struct MockSkinningRuntimeNode {
    pub computed_joint_matrices: Vec<Matrix4>,
}

/// Standalone render resources for SkinningPipelineStage.
#[derive(Debug, Default)]
pub struct MockSkinningRenderResources {
    pub shader_builder: ShaderBuilder,
    pub runtime_node: MockSkinningRuntimeNode,
    pub bound_joint_matrices: Option<Vec<Matrix4>>,
}

/// Trait defining the render resources required by [`SkinningPipelineStage`].
pub trait SkinningRenderResources {
    fn shader_builder_mut(&mut self) -> &mut ShaderBuilder;
    fn computed_joint_matrices(&self) -> &[Matrix4];
    fn set_joint_matrices_uniform(&mut self, matrices: Vec<Matrix4>);
}

impl SkinningRenderResources for MockSkinningRenderResources {
    fn shader_builder_mut(&mut self) -> &mut ShaderBuilder {
        &mut self.shader_builder
    }

    fn computed_joint_matrices(&self) -> &[Matrix4] {
        &self.runtime_node.computed_joint_matrices
    }

    fn set_joint_matrices_uniform(&mut self, matrices: Vec<Matrix4>) {
        self.bound_joint_matrices = Some(matrices);
    }
}

/// The skinning pipeline stage processes the joint matrices of a skinned primitive.
pub struct SkinningPipelineStage;

impl SkinningPipelineStage {
    pub const NAME: &'static str = "SkinningPipelineStage";
    pub const FUNCTION_ID_GET_SKINNING_MATRIX: &'static str = "getSkinningMatrix";
    pub const FUNCTION_SIGNATURE_GET_SKINNING_MATRIX: &'static str = "mat4 getSkinningMatrix()";

    /// Process a primitive's skinning setup and uniforms.
    pub fn process<R: SkinningRenderResources>(
        render_resources: &mut R,
        primitive: &Primitive,
    ) {
        let shader_builder = render_resources.shader_builder_mut();

        shader_builder.add_define("HAS_SKINNING", None, Some(ShaderDestination::VERTEX));

        add_get_skinning_matrix_function(shader_builder, primitive);

        let joint_matrices = render_resources.computed_joint_matrices().to_vec();
        let joint_count = joint_matrices.len();

        let shader_builder = render_resources.shader_builder_mut();
        shader_builder.add_uniform(
            "mat4",
            &format!("u_jointMatrices[{joint_count}]"),
            Some(ShaderDestination::VERTEX),
        );

        shader_builder.add_vertex_lines(&[SKINNING_STAGE_VS]);

        render_resources.set_joint_matrices_uniform(joint_matrices);
    }
}

fn get_maximum_attribute_set_index(primitive: &Primitive) -> i32 {
    let mut set_index: i32 = -1;
    for attr in &primitive.attributes {
        if let Some(ref sem) = attr.semantic {
            let is_joints_or_weights = sem == "JOINTS"
                || sem == "WEIGHTS"
                || sem.starts_with("JOINTS_")
                || sem.starts_with("WEIGHTS_");

            if is_joints_or_weights {
                let idx = attr.set_index.unwrap_or(0) as i32;
                set_index = set_index.max(idx);
            }
        }
    }
    set_index
}

fn add_get_skinning_matrix_function(shader_builder: &mut ShaderBuilder, primitive: &Primitive) {
    shader_builder.add_function(
        SkinningPipelineStage::FUNCTION_ID_GET_SKINNING_MATRIX,
        SkinningPipelineStage::FUNCTION_SIGNATURE_GET_SKINNING_MATRIX,
        ShaderDestination::VERTEX,
    );

    let initial_line = "    mat4 skinnedMatrix = mat4(0);";
    shader_builder.add_function_lines(
        SkinningPipelineStage::FUNCTION_ID_GET_SKINNING_MATRIX,
        &[initial_line],
    );

    let component_strings = ["x", "y", "z", "w"];
    let maximum_set_index = get_maximum_attribute_set_index(primitive);
    let max_idx = if maximum_set_index >= 0 { maximum_set_index } else { 0 };

    for set_index in 0..=max_idx {
        for component in component_strings {
            let line = format!(
                "    skinnedMatrix += a_weights_{set_index}.{component} * u_jointMatrices[int(a_joints_{set_index}.{component})];"
            );
            shader_builder.add_function_lines(
                SkinningPipelineStage::FUNCTION_ID_GET_SKINNING_MATRIX,
                &[&line],
            );
        }
    }

    let return_line = "    return skinnedMatrix;";
    shader_builder.add_function_lines(
        SkinningPipelineStage::FUNCTION_ID_GET_SKINNING_MATRIX,
        &[return_line],
    );
}
