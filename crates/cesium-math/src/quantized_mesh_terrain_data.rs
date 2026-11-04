use std::collections::HashMap;

use glam::DVec3;

use crate::axis_aligned_bounding_box::AxisAlignedBoundingBox;
use crate::cartesian::Cartesian3;
use crate::errors::DeveloperError;
use crate::fixed_frame_transforms::FixedFrameTransforms;
use crate::heightmap_terrain_data::CreateMeshOptions;
use crate::intersections_2d::Intersections2D;
use crate::matrices::Matrix4;
use crate::oriented_bounding_box::OrientedBoundingBox;
use crate::rectangle::GeoRectangle;
use crate::terrain_encoding::TerrainEncoding;
use crate::terrain_mesh::TerrainMesh;
use crate::tiling::TilingScheme;
use crate::{Cartographic, Sphere};

const MAX_SHORT: f64 = 32767.0;
const HALF_MAX_SHORT: f64 = 16383.0;

/// Construction options for `QuantizedMeshTerrainData`.
#[derive(Debug, Clone, Default)]
pub struct QuantizedMeshTerrainDataOptions {
    pub quantized_vertices: Option<Vec<u16>>,
    pub indices: Option<Vec<u32>>,
    pub minimum_height: Option<f64>,
    pub maximum_height: Option<f64>,
    pub bounding_sphere: Option<Sphere>,
    pub oriented_bounding_box: Option<OrientedBoundingBox>,
    pub horizon_occlusion_point: Option<Cartesian3>,
    pub west_indices: Option<Vec<u32>>,
    pub south_indices: Option<Vec<u32>>,
    pub east_indices: Option<Vec<u32>>,
    pub north_indices: Option<Vec<u32>>,
    pub west_skirt_height: Option<f64>,
    pub south_skirt_height: Option<f64>,
    pub east_skirt_height: Option<f64>,
    pub north_skirt_height: Option<f64>,
    pub child_tile_mask: Option<u8>,
    pub created_by_upsampling: Option<bool>,
    pub encoded_normals: Option<Vec<u8>>,
    pub water_mask: Option<Vec<u8>>,
}

/// Terrain data for a single tile represented as a quantized mesh.
#[derive(Debug, Clone)]
pub struct QuantizedMeshTerrainData {
    pub quantized_vertices: Vec<u16>,
    pub indices: Vec<u32>,
    pub minimum_height: f64,
    pub maximum_height: f64,
    pub bounding_sphere: Sphere,
    pub oriented_bounding_box: Option<OrientedBoundingBox>,
    pub horizon_occlusion_point: Cartesian3,
    pub west_indices: Vec<u32>,
    pub south_indices: Vec<u32>,
    pub east_indices: Vec<u32>,
    pub north_indices: Vec<u32>,
    pub west_skirt_height: f64,
    pub south_skirt_height: f64,
    pub east_skirt_height: f64,
    pub north_skirt_height: f64,
    pub child_tile_mask: u8,
    pub created_by_upsampling: bool,
    pub encoded_normals: Option<Vec<u8>>,
    pub water_mask: Option<Vec<u8>>,
    pub mesh: Option<TerrainMesh>,
    u_values: Vec<u16>,
    v_values: Vec<u16>,
    height_values: Vec<u16>,
}

