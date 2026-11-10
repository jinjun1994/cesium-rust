use cesium_math::{
    AttributeType, Cartesian3, ComponentDatatype, Matrix4, PrimitiveType, EPSILON9,
};
use cesium_renderer::lighting_pipeline_stage::LightingModel;
use cesium_renderer::model_components::{Attribute, Indices, Node, Primitive};
use cesium_renderer::model_render_resources::ModelRenderResources;
use cesium_renderer::model_runtime_node::{ModelRuntimeNode, ModelRuntimeNodeArgs};
use cesium_renderer::model_runtime_primitive::{
    ModelRuntimePrimitive, ModelRuntimePrimitiveArgs, ModelRuntimePrimitiveModel,
};
use cesium_renderer::node_render_resources::NodeRenderResources;
use cesium_renderer::primitive_render_resources::PrimitiveRenderResources;
use cesium_renderer::render_state::{
    BlendingState, CullFace, DepthFunction, DepthTestState, RenderState,
};
use cesium_renderer::VertexAttributeSemantic;

#[derive(Clone, Debug, PartialEq)]
struct MockModel {
    id: &'static str,
}

fn create_mock_runtime_node() -> ModelRuntimeNode {
    let raw_node = Node {
        name: Some("test_node".to_string()),
        ..Default::default()
    };
    let transform_to_root = Matrix4::from_translation(Cartesian3::new(1.0, 2.0, 3.0));

    ModelRuntimeNode::try_new(ModelRuntimeNodeArgs {
        node: Some(raw_node),
        transform: Some(Matrix4::IDENTITY),
        transform_to_root: Some(transform_to_root),
        children: Some(Vec::new()),
        scene_graph: Some(()),
    })
    .expect("Valid node options")
}

fn create_test_primitive_with_indices() -> (Primitive, ModelRuntimePrimitive) {
    let position_attr = Attribute {
        semantic: Some(VertexAttributeSemantic::POSITION.to_string()),
        attribute_type: Some(AttributeType::Vec3),
        component_datatype: Some(ComponentDatatype::Float),
        min: Some(vec![-1.0, -1.0, -1.0]),
        max: Some(vec![1.0, 1.0, 1.0]),
        count: 6,
        ..Default::default()
    };

    let primitive = Primitive {
        indices: Some(Indices {
            count: 6,
            ..Default::default()
        }),
        primitive_type: Some(PrimitiveType::Triangles),
        attributes: vec![position_attr],
        ..Default::default()
    };

    let runtime_primitive = ModelRuntimePrimitive::try_new(ModelRuntimePrimitiveArgs {
        primitive: Some(primitive.clone()),
        node: Some(Node::default()),
        model: Some(ModelRuntimePrimitiveModel::default()),
    })
    .expect("Valid runtime primitive");

    (primitive, runtime_primitive)
}

fn create_test_primitive_without_indices() -> (Primitive, ModelRuntimePrimitive) {
    let position_attr = Attribute {
        semantic: Some(VertexAttributeSemantic::POSITION.to_string()),
        attribute_type: Some(AttributeType::Vec3),
        component_datatype: Some(ComponentDatatype::Float),
        count: 8,
        min: Some(vec![-2.0, -2.0, -2.0]),
        max: Some(vec![2.0, 2.0, 2.0]),
        ..Default::default()
    };

    let primitive = Primitive {
        indices: None,
        primitive_type: Some(PrimitiveType::Points),
        attributes: vec![position_attr],
        ..Default::default()
    };

    let runtime_primitive = ModelRuntimePrimitive::try_new(ModelRuntimePrimitiveArgs {
        primitive: Some(primitive.clone()),
        node: Some(Node::default()),
        model: Some(ModelRuntimePrimitiveModel::default()),
    })
    .expect("Valid runtime primitive without indices");

    (primitive, runtime_primitive)
}

#[test]
fn test_throws_for_undefined_node_render_resources() {
    let (_, runtime_prim) = create_test_primitive_with_indices();
    let result = PrimitiveRenderResources::<MockModel>::try_new(None, Some(&runtime_prim));
    assert!(result.is_err());
    assert_eq!(
        result.err().unwrap().message,
        "nodeRenderResources is required."
    );
}

#[test]
fn test_throws_for_undefined_runtime_primitive() {
    let model = MockModel { id: "test_model" };
    let model_resources = ModelRenderResources::new(model);
    let runtime_node = create_mock_runtime_node();
    let node_resources = NodeRenderResources::new(&model_resources, &runtime_node);

    let result = PrimitiveRenderResources::try_new(Some(&node_resources), None);
    assert!(result.is_err());
    assert_eq!(
        result.err().unwrap().message,
        "runtimePrimitive is required."
    );
}

