use std::cell::{Cell, RefCell};
use std::rc::Rc;

use cesium_math::{Cartesian3, Matrix4};
use cesium_renderer::{ModelNode, ModelRuntimeNode, ModelRuntimeNodeOptions};

#[test]
fn test_throws_for_undefined_model() {
    let runtime_node = Rc::new(RefCell::new(ModelRuntimeNode::from_options(
        ModelRuntimeNodeOptions {
            name: "Mock Node".to_string(),
            id: 0,
            transform: Matrix4::IDENTITY,
            transform_to_root: Matrix4::IDENTITY,
            children: Vec::new(),
            show: Some(true),
        },
    )));
    let result = ModelNode::new(false, Some(runtime_node), None);
    assert!(result.is_err());
    assert!(result.err().unwrap().to_string().contains("model is required"));
}

#[test]
fn test_throws_for_undefined_runtime_node() {
    let result = ModelNode::new(true, None, None);
    assert!(result.is_err());
    assert!(result.err().unwrap().to_string().contains("runtimeNode is required"));
}

#[test]
fn test_constructs() {
    let node_transform = Matrix4::from_translation(Cartesian3::new(1.0, 2.0, 3.0));
    let runtime_node = Rc::new(RefCell::new(ModelRuntimeNode::from_options(
        ModelRuntimeNodeOptions {
            name: "Mock Node".to_string(),
            id: 0,
            transform: node_transform,
            transform_to_root: Matrix4::IDENTITY,
            children: Vec::new(),
            show: Some(true),
        },
    )));

    let node = ModelNode::new(true, Some(runtime_node.clone()), None).expect("valid ModelNode");
    assert_eq!(node.name(), "Mock Node");
    assert_eq!(node.id(), 0);
    assert!(node.show());
    assert_eq!(node.matrix(), runtime_node.borrow().transform);
    assert_eq!(node.original_matrix(), runtime_node.borrow().original_transform);
}

#[test]
fn test_sets_show_for_runtime_node() {
    let runtime_node = Rc::new(RefCell::new(ModelRuntimeNode::from_options(
        ModelRuntimeNodeOptions {
            name: "Mock Node".to_string(),
            id: 0,
            transform: Matrix4::IDENTITY,
            transform_to_root: Matrix4::IDENTITY,
            children: Vec::new(),
            show: Some(true),
        },
    )));

    let mut node = ModelNode::new(true, Some(runtime_node.clone()), None).expect("valid ModelNode");

    node.set_show(false);
    assert!(!runtime_node.borrow().show);

    node.set_show(true);
    assert!(runtime_node.borrow().show);
}

#[test]
fn test_sets_matrix_for_runtime_node() {
    let node_transform = Matrix4::from_translation(Cartesian3::new(1.0, 2.0, 3.0));
    let runtime_node = Rc::new(RefCell::new(ModelRuntimeNode::from_options(
        ModelRuntimeNodeOptions {
            name: "Mock Node".to_string(),
            id: 0,
            transform: node_transform,
            transform_to_root: Matrix4::IDENTITY,
            children: Vec::new(),
            show: Some(true),
        },
    )));

    let user_animation_dirty = Rc::new(Cell::new(false));
    let mut node = ModelNode::new(
        true,
        Some(runtime_node.clone()),
        Some(user_animation_dirty.clone()),
    )
    .expect("valid ModelNode");

    assert_eq!(node.matrix(), node_transform);
    assert_eq!(node.original_matrix(), node_transform);
    assert!(!runtime_node.borrow().user_animated);
    assert!(!user_animation_dirty.get());

    let matrix = Matrix4::from_translation(Cartesian3::new(10.0, 10.0, 10.0));
    node.set_matrix(Some(matrix));

    assert_eq!(node.matrix(), matrix);
    assert_eq!(node.original_matrix(), node_transform);

    let rt = runtime_node.borrow();
    assert_eq!(rt.transform, matrix);
    assert_eq!(rt.original_transform, node_transform);
    assert!(rt.user_animated);
    assert!(rt.transform_dirty);
    assert!(user_animation_dirty.get());
}

#[test]
fn test_setting_matrix_to_none_resets_node_transform() {
    let node_transform = Matrix4::from_translation(Cartesian3::new(1.0, 2.0, 3.0));
    let runtime_node = Rc::new(RefCell::new(ModelRuntimeNode::from_options(
        ModelRuntimeNodeOptions {
            name: "Mock Node".to_string(),
            id: 0,
            transform: node_transform,
            transform_to_root: Matrix4::IDENTITY,
            children: Vec::new(),
            show: Some(true),
        },
    )));

    let user_animation_dirty = Rc::new(Cell::new(false));
    let mut node = ModelNode::new(
        true,
        Some(runtime_node.clone()),
        Some(user_animation_dirty.clone()),
    )
    .expect("valid ModelNode");

    let matrix = Matrix4::from_translation(Cartesian3::new(10.0, 10.0, 10.0));
    node.set_matrix(Some(matrix));

    assert_eq!(node.matrix(), matrix);
    assert_eq!(node.original_matrix(), node_transform);

    {
        let rt = runtime_node.borrow();
        assert_eq!(rt.transform, matrix);
        assert_eq!(rt.original_transform, node_transform);
        assert!(rt.user_animated);
        assert!(user_animation_dirty.get());
    }

    // Reset matrix
    node.set_matrix(None);
    assert_eq!(node.matrix(), node.original_matrix());
    assert_eq!(node.original_matrix(), node_transform);

    let rt = runtime_node.borrow();
    assert_eq!(rt.transform, rt.original_transform);
    assert!(!rt.user_animated);
}
