//! ModelSkin matching CesiumJS `Scene/Model/ModelSkin.js` (Apache-2.0).

use cesium_math::{DeveloperError, Matrix4};

use crate::model_components::Skin;
use crate::model_runtime_node::ModelRuntimeNode;

/// Options for instantiating a [`ModelSkin`].
#[derive(Clone, Debug, Default)]
pub struct ModelSkinOptions<'a> {
    pub skin: Option<Skin>,
    pub runtime_nodes: Option<&'a [ModelRuntimeNode]>,
}

/// An in-memory representation of a skin that affects nodes in the `ModelSceneGraph`.
#[derive(Clone, Debug, PartialEq)]
pub struct ModelSkin {
    pub skin: Skin,
    pub inverse_bind_matrices: Vec<Matrix4>,
    pub joints: Vec<usize>,
    pub joint_matrices: Vec<Matrix4>,
}

impl ModelSkin {
    /// Attempts to construct a `ModelSkin` verifying that all arguments are defined.
    pub fn try_new(options: ModelSkinOptions<'_>) -> Result<Self, DeveloperError> {
        let skin = options
            .skin
            .ok_or_else(|| DeveloperError::new("options.skin is required."))?;
        let runtime_nodes = options
            .runtime_nodes
            .ok_or_else(|| DeveloperError::new("options.sceneGraph is required."))?;

        let inverse_bind_matrices = skin.inverse_bind_matrices.clone();
        let mut joints = Vec::with_capacity(skin.joints.len());
        let mut joint_matrices = Vec::with_capacity(skin.joints.len());

        for (i, &joint_idx) in skin.joints.iter().enumerate() {
            joints.push(joint_idx);
            let ibm = inverse_bind_matrices
                .get(i)
                .copied()
                .unwrap_or(Matrix4::IDENTITY);
            let runtime_node = runtime_nodes
                .get(joint_idx)
                .ok_or_else(|| DeveloperError::new("joint runtime node not found"))?;
            let joint_matrix = Self::compute_joint_matrix(runtime_node, &ibm);
            joint_matrices.push(joint_matrix);
        }

        Ok(Self {
            skin,
            inverse_bind_matrices,
            joints,
            joint_matrices,
        })
    }

    /// Computes joint matrix: `jointWorldTransform * inverseBindMatrix`.
    pub fn compute_joint_matrix(joint: &ModelRuntimeNode, inverse_bind_matrix: &Matrix4) -> Matrix4 {
        let joint_world_transform = joint
            .transform_to_root
            .multiply_transformation(&joint.transform);
        joint_world_transform.multiply_transformation(inverse_bind_matrix)
    }

    /// Updates all joint matrices from current runtime node transforms.
    pub fn update_joint_matrices(&mut self, runtime_nodes: &[ModelRuntimeNode]) {
        for (i, &joint_idx) in self.joints.iter().enumerate() {
            if let Some(joint) = runtime_nodes.get(joint_idx) {
                if let Some(ibm) = self.inverse_bind_matrices.get(i) {
                    self.joint_matrices[i] = Self::compute_joint_matrix(joint, ibm);
                }
            }
        }
    }
}
