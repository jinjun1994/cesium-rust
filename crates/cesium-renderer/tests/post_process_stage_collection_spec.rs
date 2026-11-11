use cesium_renderer::post_process_stage::{PostProcessStage, PostProcessStageOptions};
use cesium_renderer::post_process_stage_collection::{PostProcessStageCollection, Tonemapper};
use cesium_renderer::post_process_stage_composite::PostProcessStageNode;

#[test]
fn test_post_process_stage_collection_constructs() {
    let stages = PostProcessStageCollection::new();
    assert!(!stages.ready());
    assert!(!stages.fxaa.name.is_empty());
    assert!(!stages.ambient_occlusion.name.is_empty());
    assert!(!stages.bloom.name.is_empty());
    assert_eq!(stages.len(), 0);
    assert!(stages.output_texture.is_none());
}

#[test]
fn test_post_process_stage_collection_adds_stages() {
    let mut stages = PostProcessStageCollection::new();
    let stage1 = PostProcessStage::new(PostProcessStageOptions::new(
        "void main() { out_FragColor = vec4(1.0, 1.0, 0.0, 1.0); }",
    ))
    .unwrap();
    stages.add(PostProcessStageNode::Stage(stage1)).unwrap();
    assert_eq!(stages.len(), 1);

    let stage2 = PostProcessStage::new(PostProcessStageOptions::new(
        "uniform sampler2D colorTexture; in vec2 v_textureCoordinates; void main() { out_FragColor = texture(colorTexture, v_textureCoordinates); }",
    ))
    .unwrap();
    stages.add(PostProcessStageNode::Stage(stage2)).unwrap();
    assert_eq!(stages.len(), 2);
}

#[test]
fn test_post_process_stage_collection_throws_when_adding_the_same_stage() {
    let mut stages = PostProcessStageCollection::new();
    let stage = PostProcessStage::new(PostProcessStageOptions::new(
        "void main() { out_FragColor = vec4(1.0); }",
    ))
    .unwrap();
    let node = PostProcessStageNode::Stage(stage);
    stages.add(node.clone()).unwrap();

    let err = stages.add(node);
    assert!(err.is_err());
}

#[test]
fn test_post_process_stage_collection_removes_a_single_stage() {
    let mut stages = PostProcessStageCollection::new();
    let stage1 = PostProcessStage::new(PostProcessStageOptions::new(
        "void main() { out_FragColor = vec4(1.0); }",
    ))
    .unwrap();
    let name = stage1.name.clone();
    stages.add(PostProcessStageNode::Stage(stage1)).unwrap();
    assert_eq!(stages.len(), 1);

    assert!(stages.remove(&name));
    assert_eq!(stages.len(), 0);
}

#[test]
fn test_post_process_stage_collection_removes_stages() {
    let mut stages = PostProcessStageCollection::new();
    let stage1 = PostProcessStage::new(PostProcessStageOptions::new(
        "void main() { out_FragColor = vec4(1.0); }",
    ))
    .unwrap();
    let stage2 = PostProcessStage::new(PostProcessStageOptions::new(
        "void main() { out_FragColor = vec4(0.0); }",
    ))
    .unwrap();
    let name1 = stage1.name.clone();
    let name2 = stage2.name.clone();

    stages.add(PostProcessStageNode::Stage(stage1)).unwrap();
    stages.add(PostProcessStageNode::Stage(stage2)).unwrap();
    assert_eq!(stages.len(), 2);

    assert!(stages.remove(&name1));
    assert_eq!(stages.len(), 1);
    assert!(!stages.contains(&name1));
    assert!(stages.contains(&name2));

    assert!(stages.remove(&name2));
    assert_eq!(stages.len(), 0);
    assert!(!stages.contains(&name2));

    assert!(!stages.remove(&name1));
}

#[test]
fn test_post_process_stage_collection_gets_stages_at_index() {
    let mut stages = PostProcessStageCollection::new();
    let stage1 = PostProcessStage::new(PostProcessStageOptions::new(
        "void main() { out_FragColor = vec4(1.0); }",
    ))
    .unwrap();
    let stage2 = PostProcessStage::new(PostProcessStageOptions::new(
        "void main() { out_FragColor = vec4(0.0); }",
    ))
    .unwrap();
    let name1 = stage1.name.clone();
    let name2 = stage2.name.clone();

    stages.add(PostProcessStageNode::Stage(stage1)).unwrap();
    stages.add(PostProcessStageNode::Stage(stage2)).unwrap();
    assert_eq!(stages.len(), 2);

    assert_eq!(stages.get(0).unwrap().name(), &name1);
    assert_eq!(stages.get(1).unwrap().name(), &name2);

    assert!(stages.remove(&name1));
    assert_eq!(stages.get(0).unwrap().name(), &name2);
}

