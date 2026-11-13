//! Ray-casting picking for 3D Models matching CesiumJS `Scene/Model/pickModel.js` (Apache-2.0).

use cesium_math::{
    Cartesian3, Ellipsoid, FixedFrameTransforms, IntersectionTests, MapProjection,
    Matrix4, Quaternion, Ray, SceneMode, VerticalExaggeration,
};

use crate::model_components::{Attribute, Indices, Instances, Primitive};
use crate::model_reader::ModelReader;
use crate::model_runtime_node::ModelRuntimeNode;
use crate::model_scene_graph::ModelSceneGraph;

pub use crate::model::Model;

/// Frame state options passed to `pick_model`.
#[derive(Clone, Debug)]
pub struct PickFrameState<'a, P: MapProjection = cesium_math::GeographicProjection> {
    pub mode: SceneMode,
    pub map_projection: Option<&'a P>,
}

impl<'a, P: MapProjection> PickFrameState<'a, P> {
    pub fn new(mode: SceneMode, map_projection: Option<&'a P>) -> Self {
        Self {
            mode,
            map_projection,
        }
    }
}

impl<'a> PickFrameState<'a, cesium_math::GeographicProjection> {
    pub fn new_3d(mode: SceneMode) -> Self {
        Self {
            mode,
            map_projection: None,
        }
    }
}

impl Default for PickFrameState<'_, cesium_math::GeographicProjection> {
    fn default() -> Self {
        Self {
            mode: SceneMode::Scene3D,
            map_projection: None,
        }
    }
}

fn read_position(vertices: &[f32], index: usize, stride: usize) -> Cartesian3 {
    let offset = index * stride;
    if offset + 2 < vertices.len() {
        Cartesian3::new(
            vertices[offset] as f64,
            vertices[offset + 1] as f64,
            vertices[offset + 2] as f64,
        )
    } else {
        Cartesian3::ZERO
    }
}

fn read_indices_as_u32(indices: &Indices) -> Vec<u32> {
    let bytes = match &indices.typed_array {
        Some(b) => b,
        None => return Vec::new(),
    };

    let count = indices.count;
    let mut result = Vec::with_capacity(count);

    let datatype = indices.index_datatype.unwrap_or_else(|| {
        if bytes.len() == count {
            cesium_math::IndexDatatype::UnsignedByte
        } else if bytes.len() == count * 2 {
            cesium_math::IndexDatatype::UnsignedShort
        } else {
            cesium_math::IndexDatatype::UnsignedInt
        }
    });

    match datatype {
        cesium_math::IndexDatatype::UnsignedByte => {
            for &b in bytes.iter().take(count) {
                result.push(b as u32);
            }
        }
        cesium_math::IndexDatatype::UnsignedShort => {
            for chunk in bytes.as_chunks::<2>().0.iter().take(count) {
                let val = u16::from_le_bytes(*chunk);
                result.push(val as u32);
            }
        }
        cesium_math::IndexDatatype::UnsignedInt => {
            for chunk in bytes.as_chunks::<4>().0.iter().take(count) {
                let val = u32::from_le_bytes(*chunk);
                result.push(val);
            }
        }
    }

    result
}

struct NodeTransforms {
    node_computed_transform: Matrix4,
    model_matrix: Matrix4,
    computed_model_matrix: Matrix4,
}

fn compute_node_transforms(
    runtime_node: &ModelRuntimeNode,
    scene_graph: &ModelSceneGraph,
    model: &Model,
) -> NodeTransforms {
    let mut node_computed_transform = runtime_node.computed_transform;
    let mut model_matrix = scene_graph.computed_model_matrix;

    if let Some(node) = &runtime_node.node {
        if let Some(instances) = &node.instances {
            if instances.transform_in_world_space {
                model_matrix = model
                    .model_matrix
                    .multiply_transformation(&scene_graph.components.transform);
                node_computed_transform = scene_graph
                    .axis_correction_matrix
                    .multiply_transformation(&runtime_node.computed_transform);
            }
        }
    }

    let computed_model_matrix = model_matrix.multiply_transformation(&node_computed_transform);

    NodeTransforms {
        node_computed_transform,
        model_matrix,
        computed_model_matrix,
    }
}

