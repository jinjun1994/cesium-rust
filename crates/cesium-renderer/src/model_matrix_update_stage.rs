//! ModelMatrixUpdateStage matching CesiumJS `Scene/Model/ModelMatrixUpdateStage.js` (Apache-2.0).

use cesium_math::Matrix4;

use crate::model_runtime_node::ModelRuntimeNode;
use crate::model_utility::ModelUtility;

/// The model matrix update stage updates the model matrices and bounding volumes of draw commands.
pub struct ModelMatrixUpdateStage;

impl ModelMatrixUpdateStage {
    /// Processes a runtime node tree and updates draw commands if `transform_dirty` is true.
    pub fn update(
        root_idx: usize,
        nodes: &mut [ModelRuntimeNode],
        computed_model_matrix: &Matrix4,
        is_2d: bool,
        project_to_2d: bool,
    ) {
        if is_2d && project_to_2d {
            return;
        }

        if root_idx >= nodes.len() {
            return;
        }

        if nodes[root_idx].transform_dirty {
            let transform_to_root = nodes[root_idx].transform_to_root;
            Self::update_runtime_node(root_idx, nodes, computed_model_matrix, transform_to_root);
            nodes[root_idx].transform_dirty = false;
        }
    }

    /// Recursively update all child runtime nodes and their runtime primitives.
    fn update_runtime_node(
        node_idx: usize,
        nodes: &mut [ModelRuntimeNode],
        model_matrix: &Matrix4,
        transform_to_root: Matrix4,
    ) {
        let current_transform = nodes[node_idx].transform;
        let next_transform_to_root = transform_to_root.multiply(&current_transform);

        nodes[node_idx].transform_to_root = transform_to_root;
        nodes[node_idx].update_computed_transform(model_matrix);

        let computed_transform = nodes[node_idx].computed_transform;

        for prim in &mut nodes[node_idx].runtime_primitives {
            if let Some(cmd) = &mut prim.draw_command {
                let new_model_matrix = model_matrix.multiply(&next_transform_to_root);
                cmd.set_model_matrix(new_model_matrix.into());
                let cull_face = ModelUtility::get_cull_face(&new_model_matrix, cmd.primitive_type());
                cmd.set_cull_face(cull_face);

                if let Some(bs) = &prim.bounding_sphere {
                    let bs_trans = bs.transform(computed_transform.into()).unwrap_or(*bs);
                    cmd.set_bounding_volume(bs_trans);
                }
            }
        }

        let children = nodes[node_idx].children.clone();
        for child_idx in children {
            if child_idx < nodes.len() {
                nodes[child_idx].transform_to_root = next_transform_to_root;
                Self::update_runtime_node(child_idx, nodes, model_matrix, next_transform_to_root);
                nodes[child_idx].transform_dirty = false;
            }
        }
    }
}
