use cesium_math::{Cartesian3, Matrix4};
use cesium_renderer::model_components::{Components, Node, Primitive, Scene, Skin};
use cesium_renderer::{
    Axis, DrawCommand, ModelRuntimePrimitiveModel, ModelSceneGraph, ModelSceneGraphOptions, Pass,
    PrimitiveFrameState,
};

#[test]
fn test_creates_runtime_nodes_and_primitives() {
    let node0 = Node {
        name: Some("node0".to_string()),
        index: Some(0),
        children: vec![1],
        primitives: vec![Primitive::default()],
        ..Default::default()
    };
    let node1 = Node {
        name: Some("node1".to_string()),
        index: Some(1),
        children: vec![],
        primitives: vec![Primitive::default()],
        ..Default::default()
    };

    let components = Components {
        scene: Some(Scene { nodes: vec![0] }),
        nodes: vec![node0, node1],
        ..Default::default()
    };

    let scene_graph = ModelSceneGraph::new(ModelSceneGraphOptions {
        components,
        ..Default::default()
    });

    assert_eq!(scene_graph.runtime_nodes.len(), 2);
    assert_eq!(scene_graph.runtime_nodes[0].runtime_primitives.len(), 1);
    assert_eq!(scene_graph.runtime_nodes[1].runtime_primitives.len(), 1);
    assert_eq!(scene_graph.root_nodes, vec![0]);
}

#[test]
fn test_propagates_node_transforms_correctly() {
    let parent_transform = Matrix4::from_translation(Cartesian3::new(10.0, 0.0, 0.0));
    let child_transform = Matrix4::from_translation(Cartesian3::new(0.0, 20.0, 0.0));

    let node0 = Node {
        name: Some("parent".to_string()),
        index: Some(0),
        matrix: Some(parent_transform),
        children: vec![1],
        ..Default::default()
    };
    let node1 = Node {
        name: Some("child".to_string()),
        index: Some(1),
        matrix: Some(child_transform),
        children: vec![],
        ..Default::default()
    };

    let components = Components {
        scene: Some(Scene { nodes: vec![0] }),
        nodes: vec![node0, node1],
        up_axis: Some(Axis::Z),
        forward_axis: Some(Axis::X),
        ..Default::default()
    };

    let scene_graph = ModelSceneGraph::new(ModelSceneGraphOptions {
        components,
        ..Default::default()
    });

    assert_eq!(scene_graph.runtime_nodes[0].transform, parent_transform);
    assert_eq!(
        scene_graph.runtime_nodes[0].transform_to_root,
        Matrix4::IDENTITY
    );

    assert_eq!(scene_graph.runtime_nodes[1].transform, child_transform);
    assert_eq!(
        scene_graph.runtime_nodes[1].transform_to_root,
        parent_transform
    );

    let expected_child_computed = parent_transform.multiply_transformation(&child_transform);
    assert_eq!(
        scene_graph.runtime_nodes[1].computed_transform,
        expected_child_computed
    );
}

#[test]
fn test_creates_runtime_skin_from_model() {
    let node0 = Node {
        name: Some("root".to_string()),
        index: Some(0),
        children: vec![1, 2],
        ..Default::default()
    };
    let node1 = Node {
        name: Some("joint0".to_string()),
        index: Some(1),
        children: vec![],
        ..Default::default()
    };
    let node2 = Node {
        name: Some("joint1".to_string()),
        index: Some(2),
        children: vec![],
        skin: Some(0),
        ..Default::default()
    };

    let skin = Skin {
        index: Some(0),
        joints: vec![1, 2],
        inverse_bind_matrices: vec![Matrix4::IDENTITY, Matrix4::IDENTITY],
    };

    let components = Components {
        scene: Some(Scene { nodes: vec![0] }),
        nodes: vec![node0, node1, node2],
        skins: vec![skin],
        ..Default::default()
    };

    let scene_graph = ModelSceneGraph::new(ModelSceneGraphOptions {
        components,
        ..Default::default()
    });

    assert_eq!(scene_graph.runtime_skins.len(), 1);
    assert_eq!(scene_graph.runtime_skins[0].joints, vec![1, 2]);
    assert_eq!(scene_graph.skinned_nodes, vec![2]);
}

#[test]
fn test_builds_and_pushes_draw_commands() {
    let node0 = Node {
        name: Some("mesh_node".to_string()),
        index: Some(0),
        children: vec![],
        primitives: vec![Primitive::default(), Primitive::default()],
        ..Default::default()
    };

    let components = Components {
        scene: Some(Scene { nodes: vec![0] }),
        nodes: vec![node0],
        ..Default::default()
    };

    let mut scene_graph = ModelSceneGraph::new(ModelSceneGraphOptions {
        components,
        ..Default::default()
    });

    let frame_state = PrimitiveFrameState::default();
    scene_graph.build_draw_commands(&frame_state);
    assert!(scene_graph.draw_commands_built);

    let mut command_list: Vec<DrawCommand> = Vec::new();
    scene_graph.push_draw_commands(&mut command_list);

    assert_eq!(command_list.len(), 2);
    assert_eq!(command_list[0].pass, Some(Pass::Opaque));
    assert_eq!(command_list[1].pass, Some(Pass::Opaque));
}

#[test]
fn test_builds_draw_commands_with_translucent_style() {
    let node0 = Node {
        name: Some("mesh_node".to_string()),
        index: Some(0),
        children: vec![],
        primitives: vec![Primitive::default()],
        ..Default::default()
    };

    let components = Components {
        scene: Some(Scene { nodes: vec![0] }),
        nodes: vec![node0],
        ..Default::default()
    };

    let mut scene_graph = ModelSceneGraph::new(ModelSceneGraphOptions {
        components,
        model_parameters: ModelRuntimePrimitiveModel {
            has_style: true,
            ..Default::default()
        },
        ..Default::default()
    });

    let frame_state = PrimitiveFrameState::default();
    scene_graph.build_draw_commands(&frame_state);

    let mut command_list: Vec<DrawCommand> = Vec::new();
    scene_graph.push_draw_commands(&mut command_list);

    assert_eq!(command_list.len(), 1);
    assert_eq!(command_list[0].pass, Some(Pass::Translucent));
}

#[test]
fn test_updates_model_matrix() {
    let parent_transform = Matrix4::from_translation(Cartesian3::new(1.0, 0.0, 0.0));
    let child_transform = Matrix4::from_translation(Cartesian3::new(0.0, 2.0, 0.0));

    let node0 = Node {
        name: Some("parent".to_string()),
        index: Some(0),
        matrix: Some(parent_transform),
        children: vec![1],
        ..Default::default()
    };
    let node1 = Node {
        name: Some("child".to_string()),
        index: Some(1),
        matrix: Some(child_transform),
        children: vec![],
        ..Default::default()
    };

    let components = Components {
        scene: Some(Scene { nodes: vec![0] }),
        nodes: vec![node0, node1],
        ..Default::default()
    };

    let mut scene_graph = ModelSceneGraph::new(ModelSceneGraphOptions {
        components,
        ..Default::default()
    });

    let new_model_matrix = Matrix4::from_translation(Cartesian3::new(100.0, 0.0, 0.0));
    scene_graph.update_model_matrix(&new_model_matrix);

    assert_eq!(
        scene_graph.runtime_nodes[0].transform_to_root,
        Matrix4::IDENTITY
    );
    assert_eq!(
        scene_graph.runtime_nodes[1].transform_to_root,
        parent_transform
    );
}
