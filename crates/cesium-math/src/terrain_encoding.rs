use glam::{DMat4, DVec3};

use crate::attribute_compression::AttributeCompression;
use crate::axis_aligned_bounding_box::AxisAlignedBoundingBox;
use crate::cartesian::{Cartesian2, Cartesian3};
use crate::core_math::clamp;
use crate::ellipsoid::Ellipsoid;
use crate::geometry::ComponentDatatype;
use crate::matrices::Matrix4;
use crate::vertical_exaggeration::VerticalExaggeration;

/// How the vertices of a terrain mesh were compressed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[repr(u8)]
pub enum TerrainQuantization {
    #[default]
    None = 0,
    Bits12 = 1,
}

impl TerrainQuantization {
    pub const NONE: Self = Self::None;
    pub const BITS12: Self = Self::Bits12;
}

/// Description of a vertex attribute within a terrain vertex buffer.
#[derive(Debug, Clone, PartialEq)]
pub struct TerrainAttribute {
    pub index: usize,
    pub component_datatype: ComponentDatatype,
    pub components_per_attribute: usize,
    pub offset_in_bytes: usize,
    pub stride_in_bytes: usize,
}

/// Indices pointing to attribute locations in the terrain vertex buffer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TerrainAttributeLocations {
    pub position_3d_and_height: usize,
    pub texture_coord_and_encoded_normals: usize,
    pub geodetic_surface_normal: usize,
}

/// Data used to quantize and pack a terrain mesh.
#[derive(Debug, Clone, PartialEq)]
pub struct TerrainEncoding {
    pub quantization: TerrainQuantization,
    pub minimum_height: Option<f64>,
    pub maximum_height: Option<f64>,
    pub center: Option<Cartesian3>,
    pub to_scaled_enu: Option<Matrix4>,
    pub from_scaled_enu: Option<Matrix4>,
    pub matrix: Option<Matrix4>,
    pub has_vertex_normals: bool,
    pub has_web_mercator_t: bool,
    pub has_geodetic_surface_normals: bool,
    pub exaggeration: f64,
    pub exaggeration_relative_height: f64,
    pub stride: usize,
    offset_geodetic_surface_normal: usize,
    offset_vertex_normal: usize,
}

impl Default for TerrainEncoding {
    fn default() -> Self {
        Self::new_default()
    }
}

impl TerrainEncoding {
    /// Creates an empty default TerrainEncoding.
    pub fn new_default() -> Self {
        let mut encoding = Self {
            quantization: TerrainQuantization::None,
            minimum_height: None,
            maximum_height: None,
            center: None,
            to_scaled_enu: None,
            from_scaled_enu: None,
            matrix: None,
            has_vertex_normals: false,
            has_web_mercator_t: false,
            has_geodetic_surface_normals: false,
            exaggeration: 1.0,
            exaggeration_relative_height: 0.0,
            stride: 6,
            offset_geodetic_surface_normal: 0,
            offset_vertex_normal: 0,
        };
        encoding.calculate_stride_and_offsets();
        encoding
    }

