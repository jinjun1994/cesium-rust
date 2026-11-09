use cesium_renderer::keyframe_node::{KeyframeLoadState, KeyframeNode};
use serde_json::json;

#[test]
fn constructs_with_defaults() {
    let dummy_spatial_node = json!({ "id": "dummy" });
    let keyframe = 13;
    let node = KeyframeNode::new(dummy_spatial_node.clone(), keyframe);

    assert_eq!(node.spatial_node, Some(dummy_spatial_node));
    assert_eq!(node.keyframe, keyframe);
    assert_eq!(node.state, KeyframeLoadState::Unloaded);
    assert!(node.content.is_none());
    assert_eq!(node.megatexture_index, -1);
    assert_eq!(node.priority, -f64::MAX);
    assert_eq!(node.high_priority_frame_number, -1);
}

#[test]
fn priority_comparator_compares_priorities() {
    let dummy = json!({});
    let mut node1 = KeyframeNode::new(dummy.clone(), 13);
    let mut node2 = KeyframeNode::new(dummy, 7);

    node1.priority = 1.0;
    node2.priority = 2.0;

    let comparison = KeyframeNode::priority_comparator(&node1, &node2);
    assert_eq!(comparison, -1.0);
}

#[test]
fn search_comparator_compares_keyframe_indices() {
    let dummy = json!({});
    let node1 = KeyframeNode::new(dummy.clone(), 13);
    let node2 = KeyframeNode::new(dummy, 7);

    let comparison = KeyframeNode::search_comparator(&node1, &node2);
    assert_eq!(comparison, 6);
}

#[test]
fn unload_clears_resources_and_resets_state() {
    let dummy = json!({ "name": "spatial" });
    let mut node = KeyframeNode::new(dummy, 42);
    node.state = KeyframeLoadState::Loaded;
    node.content = Some(json!({ "texture": "megatex" }));
    node.megatexture_index = 5;
    node.priority = 100.0;
    node.high_priority_frame_number = 10;

    node.unload();

    assert!(node.spatial_node.is_none());
    assert!(node.content.is_none());
    assert_eq!(node.state, KeyframeLoadState::Unloaded);
    assert_eq!(node.megatexture_index, -1);
    assert_eq!(node.priority, -f64::MAX);
    assert_eq!(node.high_priority_frame_number, -1);
}
