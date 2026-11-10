use cesium_math::{Cartesian3, Matrix4, Quaternion};
use cesium_renderer::model_components::{Instances, Node, NodeVisibility};
use cesium_renderer::{
    ModelRuntimeNode, ModelRuntimeNodeArgs, NodePipelineStage, NodeUpdateStage,
};

fn verify_transforms(
    transform: Matrix4,
    transform_to_root: Matrix4,
    runtime_node: &ModelRuntimeNode,
    original_transform: Option<Matrix4>,
) {
    let original = original_transform.unwrap_or(transform);
    assert_eq!(runtime_node.transform, transform);
    assert_eq!(runtime_node.original_transform, original);
    assert_eq!(runtime_node.transform_to_root, transform_to_root);

    let expected_computed = transform_to_root.multiply_transformation(&transform);
    assert_eq!(runtime_node.computed_transform, expected_computed);
}

#[test]
fn test_throws_for_undefined_arguments() {
    let base_node = Node::default();
    let transform = Matrix4::IDENTITY;
    let transform_to_root = Matrix4::IDENTITY;

    // throws for undefined node
    let err = ModelRuntimeNode::try_new(ModelRuntimeNodeArgs {
        node: None,
        transform: Some(transform),
        transform_to_root: Some(transform_to_root),
        children: Some(vec![]),
        scene_graph: Some(()),
    })
    .unwrap_err();
    assert_eq!(err.message, "options.node is required.");

    // throws for undefined transform
    let err = ModelRuntimeNode::try_new(ModelRuntimeNodeArgs {
        node: Some(base_node.clone()),
        transform: None,
        transform_to_root: Some(transform_to_root),
        children: Some(vec![]),
        scene_graph: Some(()),
    })
    .unwrap_err();
    assert_eq!(err.message, "options.transform is required.");

    // throws for undefined transform_to_root
    let err = ModelRuntimeNode::try_new(ModelRuntimeNodeArgs {
        node: Some(base_node.clone()),
        transform: Some(transform),
        transform_to_root: None,
        children: Some(vec![]),
        scene_graph: Some(()),
    })
    .unwrap_err();
    assert_eq!(err.message, "options.transformToRoot is required.");

    // throws for undefined scene_graph
    let err = ModelRuntimeNode::try_new(ModelRuntimeNodeArgs {
        node: Some(base_node.clone()),
        transform: Some(transform),
        transform_to_root: Some(transform_to_root),
        children: Some(vec![]),
        scene_graph: None,
    })
    .unwrap_err();
    assert_eq!(err.message, "options.sceneGraph is required.");

    // throws for undefined children
    let err = ModelRuntimeNode::try_new(ModelRuntimeNodeArgs {
        node: Some(base_node),
        transform: Some(transform),
        transform_to_root: Some(transform_to_root),
        children: None,
        scene_graph: Some(()),
    })
    .unwrap_err();
    assert_eq!(err.message, "options.children is required.");
}

#[test]
fn test_constructs_with_matrix() {
    let mock_node = Node {
        matrix: Some(Matrix4::IDENTITY),
        ..Default::default()
    };
    let transform = Matrix4::IDENTITY;
    let transform_to_root = Matrix4::from_translation(Cartesian3::new(1.0, 2.0, 3.0));

    let mut runtime_node = ModelRuntimeNode::try_new(ModelRuntimeNodeArgs {
        node: Some(mock_node),
        transform: Some(transform),
        transform_to_root: Some(transform_to_root),
        children: Some(vec![]),
        scene_graph: Some(()),
    })
    .unwrap();

    assert_eq!(runtime_node.children.len(), 0);
    verify_transforms(transform, transform_to_root, &runtime_node, None);

    assert!(runtime_node.show);
    assert!(!runtime_node.user_animated);

    runtime_node.configure_pipeline();
    assert_eq!(
        runtime_node.pipeline_stages,
        vec![NodePipelineStage::NodeStatistics]
    );
    assert_eq!(
        runtime_node.update_stages,
        vec![NodeUpdateStage::ModelMatrixUpdate]
    );
    assert_eq!(runtime_node.runtime_primitives.len(), 0);

    // When matrix is defined, translation, rotation, scale parameters are undefined
    assert_eq!(runtime_node.get_translation(), None);
    assert_eq!(runtime_node.get_rotation(), None);
    assert_eq!(runtime_node.get_scale(), None);
    assert_eq!(runtime_node.morph_weights.len(), 0);
}

