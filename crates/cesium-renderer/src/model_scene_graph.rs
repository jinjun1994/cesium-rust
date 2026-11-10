//! ModelSceneGraph matching CesiumJS `Scene/Model/ModelSceneGraph.js` (Apache-2.0).

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use cesium_math::{BoundingSphere, Matrix4};

use crate::axis::Axis;
use crate::commands::{DrawCommand, DrawCommandOptions, Pass};
use crate::model_components::{Components, Node};
use crate::model_draw_command::ModelDrawCommand;
use crate::model_node::ModelNode;
use crate::model_runtime_node::ModelRuntimeNode;
use crate::model_runtime_primitive::{
    ModelRuntimePrimitive, ModelRuntimePrimitiveModel, PrimitiveFrameState,
};
use crate::model_skin::{ModelSkin, ModelSkinOptions};
use crate::model_utility::ModelUtility;

/// Options for instantiating a [`ModelSceneGraph`].
#[derive(Clone, Debug)]
pub struct ModelSceneGraphOptions {
    pub model_matrix: Matrix4,
    pub computed_scale: f64,
    pub components: Components,
    pub model_parameters: ModelRuntimePrimitiveModel,
}

impl Default for ModelSceneGraphOptions {
    fn default() -> Self {
        Self {
            model_matrix: Matrix4::IDENTITY,
            computed_scale: 1.0,
            components: Components::default(),
            model_parameters: ModelRuntimePrimitiveModel::default(),
        }
    }
}

/// An in-memory representation of the scene graph for a [`Model`].
#[derive(Clone, Debug)]
pub struct ModelSceneGraph {
    pub components: Components,
    pub runtime_nodes: Vec<ModelRuntimeNode>,
    pub root_nodes: Vec<usize>,
    pub skinned_nodes: Vec<usize>,
    pub runtime_skins: Vec<ModelSkin>,
    pub axis_correction_matrix: Matrix4,
    pub computed_model_matrix: Matrix4,
    pub bounding_sphere: Option<BoundingSphere>,
    pub bounding_sphere_2d: Option<BoundingSphere>,
    pub nodes_by_name: HashMap<String, ModelNode>,
    pub draw_commands_built: bool,
    pub model_parameters: ModelRuntimePrimitiveModel,
}

impl ModelSceneGraph {
    /// Constructs and initializes a `ModelSceneGraph` from components and options.
    pub fn new(options: ModelSceneGraphOptions) -> Self {
        let components = options.components;
        let up_axis = components.up_axis.unwrap_or(Axis::Y);
        let forward_axis = components.forward_axis.unwrap_or(Axis::Z);

        let axis_correction_matrix = ModelUtility::get_axis_correction_matrix(up_axis, forward_axis);

        let mut computed_model_matrix = options
            .model_matrix
            .multiply_transformation(&components.transform);
        computed_model_matrix =
            computed_model_matrix.multiply_transformation(&axis_correction_matrix);
        computed_model_matrix =
            computed_model_matrix.multiply_by_uniform_scale(options.computed_scale);

        let nodes_count = components.nodes.len();
        let mut runtime_nodes: Vec<Option<ModelRuntimeNode>> = (0..nodes_count).map(|_| None).collect();
        let mut root_nodes = Vec::new();
        let mut skinned_nodes = Vec::new();
        let mut nodes_by_name = HashMap::new();

        // Determine root nodes from scene or default to all top-level nodes
        let roots_to_traverse: Vec<usize> = if let Some(ref scene) = components.scene {
            if !scene.nodes.is_empty() {
                scene.nodes.clone()
            } else {
                (0..nodes_count).collect()
            }
        } else {
            (0..nodes_count).collect()
        };

        for &root_idx in &roots_to_traverse {
            if root_idx < nodes_count {
                Self::traverse_and_create_node(
                    root_idx,
                    Matrix4::IDENTITY,
                    &components.nodes,
                    &options.model_parameters,
                    &mut runtime_nodes,
                    &mut skinned_nodes,
                    &mut nodes_by_name,
                );
                root_nodes.push(root_idx);
            }
        }

        // Flatten runtime nodes, replacing any unreachable with default
        let finalized_runtime_nodes: Vec<ModelRuntimeNode> = runtime_nodes
            .into_iter()
            .enumerate()
            .map(|(i, opt)| {
                opt.unwrap_or_else(|| ModelRuntimeNode {
                    id: i,
                    ..Default::default()
                })
            })
            .collect();

        // Instantiate skins
        let mut runtime_skins = Vec::with_capacity(components.skins.len());
        for skin in &components.skins {
            if let Ok(runtime_skin) = ModelSkin::try_new(ModelSkinOptions {
                skin: Some(skin.clone()),
                runtime_nodes: Some(&finalized_runtime_nodes),
            }) {
                runtime_skins.push(runtime_skin);
            }
        }

        Self {
            components,
            runtime_nodes: finalized_runtime_nodes,
            root_nodes,
            skinned_nodes,
            runtime_skins,
            axis_correction_matrix,
            computed_model_matrix,
            bounding_sphere: None,
            bounding_sphere_2d: None,
            nodes_by_name,
            draw_commands_built: false,
            model_parameters: options.model_parameters,
        }
    }

