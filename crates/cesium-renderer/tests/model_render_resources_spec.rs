use cesium_renderer::{
    DepthFunction, DepthTestState, ModelRenderResources, RenderState,
};

#[derive(Clone, Debug, PartialEq)]
struct MockModel {
    pub id: u64,
}

#[test]
fn test_throws_for_undefined_model() {
    let result: Result<ModelRenderResources<MockModel>, _> = ModelRenderResources::try_new(None);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.message, "model is required.");
}

#[test]
fn test_constructs() {
    let mock_model = MockModel { id: 42 };
    let model_resources = ModelRenderResources::new(mock_model.clone());

    let default_render_state = RenderState {
        depth_test: DepthTestState {
            enabled: true,
            func: DepthFunction::LEqual,
        },
        ..Default::default()
    };

    assert_eq!(model_resources.model, mock_model);
    assert_eq!(model_resources.render_state_options, default_render_state);
    assert!(!model_resources.has_silhouette);
    assert!(!model_resources.has_skip_level_of_detail);
    assert_eq!(model_resources.shader_builder.fragment_define_lines().len(), 0);
    assert_eq!(model_resources.uniform_map.len(), 0);
    assert_eq!(model_resources.alpha_options.pass, None);
    assert_eq!(model_resources.alpha_options.alpha_cutoff, None);
}
