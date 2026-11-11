use std::collections::HashMap;

use cesium_math::Color;
use cesium_renderer::post_process_stage::{
    PostProcessStage, PostProcessStageOptions, PostProcessUniformValue,
};
use cesium_renderer::post_process_stage_composite::{
    PostProcessStageComposite, PostProcessStageCompositeOptions, PostProcessStageNode,
};

#[test]
fn test_post_process_stage_composite_constructs() {
    let mut uniforms = HashMap::new();
    uniforms.insert("color".to_string(), PostProcessUniformValue::Color(Color::RED));

    let stage = PostProcessStage::new(PostProcessStageOptions::new(
        "uniform vec4 color; void main() { out_FragColor = color; }",
    ))
    .unwrap();

    let mut comp_options =
        PostProcessStageCompositeOptions::new(vec![PostProcessStageNode::Stage(stage)]);
    comp_options.uniforms = Some(uniforms.clone());
    comp_options.input_previous_stage_texture = Some(false);
    comp_options.name = Some("kaleidoscope".to_string());

    let composite = PostProcessStageComposite::new(comp_options).unwrap();
    assert!(!composite.ready);
    assert_eq!(composite.name, "kaleidoscope");
    assert!(composite.enabled);
    assert_eq!(composite.uniforms, Some(uniforms));
    assert!(!composite.input_previous_stage_texture);
    assert_eq!(composite.len(), 1);
}

#[test]
fn test_post_process_stage_composite_default_constructs() {
    let stage1 = PostProcessStage::new(PostProcessStageOptions::new(
        "void main() { out_FragColor = vec4(1.0); }",
    ))
    .unwrap();
    let stage2 = PostProcessStage::new(PostProcessStageOptions::new(
        "void main() { out_FragColor = vec4(1.0, 0.0, 0.0, 1.0); }",
    ))
    .unwrap();

    let composite = PostProcessStageComposite::new(PostProcessStageCompositeOptions::new(vec![
        PostProcessStageNode::Stage(stage1),
        PostProcessStageNode::Stage(stage2),
    ]))
    .unwrap();

    assert!(!composite.ready);
    assert!(!composite.name.is_empty());
    assert!(composite.enabled);
    assert!(composite.uniforms.is_none());
    assert!(composite.input_previous_stage_texture);
    assert_eq!(composite.len(), 2);
}

#[test]
fn test_post_process_stage_composite_throws_without_stages() {
    let options = PostProcessStageCompositeOptions::new(Vec::new());
    assert!(PostProcessStageComposite::new(options).is_err());
}

#[test]
fn test_post_process_stage_composite_gets_stages() {
    let stage1 = PostProcessStage::new(PostProcessStageOptions::new(
        "void main() { out_FragColor = vec4(1.0); }",
    ))
    .unwrap();
    let stage2 = PostProcessStage::new(PostProcessStageOptions::new(
        "void main() { out_FragColor = vec4(1.0, 0.0, 0.0, 1.0); }",
    ))
    .unwrap();
    let name1 = stage1.name.clone();
    let name2 = stage2.name.clone();

    let composite = PostProcessStageComposite::new(PostProcessStageCompositeOptions::new(vec![
        PostProcessStageNode::Stage(stage1),
        PostProcessStageNode::Stage(stage2),
    ]))
    .unwrap();

    assert_eq!(composite.get(0).unwrap().name(), &name1);
    assert_eq!(composite.get(1).unwrap().name(), &name2);
}

#[test]
fn test_post_process_stage_composite_throws_when_get_index_is_invalid() {
    let stage1 = PostProcessStage::new(PostProcessStageOptions::new(
        "void main() { out_FragColor = vec4(1.0); }",
    ))
    .unwrap();
    let stage2 = PostProcessStage::new(PostProcessStageOptions::new(
        "void main() { out_FragColor = vec4(1.0, 0.0, 0.0, 1.0); }",
    ))
    .unwrap();

    let composite = PostProcessStageComposite::new(PostProcessStageCompositeOptions::new(vec![
        PostProcessStageNode::Stage(stage1),
        PostProcessStageNode::Stage(stage2),
    ]))
    .unwrap();

    assert!(composite.get(2).is_err());
    assert!(composite.get(10).is_err());
}

#[test]
fn test_post_process_stage_composite_destroys() {
    let stage1 = PostProcessStage::new(PostProcessStageOptions::new(
        "void main() { out_FragColor = vec4(1.0); }",
    ))
    .unwrap();
    let stage2 = PostProcessStage::new(PostProcessStageOptions::new(
        "void main() { out_FragColor = vec4(1.0, 0.0, 0.0, 1.0); }",
    ))
    .unwrap();

    let mut composite = PostProcessStageComposite::new(PostProcessStageCompositeOptions::new(vec![
        PostProcessStageNode::Stage(stage1),
        PostProcessStageNode::Stage(stage2),
    ]))
    .unwrap();

    assert!(!composite.is_destroyed());
    assert!(!composite.get(0).unwrap().is_destroyed());
    assert!(!composite.get(1).unwrap().is_destroyed());

    composite.destroy();

    assert!(composite.is_destroyed());
    assert!(composite.get(0).unwrap().is_destroyed());
    assert!(composite.get(1).unwrap().is_destroyed());
}