#[test]
fn test_constructs() {
    let model = MockModel { id: "test_model" };
    let model_resources = ModelRenderResources::new(model);
    let runtime_node = create_mock_runtime_node();
    let node_resources = NodeRenderResources::new(&model_resources, &runtime_node);

    let (primitive, runtime_primitive) = create_test_primitive_with_indices();
    let primitive_resources =
        PrimitiveRenderResources::new(&node_resources, &runtime_primitive);

    assert_eq!(primitive_resources.runtime_primitive, &runtime_primitive);
    assert!(primitive_resources.pick_id.is_none());
    assert_eq!(primitive_resources.count, 6);
    assert_eq!(primitive_resources.indices, primitive.indices);
    assert_eq!(
        primitive_resources.primitive_type,
        Some(PrimitiveType::Triangles)
    );
    assert_eq!(
        primitive_resources.position_min,
        Cartesian3::new(-1.0, -1.0, -1.0)
    );
    assert_eq!(
        primitive_resources.position_max,
        Cartesian3::new(1.0, 1.0, 1.0)
    );

    // Diagonal is 2 * sqrt(3), half is sqrt(3), center is (0,0,0)
    assert!(primitive_resources
        .bounding_sphere
        .center
        .abs_diff_eq(Cartesian3::ZERO.into(), EPSILON9));
    assert!(
        (primitive_resources.bounding_sphere.radius - 3.0_f64.sqrt()).abs()
            < EPSILON9
    );

    assert!(primitive_resources.uniform_map.is_empty());
    assert_eq!(
        primitive_resources.lighting_options.lighting_model,
        LightingModel::Unlit
    );

    let default_render_state = RenderState {
        depth_test: DepthTestState {
            enabled: true,
            func: DepthFunction::LEqual,
        },
        ..Default::default()
    };
    assert_eq!(
        primitive_resources.render_state_options,
        default_render_state
    );
    assert!(!primitive_resources.has_silhouette);
    assert!(!primitive_resources.has_skip_level_of_detail);
}

#[test]
fn test_constructs_from_primitive_without_indices() {
    let model = MockModel { id: "test_model" };
    let model_resources = ModelRenderResources::new(model);
    let runtime_node = create_mock_runtime_node();
    let node_resources = NodeRenderResources::new(&model_resources, &runtime_node);

    let (_, runtime_primitive_no_idx) = create_test_primitive_without_indices();
    let primitive_resources =
        PrimitiveRenderResources::new(&node_resources, &runtime_primitive_no_idx);

    assert_eq!(primitive_resources.count, 8);
    assert!(primitive_resources.indices.is_none());
    assert_eq!(
        primitive_resources.primitive_type,
        Some(PrimitiveType::Points)
    );
    assert_eq!(
        primitive_resources.position_min,
        Cartesian3::new(-2.0, -2.0, -2.0)
    );
    assert_eq!(
        primitive_resources.position_max,
        Cartesian3::new(2.0, 2.0, 2.0)
    );

    // Diagonal is 4 * sqrt(3), half is 2 * sqrt(3)
    assert!(primitive_resources
        .bounding_sphere
        .center
        .abs_diff_eq(Cartesian3::ZERO.into(), EPSILON9));
    assert!(
        (primitive_resources.bounding_sphere.radius - 2.0 * 3.0_f64.sqrt()).abs()
            < EPSILON9
    );

    assert!(primitive_resources.uniform_map.is_empty());
    assert_eq!(
        primitive_resources.lighting_options.lighting_model,
        LightingModel::Unlit
    );

    let default_render_state = RenderState {
        depth_test: DepthTestState {
            enabled: true,
            func: DepthFunction::LEqual,
        },
        ..Default::default()
    };
    assert_eq!(
        primitive_resources.render_state_options,
        default_render_state
    );
}

#[test]
fn test_inherits_from_model_render_resources() {
    let model = MockModel { id: "test_model" };
    let mut model_resources = ModelRenderResources::new(model.clone());
    model_resources.shader_builder.add_define("MODEL", None, None);
    model_resources.render_state_options.cull.enabled = true;
    model_resources.render_state_options.cull.face = CullFace::Back;
    model_resources.has_silhouette = true;
    model_resources.has_skip_level_of_detail = true;

    let runtime_node = create_mock_runtime_node();
    let mut node_resources = NodeRenderResources::new(&model_resources, &runtime_node);
    node_resources.shader_builder.add_define("NODE", None, None);

    let (_, runtime_primitive) = create_test_primitive_with_indices();
    let mut primitive_resources =
        PrimitiveRenderResources::new(&node_resources, &runtime_primitive);
    primitive_resources
        .shader_builder
        .add_define("PRIMITIVE", None, None);

    assert_eq!(primitive_resources.model, &model);
    assert!(primitive_resources.render_state_options.cull.enabled);
    assert_eq!(
        primitive_resources.render_state_options.cull.face,
        CullFace::Back
    );
    assert!(primitive_resources.has_silhouette);
    assert!(primitive_resources.has_skip_level_of_detail);

    // The defines should cascade through the three levels
    assert_eq!(
        model_resources.shader_builder.fragment_define_lines(),
        &["MODEL".to_string()]
    );
    assert_eq!(
        node_resources.shader_builder.fragment_define_lines(),
        &["MODEL".to_string(), "NODE".to_string()]
    );
    assert_eq!(
        primitive_resources.shader_builder.fragment_define_lines(),
        &[
            "MODEL".to_string(),
            "NODE".to_string(),
            "PRIMITIVE".to_string()
        ]
    );
}

#[test]
fn test_inherits_from_node_render_resources() {
    let model = MockModel { id: "test_model" };
    let mut model_resources = ModelRenderResources::new(model);
    model_resources.shader_builder.add_define("MODEL", None, None);
    model_resources.render_state_options.cull.enabled = true;

    let runtime_node = create_mock_runtime_node();
    let mut node_resources = NodeRenderResources::new(&model_resources, &runtime_node);
    node_resources.shader_builder.add_define("NODE", None, None);
    node_resources.render_state_options.blending = BlendingState::ALPHA_BLEND;

    let (_, runtime_primitive) = create_test_primitive_with_indices();
    let primitive_resources =
        PrimitiveRenderResources::new(&node_resources, &runtime_primitive);

    assert_eq!(primitive_resources.runtime_node, &runtime_node);
    assert!(primitive_resources.attributes.is_empty());
    assert!(primitive_resources.render_state_options.cull.enabled);
    assert_eq!(
        primitive_resources.render_state_options.blending,
        BlendingState::ALPHA_BLEND
    );
}
