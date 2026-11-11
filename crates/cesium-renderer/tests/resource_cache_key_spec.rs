//! ResourceCacheKeySpec matching CesiumJS `Scene/ResourceCacheKeySpec.js` (Apache-2.0).

use cesium_renderer::{
    IndexBufferCacheKeyOptions, ResourceCacheKey, SchemaCacheKeyOptions, SupportedImageFormats,
    TextureCacheKeyOptions, VertexBufferCacheKeyOptions,
};
use serde_json::json;

#[test]
fn test_get_schema_cache_key() {
    let schema_uri = "https://example.com/schema.json";
    let external_key = ResourceCacheKey::get_schema_cache_key(SchemaCacheKeyOptions {
        schema: None,
        resource: Some(schema_uri),
    })
    .unwrap();
    assert_eq!(external_key, format!("external-schema:{schema_uri}"));

    let schema_json = json!({"classes": {}});
    let embedded_key = ResourceCacheKey::get_schema_cache_key(SchemaCacheKeyOptions {
        schema: Some(&schema_json),
        resource: None,
    })
    .unwrap();
    assert_eq!(
        embedded_key,
        format!("embedded-schema:{}", serde_json::to_string(&schema_json).unwrap())
    );

    // Both defined -> error
    assert!(ResourceCacheKey::get_schema_cache_key(SchemaCacheKeyOptions {
        schema: Some(&schema_json),
        resource: Some(schema_uri),
    })
    .is_err());

    // Neither defined -> error
    assert!(ResourceCacheKey::get_schema_cache_key(SchemaCacheKeyOptions {
        schema: None,
        resource: None,
    })
    .is_err());
}

#[test]
fn test_buffer_and_gltf_cache_keys() {
    let buffer_uri = "https://example.com/external.bin";
    assert_eq!(
        ResourceCacheKey::get_external_buffer_cache_key(buffer_uri),
        format!("external-buffer:{buffer_uri}")
    );

    let gltf_uri = "https://example.com/parent.gltf";
    assert_eq!(
        ResourceCacheKey::get_embedded_buffer_cache_key(gltf_uri, 0),
        format!("embedded-buffer:{gltf_uri}-buffer-id-0")
    );

    assert_eq!(
        ResourceCacheKey::get_gltf_cache_key(gltf_uri),
        format!("gltf:{gltf_uri}")
    );
}

#[test]
fn test_buffer_view_cache_key() {
    let gltf_uri = "https://example.com/parent.gltf";
    let base_uri = "https://example.com/resources/";

    let gltf_embedded = json!({
        "buffers": [{"byteLength": 100}],
        "bufferViews": [{"buffer": 0, "byteOffset": 0, "byteLength": 100}]
    });
    let key = ResourceCacheKey::get_buffer_view_cache_key(&gltf_embedded, 0, gltf_uri, base_uri)
        .unwrap();
    assert_eq!(key, format!("buffer-view:{gltf_uri}-buffer-id-0-range-0-100"));

    let gltf_external = json!({
        "buffers": [{"uri": "external.bin", "byteLength": 100}],
        "bufferViews": [{"buffer": 0, "byteOffset": 0, "byteLength": 100}]
    });
    let key = ResourceCacheKey::get_buffer_view_cache_key(&gltf_external, 0, gltf_uri, base_uri)
        .unwrap();
    assert_eq!(
        key,
        "buffer-view:https://example.com/resources/external.bin-range-0-100"
    );

    let gltf_meshopt_ext = json!({
        "buffers": [{"byteLength": 100}],
        "bufferViews": [{
            "buffer": 0,
            "byteOffset": 0,
            "byteLength": 100,
            "extensions": {
                "EXT_meshopt_compression": {
                    "buffer": 1,
                    "byteOffset": 25,
                    "byteLength": 50
                }
            }
        }]
    });
    let key =
        ResourceCacheKey::get_buffer_view_cache_key(&gltf_meshopt_ext, 0, gltf_uri, base_uri)
            .unwrap();
    assert_eq!(key, format!("buffer-view:{gltf_uri}-buffer-id-1-range-25-75"));

    let gltf_meshopt_khr = json!({
        "buffers": [{"byteLength": 100}],
        "bufferViews": [{
            "buffer": 0,
            "byteOffset": 0,
            "byteLength": 100,
            "extensions": {
                "KHR_meshopt_compression": {
                    "buffer": 1,
                    "byteOffset": 25,
                    "byteLength": 50
                }
            }
        }]
    });
    let key =
        ResourceCacheKey::get_buffer_view_cache_key(&gltf_meshopt_khr, 0, gltf_uri, base_uri)
            .unwrap();
    assert_eq!(key, format!("buffer-view:{gltf_uri}-buffer-id-1-range-25-75"));
}

