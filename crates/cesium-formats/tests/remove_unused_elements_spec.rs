use cesium_formats::gltf_pipeline::remove_unused_elements;
use serde_json::json;

fn create_full_gltf() -> serde_json::Value {
    json!({
        "nodes": [
            { "name": "skin", "skin": 0, "mesh": 0, "translation": [0.0, 0.0, 0.0] },
            { "name": "used", "mesh": 0, "children": [2, 5] },
            { "name": "unused" },
            { "name": "nodeWithEmptyMesh", "mesh": 1 },
            { "name": "unusedParent", "children": [2] },
            { "name": "camera", "camera": 0 },
            {
                "name": "light",
                "extensions": {
                    "KHR_lights_punctual": {
                        "light": 0
                    }
                }
            }
        ],
        "cameras": [
            { "name": "cam", "type": "perspective", "perspective": { "yfov": 0.6, "znear": 1.0 } }
        ],
        "buffers": [
            { "name": "mesh", "byteLength": 246, "uri": "data0.bin" },
            { "name": "other", "byteLength": 80, "uri": "data1.bin" },
            { "name": "image01", "byteLength": 1000, "uri": "data2.bin" },
            { "image": "image2", "byteLength": 500, "uri": "data3.bin" }
        ],
        "bufferViews": [
            { "name": "positions", "buffer": 0, "byteOffset": 0, "byteLength": 36 },
            { "name": "normals", "buffer": 0, "byteOffset": 36, "byteLength": 36 },
            { "name": "texcoords", "buffer": 0, "byteOffset": 72, "byteLength": 24 },
            { "name": "positions-target0", "buffer": 0, "byteOffset": 96, "byteLength": 36 },
            { "name": "normals-target0", "buffer": 0, "byteOffset": 132, "byteLength": 36 },
            { "name": "positions-target1", "buffer": 0, "byteOffset": 168, "byteLength": 36 },
            { "name": "normals-target1", "buffer": 0, "byteOffset": 204, "byteLength": 36 },
            { "name": "indices", "buffer": 0, "byteOffset": 240, "byteLength": 6 },
            { "name": "other", "buffer": 1, "byteOffset": 0, "byteLength": 80 },
            { "name": "image0", "buffer": 2, "byteOffset": 0, "byteLength": 500 },
            { "name": "image1", "buffer": 2, "byteOffset": 500, "byteLength": 500 },
            { "name": "image2", "buffer": 3, "byteOffset": 1000, "byteLength": 500 }
        ],
        "accessors": [
            { "name": "positions", "bufferView": 0, "byteOffset": 0, "componentType": 5126, "count": 3, "type": "VEC3" },
            { "name": "normals", "bufferView": 1, "byteOffset": 36, "componentType": 5126, "count": 3, "type": "VEC3" },
            { "name": "texcoords", "bufferView": 2, "byteOffset": 72, "componentType": 5126, "count": 3, "type": "VEC2" },
            { "name": "positions-target0", "bufferView": 3, "byteOffset": 96, "componentType": 5126, "count": 3, "type": "VEC3" },
            { "name": "normals-target0", "bufferView": 4, "byteOffset": 132, "componentType": 5126, "count": 3, "type": "VEC3" },
            { "name": "positions-target1", "bufferView": 5, "byteOffset": 168, "componentType": 5126, "count": 3, "type": "VEC3" },
            { "name": "normals-target1", "bufferView": 6, "byteOffset": 204, "componentType": 5126, "count": 3, "type": "VEC3" },
            { "name": "indices", "bufferView": 7, "byteOffset": 240, "componentType": 5123, "count": 3, "type": "SCALAR" },
            { "name": "bone-matrix", "bufferView": 8, "byteOffset": 0, "componentType": 5126, "count": 1, "type": "MAT4" },
            { "name": "times", "bufferView": 8, "byteOffset": 64, "componentType": 5126, "count": 1, "type": "SCALAR" },
            { "name": "translations", "bufferView": 8, "byteOffset": 68, "componentType": 5126, "count": 1, "type": "VEC3" }
        ],
        "meshes": [
            {
                "name": "mesh0",
                "primitives": [
                    {
                        "attributes": {
                            "POSITION": 0,
                            "NORMAL": 1,
                            "TEXCOORD_0": 2
                        },
                        "targets": [
                            { "POSITION": 3, "NORMAL": 4 },
                            { "POSITION": 5, "NORMAL": 6 }
                        ],
                        "indices": 7,
                        "mode": 4,
                        "material": 1
                    }
                ]
            },
            {
                "name": "empty-mesh",
                "primitives": []
            }
        ],
        "materials": [
            { "name": "unused" },
            {
                "name": "used",
                "pbrMetallicRoughness": {
                    "baseColorTexture": { "index": 0 }
                }
            }
        ],
        "textures": [
            { "name": "texture", "sampler": 0, "source": 1 },
            { "name": "unusedTexture" }
        ],
        "images": [
            { "bufferView": 9, "mimeType": "image/png" },
            { "bufferView": 10, "mimeType": "image/png" },
            { "bufferView": 11, "mimeType": "image/png" },
            { "name": "image", "bufferView": 12, "mimeType": "image/png" }
        ],
        "extensions": {
            "KHR_lights_punctual": {
                "lights": [
                    { "name": "sun", "type": "directional" }
                ]
            }
        },
        "samplers": [
            { "name": "sampler", "magFilter": 9729, "minFilter": 9987, "wrapS": 33648, "wrapt": 33648 },
            { "magFilter": 9729, "minFilter": 9987, "wrapS": 33648, "wrapt": 33648 }
        ],
        "scenes": [
            { "nodes": [2, 3] }
        ]
    })
}

