//! glTF pipeline helper functions matching CesiumJS `packages/gltf/Source` (Apache-2.0).

use serde_json::{json, Value};
use std::collections::HashSet;

/// WebGL constants used in glTF 2.0 specifications.
pub const WEBGL_TRIANGLES: u32 = 4;
pub const WEBGL_ARRAY_BUFFER: u32 = 34962;
pub const WEBGL_ELEMENT_ARRAY_BUFFER: u32 = 34963;
pub const WEBGL_REPEAT: u32 = 10497;

/// Returns the number of components for a glTF accessor type string.
pub fn number_of_components_for_type(type_name: &str) -> usize {
    match type_name {
        "SCALAR" => 1,
        "VEC2" => 2,
        "VEC3" => 3,
        "VEC4" | "MAT2" => 4,
        "MAT3" => 9,
        "MAT4" => 16,
        _ => 1,
    }
}

/// Returns the byte size of a glTF accessor componentType.
pub fn component_type_size_in_bytes(component_type: u32) -> usize {
    match component_type {
        5120 | 5121 => 1, // BYTE, UNSIGNED_BYTE
        5122 | 5123 => 2, // SHORT, UNSIGNED_SHORT
        5124..=5126 => 4, // INT, UNSIGNED_INT, FLOAT
        5130 => 8,        // DOUBLE
        _ => 1,
    }
}

/// Returns the byte stride of the provided accessor.
pub fn get_accessor_byte_stride(gltf: &Value, accessor: &Value) -> usize {
    if let Some(buffer_view_id) = accessor.get("bufferView").and_then(|v| v.as_u64()) {
        if let Some(buffer_views) = gltf.get("bufferViews").and_then(|v| v.as_array()) {
            if let Some(bv) = buffer_views.get(buffer_view_id as usize) {
                if let Some(stride) = bv.get("byteStride").and_then(|v| v.as_u64()) {
                    if stride > 0 {
                        return stride as usize;
                    }
                }
            }
        }
    }

    let comp_type = accessor
        .get("componentType")
        .and_then(|v| v.as_u64())
        .unwrap_or(5126) as u32;
    let type_str = accessor
        .get("type")
        .and_then(|v| v.as_str())
        .unwrap_or("SCALAR");

    component_type_size_in_bytes(comp_type) * number_of_components_for_type(type_str)
}

/// Adds an element to an array in a `Value` and returns the element's index.
pub fn add_to_array(array: &mut Value, element: Value, check_duplicates: bool) -> usize {
    if !array.is_array() {
        *array = Value::Array(Vec::new());
    }
    let arr = array.as_array_mut().unwrap();

    if check_duplicates {
        if let Some(pos) = arr.iter().position(|item| *item == element) {
            return pos;
        }
    }

    arr.push(element);
    arr.len() - 1
}

/// Adds a buffer to glTF and returns the ID of the newly added bufferView.
pub fn add_buffer(gltf: &mut Value, buffer: &[u8]) -> usize {
    if gltf.get("buffers").is_none() {
        gltf["buffers"] = Value::Array(Vec::new());
    }
    if gltf.get("bufferViews").is_none() {
        gltf["bufferViews"] = Value::Array(Vec::new());
    }

    let byte_length = buffer.len();
    let source_json = Value::Array(buffer.iter().map(|&b| json!(b)).collect());

    let new_buffer = json!({
        "byteLength": byte_length,
        "extras": {
            "_pipeline": {
                "source": source_json
            }
        }
    });

    let buffer_id = add_to_array(&mut gltf["buffers"], new_buffer, false);

    let buffer_view = json!({
        "buffer": buffer_id,
        "byteOffset": 0,
        "byteLength": byte_length
    });

    add_to_array(&mut gltf["bufferViews"], buffer_view, false)
}

fn add_texture_defaults(texture: &mut Value) {
    if texture.is_object() && texture.get("texCoord").is_none() {
        texture["texCoord"] = json!(0);
    }
}

