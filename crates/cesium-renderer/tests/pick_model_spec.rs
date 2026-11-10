use cesium_math::{
    AttributeType, BoundingSphere, Cartesian3, ComponentDatatype, Ellipsoid, GeographicProjection,
    IndexDatatype, Matrix4, Ray, SceneMode,
};
use cesium_renderer::model_components::{
    Attribute, Components, Indices, Instances, Node, Primitive, Scene,
};
use cesium_renderer::model_runtime_node::ModelRuntimeNode;
use cesium_renderer::model_runtime_primitive::ModelRuntimePrimitive;
use cesium_renderer::model_scene_graph::{ModelSceneGraph, ModelSceneGraphOptions};
use cesium_renderer::pick_model::{pick_model, Model, PickFrameState};

fn create_ray(origin: Cartesian3, direction: Cartesian3) -> Ray {
    Ray::new(origin.into(), direction.into()).unwrap()
}

fn create_position_attribute(positions: &[f32]) -> Attribute {
    let mut bytes = Vec::with_capacity(positions.len() * 4);
    for &p in positions {
        bytes.extend_from_slice(&p.to_le_bytes());
    }
    Attribute {
        semantic: Some("POSITION".to_string()),
        count: positions.len() / 3,
        typed_array: Some(bytes),
        component_datatype: Some(ComponentDatatype::Float),
        attribute_type: Some(AttributeType::Vec3),
        ..Default::default()
    }
}

fn create_indices(indices: &[u32]) -> Indices {
    let mut bytes = Vec::with_capacity(indices.len() * 4);
    for &idx in indices {
        bytes.extend_from_slice(&idx.to_le_bytes());
    }
    Indices {
        count: indices.len(),
        typed_array: Some(bytes),
        index_datatype: Some(IndexDatatype::UnsignedInt),
        ..Default::default()
    }
}

fn create_triangle_model(
    positions: &[f32],
    indices: &[u32],
    node_transform: Matrix4,
    back_face_culling: bool,
) -> Model {
    let primitive = Primitive {
        attributes: vec![create_position_attribute(positions)],
        indices: Some(create_indices(indices)),
        ..Default::default()
    };

    let runtime_primitive = ModelRuntimePrimitive {
        primitive: Some(primitive),
        bounding_sphere: Some(BoundingSphere::new(Cartesian3::ZERO, 10.0)),
        ..Default::default()
    };

    let runtime_node = ModelRuntimeNode {
        name: "test_node".to_string(),
        id: 0,
        node: Some(Node::default()),
        has_transform_parameters: false,
        translation: Cartesian3::ZERO,
        rotation: cesium_math::Quaternion::IDENTITY,
        scale: Cartesian3::new(1.0, 1.0, 1.0),
        morph_weights: Vec::new(),
        original_transform: node_transform,
        transform: node_transform,
        transform_to_root: node_transform,
        computed_transform: node_transform,
        transform_dirty: false,
        show: true,
        user_animated: false,
        children: Vec::new(),
        runtime_primitives: vec![runtime_primitive],
        pipeline_stages: Vec::new(),
        update_stages: Vec::new(),
        is_instanced: false,
        instancing_transforms_buffer: None,
        instancing_transforms_buffer_2d: None,
        instancing_translation_buffer_2d: None,
    };

    let mut scene_graph = ModelSceneGraph::new(ModelSceneGraphOptions {
        components: Components {
            scene: Some(Scene { nodes: vec![0] }),
            nodes: vec![Node::default()],
            ..Default::default()
        },
        ..Default::default()
    });
    scene_graph.axis_correction_matrix = Matrix4::IDENTITY;
    scene_graph.computed_model_matrix = Matrix4::IDENTITY;
    scene_graph.runtime_nodes = vec![runtime_node];
    scene_graph.bounding_sphere = Some(BoundingSphere::new(Cartesian3::ZERO, 10.0));

    Model {
        ready: true,
        scene_graph: Some(scene_graph),
        back_face_culling: Some(back_face_culling),
        bounding_sphere: Some(BoundingSphere::new(Cartesian3::ZERO, 10.0)),
        model_matrix: Matrix4::IDENTITY,
    }
}

#[test]
fn test_returns_none_if_model_not_ready() {
    let mut model = create_triangle_model(
        &[-1.0, -1.0, 0.0, 1.0, -1.0, 0.0, 0.0, 1.0, 0.0],
        &[0, 1, 2],
        Matrix4::IDENTITY,
        true,
    );
    model.ready = false;

    let ray = create_ray(Cartesian3::new(0.0, 0.0, 5.0), Cartesian3::new(0.0, 0.0, -1.0));
    let frame_state = PickFrameState::default();

    assert!(pick_model(&model, &ray, &frame_state, None, None, None, None).is_none());
}

