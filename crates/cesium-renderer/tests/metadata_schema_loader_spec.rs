use cesium_renderer::{
    MetadataSchemaLoader, MetadataSchemaLoaderOptions, ResourceLoaderState,
};
use serde_json::json;

fn sample_schema_json() -> serde_json::Value {
    json!({
        "classes": {
            "tree": {
                "properties": {
                    "height": {
                        "type": "SCALAR",
                        "componentType": "FLOAT32"
                    },
                    "type": {
                        "type": "ENUM",
                        "enumType": "treeType"
                    }
                }
            }
        },
        "enums": {
            "treeType": {
                "values": [
                    { "name": "Coniferous", "value": 0 },
                    { "name": "Deciduous", "value": 1 }
                ]
            }
        }
    })
}

#[test]
fn test_throws_if_neither_schema_nor_resource() {
    let opts = MetadataSchemaLoaderOptions {
        schema: None,
        resource: None,
        cache_key: None,
    };
    assert!(MetadataSchemaLoader::new(opts).is_err());
}

#[test]
fn test_throws_if_both_schema_and_resource() {
    let opts = MetadataSchemaLoaderOptions {
        schema: Some(sample_schema_json()),
        resource: Some("https://example.com/schema.json".to_string()),
        cache_key: None,
    };
    assert!(MetadataSchemaLoader::new(opts).is_err());
}

#[test]
fn test_loads_schema_from_json() {
    pollster::block_on(async {
        let opts = MetadataSchemaLoaderOptions {
            schema: Some(sample_schema_json()),
            resource: None,
            cache_key: None,
        };
        let loader = MetadataSchemaLoader::new(opts).unwrap();
        let result = loader
            .load::<fn(String) -> std::future::Ready<Result<serde_json::Value, String>>, _>(None)
            .await;
        assert!(result.is_ok());

        let schema = loader.schema().expect("Schema should be defined");
        assert!(schema.enums.contains_key("treeType"));
        assert!(schema.classes.contains_key("tree"));
        assert_eq!(loader.state(), ResourceLoaderState::Ready);
    });
}

#[test]
fn test_loads_external_schema() {
    pollster::block_on(async {
        let opts = MetadataSchemaLoaderOptions {
            schema: None,
            resource: Some("https://example.com/schema.json".to_string()),
            cache_key: Some("schema_cache".to_string()),
        };
        let loader = MetadataSchemaLoader::new(opts).unwrap();
        assert_eq!(loader.cache_key().as_deref(), Some("schema_cache"));
        assert!(loader.schema().is_none());

        let schema_val = sample_schema_json();
        let fetcher = move |_url: String| {
            let val = schema_val.clone();
            async move { Ok(val) }
        };

        let result = loader.load(Some(fetcher)).await;
        assert!(result.is_ok());

        let schema = loader.schema().expect("Schema should be defined");
        assert!(schema.enums.contains_key("treeType"));
        assert!(schema.classes.contains_key("tree"));
        assert_eq!(loader.state(), ResourceLoaderState::Ready);
    });
}

#[test]
fn test_load_throws_if_schema_cannot_be_fetched() {
    pollster::block_on(async {
        let opts = MetadataSchemaLoaderOptions {
            schema: None,
            resource: Some("https://example.com/schema.json".to_string()),
            cache_key: None,
        };
        let loader = MetadataSchemaLoader::new(opts).unwrap();

        let fetcher = |_url: String| async { Err("404 Not Found".to_string()) };

        let result = loader.load(Some(fetcher)).await;
        assert!(result.is_err());
        assert_eq!(loader.state(), ResourceLoaderState::Failed);
    });
}

#[test]
fn test_destroys_schema() {
    pollster::block_on(async {
        let opts = MetadataSchemaLoaderOptions {
            schema: None,
            resource: Some("https://example.com/schema.json".to_string()),
            cache_key: None,
        };
        let loader = MetadataSchemaLoader::new(opts).unwrap();
        assert!(loader.schema().is_none());

        let schema_val = sample_schema_json();
        let fetcher = move |_url: String| {
            let val = schema_val.clone();
            async move { Ok(val) }
        };

        let result = loader.load(Some(fetcher)).await;
        assert!(result.is_ok());
        assert!(loader.schema().is_some());
        assert!(!loader.is_destroyed());

        loader.destroy();
        assert!(loader.schema().is_none());
        assert!(loader.is_destroyed());
    });
}

#[test]
fn test_handles_resolving_and_rejecting_after_destroy() {
    pollster::block_on(async {
        // Resolve after destroy
        let opts = MetadataSchemaLoaderOptions {
            schema: None,
            resource: Some("https://example.com/schema.json".to_string()),
            cache_key: None,
        };
        let loader = MetadataSchemaLoader::new(opts).unwrap();
        loader.destroy();

        let schema_val = sample_schema_json();
        let fetcher = move |_url: String| {
            let val = schema_val.clone();
            async move { Ok(val) }
        };
        let result = loader.load(Some(fetcher)).await;
        assert!(result.is_ok());
        assert!(loader.schema().is_none());
        assert!(loader.is_destroyed());

        // Reject after destroy
        let opts2 = MetadataSchemaLoaderOptions {
            schema: None,
            resource: Some("https://example.com/schema.json".to_string()),
            cache_key: None,
        };
        let loader2 = MetadataSchemaLoader::new(opts2).unwrap();
        loader2.destroy();

        let fetcher2 = |_url: String| async { Err("404 Not Found".to_string()) };
        let result2 = loader2.load(Some(fetcher2)).await;
        assert!(result2.is_ok());
        assert!(loader2.schema().is_none());
        assert!(loader2.is_destroyed());
    });
}
