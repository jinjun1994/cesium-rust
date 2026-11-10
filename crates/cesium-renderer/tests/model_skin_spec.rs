use cesium_math::{Cartesian3, Matrix4};
use cesium_renderer::model_components::Skin;
use cesium_renderer::{ModelRuntimeNode, ModelRuntimeNodeOptions, ModelSkin, ModelSkinOptions};

#[test]
fn test_throws_for_undefined_arguments() {
    let mock_skin = Skin {
        index: Some(0),
        joints: vec![1, 2],
        inverse_bind_matrices: vec![Matrix4::IDENTITY, Matrix4::IDENTITY],
    };

    // throws for undefined skin
    let err = ModelSkin::try_new(ModelSkinOptions {
        skin: None,
        runtime_nodes: Some(&[]),
    })
    .unwrap_err();
    assert_eq!(err.message, "options.skin is required.");

    // throws for undefined scene graph
    let err = ModelSkin::try_new(ModelSkinOptions {
        skin: Some(mock_skin),
        runtime_nodes: None,
    })
    .unwrap_err();
    assert_eq!(err.message, "options.sceneGraph is required.");
}

#[test]
fn test_constructs_and_updates_joint_matrices() {
    let mock_runtime_nodes = vec![
        ModelRuntimeNode::from_options(ModelRuntimeNodeOptions {
            name: "node0".to_string(),
            id: 0,
            transform: Matrix4::IDENTITY,
            transform_to_root: Matrix4::IDENTITY,
            children: vec![],
            show: None,
        }),
        ModelRuntimeNode::from_options(ModelRuntimeNodeOptions {
            name: "node1".to_string(),
            id: 1,
            transform: Matrix4::IDENTITY,
            transform_to_root: Matrix4::IDENTITY,
            children: vec![],
            show: None,
        }),
        ModelRuntimeNode::from_options(ModelRuntimeNodeOptions {
            name: "node2".to_string(),
            id: 2,
            transform: Matrix4::IDENTITY,
            transform_to_root: Matrix4::IDENTITY,
            children: vec![],
            show: None,
        }),
    ];

    let mock_skin = Skin {
        index: Some(0),
        joints: vec![1, 2],
        inverse_bind_matrices: vec![Matrix4::IDENTITY, Matrix4::IDENTITY],
    };

    let mut skin = ModelSkin::try_new(ModelSkinOptions {
        skin: Some(mock_skin.clone()),
        runtime_nodes: Some(&mock_runtime_nodes),
    })
    .unwrap();

    assert_eq!(skin.skin.joints, mock_skin.joints);
    assert_eq!(skin.inverse_bind_matrices, mock_skin.inverse_bind_matrices);
    assert_eq!(skin.joints, vec![1, 2]);
    assert_eq!(skin.joint_matrices, vec![Matrix4::IDENTITY, Matrix4::IDENTITY]);

    // Update joint matrices with modified transforms
    let mut modified_runtime_nodes = mock_runtime_nodes.clone();
    modified_runtime_nodes[1].transform = Matrix4::from_translation(Cartesian3::new(10.0, 0.0, 0.0));

    skin.update_joint_matrices(&modified_runtime_nodes);
    assert_eq!(
        skin.joint_matrices[0],
        Matrix4::from_translation(Cartesian3::new(10.0, 0.0, 0.0))
    );
    assert_eq!(skin.joint_matrices[1], Matrix4::IDENTITY);
}