#[test]
fn test_draco_and_spz_cache_keys() {
    let gltf_uri = "https://example.com/parent.gltf";
    let base_uri = "https://example.com/resources/";

    let gltf = json!({
        "buffers": [{"uri": "external.bin", "byteLength": 100}],
        "bufferViews": [
            {"buffer": 0, "byteOffset": 0, "byteLength": 40},
            {"buffer": 0, "byteOffset": 40, "byteLength": 40}
        ]
    });

    let draco = json!({"bufferView": 0});
    let draco_key = ResourceCacheKey::get_draco_cache_key(&gltf, &draco, gltf_uri, base_uri).unwrap();
    assert_eq!(
        draco_key,
        "draco:https://example.com/resources/external.bin-range-0-40"
    );

    let spz = json!({"bufferView": 1});
    let spz_key = ResourceCacheKey::get_spz_cache_key(&gltf, &spz, gltf_uri, base_uri).unwrap();
    assert_eq!(
        spz_key,
        "spz:https://example.com/resources/external.bin-range-40-80"
    );
}

#[test]
fn test_vertex_buffer_cache_key() {
    let gltf_uri = "https://example.com/parent.gltf";
    let base_uri = "https://example.com/resources/";
    let context_id = "01234";

    let gltf_uncompressed = json!({
        "buffers": [{"uri": "external.bin", "byteLength": 100}],
        "bufferViews": [
            {"buffer": 0, "byteOffset": 0, "byteLength": 40},
            {"buffer": 0, "byteOffset": 40, "byteLength": 40}
        ]
    });

    // 1. From buffer view
    let key = ResourceCacheKey::get_vertex_buffer_cache_key(VertexBufferCacheKeyOptions {
        gltf: Some(&gltf_uncompressed),
        gltf_resource: Some(gltf_uri),
        base_resource: Some(base_uri),
        context_id: Some(context_id),
        buffer_view_id: Some(0),
        load_buffer: true,
        ..Default::default()
    })
    .unwrap();
    assert_eq!(
        key,
        "vertex-buffer:https://example.com/resources/external.bin-range-0-40-buffer-context-01234"
    );

    // 2. From draco
    let gltf_draco = json!({
        "buffers": [{"uri": "external.bin", "byteLength": 100}],
        "bufferViews": [{"buffer": 0, "byteOffset": 0, "byteLength": 100}]
    });
    let draco = json!({
        "bufferView": 0,
        "attributes": { "POSITION": 0, "NORMAL": 1 }
    });
    let key = ResourceCacheKey::get_vertex_buffer_cache_key(VertexBufferCacheKeyOptions {
        gltf: Some(&gltf_draco),
        gltf_resource: Some(gltf_uri),
        base_resource: Some(base_uri),
        context_id: Some(context_id),
        draco: Some(&draco),
        attribute_semantic: Some("POSITION"),
        load_buffer: true,
        ..Default::default()
    })
    .unwrap();
    assert_eq!(
        key,
        "vertex-buffer:https://example.com/resources/external.bin-range-0-100-draco-POSITION-buffer-context-01234"
    );

    // 3. From spz
    let spz = json!({"bufferView": 1});
    let key = ResourceCacheKey::get_vertex_buffer_cache_key(VertexBufferCacheKeyOptions {
        gltf: Some(&gltf_uncompressed),
        gltf_resource: Some(gltf_uri),
        base_resource: Some(base_uri),
        context_id: Some(context_id),
        spz: Some(&spz),
        attribute_semantic: Some("POSITION"),
        load_buffer: true,
        ..Default::default()
    })
    .unwrap();
    assert_eq!(
        key,
        "vertex-buffer:https://example.com/resources/external.bin-range-40-80-spz-POSITION-buffer-context-01234"
    );

    // 4. From spz with unrelated draco
    let spz_zero = json!({"bufferView": 0});
    let key = ResourceCacheKey::get_vertex_buffer_cache_key(VertexBufferCacheKeyOptions {
        gltf: Some(&gltf_draco),
        gltf_resource: Some(gltf_uri),
        base_resource: Some(base_uri),
        context_id: Some(context_id),
        draco: Some(&draco),
        spz: Some(&spz_zero),
        attribute_semantic: Some("TANGENT"),
        load_buffer: true,
        ..Default::default()
    })
    .unwrap();
    assert_eq!(
        key,
        "vertex-buffer:https://example.com/resources/external.bin-range-0-100-spz-TANGENT-buffer-context-01234"
    );

    // 5. Dequantize + loadTypedArray + loadBuffer
    let key = ResourceCacheKey::get_vertex_buffer_cache_key(VertexBufferCacheKeyOptions {
        gltf: Some(&gltf_uncompressed),
        gltf_resource: Some(gltf_uri),
        base_resource: Some(base_uri),
        context_id: Some(context_id),
        buffer_view_id: Some(0),
        dequantize: true,
        load_buffer: true,
        load_typed_array: true,
        ..Default::default()
    })
    .unwrap();
    assert_eq!(
        key,
        "vertex-buffer:https://example.com/resources/external.bin-range-0-40-dequantize-buffer-context-01234-typed-array"
    );

    // 6. Validation errors
    // Both loadBuffer and loadTypedArray false
    assert!(ResourceCacheKey::get_vertex_buffer_cache_key(VertexBufferCacheKeyOptions {
        gltf: Some(&gltf_uncompressed),
        gltf_resource: Some(gltf_uri),
        base_resource: Some(base_uri),
        context_id: Some(context_id),
        buffer_view_id: Some(0),
        load_buffer: false,
        load_typed_array: false,
        ..Default::default()
    })
    .is_err());

    // Both bufferViewId and draco defined
    assert!(ResourceCacheKey::get_vertex_buffer_cache_key(VertexBufferCacheKeyOptions {
        gltf: Some(&gltf_draco),
        gltf_resource: Some(gltf_uri),
        base_resource: Some(base_uri),
        context_id: Some(context_id),
        buffer_view_id: Some(0),
        draco: Some(&draco),
        attribute_semantic: Some("POSITION"),
        load_buffer: true,
        ..Default::default()
    })
    .is_err());

    // Draco defined but attribute_semantic is None
    assert!(ResourceCacheKey::get_vertex_buffer_cache_key(VertexBufferCacheKeyOptions {
        gltf: Some(&gltf_draco),
        gltf_resource: Some(gltf_uri),
        base_resource: Some(base_uri),
        context_id: Some(context_id),
        draco: Some(&draco),
        attribute_semantic: None,
        load_buffer: true,
        ..Default::default()
    })
    .is_err());
}

