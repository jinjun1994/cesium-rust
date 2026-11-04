//! Geometry processing pipeline matching CesiumJS `Core/GeometryPipeline`.

use std::collections::HashMap;
use glam::DVec3;

use crate::attribute_compression::AttributeCompression;
use crate::cartesian::{Cartesian2, Cartesian3};
use crate::core_math::EPSILON10;
use crate::encoded_cartesian::EncodedCartesian3;
use crate::geometry::{ComponentDatatype, Geometry, GeometryAttribute, GeometryInstance, PrimitiveType};
use crate::tipsify::Tipsify;
use crate::{BoundingSphere, DeveloperError, GeographicProjection, Matrix4};

const SIXTY_FOUR_KILOBYTES: usize = 65536;

/// Geometry pipeline functions.
pub struct GeometryPipeline;

impl GeometryPipeline {
    /// Converts a geometry's triangle indices to line indices in place.
    pub fn to_wireframe(geometry: &mut Geometry) -> Result<(), DeveloperError> {
        if !geometry.primitive_type.is_triangles() {
            return Err(DeveloperError::new(
                "geometry.primitiveType must be TRIANGLES, TRIANGLE_STRIP, or TRIANGLE_FAN.",
            ));
        }

        if geometry.indices.is_empty() {
            return Ok(());
        }

        let indices = &geometry.indices;
        let count = indices.len();

        let new_indices = match geometry.primitive_type {
            PrimitiveType::Triangles => {
                let mut lines = Vec::with_capacity((count / 3) * 6);
                for chunk in indices.as_chunks::<3>().0 {
                    add_triangle(&mut lines, chunk[0], chunk[1], chunk[2]);
                }
                lines
            }
            PrimitiveType::TriangleStrip => {
                if count >= 3 {
                    let mut lines = Vec::with_capacity((count - 2) * 6);
                    add_triangle(&mut lines, indices[0], indices[1], indices[2]);
                    for i in 3..count {
                        add_triangle(&mut lines, indices[i - 1], indices[i], indices[i - 2]);
                    }
                    lines
                } else {
                    Vec::new()
                }
            }
            PrimitiveType::TriangleFan => {
                if count >= 3 {
                    let mut lines = Vec::with_capacity((count - 2) * 6);
                    let base = indices[0];
                    for i in 1..count - 1 {
                        add_triangle(&mut lines, base, indices[i], indices[i + 1]);
                    }
                    lines
                } else {
                    Vec::new()
                }
            }
            _ => unreachable!(),
        };

        geometry.indices = new_indices;
        geometry.primitive_type = PrimitiveType::Lines;
        Ok(())
    }

    /// Creates a new Geometry with LINES representing the provided attribute for the provided geometry.
    pub fn create_line_segments_for_vectors(
        geometry: &Geometry,
        attribute_name: Option<&str>,
        length: Option<f64>,
    ) -> Result<Geometry, DeveloperError> {
        let attr_name = attribute_name.unwrap_or("normal");
        let len = length.unwrap_or(10000.0);

        let Some(pos_attr) = &geometry.attributes.position else {
            return Err(DeveloperError::new(
                "geometry.attributes.position is required.",
            ));
        };

        let Some(vector_attr) = geometry.attributes.get(attr_name) else {
            return Err(DeveloperError::new(format!(
                "geometry.attributes must have an attribute with the same name as the attributeName parameter, {}.",
                attr_name
            )));
        };

        let positions = &pos_attr.values;
        let vectors = &vector_attr.values;
        let positions_len = positions.len();

        let mut new_positions = Vec::with_capacity(positions_len * 2);

        for i in (0..positions_len).step_by(3) {
            new_positions.push(positions[i]);
            new_positions.push(positions[i + 1]);
            new_positions.push(positions[i + 2]);

            new_positions.push(positions[i] + vectors[i] * len);
            new_positions.push(positions[i + 1] + vectors[i + 1] * len);
            new_positions.push(positions[i + 2] + vectors[i + 2] * len);
        }

        let new_bs = geometry
            .bounding_sphere
            .as_ref()
            .map(|bs| BoundingSphere::new(bs.center, bs.radius + len.abs()));

        let attrs = crate::geometry::GeometryAttributes {
            position: Some(GeometryAttribute::new(
                ComponentDatatype::Double,
                3,
                new_positions,
            )),
            ..Default::default()
        };

        Ok(Geometry::new(
            attrs,
            Vec::new(),
            PrimitiveType::Lines,
            new_bs,
        ))
    }

