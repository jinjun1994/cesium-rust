//! glTF traversal helper functions matching CesiumJS `packages/gltf/Source/ForEach.js` (Apache-2.0).

use std::collections::HashSet;
use serde_json::Value;

/// An identifier passed during traversal, representing either an index (glTF 2.0 array)
/// or a string key (glTF 1.0 dictionary object).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum GltfId<'a> {
    Index(usize),
    Key(&'a str),
}

impl<'a> GltfId<'a> {
    pub fn as_index(&self) -> Option<usize> {
        match self {
            Self::Index(i) => Some(*i),
            Self::Key(s) => s.parse::<usize>().ok(),
        }
    }

    pub fn as_key(&self) -> &'a str {
        match self {
            Self::Key(s) => s,
            _ => "",
        }
    }
}

/// Checks whether the glTF uses the given extension.
pub fn uses_extension(gltf: &Value, extension: &str) -> bool {
    if let Some(extensions_used) = gltf.get("extensionsUsed").and_then(Value::as_array) {
        extensions_used
            .iter()
            .any(|ext| ext.as_str() == Some(extension))
    } else {
        false
    }
}

/// Helper struct for traversing glTF structures.
pub struct ForEach;

impl ForEach {
    /// Iterates over an id-keyed dictionary object (glTF 1.0 fallback).
    pub fn object_legacy<'a, R>(
        objects: &'a Value,
        mut handler: impl FnMut(&'a Value, &'a str) -> Option<R>,
    ) -> Option<R> {
        if let Some(obj_map) = objects.as_object() {
            for (key, val) in obj_map {
                if let Some(res) = handler(val, key.as_str()) {
                    return Some(res);
                }
            }
        }
        None
    }

    /// Iterates over an array of objects (glTF 2.0).
    pub fn object<'a, R>(
        array_of_objects: &'a Value,
        mut handler: impl FnMut(&'a Value, usize) -> Option<R>,
    ) -> Option<R> {
        if let Some(arr) = array_of_objects.as_array() {
            for (i, val) in arr.iter().enumerate() {
                if let Some(res) = handler(val, i) {
                    return Some(res);
                }
            }
        }
        None
    }

    /// Supports glTF 1.0 and 2.0 top-level property traversal.
    pub fn top_level<'a, R>(
        gltf: &'a Value,
        name: &str,
        mut handler: impl FnMut(&'a Value, GltfId<'a>) -> Option<R>,
    ) -> Option<R> {
        if let Some(prop) = gltf.get(name) {
            if let Some(arr) = prop.as_array() {
                for (i, val) in arr.iter().enumerate() {
                    if let Some(res) = handler(val, GltfId::Index(i)) {
                        return Some(res);
                    }
                }
            } else if let Some(map) = prop.as_object() {
                for (key, val) in map {
                    if let Some(res) = handler(val, GltfId::Key(key.as_str())) {
                        return Some(res);
                    }
                }
            }
        }
        None
    }

    /// Loops over accessors.
    pub fn accessor<'a, R>(
        gltf: &'a Value,
        handler: impl FnMut(&'a Value, GltfId<'a>) -> Option<R>,
    ) -> Option<R> {
        Self::top_level(gltf, "accessors", handler)
    }

    /// Loops over meshes.
    pub fn mesh<'a, R>(
        gltf: &'a Value,
        handler: impl FnMut(&'a Value, GltfId<'a>) -> Option<R>,
    ) -> Option<R> {
        Self::top_level(gltf, "meshes", handler)
    }

    /// Loops over mesh primitives.
    pub fn mesh_primitive<'a, R>(
        mesh: &'a Value,
        mut handler: impl FnMut(&'a Value, usize) -> Option<R>,
    ) -> Option<R> {
        if let Some(primitives) = mesh.get("primitives").and_then(Value::as_array) {
            for (i, prim) in primitives.iter().enumerate() {
                if let Some(res) = handler(prim, i) {
                    return Some(res);
                }
            }
        }
        None
    }

    /// Loops over mesh primitive attributes.
    pub fn mesh_primitive_attribute<'a, R>(
        primitive: &'a Value,
        mut handler: impl FnMut(&'a Value, &'a str) -> Option<R>,
    ) -> Option<R> {
        if let Some(attributes) = primitive.get("attributes").and_then(Value::as_object) {
            for (semantic, accessor_id) in attributes {
                if let Some(res) = handler(accessor_id, semantic.as_str()) {
                    return Some(res);
                }
            }
        }
        None
    }

    /// Loops over mesh primitive morph targets.
    pub fn mesh_primitive_target<'a, R>(
        primitive: &'a Value,
        mut handler: impl FnMut(&'a Value, usize) -> Option<R>,
    ) -> Option<R> {
        if let Some(targets) = primitive.get("targets").and_then(Value::as_array) {
            for (i, target) in targets.iter().enumerate() {
                if let Some(res) = handler(target, i) {
                    return Some(res);
                }
            }
        }
        None
    }

    /// Loops over morph target attributes.
    pub fn mesh_primitive_target_attribute<'a, R>(
        target: &'a Value,
        mut handler: impl FnMut(&'a Value, &'a str) -> Option<R>,
    ) -> Option<R> {
        if let Some(target_map) = target.as_object() {
            for (semantic, accessor_id) in target_map {
                if let Some(res) = handler(accessor_id, semantic.as_str()) {
                    return Some(res);
                }
            }
        }
        None
    }

    /// Loops over accessors matching a semantic prefix.
    pub fn accessor_with_semantic<'a, R>(
        gltf: &'a Value,
        semantic: &str,
        mut handler: impl FnMut(&'a Value) -> Option<R>,
    ) -> Option<R> {
        let mut visited = HashSet::new();

        Self::mesh(gltf, |mesh, _| {
            Self::mesh_primitive(mesh, |primitive, _| {
                let attr_res = Self::mesh_primitive_attribute(primitive, |accessor_id, attr_semantic| {
                    if attr_semantic.starts_with(semantic) {
                        let key = accessor_id.to_string();
                        if visited.insert(key) {
                            if let Some(res) = handler(accessor_id) {
                                return Some(res);
                            }
                        }
                    }
                    None
                });

                if attr_res.is_some() {
                    return attr_res;
                }

                Self::mesh_primitive_target(primitive, |target, _| {
                    Self::mesh_primitive_target_attribute(target, |accessor_id, attr_semantic| {
                        if attr_semantic.starts_with(semantic) {
                            let key = accessor_id.to_string();
                            if visited.insert(key) {
                                if let Some(res) = handler(accessor_id) {
                                    return Some(res);
                                }
                            }
                        }
                        None
                    })
                })
            })
        })
    }

    /// Loops once over each accessor containing vertex attribute data.
    pub fn accessor_containing_vertex_attribute_data<'a, R>(
        gltf: &'a Value,
        mut handler: impl FnMut(&'a Value) -> Option<R>,
    ) -> Option<R> {
        let mut visited = HashSet::new();

        Self::mesh(gltf, |mesh, _| {
            Self::mesh_primitive(mesh, |primitive, _| {
                let attr_res = Self::mesh_primitive_attribute(primitive, |accessor_id, _| {
                    let key = accessor_id.to_string();
                    if visited.insert(key) {
                        if let Some(res) = handler(accessor_id) {
                            return Some(res);
                        }
                    }
                    None
                });

                if attr_res.is_some() {
                    return attr_res;
                }

                Self::mesh_primitive_target(primitive, |target, _| {
                    Self::mesh_primitive_target_attribute(target, |accessor_id, _| {
                        let key = accessor_id.to_string();
                        if visited.insert(key) {
                            if let Some(res) = handler(accessor_id) {
                                return Some(res);
                            }
                        }
                        None
                    })
                })
            })
        })
    }

    /// Loops once over each accessor containing index data.
    pub fn accessor_containing_index_data<'a, R>(
        gltf: &'a Value,
        mut handler: impl FnMut(&'a Value) -> Option<R>,
    ) -> Option<R> {
        let mut visited = HashSet::new();

        Self::mesh(gltf, |mesh, _| {
            Self::mesh_primitive(mesh, |primitive, _| {
                if let Some(indices) = primitive.get("indices") {
                    let key = indices.to_string();
                    if visited.insert(key) {
                        if let Some(res) = handler(indices) {
                            return Some(res);
                        }
                    }
                }
                None
            })
        })
    }

    /// Loops over animations.
    pub fn animation<'a, R>(
        gltf: &'a Value,
        handler: impl FnMut(&'a Value, GltfId<'a>) -> Option<R>,
    ) -> Option<R> {
        Self::top_level(gltf, "animations", handler)
    }

    /// Loops over animation channels.
    pub fn animation_channel<'a, R>(
        animation: &'a Value,
        handler: impl FnMut(&'a Value, usize) -> Option<R>,
    ) -> Option<R> {
        if let Some(channels) = animation.get("channels") {
            Self::object(channels, handler)
        } else {
            None
        }
    }

    /// Loops over animation samplers.
    pub fn animation_sampler<'a, R>(
        animation: &'a Value,
        handler: impl FnMut(&'a Value, usize) -> Option<R>,
    ) -> Option<R> {
        if let Some(samplers) = animation.get("samplers") {
            Self::object(samplers, handler)
        } else {
            None
        }
    }

    /// Loops over buffers.
    pub fn buffer<'a, R>(
        gltf: &'a Value,
        handler: impl FnMut(&'a Value, GltfId<'a>) -> Option<R>,
    ) -> Option<R> {
        Self::top_level(gltf, "buffers", handler)
    }

    /// Loops over buffer views.
    pub fn buffer_view<'a, R>(
        gltf: &'a Value,
        handler: impl FnMut(&'a Value, GltfId<'a>) -> Option<R>,
    ) -> Option<R> {
        Self::top_level(gltf, "bufferViews", handler)
    }

    /// Loops over cameras.
    pub fn camera<'a, R>(
        gltf: &'a Value,
        handler: impl FnMut(&'a Value, GltfId<'a>) -> Option<R>,
    ) -> Option<R> {
        Self::top_level(gltf, "cameras", handler)
    }

    /// Loops over images.
    pub fn image<'a, R>(
        gltf: &'a Value,
        handler: impl FnMut(&'a Value, GltfId<'a>) -> Option<R>,
    ) -> Option<R> {
        Self::top_level(gltf, "images", handler)
    }

    /// Loops over materials.
    pub fn material<'a, R>(
        gltf: &'a Value,
        handler: impl FnMut(&'a Value, GltfId<'a>) -> Option<R>,
    ) -> Option<R> {
        Self::top_level(gltf, "materials", handler)
    }

    /// Loops over material values (supporting KHR_techniques_webgl extension).
    pub fn material_value<'a, R>(
        material: &'a Value,
        mut handler: impl FnMut(&'a Value, &'a str) -> Option<R>,
    ) -> Option<R> {
        let values = if let Some(khr) = material
            .get("extensions")
            .and_then(|ext| ext.get("KHR_techniques_webgl"))
            .and_then(|tech| tech.get("values"))
        {
            Some(khr)
        } else {
            material.get("values")
        };

        if let Some(val_map) = values.and_then(Value::as_object) {
            for (name, val) in val_map {
                if let Some(res) = handler(val, name.as_str()) {
                    return Some(res);
                }
            }
        }
        None
    }

    /// Loops over nodes.
    pub fn node<'a, R>(
        gltf: &'a Value,
        handler: impl FnMut(&'a Value, GltfId<'a>) -> Option<R>,
    ) -> Option<R> {
        Self::top_level(gltf, "nodes", handler)
    }

    /// Loops over nodes in a tree rooted at node_ids.
    pub fn node_in_tree<'a, R>(
        gltf: &'a Value,
        node_ids: &'a [Value],
        handler: &mut dyn FnMut(&'a Value, &'a Value) -> Option<R>,
    ) -> Option<R> {
        let nodes = gltf.get("nodes")?;
        for node_id in node_ids {
            let node = if let Some(idx) = node_id.as_u64() {
                nodes.get(idx as usize)
            } else if let Some(key) = node_id.as_str() {
                nodes.get(key)
            } else {
                None
            };

            if let Some(node) = node {
                if let Some(res) = handler(node, node_id) {
                    return Some(res);
                }
                if let Some(children) = node.get("children").and_then(Value::as_array) {
                    if let Some(res) = Self::node_in_tree(gltf, children, handler) {
                        return Some(res);
                    }
                }
            }
        }
        None
    }

    /// Loops over nodes in a scene.
    pub fn node_in_scene<'a, R>(
        gltf: &'a Value,
        scene: &'a Value,
        mut handler: impl FnMut(&'a Value, &'a Value) -> Option<R>,
    ) -> Option<R> {
        if let Some(scene_node_ids) = scene.get("nodes").and_then(Value::as_array) {
            Self::node_in_tree(gltf, scene_node_ids, &mut handler)
        } else {
            None
        }
    }

    /// Loops over programs (supporting KHR_techniques_webgl).
    pub fn program<'a, R>(
        gltf: &'a Value,
        mut handler: impl FnMut(&'a Value, GltfId<'a>) -> Option<R>,
    ) -> Option<R> {
        if uses_extension(gltf, "KHR_techniques_webgl") {
            if let Some(programs) = gltf
                .get("extensions")
                .and_then(|ext| ext.get("KHR_techniques_webgl"))
                .and_then(|khr| khr.get("programs"))
            {
                if let Some(arr) = programs.as_array() {
                    for (i, p) in arr.iter().enumerate() {
                        if let Some(res) = handler(p, GltfId::Index(i)) {
                            return Some(res);
                        }
                    }
                } else if let Some(obj) = programs.as_object() {
                    for (k, p) in obj {
                        if let Some(res) = handler(p, GltfId::Key(k.as_str())) {
                            return Some(res);
                        }
                    }
                }
                return None;
            }
        }
        Self::top_level(gltf, "programs", handler)
    }

    /// Loops over samplers.
    pub fn sampler<'a, R>(
        gltf: &'a Value,
        handler: impl FnMut(&'a Value, GltfId<'a>) -> Option<R>,
    ) -> Option<R> {
        Self::top_level(gltf, "samplers", handler)
    }

    /// Loops over scenes.
    pub fn scene<'a, R>(
        gltf: &'a Value,
        handler: impl FnMut(&'a Value, GltfId<'a>) -> Option<R>,
    ) -> Option<R> {
        Self::top_level(gltf, "scenes", handler)
    }

    /// Loops over shaders (supporting KHR_techniques_webgl).
    pub fn shader<'a, R>(
        gltf: &'a Value,
        mut handler: impl FnMut(&'a Value, GltfId<'a>) -> Option<R>,
    ) -> Option<R> {
        if uses_extension(gltf, "KHR_techniques_webgl") {
            if let Some(shaders) = gltf
                .get("extensions")
                .and_then(|ext| ext.get("KHR_techniques_webgl"))
                .and_then(|khr| khr.get("shaders"))
            {
                if let Some(arr) = shaders.as_array() {
                    for (i, s) in arr.iter().enumerate() {
                        if let Some(res) = handler(s, GltfId::Index(i)) {
                            return Some(res);
                        }
                    }
                } else if let Some(obj) = shaders.as_object() {
                    for (k, s) in obj {
                        if let Some(res) = handler(s, GltfId::Key(k.as_str())) {
                            return Some(res);
                        }
                    }
                }
                return None;
            }
        }
        Self::top_level(gltf, "shaders", handler)
    }

    /// Loops over skins.
    pub fn skin<'a, R>(
        gltf: &'a Value,
        handler: impl FnMut(&'a Value, GltfId<'a>) -> Option<R>,
    ) -> Option<R> {
        Self::top_level(gltf, "skins", handler)
    }

    /// Loops over skin joints.
    pub fn skin_joint<'a, R>(
        skin: &'a Value,
        mut handler: impl FnMut(&'a Value) -> Option<R>,
    ) -> Option<R> {
        if let Some(joints) = skin.get("joints").and_then(Value::as_array) {
            for joint in joints {
                if let Some(res) = handler(joint) {
                    return Some(res);
                }
            }
        }
        None
    }

    /// Loops over techniques (supporting KHR_techniques_webgl).
    pub fn technique<'a, R>(
        gltf: &'a Value,
        mut handler: impl FnMut(&'a Value, GltfId<'a>) -> Option<R>,
    ) -> Option<R> {
        if uses_extension(gltf, "KHR_techniques_webgl") {
            if let Some(techniques) = gltf
                .get("extensions")
                .and_then(|ext| ext.get("KHR_techniques_webgl"))
                .and_then(|khr| khr.get("techniques"))
            {
                if let Some(arr) = techniques.as_array() {
                    for (i, t) in arr.iter().enumerate() {
                        if let Some(res) = handler(t, GltfId::Index(i)) {
                            return Some(res);
                        }
                    }
                } else if let Some(obj) = techniques.as_object() {
                    for (k, t) in obj {
                        if let Some(res) = handler(t, GltfId::Key(k.as_str())) {
                            return Some(res);
                        }
                    }
                }
                return None;
            }
        }
        Self::top_level(gltf, "techniques", handler)
    }

    /// Loops over technique attributes.
    pub fn technique_attribute<'a, R>(
        technique: &'a Value,
        mut handler: impl FnMut(&'a Value, &'a str) -> Option<R>,
    ) -> Option<R> {
        if let Some(attributes) = technique.get("attributes").and_then(Value::as_object) {
            for (attr_name, attr) in attributes {
                if let Some(res) = handler(attr, attr_name.as_str()) {
                    return Some(res);
                }
            }
        }
        None
    }

    /// Loops over technique uniforms.
    pub fn technique_uniform<'a, R>(
        technique: &'a Value,
        mut handler: impl FnMut(&'a Value, &'a str) -> Option<R>,
    ) -> Option<R> {
        if let Some(uniforms) = technique.get("uniforms").and_then(Value::as_object) {
            for (uniform_name, uniform) in uniforms {
                if let Some(res) = handler(uniform, uniform_name.as_str()) {
                    return Some(res);
                }
            }
        }
        None
    }

    /// Loops over technique parameters (glTF 1.0).
    pub fn technique_parameter<'a, R>(
        technique: &'a Value,
        mut handler: impl FnMut(&'a Value, &'a str) -> Option<R>,
    ) -> Option<R> {
        if let Some(parameters) = technique.get("parameters").and_then(Value::as_object) {
            for (param_name, param) in parameters {
                if let Some(res) = handler(param, param_name.as_str()) {
                    return Some(res);
                }
            }
        }
        None
    }

    /// Loops over textures.
    pub fn texture<'a, R>(
        gltf: &'a Value,
        handler: impl FnMut(&'a Value, GltfId<'a>) -> Option<R>,
    ) -> Option<R> {
        Self::top_level(gltf, "textures", handler)
    }
}

