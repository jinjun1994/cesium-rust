//! ResourceCacheSpec matching CesiumJS `Scene/ResourceCacheSpec.js` (Apache-2.0).

use cesium_renderer::{
    BaseResourceLoader, ResourceCache, ResourceCacheKey, ResourceLoader, SchemaCacheKeyOptions,
    VertexBufferCacheKeyOptions,
};
use serde_json::json;
use std::sync::{Arc, Mutex};

#[test]
fn test_adds_resource_and_throws_on_duplicate() {
    let mut cache = ResourceCache::new();
    let schema_uri = "https://example.com/schema.json";
    let cache_key = ResourceCacheKey::get_schema_cache_key(SchemaCacheKeyOptions {
        schema: None,
        resource: Some(schema_uri),
    })
    .unwrap();

    let loader: Arc<Mutex<dyn ResourceLoader>> =
        Arc::new(Mutex::new(BaseResourceLoader::with_cache_key(&cache_key)));

    assert!(cache.add(Arc::clone(&loader)).is_ok());

    let entry = cache.cache_entries().get(&cache_key).unwrap();
    assert_eq!(entry.reference_count, 1);

    // Duplicate add throws DeveloperError
    assert!(cache.add(Arc::clone(&loader)).is_err());
}

#[test]
fn test_destroys_resource_when_reference_count_reaches_zero() {
    let mut cache = ResourceCache::new();
    let cache_key = "test-resource-destroy";

    let loader: Arc<Mutex<dyn ResourceLoader>> =
        Arc::new(Mutex::new(BaseResourceLoader::with_cache_key(cache_key)));

    cache.add(Arc::clone(&loader)).unwrap();

    // Increment reference count via get()
    let fetched = cache.get(cache_key).unwrap();
    assert_eq!(cache.cache_entries().get(cache_key).unwrap().reference_count, 2);

    // First unload: reference count becomes 1, loader not destroyed
    cache.unload(&fetched).unwrap();
    assert_eq!(cache.cache_entries().get(cache_key).unwrap().reference_count, 1);
    assert!(!loader.lock().unwrap().is_destroyed());

    // Second unload: reference count becomes 0, loader is destroyed and removed from cache
    cache.unload(&loader).unwrap();
    assert!(cache.cache_entries().get(cache_key).is_none());
    assert!(loader.lock().unwrap().is_destroyed());

    // Unload again throws DeveloperError
    assert!(cache.unload(&loader).is_err());
}

#[test]
fn test_gets_resource_and_increments_reference_count() {
    let mut cache = ResourceCache::new();
    let cache_key = "test-resource-get";

    let loader: Arc<Mutex<dyn ResourceLoader>> =
        Arc::new(Mutex::new(BaseResourceLoader::with_cache_key(cache_key)));

    cache.add(loader).unwrap();

    assert_eq!(cache.cache_entries().get(cache_key).unwrap().reference_count, 1);

    assert!(cache.get(cache_key).is_some());
    assert_eq!(cache.cache_entries().get(cache_key).unwrap().reference_count, 2);

    assert!(cache.get(cache_key).is_some());
    assert_eq!(cache.cache_entries().get(cache_key).unwrap().reference_count, 3);

    // Unknown cache key returns None
    assert!(cache.get("unknown-key").is_none());
}

#[test]
fn test_gets_schema_loader() {
    let mut cache = ResourceCache::new();
    let schema_json = json!({"classes": {}});

    let expected_key = ResourceCacheKey::get_schema_cache_key(SchemaCacheKeyOptions {
        schema: Some(&schema_json),
        resource: None,
    })
    .unwrap();

    let loader1 = cache.get_schema_loader(Some(&schema_json), None).unwrap();
    assert_eq!(loader1.lock().unwrap().cache_key().unwrap(), expected_key);
    assert_eq!(
        cache.cache_entries().get(&expected_key).unwrap().reference_count,
        1
    );

    // Re-getting returns the same cached loader and increments reference count
    let _loader2 = cache.get_schema_loader(Some(&schema_json), None).unwrap();
    assert_eq!(
        cache.cache_entries().get(&expected_key).unwrap().reference_count,
        2
    );
}