impl QuantizedMeshTerrainData {
    pub fn new(options: QuantizedMeshTerrainDataOptions) -> Result<Self, DeveloperError> {
        let quantized_vertices = options
            .quantized_vertices
            .ok_or_else(|| DeveloperError::new("options.quantizedVertices is required."))?;
        let indices = options
            .indices
            .ok_or_else(|| DeveloperError::new("options.indices is required."))?;
        let minimum_height = options
            .minimum_height
            .ok_or_else(|| DeveloperError::new("options.minimumHeight is required."))?;
        let maximum_height = options
            .maximum_height
            .ok_or_else(|| DeveloperError::new("options.maximumHeight is required."))?;
        let bounding_sphere = options
            .bounding_sphere
            .ok_or_else(|| DeveloperError::new("options.boundingSphere is required."))?;
        let horizon_occlusion_point = options
            .horizon_occlusion_point
            .ok_or_else(|| DeveloperError::new("options.horizonOcclusionPoint is required."))?;
        let west_indices = options
            .west_indices
            .ok_or_else(|| DeveloperError::new("options.westIndices is required."))?;
        let south_indices = options
            .south_indices
            .ok_or_else(|| DeveloperError::new("options.southIndices is required."))?;
        let east_indices = options
            .east_indices
            .ok_or_else(|| DeveloperError::new("options.eastIndices is required."))?;
        let north_indices = options
            .north_indices
            .ok_or_else(|| DeveloperError::new("options.northIndices is required."))?;
        let west_skirt_height = options
            .west_skirt_height
            .ok_or_else(|| DeveloperError::new("options.westSkirtHeight is required."))?;
        let south_skirt_height = options
            .south_skirt_height
            .ok_or_else(|| DeveloperError::new("options.southSkirtHeight is required."))?;
        let east_skirt_height = options
            .east_skirt_height
            .ok_or_else(|| DeveloperError::new("options.eastSkirtHeight is required."))?;
        let north_skirt_height = options
            .north_skirt_height
            .ok_or_else(|| DeveloperError::new("options.northSkirtHeight is required."))?;

        let vertex_count = quantized_vertices.len() / 3;
        let u_values = quantized_vertices[0..vertex_count].to_vec();
        let v_values = quantized_vertices[vertex_count..2 * vertex_count].to_vec();
        let height_values = quantized_vertices[2 * vertex_count..3 * vertex_count].to_vec();

        Ok(Self {
            quantized_vertices,
            indices,
            minimum_height,
            maximum_height,
            bounding_sphere,
            oriented_bounding_box: options.oriented_bounding_box,
            horizon_occlusion_point,
            west_indices,
            south_indices,
            east_indices,
            north_indices,
            west_skirt_height,
            south_skirt_height,
            east_skirt_height,
            north_skirt_height,
            child_tile_mask: options.child_tile_mask.unwrap_or(15),
            created_by_upsampling: options.created_by_upsampling.unwrap_or(false),
            encoded_normals: options.encoded_normals,
            water_mask: options.water_mask,
            mesh: None,
            u_values,
            v_values,
            height_values,
        })
    }

    pub fn was_created_by_upsampling(&self) -> bool {
        self.created_by_upsampling
    }

    pub fn child_tile_mask(&self) -> u8 {
        self.child_tile_mask
    }

    pub fn minimum_height(&self) -> f64 {
        self.minimum_height
    }

    pub fn maximum_height(&self) -> f64 {
        self.maximum_height
    }

    pub fn bounding_sphere(&self) -> Sphere {
        self.bounding_sphere
    }

    pub fn horizon_occlusion_point(&self) -> Cartesian3 {
        self.horizon_occlusion_point
    }

    pub fn u_values(&self) -> &[u16] {
        &self.u_values
    }

    pub fn v_values(&self) -> &[u16] {
        &self.v_values
    }

    pub fn height_values(&self) -> &[u16] {
        &self.height_values
    }

    pub fn indices(&self) -> &[u32] {
        &self.indices
    }

    pub fn encoded_normals(&self) -> Option<&[u8]> {
        self.encoded_normals.as_deref()
    }

    pub fn west_indices(&self) -> &[u32] {
        &self.west_indices
    }

    pub fn south_indices(&self) -> &[u32] {
        &self.south_indices
    }

    pub fn east_indices(&self) -> &[u32] {
        &self.east_indices
    }

    pub fn north_indices(&self) -> &[u32] {
        &self.north_indices
    }

    pub fn is_child_available(
        &self,
        this_x: Option<u32>,
        this_y: Option<u32>,
        child_x: Option<u32>,
        child_y: Option<u32>,
    ) -> Result<bool, DeveloperError> {
        let this_x = this_x.ok_or_else(|| DeveloperError::new("thisX is required."))?;
        let this_y = this_y.ok_or_else(|| DeveloperError::new("thisY is required."))?;
        let child_x = child_x.ok_or_else(|| DeveloperError::new("childX is required."))?;
        let child_y = child_y.ok_or_else(|| DeveloperError::new("childY is required."))?;

        let mut bit_number = 2; // northwest
        if child_x != this_x * 2 {
            bit_number += 1; // east
        }
        if child_y != this_y * 2 {
            bit_number -= 2; // south
        }

        Ok((self.child_tile_mask & (1 << bit_number)) != 0)
    }

