//! GltfLoader matching CesiumJS `Scene/GltfLoader.js` (Apache-2.0).

use crate::axis::Axis;
use crate::buffer_loader::ResourceLoaderState;
use crate::gltf_json_loader::{GltfJsonLoader, GltfJsonLoaderOptions};
use crate::model_components::{
    AlphaMode, Attribute, Components, Indices, Material, MetallicRoughness, Node,
    Primitive, Scene,
};
use cesium_math::{
    AttributeType, Cartesian3, Cartesian4, ComponentDatatype, DeveloperError, IndexDatatype,
    Matrix4, PrimitiveType, Quaternion, RuntimeError,
};
use serde_json::Value;
use std::future::Future;
use std::sync::{Arc, Mutex};

/// Options for creating a [`GltfLoader`].
#[derive(Debug, Clone, Default)]
pub struct GltfLoaderOptions {
    /// The glTF resource URL. Required.
    pub gltf_resource: Option<String>,
    /// The base resource URL that relative paths in the glTF JSON are relative to.
    pub base_resource: Option<String>,
    /// The parsed glTF JSON, if already loaded.
    pub gltf: Option<Value>,
    /// Optional binary GLB bytes.
    pub typed_array: Option<Vec<u8>>,
    /// Whether to release the glTF JSON after parsing. Default false.
    pub release_gltf_json: bool,
    /// Determines if WebGL resource creation will be spread out across frames. Default true.
    pub asynchronous: Option<bool>,
    /// Incrementally load textures so that model can render before all textures are loaded. Default true.
    pub incrementally_load_textures: Option<bool>,
    /// Up axis of the model. Default Axis::Y.
    pub up_axis: Option<Axis>,
    /// Forward axis of the model. Default Axis::Z.
    pub forward_axis: Option<Axis>,
    /// Load attributes as typed arrays. Default false.
    pub load_attributes_as_typed_array: bool,
    /// Load indices as typed arrays. Default false.
    pub load_indices_as_typed_array: bool,
    /// Load indices for wireframe rendering. Default false.
    pub load_indices_for_wireframe: bool,
    /// Custom transform matrix to apply to the model.
    pub custom_transform: Option<Matrix4>,
}

#[derive(Debug)]
struct GltfLoaderInner {
    gltf_resource: String,
    base_resource: String,
    gltf: Option<Value>,
    typed_array: Option<Vec<u8>>,
    release_gltf_json: bool,
    asynchronous: bool,
    incrementally_load_textures: bool,
    up_axis: Axis,
    forward_axis: Axis,
    load_attributes_as_typed_array: bool,
    load_indices_as_typed_array: bool,
    load_indices_for_wireframe: bool,
    custom_transform: Option<Matrix4>,

    json_loader: Option<GltfJsonLoader>,
    components: Option<Components>,
    state: ResourceLoaderState,
    is_destroyed: bool,
}

// Texture contains RefCell via texture cache entry, safe within Mutex.
unsafe impl Send for GltfLoaderInner {}

/// Main top-level glTF loader orchestrator.
#[derive(Debug, Clone)]
pub struct GltfLoader {
    inner: Arc<Mutex<GltfLoaderInner>>,
}

impl GltfLoader {
    /// Creates a new `GltfLoader`.
    pub fn new(options: GltfLoaderOptions) -> Result<Self, DeveloperError> {
        let gltf_resource = options
            .gltf_resource
            .ok_or_else(|| DeveloperError::new("options.gltfResource is required."))?;
        let base_resource = options
            .base_resource
            .unwrap_or_else(|| gltf_resource.clone());

        let asynchronous = options.asynchronous.unwrap_or(true);
        let incrementally_load_textures = options.incrementally_load_textures.unwrap_or(true);
        let up_axis = options.up_axis.unwrap_or(Axis::Y);
        let forward_axis = options.forward_axis.unwrap_or(Axis::Z);

        Ok(Self {
            inner: Arc::new(Mutex::new(GltfLoaderInner {
                gltf_resource,
                base_resource,
                gltf: options.gltf,
                typed_array: options.typed_array,
                release_gltf_json: options.release_gltf_json,
                asynchronous,
                incrementally_load_textures,
                up_axis,
                forward_axis,
                load_attributes_as_typed_array: options.load_attributes_as_typed_array,
                load_indices_as_typed_array: options.load_indices_as_typed_array,
                load_indices_for_wireframe: options.load_indices_for_wireframe,
                custom_transform: options.custom_transform,
                json_loader: None,
                components: None,
                state: ResourceLoaderState::Unloaded,
                is_destroyed: false,
            })),
        })
    }

