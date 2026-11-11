use cesium_formats::gltf_for_each::{uses_extension, ForEach, GltfId};
use serde_json::json;

#[test]
fn test_loops_over_accessors() {
    let gltf = json!({
        "accessors": [
            { "bufferView": 0 },
            { "bufferView": 1 },
            { "bufferView": 2 }
        ]
    });

    ForEach::accessor(&gltf, |accessor, id| {
        if let GltfId::Index(idx) = id {
            assert_eq!(accessor["bufferView"], idx);
        }
        None::<()>
    });

    let return_value = ForEach::accessor(&gltf, |accessor, id| {
        if id == GltfId::Index(1) {
            Some(accessor.clone())
        } else {
            None
        }
    });

    assert_eq!(return_value, Some(gltf["accessors"][1].clone()));
}

#[test]
fn test_loops_over_accessor_with_semantic() {
    let gltf = json!({
        "accessors": [
            { "bufferView": 0 },
            { "bufferView": 1 },
            { "bufferView": 2 },
            { "bufferView": 3 },
            { "bufferView": 4 },
            { "bufferView": 5 },
            { "bufferView": 6 },
            { "bufferView": 7 },
            { "bufferView": 8 },
            { "bufferView": 9 },
            { "bufferView": 10 },
            { "bufferView": 11 }
        ],
        "meshes": [
            {
                "primitives": [
                    {
                        "attributes": { "POSITION": 2, "NORMAL": 3 },
                        "indices": 0,
                        "targets": [
                            { "POSITION": 4, "NORMAL": 5 },
                            { "POSITION": 6, "NORMAL": 7 }
                        ]
                    }
                ]
            },
            {
                "primitives": [
                    {
                        "attributes": { "POSITION": 8, "NORMAL": 9 },
                        "indices": 1,
                        "targets": [
                            { "POSITION": 10, "NORMAL": 11 }
                        ]
                    }
                ]
            }
        ]
    });

    let mut position_accessor_count = 0;
    let return_value = ForEach::accessor_with_semantic(&gltf, "POSITION", |accessor_id| {
        let id_val = accessor_id.as_u64().unwrap() as usize;
        assert_eq!(gltf["accessors"][id_val]["bufferView"], id_val);
        position_accessor_count += 1;
        if position_accessor_count == 5 {
            Some(id_val)
        } else {
            None
        }
    });

    assert_eq!(position_accessor_count, 5);
    assert_eq!(return_value, Some(10));
}

#[test]
fn test_loops_over_accessors_containing_vertex_data() {
    let gltf = json!({
        "accessors": [
            { "bufferView": 0 }, { "bufferView": 1 }, { "bufferView": 2 },
            { "bufferView": 3 }, { "bufferView": 4 }, { "bufferView": 5 },
            { "bufferView": 6 }, { "bufferView": 7 }, { "bufferView": 8 },
            { "bufferView": 9 }, { "bufferView": 10 }, { "bufferView": 11 }
        ],
        "meshes": [
            {
                "primitives": [
                    {
                        "attributes": { "POSITION": 2, "NORMAL": 3 },
                        "indices": 0,
                        "targets": [
                            { "POSITION": 4, "NORMAL": 5 },
                            { "POSITION": 6, "NORMAL": 7 }
                        ]
                    }
                ]
            },
            {
                "primitives": [
                    {
                        "attributes": { "POSITION": 8, "NORMAL": 9 },
                        "indices": 1,
                        "targets": [
                            { "POSITION": 10, "NORMAL": 11 }
                        ]
                    }
                ]
            }
        ]
    });

    let mut vertex_accessors_count = 0;
    let return_value = ForEach::accessor_containing_vertex_attribute_data(&gltf, |accessor_id| {
        let id_val = accessor_id.as_u64().unwrap() as usize;
        assert_eq!(gltf["accessors"][id_val]["bufferView"], id_val);
        vertex_accessors_count += 1;
        if vertex_accessors_count == 10 {
            Some(id_val)
        } else {
            None
        }
    });

    assert_eq!(vertex_accessors_count, 10);
    assert!(return_value == Some(10) || return_value == Some(11));
}