    pub fn interpolate_height(
        &self,
        rectangle: &GeoRectangle,
        longitude: f64,
        latitude: f64,
    ) -> Option<f64> {
        let width = rectangle.east - rectangle.west;
        let height = rectangle.north - rectangle.south;
        if width <= 0.0 || height <= 0.0 {
            return None;
        }

        let u_norm = ((longitude - rectangle.west) / width).clamp(0.0, 1.0);
        let v_norm = ((latitude - rectangle.south) / height).clamp(0.0, 1.0);
        let u = u_norm * MAX_SHORT;
        let v = v_norm * MAX_SHORT;

        let u_buf = &self.u_values;
        let v_buf = &self.v_values;
        let h_buf = &self.height_values;

        for chunk in self.indices.as_chunks::<3>().0 {
            let i0 = chunk[0] as usize;
            let i1 = chunk[1] as usize;
            let i2 = chunk[2] as usize;

            if i0 >= u_buf.len() || i1 >= u_buf.len() || i2 >= u_buf.len() {
                continue;
            }

            let u0 = u_buf[i0] as f64;
            let u1 = u_buf[i1] as f64;
            let u2 = u_buf[i2] as f64;

            let v0 = v_buf[i0] as f64;
            let v1 = v_buf[i1] as f64;
            let v2 = v_buf[i2] as f64;

            let min_u = u0.min(u1).min(u2);
            let max_u = u0.max(u1).max(u2);
            let min_v = v0.min(v1).min(v2);
            let max_v = v0.max(v1).max(v2);

            if u >= min_u && u <= max_u && v >= min_v && v <= max_v {
                let bary = Intersections2D::compute_barycentric_coordinates(u, v, u0, v0, u1, v1, u2, v2);
                if bary.x >= -1e-10 && bary.y >= -1e-10 && bary.z >= -1e-10 {
                    let q_height = bary.x * (h_buf[i0] as f64)
                        + bary.y * (h_buf[i1] as f64)
                        + bary.z * (h_buf[i2] as f64);
                    let h_lerp = self.minimum_height
                        + (self.maximum_height - self.minimum_height) * (q_height / MAX_SHORT);
                    return Some(h_lerp);
                }
            }
        }

        None
    }

