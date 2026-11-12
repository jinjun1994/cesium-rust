//! glTF pipeline helper functions matching CesiumJS `packages/gltf/Source` (Apache-2.0).

use glam::{DMat4, DQuat, DVec3, Mat3, Quat, Vec3};
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

/// Converts the `boundingVolume` of a glTF node using the `3DTILES_tileset` extension
/// to the box or sphere bounding volume format of a 3D Tiles tileset JSON.
/// Only box and sphere shapes are supported.
pub fn construct_bounding_volume_json(
    bounding_volume_json: &Value,
    shapes: &[Value],
) -> Result<Value, String> {
    let shape_idx = bounding_volume_json
        .get("shape")
        .and_then(|v| v.as_u64())
        .ok_or_else(|| "boundingVolume.shape is missing".to_string())? as usize;

    let shape = shapes
        .get(shape_idx)
        .ok_or_else(|| format!("Shape index {shape_idx} out of bounds"))?;

    let translation = bounding_volume_json
        .get("translation")
        .and_then(|v| v.as_array())
        .map(|arr| {
            [
                arr.first().and_then(|v| v.as_f64()).unwrap_or(0.0),
                arr.get(1).and_then(|v| v.as_f64()).unwrap_or(0.0),
                arr.get(2).and_then(|v| v.as_f64()).unwrap_or(0.0),
            ]
        })
        .unwrap_or([0.0, 0.0, 0.0]);

    let scale = bounding_volume_json
        .get("scale")
        .and_then(|v| v.as_array())
        .map(|arr| {
            [
                arr.first().and_then(|v| v.as_f64()).unwrap_or(1.0),
                arr.get(1).and_then(|v| v.as_f64()).unwrap_or(1.0),
                arr.get(2).and_then(|v| v.as_f64()).unwrap_or(1.0),
            ]
        });

    let rotation = bounding_volume_json
        .get("rotation")
        .and_then(|v| v.as_array())
        .map(|arr| {
            [
                arr.first().and_then(|v| v.as_f64()).unwrap_or(0.0),
                arr.get(1).and_then(|v| v.as_f64()).unwrap_or(0.0),
                arr.get(2).and_then(|v| v.as_f64()).unwrap_or(0.0),
                arr.get(3).and_then(|v| v.as_f64()).unwrap_or(1.0),
            ]
        });

    if let Some(box_obj) = shape.get("box") {
        let size = box_obj
            .get("size")
            .and_then(|v| v.as_array())
            .map(|arr| {
                [
                    arr.first().and_then(|v| v.as_f64()).unwrap_or(1.0),
                    arr.get(1).and_then(|v| v.as_f64()).unwrap_or(1.0),
                    arr.get(2).and_then(|v| v.as_f64()).unwrap_or(1.0),
                ]
            })
            .unwrap_or([1.0, 1.0, 1.0]);

        // Half axes initialized from half size: size * 0.5
        let mut half_axes = Mat3::from_diagonal(Vec3::new(
            (size[0] * 0.5) as f32,
            (size[1] * 0.5) as f32,
            (size[2] * 0.5) as f32,
        ));

        if let Some(s) = scale {
            // multiplyByScale: each column scaled by s[i]
            half_axes = Mat3::from_cols(
                half_axes.x_axis * s[0] as f32,
                half_axes.y_axis * s[1] as f32,
                half_axes.z_axis * s[2] as f32,
            );
        }

        if let Some(r) = rotation {
            let quat = Quat::from_xyzw(r[0] as f32, r[1] as f32, r[2] as f32, r[3] as f32);
            let rot_mat = Mat3::from_quat(quat);
            half_axes = rot_mat * half_axes;
        }

        let box_data = vec![
            translation[0],
            translation[1],
            translation[2],
            half_axes.x_axis.x as f64,
            half_axes.x_axis.y as f64,
            half_axes.x_axis.z as f64,
            half_axes.y_axis.x as f64,
            half_axes.y_axis.y as f64,
            half_axes.y_axis.z as f64,
            half_axes.z_axis.x as f64,
            half_axes.z_axis.y as f64,
            half_axes.z_axis.z as f64,
        ];

        return Ok(json!({ "box": box_data }));
    }

    if let Some(sphere_obj) = shape.get("sphere") {
        let mut radius = sphere_obj
            .get("radius")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.5);

        if let Some(s) = scale {
            let max_s = s[0].abs().max(s[1].abs()).max(s[2].abs());
            radius *= max_s;
        }

        let sphere_data = vec![translation[0], translation[1], translation[2], radius];
        return Ok(json!({ "sphere": sphere_data }));
    }

    Err("Only box and sphere shapes are supported for the bounding volume.".to_string())
}

/// Result of `find_accessor_min_max`.
#[derive(Clone, Debug, PartialEq)]
pub struct AccessorMinMax {
    pub min: Vec<f64>,
    pub max: Vec<f64>,
}

pub type ComponentReader = Box<dyn Fn(&[u8], usize, usize, &mut [f64]) + Send + Sync>;