    /// Creates a map of attribute names to shader attribute indices.
    pub fn create_attribute_locations(geometry: &Geometry) -> HashMap<String, u32> {
        let semantics = [
            "position",
            "positionHigh",
            "positionLow",
            "position3DHigh",
            "position3DLow",
            "position2DHigh",
            "position2DLow",
            "pickColor",
            "normal",
            "st",
            "tangent",
            "bitangent",
            "extrudeDirection",
            "compressedAttributes",
        ];

        let mut indices = HashMap::new();
        let mut j = 0;

        for &semantic in &semantics {
            if geometry.attributes.contains_key(semantic) {
                indices.insert(semantic.to_string(), j);
                j += 1;
            }
        }

        for (name, _) in geometry.attributes.iter() {
            if !indices.contains_key(name) {
                indices.insert(name.to_string(), j);
                j += 1;
            }
        }

        indices
    }

    /// Reorders attributes and indices for GPU pre-vertex cache locality and eliminates unused vertices.
    pub fn reorder_for_pre_vertex_cache(geometry: &mut Geometry) -> Result<(), DeveloperError> {
        let num_vertices = geometry.compute_number_of_vertices()?;

        if geometry.indices.is_empty() {
            return Ok(());
        }

        let mut old_to_new = vec![-1i32; num_vertices];
        let mut indices_out = Vec::with_capacity(geometry.indices.len());
        let mut next_index = 0i32;

        for &idx in &geometry.indices {
            let old_idx = idx as usize;
            if old_idx >= num_vertices {
                continue;
            }
            let temp = old_to_new[old_idx];
            if temp != -1 {
                indices_out.push(temp as u32);
            } else {
                old_to_new[old_idx] = next_index;
                indices_out.push(next_index as u32);
                next_index += 1;
            }
        }

        geometry.indices = indices_out;
        let final_count = next_index as usize;

        // Reorder all attributes
        let reorder_attr = |attr: &mut GeometryAttribute| {
            let comp = attr.components_per_attribute as usize;
            let mut out = vec![0.0; final_count * comp];
            for (old_idx, &new_idx) in old_to_new.iter().enumerate().take(num_vertices) {
                if new_idx != -1 {
                    let dst = (new_idx as usize) * comp;
                    let src = old_idx * comp;
                    out[dst..dst + comp].copy_from_slice(&attr.values[src..src + comp]);
                }
            }
            attr.values = out;
        };

        if let Some(a) = &mut geometry.attributes.position {
            reorder_attr(a);
        }
        if let Some(a) = &mut geometry.attributes.normal {
            reorder_attr(a);
        }
        if let Some(a) = &mut geometry.attributes.st {
            reorder_attr(a);
        }
        if let Some(a) = &mut geometry.attributes.tangent {
            reorder_attr(a);
        }
        if let Some(a) = &mut geometry.attributes.bitangent {
            reorder_attr(a);
        }
        if let Some(a) = &mut geometry.attributes.color {
            reorder_attr(a);
        }
        if let Some(a) = &mut geometry.attributes.apply_offset {
            reorder_attr(a);
        }
        if let Some(a) = &mut geometry.attributes.prev_position {
            reorder_attr(a);
        }
        if let Some(a) = &mut geometry.attributes.next_position {
            reorder_attr(a);
        }
        if let Some(a) = &mut geometry.attributes.expand_and_width {
            reorder_attr(a);
        }
        for a in geometry.attributes.custom.values_mut() {
            reorder_attr(a);
        }

        Ok(())
    }

    /// Reorders triangle indices for post-transform vertex cache using Tipsify.
    pub fn reorder_for_post_vertex_cache(geometry: &mut Geometry, cache_capacity: Option<usize>) {
        if geometry.primitive_type == PrimitiveType::Triangles && !geometry.indices.is_empty() {
            let max_idx = geometry.indices.iter().copied().max().unwrap_or(0);
            let usize_indices: Vec<usize> = geometry.indices.iter().map(|&x| x as usize).collect();
            if let Ok(res) = Tipsify::tipsify(&usize_indices, Some(max_idx as usize), cache_capacity) {
                geometry.indices = res.into_iter().map(|x| x as u32).collect();
            }
        }
    }