    pub fn create_mesh<T: TilingScheme>(
        &mut self,
        options: CreateMeshOptions<'_, T>,
    ) -> Result<Option<&TerrainMesh>, DeveloperError> {
        let tiling_scheme = options.tiling_scheme;
        let ellipsoid = tiling_scheme.ellipsoid();
        let rectangle = tiling_scheme
            .tile_rectangle(options.x, options.y, options.level)
            .ok_or_else(|| DeveloperError::new("Failed to compute tile rectangle"))?;

        let quantized_vertex_count = self.u_values.len();
        let center = Cartesian3::from(self.bounding_sphere.center);

        let from_enu = FixedFrameTransforms::east_north_up_to_fixed_frame(center, Some(ellipsoid))
            .unwrap_or(Matrix4::IDENTITY);
        let to_enu = from_enu.inverse_transformation().unwrap_or(Matrix4::IDENTITY);

        let mut positions = Vec::with_capacity(quantized_vertex_count);
        let mut heights = Vec::with_capacity(quantized_vertex_count);
        let mut min_pt = DVec3::splat(f64::MAX);
        let mut max_pt = DVec3::splat(f64::MIN);

        for i in 0..quantized_vertex_count {
            let u = (self.u_values[i] as f64) / MAX_SHORT;
            let v = (self.v_values[i] as f64) / MAX_SHORT;
            let h = self.minimum_height + (self.maximum_height - self.minimum_height) * ((self.height_values[i] as f64) / MAX_SHORT);

            let lon = rectangle.west + u * (rectangle.east - rectangle.west);
            let lat = rectangle.south + v * (rectangle.north - rectangle.south);

            let carto = Cartographic::new(lon, lat, h);
            let pos = Cartesian3::from(ellipsoid.cartographic_to_cartesian(carto).unwrap_or(DVec3::ZERO));
            positions.push(pos);
            heights.push(h);

            let enu_pos: DVec3 = DVec3::from(to_enu.multiply_by_point(pos));
            min_pt = min_pt.min(enu_pos);
            max_pt = max_pt.max(enu_pos);
        }

        let aabox = AxisAlignedBoundingBox::new(min_pt, max_pt, Some(center.into()));

        let encoding = TerrainEncoding::new(
            center,
            Some(&aabox),
            Some(self.minimum_height),
            Some(self.maximum_height),
            Some(from_enu),
            self.encoded_normals.is_some(),
            Some(true),
            Some(options.exaggeration != 1.0),
            Some(options.exaggeration),
            Some(options.exaggeration_relative_height),
        );

        let stride = encoding.stride;
        let edge_vertex_count = self.west_indices.len()
            + self.south_indices.len()
            + self.east_indices.len()
            + self.north_indices.len();
        let total_vertex_count = quantized_vertex_count + edge_vertex_count;
        let mut vertices = Vec::with_capacity(total_vertex_count * stride);

        // Pack base vertices
        for i in 0..quantized_vertex_count {
            let pos = positions[i];
            let rel = pos - center;
            let u = (self.u_values[i] as f64) / MAX_SHORT;
            let v = (self.v_values[i] as f64) / MAX_SHORT;
            let h = heights[i];

            vertices.push(rel.x as f32);
            vertices.push(rel.y as f32);
            vertices.push(rel.z as f32);
            vertices.push(h as f32);
            vertices.push(u as f32);
            vertices.push(v as f32);
            vertices.extend(std::iter::repeat_n(0.0, stride.saturating_sub(6)));
        }

        // Pack skirt vertices and build skirt triangles
        let mut all_indices = self.indices.clone();
        let mut current_skirt_vertex_idx = quantized_vertex_count as u32;

        let edge_lists = [
            (&self.west_indices, self.west_skirt_height),
            (&self.south_indices, self.south_skirt_height),
            (&self.east_indices, self.east_skirt_height),
            (&self.north_indices, self.north_skirt_height),
        ];

        for (edge_indices, skirt_height) in edge_lists {
            for &orig_idx in edge_indices.iter() {
                let pos = positions[orig_idx as usize];
                let normal = pos.normalize();
                let skirt_pos = pos - normal * skirt_height;
                let rel = skirt_pos - center;
                let u = (self.u_values[orig_idx as usize] as f64) / MAX_SHORT;
                let v = (self.v_values[orig_idx as usize] as f64) / MAX_SHORT;
                let h = heights[orig_idx as usize] - skirt_height;

                vertices.push(rel.x as f32);
                vertices.push(rel.y as f32);
                vertices.push(rel.z as f32);
                vertices.push(h as f32);
                vertices.push(u as f32);
                vertices.push(v as f32);
                vertices.extend(std::iter::repeat_n(0.0, stride.saturating_sub(6)));
            }

            // Generate skirt quads between adjacent edge vertices
            for k in 0..edge_indices.len().saturating_sub(1) {
                let v0 = edge_indices[k];
                let v1 = edge_indices[k + 1];
                let s0 = current_skirt_vertex_idx + (k as u32);
                let s1 = current_skirt_vertex_idx + ((k + 1) as u32);

                all_indices.push(v0);
                all_indices.push(s0);
                all_indices.push(v1);

                all_indices.push(v1);
                all_indices.push(s0);
                all_indices.push(s1);
            }
            current_skirt_vertex_idx += edge_indices.len() as u32;
        }

        let mesh = TerrainMesh::new(
            center,
            vertices,
            all_indices,
            self.indices.len(),
            quantized_vertex_count,
            self.minimum_height,
            self.maximum_height,
            rectangle,
            self.bounding_sphere,
            self.horizon_occlusion_point,
            Some(stride),
            self.oriented_bounding_box,
            encoding,
            self.west_indices.clone(),
            self.south_indices.clone(),
            self.east_indices.clone(),
            self.north_indices.clone(),
        );

        self.mesh = Some(mesh);
        Ok(self.mesh.as_ref())
    }