/// Returns a function to read and convert data from a byte slice into an array.
#[allow(clippy::needless_range_loop)]
pub fn get_component_reader(component_type: u32) -> Result<ComponentReader, String> {
    let byte_length = component_type_size_in_bytes(component_type);
    match component_type {
        5120 => Ok(Box::new(move |bytes, offset, count, result| {
            for i in 0..count {
                let idx = offset + i * byte_length;
                result[i] = (bytes[idx] as i8) as f64;
            }
        })),
        5121 => Ok(Box::new(move |bytes, offset, count, result| {
            for i in 0..count {
                let idx = offset + i * byte_length;
                result[i] = bytes[idx] as f64;
            }
        })),
        5122 => Ok(Box::new(move |bytes, offset, count, result| {
            for i in 0..count {
                let idx = offset + i * byte_length;
                let v = i16::from_le_bytes([bytes[idx], bytes[idx + 1]]);
                result[i] = v as f64;
            }
        })),
        5123 => Ok(Box::new(move |bytes, offset, count, result| {
            for i in 0..count {
                let idx = offset + i * byte_length;
                let v = u16::from_le_bytes([bytes[idx], bytes[idx + 1]]);
                result[i] = v as f64;
            }
        })),
        5124 => Ok(Box::new(move |bytes, offset, count, result| {
            for i in 0..count {
                let idx = offset + i * byte_length;
                let v = i32::from_le_bytes([
                    bytes[idx],
                    bytes[idx + 1],
                    bytes[idx + 2],
                    bytes[idx + 3],
                ]);
                result[i] = v as f64;
            }
        })),
        5125 => Ok(Box::new(move |bytes, offset, count, result| {
            for i in 0..count {
                let idx = offset + i * byte_length;
                let v = u32::from_le_bytes([
                    bytes[idx],
                    bytes[idx + 1],
                    bytes[idx + 2],
                    bytes[idx + 3],
                ]);
                result[i] = v as f64;
            }
        })),
        5126 => Ok(Box::new(move |bytes, offset, count, result| {
            for i in 0..count {
                let idx = offset + i * byte_length;
                let v = f32::from_le_bytes([
                    bytes[idx],
                    bytes[idx + 1],
                    bytes[idx + 2],
                    bytes[idx + 3],
                ]);
                result[i] = v as f64;
            }
        })),
        5130 => Ok(Box::new(move |bytes, offset, count, result| {
            for i in 0..count {
                let idx = offset + i * byte_length;
                let v = f64::from_le_bytes([
                    bytes[idx],
                    bytes[idx + 1],
                    bytes[idx + 2],
                    bytes[idx + 3],
                    bytes[idx + 4],
                    bytes[idx + 5],
                    bytes[idx + 6],
                    bytes[idx + 7],
                ]);
                result[i] = v;
            }
        })),
        _ => Err(format!("Unsupported component type: {component_type}")),
    }
}

/// Gets the local transform of a glTF node as a column-major array of 16 numbers.
/// Returns `None` if the node has no `matrix`, `translation`, `rotation`, or `scale`.
pub fn get_node_transform(node_json: &Value) -> Option<Vec<f64>> {
    if let Some(matrix) = node_json.get("matrix").and_then(|v| v.as_array()) {
        return Some(matrix.iter().filter_map(|v| v.as_f64()).collect());
    }

    let translation = node_json.get("translation").and_then(|v| v.as_array());
    let rotation = node_json.get("rotation").and_then(|v| v.as_array());
    let scale = node_json.get("scale").and_then(|v| v.as_array());

    if translation.is_none() && rotation.is_none() && scale.is_none() {
        return None;
    }

    let t = translation.map(|arr| {
        DVec3::new(
            arr.first().and_then(|v| v.as_f64()).unwrap_or(0.0),
            arr.get(1).and_then(|v| v.as_f64()).unwrap_or(0.0),
            arr.get(2).and_then(|v| v.as_f64()).unwrap_or(0.0),
        )
    }).unwrap_or(DVec3::ZERO);

    let r = rotation.map(|arr| {
        DQuat::from_xyzw(
            arr.first().and_then(|v| v.as_f64()).unwrap_or(0.0),
            arr.get(1).and_then(|v| v.as_f64()).unwrap_or(0.0),
            arr.get(2).and_then(|v| v.as_f64()).unwrap_or(0.0),
            arr.get(3).and_then(|v| v.as_f64()).unwrap_or(1.0),
        )
    }).unwrap_or(DQuat::IDENTITY);

    let s = scale.map(|arr| {
        DVec3::new(
            arr.first().and_then(|v| v.as_f64()).unwrap_or(1.0),
            arr.get(1).and_then(|v| v.as_f64()).unwrap_or(1.0),
            arr.get(2).and_then(|v| v.as_f64()).unwrap_or(1.0),
        )
    }).unwrap_or(DVec3::ONE);

    let mat = DMat4::from_scale_rotation_translation(s, r, t);
    Some(mat.to_cols_array().to_vec())
}

pub const WEBGL_DEPTH_TEST: u32 = 2929;
pub const WEBGL_BLEND: u32 = 3042;
pub const WEBGL_CULL_FACE: u32 = 2884;
pub const WEBGL_FUNC_ADD: u32 = 32774;
pub const WEBGL_ZERO: u32 = 0;
pub const WEBGL_ONE: u32 = 1;

const SUPPORTED_BLEND_FACTORS: [u32; 10] = [
    0,     // ZERO
    1,     // ONE
    768,   // SRC_COLOR
    769,   // ONE_MINUS_SRC_COLOR
    770,   // SRC_ALPHA
    771,   // ONE_MINUS_SRC_ALPHA
    772,   // DST_ALPHA
    773,   // ONE_MINUS_DST_ALPHA
    774,   // DST_COLOR
    775,   // ONE_MINUS_DST_COLOR
];

fn is_state_enabled(render_states: &Value, state: u32) -> bool {
    if let Some(enable) = render_states.get("enable").and_then(|v| v.as_array()) {
        enable.iter().any(|item| item.as_u64() == Some(state as u64))
    } else {
        false
    }
}

fn get_supported_blend_factors(val: Option<&Value>, default_factors: &Value) -> Value {
    if let Some(arr) = val.and_then(|v| v.as_array()) {
        if arr.len() == 4 {
            let all_supported = arr.iter().all(|item| {
                if let Some(f) = item.as_u64() {
                    SUPPORTED_BLEND_FACTORS.contains(&(f as u32))
                } else {
                    false
                }
            });
            if all_supported {
                return Value::Array(arr.clone());
            }
        }
    }
    default_factors.clone()
}

