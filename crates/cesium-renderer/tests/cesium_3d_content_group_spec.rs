use cesium_renderer::bounding_volume_semantics::MetadataClass;
use cesium_renderer::{Cesium3DContentGroup, GroupMetadata, GroupMetadataOptions};
use serde_json::json;

fn mock_group_metadata() -> GroupMetadata {
    let class_def = MetadataClass::from_value(&json!({ "properties": {} }));
    GroupMetadata::new(GroupMetadataOptions {
        id: Some("test_group".to_string()),
        group: Some(json!({})),
        class_def: Some(class_def),
    })
    .unwrap()
}

#[test]
fn constructs() {
    let metadata = mock_group_metadata();
    let group = Cesium3DContentGroup::try_new(Some(metadata.clone())).unwrap();
    assert_eq!(group.metadata.id(), metadata.id());
}

#[test]
fn throws_without_metadata() {
    assert!(Cesium3DContentGroup::try_new(None).is_err());
}