    /// Recursively traverses and instantiates runtime nodes.
    fn traverse_and_create_node(
        node_idx: usize,
        transform_to_root: Matrix4,
        nodes: &[Node],
        model_parameters: &ModelRuntimePrimitiveModel,
        runtime_nodes: &mut [Option<ModelRuntimeNode>],
        skinned_nodes: &mut Vec<usize>,
        nodes_by_name: &mut HashMap<String, ModelNode>,
    ) {
        let node = match nodes.get(node_idx) {
            Some(n) => n,
            None => return,
        };

        let node_transform = ModelUtility::get_node_transform(
            node.matrix,
            node.translation,
            node.rotation,
            node.scale,
        );
        let child_transform_to_root = transform_to_root.multiply_transformation(&node_transform);

        let mut children_indices = Vec::with_capacity(node.children.len());
        for &child_idx in &node.children {
            Self::traverse_and_create_node(
                child_idx,
                child_transform_to_root,
                nodes,
                model_parameters,
                runtime_nodes,
                skinned_nodes,
                nodes_by_name,
            );
            children_indices.push(child_idx);
        }

        let mut runtime_node = ModelRuntimeNode::from_node(
            node.clone(),
            node_transform,
            transform_to_root,
            children_indices,
        );

        // Convert primitives to ModelRuntimePrimitive
        for prim in &node.primitives {
            runtime_node.runtime_primitives.push(ModelRuntimePrimitive {
                primitive: Some(prim.clone()),
                node: Some(node.clone()),
                model: Some(model_parameters.clone()),
                ..Default::default()
            });
        }

        if node.skin.is_some() {
            skinned_nodes.push(node_idx);
        }

        if let Some(ref name) = node.name {
            if let Ok(public_node) = ModelNode::new(
                true,
                Some(Rc::new(RefCell::new(runtime_node.clone()))),
                None,
            ) {
                nodes_by_name.insert(name.clone(), public_node);
            }
        }

        runtime_nodes[node_idx] = Some(runtime_node);
    }

    /// Builds draw commands for each runtime primitive across the scene graph.
    pub fn build_draw_commands(&mut self, frame_state: &PrimitiveFrameState) {
        for node in &mut self.runtime_nodes {
            for prim in &mut node.runtime_primitives {
                prim.configure_pipeline(frame_state);

                // Build draw command
                let mut cmd = DrawCommand::new(DrawCommandOptions::default());
                cmd.model_matrix = Some(node.computed_transform.into());

                let pass = if self.model_parameters.has_style {
                    Pass::Translucent
                } else {
                    Pass::Opaque
                };
                cmd.pass = Some(pass);

                prim.draw_command = Some(ModelDrawCommand::from_command(cmd));
            }
        }
        self.draw_commands_built = true;
    }

    /// Pushes all built draw commands into the target command list.
    pub fn push_draw_commands(&self, command_list: &mut Vec<DrawCommand>) {
        for node in &self.runtime_nodes {
            for prim in &node.runtime_primitives {
                if let Some(ref cmd) = prim.draw_command {
                    command_list.push(cmd.command().clone());
                }
            }
        }
    }

    /// Updates the root model matrix and propagates transforms across nodes.
    pub fn update_model_matrix(&mut self, model_matrix: &Matrix4) {
        let mut computed = model_matrix.multiply_transformation(&self.components.transform);
        computed = computed.multiply_transformation(&self.axis_correction_matrix);
        self.computed_model_matrix = computed;

        let roots = self.root_nodes.clone();
        for root_idx in roots {
            self.propagate_transform(root_idx, Matrix4::IDENTITY);
        }
    }

    fn propagate_transform(&mut self, node_idx: usize, transform_to_root: Matrix4) {
        if node_idx >= self.runtime_nodes.len() {
            return;
        }

        self.runtime_nodes[node_idx].transform_to_root = transform_to_root;
        let next_transform_to_root = transform_to_root.multiply_transformation(&self.runtime_nodes[node_idx].transform);
        self.runtime_nodes[node_idx].update_computed_transform();

        let children = self.runtime_nodes[node_idx].children.clone();
        for child_idx in children {
            self.propagate_transform(child_idx, next_transform_to_root);
        }
    }
}