/// Moves glTF 1.0 technique render states to glTF 2.0 materials properties and KHR_blend extension.
pub fn move_technique_render_states(mut gltf: Value) -> Value {
    if gltf.get("techniques").is_none() {
        return gltf;
    }

    let default_blend_equation = json!([WEBGL_FUNC_ADD, WEBGL_FUNC_ADD]);
    let default_blend_factors = json!([WEBGL_ONE, WEBGL_ZERO, WEBGL_ONE, WEBGL_ZERO]);

    let mut blending_for_technique: std::collections::HashMap<String, Value> = std::collections::HashMap::new();
    let mut material_properties_for_technique: std::collections::HashMap<String, serde_json::Map<String, Value>> =
        std::collections::HashMap::new();

    // Iterate over techniques
    if let Some(techniques) = gltf.get_mut("techniques") {
        let mut process_technique = |tech_id: String, tech: &mut Value| {
            if let Some(render_states) = tech.get_mut("states") {
                let mut material_props = serde_json::Map::new();

                if is_state_enabled(render_states, WEBGL_BLEND) {
                    material_props.insert("alphaMode".to_string(), json!("BLEND"));

                    if let Some(blend_functions) = render_states.get("functions") {
                        let has_eq = blend_functions.get("blendEquationSeparate").is_some();
                        let has_func = blend_functions.get("blendFuncSeparate").is_some();
                        if has_eq || has_func {
                            let eq = blend_functions
                                .get("blendEquationSeparate")
                                .cloned()
                                .unwrap_or_else(|| default_blend_equation.clone());
                            let factors = get_supported_blend_factors(
                                blend_functions.get("blendFuncSeparate"),
                                &default_blend_factors,
                            );

                            blending_for_technique.insert(
                                tech_id.clone(),
                                json!({
                                    "blendEquation": eq,
                                    "blendFactors": factors
                                }),
                            );
                        }
                    }
                }

                if !is_state_enabled(render_states, WEBGL_CULL_FACE) {
                    material_props.insert("doubleSided".to_string(), json!(true));
                }

                material_properties_for_technique.insert(tech_id, material_props);
            }
            if let Some(tech_obj) = tech.as_object_mut() {
                tech_obj.remove("states");
            }
        };

        if let Some(arr) = techniques.as_array_mut() {
            for (idx, tech) in arr.iter_mut().enumerate() {
                process_technique(idx.to_string(), tech);
            }
        } else if let Some(map) = techniques.as_object_mut() {
            for (tech_id, tech) in map.iter_mut() {
                process_technique(tech_id.clone(), tech);
            }
        }
    }

    if !blending_for_technique.is_empty() {
        add_extensions_used(&mut gltf, "KHR_blend");
    }

    if let Some(materials) = gltf.get_mut("materials").and_then(|v| v.as_array_mut()) {
        for mat in materials {
            let tech_id_opt = mat
                .get("technique")
                .and_then(|t| t.as_str().map(|s| s.to_string()))
                .or_else(|| mat.get("technique").and_then(|t| t.as_u64().map(|u| u.to_string())));

            if let Some(tech_id) = tech_id_opt {
                if let Some(props) = material_properties_for_technique.get(&tech_id) {
                    if let Some(mat_obj) = mat.as_object_mut() {
                        for (k, v) in props {
                            mat_obj.insert(k.clone(), v.clone());
                        }
                    }
                }

                if let Some(blending) = blending_for_technique.get(&tech_id) {
                    if mat.get("extensions").is_none() {
                        mat["extensions"] = json!({});
                    }
                    if let Some(ext) = mat.get_mut("extensions").and_then(|v| v.as_object_mut()) {
                        ext.insert("KHR_blend".to_string(), blending.clone());
                    }
                }
            }
        }
    }

    gltf
}