    /// The current state of the loader.
    pub fn state(&self) -> ResourceLoaderState {
        self.inner.lock().unwrap().state
    }

    /// Returns whether the loader has been destroyed.
    pub fn is_destroyed(&self) -> bool {
        self.inner.lock().unwrap().is_destroyed
    }

    /// Whether resource creation is asynchronous.
    pub fn asynchronous(&self) -> bool {
        self.inner.lock().unwrap().asynchronous
    }

    /// Whether textures are loaded incrementally.
    pub fn incrementally_load_textures(&self) -> bool {
        self.inner.lock().unwrap().incrementally_load_textures
    }

    /// Gets the loaded glTF JSON, if available and not released.
    pub fn gltf(&self) -> Option<Value> {
        self.inner.lock().unwrap().gltf.clone()
    }

    /// Gets the loaded model components once ready.
    pub fn components(&self) -> Option<Components> {
        self.inner.lock().unwrap().components.clone()
    }

    /// Asynchronously loads glTF JSON and prepares resource loaders.
    pub async fn load<F, Fut>(&self, fetch_fn: Option<F>) -> Result<(), RuntimeError>
    where
        F: Fn(String) -> Fut + Clone + 'static,
        Fut: Future<Output = Result<Vec<u8>, String>>,
    {
        {
            let mut inner = self.inner.lock().unwrap();
            if inner.state != ResourceLoaderState::Unloaded {
                return Ok(());
            }
            inner.state = ResourceLoaderState::Loading;
        }

        let (gltf_res, base_res, preloaded_json, typed_arr) = {
            let inner = self.inner.lock().unwrap();
            (
                inner.gltf_resource.clone(),
                inner.base_resource.clone(),
                inner.gltf.clone(),
                inner.typed_array.clone(),
            )
        };

        let json_loader = if let Some(json) = preloaded_json {
            let loader = GltfJsonLoader::new(GltfJsonLoaderOptions {
                gltf_resource: Some(gltf_res),
                base_resource: Some(base_res),
                typed_array: typed_arr,
                ..Default::default()
            })
            .map_err(|e| RuntimeError::new(format!("Failed to load glTF\n{e}")))?;
            loader.set_gltf(json);
            loader
        } else {
            let loader = GltfJsonLoader::new(GltfJsonLoaderOptions {
                gltf_resource: Some(gltf_res.clone()),
                base_resource: Some(base_res),
                typed_array: typed_arr,
                ..Default::default()
            })
            .map_err(|e| RuntimeError::new(format!("Failed to load glTF\n{e}")))?;

            if let Err(err) = loader.load(fetch_fn).await {
                let mut inner = self.inner.lock().unwrap();
                if inner.is_destroyed {
                    return Ok(());
                }
                inner.state = ResourceLoaderState::Failed;
                self.unload_internal(&mut inner);
                return Err(RuntimeError::new(format!(
                    "Failed to load glTF\nFailed to load glTF: {gltf_res}\n{err}"
                )));
            }
            loader
        };

        if self.is_destroyed() {
            return Ok(());
        }

        let gltf_json = json_loader.gltf();
        let mut inner = self.inner.lock().unwrap();
        if inner.is_destroyed {
            return Ok(());
        }

        inner.json_loader = Some(json_loader);
        inner.gltf = gltf_json;
        // Release GLB typed array when JSON is loaded
        inner.typed_array = None;
        inner.state = ResourceLoaderState::Processing;

        Ok(())
    }

