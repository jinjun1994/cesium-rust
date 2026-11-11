use std::collections::HashMap;
use std::rc::Rc;

use cesium_renderer::metadata_class::MetadataClass;
use cesium_renderer::property_texture::PropertyTexture;
use cesium_renderer::texture::{Texture, TextureOptions};
use serde_json::json;

fn setup_class() -> MetadataClass {
    let class_json = json!({
        "id": "map",
        "class": {
            "properties": {
                "color": {
                    "type": "VEC3",
                    "componentType": "UINT8"
                },
                "intensity": {
                    "type": "SCALAR",
                    "componentType": "UINT8"
                },
                "ortho": {
                    "type": "SCALAR",
                    "componentType": "UINT8",
                    "normalized": true
                }
            }
        }
    });
    MetadataClass::from_json(&class_json).unwrap()
}

#[test]
fn test_creates_property_texture() {
    let class_def = setup_class();
    let texture0 = Rc::new(Texture::new(TextureOptions {
        width: 1,
        height: 1,
        ..Default::default()
    }));
    let texture1 = Rc::new(Texture::new(TextureOptions {
        width: 1,
        height: 1,
        ..Default::default()
    }));

    let mut textures = HashMap::new();
    textures.insert(0, texture0);
    textures.insert(1, texture1);

    let extras = json!({"description": "Extra"});
    let extensions = json!({"EXT_other_extension": {}});

    let property_texture_json = json!({
        "class": "map",
        "extras": extras,
        "extensions": extensions,
        "properties": {
            "color": {
                "channels": [0, 1, 2],
                "index": 0,
                "texCoord": 0
            },
            "intensity": {
                "channels": [3],
                "index": 0,
                "texCoord": 0
            },
            "ortho": {
                "channels": [0],
                "index": 1,
                "texCoord": 0
            }
        }
    });

    let pt = PropertyTexture::new(
        Some(&property_texture_json),
        Some(&class_def),
        Some(&textures),
        Some("textureMap".to_string()),
        Some(json!(1)),
    )
    .unwrap();

    assert_eq!(pt.name.as_deref(), Some("textureMap"));
    assert_eq!(pt.id, Some(json!(1)));
    assert_eq!(pt.class, class_def);
    assert_eq!(pt.extras, Some(extras));
    assert_eq!(pt.extensions, Some(extensions));

    // get_property
    assert!(pt.get_property(Some("color")).unwrap().is_some());
    assert!(pt.get_property(Some("intensity")).unwrap().is_some());
    assert!(pt.get_property(Some("ortho")).unwrap().is_some());

    assert!(pt.get_property(Some("other")).unwrap().is_none());
    assert!(pt.get_property(None).is_err());
}

#[test]
fn test_constructor_throws_without_property_texture() {
    let class_def = setup_class();
    let textures = HashMap::new();
    assert!(PropertyTexture::new(None, Some(&class_def), Some(&textures), None, None).is_err());
}

#[test]
fn test_constructor_throws_without_class() {
    let property_texture_json = json!({});
    let textures = HashMap::new();
    assert!(PropertyTexture::new(Some(&property_texture_json), None, Some(&textures), None, None).is_err());
}

#[test]
fn test_constructor_throws_without_textures() {
    let class_def = setup_class();
    let property_texture_json = json!({});
    assert!(PropertyTexture::new(Some(&property_texture_json), Some(&class_def), None, None, None).is_err());
}
