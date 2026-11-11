//! PrimitivePipeline matching CesiumJS `Scene/PrimitivePipeline.js` (Apache-2.0).

use std::collections::HashMap;

use cesium_math::{
    BoundingSphere, ComponentDatatype, Geometry, GeometryAttribute, GeometryAttributes,
    PrimitiveType,
};
use glam::DVec3;

/// Result structure of packing geometries for worker/transfer pipelines.
#[derive(Clone, Debug, PartialEq)]
pub struct CreateGeometryResults {
    pub string_table: Vec<String>,
    pub packed_data: Vec<f64>,
}

/// Pipeline utility functions for geometry creation, packing, unpacking, and batching.
pub struct PrimitivePipeline;

impl PrimitivePipeline {
    /// Packs geometries into a flat float64 array and string table, compatible with CesiumJS worker transfer.
    pub fn pack_create_geometry_results(
        items: &[Option<Geometry>],
        transferable_objects: &mut Vec<Vec<u8>>,
    ) -> CreateGeometryResults {
        let mut string_table = Vec::new();
        let mut string_hash: HashMap<String, usize> = HashMap::new();
        let mut packed_data = Vec::new();

        let length = items.len();
        packed_data.push(length as f64);

        for item in items {
            if let Some(geometry) = item {
                packed_data.push(1.0); // valid geometry

                packed_data.push(geometry.primitive_type as u32 as f64);
                packed_data.push(0.0); // geometryType (NONE)
                packed_data.push(-1.0); // offsetAttribute (none)

                if let Some(sphere) = &geometry.bounding_sphere {
                    packed_data.push(1.0);
                    packed_data.push(sphere.center.x);
                    packed_data.push(sphere.center.y);
                    packed_data.push(sphere.center.z);
                    packed_data.push(sphere.radius);
                } else {
                    packed_data.push(0.0);
                    packed_data.extend_from_slice(&[0.0, 0.0, 0.0, 0.0]);
                }

                // boundingSphereCV
                packed_data.push(0.0);
                packed_data.extend_from_slice(&[0.0, 0.0, 0.0, 0.0]);

                let attr_list = geometry.attributes.iter();
                packed_data.push(attr_list.len() as f64);

                for (name, attribute) in attr_list {
                    let str_idx = *string_hash.entry(name.to_string()).or_insert_with(|| {
                        let idx = string_table.len();
                        string_table.push(name.to_string());
                        idx
                    });

                    packed_data.push(str_idx as f64);
                    packed_data.push(attribute.component_datatype as u32 as f64);
                    packed_data.push(attribute.components_per_attribute as f64);
                    packed_data.push(if attribute.normalize { 1.0 } else { 0.0 });
                    packed_data.push(attribute.values.len() as f64);
                    packed_data.extend_from_slice(&attribute.values);
                }

                packed_data.push(geometry.indices.len() as f64);
                for idx in &geometry.indices {
                    packed_data.push(*idx as f64);
                }
            } else {
                packed_data.push(0.0); // invalid
            }
        }

        // Add binary representation to transferable objects
        let byte_slice = bytemuck::cast_slice(&packed_data).to_vec();
        transferable_objects.push(byte_slice);

        CreateGeometryResults {
            string_table,
            packed_data,
        }
    }

    /// Unpacks create geometry results back into geometries.
    pub fn unpack_create_geometry_results(
        results: &CreateGeometryResults,
    ) -> Vec<Option<Geometry>> {
        let string_table = &results.string_table;
        let packed_data = &results.packed_data;

        if packed_data.is_empty() {
            return Vec::new();
        }

        let num_items = packed_data[0] as usize;
        let mut geometries = Vec::with_capacity(num_items);
        let mut idx = 1;

        while idx < packed_data.len() {
            let valid = packed_data[idx] == 1.0;
            idx += 1;

            if !valid {
                geometries.push(None);
                continue;
            }

            let primitive_type_val = packed_data[idx] as u32;
            let primitive_type = match primitive_type_val {
                0 => PrimitiveType::Points,
                1 => PrimitiveType::Lines,
                2 => PrimitiveType::LineLoop,
                3 => PrimitiveType::LineStrip,
                4 => PrimitiveType::Triangles,
                5 => PrimitiveType::TriangleStrip,
                6 => PrimitiveType::TriangleFan,
                _ => PrimitiveType::Triangles,
            };
            idx += 1;

            let _geometry_type = packed_data[idx];
            idx += 1;

            let _offset_attribute = packed_data[idx];
            idx += 1;

            let valid_bounding_sphere = packed_data[idx] == 1.0;
            idx += 1;
            let bounding_sphere = if valid_bounding_sphere {
                let center = DVec3::new(packed_data[idx], packed_data[idx + 1], packed_data[idx + 2]);
                let radius = packed_data[idx + 3];
                Some(BoundingSphere::new(center, radius))
            } else {
                None
            };
            idx += 4;

            let _valid_bounding_sphere_cv = packed_data[idx] == 1.0;
            idx += 1;
            idx += 4; // skip boundingSphereCV

            let num_attributes = packed_data[idx] as usize;
            idx += 1;

            let mut attributes = GeometryAttributes::default();

            for _ in 0..num_attributes {
                let str_idx = packed_data[idx] as usize;
                idx += 1;
                let attr_name = &string_table[str_idx];

                let comp_datatype_val = packed_data[idx] as u32;
                idx += 1;
                let component_datatype = match comp_datatype_val {
                    5120 => ComponentDatatype::Byte,
                    5121 => ComponentDatatype::UnsignedByte,
                    5122 => ComponentDatatype::Short,
                    5123 => ComponentDatatype::UnsignedShort,
                    5125 => ComponentDatatype::UnsignedInt,
                    5126 => ComponentDatatype::Float,
                    5130 => ComponentDatatype::Double,
                    _ => ComponentDatatype::Float,
                };

                let components_per_attribute = packed_data[idx] as u8;
                idx += 1;

                let normalize = packed_data[idx] == 1.0;
                idx += 1;

                let values_len = packed_data[idx] as usize;
                idx += 1;

                let values = packed_data[idx..idx + values_len].to_vec();
                idx += values_len;

                let attr = GeometryAttribute {
                    component_datatype,
                    components_per_attribute,
                    normalize,
                    values,
                };

                match attr_name.as_str() {
                    "position" => attributes.position = Some(attr),
                    "normal" => attributes.normal = Some(attr),
                    "st" => attributes.st = Some(attr),
                    "tangent" => attributes.tangent = Some(attr),
                    "bitangent" => attributes.bitangent = Some(attr),
                    "color" => attributes.color = Some(attr),
                    _ => {
                        attributes.custom.insert(attr_name.to_string(), attr);
                    }
                }
            }

            let indices_len = packed_data[idx] as usize;
            idx += 1;

            let indices = if indices_len > 0 {
                let ind = packed_data[idx..idx + indices_len]
                    .iter()
                    .map(|v| *v as u32)
                    .collect();
                idx += indices_len;
                ind
            } else {
                Vec::new()
            };

            geometries.push(Some(Geometry::new(
                attributes,
                indices,
                primitive_type,
                bounding_sphere,
            )));
        }

        geometries
    }
}