#[test]
fn test_returns_none_when_morphing() {
    let model = create_triangle_model(
        &[-1.0, -1.0, 0.0, 1.0, -1.0, 0.0, 0.0, 1.0, 0.0],
        &[0, 1, 2],
        Matrix4::IDENTITY,
        true,
    );

    let ray = create_ray(Cartesian3::new(0.0, 0.0, 5.0), Cartesian3::new(0.0, 0.0, -1.0));
    let frame_state = PickFrameState::new_3d(SceneMode::Morphing);

    assert!(pick_model(&model, &ray, &frame_state, None, None, None, None).is_none());
}

#[test]
fn test_returns_none_if_ray_does_not_intersect() {
    let model = create_triangle_model(
        &[-1.0, -1.0, 0.0, 1.0, -1.0, 0.0, 0.0, 1.0, 0.0],
        &[0, 1, 2],
        Matrix4::IDENTITY,
        true,
    );

    // Ray pointing away
    let ray = create_ray(Cartesian3::new(10.0, 10.0, 5.0), Cartesian3::new(0.0, 0.0, 1.0));
    let frame_state = PickFrameState::default();

    assert!(pick_model(&model, &ray, &frame_state, None, None, None, None).is_none());
}

#[test]
fn test_returns_intersection_point() {
    // Triangle counter-clockwise in XY plane, facing +Z
    let model = create_triangle_model(
        &[-1.0, -1.0, 0.0, 1.0, -1.0, 0.0, 0.0, 1.0, 0.0],
        &[0, 1, 2],
        Matrix4::IDENTITY,
        true,
    );

    let ray = create_ray(Cartesian3::new(0.0, 0.0, 5.0), Cartesian3::new(0.0, 0.0, -1.0));
    let frame_state = PickFrameState::default();

    let hit = pick_model(&model, &ray, &frame_state, None, None, None, None);
    assert!(hit.is_some());
    let pt = hit.unwrap();
    assert!((pt.x - 0.0).abs() < 1e-6);
    assert!((pt.y - 0.0).abs() < 1e-6);
    assert!((pt.z - 0.0).abs() < 1e-6);
}

#[test]
fn test_returns_intersection_accounting_for_node_transforms() {
    let transform = Matrix4::from_translation(Cartesian3::new(0.0, 5.0, 2.0));
    let model = create_triangle_model(
        &[-1.0, -1.0, 0.0, 1.0, -1.0, 0.0, 0.0, 1.0, 0.0],
        &[0, 1, 2],
        transform,
        true,
    );

    let ray = create_ray(Cartesian3::new(0.0, 5.0, 10.0), Cartesian3::new(0.0, 0.0, -1.0));
    let frame_state = PickFrameState::default();

    let hit = pick_model(&model, &ray, &frame_state, None, None, None, None);
    assert!(hit.is_some());
    let pt = hit.unwrap();
    assert!((pt.x - 0.0).abs() < 1e-6);
    assert!((pt.y - 5.0).abs() < 1e-6);
    assert!((pt.z - 2.0).abs() < 1e-6);
}

#[test]
fn test_backface_culling() {
    // Triangle facing +Z
    let mut model = create_triangle_model(
        &[-1.0, -1.0, 0.0, 1.0, -1.0, 0.0, 0.0, 1.0, 0.0],
        &[0, 1, 2],
        Matrix4::IDENTITY,
        true,
    );

    // Ray coming from behind (-Z towards +Z)
    let ray = create_ray(Cartesian3::new(0.0, 0.0, -5.0), Cartesian3::new(0.0, 0.0, 1.0));
    let frame_state = PickFrameState::default();

    // With backface culling enabled -> None
    assert!(pick_model(&model, &ray, &frame_state, None, None, None, None).is_none());

    // With backface culling disabled -> Hits triangle at (0, 0, 0)
    model.back_face_culling = Some(false);
    let hit = pick_model(&model, &ray, &frame_state, None, None, None, None);
    assert!(hit.is_some());
    let pt = hit.unwrap();
    assert!((pt.z - 0.0).abs() < 1e-6);
}

#[test]
fn test_uses_result_parameter() {
    let model = create_triangle_model(
        &[-1.0, -1.0, 0.0, 1.0, -1.0, 0.0, 0.0, 1.0, 0.0],
        &[0, 1, 2],
        Matrix4::IDENTITY,
        true,
    );

    let ray = create_ray(Cartesian3::new(0.0, 0.0, 5.0), Cartesian3::new(0.0, 0.0, -1.0));
    let frame_state = PickFrameState::default();

    let mut result_cartesian = Cartesian3::ZERO;
    let returned = pick_model(
        &model,
        &ray,
        &frame_state,
        None,
        None,
        None,
        Some(&mut result_cartesian),
    );

    assert_eq!(returned, Some(result_cartesian));
    assert!((result_cartesian.z - 0.0).abs() < 1e-6);
}