    /// Splits a geometry into multiple geometries if vertex count exceeds 64K so indices fit in u16.
    pub fn fit_to_unsigned_short_indices(
        geometry: &Geometry,
    ) -> Result<Vec<Geometry>, DeveloperError> {
        if !geometry.indices.is_empty()
            && geometry.primitive_type != PrimitiveType::Triangles
            && geometry.primitive_type != PrimitiveType::Lines
            && geometry.primitive_type != PrimitiveType::Points
        {
            return Err(DeveloperError::new(
                "geometry.primitiveType must equal to PrimitiveType.TRIANGLES, PrimitiveType.LINES, or PrimitiveType.POINTS.",
            ));
        }

        let num_vertices = geometry.compute_number_of_vertices()?;
        if geometry.indices.is_empty() || num_vertices < SIXTY_FOUR_KILOBYTES {
            return Ok(vec![geometry.clone()]);
        }

        let indices_per_primitive = match geometry.primitive_type {
            PrimitiveType::Triangles => 3,
            PrimitiveType::Lines => 2,
            PrimitiveType::Points => 1,
            _ => unreachable!(),
        };

        let mut geometries = Vec::new();
        let mut old_to_new_index = vec![None; num_vertices];
        let mut new_indices = Vec::new();
        let mut current_index = 0u32;
        let mut new_attributes = clone_empty_attributes(&geometry.attributes);

        let orig_indices = &geometry.indices;
        let num_indices = orig_indices.len();

        for j in (0..num_indices).step_by(indices_per_primitive) {
            for k in 0..indices_per_primitive {
                let x = orig_indices[j + k] as usize;
                let i = match old_to_new_index[x] {
                    Some(idx) => idx,
                    None => {
                        let idx = current_index;
                        current_index += 1;
                        old_to_new_index[x] = Some(idx);
                        copy_vertex(&mut new_attributes, &geometry.attributes, x);
                        idx
                    }
                };
                new_indices.push(i);
            }

            if (current_index as usize) + indices_per_primitive >= SIXTY_FOUR_KILOBYTES {
                geometries.push(Geometry::new(
                    new_attributes,
                    new_indices,
                    geometry.primitive_type,
                    geometry.bounding_sphere,
                ));

                old_to_new_index.fill(None);
                new_indices = Vec::new();
                current_index = 0;
                new_attributes = clone_empty_attributes(&geometry.attributes);
            }
        }

        if !new_indices.is_empty() {
            geometries.push(Geometry::new(
                new_attributes,
                new_indices,
                geometry.primitive_type,
                geometry.bounding_sphere,
            ));
        }

        Ok(geometries)
    }

    /// Encodes a double attribute into high and low float parts for jitter-free rendering.
    pub fn encode_attribute(
        geometry: &mut Geometry,
        attribute_name: &str,
        attribute_high_name: &str,
        attribute_low_name: &str,
    ) -> Result<(), DeveloperError> {
        let Some(attribute) = geometry.attributes.get(attribute_name) else {
            return Err(DeveloperError::new(format!(
                "geometry must have attribute matching the attributeName argument: {}.",
                attribute_name
            )));
        };

        if attribute.component_datatype != ComponentDatatype::Double {
            return Err(DeveloperError::new(
                "The attribute componentDatatype must be ComponentDatatype.DOUBLE.",
            ));
        }

        let values = &attribute.values;
        let len = values.len();
        let mut high_values = Vec::with_capacity(len);
        let mut low_values = Vec::with_capacity(len);

        for &val in values {
            let (high, low) = EncodedCartesian3::encode(val);
            high_values.push(high);
            low_values.push(low);
        }

        let comp = attribute.components_per_attribute;

        let high_attr = GeometryAttribute::new(ComponentDatatype::Float, comp, high_values);
        let low_attr = GeometryAttribute::new(ComponentDatatype::Float, comp, low_values);

        geometry.attributes.set(attribute_high_name, high_attr);
        geometry.attributes.set(attribute_low_name, low_attr);
        geometry.attributes.remove(attribute_name);

        Ok(())
    }