fn parse_instance_transforms(instances: &Instances) -> Vec<Matrix4> {
    let count = instances
        .attributes
        .first()
        .map(|a| a.count)
        .unwrap_or(0);
    if count == 0 {
        return Vec::new();
    }

    let mut translations = Vec::new();
    let mut rotations = Vec::new();
    let mut scales = Vec::new();

    for attr in &instances.attributes {
        if let Some(sem) = &attr.semantic {
            match sem.as_str() {
                "TRANSLATION" => {
                    if let Ok(floats) = ModelReader::read_attribute_as_typed_array(attr) {
                        for chunk in floats.as_chunks::<3>().0.iter().take(count) {
                            translations.push(Cartesian3::new(
                                chunk[0] as f64,
                                chunk[1] as f64,
                                chunk[2] as f64,
                            ));
                        }
                    }
                }
                "ROTATION" => {
                    if let Ok(floats) = ModelReader::read_attribute_as_typed_array(attr) {
                        for chunk in floats.as_chunks::<4>().0.iter().take(count) {
                            rotations.push(Quaternion::new(
                                chunk[0] as f64,
                                chunk[1] as f64,
                                chunk[2] as f64,
                                chunk[3] as f64,
                            ));
                        }
                    }
                }
                "SCALE" => {
                    if let Ok(floats) = ModelReader::read_attribute_as_typed_array(attr) {
                        for chunk in floats.as_chunks::<3>().0.iter().take(count) {
                            scales.push(Cartesian3::new(
                                chunk[0] as f64,
                                chunk[1] as f64,
                                chunk[2] as f64,
                            ));
                        }
                    }
                }
                _ => {}
            }
        }
    }

    let mut result = Vec::with_capacity(count);
    for i in 0..count {
        let t = translations.get(i).copied().unwrap_or(Cartesian3::ZERO);
        let r = rotations.get(i).copied().unwrap_or(Quaternion::IDENTITY);
        let s = scales.get(i).copied().unwrap_or(Cartesian3::new(1.0, 1.0, 1.0));
        result.push(Matrix4::from_translation_quaternion_rotation_scale(t, r, s));
    }

    result
}

fn compute_instance_transforms(
    runtime_node: &ModelRuntimeNode,
    computed_model_matrix: &Matrix4,
    node_computed_transform: &Matrix4,
    model_matrix: &Matrix4,
) -> Vec<Matrix4> {
    let mut transforms = Vec::new();

    if let Some(node) = &runtime_node.node {
        if let Some(instances) = &node.instances {
            let mut inst_transforms = parse_instance_transforms(instances);
            if !inst_transforms.is_empty() {
                for transform in &mut inst_transforms {
                    if instances.transform_in_world_space {
                        *transform = transform.multiply_transformation(node_computed_transform);
                        *transform = model_matrix.multiply_transformation(transform);
                    } else {
                        *transform = computed_model_matrix.multiply_transformation(transform);
                    }
                }
                transforms = inst_transforms;
            }
        }
    }

    if transforms.is_empty() {
        transforms.push(*computed_model_matrix);
    }

    transforms
}