/// Adds default glTF values if they don't exist.
pub fn add_defaults(mut gltf: Value) -> Value {
    // 1. accessors: bufferView byteOffset default 0
    if let Some(accessors) = gltf.get_mut("accessors").and_then(|v| v.as_array_mut()) {
        for acc in accessors {
            if acc.get("bufferView").is_some() && acc.get("byteOffset").is_none() {
                acc["byteOffset"] = json!(0);
            }
        }
    }

    // 2. bufferViews: buffer byteOffset default 0
    if let Some(bvs) = gltf.get_mut("bufferViews").and_then(|v| v.as_array_mut()) {
        for bv in bvs {
            if bv.get("buffer").is_some() && bv.get("byteOffset").is_none() {
                bv["byteOffset"] = json!(0);
            }
        }
    }

    // 3. Collect vertex attribute and index accessor IDs
    let mut vertex_accessor_ids = HashSet::new();
    let mut index_accessor_ids = HashSet::new();

    if let Some(meshes) = gltf.get("meshes").and_then(|v| v.as_array()) {
        for mesh in meshes {
            if let Some(primitives) = mesh.get("primitives").and_then(|v| v.as_array()) {
                for prim in primitives {
                    if let Some(attrs) = prim.get("attributes").and_then(|v| v.as_object()) {
                        for (_, acc_id_val) in attrs {
                            if let Some(id) = acc_id_val.as_u64() {
                                vertex_accessor_ids.insert(id as usize);
                            }
                        }
                    }
                    if let Some(targets) = prim.get("targets").and_then(|v| v.as_array()) {
                        for target in targets {
                            if let Some(t_attrs) = target.as_object() {
                                for (_, acc_id_val) in t_attrs {
                                    if let Some(id) = acc_id_val.as_u64() {
                                        vertex_accessor_ids.insert(id as usize);
                                    }
                                }
                            }
                        }
                    }
                    if let Some(indices) = prim.get("indices").and_then(|v| v.as_u64()) {
                        index_accessor_ids.insert(indices as usize);
                    }
                }
            }
        }
    }

    // 4. Update meshes primitives: mode and material
    if let Some(meshes) = gltf.get_mut("meshes").and_then(|v| v.as_array_mut()) {
        for mesh in meshes {
            if let Some(primitives) = mesh.get_mut("primitives").and_then(|v| v.as_array_mut()) {
                for prim in primitives {
                    if prim.get("mode").is_none() {
                        prim["mode"] = json!(WEBGL_TRIANGLES);
                    }
                    if prim.get("material").is_none() {
                        // Will attach material id 0 if materials is empty or create default
                        prim["material"] = json!(0);
                    }
                }
            }
        }
    }

    // If any primitive needed material, ensure default material exists
    let needs_default_material = gltf
        .get("meshes")
        .and_then(|v| v.as_array())
        .map(|meshes| {
            meshes.iter().any(|m| {
                m.get("primitives")
                    .and_then(|p| p.as_array())
                    .map(|prims| prims.iter().any(|pr| pr.get("material").is_some()))
                    .unwrap_or(false)
            })
        })
        .unwrap_or(false);

    if needs_default_material && gltf.get("materials").is_none() {
        gltf["materials"] = json!([{
            "name": "default"
        }]);
    }

    // 5. Update vertex attribute accessors & their bufferViews
    let gltf_clone = gltf.clone();
    for &acc_id in &vertex_accessor_ids {
        if let Some(accessors) = gltf.get_mut("accessors").and_then(|v| v.as_array_mut()) {
            if let Some(acc) = accessors.get_mut(acc_id) {
                if acc.get("normalized").is_none() {
                    acc["normalized"] = json!(false);
                }
                if let Some(bv_id) = acc.get("bufferView").and_then(|v| v.as_u64()) {
                    let stride = get_accessor_byte_stride(&gltf_clone, acc);
                    if let Some(bvs) = gltf.get_mut("bufferViews").and_then(|v| v.as_array_mut()) {
                        if let Some(bv) = bvs.get_mut(bv_id as usize) {
                            bv["byteStride"] = json!(stride);
                            bv["target"] = json!(WEBGL_ARRAY_BUFFER);
                        }
                    }
                }
            }
        }
    }

    // 6. Update index accessors & their bufferViews
    for &acc_id in &index_accessor_ids {
        if let Some(accessors) = gltf.get("accessors").and_then(|v| v.as_array()) {
            if let Some(acc) = accessors.get(acc_id) {
                if let Some(bv_id) = acc.get("bufferView").and_then(|v| v.as_u64()) {
                    if let Some(bvs) = gltf.get_mut("bufferViews").and_then(|v| v.as_array_mut()) {
                        if let Some(bv) = bvs.get_mut(bv_id as usize) {
                            bv["target"] = json!(WEBGL_ELEMENT_ARRAY_BUFFER);
                        }
                    }
                }
            }
        }
    }

    // 7. Update materials defaults
    if let Some(materials) = gltf.get_mut("materials").and_then(|v| v.as_array_mut()) {
        for mat in materials {
            let has_materials_common = mat
                .get("extensions")
                .and_then(|e| e.get("KHR_materials_common"))
                .is_some();

            if has_materials_common {
                let ext = mat.get_mut("extensions").unwrap();
                let mc = ext.get_mut("KHR_materials_common").unwrap();

                let technique = mc
                    .get("technique")
                    .and_then(|t| t.as_str())
                    .unwrap_or("PHONG")
                    .to_string();

                if mc.get("values").is_none() {
                    mc["values"] = json!({});
                }
                let values = mc.get_mut("values").unwrap();

                if values.get("ambient").is_none() {
                    values["ambient"] = json!([0.0, 0.0, 0.0, 1.0]);
                }
                if values.get("emission").is_none() {
                    values["emission"] = json!([0.0, 0.0, 0.0, 1.0]);
                }
                if values.get("transparency").is_none() {
                    values["transparency"] = json!(1.0);
                }

                if technique != "CONSTANT" {
                    if values.get("diffuse").is_none() {
                        values["diffuse"] = json!([0.0, 0.0, 0.0, 1.0]);
                    }
                    if technique != "LAMBERT" {
                        if values.get("specular").is_none() {
                            values["specular"] = json!([0.0, 0.0, 0.0, 1.0]);
                        }
                        if values.get("shininess").is_none() {
                            values["shininess"] = json!(0.0);
                        }
                    }
                }

                if mc.get("transparent").is_none() {
                    mc["transparent"] = json!(false);
                }
                if mc.get("doubleSided").is_none() {
                    mc["doubleSided"] = json!(false);
                }
                continue;
            }

            if mat.get("emissiveFactor").is_none() {
                mat["emissiveFactor"] = json!([0.0, 0.0, 0.0]);
            }
            if mat.get("alphaMode").is_none() {
                mat["alphaMode"] = json!("OPAQUE");
            }
            if mat.get("doubleSided").is_none() {
                mat["doubleSided"] = json!(false);
            }
            if mat.get("alphaMode").and_then(|v| v.as_str()) == Some("MASK")
                && mat.get("alphaCutoff").is_none()
            {
                mat["alphaCutoff"] = json!(0.5);
            }

            if let Some(ext) = mat.get_mut("extensions").and_then(|v| v.as_object_mut()) {
                if let Some(tech) = ext.get_mut("KHR_techniques_webgl") {
                    if let Some(vals) = tech.get_mut("values").and_then(|v| v.as_object_mut()) {
                        for (_, val) in vals.iter_mut() {
                            if val.is_object() && val.get("index").is_some() {
                                add_texture_defaults(val);
                            }
                        }
                    }
                }
                if let Some(spec_gloss) = ext.get_mut("KHR_materials_pbrSpecularGlossiness") {
                    if spec_gloss.get("diffuseFactor").is_none() {
                        spec_gloss["diffuseFactor"] = json!([1.0, 1.0, 1.0, 1.0]);
                    }
                    if spec_gloss.get("specularFactor").is_none() {
                        spec_gloss["specularFactor"] = json!([1.0, 1.0, 1.0]);
                    }
                    if spec_gloss.get("glossinessFactor").is_none() {
                        spec_gloss["glossinessFactor"] = json!(1.0);
                    }
                    if let Some(tex) = spec_gloss.get_mut("diffuseTexture") {
                        add_texture_defaults(tex);
                    }
                    if let Some(tex) = spec_gloss.get_mut("specularGlossinessTexture") {
                        add_texture_defaults(tex);
                    }
                }
            }

            if let Some(tex) = mat.get_mut("emissiveTexture") {
                add_texture_defaults(tex);
            }
            if let Some(tex) = mat.get_mut("normalTexture") {
                add_texture_defaults(tex);
            }
            if let Some(tex) = mat.get_mut("occlusionTexture") {
                add_texture_defaults(tex);
            }

            if let Some(pbr) = mat.get_mut("pbrMetallicRoughness") {
                if pbr.get("baseColorFactor").is_none() {
                    pbr["baseColorFactor"] = json!([1.0, 1.0, 1.0, 1.0]);
                }
                if pbr.get("metallicFactor").is_none() {
                    pbr["metallicFactor"] = json!(1.0);
                }
                if pbr.get("roughnessFactor").is_none() {
                    pbr["roughnessFactor"] = json!(1.0);
                }
                if let Some(tex) = pbr.get_mut("baseColorTexture") {
                    add_texture_defaults(tex);
                }
                if let Some(tex) = pbr.get_mut("metallicRoughnessTexture") {
                    add_texture_defaults(tex);
                }
            }
        }
    }

    // 8. Update animations samplers
    if let Some(anims) = gltf.get_mut("animations").and_then(|v| v.as_array_mut()) {
        for anim in anims {
            if let Some(samplers) = anim.get_mut("samplers").and_then(|v| v.as_array_mut()) {
                for s in samplers {
                    if s.get("interpolation").is_none() {
                        s["interpolation"] = json!("LINEAR");
                    }
                }
            }
        }
    }

    // 9. Collect animated node IDs
    let mut animated_nodes = HashSet::new();
    if let Some(anims) = gltf.get("animations").and_then(|v| v.as_array()) {
        for anim in anims {
            if let Some(channels) = anim.get("channels").and_then(|v| v.as_array()) {
                for ch in channels {
                    if let Some(target) = ch.get("target") {
                        let path = target.get("path").and_then(|p| p.as_str()).unwrap_or("");
                        if path == "translation" || path == "rotation" || path == "scale" {
                            if let Some(node_id) = target.get("node").and_then(|n| n.as_u64()) {
                                animated_nodes.insert(node_id as usize);
                            }
                        }
                    }
                }
            }
        }
    }

    // Update nodes
    if let Some(nodes) = gltf.get_mut("nodes").and_then(|v| v.as_array_mut()) {
        for (i, node) in nodes.iter_mut().enumerate() {
            let animated = animated_nodes.contains(&i);
            let has_trs = node.get("translation").is_some()
                || node.get("rotation").is_some()
                || node.get("scale").is_some();

            if animated || has_trs {
                if node.get("translation").is_none() {
                    node["translation"] = json!([0.0, 0.0, 0.0]);
                }
                if node.get("rotation").is_none() {
                    node["rotation"] = json!([0.0, 0.0, 0.0, 1.0]);
                }
                if node.get("scale").is_none() {
                    node["scale"] = json!([1.0, 1.0, 1.0]);
                }
            } else if node.get("matrix").is_none() {
                node["matrix"] = json!([
                    1.0, 0.0, 0.0, 0.0,
                    0.0, 1.0, 0.0, 0.0,
                    0.0, 0.0, 1.0, 0.0,
                    0.0, 0.0, 0.0, 1.0
                ]);
            }
        }
    }

    // 10. Update samplers
    if let Some(samplers) = gltf.get_mut("samplers").and_then(|v| v.as_array_mut()) {
        for s in samplers {
            if s.get("wrapS").is_none() {
                s["wrapS"] = json!(WEBGL_REPEAT);
            }
            if s.get("wrapT").is_none() {
                s["wrapT"] = json!(WEBGL_REPEAT);
            }
        }
    }

    // 11. Update scene
    if gltf.get("scenes").is_some() && gltf.get("scene").is_none() {
        gltf["scene"] = json!(0);
    }

    gltf
}

