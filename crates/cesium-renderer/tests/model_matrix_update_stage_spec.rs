use cesium_math::{BoundingSphere, Cartesian3, Matrix4, PrimitiveType, Quaternion};
use cesium_renderer::model_draw_command::{
    ModelDrawCommand, ModelDrawCommandMaterial, ModelDrawCommandModelOptions,
    ModelDrawCommandPrimitive, ModelDrawCommandRuntimePrimitive, ModelPrimitiveRenderResources,
};
use cesium_renderer::render_state::CullFace;
use cesium_renderer::{
    DrawCommand, DrawCommandOptions, ModelMatrixUpdateStage, ModelRuntimeNode,
    ModelRuntimeNodeOptions, ModelRuntimePrimitive,
};
use glam::DMat4;

fn create_mock_render_resources() -> ModelPrimitiveRenderResources {
    ModelPrimitiveRenderResources {
        model: ModelDrawCommandModelOptions::default(),
        runtime_primitive: ModelDrawCommandRuntimePrimitive {
            primitive: ModelDrawCommandPrimitive {
                material: ModelDrawCommandMaterial { double_sided: false },
            },
            bounding_sphere: BoundingSphere::new(Cartesian3::ZERO, 1.0),
        },
        has_silhouette: false,
        has_skip_level_of_detail: false,
        planar_fill_id_pass: false,
        edge_geometry: None,
    }
}

fn create_mock_draw_command() -> ModelDrawCommand {
    let mut raw_cmd = DrawCommand::new(DrawCommandOptions {
        model_matrix: Some(DMat4::IDENTITY),
        bounding_volume: Some(BoundingSphere::new(Cartesian3::ZERO, 1.0)),
        primitive_type: Some(PrimitiveType::Triangles),
        ..Default::default()
    });
    raw_cmd.primitive_type = PrimitiveType::Triangles;
    let rr = create_mock_render_resources();
    ModelDrawCommand::new(Some(raw_cmd), Some(rr)).expect("valid ModelDrawCommand")
}

fn setup_simple_tree() -> Vec<ModelRuntimeNode> {
    let static_leaf = ModelRuntimeNode::from_options(ModelRuntimeNodeOptions {
        name: "StaticLeaf".to_string(),
        id: 0,
        transform: Matrix4::IDENTITY,
        transform_to_root: Matrix4::IDENTITY,
        children: Vec::new(),
        show: Some(true),
    });

    let parent_root = ModelRuntimeNode::from_options(ModelRuntimeNodeOptions {
        name: "ParentRoot".to_string(),
        id: 1,
        transform: Matrix4::IDENTITY,
        transform_to_root: Matrix4::IDENTITY,
        children: vec![2],
        show: Some(true),
    });

    let child_leaf = ModelRuntimeNode::from_options(ModelRuntimeNodeOptions {
        name: "ChildLeaf".to_string(),
        id: 2,
        transform: Matrix4::IDENTITY,
        transform_to_root: Matrix4::IDENTITY,
        children: Vec::new(),
        show: Some(true),
    });

    let mut nodes = vec![static_leaf, parent_root, child_leaf];

    for node in &mut nodes {
        let cmd = create_mock_draw_command();
        let bs = Some(BoundingSphere::new(Cartesian3::ZERO, 1.0));
        node.runtime_primitives.push(ModelRuntimePrimitive::with_command(cmd, bs));
    }

    nodes
}

#[test]
fn test_updates_leaf_nodes_using_node_transform_setter() {
    let mut nodes = setup_simple_tree();
    let root_matrix = Matrix4::IDENTITY;

    let node = &mut nodes[0];
    let original_transform = node.original_transform;
    assert!(!node.transform_dirty);

    let transform = Matrix4::from_translation(Cartesian3::new(0.0, 5.0, 0.0));
    let new_transform = node.transform.multiply(&transform);
    node.set_transform(new_transform);
    assert!(node.transform_dirty);
    assert_eq!(node.original_transform, original_transform);

    let expected_computed_transform = root_matrix.multiply(&node.transform);
    let initial_cmd_matrix = Matrix4::from(nodes[0].runtime_primitives[0]
        .draw_command
        .as_ref()
        .unwrap()
        .model_matrix());
    let expected_model_matrix = initial_cmd_matrix.multiply(&transform);

    let primitive_bs = nodes[0].runtime_primitives[0].bounding_sphere.unwrap();
    let expected_bounding_sphere = primitive_bs.transform(expected_computed_transform.into()).unwrap();

    ModelMatrixUpdateStage::update(0, &mut nodes, &root_matrix, false, false);

    let draw_cmd = nodes[0].runtime_primitives[0]
        .draw_command
        .as_ref()
        .unwrap();

    let actual_model_matrix = Matrix4::from(draw_cmd.model_matrix());
    assert!(actual_model_matrix.equals_epsilon(&expected_model_matrix, 1e-14));
    assert_eq!(draw_cmd.bounding_volume(), expected_bounding_sphere);
    assert!(!nodes[0].transform_dirty);
}