    #[allow(clippy::too_many_arguments)]
    pub fn upsample<T: TilingScheme>(
        &self,
        tiling_scheme: Option<&T>,
        this_x: Option<u32>,
        this_y: Option<u32>,
        this_level: Option<u32>,
        descendant_x: Option<u32>,
        descendant_y: Option<u32>,
        descendant_level: Option<u32>,
    ) -> Result<Self, DeveloperError> {
        let tiling_scheme = tiling_scheme.ok_or_else(|| DeveloperError::new("tilingScheme is required."))?;
        let this_x = this_x.ok_or_else(|| DeveloperError::new("thisX is required."))?;
        let this_y = this_y.ok_or_else(|| DeveloperError::new("thisY is required."))?;
        let this_level = this_level.ok_or_else(|| DeveloperError::new("thisLevel is required."))?;
        let descendant_x = descendant_x.ok_or_else(|| DeveloperError::new("descendantX is required."))?;
        let descendant_y = descendant_y.ok_or_else(|| DeveloperError::new("descendantY is required."))?;
        let descendant_level = descendant_level.ok_or_else(|| DeveloperError::new("descendantLevel is required."))?;

        if descendant_level < this_level || descendant_level - this_level > 1 {
            return Err(DeveloperError::new(
                "Upsampling through more than one level at a time is not currently supported.",
            ));
        }

        let is_east_child = this_x * 2 != descendant_x;
        let is_north_child = this_y * 2 == descendant_y;

        let min_u = if is_east_child { HALF_MAX_SHORT } else { 0.0 };
        let max_u = if is_east_child { MAX_SHORT } else { HALF_MAX_SHORT };
        let min_v = if is_north_child { HALF_MAX_SHORT } else { 0.0 };
        let max_v = if is_north_child { MAX_SHORT } else { HALF_MAX_SHORT };

        let has_normals = self.encoded_normals.is_some();
        let normal_bytes = self.encoded_normals.as_deref().unwrap_or(&[]);

        let mut u_buffer: Vec<f64> = Vec::new();
        let mut v_buffer: Vec<f64> = Vec::new();
        let mut h_buffer: Vec<f64> = Vec::new();
        let mut normal_buffer: Vec<u8> = Vec::new();
        let mut indices: Vec<u32> = Vec::new();

        let mut vertex_map: HashMap<usize, usize> = HashMap::new();
        let mut new_vertex_map: HashMap<String, usize> = HashMap::new();

        let vertex_count = self.u_values.len();
        for i in 0..vertex_count {
            let u = self.u_values[i] as f64;
            let v = self.v_values[i] as f64;

            if ((is_east_child && u >= HALF_MAX_SHORT) || (!is_east_child && u <= HALF_MAX_SHORT))
                && ((is_north_child && v >= HALF_MAX_SHORT) || (!is_north_child && v <= HALF_MAX_SHORT))
            {
                let new_idx = u_buffer.len();
                vertex_map.insert(i, new_idx);
                u_buffer.push(u);
                v_buffer.push(v);
                h_buffer.push(self.height_values[i] as f64);
                if has_normals && i * 2 + 1 < normal_bytes.len() {
                    normal_buffer.push(normal_bytes[i * 2]);
                    normal_buffer.push(normal_bytes[i * 2 + 1]);
                }
            }
        }

        for chunk in self.indices.as_chunks::<3>().0 {
            let i0 = chunk[0] as usize;
            let i1 = chunk[1] as usize;
            let i2 = chunk[2] as usize;

            let u0 = self.u_values[i0] as f64;
            let u1 = self.u_values[i1] as f64;
            let u2 = self.u_values[i2] as f64;

            let clipped_ew = Intersections2D::clip_triangle_at_axis_aligned_threshold(
                HALF_MAX_SHORT,
                is_east_child,
                u0,
                u1,
                u2,
            );

            if clipped_ew.is_empty() {
                continue;
            }

            // Clip EW triangles against NS boundary
            let mut ew_triangles = Vec::new();
            let mut clip_idx = 0;
            while clip_idx < clipped_ew.len() {
                let (v0, next) = ClipVertex::from_clip_result(&clipped_ew, clip_idx, i0, i1, i2);
                clip_idx = next;
                let (v1, next) = ClipVertex::from_clip_result(&clipped_ew, clip_idx, i0, i1, i2);
                clip_idx = next;
                let (v2, next) = ClipVertex::from_clip_result(&clipped_ew, clip_idx, i0, i1, i2);
                clip_idx = next;
                ew_triangles.push((v0, v1, v2));

                if clip_idx < clipped_ew.len() {
                    let (v3, next) = ClipVertex::from_clip_result(&clipped_ew, clip_idx, i0, i1, i2);
                    clip_idx = next;
                    ew_triangles.push((ew_triangles[ew_triangles.len() - 1].0.clone(), ew_triangles[ew_triangles.len() - 1].2.clone(), v3));
                }
            }

            for (tv0, tv1, tv2) in ew_triangles {
                let clipped_ns = Intersections2D::clip_triangle_at_axis_aligned_threshold(
                    HALF_MAX_SHORT,
                    is_north_child,
                    tv0.get_v(&self.v_values),
                    tv1.get_v(&self.v_values),
                    tv2.get_v(&self.v_values),
                );

                if clipped_ns.is_empty() {
                    continue;
                }

                let mut ns_poly_vertices = Vec::new();
                let mut c_idx = 0;
                while c_idx < clipped_ns.len() {
                    let (poly_v, next) = ClipVertex::from_composite_clip_result(&clipped_ns, c_idx, &tv0, &tv1, &tv2);
                    c_idx = next;
                    ns_poly_vertices.push(poly_v);
                }

                let mut poly_indices = Vec::new();
                for pv in &ns_poly_vertices {
                    if let Some(orig) = pv.direct_index {
                        if let Some(&new_i) = vertex_map.get(&orig) {
                            poly_indices.push(new_i as u32);
                            continue;
                        }
                    }

                    let key = pv.key(&self.u_values, &self.v_values);
                    if let Some(&existing_idx) = new_vertex_map.get(&key) {
                        poly_indices.push(existing_idx as u32);
                    } else {
                        let new_idx = u_buffer.len();
                        u_buffer.push(pv.get_u(&self.u_values));
                        v_buffer.push(pv.get_v(&self.v_values));
                        h_buffer.push(pv.get_h(&self.height_values));
                        if has_normals {
                            let (nx, ny) = pv.get_normal(normal_bytes);
                            normal_buffer.push(nx);
                            normal_buffer.push(ny);
                        }
                        new_vertex_map.insert(key, new_idx);
                        poly_indices.push(new_idx as u32);
                    }
                }

                if poly_indices.len() == 3 {
                    indices.push(poly_indices[0]);
                    indices.push(poly_indices[1]);
                    indices.push(poly_indices[2]);
                } else if poly_indices.len() == 4 {
                    indices.push(poly_indices[0]);
                    indices.push(poly_indices[1]);
                    indices.push(poly_indices[2]);

                    indices.push(poly_indices[0]);
                    indices.push(poly_indices[2]);
                    indices.push(poly_indices[3]);
                }
            }
        }

        // Remap coordinates to child range [0, 32767]
        let u_offset = if is_east_child { -MAX_SHORT } else { 0.0 };
        let v_offset = if is_north_child { -MAX_SHORT } else { 0.0 };

        let mut west_indices = Vec::new();
        let mut south_indices = Vec::new();
        let mut east_indices = Vec::new();
        let mut north_indices = Vec::new();

        let mut child_u = Vec::with_capacity(u_buffer.len());
        let mut child_v = Vec::with_capacity(v_buffer.len());
        let mut child_h = Vec::with_capacity(h_buffer.len());

        let mut child_min_height = f64::MAX;
        let mut child_max_height = f64::MIN;

        for i in 0..u_buffer.len() {
            let u_raw = u_buffer[i];
            let v_raw = v_buffer[i];
            let h_raw = h_buffer[i];

            let remapped_u = if u_raw <= min_u {
                west_indices.push(i as u32);
                0.0
            } else if u_raw >= max_u {
                east_indices.push(i as u32);
                MAX_SHORT
            } else {
                (u_raw * 2.0 + u_offset).clamp(0.0, MAX_SHORT)
            };

            let remapped_v = if v_raw <= min_v {
                south_indices.push(i as u32);
                0.0
            } else if v_raw >= max_v {
                north_indices.push(i as u32);
                MAX_SHORT
            } else {
                (v_raw * 2.0 + v_offset).clamp(0.0, MAX_SHORT)
            };

            let height = self.minimum_height + (self.maximum_height - self.minimum_height) * (h_raw / MAX_SHORT);
            if height < child_min_height { child_min_height = height; }
            if height > child_max_height { child_max_height = height; }

            child_u.push(remapped_u.round() as u16);
            child_v.push(remapped_v.round() as u16);
            child_h.push(height);
        }

        if child_min_height > child_max_height {
            child_min_height = self.minimum_height;
            child_max_height = self.maximum_height;
        }

        let height_range = (child_max_height - child_min_height).max(1e-7);
        let mut quantized_child_h = Vec::with_capacity(child_h.len());
        for h in child_h {
            let q = ((h - child_min_height) / height_range * MAX_SHORT).clamp(0.0, MAX_SHORT).round() as u16;
            quantized_child_h.push(q);
        }

        let mut quantized_vertices = Vec::with_capacity(child_u.len() * 3);
        quantized_vertices.extend_from_slice(&child_u);
        quantized_vertices.extend_from_slice(&child_v);
        quantized_vertices.extend_from_slice(&quantized_child_h);

        let dest_rect = tiling_scheme
            .tile_rectangle(descendant_x, descendant_y, descendant_level)
            .ok_or_else(|| DeveloperError::new("Failed to compute destination tile rectangle"))?;
        let ellipsoid = tiling_scheme.ellipsoid();

        let center_carto = Cartographic::new(
            (dest_rect.west + dest_rect.east) * 0.5,
            (dest_rect.south + dest_rect.north) * 0.5,
            (child_min_height + child_max_height) * 0.5,
        );
        let center = Cartesian3::from(ellipsoid.cartographic_to_cartesian(center_carto).unwrap_or(DVec3::ZERO));

        let shortest_skirt = self.west_skirt_height
            .min(self.east_skirt_height)
            .min(self.south_skirt_height)
            .min(self.north_skirt_height);

        let west_skirt_height = if is_east_child { shortest_skirt * 0.5 } else { self.west_skirt_height };
        let south_skirt_height = if is_north_child { shortest_skirt * 0.5 } else { self.south_skirt_height };
        let east_skirt_height = if is_east_child { self.east_skirt_height } else { shortest_skirt * 0.5 };
        let north_skirt_height = if is_north_child { self.north_skirt_height } else { shortest_skirt * 0.5 };

        Self::new(QuantizedMeshTerrainDataOptions {
            quantized_vertices: Some(quantized_vertices),
            indices: Some(indices),
            minimum_height: Some(child_min_height),
            maximum_height: Some(child_max_height),
            bounding_sphere: Some(Sphere::new(center, self.bounding_sphere.radius * 0.5)),
            oriented_bounding_box: None,
            horizon_occlusion_point: Some(self.horizon_occlusion_point),
            west_indices: Some(west_indices),
            south_indices: Some(south_indices),
            east_indices: Some(east_indices),
            north_indices: Some(north_indices),
            west_skirt_height: Some(west_skirt_height),
            south_skirt_height: Some(south_skirt_height),
            east_skirt_height: Some(east_skirt_height),
            north_skirt_height: Some(north_skirt_height),
            child_tile_mask: Some(0),
            created_by_upsampling: Some(true),
            encoded_normals: if has_normals { Some(normal_buffer) } else { None },
            water_mask: self.water_mask.clone(),
        })
    }
}