/// Adds an extension to `gltf.extensionsUsed` if it does not already exist.
/// Initializes `extensionsUsed` if it is not defined.
pub fn add_extensions_used(gltf: &mut Value, extension: &str) {
    if !gltf.is_object() {
        *gltf = json!({});
    }
    if gltf.get("extensionsUsed").is_none() {
        gltf["extensionsUsed"] = Value::Array(Vec::new());
    }
    add_to_array(&mut gltf["extensionsUsed"], json!(extension), true);
}

/// Adds an extension to `gltf.extensionsRequired` if it does not already exist,
/// and also adds it to `gltf.extensionsUsed`.
/// Initializes `extensionsRequired` and `extensionsUsed` if they are not defined.
pub fn add_extensions_required(gltf: &mut Value, extension: &str) {
    if !gltf.is_object() {
        *gltf = json!({});
    }
    if gltf.get("extensionsRequired").is_none() {
        gltf["extensionsRequired"] = Value::Array(Vec::new());
    }
    add_to_array(&mut gltf["extensionsRequired"], json!(extension), true);
    add_extensions_used(gltf, extension);
}

fn add_extras_to_object(object: &mut Value) {
    if !object.is_object() {
        return;
    }
    if object.get("extras").is_none() {
        object["extras"] = json!({});
    }
    if let Some(extras) = object.get_mut("extras").and_then(|e| e.as_object_mut()) {
        if !extras.contains_key("_pipeline") {
            extras.insert("_pipeline".to_string(), json!({}));
        }
    }
}