#[test]
fn test_updates_nodes_with_children_using_node_transform_setter() {
    let mut nodes = setup_simple_tree();
    let root_matrix = Matrix4::IDENTITY;

    let child_trans = Matrix4::from_translation(Cartesian3::new(0.0, 5.0, 0.0));
    let new_child = nodes[2].transform.multiply(&child_trans);
    nodes[2].set_transform(new_child);

    let root_trans = Matrix4::from_translation(Cartesian3::new(12.0, 5.0, 0.0));
    let new_root = nodes[1].transform.multiply(&root_trans);
    nodes[1].set_transform(new_root);

    let initial_root_cmd_matrix = Matrix4::from(nodes[1].runtime_primitives[0]
        .draw_command
        .as_ref()
        .unwrap()
        .model_matrix());
    let initial_static_cmd_matrix = Matrix4::from(nodes[0].runtime_primitives[0]
        .draw_command
        .as_ref()
        .unwrap()
        .model_matrix());
    let initial_child_cmd_matrix = Matrix4::from(nodes[2].runtime_primitives[0]
        .draw_command
        .as_ref()
        .unwrap()
        .model_matrix());

    let expected_root_matrix = root_trans.multiply(&initial_root_cmd_matrix);
    let expected_static_matrix = initial_static_cmd_matrix;
    let final_transform = root_trans.multiply(&child_trans);
    let expected_child_matrix = final_transform.multiply(&initial_child_cmd_matrix);

    // Update parent root
    ModelMatrixUpdateStage::update(1, &mut nodes, &root_matrix, false, false);

    let root_draw_cmd = nodes[1].runtime_primitives[0]
        .draw_command
        .as_ref()
        .unwrap();
    let static_draw_cmd = nodes[0].runtime_primitives[0]
        .draw_command
        .as_ref()
        .unwrap();
    let child_draw_cmd = nodes[2].runtime_primitives[0]
        .draw_command
        .as_ref()
        .unwrap();

    assert_eq!(Matrix4::from(root_draw_cmd.model_matrix()), expected_root_matrix);
    assert_eq!(Matrix4::from(static_draw_cmd.model_matrix()), expected_static_matrix);
    assert_eq!(Matrix4::from(child_draw_cmd.model_matrix()), expected_child_matrix);
}

#[test]
fn test_updates_with_new_model_matrix() {
    let mut nodes = setup_simple_tree();

    let rotation = Quaternion::from_axis_angle(
        Cartesian3::UNIT_Y,
        std::f64::consts::PI,
    );
    let model_matrix = Matrix4::from_translation_quaternion_rotation_scale(
        Cartesian3::new(10.0, 0.0, 0.0),
        rotation,
        Cartesian3::new(1.0, 1.0, 1.0),
    );

    let initial_root_matrix = Matrix4::from(nodes[1].runtime_primitives[0]
        .draw_command
        .as_ref()
        .unwrap()
        .model_matrix());
    let initial_static_matrix = Matrix4::from(nodes[0].runtime_primitives[0]
        .draw_command
        .as_ref()
        .unwrap()
        .model_matrix());
    let initial_child_matrix = Matrix4::from(nodes[2].runtime_primitives[0]
        .draw_command
        .as_ref()
        .unwrap()
        .model_matrix());

    let expected_root = model_matrix.multiply(&initial_root_matrix);
    let expected_static = model_matrix.multiply(&initial_static_matrix);
    let expected_child = model_matrix.multiply(&initial_child_matrix);

    nodes[0].transform_dirty = true;
    nodes[1].transform_dirty = true;

    ModelMatrixUpdateStage::update(0, &mut nodes, &model_matrix, false, false);
    ModelMatrixUpdateStage::update(1, &mut nodes, &model_matrix, false, false);

    assert_eq!(
        Matrix4::from(nodes[1].runtime_primitives[0]
            .draw_command
            .as_ref()
            .unwrap()
            .model_matrix()),
        expected_root
    );
    assert_eq!(
        Matrix4::from(nodes[0].runtime_primitives[0]
            .draw_command
            .as_ref()
            .unwrap()
            .model_matrix()),
        expected_static
    );
    assert_eq!(
        Matrix4::from(nodes[2].runtime_primitives[0]
            .draw_command
            .as_ref()
            .unwrap()
            .model_matrix()),
        expected_child
    );
}

#[test]
fn test_updates_render_state_cull_face_when_scale_is_negative() {
    let mut nodes = setup_simple_tree();

    assert_eq!(
        nodes[1].runtime_primitives[0]
            .draw_command
            .as_ref()
            .unwrap()
            .cull_face(),
        CullFace::Back
    );
    assert_eq!(
        nodes[2].runtime_primitives[0]
            .draw_command
            .as_ref()
            .unwrap()
            .cull_face(),
        CullFace::Back
    );

    // Negative uniform scale flips triangles orientation
    let neg_scale_matrix = Matrix4::from_uniform_scale(-1.0);
    nodes[1].transform_dirty = true;

    ModelMatrixUpdateStage::update(1, &mut nodes, &neg_scale_matrix, false, false);

    assert_eq!(
        nodes[1].runtime_primitives[0]
            .draw_command
            .as_ref()
            .unwrap()
            .cull_face(),
        CullFace::Front
    );
    assert_eq!(
        nodes[2].runtime_primitives[0]
            .draw_command
            .as_ref()
            .unwrap()
            .cull_face(),
        CullFace::Front
    );
}