/// Moves glTF 1.0 material techniques to glTF 2.0 `KHR_techniques_webgl` extension.
pub fn move_techniques_to_extension(mut gltf: Value) -> Value {
    if gltf.get("techniques").is_none() {
        return gltf;
    }

    let gl_extensions = gltf.as_object_mut().and_then(|obj| obj.remove("glExtensionsUsed"));

    let mut extension_programs = json!([]);
    let mut extension_shaders = json!([]);
    let mut extension_techniques = json!([]);

    let mut mapped_uniforms: std::collections::HashMap<String, std::collections::HashMap<String, String>> =
        std::collections::HashMap::new();
    let mut updated_technique_indices: std::collections::HashMap<String, usize> =
        std::collections::HashMap::new();
    let mut seen_programs: std::collections::HashMap<String, usize> =
        std::collections::HashMap::new();

    // Iterate over gltf.techniques
    let techniques_val = gltf.get("techniques").cloned();
    if let Some(techniques) = techniques_val {
        let mut process_technique = |technique_id: String, technique_legacy: &Value| {
            let mut tech_obj = serde_json::Map::new();
            if let Some(name) = technique_legacy.get("name") {
                tech_obj.insert("name".to_string(), name.clone());
            }

            let mut attributes_map = serde_json::Map::new();
            if let Some(attributes) = technique_legacy.get("attributes").and_then(|v| v.as_object()) {
                for (attr_name, param_name_val) in attributes {
                    if let Some(param_name) = param_name_val.as_str() {
                        if let Some(param_legacy) = technique_legacy
                            .get("parameters")
                            .and_then(|p| p.get(param_name))
                        {
                            let mut attr_entry = serde_json::Map::new();
                            if let Some(semantic) = param_legacy.get("semantic") {
                                attr_entry.insert("semantic".to_string(), semantic.clone());
                            }
                            attributes_map.insert(attr_name.clone(), Value::Object(attr_entry));
                        }
                    }
                }
            }
            tech_obj.insert("attributes".to_string(), Value::Object(attributes_map));

            let mut uniforms_map = serde_json::Map::new();
            if let Some(uniforms) = technique_legacy.get("uniforms").and_then(|v| v.as_object()) {
                for (uniform_name, param_name_val) in uniforms {
                    if let Some(param_name) = param_name_val.as_str() {
                        if let Some(param_legacy) = technique_legacy
                            .get("parameters")
                            .and_then(|p| p.get(param_name))
                        {
                            let mut uniform_entry = serde_json::Map::new();
                            for key in &["count", "node", "type", "semantic", "value"] {
                                if let Some(val) = param_legacy.get(*key) {
                                    uniform_entry.insert(key.to_string(), val.clone());
                                }
                            }
                            uniforms_map.insert(uniform_name.clone(), Value::Object(uniform_entry));

                            mapped_uniforms
                                .entry(technique_id.clone())
                                .or_default()
                                .insert(param_name.to_string(), uniform_name.clone());
                        }
                    }
                }
            }
            tech_obj.insert("uniforms".to_string(), Value::Object(uniforms_map));

            // Program processing
            let prog_key_opt = technique_legacy
                .get("program")
                .and_then(|p| p.as_str().map(|s| s.to_string()))
                .or_else(|| technique_legacy.get("program").and_then(|p| p.as_u64().map(|u| u.to_string())));

            if let Some(prog_key) = prog_key_opt {
                if let Some(&cached_idx) = seen_programs.get(&prog_key) {
                    tech_obj.insert("program".to_string(), json!(cached_idx));
                } else if let Some(program_legacy) = gltf
                    .get("programs")
                    .and_then(|pr| pr.get(&prog_key).or_else(|| pr.as_array().and_then(|arr| prog_key.parse::<usize>().ok().and_then(|i| arr.get(i)))))
                {
                    let mut new_program = serde_json::Map::new();
                    if let Some(name) = program_legacy.get("name") {
                        new_program.insert("name".to_string(), name.clone());
                    }
                    if let Some(gl_ext) = &gl_extensions {
                        new_program.insert("glExtensions".to_string(), gl_ext.clone());
                    }

                    // fragment shader
                    if let Some(fs_key) = program_legacy.get("fragmentShader").and_then(|s| s.as_str()) {
                        if let Some(fs) = gltf.get("shaders").and_then(|sh| sh.get(fs_key)) {
                            let fs_idx = add_to_array(&mut extension_shaders, fs.clone(), true);
                            new_program.insert("fragmentShader".to_string(), json!(fs_idx));
                        }
                    }

                    // vertex shader
                    if let Some(vs_key) = program_legacy.get("vertexShader").and_then(|s| s.as_str()) {
                        if let Some(vs) = gltf.get("shaders").and_then(|sh| sh.get(vs_key)) {
                            let vs_idx = add_to_array(&mut extension_shaders, vs.clone(), true);
                            new_program.insert("vertexShader".to_string(), json!(vs_idx));
                        }
                    }

                    let prog_idx = add_to_array(&mut extension_programs, Value::Object(new_program), false);
                    seen_programs.insert(prog_key, prog_idx);
                    tech_obj.insert("program".to_string(), json!(prog_idx));
                }
            }

            let tech_idx = add_to_array(&mut extension_techniques, Value::Object(tech_obj), false);
            updated_technique_indices.insert(technique_id, tech_idx);
        };

        if let Some(map) = techniques.as_object() {
            for (id, tech) in map {
                process_technique(id.clone(), tech);
            }
        } else if let Some(arr) = techniques.as_array() {
            for (idx, tech) in arr.iter().enumerate() {
                process_technique(idx.to_string(), tech);
            }
        }
    }

    if let Some(tech_arr) = extension_techniques.as_array() {
        if !tech_arr.is_empty() {
            if gltf.get("extensions").is_none() {
                gltf["extensions"] = json!({});
            }
            gltf["extensions"]["KHR_techniques_webgl"] = json!({
                "programs": extension_programs,
                "shaders": extension_shaders,
                "techniques": extension_techniques,
            });
            add_extensions_used(&mut gltf, "KHR_techniques_webgl");
            add_extensions_required(&mut gltf, "KHR_techniques_webgl");
        }
    }

    if let Some(materials) = gltf.get_mut("materials").and_then(|v| v.as_array_mut()) {
        for mat in materials {
            let tech_id_opt = mat
                .get("technique")
                .and_then(|t| t.as_str().map(|s| s.to_string()))
                .or_else(|| mat.get("technique").and_then(|t| t.as_u64().map(|u| u.to_string())));

            if let Some(tech_id) = tech_id_opt {
                let mut material_ext = serde_json::Map::new();
                if let Some(&new_tech_idx) = updated_technique_indices.get(&tech_id) {
                    material_ext.insert("technique".to_string(), json!(new_tech_idx));
                }

                if let Some(values) = mat.get("values").and_then(|v| v.as_object()) {
                    let mut ext_values = serde_json::Map::new();
                    if let Some(param_to_uniform) = mapped_uniforms.get(&tech_id) {
                        for (param_name, val) in values {
                            if let Some(uniform_name) = param_to_uniform.get(param_name) {
                                ext_values.insert(uniform_name.clone(), val.clone());
                            }
                        }
                    }
                    if !ext_values.is_empty() {
                        material_ext.insert("values".to_string(), Value::Object(ext_values));
                    }
                }

                if mat.get("extensions").is_none() {
                    mat["extensions"] = json!({});
                }
                if let Some(ext) = mat.get_mut("extensions").and_then(|v| v.as_object_mut()) {
                    ext.insert("KHR_techniques_webgl".to_string(), Value::Object(material_ext));
                }

                if let Some(mat_obj) = mat.as_object_mut() {
                    mat_obj.remove("technique");
                    mat_obj.remove("values");
                }
            }
        }
    }

    if let Some(obj) = gltf.as_object_mut() {
        obj.remove("techniques");
        obj.remove("programs");
        obj.remove("shaders");
    }

    gltf
}