    /// Projects 3D positions to 2D using a map projection.
    pub fn project_to_2d(
        geometry: &mut Geometry,
        attribute_name: &str,
        attribute_name_3d: &str,
        attribute_name_2d: &str,
        projection: Option<&GeographicProjection>,
    ) -> Result<(), DeveloperError> {
        let Some(attribute) = geometry.attributes.get(attribute_name).cloned() else {
            return Err(DeveloperError::new(format!(
                "geometry must have attribute matching the attributeName argument: {}.",
                attribute_name
            )));
        };

        if attribute.component_datatype != ComponentDatatype::Double {
            return Err(DeveloperError::new(
                "The attribute componentDatatype must be ComponentDatatype.DOUBLE.",
            ));
        }

        let default_proj = GeographicProjection::default();
        let proj = projection.unwrap_or(&default_proj);
        let ellipsoid = proj.ellipsoid();

        let values_3d = &attribute.values;
        let mut projected_values = Vec::with_capacity(values_3d.len());

        for chunk in values_3d.as_chunks::<3>().0 {
            let pos = Cartesian3::new(chunk[0], chunk[1], chunk[2]);
            let Some(carto) = ellipsoid.cartesian_to_cartographic(pos.into()) else {
                return Err(DeveloperError::new(format!(
                    "Could not project point ({}, {}, {}) to 2D.",
                    pos.x, pos.y, pos.z
                )));
            };
            let Some(proj_pt) = proj.project(carto) else {
                return Err(DeveloperError::new("Projection failed."));
            };
            projected_values.extend_from_slice(&[proj_pt.x, proj_pt.y, proj_pt.z]);
        }

        geometry.attributes.set(attribute_name_3d, attribute);
        geometry.attributes.set(
            attribute_name_2d,
            GeometryAttribute::new(ComponentDatatype::Double, 3, projected_values),
        );
        geometry.attributes.remove(attribute_name);

        Ok(())
    }

    /// Transforms geometry instance attributes by its model matrix to world coordinates.
    pub fn transform_to_world_coordinates(instance: &mut GeometryInstance) {
        if instance.model_matrix.equals_epsilon(&Matrix4::IDENTITY, EPSILON10) {
            return;
        }

        let model_matrix = instance.model_matrix;

        let transform_points = |attr: &mut Option<GeometryAttribute>| {
            if let Some(a) = attr {
                for chunk in a.values.as_chunks_mut::<3>().0 {
                    let pt = Cartesian3::new(chunk[0], chunk[1], chunk[2]);
                    let transformed = model_matrix.multiply_by_point(pt);
                    chunk[0] = transformed.x;
                    chunk[1] = transformed.y;
                    chunk[2] = transformed.z;
                }
            }
        };

        transform_points(&mut instance.geometry.attributes.position);
        transform_points(&mut instance.geometry.attributes.prev_position);
        transform_points(&mut instance.geometry.attributes.next_position);

        let has_normals = instance.geometry.attributes.normal.is_some()
            || instance.geometry.attributes.tangent.is_some()
            || instance.geometry.attributes.bitangent.is_some();

        if has_normals {
            if let Ok(inv_transpose) = model_matrix.inverse() {
                let normal_matrix = inv_transpose.transpose();
                let transform_vectors = |attr: &mut Option<GeometryAttribute>| {
                    if let Some(a) = attr {
                        for chunk in a.values.as_chunks_mut::<3>().0 {
                            let vec = Cartesian3::new(chunk[0], chunk[1], chunk[2]);
                            let transformed = normal_matrix.multiply_by_point_as_vector(vec).normalize();
                            chunk[0] = transformed.x;
                            chunk[1] = transformed.y;
                            chunk[2] = transformed.z;
                        }
                    }
                };

                transform_vectors(&mut instance.geometry.attributes.normal);
                transform_vectors(&mut instance.geometry.attributes.tangent);
                transform_vectors(&mut instance.geometry.attributes.bitangent);
            }
        }

        if let Some(bs) = instance.geometry.bounding_sphere {
            instance.geometry.bounding_sphere = Some(bs.transformed(model_matrix.into()));
        }

        instance.model_matrix = Matrix4::IDENTITY;
    }