#[test]
fn test_loops_over_accessors_containing_index_data() {
    let gltf = json!({
        "accessors": [
            { "bufferView": 0 }, { "bufferView": 1 }
        ],
        "meshes": [
            { "primitives": [{ "indices": 0 }] },
            { "primitives": [{ "indices": 1 }] }
        ]
    });

    let mut count = 0;
    let return_value = ForEach::accessor_containing_index_data(&gltf, |accessor_id| {
        let id_val = accessor_id.as_u64().unwrap() as usize;
        assert_eq!(gltf["accessors"][id_val]["bufferView"], id_val);
        count += 1;
        if count == 2 {
            Some(id_val)
        } else {
            None
        }
    });

    assert_eq!(count, 2);
    assert_eq!(return_value, Some(1));
}

#[test]
fn test_loops_over_animations() {
    let gltf = json!({
        "animations": [
            {
                "channels": [
                    { "sampler": 0, "target": { "node": 0, "path": "translation" } },
                    { "sampler": 1, "target": { "node": 0, "path": "translation" } }
                ],
                "samplers": [
                    { "input": 0, "output": 2 },
                    { "input": 1, "output": 3 }
                ]
            },
            {
                "channels": [
                    { "sampler": 0, "target": { "node": 1, "path": "translation" } }
                ],
                "samplers": [
                    { "input": 4, "output": 5 }
                ]
            }
        ]
    });

    let return_value = ForEach::animation(&gltf, |animation, id| {
        if let GltfId::Index(idx) = id {
            assert_eq!(animation["channels"][0]["target"]["node"], idx);
            if idx == 1 {
                return Some(animation.clone());
            }
        }
        None
    });
    assert_eq!(return_value, Some(gltf["animations"][1].clone()));

    // animationChannel
    let chan_return = ForEach::animation_channel(&gltf["animations"][0], |channel, idx| {
        assert_eq!(channel["sampler"], idx);
        if idx == 1 {
            Some(channel.clone())
        } else {
            None
        }
    });
    assert_eq!(chan_return, Some(gltf["animations"][0]["channels"][1].clone()));

    // animationSampler
    let samp_return = ForEach::animation_sampler(&gltf["animations"][0], |sampler, idx| {
        assert_eq!(sampler["input"], idx);
        if idx == 1 {
            Some(sampler.clone())
        } else {
            None
        }
    });
    assert_eq!(samp_return, Some(gltf["animations"][0]["samplers"][1].clone()));
}

#[test]
fn test_loops_over_buffers_and_legacy() {
    let gltf = json!({
        "buffers": [
            { "uri": "0.bin", "byteLength": 10 },
            { "uri": "1.bin", "byteLength": 10 }
        ]
    });
    let return_val = ForEach::buffer(&gltf, |buffer, id| {
        if let GltfId::Index(idx) = id {
            assert_eq!(buffer["uri"], format!("{}.bin", idx));
            if idx == 1 {
                return Some(buffer.clone());
            }
        }
        None
    });
    assert_eq!(return_val, Some(gltf["buffers"][1].clone()));

    // glTF 1.0 buffers
    let gltf_legacy = json!({
        "buffers": {
            "buffer0": { "uri": "buffer0.bin" },
            "buffer1": { "uri": "buffer1.bin" }
        }
    });
    let legacy_ret = ForEach::buffer(&gltf_legacy, |buffer, id| {
        if let GltfId::Key(name) = id {
            assert_eq!(buffer["uri"], format!("{}.bin", name));
            if name == "buffer1" {
                return Some(buffer.clone());
            }
        }
        None
    });
    assert_eq!(legacy_ret, Some(gltf_legacy["buffers"]["buffer1"].clone()));
}