#[derive(Debug, Clone)]
enum VertexSource {
    Direct(usize),
    Interpolated {
        first: Box<ClipVertex>,
        second: Box<ClipVertex>,
        ratio: f64,
    },
}

#[derive(Debug, Clone)]
struct ClipVertex {
    direct_index: Option<usize>,
    source: VertexSource,
}

impl ClipVertex {
    fn from_direct(index: usize) -> Self {
        Self {
            direct_index: Some(index),
            source: VertexSource::Direct(index),
        }
    }

    fn from_interpolated(first: ClipVertex, second: ClipVertex, ratio: f64) -> Self {
        Self {
            direct_index: None,
            source: VertexSource::Interpolated {
                first: Box::new(first),
                second: Box::new(second),
                ratio,
            },
        }
    }

    fn from_clip_result(
        clip_result: &[f64],
        index: usize,
        i0: usize,
        i1: usize,
        i2: usize,
    ) -> (Self, usize) {
        let tri_indices = [i0, i1, i2];
        if clip_result[index] >= 0.0 {
            let v_idx = tri_indices[clip_result[index] as usize];
            (Self::from_direct(v_idx), index + 1)
        } else {
            let v_idx1 = tri_indices[clip_result[index + 1] as usize];
            let v_idx2 = tri_indices[clip_result[index + 2] as usize];
            let ratio = clip_result[index + 3];
            (
                Self::from_interpolated(
                    Self::from_direct(v_idx1),
                    Self::from_direct(v_idx2),
                    ratio,
                ),
                index + 4,
            )
        }
    }