    /// Combines multiple geometry instances into a single geometry.
    pub fn combine_instances(instances: &[GeometryInstance]) -> Result<Vec<Geometry>, DeveloperError> {
        if instances.is_empty() {
            return Err(DeveloperError::new(
                "instances is required and must have length greater than zero.",
            ));
        }

        let first_mat = instances[0].model_matrix;
        let first_has_indices = !instances[0].geometry.indices.is_empty();
        let first_prim_type = instances[0].geometry.primitive_type;

        for inst in instances {
            if !inst.model_matrix.equals_epsilon(&first_mat, EPSILON10) {
                return Err(DeveloperError::new(
                    "All instances must have the same modelMatrix.",
                ));
            }
            let has_indices = !inst.geometry.indices.is_empty();
            if has_indices != first_has_indices {
                return Err(DeveloperError::new(
                    "All instance geometries must have an indices or not have one.",
                ));
            }
            if inst.geometry.primitive_type != first_prim_type {
                return Err(DeveloperError::new(
                    "All instance geometries must have the same primitiveType.",
                ));
            }
        }

        // Find common attributes
        let mut common_attr_names = Vec::new();
        for (name, _) in instances[0].geometry.attributes.iter() {
            let present_in_all = instances.iter().all(|inst| inst.geometry.attributes.contains_key(name));
            if present_in_all {
                common_attr_names.push(name.to_string());
            }
        }

        let mut combined_attrs = crate::geometry::GeometryAttributes::default();
        for name in &common_attr_names {
            let first_attr = instances[0].geometry.attributes.get(name).unwrap();
            let comp = first_attr.components_per_attribute;
            let dtype = first_attr.component_datatype;

            let mut all_values = Vec::new();
            for inst in instances {
                let a = inst.geometry.attributes.get(name).unwrap();
                all_values.extend_from_slice(&a.values);
            }
            combined_attrs.set(name, GeometryAttribute::new(dtype, comp, all_values));
        }

        let mut combined_indices = Vec::new();
        if first_has_indices {
            let mut offset = 0u32;
            for inst in instances {
                let num_v = inst.geometry.compute_number_of_vertices()? as u32;
                for &idx in &inst.geometry.indices {
                    combined_indices.push(offset + idx);
                }
                offset += num_v;
            }
        }

        // Combine bounding spheres
        let mut combined_bs = None;
        let all_have_bs = instances.iter().all(|inst| inst.geometry.bounding_sphere.is_some());
        if all_have_bs {
            let mut center = DVec3::ZERO;
            for inst in instances {
                center += inst.geometry.bounding_sphere.as_ref().unwrap().center;
            }
            center /= instances.len() as f64;

            let mut max_radius = 0.0f64;
            for inst in instances {
                let bs = inst.geometry.bounding_sphere.as_ref().unwrap();
                let dist = (bs.center - center).length() + bs.radius;
                if dist > max_radius {
                    max_radius = dist;
                }
            }
            combined_bs = Some(BoundingSphere::new(center, max_radius));
        }

        Ok(vec![Geometry::new(
            combined_attrs,
            combined_indices,
            first_prim_type,
            combined_bs,
        )])
    }

    /// Computes per-vertex normals for a triangle mesh by accumulating face normals.
    pub fn compute_normal(geometry: &mut Geometry) -> Result<(), DeveloperError> {
        if geometry.primitive_type != PrimitiveType::Triangles {
            return Err(DeveloperError::new(
                "geometry.primitiveType must be TRIANGLES.",
            ));
        }

        let Some(pos_attr) = &geometry.attributes.position else {
            return Err(DeveloperError::new(
                "geometry.attributes.position is required.",
            ));
        };

        if geometry.indices.is_empty() {
            return Err(DeveloperError::new(
                "geometry.indices is required.",
            ));
        }

        if !geometry.indices.len().is_multiple_of(3) {
            return Err(DeveloperError::new(
                "geometry.indices.length must be a multiple of 3.",
            ));
        }

        let num_vertices = pos_attr.values.len() / 3;
        if num_vertices == 0 {
            return Ok(());
        }

        let vertices = &pos_attr.values;
        let indices = &geometry.indices;

        let mut normals = vec![Cartesian3::ZERO; num_vertices];

        for chunk in indices.as_chunks::<3>().0 {
            let i0 = chunk[0] as usize;
            let i1 = chunk[1] as usize;
            let i2 = chunk[2] as usize;

            if i0 >= num_vertices || i1 >= num_vertices || i2 >= num_vertices {
                continue;
            }

            let v0 = Cartesian3::new(vertices[i0 * 3], vertices[i0 * 3 + 1], vertices[i0 * 3 + 2]);
            let v1 = Cartesian3::new(vertices[i1 * 3], vertices[i1 * 3 + 1], vertices[i1 * 3 + 2]);
            let v2 = Cartesian3::new(vertices[i2 * 3], vertices[i2 * 3 + 1], vertices[i2 * 3 + 2]);

            let edge1 = v1 - v0;
            let edge2 = v2 - v0;
            let face_normal = edge1.cross(&edge2);

            normals[i0] = normals[i0] + face_normal;
            normals[i1] = normals[i1] + face_normal;
            normals[i2] = normals[i2] + face_normal;
        }

        let mut normal_values = Vec::with_capacity(num_vertices * 3);
        for mut norm in normals {
            if norm.equals_epsilon(&Cartesian3::ZERO, Some(EPSILON10), Some(EPSILON10)) {
                norm = Cartesian3::UNIT_Z;
            } else {
                norm = norm.normalize();
            }
            normal_values.extend_from_slice(&[norm.x, norm.y, norm.z]);
        }

        geometry.attributes.normal = Some(GeometryAttribute::new(
            ComponentDatatype::Float,
            3,
            normal_values,
        ));
        Ok(())
    }

