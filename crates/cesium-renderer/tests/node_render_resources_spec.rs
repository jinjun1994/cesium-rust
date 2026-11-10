use cesium_math::{Cartesian3, Matrix4};
use cesium_renderer::model_components::Node;
use cesium_renderer::model_render_resources::ModelRenderResources;
use cesium_renderer::model_runtime_node::{ModelRuntimeNode, ModelRuntimeNodeArgs};
use cesium_renderer::node_render_resources::NodeRenderResources;
use cesium_renderer::render_state::{CullFace, DepthFunction, DepthTestState, RenderState};

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

#[test]
fn test_throws_for_undefined_model_render_resources() {
    let runtime_node = create_mock_runtime_node();
    let result = NodeRenderResources::<MockModel>::try_new(None, Some(&runtime_node));
    assert!(result.is_err());
    assert_eq!(
        result.err().unwrap().message,
        "modelRenderResources is required."
    );
}

#[test]
fn test_throws_for_undefined_runtime_node() {
    let model = MockModel { id: "test_model" };
    let model_resources = ModelRenderResources::new(model);
    let result = NodeRenderResources::try_new(Some(&model_resources), None);
    assert!(result.is_err());
    assert_eq!(
        result.err().unwrap().message,
        "runtimeNode is required."
    );
}

#[test]
fn test_constructs() {
    let model = MockModel { id: "test_model" };
    let model_resources = ModelRenderResources::new(model.clone());
    let runtime_node = create_mock_runtime_node();

    let node_resources = NodeRenderResources::new(&model_resources, &runtime_node);

    let default_render_state = RenderState {
        depth_test: DepthTestState {
            enabled: true,
            func: DepthFunction::LEqual,
        },
        ..Default::default()
    };

    assert_eq!(node_resources.model, &model);
    assert_eq!(
        node_resources
            .runtime_node
            .node
            .as_ref()
            .and_then(|n| n.name.as_deref()),
        Some("test_node")
    );
    assert!(node_resources.attributes.is_empty());
    assert_eq!(node_resources.render_state_options, default_render_state);
    assert!(!node_resources.has_silhouette);
    assert!(!node_resources.has_skip_level_of_detail);
    assert_eq!(node_resources.attribute_index, 1);
    assert_eq!(node_resources.feature_id_vertex_attribute_set_index, 0);
    assert_eq!(node_resources.instance_count, 0);
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

    assert_eq!(node_resources.model, &model);
    assert!(node_resources.render_state_options.cull.enabled);
    assert_eq!(node_resources.render_state_options.cull.face, CullFace::Back);
    assert!(node_resources.has_silhouette);
    assert!(node_resources.has_skip_level_of_detail);

    // The model shader must not be modified by the node
    assert_eq!(
        model_resources.shader_builder.fragment_define_lines(),
        &["MODEL".to_string()]
    );

    // But the node shader will have both
    assert_eq!(
        node_resources.shader_builder.fragment_define_lines(),
        &["MODEL".to_string(), "NODE".to_string()]
    );
}