fn add_extras_to_collection(collection: &mut Value) {
    if let Some(arr) = collection.as_array_mut() {
        for item in arr {
            add_extras_to_object(item);
        }
    } else if let Some(map) = collection.as_object_mut() {
        for (_, item) in map {
            add_extras_to_object(item);
        }
    }
}

/// Adds `extras._pipeline` to each object that can have extras in the glTF asset.
/// This stage runs before `updateVersion` and handles both glTF 1.0 and glTF 2.0 assets.
pub fn add_pipeline_extras(mut gltf: Value) -> Value {
    if let Some(buffers) = gltf.get_mut("buffers") {
        add_extras_to_collection(buffers);
    }
    if let Some(images) = gltf.get_mut("images") {
        add_extras_to_collection(images);
    }
    if let Some(shaders) = gltf.get_mut("shaders") {
        add_extras_to_collection(shaders);
    }
    if let Some(ext) = gltf.get_mut("extensions").and_then(|e| e.as_object_mut()) {
        if let Some(tech) = ext.get_mut("KHR_techniques_webgl").and_then(|t| t.as_object_mut()) {
            if let Some(shaders) = tech.get_mut("shaders") {
                add_extras_to_collection(shaders);
            }
        }
    }

    add_extras_to_object(&mut gltf);

    gltf
}