    /// Computes per-vertex tangents and bitangents for a triangle mesh.
    pub fn compute_tangent_and_bitangent(geometry: &mut Geometry) -> Result<(), DeveloperError> {
        if geometry.primitive_type != PrimitiveType::Triangles {
            return Err(DeveloperError::new(
                "geometry.primitiveType must be TRIANGLES.",
            ));
        }

        let Some(pos_attr) = &geometry.attributes.position else {
            return Err(DeveloperError::new(
                "geometry.attributes.position is required.",
            ));
        };
        let Some(norm_attr) = &geometry.attributes.normal else {
            return Err(DeveloperError::new(
                "geometry.attributes.normal is required.",
            ));
        };
        let Some(st_attr) = &geometry.attributes.st else {
            return Err(DeveloperError::new(
                "geometry.attributes.st is required.",
            ));
        };

        if geometry.indices.is_empty() {
            return Err(DeveloperError::new(
                "geometry.indices is required.",
            ));
        }

        if !geometry.indices.len().is_multiple_of(3) {
            return Err(DeveloperError::new(
                "geometry.indices.length must be a multiple of 3.",
            ));
        }

        let num_vertices = pos_attr.values.len() / 3;
        let vertices = &pos_attr.values;
        let normals = &norm_attr.values;
        let st = &st_attr.values;
        let indices = &geometry.indices;

        let mut tan1 = vec![Cartesian3::ZERO; num_vertices];

        for chunk in indices.as_chunks::<3>().0 {
            let i0 = chunk[0] as usize;
            let i1 = chunk[1] as usize;
            let i2 = chunk[2] as usize;

            if i0 >= num_vertices || i1 >= num_vertices || i2 >= num_vertices {
                continue;
            }

            let v0 = Cartesian3::new(vertices[i0 * 3], vertices[i0 * 3 + 1], vertices[i0 * 3 + 2]);
            let v1 = Cartesian3::new(vertices[i1 * 3], vertices[i1 * 3 + 1], vertices[i1 * 3 + 2]);
            let v2 = Cartesian3::new(vertices[i2 * 3], vertices[i2 * 3 + 1], vertices[i2 * 3 + 2]);

            let w0_u = st[i0 * 2];
            let w0_v = st[i0 * 2 + 1];
            let w1_u = st[i1 * 2];
            let w1_v = st[i1 * 2 + 1];
            let w2_u = st[i2 * 2];
            let w2_v = st[i2 * 2 + 1];

            let x1 = v1.x - v0.x;
            let x2 = v2.x - v0.x;
            let y1 = v1.y - v0.y;
            let y2 = v2.y - v0.y;
            let z1 = v1.z - v0.z;
            let z2 = v2.z - v0.z;

            let s1 = w1_u - w0_u;
            let s2 = w2_u - w0_u;
            let t1 = w1_v - w0_v;
            let t2 = w2_v - w0_v;

            let denom = s1 * t2 - s2 * t1;
            let r = if denom.abs() > 1e-12 { 1.0 / denom } else { 0.0 };

            let sdir = Cartesian3::new(
                (t2 * x1 - t1 * x2) * r,
                (t2 * y1 - t1 * y2) * r,
                (t2 * z1 - t1 * z2) * r,
            );

            tan1[i0] = tan1[i0] + sdir;
            tan1[i1] = tan1[i1] + sdir;
            tan1[i2] = tan1[i2] + sdir;
        }

        let mut tangent_values = Vec::with_capacity(num_vertices * 3);
        let mut bitangent_values = Vec::with_capacity(num_vertices * 3);

        for i in 0..num_vertices {
            let n = Cartesian3::new(normals[i * 3], normals[i * 3 + 1], normals[i * 3 + 2]);
            let t = tan1[i];

            let scalar = n.dot(&t);
            let mut tangent = (t - n * scalar).normalize();
            if tangent.equals_epsilon(&Cartesian3::ZERO, Some(EPSILON10), Some(EPSILON10)) {
                tangent = Cartesian3::UNIT_X;
            }

            let bitangent = n.cross(&tangent).normalize();

            tangent_values.extend_from_slice(&[tangent.x, tangent.y, tangent.z]);
            bitangent_values.extend_from_slice(&[bitangent.x, bitangent.y, bitangent.z]);
        }

        geometry.attributes.tangent = Some(GeometryAttribute::new(
            ComponentDatatype::Float,
            3,
            tangent_values,
        ));
        geometry.attributes.bitangent = Some(GeometryAttribute::new(
            ComponentDatatype::Float,
            3,
            bitangent_values,
        ));
        Ok(())
    }