/// Removes an extension from `gltf.extensionsRequired` if it is present.
pub fn remove_extensions_required(gltf: &mut Value, extension: &str) {
    if let Some(extensions_required) = gltf
        .get_mut("extensionsRequired")
        .and_then(|v| v.as_array_mut())
    {
        if let Some(pos) = extensions_required.iter().position(|v| v == extension) {
            extensions_required.remove(pos);
        }
    }
    if let Some(arr) = gltf.get("extensionsRequired").and_then(|v| v.as_array()) {
        if arr.is_empty() {
            if let Some(obj) = gltf.as_object_mut() {
                obj.remove("extensionsRequired");
            }
        }
    }
}

/// Removes an extension from `gltf.extensionsUsed` and `gltf.extensionsRequired` if it is present.
pub fn remove_extensions_used(gltf: &mut Value, extension: &str) {
    if let Some(extensions_used) = gltf.get_mut("extensionsUsed").and_then(|v| v.as_array_mut()) {
        if let Some(pos) = extensions_used.iter().position(|v| v == extension) {
            extensions_used.remove(pos);
        }
    }
    remove_extensions_required(gltf, extension);

    if let Some(arr) = gltf.get("extensionsUsed").and_then(|v| v.as_array()) {
        if arr.is_empty() {
            if let Some(obj) = gltf.as_object_mut() {
                obj.remove("extensionsUsed");
            }
        }
    }
}

fn remove_extension_recursive(value: &mut Value, extension: &str) -> Option<Value> {
    if let Some(arr) = value.as_array_mut() {
        let mut found = None;
        for item in arr {
            if let Some(res) = remove_extension_recursive(item, extension) {
                if found.is_none() {
                    found = Some(res);
                }
            }
        }
        return found;
    }

    if let Some(obj) = value.as_object_mut() {
        let mut top_removed = None;
        if let Some(ext_val) = obj.get_mut("extensions").and_then(|e| e.as_object_mut()) {
            if let Some(removed) = ext_val.remove(extension) {
                top_removed = Some(removed);
            }
        }

        if let Some(ext_val) = obj.get("extensions").and_then(|e| e.as_object()) {
            if ext_val.is_empty() {
                obj.remove("extensions");
            }
        }

        let mut child_removed = None;
        for (_, child) in obj.iter_mut() {
            if let Some(res) = remove_extension_recursive(child, extension) {
                if child_removed.is_none() {
                    child_removed = Some(res);
                }
            }
        }

        return top_removed.or(child_removed);
    }

    None
}

fn update_cesium_rtc_semantics(value: &mut Value) {
    if let Some(techniques) = value.get_mut("techniques") {
        let check_tech = |tech: &mut Value| {
            if let Some(uniforms) = tech.get_mut("uniforms").and_then(|u| u.as_object_mut()) {
                for (_, uniform) in uniforms {
                    if let Some(u_obj) = uniform.as_object_mut() {
                        if u_obj.get("semantic").and_then(|s| s.as_str()) == Some("CESIUM_RTC_MODELVIEW") {
                            u_obj.insert("semantic".to_string(), json!("MODELVIEW"));
                        }
                    }
                }
            }
        };

        if let Some(arr) = techniques.as_array_mut() {
            for tech in arr {
                check_tech(tech);
            }
        } else if let Some(map) = techniques.as_object_mut() {
            for (_, tech) in map {
                check_tech(tech);
            }
        }
    }

    if let Some(ext) = value.get_mut("extensions").and_then(|e| e.as_object_mut()) {
        if let Some(tech_webgl) = ext.get_mut("KHR_techniques_webgl") {
            update_cesium_rtc_semantics(tech_webgl);
        }
    }
}

/// Removes an extension from `gltf.extensions`, `gltf.extensionsUsed`, `gltf.extensionsRequired`,
/// and any other objects in the glTF if it is present. Returns the extension data removed from `gltf.extensions`.
pub fn remove_extension(gltf: &mut Value, extension: &str) -> Option<Value> {
    remove_extensions_used(gltf, extension);

    if extension == "CESIUM_RTC" {
        update_cesium_rtc_semantics(gltf);
    }

    // Capture the top-level gltf.extensions[extension] before recursive removal
    let top_extension_data = gltf
        .get_mut("extensions")
        .and_then(|e| e.as_object_mut())
        .and_then(|m| m.remove(extension));

    // Remove from all nested objects
    remove_extension_recursive(gltf, extension);

    // Clean up top-level gltf.extensions if empty
    if let Some(ext) = gltf.get("extensions").and_then(|e| e.as_object()) {
        if ext.is_empty() {
            if let Some(obj) = gltf.as_object_mut() {
                obj.remove("extensions");
            }
        }
    }

    top_extension_data
}


fn remove_extras_from_object(object: &mut Value) {
    if let Some(obj) = object.as_object_mut() {
        if let Some(extras) = obj.get_mut("extras").and_then(|e| e.as_object_mut()) {
            extras.remove("_pipeline");
        }
        if let Some(extras) = obj.get("extras").and_then(|e| e.as_object()) {
            if extras.is_empty() {
                obj.remove("extras");
            }
        }
    }
}

fn remove_extras_from_collection(collection: &mut Value) {
    if let Some(arr) = collection.as_array_mut() {
        for item in arr {
            remove_extras_from_object(item);
        }
    } else if let Some(map) = collection.as_object_mut() {
        for (_, item) in map {
            remove_extras_from_object(item);
        }
    }
}