    /// Constructs a configured TerrainEncoding.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        center: Cartesian3,
        axis_aligned_bounding_box: Option<&AxisAlignedBoundingBox>,
        minimum_height: Option<f64>,
        maximum_height: Option<f64>,
        from_enu: Option<Matrix4>,
        has_vertex_normals: bool,
        has_web_mercator_t: Option<bool>,
        has_geodetic_surface_normals: Option<bool>,
        exaggeration: Option<f64>,
        exaggeration_relative_height: Option<f64>,
    ) -> Self {
        let mut quantization = TerrainQuantization::None;
        let mut to_enu = None;
        let mut from_enu_scaled = None;
        let mut calculated_matrix = None;

        if let (Some(aabox), Some(min_h), Some(max_h), Some(from_enu_mat4)) =
            (axis_aligned_bounding_box, minimum_height, maximum_height, from_enu)
        {
            let from_enu_mat: DMat4 = from_enu_mat4.into();
            let minimum = aabox.minimum;
            let maximum = aabox.maximum;
            let dimensions = DVec3::new(
                maximum.x - minimum.x,
                maximum.y - minimum.y,
                maximum.z - minimum.z,
            );
            let minimum_vec = DVec3::new(minimum.x, minimum.y, minimum.z);
            let h_dim = max_h - min_h;
            let max_dim = dimensions.x.max(dimensions.y).max(dimensions.z).max(h_dim);

            if max_dim < (4096.0 - 1.0) {
                quantization = TerrainQuantization::Bits12;
            } else {
                quantization = TerrainQuantization::None;
            }

            let st = DMat4::from_scale_rotation_translation(dimensions, glam::DQuat::IDENTITY, minimum_vec);
            let inv_scale = DVec3::new(1.0 / dimensions.x, 1.0 / dimensions.y, 1.0 / dimensions.z);
            let inv_st = DMat4::from_scale(inv_scale) * DMat4::from_translation(-minimum_vec);

            let mut mat = from_enu_mat;
            let rtc_offset = from_enu_mat.w_axis.truncate() - DVec3::new(center.x, center.y, center.z);
            mat.w_axis = rtc_offset.extend(1.0);
            calculated_matrix = Some(Matrix4::from(mat * st));

            let inv_from_enu = from_enu_mat.inverse();
            to_enu = Some(Matrix4::from(inv_st * inv_from_enu));
            from_enu_scaled = Some(Matrix4::from(from_enu_mat * st));
        }

        let mut encoding = Self {
            quantization,
            minimum_height,
            maximum_height,
            center: Some(center),
            to_scaled_enu: to_enu,
            from_scaled_enu: from_enu_scaled,
            matrix: calculated_matrix,
            has_vertex_normals,
            has_web_mercator_t: has_web_mercator_t.unwrap_or(false),
            has_geodetic_surface_normals: has_geodetic_surface_normals.unwrap_or(false),
            exaggeration: exaggeration.unwrap_or(1.0),
            exaggeration_relative_height: exaggeration_relative_height.unwrap_or(0.0),
            stride: 0,
            offset_geodetic_surface_normal: 0,
            offset_vertex_normal: 0,
        };

        encoding.calculate_stride_and_offsets();
        encoding
    }

    /// Recalculates the vertex stride and internal component offsets.
    pub fn calculate_stride_and_offsets(&mut self) {
        let mut vertex_stride = match self.quantization {
            TerrainQuantization::Bits12 => 3,
            _ => 6,
        };

        if self.has_web_mercator_t {
            vertex_stride += 1;
        }
        if self.has_vertex_normals {
            self.offset_vertex_normal = vertex_stride;
            vertex_stride += 1;
        }
        if self.has_geodetic_surface_normals {
            self.offset_geodetic_surface_normal = vertex_stride;
            vertex_stride += 3;
        }

        self.stride = vertex_stride;
    }

    /// Encode vertex attributes into the buffer.
    #[allow(clippy::too_many_arguments)]
    pub fn encode(
        &self,
        vertex_buffer: &mut Vec<f32>,
        mut buffer_index: usize,
        position: Cartesian3,
        uv: Cartesian2,
        height: f64,
        normal_to_pack: Option<Cartesian2>,
        web_mercator_t: Option<f64>,
        geodetic_surface_normal: Option<Cartesian3>,
    ) -> usize {
        let needed_len = buffer_index + self.stride;
        if vertex_buffer.len() < needed_len {
            vertex_buffer.resize(needed_len, 0.0);
        }

        let u = uv.x;
        let v = uv.y;

        if self.quantization == TerrainQuantization::Bits12 {
            let to_scaled: DMat4 = self.to_scaled_enu.map(|m| m.into()).unwrap_or(DMat4::IDENTITY);
            let pos_dvec = DVec3::new(position.x, position.y, position.z);
            let p = to_scaled.transform_point3(pos_dvec);

            let px = clamp(p.x, 0.0, 1.0);
            let py = clamp(p.y, 0.0, 1.0);
            let pz = clamp(p.z, 0.0, 1.0);

            let min_h = self.minimum_height.unwrap_or(0.0);
            let max_h = self.maximum_height.unwrap_or(1.0);
            let h_dim = max_h - min_h;
            let h = if h_dim > 0.0 {
                clamp((height - min_h) / h_dim, 0.0, 1.0)
            } else {
                0.0
            };

            let comp0 = AttributeCompression::compress_texture_coordinates(Cartesian2::new(px, py));
            let comp1 = AttributeCompression::compress_texture_coordinates(Cartesian2::new(pz, h));
            let comp2 = AttributeCompression::compress_texture_coordinates(Cartesian2::new(u, v));

            vertex_buffer[buffer_index] = comp0 as f32;
            buffer_index += 1;
            vertex_buffer[buffer_index] = comp1 as f32;
            buffer_index += 1;
            vertex_buffer[buffer_index] = comp2 as f32;
            buffer_index += 1;

            if self.has_web_mercator_t {
                let wm_t = web_mercator_t.unwrap_or(0.0);
                let comp3 = AttributeCompression::compress_texture_coordinates(Cartesian2::new(wm_t, 0.0));
                vertex_buffer[buffer_index] = comp3 as f32;
                buffer_index += 1;
            }
        } else {
            let c = self.center.unwrap_or(Cartesian3::ZERO);
            vertex_buffer[buffer_index] = (position.x - c.x) as f32;
            buffer_index += 1;
            vertex_buffer[buffer_index] = (position.y - c.y) as f32;
            buffer_index += 1;
            vertex_buffer[buffer_index] = (position.z - c.z) as f32;
            buffer_index += 1;
            vertex_buffer[buffer_index] = height as f32;
            buffer_index += 1;
            vertex_buffer[buffer_index] = u as f32;
            buffer_index += 1;
            vertex_buffer[buffer_index] = v as f32;
            buffer_index += 1;

            if self.has_web_mercator_t {
                vertex_buffer[buffer_index] = web_mercator_t.unwrap_or(0.0) as f32;
                buffer_index += 1;
            }
        }

        if self.has_vertex_normals {
            if let Some(norm_2d) = normal_to_pack {
                let packed = AttributeCompression::oct_pack_float(norm_2d);
                vertex_buffer[buffer_index] = packed as f32;
            } else {
                vertex_buffer[buffer_index] = 0.0;
            }
            buffer_index += 1;
        }

        if self.has_geodetic_surface_normals {
            let g_norm = geodetic_surface_normal.unwrap_or(Cartesian3::UNIT_Z);
            vertex_buffer[buffer_index] = g_norm.x as f32;
            buffer_index += 1;
            vertex_buffer[buffer_index] = g_norm.y as f32;
            buffer_index += 1;
            vertex_buffer[buffer_index] = g_norm.z as f32;
            buffer_index += 1;
        }

        buffer_index
    }

    /// Decode the uncompressed position of the vertex at the given index.
    pub fn decode_position(&self, buffer: &[f32], vertex_index: usize) -> Cartesian3 {
        let base = vertex_index * self.stride;

        if self.quantization == TerrainQuantization::Bits12 {
            let xy = AttributeCompression::decompress_texture_coordinates(buffer[base] as f64);
            let zh = AttributeCompression::decompress_texture_coordinates(buffer[base + 1] as f64);

            let local = DVec3::new(xy.x, xy.y, zh.x);
            let from_scaled: DMat4 = self.from_scaled_enu.map(|m| m.into()).unwrap_or(DMat4::IDENTITY);
            let world = from_scaled.transform_point3(local);
            Cartesian3::new(world.x, world.y, world.z)
        } else {
            let c = self.center.unwrap_or(Cartesian3::ZERO);
            Cartesian3::new(
                buffer[base] as f64 + c.x,
                buffer[base + 1] as f64 + c.y,
                buffer[base + 2] as f64 + c.z,
            )
        }
    }

    /// Decode position and apply vertical exaggeration if configured.
    pub fn get_exaggerated_position(&self, buffer: &[f32], vertex_index: usize) -> Cartesian3 {
        let mut pos = self.decode_position(buffer, vertex_index);

        if self.exaggeration != 1.0 && self.has_geodetic_surface_normals {
            let normal = self.decode_geodetic_surface_normal(buffer, vertex_index);
            let raw_height = self.decode_height(buffer, vertex_index);
            let exaggerated_height = VerticalExaggeration::get_height(
                raw_height,
                self.exaggeration,
                self.exaggeration_relative_height,
            );
            let diff = exaggerated_height - raw_height;

            pos.x += normal.x * diff;
            pos.y += normal.y * diff;
            pos.z += normal.z * diff;
        }

        pos
    }

    /// Decode texture coordinates from the vertex buffer.
    pub fn decode_texture_coordinates(&self, buffer: &[f32], vertex_index: usize) -> Cartesian2 {
        let base = vertex_index * self.stride;

        if self.quantization == TerrainQuantization::Bits12 {
            AttributeCompression::decompress_texture_coordinates(buffer[base + 2] as f64)
        } else {
            Cartesian2::new(buffer[base + 4] as f64, buffer[base + 5] as f64)
        }
    }

    /// Decode vertex height from the vertex buffer.
    pub fn decode_height(&self, buffer: &[f32], vertex_index: usize) -> f64 {
        let base = vertex_index * self.stride;

        if self.quantization == TerrainQuantization::Bits12 {
            let zh = AttributeCompression::decompress_texture_coordinates(buffer[base + 1] as f64);
            let min_h = self.minimum_height.unwrap_or(0.0);
            let max_h = self.maximum_height.unwrap_or(1.0);
            zh.y * (max_h - min_h) + min_h
        } else {
            buffer[base + 3] as f64
        }
    }

    /// Decode Web Mercator T coordinate from the vertex buffer.
    pub fn decode_web_mercator_t(&self, buffer: &[f32], vertex_index: usize) -> f64 {
        let base = vertex_index * self.stride;

        if self.quantization == TerrainQuantization::Bits12 {
            AttributeCompression::decompress_texture_coordinates(buffer[base + 3] as f64).x
        } else {
            buffer[base + 6] as f64
        }
    }

    /// Retrieve the oct-encoded normal components (0..255) packed in the vertex.
    pub fn get_oct_encoded_normal(&self, buffer: &[f32], vertex_index: usize) -> Cartesian2 {
        let base = vertex_index * self.stride + self.offset_vertex_normal;
        let temp = (buffer[base] as f64) / 256.0;
        let x = temp.floor();
        let y = (temp - x) * 256.0;
        Cartesian2::new(x, y)
    }

    /// Decode the 3D unit normal vector from the vertex buffer.
    pub fn decode_normal(&self, buffer: &[f32], vertex_index: usize) -> Result<Cartesian3, crate::DeveloperError> {
        let base = vertex_index * self.stride + self.offset_vertex_normal;
        AttributeCompression::oct_decode_float(buffer[base] as f64)
    }

    /// Decode the geodetic surface normal from the vertex buffer.
    pub fn decode_geodetic_surface_normal(&self, buffer: &[f32], vertex_index: usize) -> Cartesian3 {
        let base = vertex_index * self.stride + self.offset_geodetic_surface_normal;
        Cartesian3::new(buffer[base] as f64, buffer[base + 1] as f64, buffer[base + 2] as f64)
    }

    /// Add geodetic surface normals to a terrain vertex buffer.
    pub fn add_geodetic_surface_normals(
        &mut self,
        old_buffer: &[f32],
        new_buffer: &mut Vec<f32>,
        ellipsoid: &Ellipsoid,
    ) {
        if self.has_geodetic_surface_normals {
            new_buffer.clear();
            new_buffer.extend_from_slice(old_buffer);
            return;
        }

        let old_stride = self.stride;
        let vertex_count = old_buffer.len() / old_stride;
        self.has_geodetic_surface_normals = true;
        self.calculate_stride_and_offsets();
        let new_stride = self.stride;

        new_buffer.resize(vertex_count * new_stride, 0.0);

        for index in 0..vertex_count {
            let old_base = index * old_stride;
            let new_base = index * new_stride;

            new_buffer[new_base..(new_base + old_stride)]
                .copy_from_slice(&old_buffer[old_base..(old_base + old_stride)]);

            let position = self.decode_position(new_buffer, index);
            let pos_dvec = DVec3::new(position.x, position.y, position.z);
            let normal = ellipsoid
                .geodetic_surface_normal(pos_dvec)
                .unwrap_or(DVec3::Z);

            let normal_offset = new_base + self.offset_geodetic_surface_normal;
            new_buffer[normal_offset] = normal.x as f32;
            new_buffer[normal_offset + 1] = normal.y as f32;
            new_buffer[normal_offset + 2] = normal.z as f32;
        }
    }

    /// Remove geodetic surface normals from a terrain vertex buffer.
    pub fn remove_geodetic_surface_normals(&mut self, old_buffer: &[f32], new_buffer: &mut Vec<f32>) {
        if !self.has_geodetic_surface_normals {
            new_buffer.clear();
            new_buffer.extend_from_slice(old_buffer);
            return;
        }

        let old_stride = self.stride;
        let vertex_count = old_buffer.len() / old_stride;
        self.has_geodetic_surface_normals = false;
        self.calculate_stride_and_offsets();
        let new_stride = self.stride;

        new_buffer.resize(vertex_count * new_stride, 0.0);

        for index in 0..vertex_count {
            let old_base = index * old_stride;
            let new_base = index * new_stride;

            new_buffer[new_base..(new_base + new_stride)]
                .copy_from_slice(&old_buffer[old_base..(old_base + new_stride)]);
        }
    }

    /// Returns attribute descriptors describing the memory layout in the vertex buffer.
    pub fn get_attributes(&self, _buffer: &[f32]) -> Vec<TerrainAttribute> {
        let datatype = ComponentDatatype::Float;
        let size_in_bytes = datatype.size_in_bytes();
        let stride_in_bytes = self.stride * size_in_bytes;
        let mut offset_in_bytes = 0;
        let mut attributes = Vec::new();

        let mut add_attribute = |index: usize, components_per_attribute: usize| {
            attributes.push(TerrainAttribute {
                index,
                component_datatype: datatype,
                components_per_attribute,
                offset_in_bytes,
                stride_in_bytes,
            });
            offset_in_bytes += components_per_attribute * size_in_bytes;
        };

        if self.quantization == TerrainQuantization::None {
            add_attribute(0, 4);

            let mut components_tex_and_normals = 2;
            if self.has_web_mercator_t {
                components_tex_and_normals += 1;
            }
            if self.has_vertex_normals {
                components_tex_and_normals += 1;
            }
            add_attribute(1, components_tex_and_normals);

            if self.has_geodetic_surface_normals {
                add_attribute(2, 3);
            }
        } else {
            let using_attribute0_component4 = self.has_web_mercator_t || self.has_vertex_normals;
            let using_attribute1_component1 = self.has_web_mercator_t && self.has_vertex_normals;

            add_attribute(0, if using_attribute0_component4 { 4 } else { 3 });

            if using_attribute1_component1 {
                add_attribute(1, 1);
            }

            if self.has_geodetic_surface_normals {
                add_attribute(2, 3);
            }
        }

        attributes
    }

    /// Returns attribute indices used for shaders.
    pub fn get_attribute_locations(&self) -> TerrainAttributeLocations {
        TerrainAttributeLocations {
            position_3d_and_height: 0,
            texture_coord_and_encoded_normals: 1,
            geodetic_surface_normal: 2,
        }
    }

    /// Clones this encoding into another or new instance.
    pub fn clone_into(&self, target: &mut Self) {
        target.quantization = self.quantization;
        target.minimum_height = self.minimum_height;
        target.maximum_height = self.maximum_height;
        target.center = self.center;
        target.to_scaled_enu = self.to_scaled_enu;
        target.from_scaled_enu = self.from_scaled_enu;
        target.matrix = self.matrix;
        target.has_vertex_normals = self.has_vertex_normals;
        target.has_web_mercator_t = self.has_web_mercator_t;
        target.has_geodetic_surface_normals = self.has_geodetic_surface_normals;
        target.exaggeration = self.exaggeration;
        target.exaggeration_relative_height = self.exaggeration_relative_height;
        target.calculate_stride_and_offsets();
    }
}