#[test]
fn test_constructs_with_default_transform_parameters() {
    let mock_node_no_matrix = Node {
        matrix: None,
        ..Default::default()
    };
    let transform = Matrix4::IDENTITY;
    let transform_to_root = Matrix4::IDENTITY;

    let runtime_node = ModelRuntimeNode::try_new(ModelRuntimeNodeArgs {
        node: Some(mock_node_no_matrix),
        transform: Some(transform),
        transform_to_root: Some(transform_to_root),
        children: Some(vec![]),
        scene_graph: Some(()),
    })
    .unwrap();

    verify_transforms(transform, transform_to_root, &runtime_node, None);
    assert_eq!(runtime_node.get_translation(), Some(Cartesian3::ZERO));
    assert_eq!(runtime_node.get_rotation(), Some(Quaternion::IDENTITY));
    assert_eq!(
        runtime_node.get_scale(),
        Some(Cartesian3::new(1.0, 1.0, 1.0))
    );
}

#[test]
fn test_constructs_with_given_transform_parameters() {
    let translation = Cartesian3::new(1.0, 2.0, 3.0);
    let rotation = Quaternion::new(0.707, 0.0, 0.707, 0.0);
    let scale = Cartesian3::new(1.0, 1.0, 2.0);

    let mock_node = Node {
        matrix: None,
        translation: Some(translation),
        rotation: Some(rotation),
        scale: Some(scale),
        ..Default::default()
    };
    let transform = Matrix4::IDENTITY;
    let transform_to_root = Matrix4::IDENTITY;

    let runtime_node = ModelRuntimeNode::try_new(ModelRuntimeNodeArgs {
        node: Some(mock_node),
        transform: Some(transform),
        transform_to_root: Some(transform_to_root),
        children: Some(vec![]),
        scene_graph: Some(()),
    })
    .unwrap();

    verify_transforms(transform, transform_to_root, &runtime_node, None);
    assert_eq!(runtime_node.get_translation(), Some(translation));
    assert_eq!(runtime_node.get_rotation(), Some(rotation));
    assert_eq!(runtime_node.get_scale(), Some(scale));
}

#[test]
fn test_setting_transform_parameters_throws_if_constructed_with_matrix() {
    let mock_node = Node {
        matrix: Some(Matrix4::IDENTITY),
        ..Default::default()
    };
    let mut runtime_node = ModelRuntimeNode::try_new(ModelRuntimeNodeArgs {
        node: Some(mock_node),
        transform: Some(Matrix4::IDENTITY),
        transform_to_root: Some(Matrix4::IDENTITY),
        children: Some(vec![]),
        scene_graph: Some(()),
    })
    .unwrap();

    assert!(runtime_node
        .set_translation(Cartesian3::new(1.0, 2.0, 3.0))
        .is_err());
    assert!(runtime_node.set_rotation(Quaternion::IDENTITY).is_err());
    assert!(runtime_node
        .set_scale(Cartesian3::new(1.0, 1.0, 2.0))
        .is_err());
}

#[test]
fn test_setting_morph_weights_throws_for_different_length() {
    let mock_node = Node {
        morph_weights: vec![0.0, 0.0],
        ..Default::default()
    };
    let mut runtime_node = ModelRuntimeNode::try_new(ModelRuntimeNodeArgs {
        node: Some(mock_node),
        transform: Some(Matrix4::IDENTITY),
        transform_to_root: Some(Matrix4::IDENTITY),
        children: Some(vec![]),
        scene_graph: Some(()),
    })
    .unwrap();

    assert!(runtime_node.set_morph_weights(&[0.0, 1.0, 2.0]).is_err());
    assert!(runtime_node.set_morph_weights(&[0.5, 0.5]).is_ok());
    assert_eq!(runtime_node.morph_weights, vec![0.5, 0.5]);
}

#[test]
fn test_sets_translation_and_updates_computed_transform() {
    let mock_node = Node {
        translation: Some(Cartesian3::ZERO),
        ..Default::default()
    };
    let transform = Matrix4::IDENTITY;
    let transform_to_root = Matrix4::IDENTITY;

    let mut runtime_node = ModelRuntimeNode::try_new(ModelRuntimeNodeArgs {
        node: Some(mock_node),
        transform: Some(transform),
        transform_to_root: Some(transform_to_root),
        children: Some(vec![]),
        scene_graph: Some(()),
    })
    .unwrap();

    assert_eq!(runtime_node.get_translation(), Some(Cartesian3::ZERO));

    let translation = Cartesian3::new(1.0, 2.0, 3.0);
    runtime_node.set_translation(translation).unwrap();
    runtime_node.update_computed_transform();

    let translation_matrix = Matrix4::from_translation(translation);
    assert_eq!(runtime_node.get_translation(), Some(translation));
    verify_transforms(
        translation_matrix,
        transform_to_root,
        &runtime_node,
        Some(transform),
    );
}

