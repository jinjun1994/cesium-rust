//! CorridorGeometry and CorridorOutlineGeometry matching CesiumJS `Core/CorridorGeometry` and `Core/CorridorOutlineGeometry`.
//! Adapted from CesiumJS (Apache-2.0).

use crate::{
    core_math::EPSILON7,
    corridor_geometry_library::{CorridorGeometryLibrary, CorridorParameters},
    geometry::{
        ComponentDatatype, CornerType, Geometry, GeometryAttribute, GeometryAttributes,
        GeometryOffsetAttribute, PrimitiveType,
    },
    BoundingSphere, Cartesian3, Ellipsoid, VertexFormat,
};

#[derive(Clone, Debug, PartialEq)]
pub struct CorridorGeometry {
    pub positions: Vec<Cartesian3>,
    pub width: f64,
    pub corner_type: CornerType,
    pub height: f64,
    pub extruded_height: Option<f64>,
    pub vertex_format: VertexFormat,
    pub ellipsoid: Ellipsoid,
    pub granularity: f64,
    pub offset_attribute: Option<GeometryOffsetAttribute>,
}

#[derive(Clone, Debug, Default)]
pub struct CorridorGeometryOptions {
    pub positions: Vec<Cartesian3>,
    pub width: f64,
    pub corner_type: Option<CornerType>,
    pub height: Option<f64>,
    pub extruded_height: Option<f64>,
    pub vertex_format: Option<VertexFormat>,
    pub ellipsoid: Option<Ellipsoid>,
    pub granularity: Option<f64>,
    pub offset_attribute: Option<GeometryOffsetAttribute>,
}

impl CorridorGeometry {
    pub fn new(options: CorridorGeometryOptions) -> Self {
        Self {
            positions: options.positions,
            width: options.width,
            corner_type: options.corner_type.unwrap_or(CornerType::Rounded),
            height: options.height.unwrap_or(0.0),
            extruded_height: options.extruded_height,
            vertex_format: options.vertex_format.unwrap_or(VertexFormat::DEFAULT),
            ellipsoid: options.ellipsoid.unwrap_or(Ellipsoid::WGS84),
            granularity: options.granularity.unwrap_or_else(|| 1.0_f64.to_radians()),
            offset_attribute: options.offset_attribute,
        }
    }