    fn from_composite_clip_result(
        clip_result: &[f64],
        index: usize,
        v0: &ClipVertex,
        v1: &ClipVertex,
        v2: &ClipVertex,
    ) -> (Self, usize) {
        let parent_vertices = [v0, v1, v2];
        if clip_result[index] >= 0.0 {
            let idx = clip_result[index] as usize;
            (parent_vertices[idx].clone(), index + 1)
        } else {
            let idx1 = clip_result[index + 1] as usize;
            let idx2 = clip_result[index + 2] as usize;
            let ratio = clip_result[index + 3];
            (
                Self::from_interpolated(
                    parent_vertices[idx1].clone(),
                    parent_vertices[idx2].clone(),
                    ratio,
                ),
                index + 4,
            )
        }
    }

    fn get_u(&self, u_buffer: &[u16]) -> f64 {
        match &self.source {
            VertexSource::Direct(i) => u_buffer[*i] as f64,
            VertexSource::Interpolated { first, second, ratio } => {
                let u1 = first.get_u(u_buffer);
                let u2 = second.get_u(u_buffer);
                u1 + (u2 - u1) * ratio
            }
        }
    }

    fn get_v(&self, v_buffer: &[u16]) -> f64 {
        match &self.source {
            VertexSource::Direct(i) => v_buffer[*i] as f64,
            VertexSource::Interpolated { first, second, ratio } => {
                let v1 = first.get_v(v_buffer);
                let v2 = second.get_v(v_buffer);
                v1 + (v2 - v1) * ratio
            }
        }
    }

