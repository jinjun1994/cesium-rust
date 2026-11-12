use cesium_renderer::{
    Cesium3DTilesetMetadata, Cesium3DTilesetMetadataOptions, MetadataSchema,
};
use serde_json::{json, Value};

fn schema_json() -> Value {
    json!({
        "classes": {
            "city": {
                "properties": {
                    "name": { "type": "STRING" }
                }
            },
            "neighborhood": {
                "properties": {
                    "color": { "type": "STRING" }
                }
            },
            "tree": {
                "properties": {
                    "species": { "type": "STRING" }
                }
            }
        }
    })
}

#[test]
fn creates_3d_tiles_metadata_with_default_values() {
    let schema = MetadataSchema::from_json(Some(&schema_json())).unwrap();
    let metadata = Cesium3DTilesetMetadata::new(Cesium3DTilesetMetadataOptions {
        metadata_json: Some(json!({})),
        schema: Some(schema.clone()),
    })
    .unwrap();

    assert_eq!(metadata.schema, schema);
    assert!(metadata.groups.is_empty());
    assert!(metadata.group_ids.is_empty());
    assert!(metadata.tileset.is_none());
    assert!(metadata.statistics.is_none());
    assert!(metadata.extras.is_none());
    assert!(metadata.extensions.is_none());
}

#[test]
fn creates_3d_tiles_metadata() {
    let statistics = json!({
        "classes": {
            "tree": {
                "count": 100,
                "properties": {
                    "height": { "min": 10.0, "max": 20.0 }
                }
            }
        }
    });
    let extras = json!({ "description": "Extra" });
    let extensions = json!({ "EXT_other_extension": {} });

    let tileset_json = json!({
        "schema": schema_json(),
        "groups": [
            { "class": "neighborhood", "properties": { "color": "RED" } },
            { "class": "neighborhood", "properties": { "color": "GREEN" } }
        ],
        "metadata": {
            "class": "city",
            "properties": { "name": "City" }
        },
        "statistics": statistics,
        "extras": extras,
        "extensions": extensions
    });

    let schema = MetadataSchema::from_json(Some(&schema_json())).unwrap();
    let metadata = Cesium3DTilesetMetadata::new(Cesium3DTilesetMetadataOptions {
        metadata_json: Some(tileset_json),
        schema: Some(schema),
    })
    .unwrap();

    let city_class = metadata.schema.classes.get("city").unwrap();
    let neighborhood_class = metadata.schema.classes.get("neighborhood").unwrap();
    let tree_class = metadata.schema.classes.get("tree").unwrap();
    assert_eq!(city_class.id, "city");
    assert_eq!(neighborhood_class.id, "neighborhood");
    assert_eq!(tree_class.id, "tree");

    let tileset_meta = metadata.tileset.as_ref().unwrap();
    assert_eq!(tileset_meta.class_def.as_ref(), Some(city_class));
    assert_eq!(tileset_meta.get_property("name").unwrap(), json!("City"));

    let n_a = &metadata.groups[0];
    let n_b = &metadata.groups[1];
    assert_eq!(n_a.class_def(), neighborhood_class);
    assert_eq!(n_a.get_property(Some("color")).unwrap(), Some(json!("RED")));
    assert_eq!(n_b.class_def(), neighborhood_class);
    assert_eq!(n_b.get_property(Some("color")).unwrap(), Some(json!("GREEN")));

    assert_eq!(metadata.statistics, Some(statistics));
    assert_eq!(metadata.extras, Some(extras));
    assert_eq!(metadata.extensions, Some(extensions));
}

#[test]
fn creates_3d_tiles_metadata_legacy() {
    let statistics = json!({ "classes": { "tree": { "count": 100 } } });
    let extras = json!({ "description": "Extra" });
    let extensions = json!({ "EXT_other_extension": {} });

    let extension = json!({
        "schema": schema_json(),
        "groups": {
            "neighborhoodB": { "class": "neighborhood", "properties": { "color": "GREEN" } },
            "neighborhoodA": { "class": "neighborhood", "properties": { "color": "RED" } }
        },
        "tileset": {
            "class": "city",
            "properties": { "name": "City" }
        },
        "statistics": statistics,
        "extras": extras,
        "extensions": extensions
    });

    let schema = MetadataSchema::from_json(Some(&schema_json())).unwrap();
    let metadata = Cesium3DTilesetMetadata::new(Cesium3DTilesetMetadataOptions {
        metadata_json: Some(extension),
        schema: Some(schema),
    })
    .unwrap();

    assert_eq!(metadata.group_ids, vec!["neighborhoodA", "neighborhoodB"]);
    assert_eq!(metadata.groups[0].id(), Some("neighborhoodA"));
    assert_eq!(
        metadata.groups[0].get_property(Some("color")).unwrap(),
        Some(json!("RED"))
    );
    assert_eq!(metadata.groups[1].id(), Some("neighborhoodB"));
    assert_eq!(
        metadata.groups[1].get_property(Some("color")).unwrap(),
        Some(json!("GREEN"))
    );
    assert_eq!(
        metadata.tileset.as_ref().unwrap().get_property("name").unwrap(),
        json!("City")
    );
}

#[test]
fn constructor_throws_without_metadata_json_or_schema() {
    let schema = MetadataSchema::from_json(Some(&schema_json())).unwrap();
    assert!(Cesium3DTilesetMetadata::new(Cesium3DTilesetMetadataOptions {
        metadata_json: None,
        schema: Some(schema),
    })
    .is_err());

    assert!(Cesium3DTilesetMetadata::new(Cesium3DTilesetMetadataOptions {
        metadata_json: Some(json!({})),
        schema: None,
    })
    .is_err());
}