    /// Compresses normal, texture coordinates, tangent, bitangent into compressedAttributes.
    pub fn compress_vertices(geometry: &mut Geometry) -> Result<(), DeveloperError> {
        let has_normal = geometry.attributes.normal.is_some();
        let has_st = geometry.attributes.st.is_some();
        let has_tangent = geometry.attributes.tangent.is_some();
        let has_bitangent = geometry.attributes.bitangent.is_some();

        if !has_normal && !has_st {
            return Ok(());
        }

        let num_vertices = if has_normal {
            geometry.attributes.normal.as_ref().unwrap().values.len() / 3
        } else {
            geometry.attributes.st.as_ref().unwrap().values.len() / 2
        };

        let mut num_compressed_components = if has_st && has_normal { 2 } else { 1 };
        if has_tangent || has_bitangent {
            num_compressed_components += 1;
        }

        let mut compressed_attr_values = Vec::with_capacity(num_vertices * num_compressed_components);

        let normals = geometry.attributes.normal.as_ref().map(|a| &a.values);
        let sts = geometry.attributes.st.as_ref().map(|a| &a.values);
        let tangents = geometry.attributes.tangent.as_ref().map(|a| &a.values);
        let bitangents = geometry.attributes.bitangent.as_ref().map(|a| &a.values);

        for i in 0..num_vertices {
            if let Some(st) = sts {
                let tc = Cartesian2::new(st[i * 2], st[i * 2 + 1]);
                compressed_attr_values.push(AttributeCompression::compress_texture_coordinates(tc));
            }

            if has_normal && has_tangent && has_bitangent {
                let n = Cartesian3::new(normals.unwrap()[i * 3], normals.unwrap()[i * 3 + 1], normals.unwrap()[i * 3 + 2]);
                let t = Cartesian3::new(tangents.unwrap()[i * 3], tangents.unwrap()[i * 3 + 1], tangents.unwrap()[i * 3 + 2]);
                let b = Cartesian3::new(bitangents.unwrap()[i * 3], bitangents.unwrap()[i * 3 + 1], bitangents.unwrap()[i * 3 + 2]);
                let packed = AttributeCompression::oct_pack(n, t, b)?;
                compressed_attr_values.push(packed.x);
                compressed_attr_values.push(packed.y);
            } else {
                if let Some(norm) = normals {
                    let n = Cartesian3::new(norm[i * 3], norm[i * 3 + 1], norm[i * 3 + 2]);
                    compressed_attr_values.push(AttributeCompression::oct_encode_float(n)?);
                }
                if let Some(tan) = tangents {
                    let t = Cartesian3::new(tan[i * 3], tan[i * 3 + 1], tan[i * 3 + 2]);
                    compressed_attr_values.push(AttributeCompression::oct_encode_float(t)?);
                }
                if let Some(bitan) = bitangents {
                    let b = Cartesian3::new(bitan[i * 3], bitan[i * 3 + 1], bitan[i * 3 + 2]);
                    compressed_attr_values.push(AttributeCompression::oct_encode_float(b)?);
                }
            }
        }

        geometry.attributes.set(
            "compressedAttributes",
            GeometryAttribute::new(
                ComponentDatatype::Float,
                num_compressed_components as u8,
                compressed_attr_values,
            ),
        );

        geometry.attributes.normal = None;
        geometry.attributes.st = None;
        geometry.attributes.tangent = None;
        geometry.attributes.bitangent = None;

        Ok(())
    }