#[test]
fn test_vertical_exaggeration() {
    let model = create_triangle_model(
        &[-1.0, -1.0, 0.0, 1.0, -1.0, 0.0, 0.0, 1.0, 0.0],
        &[0, 1, 2],
        Matrix4::IDENTITY,
        false,
    );

    let ray = create_ray(Cartesian3::new(0.0, 0.0, 5.0), Cartesian3::new(0.0, 0.0, -1.0));
    let frame_state = PickFrameState::default();

    let hit = pick_model(
        &model,
        &ray,
        &frame_state,
        Some(2.0),
        Some(0.0),
        Some(&Ellipsoid::default()),
        None,
    );

    assert!(hit.is_some());
}

#[test]
fn test_returns_intersection_with_instanced_model() {
    let primitive = Primitive {
        attributes: vec![create_position_attribute(&[
            -1.0, -1.0, 0.0, 1.0, -1.0, 0.0, 0.0, 1.0, 0.0,
        ])],
        indices: Some(create_indices(&[0, 1, 2])),
        ..Default::default()
    };

    let runtime_primitive = ModelRuntimePrimitive {
        primitive: Some(primitive),
        bounding_sphere: None,
        ..Default::default()
    };

    // Instances: two instances, one at (0, 0, 0), one at (0, 10, 0)
    let inst_trans = vec![0.0f32, 0.0, 0.0, 0.0, 10.0, 0.0];
    let mut trans_bytes = Vec::new();
    for &val in &inst_trans {
        trans_bytes.extend_from_slice(&val.to_le_bytes());
    }
    let trans_attr = Attribute {
        semantic: Some("TRANSLATION".to_string()),
        count: 2,
        typed_array: Some(trans_bytes),
        component_datatype: Some(ComponentDatatype::Float),
        attribute_type: Some(AttributeType::Vec3),
        ..Default::default()
    };

    let node = Node {
        instances: Some(Instances {
            attributes: vec![trans_attr],
            feature_ids: Vec::new(),
            transform_in_world_space: false,
        }),
        ..Default::default()
    };

    let runtime_node = ModelRuntimeNode {
        name: "instanced_node".to_string(),
        id: 0,
        node: Some(node),
        has_transform_parameters: false,
        translation: Cartesian3::ZERO,
        rotation: cesium_math::Quaternion::IDENTITY,
        scale: Cartesian3::new(1.0, 1.0, 1.0),
        morph_weights: Vec::new(),
        original_transform: Matrix4::IDENTITY,
        transform: Matrix4::IDENTITY,
        transform_to_root: Matrix4::IDENTITY,
        computed_transform: Matrix4::IDENTITY,
        transform_dirty: false,
        show: true,
        user_animated: false,
        children: Vec::new(),
        runtime_primitives: vec![runtime_primitive],
        pipeline_stages: Vec::new(),
        update_stages: Vec::new(),
        is_instanced: true,
        instancing_transforms_buffer: None,
        instancing_transforms_buffer_2d: None,
        instancing_translation_buffer_2d: None,
    };

    let mut scene_graph = ModelSceneGraph::new(ModelSceneGraphOptions::default());
    scene_graph.axis_correction_matrix = Matrix4::IDENTITY;
    scene_graph.computed_model_matrix = Matrix4::IDENTITY;
    scene_graph.runtime_nodes = vec![runtime_node];

    let model = Model {
        ready: true,
        scene_graph: Some(scene_graph),
        back_face_culling: Some(true),
        bounding_sphere: None,
        model_matrix: Matrix4::IDENTITY,
    };

    // Ray aimed at instance 1 at (0, 10, 0)
    let ray = create_ray(Cartesian3::new(0.0, 10.0, 5.0), Cartesian3::new(0.0, 0.0, -1.0));
    let frame_state = PickFrameState::default();

    let hit = pick_model(&model, &ray, &frame_state, None, None, None, None);
    assert!(hit.is_some());
    let pt = hit.unwrap();
    assert!((pt.y - 10.0).abs() < 1e-6);
}

#[test]
fn test_columbus_view_projection() {
    let surface_pos = Cartesian3::new(6378137.0, 0.0, 0.0);
    let transform = Matrix4::from_translation(surface_pos);
    let model = create_triangle_model(
        &[-1.0, -1.0, 0.0, 1.0, -1.0, 0.0, 0.0, 1.0, 0.0],
        &[0, 1, 2],
        transform,
        false,
    );

    let proj = GeographicProjection::default();
    // Basis to 2D projects surface_pos to 2D
    if let Ok(m2d) = cesium_math::FixedFrameTransforms::basis_to_2d(&proj, &transform) {
        let center_2d = m2d.get_translation();
        let ray = create_ray(
            Cartesian3::new(center_2d.x, center_2d.y, center_2d.z + 5.0),
            Cartesian3::new(0.0, 0.0, -1.0),
        );
        let frame_state = PickFrameState::new(SceneMode::ColumbusView, Some(&proj));

        let hit = pick_model(&model, &ray, &frame_state, None, None, None, None);
        assert!(hit.is_some());
    }
}