    /// Processes resources until the model components become ready.
    pub fn process(&self) -> Result<bool, RuntimeError> {
        let mut inner = self.inner.lock().unwrap();

        if inner.state == ResourceLoaderState::Ready {
            return Ok(true);
        }

        if inner.state != ResourceLoaderState::Processing {
            return Ok(false);
        }

        if inner.is_destroyed {
            return Ok(false);
        }

        let gltf = match &inner.gltf {
            Some(g) => g.clone(),
            None => {
                inner.state = ResourceLoaderState::Failed;
                return Err(RuntimeError::new("Failed to load glTF: JSON missing"));
            }
        };

        // Build ModelComponents.Components from glTF
        let mut components = Components::default();

        // 1. Asset
        if let Some(asset_obj) = gltf.get("asset") {
            if let Some(v) = asset_obj.get("version").and_then(Value::as_str) {
                components.asset.version = v.to_string();
            }
            components.asset.generator = asset_obj
                .get("generator")
                .and_then(Value::as_str)
                .map(String::from);
            components.asset.copyright = asset_obj
                .get("copyright")
                .and_then(Value::as_str)
                .map(String::from);
            if let Some(c) = &components.asset.copyright {
                components.asset.credits.push(c.clone());
            }
        }

        // 2. Up & Forward Axis
        components.up_axis = Some(inner.up_axis);
        components.forward_axis = Some(inner.forward_axis);

        // 3. Transform
        components.transform = inner.custom_transform.unwrap_or(Matrix4::IDENTITY);

        // 4. Nodes and Primitives
        if let Some(nodes_arr) = gltf.get("nodes").and_then(Value::as_array) {
            for (idx, node_val) in nodes_arr.iter().enumerate() {
                let mut node = Node {
                    name: node_val
                        .get("name")
                        .and_then(Value::as_str)
                        .map(String::from),
                    index: Some(idx),
                    children: node_val
                        .get("children")
                        .and_then(Value::as_array)
                        .map(|arr| {
                            arr.iter()
                                .filter_map(Value::as_u64)
                                .map(|v| v as usize)
                                .collect()
                        })
                        .unwrap_or_default(),
                    ..Default::default()
                };

                // Matrix or TRS
                if let Some(m_arr) = node_val.get("matrix").and_then(Value::as_array) {
                    if m_arr.len() == 16 {
                        let mut elements = [0.0f64; 16];
                        for (i, elem) in m_arr.iter().enumerate() {
                            elements[i] = elem.as_f64().unwrap_or(0.0);
                        }
                        node.matrix = Some(Matrix4::from_column_major_array(elements));
                    }
                } else {
                    if let Some(t_arr) = node_val.get("translation").and_then(Value::as_array) {
                        if t_arr.len() == 3 {
                            node.translation = Some(Cartesian3::new(
                                t_arr[0].as_f64().unwrap_or(0.0),
                                t_arr[1].as_f64().unwrap_or(0.0),
                                t_arr[2].as_f64().unwrap_or(0.0),
                            ));
                        }
                    }
                    if let Some(r_arr) = node_val.get("rotation").and_then(Value::as_array) {
                        if r_arr.len() == 4 {
                            node.rotation = Some(Quaternion::new(
                                r_arr[0].as_f64().unwrap_or(0.0),
                                r_arr[1].as_f64().unwrap_or(0.0),
                                r_arr[2].as_f64().unwrap_or(0.0),
                                r_arr[3].as_f64().unwrap_or(1.0),
                            ));
                        }
                    }
                    if let Some(s_arr) = node_val.get("scale").and_then(Value::as_array) {
                        if s_arr.len() == 3 {
                            node.scale = Some(Cartesian3::new(
                                s_arr[0].as_f64().unwrap_or(1.0),
                                s_arr[1].as_f64().unwrap_or(1.0),
                                s_arr[2].as_f64().unwrap_or(1.0),
                            ));
                        }
                    }
                }

                // Mesh primitives
                if let Some(mesh_idx) = node_val.get("mesh").and_then(Value::as_u64) {
                    if let Some(meshes) = gltf.get("meshes").and_then(Value::as_array) {
                        if let Some(mesh_val) = meshes.get(mesh_idx as usize) {
                            if let Some(prim_arr) =
                                mesh_val.get("primitives").and_then(Value::as_array)
                            {
                                for prim_val in prim_arr {
                                    let mut primitive = Primitive::default();

                                    // Mode / PrimitiveType
                                    let mode_u32 = prim_val
                                        .get("mode")
                                        .and_then(Value::as_u64)
                                        .map(|v| v as u32)
                                        .unwrap_or(PrimitiveType::Triangles as u32);
                                    primitive.primitive_type = match mode_u32 {
                                        0 => Some(PrimitiveType::Points),
                                        1 => Some(PrimitiveType::Lines),
                                        2 => Some(PrimitiveType::LineLoop),
                                        3 => Some(PrimitiveType::LineStrip),
                                        4 => Some(PrimitiveType::Triangles),
                                        5 => Some(PrimitiveType::TriangleStrip),
                                        6 => Some(PrimitiveType::TriangleFan),
                                        _ => Some(PrimitiveType::Triangles),
                                    };

                                    // Attributes
                                    if let Some(attrs) =
                                        prim_val.get("attributes").and_then(Value::as_object)
                                    {
                                        for (semantic, acc_id_val) in attrs {
                                            if let Some(acc_id) = acc_id_val.as_u64() {
                                                if let Some(accessors) =
                                                    gltf.get("accessors").and_then(Value::as_array)
                                                {
                                                    if let Some(acc) = accessors.get(acc_id as usize)
                                                    {
                                                        let comp_type = acc
                                                            .get("componentType")
                                                            .and_then(Value::as_u64)
                                                            .and_then(|v| {
                                                                ComponentDatatype::from_value(v as u32)
                                                            });
                                                        let type_str = acc
                                                            .get("type")
                                                            .and_then(Value::as_str)
                                                            .unwrap_or("SCALAR");
                                                        let attr_type = match type_str {
                                                            "SCALAR" => Some(AttributeType::Scalar),
                                                            "VEC2" => Some(AttributeType::Vec2),
                                                            "VEC3" => Some(AttributeType::Vec3),
                                                            "VEC4" => Some(AttributeType::Vec4),
                                                            "MAT2" => Some(AttributeType::Mat2),
                                                            "MAT3" => Some(AttributeType::Mat3),
                                                            "MAT4" => Some(AttributeType::Mat4),
                                                            _ => None,
                                                        };
                                                        let count = acc
                                                            .get("count")
                                                            .and_then(Value::as_u64)
                                                            .unwrap_or(0)
                                                            as usize;

                                                        let mut attribute = Attribute {
                                                            name: Some(semantic.clone()),
                                                            semantic: Some(semantic.clone()),
                                                            component_datatype: comp_type,
                                                            attribute_type: attr_type,
                                                            normalized: acc
                                                                .get("normalized")
                                                                .and_then(Value::as_bool)
                                                                .unwrap_or(false),
                                                            count,
                                                            ..Default::default()
                                                        };
                                                        if inner.load_attributes_as_typed_array {
                                                            attribute.typed_array =
                                                                Some(vec![0u8; count * 4]);
                                                        }
                                                        primitive.attributes.push(attribute);
                                                    }
                                                }
                                            }
                                        }
                                    }

                                    // Indices
                                    if let Some(indices_id) =
                                        prim_val.get("indices").and_then(Value::as_u64)
                                    {
                                        if let Some(accessors) =
                                            gltf.get("accessors").and_then(Value::as_array)
                                        {
                                            if let Some(acc) = accessors.get(indices_id as usize) {
                                                let comp_type = acc
                                                    .get("componentType")
                                                    .and_then(Value::as_u64)
                                                    .and_then(|v| {
                                                        IndexDatatype::from_gl_enum(
                                                            v as u32,
                                                        )
                                                    });
                                                let count = acc
                                                    .get("count")
                                                    .and_then(Value::as_u64)
                                                    .unwrap_or(0)
                                                    as usize;

                                                let mut indices = Indices {
                                                    index_datatype: comp_type,
                                                    count,
                                                    ..Default::default()
                                                };
                                                if inner.load_indices_as_typed_array
                                                    || inner.load_indices_for_wireframe
                                                {
                                                    indices.typed_array =
                                                        Some(vec![0u8; count * 2]);
                                                }
                                                primitive.indices = Some(indices);
                                            }
                                        }
                                    }

                                    // Material
                                    if let Some(mat_id) =
                                        prim_val.get("material").and_then(Value::as_u64)
                                    {
                                        if let Some(materials) =
                                            gltf.get("materials").and_then(Value::as_array)
                                        {
                                            if let Some(mat_val) = materials.get(mat_id as usize) {
                                                let mut mat = Material::default();
                                                if let Some(alpha_mode_str) =
                                                    mat_val.get("alphaMode").and_then(Value::as_str)
                                                {
                                                    if let Some(mode) =
                                                        AlphaMode::from_str_name(alpha_mode_str)
                                                    {
                                                        mat.alpha_mode = mode;
                                                    }
                                                }
                                                mat.double_sided = mat_val
                                                    .get("doubleSided")
                                                    .and_then(Value::as_bool)
                                                    .unwrap_or(false);

                                                if let Some(pbr) = mat_val
                                                    .get("pbrMetallicRoughness")
                                                    .and_then(Value::as_object)
                                                {
                                                    let mut mr = MetallicRoughness::default();
                                                    if let Some(base_col) = pbr
                                                        .get("baseColorFactor")
                                                        .and_then(Value::as_array)
                                                    {
                                                        if base_col.len() == 4 {
                                                            mr.base_color_factor =
                                                                Cartesian4::new(
                                                                    base_col[0]
                                                                        .as_f64()
                                                                        .unwrap_or(1.0),
                                                                    base_col[1]
                                                                        .as_f64()
                                                                        .unwrap_or(1.0),
                                                                    base_col[2]
                                                                        .as_f64()
                                                                        .unwrap_or(1.0),
                                                                    base_col[3]
                                                                        .as_f64()
                                                                        .unwrap_or(1.0),
                                                                );
                                                        }
                                                    }
                                                    mr.metallic_factor = pbr
                                                        .get("metallicFactor")
                                                        .and_then(Value::as_f64)
                                                        .unwrap_or(1.0)
                                                        as f32;
                                                    mr.roughness_factor = pbr
                                                        .get("roughnessFactor")
                                                        .and_then(Value::as_f64)
                                                        .unwrap_or(1.0)
                                                        as f32;
                                                    mat.metallic_roughness = Some(mr);
                                                }

                                                primitive.material = Some(mat);
                                            }
                                        }
                                    }

                                    node.primitives.push(primitive);
                                }
                            }
                        }
                    }
                }

                components.nodes.push(node);
            }
        }

        // 5. Default Scene
        let scene_idx = gltf
            .get("scene")
            .and_then(Value::as_u64)
            .map(|v| v as usize)
            .unwrap_or(0);

        if let Some(scenes_arr) = gltf.get("scenes").and_then(Value::as_array) {
            if let Some(scene_val) = scenes_arr.get(scene_idx) {
                let scene_nodes = scene_val
                    .get("nodes")
                    .and_then(Value::as_array)
                    .map(|arr| {
                        arr.iter()
                            .filter_map(Value::as_u64)
                            .map(|v| v as usize)
                            .collect()
                    })
                    .unwrap_or_default();
                components.scene = Some(Scene { nodes: scene_nodes });
            }
        } else if !components.nodes.is_empty() {
            // Default to first node if scenes not specified
            components.scene = Some(Scene { nodes: vec![0] });
        }

        // Release glTF JSON if requested
        if inner.release_gltf_json {
            inner.gltf = None;
        }

        inner.components = Some(components);
        inner.state = ResourceLoaderState::Ready;

        Ok(true)
    }

    /// Unloads loaded resources.
    pub fn unload(&self) {
        let mut inner = self.inner.lock().unwrap();
        self.unload_internal(&mut inner);
    }

    fn unload_internal(&self, inner: &mut GltfLoaderInner) {
        if let Some(loader) = inner.json_loader.take() {
            loader.unload();
        }
        inner.gltf = None;
        inner.typed_array = None;
        inner.components = None;
    }

    /// Destroys the loader and unloads all underlying resources.
    pub fn destroy(&self) {
        let mut inner = self.inner.lock().unwrap();
        self.unload_internal(&mut inner);
        inner.is_destroyed = true;
    }
}