#[test]
fn test_post_process_stage_collection_throws_when_get_index_is_invalid() {
    let mut stages = PostProcessStageCollection::new();
    assert!(stages.get(0).is_err());

    let stage = PostProcessStage::new(PostProcessStageOptions::new(
        "void main() { out_FragColor = vec4(1.0); }",
    ))
    .unwrap();
    stages.add(PostProcessStageNode::Stage(stage)).unwrap();

    assert!(stages.get(1).is_err());
}

#[test]
fn test_post_process_stage_collection_removes_all() {
    let mut stages = PostProcessStageCollection::new();
    let stage1 = PostProcessStage::new(PostProcessStageOptions::new(
        "void main() { out_FragColor = vec4(1.0); }",
    ))
    .unwrap();
    let stage2 = PostProcessStage::new(PostProcessStageOptions::new(
        "void main() { out_FragColor = vec4(0.0); }",
    ))
    .unwrap();
    let name1 = stage1.name.clone();
    let name2 = stage2.name.clone();

    stages.add(PostProcessStageNode::Stage(stage1)).unwrap();
    stages.add(PostProcessStageNode::Stage(stage2)).unwrap();
    assert_eq!(stages.len(), 2);

    stages.remove_all();
    assert_eq!(stages.len(), 0);
    assert!(!stages.contains(&name1));
    assert!(!stages.contains(&name2));
}

#[test]
fn test_post_process_stage_collection_gets_by_stage_name() {
    let mut stages = PostProcessStageCollection::new();
    let stage1 = PostProcessStage::new(PostProcessStageOptions::new(
        "void main() { out_FragColor = vec4(1.0); }",
    ))
    .unwrap();
    let name1 = stage1.name.clone();

    stages.add(PostProcessStageNode::Stage(stage1)).unwrap();
    assert!(stages.get_stage_by_name(&name1).is_some());
    assert!(stages.get_stage_by_name("invalid").is_none());
}

#[test]
fn test_post_process_stage_collection_gets_output_texture_by_stage_name() {
    let mut stages = PostProcessStageCollection::new();
    let stage1 = PostProcessStage::new(PostProcessStageOptions::new(
        "void main() { out_FragColor = vec4(1.0); }",
    ))
    .unwrap();
    let stage2 = PostProcessStage::new(PostProcessStageOptions::new(
        "void main() { out_FragColor = vec4(0.0); }",
    ))
    .unwrap();
    let name1 = stage1.name.clone();
    let name2 = stage2.name.clone();

    stages.add(PostProcessStageNode::Stage(stage1)).unwrap();
    stages.add(PostProcessStageNode::Stage(stage2)).unwrap();

    stages.fxaa.enabled = true;

    assert!(stages.get_output_texture(&name1).is_some());
    assert!(stages.get_output_texture(&name2).is_some());
    assert!(stages.get_output_texture(&stages.fxaa.name).is_some());

    stages.remove(&name1);
    assert!(stages.get_output_texture(&name1).is_none());
}

#[test]
fn test_post_process_stage_collection_tonemapper_validation() {
    let mut stages = PostProcessStageCollection::new();
    assert_eq!(stages.tonemapper, Tonemapper::PbrNeutral);

    stages.set_tonemapper(Tonemapper::Aces);
    assert_eq!(stages.tonemapper, Tonemapper::Aces);
    assert_eq!(stages.tonemapper.as_str(), "ACES");

    assert!(Tonemapper::validate("REINHARD"));
    assert!(Tonemapper::validate("MODIFIED_REINHARD"));
    assert!(Tonemapper::validate("FILMIC"));
    assert!(Tonemapper::validate("ACES"));
    assert!(Tonemapper::validate("PBR_NEUTRAL"));
    assert!(!Tonemapper::validate("UNKNOWN"));
}