#[test]
fn test_loops_over_buffer_views_cameras_images_materials() {
    let gltf = json!({
        "bufferViews": [
            { "buffer": 0, "byteLength": 10 },
            { "buffer": 1, "byteLength": 10 }
        ],
        "cameras": [
            { "perspective": { "yfov": 0.0, "znear": 0.0 } },
            { "perspective": { "yfov": 1.0, "znear": 1.0 } }
        ],
        "images": [
            { "bufferView": 0 },
            { "bufferView": 1 }
        ],
        "materials": [
            { "emissiveTexture": 0 },
            { "emissiveTexture": 1 }
        ]
    });

    let bv_ret = ForEach::buffer_view(&gltf, |bv, id| {
        if let GltfId::Index(idx) = id {
            assert_eq!(bv["buffer"], idx);
            if idx == 1 {
                return Some(bv.clone());
            }
        }
        None
    });
    assert_eq!(bv_ret, Some(gltf["bufferViews"][1].clone()));

    let cam_ret = ForEach::camera(&gltf, |cam, id| {
        if let GltfId::Index(idx) = id {
            assert_eq!(cam["perspective"]["yfov"], idx as f64);
            if idx == 1 {
                return Some(cam.clone());
            }
        }
        None
    });
    assert_eq!(cam_ret, Some(gltf["cameras"][1].clone()));

    let img_ret = ForEach::image(&gltf, |img, id| {
        if let GltfId::Index(idx) = id {
            assert_eq!(img["bufferView"], idx);
            if idx == 1 {
                return Some(img.clone());
            }
        }
        None
    });
    assert_eq!(img_ret, Some(gltf["images"][1].clone()));

    let mat_ret = ForEach::material(&gltf, |mat, id| {
        if let GltfId::Index(idx) = id {
            assert_eq!(mat["emissiveTexture"], idx);
            if idx == 1 {
                return Some(mat.clone());
            }
        }
        None
    });
    assert_eq!(mat_ret, Some(gltf["materials"][1].clone()));
}

#[test]
fn test_loops_over_material_values_and_legacy() {
    let mat_khr = json!({
        "name": "Texture",
        "extensions": {
            "KHR_techniques_webgl": {
                "technique": 0,
                "values": {
                    "u_diffuse": { "index": 0 },
                    "u_shininess": 256,
                    "u_specular": [0.2, 0.2, 0.2, 1]
                }
            }
        }
    });
    let mut count = 0;
    let ret_khr = ForEach::material_value(&mat_khr, |val, name| {
        assert!(name.starts_with("u_"));
        count += 1;
        if name == "u_specular" {
            Some(val.clone())
        } else {
            None
        }
    });
    assert_eq!(count, 3);
    assert_eq!(ret_khr, Some(mat_khr["extensions"]["KHR_techniques_webgl"]["values"]["u_specular"].clone()));

    let mat_legacy = json!({
        "name": "Texture",
        "values": {
            "diffuse": { "index": 0 },
            "shininess": 256,
            "specular": [0.2, 0.2, 0.2, 1]
        }
    });
    let mut leg_count = 0;
    let ret_leg = ForEach::material_value(&mat_legacy, |val, name| {
        leg_count += 1;
        if name == "specular" {
            Some(val.clone())
        } else {
            None
        }
    });
    assert_eq!(leg_count, 3);
    assert_eq!(ret_leg, Some(mat_legacy["values"]["specular"].clone()));
}

#[test]
fn test_loops_over_meshes_primitives_attributes_targets() {
    let gltf = json!({
        "meshes": [
            {
                "primitives": [
                    {
                        "attributes": { "POSITION": 0, "NORMAL": 2 },
                        "targets": [
                            { "POSITION": 0, "NORMAL": 2 },
                            { "POSITION": 1, "NORMAL": 3 }
                        ]
                    },
                    {
                        "attributes": { "POSITION": 1, "NORMAL": 3 }
                    }
                ]
            }
        ]
    });

    let mesh = &gltf["meshes"][0];
    let prim_ret = ForEach::mesh_primitive(mesh, |prim, idx| {
        assert_eq!(prim["attributes"]["POSITION"], idx);
        if idx == 1 {
            Some(prim.clone())
        } else {
            None
        }
    });
    assert_eq!(prim_ret, Some(mesh["primitives"][1].clone()));

    let attr_ret = ForEach::mesh_primitive_attribute(&mesh["primitives"][0], |accessor_id, semantic| {
        if semantic == "NORMAL" {
            Some(accessor_id.clone())
        } else {
            None
        }
    });
    assert_eq!(attr_ret, Some(json!(2)));

    let target_ret = ForEach::mesh_primitive_target(&mesh["primitives"][0], |target, idx| {
        assert_eq!(target["POSITION"], idx);
        if idx == 1 {
            Some(target.clone())
        } else {
            None
        }
    });
    assert_eq!(target_ret, Some(mesh["primitives"][0]["targets"][1].clone()));

    let target_attr_ret = ForEach::mesh_primitive_target_attribute(&mesh["primitives"][0]["targets"][0], |accessor_id, semantic| {
        if semantic == "NORMAL" {
            Some(accessor_id.clone())
        } else {
            None
        }
    });
    assert_eq!(target_attr_ret, Some(json!(2)));
}