/// Iterates through the objects within the glTF and deletes their pipeline extras object.
pub fn remove_pipeline_extras(gltf: &mut Value) {
    if let Some(buffers) = gltf.get_mut("buffers") {
        remove_extras_from_collection(buffers);
    }
    if let Some(images) = gltf.get_mut("images") {
        remove_extras_from_collection(images);
    }
    if let Some(shaders) = gltf.get_mut("shaders") {
        remove_extras_from_collection(shaders);
    }
    if let Some(ext) = gltf.get_mut("extensions").and_then(|e| e.as_object_mut()) {
        if let Some(tech) = ext.get_mut("KHR_techniques_webgl").and_then(|t| t.as_object_mut()) {
            if let Some(shaders) = tech.get_mut("shaders") {
                remove_extras_from_collection(shaders);
            }
        }
    }

    remove_extras_from_object(gltf);
}

/// Converts a binary glTF (GLB) to a glTF JSON `Value` with pipeline extras included.
pub fn parse_glb(glb: &[u8]) -> Result<Value, String> {
    if glb.len() < 12 {
        return Err("File is not valid binary glTF".to_string());
    }

    let magic = &glb[0..4];
    if magic != b"glTF" {
        return Err("File is not valid binary glTF".to_string());
    }

    let version = u32::from_le_bytes([glb[4], glb[5], glb[6], glb[7]]);
    if version != 1 && version != 2 {
        return Err("Binary glTF version is not 1 or 2".to_string());
    }

    if version == 1 {
        if glb.len() < 20 {
            return Err("File is not valid binary glTF".to_string());
        }
        let length = u32::from_le_bytes([glb[8], glb[9], glb[10], glb[11]]) as usize;
        let content_length = u32::from_le_bytes([glb[12], glb[13], glb[14], glb[15]]) as usize;
        let content_format = u32::from_le_bytes([glb[16], glb[17], glb[18], glb[19]]);

        if content_format != 0 {
            return Err("Binary glTF scene format is not JSON".to_string());
        }

        let json_start = 20;
        let json_end = json_start + content_length;
        if json_end > glb.len() {
            return Err("Unexpected EOF in JSON chunk".to_string());
        }

        let json_str = std::str::from_utf8(&glb[json_start..json_end])
            .map_err(|e| format!("Invalid UTF-8 in JSON chunk: {e}"))?;
        let mut gltf: Value = serde_json::from_str(json_str)
            .map_err(|e| format!("Invalid JSON: {e}"))?;

        gltf = add_pipeline_extras(gltf);

        let binary_start = json_end;
        let binary_end = length.min(glb.len());
        let binary_slice = &glb[binary_start..binary_end];
        let binary_json = Value::Array(binary_slice.iter().map(|&b| json!(b)).collect());

        if let Some(buffers) = gltf.get_mut("buffers") {
            let buffer_obj_opt = if buffers.is_object() {
                if buffers.get("binary_glTF").is_some() {
                    buffers.get_mut("binary_glTF")
                } else {
                    buffers.get_mut("KHR_binary_glTF")
                }
            } else if let Some(arr) = buffers.as_array_mut() {
                arr.first_mut()
            } else {
                None
            };

            if let Some(buffer) = buffer_obj_opt {
                if let Some(extras) = buffer.get_mut("extras").and_then(|e| e.as_object_mut()) {
                    if let Some(pipeline) = extras.get_mut("_pipeline").and_then(|p| p.as_object_mut()) {
                        pipeline.insert("source".to_string(), binary_json);
                    }
                }
                if let Some(obj) = buffer.as_object_mut() {
                    obj.remove("uri");
                }
            }
        }

        remove_extensions_used(&mut gltf, "KHR_binary_glTF");
        Ok(gltf)
    } else {
        let length = u32::from_le_bytes([glb[8], glb[9], glb[10], glb[11]]) as usize;
        let mut byte_offset = 12;
        let mut gltf_opt: Option<Value> = None;
        let mut binary_buffer_opt: Option<Vec<Value>> = None;

        while byte_offset + 8 <= length && byte_offset + 8 <= glb.len() {
            let chunk_length = u32::from_le_bytes([
                glb[byte_offset],
                glb[byte_offset + 1],
                glb[byte_offset + 2],
                glb[byte_offset + 3],
            ]) as usize;
            let chunk_type = u32::from_le_bytes([
                glb[byte_offset + 4],
                glb[byte_offset + 5],
                glb[byte_offset + 6],
                glb[byte_offset + 7],
            ]);
            byte_offset += 8;

            let chunk_end = (byte_offset + chunk_length).min(glb.len());
            let chunk_data = &glb[byte_offset..chunk_end];
            byte_offset += chunk_length;

            if chunk_type == 0x4e4f534a {
                let json_str = std::str::from_utf8(chunk_data)
                    .map_err(|e| format!("Invalid UTF-8 in JSON chunk: {e}"))?;
                let gltf_parsed: Value = serde_json::from_str(json_str)
                    .map_err(|e| format!("Invalid JSON: {e}"))?;
                gltf_opt = Some(add_pipeline_extras(gltf_parsed));
            } else if chunk_type == 0x004e4942 {
                binary_buffer_opt = Some(chunk_data.iter().map(|&b| json!(b)).collect());
            }
        }

        let mut gltf = gltf_opt.ok_or_else(|| "Missing JSON chunk in GLB 2.0".to_string())?;
        if let Some(bin) = binary_buffer_opt {
            if let Some(buffers) = gltf.get_mut("buffers").and_then(|v| v.as_array_mut()) {
                if let Some(buf) = buffers.first_mut() {
                    if let Some(extras) = buf.get_mut("extras").and_then(|e| e.as_object_mut()) {
                        if let Some(pipeline) = extras.get_mut("_pipeline").and_then(|p| p.as_object_mut()) {
                            pipeline.insert("source".to_string(), Value::Array(bin));
                        }
                    }
                }
            }
        }

        Ok(gltf)
    }
}