/// Calls the provider handler function on each texture used by the material.
/// Mimics the behavior of `forEachTextureInMaterial` in gltf-pipeline.
pub fn for_each_texture_in_material<'a, F, R>(
    material: Option<&'a Value>,
    mut handler: F,
) -> Option<R>
where
    F: FnMut(usize, &'a Value) -> Option<R>,
{
    let material = material.expect("material must be defined");
    if !material.is_object() {
        panic!("material must be an object");
    }

    let mut check_texture = |tex: Option<&'a Value>| -> Option<R> {
        if let Some(texture_info) = tex {
            if let Some(index) = texture_info.get("index").and_then(Value::as_u64) {
                return handler(index as usize, texture_info);
            }
        }
        None
    };

    // 1. Metallic roughness
    if let Some(pbr) = material.get("pbrMetallicRoughness") {
        if let Some(res) = check_texture(pbr.get("baseColorTexture")) {
            return Some(res);
        }
        if let Some(res) = check_texture(pbr.get("metallicRoughnessTexture")) {
            return Some(res);
        }
    }

    // 2. Extensions
    if let Some(extensions) = material.get("extensions") {
        // Spec gloss extension
        if let Some(spec_gloss) = extensions.get("KHR_materials_pbrSpecularGlossiness") {
            if let Some(res) = check_texture(spec_gloss.get("diffuseTexture")) {
                return Some(res);
            }
            if let Some(res) = check_texture(spec_gloss.get("specularGlossinessTexture")) {
                return Some(res);
            }
        }

        // Specular extension
        if let Some(specular) = extensions.get("KHR_materials_specular") {
            if let Some(res) = check_texture(specular.get("specularTexture")) {
                return Some(res);
            }
            if let Some(res) = check_texture(specular.get("specularColorTexture")) {
                return Some(res);
            }
        }

        // Transmission extension
        if let Some(transmission) = extensions.get("KHR_materials_transmission") {
            if let Some(res) = check_texture(transmission.get("transmissionTexture")) {
                return Some(res);
            }
        }

        // Materials common extension
        if let Some(materials_common) = extensions.get("KHR_materials_common") {
            if let Some(values) = materials_common.get("values") {
                if let Some(res) = check_texture(values.get("diffuse")) {
                    return Some(res);
                }
                if let Some(res) = check_texture(values.get("ambient")) {
                    return Some(res);
                }
                if let Some(res) = check_texture(values.get("emission")) {
                    return Some(res);
                }
                if let Some(res) = check_texture(values.get("specular")) {
                    return Some(res);
                }
            }
        }
    }

    // 3. KHR_techniques_webgl extension
    if let Some(res) = ForEach::material_value(material, |material_value, _| {
        check_texture(Some(material_value))
    }) {
        return Some(res);
    }

    // 4. Top level textures
    if let Some(res) = check_texture(material.get("emissiveTexture")) {
        return Some(res);
    }
    if let Some(res) = check_texture(material.get("normalTexture")) {
        return Some(res);
    }
    if let Some(res) = check_texture(material.get("occlusionTexture")) {
        return Some(res);
    }

    None
}