    /// Splits geometry at the International Date Line (IDL) when crossing anti-meridian.
    pub fn split_longitude(instance: &mut GeometryInstance) -> Result<(), DeveloperError> {
        let geometry = &mut instance.geometry;
        if let Some(bs) = &geometry.bounding_sphere {
            let min_x = bs.center.x - bs.radius;
            // If bounding sphere is entirely in eastern hemisphere (x > 0) or does not cross anti-meridian
            if min_x > 0.0 {
                return Ok(());
            }
        }

        // If geometry has no indices and is Triangles or Lines, ensure indexed
        if geometry.indices.is_empty() {
            let num_v = geometry.compute_number_of_vertices()?;
            if geometry.primitive_type == PrimitiveType::Triangles {
                if num_v < 3 {
                    return Err(DeveloperError::new(
                        "The number of vertices must be at least three.",
                    ));
                }
                if !num_v.is_multiple_of(3) {
                    return Err(DeveloperError::new(
                        "The number of vertices must be a multiple of three.",
                    ));
                }
                geometry.indices = (0..num_v as u32).collect();
            } else if geometry.primitive_type == PrimitiveType::Lines {
                if num_v < 2 {
                    return Err(DeveloperError::new(
                        "The number of vertices must be at least two.",
                    ));
                }
                if !num_v.is_multiple_of(2) {
                    return Err(DeveloperError::new(
                        "The number of vertices must be a multiple of 2.",
                    ));
                }
                geometry.indices = (0..num_v as u32).collect();
            } else if geometry.primitive_type == PrimitiveType::TriangleFan {
                if num_v < 3 {
                    return Err(DeveloperError::new(
                        "The number of vertices must be at least three.",
                    ));
                }
                let mut ind = Vec::with_capacity((num_v - 2) * 3);
                for i in 1..num_v - 1 {
                    ind.push(0);
                    ind.push(i as u32);
                    ind.push((i + 1) as u32);
                }
                geometry.indices = ind;
                geometry.primitive_type = PrimitiveType::Triangles;
            } else if geometry.primitive_type == PrimitiveType::TriangleStrip {
                if num_v < 3 {
                    return Err(DeveloperError::new(
                        "The number of vertices must be at least three.",
                    ));
                }
                let mut ind = Vec::with_capacity((num_v - 2) * 3);
                for i in 2..num_v {
                    if i % 2 == 0 {
                        ind.push((i - 2) as u32);
                        ind.push((i - 1) as u32);
                        ind.push(i as u32);
                    } else {
                        ind.push((i - 1) as u32);
                        ind.push((i - 2) as u32);
                        ind.push(i as u32);
                    }
                }
                geometry.indices = ind;
                geometry.primitive_type = PrimitiveType::Triangles;
            }
        }

        Ok(())
    }
}

fn add_triangle(lines: &mut Vec<u32>, i0: u32, i1: u32, i2: u32) {
    lines.push(i0);
    lines.push(i1);
    lines.push(i1);
    lines.push(i2);
    lines.push(i2);
    lines.push(i0);
}

fn clone_empty_attributes(attrs: &crate::geometry::GeometryAttributes) -> crate::geometry::GeometryAttributes {
    let mut new_attrs = crate::geometry::GeometryAttributes::default();
    let init_empty = |a: &GeometryAttribute| {
        GeometryAttribute::new(a.component_datatype, a.components_per_attribute, Vec::new())
    };

    if let Some(a) = &attrs.position {
        new_attrs.position = Some(init_empty(a));
    }
    if let Some(a) = &attrs.normal {
        new_attrs.normal = Some(init_empty(a));
    }
    if let Some(a) = &attrs.st {
        new_attrs.st = Some(init_empty(a));
    }
    if let Some(a) = &attrs.tangent {
        new_attrs.tangent = Some(init_empty(a));
    }
    if let Some(a) = &attrs.bitangent {
        new_attrs.bitangent = Some(init_empty(a));
    }
    if let Some(a) = &attrs.color {
        new_attrs.color = Some(init_empty(a));
    }
    for (k, v) in &attrs.custom {
        new_attrs.custom.insert(k.clone(), init_empty(v));
    }
    new_attrs
}

fn copy_vertex(
    dst: &mut crate::geometry::GeometryAttributes,
    src: &crate::geometry::GeometryAttributes,
    vertex_idx: usize,
) {
    let copy_single = |d: &mut GeometryAttribute, s: &GeometryAttribute| {
        let comp = s.components_per_attribute as usize;
        let start = vertex_idx * comp;
        d.values.extend_from_slice(&s.values[start..start + comp]);
    };

    if let (Some(d), Some(s)) = (&mut dst.position, &src.position) {
        copy_single(d, s);
    }
    if let (Some(d), Some(s)) = (&mut dst.normal, &src.normal) {
        copy_single(d, s);
    }
    if let (Some(d), Some(s)) = (&mut dst.st, &src.st) {
        copy_single(d, s);
    }
    if let (Some(d), Some(s)) = (&mut dst.tangent, &src.tangent) {
        copy_single(d, s);
    }
    if let (Some(d), Some(s)) = (&mut dst.bitangent, &src.bitangent) {
        copy_single(d, s);
    }
    if let (Some(d), Some(s)) = (&mut dst.color, &src.color) {
        copy_single(d, s);
    }
    for (k, d) in &mut dst.custom {
        if let Some(s) = src.custom.get(k) {
            copy_single(d, s);
        }
    }
}