    fn get_h(&self, h_buffer: &[u16]) -> f64 {
        match &self.source {
            VertexSource::Direct(i) => h_buffer[*i] as f64,
            VertexSource::Interpolated { first, second, ratio } => {
                let h1 = first.get_h(h_buffer);
                let h2 = second.get_h(h_buffer);
                h1 + (h2 - h1) * ratio
            }
        }
    }

    fn get_normal(&self, normal_buffer: &[u8]) -> (u8, u8) {
        match &self.source {
            VertexSource::Direct(i) => {
                if *i * 2 + 1 < normal_buffer.len() {
                    (normal_buffer[*i * 2], normal_buffer[*i * 2 + 1])
                } else {
                    (255, 255)
                }
            }
            VertexSource::Interpolated { first, second, ratio } => {
                let (n1x, n1y) = first.get_normal(normal_buffer);
                let (n2x, n2y) = second.get_normal(normal_buffer);
                let nx = ((n1x as f64) + ((n2x as f64) - (n1x as f64)) * ratio).round() as u8;
                let ny = ((n1y as f64) + ((n2y as f64) - (n1y as f64)) * ratio).round() as u8;
                (nx, ny)
            }
        }
    }

    fn key(&self, u_buffer: &[u16], v_buffer: &[u16]) -> String {
        let u = self.get_u(u_buffer);
        let v = self.get_v(v_buffer);
        format!("{:.1},{:.1}", u * 10.0, v * 10.0)
    }
}
