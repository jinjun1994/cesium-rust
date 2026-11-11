use std::collections::HashMap;
use std::rc::Rc;

use cesium_math::Matrix3;
use cesium_renderer::metadata_class_property::MetadataClassProperty;
use cesium_renderer::property_texture_property::PropertyTextureProperty;
use cesium_renderer::texture::{Texture, TextureOptions};
use serde_json::json;

#[test]
fn test_creates_property_texture_property() {
    let class_prop_json = json!({
        "type": "SCALAR",
        "componentType": "UINT8",
        "array": true,
        "count": 3
    });
    let class_property =
        MetadataClassProperty::from_json("color".to_string(), &class_prop_json, None).unwrap();

    let texture = Rc::new(Texture::new(TextureOptions {
        width: 1,
        height: 1,
        ..Default::default()
    }));

    let extras = json!({"description": "Extra"});
    let extensions = json!({
        "KHR_texture_transform": {
            "offset": [0.5, 0.5],
            "scale": [0.1, 0.2],
            "texCoord": 1
        }
    });

    let property_json = json!({
        "channels": [0, 1, 2],
        "index": 0,
        "texCoord": 0,
        "extensions": extensions,
        "extras": extras
    });

    let mut textures = HashMap::new();
    textures.insert(0, texture);

    let ptp = PropertyTextureProperty::new(
        Some(&property_json),
        Some(&class_property),
        Some(&textures),
    )
    .unwrap();

    assert_eq!(ptp.extras, Some(extras));
    assert_eq!(ptp.extensions, Some(extensions));
    assert_eq!(ptp.class_property, class_property);

    let reader = &ptp.texture_reader;
    assert_eq!(reader.tex_coord, 1);
    assert_eq!(reader.channels.as_deref(), Some("rgb"));

    let expected_transform = Matrix3::new(
        0.1, 0.0, 0.0,
        0.0, 0.2, 0.0,
        0.5, 0.5, 1.0,
    );
    assert_eq!(reader.transform, Some(expected_transform));
}

#[test]
fn test_creates_property_texture_property_with_default_channels() {
    let class_prop_json = json!({
        "type": "SCALAR",
        "componentType": "UINT8",
        "normalized": true
    });
    let class_property =
        MetadataClassProperty::from_json("propertyId".to_string(), &class_prop_json, None).unwrap();

    let texture = Rc::new(Texture::new(TextureOptions {
        width: 1,
        height: 1,
        ..Default::default()
    }));

    let property_json = json!({
        "index": 0,
        "texCoord": 0
    });

    let mut textures = HashMap::new();
    textures.insert(0, texture);

    let ptp = PropertyTextureProperty::new(
        Some(&property_json),
        Some(&class_property),
        Some(&textures),
    )
    .unwrap();

    assert_eq!(ptp.texture_reader.channels.as_deref(), Some("r"));
}

#[test]
fn test_constructor_throws_without_property() {
    let class_prop_json = json!({
        "type": "SCALAR",
        "componentType": "UINT8"
    });
    let class_property =
        MetadataClassProperty::from_json("color".to_string(), &class_prop_json, None).unwrap();

    let mut textures = HashMap::new();
    textures.insert(0, Rc::new(Texture::new(TextureOptions::default())));

    assert!(PropertyTextureProperty::new(None, Some(&class_property), Some(&textures)).is_err());
}

#[test]
fn test_constructor_throws_without_class_property() {
    let property_json = json!({});
    let mut textures = HashMap::new();
    textures.insert(0, Rc::new(Texture::new(TextureOptions::default())));

    assert!(PropertyTextureProperty::new(Some(&property_json), None, Some(&textures)).is_err());
}

#[test]
fn test_constructor_throws_without_textures() {
    let class_prop_json = json!({
        "type": "SCALAR",
        "componentType": "UINT8"
    });
    let class_property =
        MetadataClassProperty::from_json("color".to_string(), &class_prop_json, None).unwrap();
    let property_json = json!({});

    assert!(PropertyTextureProperty::new(Some(&property_json), Some(&class_property), None).is_err());
}