#[test]
fn test_removes_unused_elements() {
    let mut gltf = create_full_gltf();

    // Mirror upstream pre-test setup:
    // gltf.meshes[0].primitives[0].targets.splice(0, 1);
    gltf["meshes"][0]["primitives"][0]["targets"].as_array_mut().unwrap().remove(0);
    // gltf.images.splice(1, 2);
    gltf["images"].as_array_mut().unwrap().remove(1);
    gltf["images"].as_array_mut().unwrap().remove(1);

    remove_unused_elements(&mut gltf, None);

    let remaining_accessor_names = [
        "positions",
        "normals",
        "texcoords",
        "positions-target1",
        "normals-target1",
        "indices",
    ];
    let remaining_accessor_bufferview_ids = [0, 1, 2, 3, 4, 5];

    let accessors = gltf["accessors"].as_array().unwrap();
    assert_eq!(accessors.len(), remaining_accessor_names.len());
    for (i, acc) in accessors.iter().enumerate() {
        assert_eq!(acc["name"], remaining_accessor_names[i]);
        assert_eq!(acc["bufferView"], remaining_accessor_bufferview_ids[i]);
    }

    let remaining_bufferview_names = [
        "positions",
        "normals",
        "texcoords",
        "positions-target1",
        "normals-target1",
        "indices",
        "image0",
    ];
    let remaining_bufferview_buffer_ids = [0, 0, 0, 0, 0, 0, 1];

    let bufferviews = gltf["bufferViews"].as_array().unwrap();
    assert_eq!(bufferviews.len(), remaining_bufferview_names.len());
    for (i, bv) in bufferviews.iter().enumerate() {
        assert_eq!(bv["name"], remaining_bufferview_names[i]);
        assert_eq!(bv["buffer"], remaining_bufferview_buffer_ids[i]);
    }

    assert_eq!(gltf["nodes"].as_array().unwrap().len(), 4);
    assert_eq!(gltf["meshes"].as_array().unwrap().len(), 1);
    assert_eq!(gltf["buffers"].as_array().unwrap().len(), 2);
    assert_eq!(gltf["materials"].as_array().unwrap().len(), 1);
    assert_eq!(gltf["textures"].as_array().unwrap().len(), 1);
    assert_eq!(gltf["images"].as_array().unwrap().len(), 1);
    assert_eq!(gltf["samplers"].as_array().unwrap().len(), 1);
}

#[test]
fn test_keeps_khr_materials_transmission_texture() {
    let mut gltf = json!({
        "materials": [
            {
                "extensions": {
                    "KHR_materials_transmission": {
                        "transmissionTexture": { "index": 1 }
                    }
                }
            }
        ],
        "textures": [
            { "source": 0, "sampler": 0 },
            { "source": 1, "sampler": 0 }
        ],
        "images": [{ "name": "img0" }, { "name": "img1" }],
        "samplers": [{}],
        "meshes": [
            {
                "primitives": [
                    { "material": 0 }
                ]
            }
        ]
    });

    let orig_name = gltf["images"][1]["name"].as_str().unwrap().to_string();

    remove_unused_elements(&mut gltf, Some(&["texture", "sampler", "image"]));

    assert_eq!(gltf["textures"].as_array().unwrap().len(), 1);
    let new_idx = gltf["materials"][0]["extensions"]["KHR_materials_transmission"]["transmissionTexture"]["index"].as_u64().unwrap() as usize;
    let source_idx = gltf["textures"][new_idx]["source"].as_u64().unwrap() as usize;
    assert_eq!(gltf["images"][source_idx]["name"], orig_name);
}

#[test]
fn test_keeps_pbr_base_color_and_khr_materials_transmission_textures() {
    let mut gltf = json!({
        "materials": [
            {
                "pbrMetallicRoughness": {
                    "baseColorTexture": { "index": 3 }
                },
                "extensions": {
                    "KHR_materials_transmission": {
                        "transmissionTexture": { "index": 2 }
                    }
                }
            }
        ],
        "textures": [
            { "source": 0, "sampler": 0 },
            { "source": 1, "sampler": 0 },
            { "source": 2, "sampler": 0 },
            { "source": 3, "sampler": 0 }
        ],
        "images": [
            { "name": "img0" },
            { "name": "img1" },
            { "name": "img2" },
            { "name": "img3" }
        ],
        "samplers": [{}],
        "meshes": [
            {
                "primitives": [
                    { "material": 0 }
                ]
            }
        ]
    });

    let orig_trans_name = gltf["images"][2]["name"].as_str().unwrap().to_string();
    let orig_base_name = gltf["images"][3]["name"].as_str().unwrap().to_string();

    remove_unused_elements(&mut gltf, Some(&["texture", "sampler", "image"]));

    assert_eq!(gltf["textures"].as_array().unwrap().len(), 2);

    let mat = &gltf["materials"][0];
    let new_trans_idx = mat["extensions"]["KHR_materials_transmission"]["transmissionTexture"]["index"].as_u64().unwrap() as usize;
    let new_base_idx = mat["pbrMetallicRoughness"]["baseColorTexture"]["index"].as_u64().unwrap() as usize;

    let trans_source = gltf["textures"][new_trans_idx]["source"].as_u64().unwrap() as usize;
    let base_source = gltf["textures"][new_base_idx]["source"].as_u64().unwrap() as usize;

    assert_eq!(gltf["images"][trans_source]["name"], orig_trans_name);
    assert_eq!(gltf["images"][base_source]["name"], orig_base_name);
    assert_ne!(new_trans_idx, new_base_idx);
}