/// Returns the accessor data in a contiguous array of f64.
pub fn read_accessor_packed(gltf: &Value, accessor: &Value) -> Result<Vec<f64>, String> {
    let type_str = accessor
        .get("type")
        .and_then(|v| v.as_str())
        .unwrap_or("SCALAR");
    let num_components = number_of_components_for_type(type_str);
    let count = accessor
        .get("count")
        .and_then(|v| v.as_u64())
        .unwrap_or(0) as usize;

    let total_values = num_components * count;

    let buffer_view_id = match accessor.get("bufferView").and_then(|v| v.as_u64()) {
        Some(id) => id as usize,
        None => return Ok(vec![0.0; total_values]),
    };

    let buffer_views = gltf
        .get("bufferViews")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "bufferViews missing in glTF".to_string())?;

    let buffer_view = buffer_views
        .get(buffer_view_id)
        .ok_or_else(|| format!("bufferView index {buffer_view_id} out of bounds"))?;

    let buffer_id = buffer_view
        .get("buffer")
        .and_then(|v| v.as_u64())
        .ok_or_else(|| "bufferView.buffer missing".to_string())? as usize;

    let buffers = gltf
        .get("buffers")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "buffers missing in glTF".to_string())?;

    let buffer = buffers
        .get(buffer_id)
        .ok_or_else(|| format!("buffer index {buffer_id} out of bounds"))?;

    let source_bytes: Vec<u8> = if let Some(source_val) = buffer
        .get("extras")
        .and_then(|e| e.get("_pipeline"))
        .and_then(|p| p.get("source"))
    {
        if let Some(arr) = source_val.as_array() {
            arr.iter()
                .filter_map(|v| v.as_u64().map(|b| b as u8))
                .collect()
        } else {
            return Err("buffer.extras._pipeline.source is not an array".to_string());
        }
    } else {
        return Err("buffer.extras._pipeline.source missing".to_string());
    };

    let component_type = accessor
        .get("componentType")
        .and_then(|v| v.as_u64())
        .unwrap_or(5126) as u32;

    let reader = get_component_reader(component_type)?;
    let byte_stride = get_accessor_byte_stride(gltf, accessor);

    let accessor_byte_offset = accessor
        .get("byteOffset")
        .and_then(|v| v.as_u64())
        .unwrap_or(0) as usize;

    let buffer_view_byte_offset = buffer_view
        .get("byteOffset")
        .and_then(|v| v.as_u64())
        .unwrap_or(0) as usize;

    let mut start_offset = accessor_byte_offset + buffer_view_byte_offset;
    let mut values = vec![0.0; total_values];
    let mut components = vec![0.0; num_components];

    for i in 0..count {
        reader(&source_bytes, start_offset, num_components, &mut components);
        for j in 0..num_components {
            values[i * num_components + j] = components[j];
        }
        start_offset += byte_stride;
    }

    Ok(values)
}

fn convert_accessor_type(gltf: &mut Value, accessor_id: usize, updated_component_type: u32) {
    let values = match read_accessor_packed(gltf, &gltf["accessors"][accessor_id]) {
        Ok(v) => v,
        Err(_) => return,
    };

    let mut new_bytes = Vec::new();
    match updated_component_type {
        5121 => {
            // UNSIGNED_BYTE
            for val in values {
                new_bytes.push(val as u8);
            }
        }
        5123 => {
            // UNSIGNED_SHORT
            for val in values {
                let u = val as u16;
                new_bytes.extend_from_slice(&u.to_le_bytes());
            }
        }
        _ => return,
    }

    let buffer_view_id = add_buffer(gltf, &new_bytes);

    if let Some(accessors) = gltf.get_mut("accessors").and_then(|v| v.as_array_mut()) {
        if let Some(accessor) = accessors.get_mut(accessor_id) {
            accessor["bufferView"] = json!(buffer_view_id);
            accessor["componentType"] = json!(updated_component_type);
            accessor["byteOffset"] = json!(0);
        }
    }
}