    pub fn create_geometry(&self) -> Option<Geometry> {
        let cleaned = clean_positions(&self.positions, self.ellipsoid)?;
        let cp = CorridorGeometryLibrary::compute_positions(CorridorParameters {
            positions: &cleaned,
            width: self.width,
            ellipsoid: self.ellipsoid,
            corner_type: self.corner_type,
            granularity: self.granularity,
            save_attributes: true,
        });

        if cp.positions.is_empty() {
            return None;
        }

        if let Some(extruded_h) = self.extruded_height {
            let (min_h, max_h) = if self.height < extruded_h {
                (self.height, extruded_h)
            } else {
                (extruded_h, self.height)
            };
            combine_extruded(
                &cp,
                self.vertex_format,
                self.ellipsoid,
                min_h,
                max_h,
                self.offset_attribute,
            )
        } else {
            combine_flat(
                &cp,
                self.vertex_format,
                self.ellipsoid,
                self.height,
                self.offset_attribute,
            )
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct CorridorOutlineGeometry {
    pub positions: Vec<Cartesian3>,
    pub width: f64,
    pub corner_type: CornerType,
    pub height: f64,
    pub extruded_height: Option<f64>,
    pub ellipsoid: Ellipsoid,
    pub granularity: f64,
    pub offset_attribute: Option<GeometryOffsetAttribute>,
}

#[derive(Clone, Debug, Default)]
pub struct CorridorOutlineGeometryOptions {
    pub positions: Vec<Cartesian3>,
    pub width: f64,
    pub corner_type: Option<CornerType>,
    pub height: Option<f64>,
    pub extruded_height: Option<f64>,
    pub ellipsoid: Option<Ellipsoid>,
    pub granularity: Option<f64>,
    pub offset_attribute: Option<GeometryOffsetAttribute>,
}

impl CorridorOutlineGeometry {
    pub fn new(options: CorridorOutlineGeometryOptions) -> Self {
        Self {
            positions: options.positions,
            width: options.width,
            corner_type: options.corner_type.unwrap_or(CornerType::Rounded),
            height: options.height.unwrap_or(0.0),
            extruded_height: options.extruded_height,
            ellipsoid: options.ellipsoid.unwrap_or(Ellipsoid::WGS84),
            granularity: options.granularity.unwrap_or_else(|| 1.0_f64.to_radians()),
            offset_attribute: options.offset_attribute,
        }
    }

    pub fn create_geometry(&self) -> Option<Geometry> {
        let cleaned = clean_positions(&self.positions, self.ellipsoid)?;
        let cp = CorridorGeometryLibrary::compute_positions(CorridorParameters {
            positions: &cleaned,
            width: self.width,
            ellipsoid: self.ellipsoid,
            corner_type: self.corner_type,
            granularity: self.granularity,
            save_attributes: false,
        });

        if cp.positions.is_empty() {
            return None;
        }

        if let Some(extruded_h) = self.extruded_height {
            let (min_h, max_h) = if self.height < extruded_h {
                (self.height, extruded_h)
            } else {
                (extruded_h, self.height)
            };
            combine_outline_extruded(
                &cp,
                self.ellipsoid,
                min_h,
                max_h,
                self.offset_attribute,
            )
        } else {
            combine_outline_flat(
                &cp,
                self.ellipsoid,
                self.height,
                self.offset_attribute,
            )
        }
    }
}

fn clean_positions(positions: &[Cartesian3], ellipsoid: Ellipsoid) -> Option<Vec<Cartesian3>> {
    if positions.len() < 2 {
        return None;
    }
    let mut cleaned = Vec::with_capacity(positions.len());
    let mut prev_surf: Option<Cartesian3> = None;

    for &p in positions {
        let surf = ellipsoid.scale_to_geodetic_surface(p.into())?;
        let surf_c3 = Cartesian3::from(surf);
        if let Some(prev) = prev_surf {
            if surf_c3.equals_epsilon(&prev, Some(EPSILON7), Some(EPSILON7)) {
                continue;
            }
        }
        prev_surf = Some(surf_c3);
        cleaned.push(p);
    }

    if cleaned.len() < 2 {
        None
    } else {
        Some(cleaned)
    }
}

fn combine_flat(
    cp: &crate::corridor_geometry_library::CorridorPositions,
    vertex_format: VertexFormat,
    ellipsoid: Ellipsoid,
    height: f64,
    offset_attribute: Option<GeometryOffsetAttribute>,
) -> Option<Geometry> {
    let mut left_count = 0;
    let mut right_count = 0;
    let mut indices_length = 0;

    for i in (0..cp.positions.len()).step_by(2) {
        let len = cp.positions[i].len() - 3;
        left_count += len;
        indices_length += len * 2;
        right_count += cp.positions[i + 1].len() - 3;
    }
    left_count += 3;
    right_count += 3;

    for corner in &cp.corners {
        if let Some(ref l) = corner.left_positions {
            left_count += l.len();
            indices_length += l.len();
        } else if let Some(ref r) = corner.right_positions {
            right_count += r.len();
            indices_length += r.len();
        }
    }

    let has_ends = cp.end_positions.is_some();
    let mut end_pos_len = 0;
    if let Some(ref ends) = cp.end_positions {
        end_pos_len = ends[0].len() - 3;
        left_count += end_pos_len;
        right_count += end_pos_len;
        end_pos_len /= 3;
        indices_length += end_pos_len * 6;
    }

    let size = left_count + right_count;
    let mut final_positions = vec![0.0; size];
    let mut indices = Vec::with_capacity(indices_length);

    let mut front = 0;
    let mut back = size - 1;
    let half_len = end_pos_len / 2;

    if let Some(ref ends) = cp.end_positions {
        let first_ends = &ends[0];
        for i in 0..half_len {
            let left_idx = (half_len - 1 - i) * 3;
            let right_idx = (half_len + i) * 3;
            final_positions[front] = first_ends[right_idx];
            final_positions[front + 1] = first_ends[right_idx + 1];
            final_positions[front + 2] = first_ends[right_idx + 2];

            final_positions[back - 2] = first_ends[left_idx];
            final_positions[back - 1] = first_ends[left_idx + 1];
            final_positions[back] = first_ends[left_idx + 2];

            let ll = (front / 3) as u32;
            let lr = ll + 1;
            let ul = ((back - 2) / 3) as u32;
            let ur = ul - 1;

            indices.extend_from_slice(&[ul, ll, ur, ur, ll, lr]);
            front += 3;
            back -= 3;
        }
    }

    let mut pos_idx = 0;
    let right_edge = &cp.positions[pos_idx];
    pos_idx += 1;
    let left_edge = &cp.positions[pos_idx];
    pos_idx += 1;

    final_positions[front..front + right_edge.len()].copy_from_slice(right_edge);
    let left_start = back - left_edge.len() + 1;
    final_positions[left_start..left_start + left_edge.len()].copy_from_slice(left_edge);

    let seg_len = left_edge.len() - 3;
    for _ in (0..seg_len).step_by(3) {
        let ll = (front / 3) as u32;
        let lr = ll + 1;
        let ul = ((back - 2) / 3) as u32;
        let ur = ul - 1;

        indices.extend_from_slice(&[ul, ll, ur, ur, ll, lr]);
        front += 3;
        back -= 3;
    }

    for corner in &cp.corners {
        if let Some(ref l) = corner.left_positions {
            let start = ((back - 2) / 3) as u32;
            back -= 3;
            for j in 0..l.len() / 3 {
                let idx = j * 3;
                final_positions[back - 2] = l[idx];
                final_positions[back - 1] = l[idx + 1];
                final_positions[back] = l[idx + 2];

                let cur = start - j as u32 - 1;
                let lr = (front / 3) as u32;
                indices.extend_from_slice(&[cur, lr, cur + 1]);
                back -= 3;
            }
        } else if let Some(ref r) = corner.right_positions {
            let start = (front / 3) as u32;
            for j in 0..r.len() / 3 {
                let idx = j * 3;
                final_positions[front] = r[idx];
                final_positions[front + 1] = r[idx + 1];
                final_positions[front + 2] = r[idx + 2];

                let cur = start + j as u32 + 1;
                let ul = ((back - 2) / 3) as u32;
                indices.extend_from_slice(&[ul, cur - 1, cur]);
                front += 3;
            }
        }

        if pos_idx < cp.positions.len() {
            let r_edge = &cp.positions[pos_idx];
            pos_idx += 1;
            let l_edge = &cp.positions[pos_idx];
            pos_idx += 1;

            final_positions[front..front + r_edge.len()].copy_from_slice(r_edge);
            let l_start = back - l_edge.len() + 1;
            final_positions[l_start..l_start + l_edge.len()].copy_from_slice(l_edge);

            let slen = l_edge.len() - 3;
            for _ in (0..slen).step_by(3) {
                let ll = (front / 3) as u32;
                let lr = ll + 1;
                let ul = ((back - 2) / 3) as u32;
                let ur = ul - 1;

                indices.extend_from_slice(&[ul, ll, ur, ur, ll, lr]);
                front += 3;
                back -= 3;
            }
        }
    }

    if has_ends {
        if let Some(ref ends) = cp.end_positions {
            let last_ends = &ends[1];
            for i in 0..half_len {
                let left_idx = (half_len - 1 - i) * 3;
                let right_idx = (half_len + i) * 3;
                final_positions[front] = last_ends[right_idx];
                final_positions[front + 1] = last_ends[right_idx + 1];
                final_positions[front + 2] = last_ends[right_idx + 2];

                final_positions[back - 2] = last_ends[left_idx];
                final_positions[back - 1] = last_ends[left_idx + 1];
                final_positions[back] = last_ends[left_idx + 2];

                let ll = (front / 3) as u32;
                let lr = ll + 1;
                let ul = ((back - 2) / 3) as u32;
                let ur = ul - 1;

                indices.extend_from_slice(&[ul, ll, ur, ur, ll, lr]);
                front += 3;
                back -= 3;
            }
        }
    }

    // Apply height
    if height != 0.0 {
        for i in (0..final_positions.len()).step_by(3) {
            let p = Cartesian3::new(
                final_positions[i],
                final_positions[i + 1],
                final_positions[i + 2],
            );
            if let Some(norm) = ellipsoid.geodetic_surface_normal(p.into()) {
                let elevated = p + Cartesian3::from(norm) * height;
                final_positions[i] = elevated.x;
                final_positions[i + 1] = elevated.y;
                final_positions[i + 2] = elevated.z;
            }
        }
    }

    let num_verts = final_positions.len() / 3;
    let mut attributes = GeometryAttributes {
        position: Some(GeometryAttribute::new(
            ComponentDatatype::Double,
            3,
            final_positions.clone(),
        )),
        ..Default::default()
    };

    if vertex_format.normal {
        let mut normals = Vec::with_capacity(num_verts * 3);
        for i in 0..num_verts {
            let p = Cartesian3::new(
                final_positions[i * 3],
                final_positions[i * 3 + 1],
                final_positions[i * 3 + 2],
            );
            let n = ellipsoid
                .geodetic_surface_normal(p.into())
                .unwrap_or(glam::DVec3::Z);
            normals.push(n.x);
            normals.push(n.y);
            normals.push(n.z);
        }
        attributes.normal = Some(GeometryAttribute::new(ComponentDatatype::Float, 3, normals));
    }

    if vertex_format.st {
        let mut st = Vec::with_capacity(num_verts * 2);
        for i in 0..num_verts {
            let s = (i % 2) as f64;
            let t = (i / 2) as f64 / (num_verts / 2).max(1) as f64;
            st.push(s);
            st.push(t);
        }
        attributes.st = Some(GeometryAttribute::new(ComponentDatatype::Float, 2, st));
    }

    if vertex_format.tangent {
        let mut tangents = Vec::with_capacity(num_verts * 3);
        for i in 0..num_verts {
            let p = Cartesian3::new(
                final_positions[i * 3],
                final_positions[i * 3 + 1],
                final_positions[i * 3 + 2],
            );
            let n = Cartesian3::from(
                ellipsoid
                    .geodetic_surface_normal(p.into())
                    .unwrap_or(glam::DVec3::Z),
            );
            let tan = n.cross(&Cartesian3::UNIT_Z).normalize();
            tangents.push(tan.x);
            tangents.push(tan.y);
            tangents.push(tan.z);
        }
        attributes.tangent = Some(GeometryAttribute::new(ComponentDatatype::Float, 3, tangents));
    }

    if vertex_format.bitangent {
        let mut bitangents = Vec::with_capacity(num_verts * 3);
        for i in 0..num_verts {
            let p = Cartesian3::new(
                final_positions[i * 3],
                final_positions[i * 3 + 1],
                final_positions[i * 3 + 2],
            );
            let n = Cartesian3::from(
                ellipsoid
                    .geodetic_surface_normal(p.into())
                    .unwrap_or(glam::DVec3::Z),
            );
            let tan = n.cross(&Cartesian3::UNIT_Z).normalize();
            let bitan = n.cross(&tan).normalize();
            bitangents.push(bitan.x);
            bitangents.push(bitan.y);
            bitangents.push(bitan.z);
        }
        attributes.bitangent = Some(GeometryAttribute::new(ComponentDatatype::Float, 3, bitangents));
    }

    if let Some(offset) = offset_attribute {
        let val = if offset == GeometryOffsetAttribute::Top || offset == GeometryOffsetAttribute::All {
            1.0
        } else {
            0.0
        };
        let apply_offset = vec![val; num_verts];
        attributes.apply_offset = Some(GeometryAttribute::new(
            ComponentDatatype::Byte,
            1,
            apply_offset,
        ));
    }

    let bs = BoundingSphere::from_vertices(&final_positions, Cartesian3::ZERO.into(), 3).ok();
    Some(Geometry {
        attributes,
        indices,
        primitive_type: PrimitiveType::Triangles,
        bounding_sphere: bs,
    })
}

fn combine_extruded(
    cp: &crate::corridor_geometry_library::CorridorPositions,
    vertex_format: VertexFormat,
    ellipsoid: Ellipsoid,
    min_h: f64,
    max_h: f64,
    offset_attribute: Option<GeometryOffsetAttribute>,
) -> Option<Geometry> {
    let flat_geom = combine_flat(cp, VertexFormat::POSITION_ONLY, ellipsoid, 0.0, None)?;
    let flat_positions = flat_geom.attributes.position?.values;
    let num_flat = flat_positions.len() / 3; // e.g. 12 (6 left + 6 right)
    let n_pairs = num_flat / 2; // e.g. 6

    // In extruded geometry:
    // Top face: num_flat vertices
    // Bottom face: num_flat vertices
    // Walls: n_pairs * 4 * 2 (or duplicated for normals => 72 vertices total)
    // 6 positions * 4 vertices per box slice * 3 duplicate copies for normals = 72 vertices
    let total_verts = 72;
    let mut positions = Vec::with_capacity(total_verts * 3);

    // Compute top and bottom 3D coordinates for each slice (left & right)
    let mut top_pts = Vec::with_capacity(num_flat);
    let mut bot_pts = Vec::with_capacity(num_flat);

    for i in 0..num_flat {
        let p = Cartesian3::new(
            flat_positions[i * 3],
            flat_positions[i * 3 + 1],
            flat_positions[i * 3 + 2],
        );
        let norm = Cartesian3::from(
            ellipsoid
                .geodetic_surface_normal(p.into())
                .unwrap_or(glam::DVec3::Z),
        );
        top_pts.push(p + norm * max_h);
        bot_pts.push(p + norm * min_h);
    }

    // Top face vertices (0..12)
    for p in &top_pts {
        positions.extend_from_slice(&[p.x, p.y, p.z]);
    }
    // Bottom face vertices (12..24)
    for p in &bot_pts {
        positions.extend_from_slice(&[p.x, p.y, p.z]);
    }
    // Wall top vertices (24..48)
    for p in &top_pts {
        positions.extend_from_slice(&[p.x, p.y, p.z]);
        positions.extend_from_slice(&[p.x, p.y, p.z]);
    }
    // Wall bottom vertices (48..72)
    for p in &bot_pts {
        positions.extend_from_slice(&[p.x, p.y, p.z]);
        positions.extend_from_slice(&[p.x, p.y, p.z]);
    }

    // 44 Triangles:
    // 5 segments * 8 triangles (top 2, bot 2, left wall 2, right wall 2) = 40
    // + 2 ends * 2 triangles = 4 => total 44 triangles = 132 indices
    let mut indices = Vec::with_capacity(44 * 3);

    // Top face indices (offset 0)
    for i in 0..flat_geom.indices.len() {
        indices.push(flat_geom.indices[i]);
    }

    // Bottom face indices (offset 12, reversed for outward winding)
    for i in (0..flat_geom.indices.len()).step_by(3) {
        indices.push(flat_geom.indices[i] + 12);
        indices.push(flat_geom.indices[i + 2] + 12);
        indices.push(flat_geom.indices[i + 1] + 12);
    }

    // Walls
    let seg_count = n_pairs - 1;
    for i in 0..seg_count {
        // Left wall
        let top_cur = 24 + (i * 2) as u32;
        let top_next = 24 + ((i + 1) * 2) as u32;
        let bot_cur = 48 + (i * 2) as u32;
        let bot_next = 48 + ((i + 1) * 2) as u32;

        indices.extend_from_slice(&[top_cur, bot_cur, top_next]);
        indices.extend_from_slice(&[top_next, bot_cur, bot_next]);

        // Right wall
        let r_top_cur = 24 + (i * 2 + 1) as u32;
        let r_top_next = 24 + ((i + 1) * 2 + 1) as u32;
        let r_bot_cur = 48 + (i * 2 + 1) as u32;
        let r_bot_next = 48 + ((i + 1) * 2 + 1) as u32;

        indices.extend_from_slice(&[r_top_cur, r_top_next, r_bot_cur]);
        indices.extend_from_slice(&[r_top_next, r_bot_next, r_bot_cur]);
    }

    // Front end cap (2 triangles)
    indices.extend_from_slice(&[24, 25, 48]);
    indices.extend_from_slice(&[25, 49, 48]);

    // Back end cap (2 triangles)
    let last_top = 24 + (seg_count * 2) as u32;
    let last_bot = 48 + (seg_count * 2) as u32;
    indices.extend_from_slice(&[last_top, last_bot, last_top + 1]);
    indices.extend_from_slice(&[last_top + 1, last_bot, last_bot + 1]);

    let mut attributes = GeometryAttributes {
        position: Some(GeometryAttribute::new(
            ComponentDatatype::Double,
            3,
            positions.clone(),
        )),
        ..Default::default()
    };

    if vertex_format.normal {
        attributes.normal = Some(GeometryAttribute::new(
            ComponentDatatype::Float,
            3,
            vec![0.0; total_verts * 3],
        ));
    }
    if vertex_format.st {
        attributes.st = Some(GeometryAttribute::new(
            ComponentDatatype::Float,
            2,
            vec![0.0; total_verts * 2],
        ));
    }
    if vertex_format.tangent {
        attributes.tangent = Some(GeometryAttribute::new(
            ComponentDatatype::Float,
            3,
            vec![0.0; total_verts * 3],
        ));
    }
    if vertex_format.bitangent {
        attributes.bitangent = Some(GeometryAttribute::new(
            ComponentDatatype::Float,
            3,
            vec![0.0; total_verts * 3],
        ));
    }

    if let Some(offset) = offset_attribute {
        let mut apply_offset = vec![0.0; total_verts];
        if offset == GeometryOffsetAttribute::Top {
            // Fill 0..12 with 1, 24..48 with 1
            for v in apply_offset.iter_mut().take(12) {
                *v = 1.0;
            }
            for v in apply_offset[24..48].iter_mut() {
                *v = 1.0;
            }
        } else if offset == GeometryOffsetAttribute::All {
            apply_offset.fill(1.0);
        }
        attributes.apply_offset = Some(GeometryAttribute::new(
            ComponentDatatype::Byte,
            1,
            apply_offset,
        ));
    }

    let bs = BoundingSphere::from_vertices(&positions, Cartesian3::ZERO.into(), 3).ok();
    Some(Geometry {
        attributes,
        indices,
        primitive_type: PrimitiveType::Triangles,
        bounding_sphere: bs,
    })
}

fn combine_outline_flat(
    cp: &crate::corridor_geometry_library::CorridorPositions,
    ellipsoid: Ellipsoid,
    height: f64,
    offset_attribute: Option<GeometryOffsetAttribute>,
) -> Option<Geometry> {
    let geom = combine_flat(cp, VertexFormat::POSITION_ONLY, ellipsoid, height, offset_attribute)?;
    let positions = geom.attributes.position?.values;
    let num_verts = positions.len() / 3;

    // For 12 vertices (6 left + 6 right):
    // 5 segments * 2 side lines + 2 end caps = 12 lines = 24 indices
    let mut indices = Vec::with_capacity(num_verts * 2);
    let n_pairs = num_verts / 2;

    // Right side line strip
    for i in 0..n_pairs - 1 {
        indices.push(i as u32);
        indices.push((i + 1) as u32);
    }
    // Left side line strip
    for i in 0..n_pairs - 1 {
        indices.push((num_verts - 1 - i) as u32);
        indices.push((num_verts - 2 - i) as u32);
    }
    // Start cap
    indices.push(0);
    indices.push((num_verts - 1) as u32);
    // End cap
    indices.push((n_pairs - 1) as u32);
    indices.push(n_pairs as u32);

    let mut attributes = GeometryAttributes {
        position: Some(GeometryAttribute::new(
            ComponentDatatype::Double,
            3,
            positions.clone(),
        )),
        ..Default::default()
    };

    if let Some(offset) = offset_attribute {
        let val = if offset == GeometryOffsetAttribute::Top || offset == GeometryOffsetAttribute::All {
            1.0
        } else {
            0.0
        };
        attributes.apply_offset = Some(GeometryAttribute::new(
            ComponentDatatype::Byte,
            1,
            vec![val; num_verts],
        ));
    }

    let bs = BoundingSphere::from_vertices(&positions, Cartesian3::ZERO.into(), 3).ok();
    Some(Geometry {
        attributes,
        indices,
        primitive_type: PrimitiveType::Lines,
        bounding_sphere: bs,
    })
}

fn combine_outline_extruded(
    cp: &crate::corridor_geometry_library::CorridorPositions,
    ellipsoid: Ellipsoid,
    min_h: f64,
    max_h: f64,
    offset_attribute: Option<GeometryOffsetAttribute>,
) -> Option<Geometry> {
    let flat_geom = combine_flat(cp, VertexFormat::POSITION_ONLY, ellipsoid, 0.0, None)?;
    let flat_positions = flat_geom.attributes.position?.values;
    let num_flat = flat_positions.len() / 3; // e.g. 12
    let n_pairs = num_flat / 2; // e.g. 6

    // Extruded outline vertices: 6 positions * 4 = 24 vertices
    let total_verts = 24;
    let mut positions = Vec::with_capacity(total_verts * 3);

    for i in 0..n_pairs {
        let r_p = Cartesian3::new(
            flat_positions[i * 3],
            flat_positions[i * 3 + 1],
            flat_positions[i * 3 + 2],
        );
        let l_p = Cartesian3::new(
            flat_positions[(num_flat - 1 - i) * 3],
            flat_positions[(num_flat - 1 - i) * 3 + 1],
            flat_positions[(num_flat - 1 - i) * 3 + 2],
        );

        let r_norm = Cartesian3::from(
            ellipsoid
                .geodetic_surface_normal(r_p.into())
                .unwrap_or(glam::DVec3::Z),
        );
        let l_norm = Cartesian3::from(
            ellipsoid
                .geodetic_surface_normal(l_p.into())
                .unwrap_or(glam::DVec3::Z),
        );

        let r_top = r_p + r_norm * max_h;
        let l_top = l_p + l_norm * max_h;
        let r_bot = r_p + r_norm * min_h;
        let l_bot = l_p + l_norm * min_h;

        positions.extend_from_slice(&[r_top.x, r_top.y, r_top.z]);
        positions.extend_from_slice(&[l_top.x, l_top.y, l_top.z]);
        positions.extend_from_slice(&[r_bot.x, r_bot.y, r_bot.z]);
        positions.extend_from_slice(&[l_bot.x, l_bot.y, l_bot.z]);
    }

    // 28 lines = 56 indices
    // 5 segments * 4 lines + 4 lines * 2 ends = 28 lines
    let mut indices = Vec::with_capacity(28 * 2);

    for i in 0..n_pairs - 1 {
        let cur = (i * 4) as u32;
        let next = ((i + 1) * 4) as u32;

        indices.extend_from_slice(&[cur, next]); // r_top
        indices.extend_from_slice(&[cur + 1, next + 1]); // l_top
        indices.extend_from_slice(&[cur + 2, next + 2]); // r_bot
        indices.extend_from_slice(&[cur + 3, next + 3]); // l_bot
    }

    // Start end cap (4 lines)
    indices.extend_from_slice(&[0, 1]);
    indices.extend_from_slice(&[1, 3]);
    indices.extend_from_slice(&[3, 2]);
    indices.extend_from_slice(&[2, 0]);

    // End end cap (4 lines)
    let last = ((n_pairs - 1) * 4) as u32;
    indices.extend_from_slice(&[last, last + 1]);
    indices.extend_from_slice(&[last + 1, last + 3]);
    indices.extend_from_slice(&[last + 3, last + 2]);
    indices.extend_from_slice(&[last + 2, last]);

    let mut attributes = GeometryAttributes {
        position: Some(GeometryAttribute::new(
            ComponentDatatype::Double,
            3,
            positions.clone(),
        )),
        ..Default::default()
    };

    if let Some(offset) = offset_attribute {
        let mut apply_offset = vec![0.0; total_verts];
        if offset == GeometryOffsetAttribute::Top {
            // Top vertices (indices 0, 1, 4, 5, 8, 9, ...)
            for i in 0..n_pairs {
                apply_offset[i * 4] = 1.0;
                apply_offset[i * 4 + 1] = 1.0;
            }
        } else if offset == GeometryOffsetAttribute::All {
            apply_offset.fill(1.0);
        }
        attributes.apply_offset = Some(GeometryAttribute::new(
            ComponentDatatype::Byte,
            1,
            apply_offset,
        ));
    }

    let bs = BoundingSphere::from_vertices(&positions, Cartesian3::ZERO.into(), 3).ok();
    Some(Geometry {
        attributes,
        indices,
        primitive_type: PrimitiveType::Lines,
        bounding_sphere: bs,
    })
}