#[test]
fn test_index_buffer_cache_key() {
    let gltf_uri = "https://example.com/parent.gltf";
    let base_uri = "https://example.com/resources/";
    let context_id = "01234";

    let gltf = json!({
        "buffers": [{"uri": "external.bin", "byteLength": 100}],
        "bufferViews": [
            {"buffer": 0, "byteOffset": 80, "byteLength": 20}
        ],
        "accessors": [
            {"bufferView": 0, "byteOffset": 0, "componentType": 5123, "type": "SCALAR", "count": 36}
        ]
    });

    let key = ResourceCacheKey::get_index_buffer_cache_key(IndexBufferCacheKeyOptions {
        gltf: Some(&gltf),
        accessor_id: Some(0),
        gltf_resource: Some(gltf_uri),
        base_resource: Some(base_uri),
        context_id: Some(context_id),
        load_buffer: true,
        load_typed_array: false,
        ..Default::default()
    })
    .unwrap();
    assert_eq!(
        key,
        "index-buffer:https://example.com/resources/external.bin-accessor-80-5123-SCALAR-36-buffer-context-01234"
    );

    let key_both = ResourceCacheKey::get_index_buffer_cache_key(IndexBufferCacheKeyOptions {
        gltf: Some(&gltf),
        accessor_id: Some(0),
        gltf_resource: Some(gltf_uri),
        base_resource: Some(base_uri),
        context_id: Some(context_id),
        load_buffer: true,
        load_typed_array: true,
        ..Default::default()
    })
    .unwrap();
    assert_eq!(
        key_both,
        "index-buffer:https://example.com/resources/external.bin-accessor-80-5123-SCALAR-36-buffer-context-01234-typed-array"
    );

    // Draco index buffer
    let draco = json!({"bufferView": 0});
    let key_draco = ResourceCacheKey::get_index_buffer_cache_key(IndexBufferCacheKeyOptions {
        gltf: Some(&gltf),
        accessor_id: Some(0),
        gltf_resource: Some(gltf_uri),
        base_resource: Some(base_uri),
        context_id: Some(context_id),
        draco: Some(&draco),
        load_buffer: true,
        load_typed_array: false,
    })
    .unwrap();
    assert_eq!(
        key_draco,
        "index-buffer:https://example.com/resources/external.bin-range-80-100-draco-buffer-context-01234"
    );
}