/// Updates accessors referenced by `JOINTS_0` and `WEIGHTS_0` attributes to use correct component types.
pub fn update_accessor_component_types(gltf: &mut Value) {
    let mut joints_conversions = Vec::new();
    let mut weights_conversions = Vec::new();

    if let Some(meshes) = gltf.get("meshes").and_then(|v| v.as_array()) {
        for mesh in meshes {
            if let Some(primitives) = mesh.get("primitives").and_then(|v| v.as_array()) {
                for prim in primitives {
                    if let Some(attrs) = prim.get("attributes").and_then(|v| v.as_object()) {
                        if let Some(joints_acc_id) = attrs.get("JOINTS_0").and_then(|v| v.as_u64()) {
                            let acc_id = joints_acc_id as usize;
                            if let Some(acc) = gltf.get("accessors").and_then(|a| a.get(acc_id)) {
                                let comp_type = acc.get("componentType").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
                                if comp_type == 5120 {
                                    joints_conversions.push((acc_id, 5121));
                                } else if comp_type != 5121 && comp_type != 5123 {
                                    joints_conversions.push((acc_id, 5123));
                                }
                            }
                        }
                        if let Some(weights_acc_id) = attrs.get("WEIGHTS_0").and_then(|v| v.as_u64()) {
                            let acc_id = weights_acc_id as usize;
                            if let Some(acc) = gltf.get("accessors").and_then(|a| a.get(acc_id)) {
                                let comp_type = acc.get("componentType").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
                                if comp_type == 5120 {
                                    weights_conversions.push((acc_id, 5121));
                                } else if comp_type == 5122 {
                                    weights_conversions.push((acc_id, 5123));
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    for (acc_id, new_type) in joints_conversions {
        convert_accessor_type(gltf, acc_id, new_type);
    }
    for (acc_id, new_type) in weights_conversions {
        convert_accessor_type(gltf, acc_id, new_type);
    }
}








fn read_component_value(bytes: &[u8], offset: usize, component_type: u32) -> Option<f64> {
    match component_type {
        5120 => bytes.get(offset).map(|&b| (b as i8) as f64),
        5121 => bytes.get(offset).map(|&b| b as f64),
        5122 => {
            if offset + 2 <= bytes.len() {
                let v = i16::from_le_bytes([bytes[offset], bytes[offset + 1]]);
                Some(v as f64)
            } else {
                None
            }
        }
        5123 => {
            if offset + 2 <= bytes.len() {
                let v = u16::from_le_bytes([bytes[offset], bytes[offset + 1]]);
                Some(v as f64)
            } else {
                None
            }
        }
        5124 => {
            if offset + 4 <= bytes.len() {
                let v = i32::from_le_bytes([
                    bytes[offset],
                    bytes[offset + 1],
                    bytes[offset + 2],
                    bytes[offset + 3],
                ]);
                Some(v as f64)
            } else {
                None
            }
        }
        5125 => {
            if offset + 4 <= bytes.len() {
                let v = u32::from_le_bytes([
                    bytes[offset],
                    bytes[offset + 1],
                    bytes[offset + 2],
                    bytes[offset + 3],
                ]);
                Some(v as f64)
            } else {
                None
            }
        }
        5126 => {
            if offset + 4 <= bytes.len() {
                let v = f32::from_le_bytes([
                    bytes[offset],
                    bytes[offset + 1],
                    bytes[offset + 2],
                    bytes[offset + 3],
                ]);
                Some(v as f64)
            } else {
                None
            }
        }
        5130 => {
            if offset + 8 <= bytes.len() {
                let v = f64::from_le_bytes([
                    bytes[offset],
                    bytes[offset + 1],
                    bytes[offset + 2],
                    bytes[offset + 3],
                    bytes[offset + 4],
                    bytes[offset + 5],
                    bytes[offset + 6],
                    bytes[offset + 7],
                ]);
                Some(v)
            } else {
                None
            }
        }
        _ => None,
    }
}

/// Finds the min and max values of the glTF accessor.
pub fn find_accessor_min_max(gltf: &Value, accessor: &Value) -> Result<AccessorMinMax, String> {
    let type_str = accessor
        .get("type")
        .and_then(|v| v.as_str())
        .unwrap_or("SCALAR");
    let num_components = number_of_components_for_type(type_str);

    let buffer_view_id = match accessor.get("bufferView").and_then(|v| v.as_u64()) {
        Some(id) => id as usize,
        None => {
            // According to spec: accessor without bufferView returns zeros
            return Ok(AccessorMinMax {
                min: vec![0.0; num_components],
                max: vec![0.0; num_components],
            });
        }
    };

    let buffer_views = gltf
        .get("bufferViews")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "bufferViews missing in glTF".to_string())?;

    let buffer_view = buffer_views
        .get(buffer_view_id)
        .ok_or_else(|| format!("bufferView index {buffer_view_id} out of bounds"))?;

    let buffer_id = buffer_view
        .get("buffer")
        .and_then(|v| v.as_u64())
        .ok_or_else(|| "bufferView.buffer missing".to_string())? as usize;

    let buffers = gltf
        .get("buffers")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "buffers missing in glTF".to_string())?;

    let buffer = buffers
        .get(buffer_id)
        .ok_or_else(|| format!("buffer index {buffer_id} out of bounds"))?;

    let source_bytes: Vec<u8> = if let Some(source_val) = buffer
        .get("extras")
        .and_then(|e| e.get("_pipeline"))
        .and_then(|p| p.get("source"))
    {
        if let Some(arr) = source_val.as_array() {
            arr.iter()
                .filter_map(|v| v.as_u64().map(|b| b as u8))
                .collect()
        } else {
            return Err("buffer.extras._pipeline.source is not an array".to_string());
        }
    } else {
        return Err("buffer.extras._pipeline.source missing".to_string());
    };

    let count = accessor
        .get("count")
        .and_then(|v| v.as_u64())
        .unwrap_or(0) as usize;

    let component_type = accessor
        .get("componentType")
        .and_then(|v| v.as_u64())
        .unwrap_or(5126) as u32;

    let byte_stride = get_accessor_byte_stride(gltf, accessor);
    let comp_size = component_type_size_in_bytes(component_type);

    let accessor_byte_offset = accessor
        .get("byteOffset")
        .and_then(|v| v.as_u64())
        .unwrap_or(0) as usize;

    let buffer_view_byte_offset = buffer_view
        .get("byteOffset")
        .and_then(|v| v.as_u64())
        .unwrap_or(0) as usize;

    let mut start_offset = accessor_byte_offset + buffer_view_byte_offset;

    let mut min = vec![f64::INFINITY; num_components];
    let mut max = vec![f64::NEG_INFINITY; num_components];

    for _ in 0..count {
        for j in 0..num_components {
            let offset = start_offset + j * comp_size;
            if let Some(val) = read_component_value(&source_bytes, offset, component_type) {
                if !val.is_nan() {
                    min[j] = min[j].min(val);
                    max[j] = max[j].max(val);
                }
            }
        }
        start_offset += byte_stride;
    }

    Ok(AccessorMinMax { min, max })
}