#[test]
fn test_loops_over_nodes_and_tree_and_scenes() {
    let gltf = json!({
        "nodes": [
            { "matrix": [1,0,0,0,0,1,0,0,0,0,1,0,0,0,0,1], "children": [4, 5] },
            { "matrix": [1,0,0,0,0,1,0,0,0,0,1,0,1,0,0,1] },
            { "matrix": [1,0,0,0,0,1,0,0,0,0,1,0,2,0,0,1] },
            { "matrix": [1,0,0,0,0,1,0,0,0,0,1,0,3,0,0,1] },
            { "matrix": [1,0,0,0,0,1,0,0,0,0,1,0,4,0,0,1] },
            { "matrix": [1,0,0,0,0,1,0,0,0,0,1,0,5,0,0,1] }
        ],
        "scenes": [
            { "nodes": [0, 1, 2] },
            { "nodes": [3] }
        ]
    });

    let mut nodes_count = 0;
    let node_ret = ForEach::node(&gltf, |node, id| {
        if let GltfId::Index(idx) = id {
            assert_eq!(node["matrix"][12], idx);
            nodes_count += 1;
            if nodes_count == 6 {
                return Some(node.clone());
            }
        }
        None
    });
    assert_eq!(nodes_count, 6);
    assert_eq!(node_ret, Some(gltf["nodes"][5].clone()));

    // nodeInTree
    let mut tree_count = 0;
    let scene_nodes = gltf["scenes"][0]["nodes"].as_array().unwrap();
    let tree_ret = ForEach::node_in_tree(&gltf, scene_nodes, &mut |node, _id| {
        tree_count += 1;
        if tree_count == 5 {
            Some(node.clone())
        } else {
            None
        }
    });
    assert_eq!(tree_count, 5);
    assert_eq!(tree_ret, Some(gltf["nodes"][2].clone()));

    // nodeInScene
    let mut scene0_count = 0;
    ForEach::node_in_scene(&gltf, &gltf["scenes"][0], |_node, _id| {
        scene0_count += 1;
        None::<()>
    });
    assert_eq!(scene0_count, 5);

    let mut scene1_count = 0;
    let scene1_ret = ForEach::node_in_scene(&gltf, &gltf["scenes"][1], |node, _id| {
        scene1_count += 1;
        Some(node.clone())
    });
    assert_eq!(scene1_count, 1);
    assert_eq!(scene1_ret, Some(gltf["nodes"][3].clone()));
}