#[test]
fn test_image_and_texture_cache_keys() {
    let gltf_uri = "https://example.com/parent.gltf";
    let base_uri = "https://example.com/resources/";
    let context_id = "01234";

    let gltf = json!({
        "buffers": [{"uri": "external.bin", "byteLength": 100}],
        "bufferViews": [{"buffer": 0, "byteOffset": 0, "byteLength": 100}],
        "images": [
            {"uri": "image.png"},
            {"mimeType": "image/jpeg", "bufferView": 0},
            {"uri": "image.webp"}
        ],
        "textures": [
            {"source": 0},
            {"source": 1, "sampler": 0},
            {
                "source": 0,
                "extensions": {
                    "EXT_texture_webp": {"source": 2}
                }
            }
        ],
        "samplers": [
            {"wrapS": 33071, "wrapT": 33648, "minFilter": 9984, "magFilter": 9728}
        ]
    });

    // Image from URI
    assert_eq!(
        ResourceCacheKey::get_image_cache_key(&gltf, 0, gltf_uri, base_uri).unwrap(),
        "image:https://example.com/resources/image.png"
    );

    // Image from buffer view
    assert_eq!(
        ResourceCacheKey::get_image_cache_key(&gltf, 1, gltf_uri, base_uri).unwrap(),
        "image:https://example.com/resources/external.bin-range-0-100"
    );

    // Texture default sampler
    let tex_key_default = ResourceCacheKey::get_texture_cache_key(TextureCacheKeyOptions {
        gltf: Some(&gltf),
        texture_info: Some(&json!({"index": 0})),
        gltf_resource: Some(gltf_uri),
        base_resource: Some(base_uri),
        context_id: Some(context_id),
        supported_image_formats: SupportedImageFormats::default(),
    })
    .unwrap();
    assert_eq!(
        tex_key_default,
        "texture:https://example.com/resources/image.png-sampler-10497-10497-9729-9729-context-01234"
    );

    // Texture explicit sampler
    let tex_key_explicit = ResourceCacheKey::get_texture_cache_key(TextureCacheKeyOptions {
        gltf: Some(&gltf),
        texture_info: Some(&json!({"index": 1})),
        gltf_resource: Some(gltf_uri),
        base_resource: Some(base_uri),
        context_id: Some(context_id),
        supported_image_formats: SupportedImageFormats::default(),
    })
    .unwrap();
    assert_eq!(
        tex_key_explicit,
        "texture:https://example.com/resources/external.bin-range-0-100-sampler-33071-33648-9984-9728-context-01234"
    );

    // Texture with webp extension supported
    let tex_key_webp = ResourceCacheKey::get_texture_cache_key(TextureCacheKeyOptions {
        gltf: Some(&gltf),
        texture_info: Some(&json!({"index": 2})),
        gltf_resource: Some(gltf_uri),
        base_resource: Some(base_uri),
        context_id: Some(context_id),
        supported_image_formats: SupportedImageFormats {
            webp: true,
            basis: false,
        },
    })
    .unwrap();
    assert_eq!(
        tex_key_webp,
        "texture:https://example.com/resources/image.webp-sampler-10497-10497-9729-9729-context-01234"
    );

    // Texture with webp extension ignored when webp unsupported
    let tex_key_no_webp = ResourceCacheKey::get_texture_cache_key(TextureCacheKeyOptions {
        gltf: Some(&gltf),
        texture_info: Some(&json!({"index": 2})),
        gltf_resource: Some(gltf_uri),
        base_resource: Some(base_uri),
        context_id: Some(context_id),
        supported_image_formats: SupportedImageFormats {
            webp: false,
            basis: false,
        },
    })
    .unwrap();
    assert_eq!(
        tex_key_no_webp,
        "texture:https://example.com/resources/image.png-sampler-10497-10497-9729-9729-context-01234"
    );
}