#[test]
fn test_sets_rotation_and_scale() {
    let mock_node = Node {
        rotation: Some(Quaternion::IDENTITY),
        scale: Some(Cartesian3::new(1.0, 1.0, 1.0)),
        ..Default::default()
    };
    let mut runtime_node = ModelRuntimeNode::try_new(ModelRuntimeNodeArgs {
        node: Some(mock_node),
        transform: Some(Matrix4::IDENTITY),
        transform_to_root: Some(Matrix4::IDENTITY),
        children: Some(vec![]),
        scene_graph: Some(()),
    })
    .unwrap();

    let rotation = Quaternion::new(0.707, 0.0, 0.707, 0.0);
    runtime_node.set_rotation(rotation).unwrap();
    assert_eq!(runtime_node.get_rotation(), Some(rotation));

    let scale = Cartesian3::new(2.0, 3.0, 4.0);
    runtime_node.set_scale(scale).unwrap();
    assert_eq!(runtime_node.get_scale(), Some(scale));

    runtime_node.update_computed_transform();
    let expected_matrix = Matrix4::from_translation_quaternion_rotation_scale(
        Cartesian3::ZERO,
        rotation,
        scale,
    );
    assert_eq!(runtime_node.transform, expected_matrix);
}

#[test]
fn test_adds_instancing_pipeline_stage_if_node_is_instanced() {
    let mock_node = Node {
        instances: Some(Instances::default()),
        ..Default::default()
    };
    let mut runtime_node = ModelRuntimeNode::try_new(ModelRuntimeNodeArgs {
        node: Some(mock_node),
        transform: Some(Matrix4::IDENTITY),
        transform_to_root: Some(Matrix4::IDENTITY),
        children: Some(vec![]),
        scene_graph: Some(()),
    })
    .unwrap();

    runtime_node.configure_pipeline();
    assert_eq!(
        runtime_node.pipeline_stages,
        vec![
            NodePipelineStage::Instancing,
            NodePipelineStage::NodeStatistics
        ]
    );
    assert_eq!(
        runtime_node.update_stages,
        vec![NodeUpdateStage::ModelMatrixUpdate]
    );
}

#[test]
fn test_get_child() {
    let mock_node = Node::default();
    let runtime_node = ModelRuntimeNode::try_new(ModelRuntimeNodeArgs {
        node: Some(mock_node),
        transform: Some(Matrix4::IDENTITY),
        transform_to_root: Some(Matrix4::IDENTITY),
        children: Some(vec![42]),
        scene_graph: Some(()),
    })
    .unwrap();

    assert_eq!(runtime_node.get_child(0).unwrap(), 42);
    assert!(runtime_node.get_child(1).is_err());
}

#[test]
fn test_sets_transform_without_replacing_original() {
    let mock_node = Node::default();
    let original = Matrix4::IDENTITY;
    let mut runtime_node = ModelRuntimeNode::try_new(ModelRuntimeNodeArgs {
        node: Some(mock_node),
        transform: Some(original),
        transform_to_root: Some(Matrix4::IDENTITY),
        children: Some(vec![]),
        scene_graph: Some(()),
    })
    .unwrap();

    let new_transform = Matrix4::from_translation(Cartesian3::new(10.0, 0.0, 0.0));
    runtime_node.set_transform(new_transform);

    assert!(runtime_node.transform_dirty);
    assert_eq!(runtime_node.transform, new_transform);
    assert_eq!(runtime_node.original_transform, original);

    runtime_node.update_computed_transform();
    assert!(!runtime_node.transform_dirty);
    assert_eq!(runtime_node.computed_transform, new_transform);
}

#[test]
fn test_node_visibility_handling() {
    let base_args = || ModelRuntimeNodeArgs {
        node: Some(Node::default()),
        transform: Some(Matrix4::IDENTITY),
        transform_to_root: Some(Matrix4::IDENTITY),
        children: Some(vec![]),
        scene_graph: Some(()),
    };

    // 1. show is true by default
    let node1 = ModelRuntimeNode::try_new(base_args()).unwrap();
    assert!(node1.show);

    // 2. show is true when nodeVisibility.visible is None
    let mut args2 = base_args();
    args2.node = Some(Node {
        node_visibility: Some(NodeVisibility { visible: None }),
        ..Default::default()
    });
    let node2 = ModelRuntimeNode::try_new(args2).unwrap();
    assert!(node2.show);

    // 3. show is true when visible is true
    let mut args3 = base_args();
    args3.node = Some(Node {
        node_visibility: Some(NodeVisibility {
            visible: Some(true),
        }),
        ..Default::default()
    });
    let node3 = ModelRuntimeNode::try_new(args3).unwrap();
    assert!(node3.show);

    // 4. show is false only when visible is explicitly false
    let mut args4 = base_args();
    args4.node = Some(Node {
        node_visibility: Some(NodeVisibility {
            visible: Some(false),
        }),
        ..Default::default()
    });
    let node4 = ModelRuntimeNode::try_new(args4).unwrap();
    assert!(!node4.show);
}