#[test]
fn test_loops_over_samplers_scenes_shaders_programs_techniques_skins_textures() {
    let gltf_khr = json!({
        "extensionsUsed": ["KHR_techniques_webgl"],
        "extensions": {
            "KHR_techniques_webgl": {
                "shaders": [
                    { "name": "BoxFS", "uri": "BoxFS.glsl" },
                    { "name": "BoxVS", "uri": "BoxVS.glsl" }
                ],
                "programs": [
                    { "fragmentShader": 0, "vertexShader": 1 },
                    { "fragmentShader": 2, "vertexShader": 3 }
                ],
                "techniques": [
                    { "name": "tech0", "program": 0 },
                    { "name": "tech1", "program": 1 }
                ]
            }
        },
        "samplers": [{ "magFilter": 9728 }, { "magFilter": 9729 }],
        "scenes": [{ "nodes": [0] }, { "nodes": [1] }],
        "skins": [{ "joints": [0] }, { "joints": [1] }],
        "textures": [{ "source": 0 }, { "source": 1 }]
    });

    assert!(uses_extension(&gltf_khr, "KHR_techniques_webgl"));
    assert!(!uses_extension(&gltf_khr, "NON_EXISTENT"));

    let mut shader_count = 0;
    let shader_ret = ForEach::shader(&gltf_khr, |s, _| {
        shader_count += 1;
        if shader_count == 2 {
            Some(s.clone())
        } else {
            None
        }
    });
    assert_eq!(shader_count, 2);
    assert_eq!(shader_ret, Some(gltf_khr["extensions"]["KHR_techniques_webgl"]["shaders"][1].clone()));

    let mut prog_count = 0;
    let prog_ret = ForEach::program(&gltf_khr, |p, _| {
        prog_count += 1;
        if prog_count == 2 {
            Some(p.clone())
        } else {
            None
        }
    });
    assert_eq!(prog_count, 2);
    assert_eq!(prog_ret, Some(gltf_khr["extensions"]["KHR_techniques_webgl"]["programs"][1].clone()));

    let mut tech_count = 0;
    let tech_ret = ForEach::technique(&gltf_khr, |t, _| {
        tech_count += 1;
        if tech_count == 2 {
            Some(t.clone())
        } else {
            None
        }
    });
    assert_eq!(tech_count, 2);
    assert_eq!(tech_ret, Some(gltf_khr["extensions"]["KHR_techniques_webgl"]["techniques"][1].clone()));

    let samp_ret = ForEach::sampler(&gltf_khr, |s, id| {
        if id == GltfId::Index(1) {
            Some(s.clone())
        } else {
            None
        }
    });
    assert_eq!(samp_ret, Some(gltf_khr["samplers"][1].clone()));

    let scene_ret = ForEach::scene(&gltf_khr, |sc, id| {
        if id == GltfId::Index(1) {
            Some(sc.clone())
        } else {
            None
        }
    });
    assert_eq!(scene_ret, Some(gltf_khr["scenes"][1].clone()));

    let skin_ret = ForEach::skin(&gltf_khr, |sk, id| {
        if id == GltfId::Index(1) {
            Some(sk.clone())
        } else {
            None
        }
    });
    assert_eq!(skin_ret, Some(gltf_khr["skins"][1].clone()));

    let mut joint_count = 0;
    ForEach::skin_joint(&gltf_khr["skins"][0], |_| {
        joint_count += 1;
        None::<()>
    });
    assert_eq!(joint_count, 1);

    let tex_ret = ForEach::texture(&gltf_khr, |tx, id| {
        if id == GltfId::Index(1) {
            Some(tx.clone())
        } else {
            None
        }
    });
    assert_eq!(tex_ret, Some(gltf_khr["textures"][1].clone()));
}

#[test]
fn test_loops_over_technique_details() {
    let technique = json!({
        "attributes": {
            "a_normal": { "semantic": "NORMAL" },
            "a_position": { "semantic": "POSITION" }
        },
        "uniforms": {
            "u_diffuse": { "type": 35678 },
            "u_modelViewMatrix": { "type": 35676 }
        },
        "parameters": {
            "diffuse": { "type": 35678 },
            "modelViewMatrix": { "type": 35676 }
        }
    });

    let mut attr_count = 0;
    ForEach::technique_attribute(&technique, |attr, name| {
        assert!(name.starts_with("a_"));
        assert!(attr["semantic"].is_string());
        attr_count += 1;
        None::<()>
    });
    assert_eq!(attr_count, 2);

    let mut unif_count = 0;
    ForEach::technique_uniform(&technique, |unif, name| {
        assert!(name.starts_with("u_"));
        assert!(unif["type"].is_number());
        unif_count += 1;
        None::<()>
    });
    assert_eq!(unif_count, 2);

    let mut param_count = 0;
    ForEach::technique_parameter(&technique, |param, name| {
        assert!(param["type"].is_number());
        assert!(!name.is_empty());
        param_count += 1;
        None::<()>
    });
    assert_eq!(param_count, 2);
}