/// Find an intersection between a ray and the model surface that was rendered.
/// The ray must be given in world coordinates.
pub fn pick_model<P: MapProjection>(
    model: &Model,
    ray: &Ray,
    frame_state: &PickFrameState<'_, P>,
    vertical_exaggeration: Option<f64>,
    relative_height: Option<f64>,
    ellipsoid: Option<&Ellipsoid>,
    result: Option<&mut Cartesian3>,
) -> Option<Cartesian3> {
    if !model.ready || frame_state.mode == SceneMode::Morphing {
        return None;
    }

    let scene_graph = model.scene_graph.as_ref()?;

    let default_ellipsoid = Ellipsoid::default();
    let ellipsoid = ellipsoid.unwrap_or(&default_ellipsoid);
    let vertical_exaggeration = vertical_exaggeration.unwrap_or(1.0);
    let relative_height = relative_height.unwrap_or(0.0);
    let cull_back_faces = model.back_face_culling.unwrap_or(true);

    let mut min_t = f64::MAX;

    for runtime_node in &scene_graph.runtime_nodes {
        let node_transforms = compute_node_transforms(runtime_node, scene_graph, model);
        let mut computed_model_matrix = node_transforms.computed_model_matrix;

        if frame_state.mode != SceneMode::Scene3D {
            if let Some(map_projection) = frame_state.map_projection {
                computed_model_matrix = FixedFrameTransforms::basis_to_2d(
                    map_projection,
                    &computed_model_matrix,
                )
                .unwrap_or(computed_model_matrix);
            }
        }

        let instance_transforms = compute_instance_transforms(
            runtime_node,
            &computed_model_matrix,
            &node_transforms.node_computed_transform,
            &node_transforms.model_matrix,
        );

        for runtime_primitive in &runtime_node.runtime_primitives {
            // Bounding sphere early-out for non-instanced primitives
            if let Some(bs) = &runtime_primitive.bounding_sphere {
                if instance_transforms.len() == 1 {
                    if let Ok(transformed_bs) = bs.transform((&computed_model_matrix).into()) {
                        if IntersectionTests::ray_sphere(ray, &transformed_bs).is_none() {
                            continue;
                        }
                    }
                }
            }

            let primitive: &Primitive = match &runtime_primitive.primitive {
                Some(p) => p,
                None => continue,
            };

            let indices_component = match &primitive.indices {
                Some(i) => i,
                None => continue, // Point clouds without indices
            };

            let pos_attr: &Attribute = match primitive
                .attributes
                .iter()
                .find(|a| a.semantic.as_deref() == Some("POSITION"))
            {
                Some(a) => a,
                None => continue,
            };

            let num_pos_components = pos_attr
                .attribute_type
                .map(|t| t.get_number_of_components())
                .unwrap_or(3);

            let vertices = match ModelReader::read_attribute_as_typed_array(pos_attr) {
                Ok(v) => v,
                Err(_) => continue,
            };

            let indices = read_indices_as_u32(indices_component);
            if indices.is_empty() || vertices.is_empty() {
                continue;
            }

            for chunk in indices.as_chunks::<3>().0 {
                let i0 = chunk[0] as usize;
                let i1 = chunk[1] as usize;
                let i2 = chunk[2] as usize;

                let v0 = read_position(&vertices, i0, num_pos_components);
                let v1 = read_position(&vertices, i1, num_pos_components);
                let v2 = read_position(&vertices, i2, num_pos_components);

                for inst_transform in &instance_transforms {
                    let mut v0_t = inst_transform.multiply_by_point(v0);
                    let mut v1_t = inst_transform.multiply_by_point(v1);
                    let mut v2_t = inst_transform.multiply_by_point(v2);

                    if vertical_exaggeration != 1.0 {
                        v0_t = VerticalExaggeration::get_position(
                            &v0_t,
                            ellipsoid,
                            vertical_exaggeration,
                            relative_height,
                        );
                        v1_t = VerticalExaggeration::get_position(
                            &v1_t,
                            ellipsoid,
                            vertical_exaggeration,
                            relative_height,
                        );
                        v2_t = VerticalExaggeration::get_position(
                            &v2_t,
                            ellipsoid,
                            vertical_exaggeration,
                            relative_height,
                        );
                    }

                    if let Some(t) = IntersectionTests::ray_triangle_parametric(
                        ray,
                        v0_t.into(),
                        v1_t.into(),
                        v2_t.into(),
                        cull_back_faces,
                    ) {
                        if t >= 0.0 && t < min_t {
                            min_t = t;
                        }
                    }
                }
            }
        }
    }

    if min_t == f64::MAX {
        return None;
    }

    let mut hit: Cartesian3 = ray.get_point(min_t).into();
    if frame_state.mode != SceneMode::Scene3D {
        let pt_2d = Cartesian3::new(hit.y, hit.z, hit.x);
        if let Some(map_projection) = frame_state.map_projection {
            let ell = map_projection.ellipsoid();
            if let Some(carto) = map_projection.unproject(pt_2d.into()) {
                if let Some(cartesian) = ell.cartographic_to_cartesian(carto) {
                    hit = cartesian.into();
                }
            }
        }
    }

    if let Some(res) = result {
        *res = hit;
    }

    Some(hit)
}