#[test]
fn test_gets_embedded_and_external_buffer_loaders() {
    let mut cache = ResourceCache::new();
    let parent_res = "https://example.com/model.glb";
    let data = vec![1, 2, 3, 4, 5];

    let embedded_loader = cache
        .get_embedded_buffer_loader(Some(parent_res), Some(0), Some(data))
        .unwrap();
    let emb_key = ResourceCacheKey::get_embedded_buffer_cache_key(parent_res, 0);
    assert_eq!(
        embedded_loader.lock().unwrap().cache_key().unwrap(),
        emb_key
    );

    let ext_res = "https://example.com/external.bin";
    let ext_loader = cache.get_external_buffer_loader(Some(ext_res)).unwrap();
    let ext_key = ResourceCacheKey::get_external_buffer_cache_key(ext_res);
    assert_eq!(ext_loader.lock().unwrap().cache_key().unwrap(), ext_key);

    // Missing arguments error
    assert!(cache
        .get_embedded_buffer_loader(None, Some(0), None)
        .is_err());
    assert!(cache.get_external_buffer_loader(None).is_err());
}

#[test]
fn test_gets_buffer_view_and_draco_and_spz_loaders() {
    let mut cache = ResourceCache::new();
    let gltf_res = "https://example.com/model.glb";
    let base_res = "https://example.com/resources/";

    let gltf = json!({
        "buffers": [{"uri": "external.bin", "byteLength": 100}],
        "bufferViews": [
            {"buffer": 0, "byteOffset": 0, "byteLength": 40},
            {"buffer": 0, "byteOffset": 40, "byteLength": 40}
        ]
    });

    let bv_loader = cache
        .get_buffer_view_loader(Some(&gltf), Some(0), Some(gltf_res), Some(base_res))
        .unwrap();
    let bv_key = ResourceCacheKey::get_buffer_view_cache_key(&gltf, 0, gltf_res, base_res).unwrap();
    assert_eq!(bv_loader.lock().unwrap().cache_key().unwrap(), bv_key);

    // Draco loader
    let draco = json!({"bufferView": 0});
    let primitive = json!({"attributes": {"POSITION": 0}});
    let draco_loader = cache
        .get_draco_loader(
            Some(&gltf),
            Some(&primitive),
            Some(&draco),
            Some(gltf_res),
            Some(base_res),
        )
        .unwrap();
    let draco_key = ResourceCacheKey::get_draco_cache_key(&gltf, &draco, gltf_res, base_res).unwrap();
    assert_eq!(draco_loader.lock().unwrap().cache_key().unwrap(), draco_key);

    // SPZ loaders distinct for different buffer views
    let spz0 = json!({"bufferView": 0});
    let spz1 = json!({"bufferView": 1});
    let spz_loader0 = cache
        .get_spz_loader(
            Some(&gltf),
            Some(&primitive),
            Some(&spz0),
            Some(gltf_res),
            Some(base_res),
        )
        .unwrap();
    let spz_loader1 = cache
        .get_spz_loader(
            Some(&gltf),
            Some(&primitive),
            Some(&spz1),
            Some(gltf_res),
            Some(base_res),
        )
        .unwrap();

    let spz_key0 = ResourceCacheKey::get_spz_cache_key(&gltf, &spz0, gltf_res, base_res).unwrap();
    let spz_key1 = ResourceCacheKey::get_spz_cache_key(&gltf, &spz1, gltf_res, base_res).unwrap();
    assert_ne!(spz_key0, spz_key1);
    assert_eq!(spz_loader0.lock().unwrap().cache_key().unwrap(), spz_key0);
    assert_eq!(spz_loader1.lock().unwrap().cache_key().unwrap(), spz_key1);
}

#[test]
fn test_gets_vertex_buffer_loaders_on_different_contexts() {
    let mut cache = ResourceCache::new();
    let gltf_res = "https://example.com/model.glb";
    let base_res = "https://example.com/resources/";

    let gltf = json!({
        "buffers": [{"uri": "external.bin", "byteLength": 100}],
        "bufferViews": [{"buffer": 0, "byteOffset": 0, "byteLength": 40}]
    });

    let loader_ctx1 = cache
        .get_vertex_buffer_loader(VertexBufferCacheKeyOptions {
            gltf: Some(&gltf),
            gltf_resource: Some(gltf_res),
            base_resource: Some(base_res),
            context_id: Some("context-1"),
            buffer_view_id: Some(0),
            load_buffer: true,
            ..Default::default()
        })
        .unwrap();

    let loader_ctx2 = cache
        .get_vertex_buffer_loader(VertexBufferCacheKeyOptions {
            gltf: Some(&gltf),
            gltf_resource: Some(gltf_res),
            base_resource: Some(base_res),
            context_id: Some("context-2"),
            buffer_view_id: Some(0),
            load_buffer: true,
            ..Default::default()
        })
        .unwrap();

    assert_ne!(
        loader_ctx1.lock().unwrap().cache_key().unwrap(),
        loader_ctx2.lock().unwrap().cache_key().unwrap()
    );
}
