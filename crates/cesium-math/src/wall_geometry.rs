//! WallGeometry and WallOutlineGeometry matching CesiumJS `Core/WallGeometry` and `Core/WallOutlineGeometry`.
//! Adapted from CesiumJS (Apache-2.0).

use crate::{
    core_math::{chord_length, EPSILON10, RADIANS_PER_DEGREE},
    geometry::{ComponentDatatype, Geometry, GeometryAttribute, GeometryAttributes, PrimitiveType},
    BoundingSphere, Cartesian3, Ellipsoid, VertexFormat,
};

#[derive(Clone, Debug, PartialEq)]
pub struct WallGeometry {
    pub positions: Vec<Cartesian3>,
    pub granularity: f64,
    pub maximum_heights: Option<Vec<f64>>,
    pub minimum_heights: Option<Vec<f64>>,
    pub ellipsoid: Ellipsoid,
    pub vertex_format: VertexFormat,
}

impl WallGeometry {
    pub fn new(
        positions: Vec<Cartesian3>,
        granularity: Option<f64>,
        maximum_heights: Option<Vec<f64>>,
        minimum_heights: Option<Vec<f64>>,
        ellipsoid: Option<Ellipsoid>,
        vertex_format: Option<VertexFormat>,
    ) -> Self {
        Self {
            positions,
            granularity: granularity.unwrap_or(RADIANS_PER_DEGREE),
            maximum_heights,
            minimum_heights,
            ellipsoid: ellipsoid.unwrap_or(Ellipsoid::WGS84),
            vertex_format: vertex_format.unwrap_or(VertexFormat::DEFAULT),
        }
    }

    pub fn from_constant_heights(
        positions: Vec<Cartesian3>,
        minimum_height: Option<f64>,
        maximum_height: Option<f64>,
        ellipsoid: Option<Ellipsoid>,
        vertex_format: Option<VertexFormat>,
    ) -> Self {
        let len = positions.len();
        let min_h = minimum_height.map(|h| vec![h; len]);
        let max_h = maximum_height.map(|h| vec![h; len]);
        Self::new(positions, None, max_h, min_h, ellipsoid, vertex_format)
    }

    pub fn create_geometry(&self) -> Option<Geometry> {
        let clean = clean_wall_positions(
            &self.positions,
            self.maximum_heights.as_deref(),
            self.minimum_heights.as_deref(),
            self.ellipsoid,
        )?;

        let (subdivided_top, subdivided_bot) = subdivide_wall(
            &clean.positions,
            &clean.top_heights,
            &clean.bottom_heights,
            self.granularity,
            self.ellipsoid,
            true,
        );

        let count = subdivided_top.len();
        if count < 2 {
            return None;
        }

        // Interleave bottom and top positions: [bot0, top0, bot1, top1, ...]
        let mut interleaved_positions = Vec::with_capacity(count * 2 * 3);
        let mut uvs = if self.vertex_format.st {
            Vec::with_capacity(count * 2 * 2)
        } else {
            Vec::new()
        };
        let mut normals = if self.vertex_format.normal {
            Vec::with_capacity(count * 2 * 3)
        } else {
            Vec::new()
        };

        for i in 0..count {
            let bot = subdivided_bot[i];
            let top = subdivided_top[i];

            interleaved_positions.push(bot.x);
            interleaved_positions.push(bot.y);
            interleaved_positions.push(bot.z);

            interleaved_positions.push(top.x);
            interleaved_positions.push(top.y);
            interleaved_positions.push(top.z);

            if self.vertex_format.st {
                let s = i as f64 / (count - 1).max(1) as f64;
                uvs.push(s);
                uvs.push(0.0);
                uvs.push(s);
                uvs.push(1.0);
            }

            if self.vertex_format.normal {
                let n: Cartesian3 = self
                    .ellipsoid
                    .geodetic_surface_normal(top.into())
                    .unwrap_or(Cartesian3::UNIT_Z.into())
                    .into();
                normals.push(n.x);
                normals.push(n.y);
                normals.push(n.z);
                normals.push(n.x);
                normals.push(n.y);
                normals.push(n.z);
            }
        }

        // Triangles: 2 per quad segment
        let num_vertices = count * 2;
        let mut indices = Vec::with_capacity((count - 1) * 6);

        for i in (0..(num_vertices - 2)).step_by(2) {
            let ll = i as u32;
            let ul = (i + 1) as u32;
            let lr = (i + 2) as u32;
            let ur = (i + 3) as u32;

            indices.push(ul);
            indices.push(ll);
            indices.push(ur);

            indices.push(ur);
            indices.push(ll);
            indices.push(lr);
        }

        let mut attributes = GeometryAttributes::default();
        if self.vertex_format.position {
            attributes.position = Some(GeometryAttribute::new(
                ComponentDatatype::Double,
                3,
                interleaved_positions.clone(),
            ));
        }
        if self.vertex_format.st {
            attributes.st = Some(GeometryAttribute::new(
                ComponentDatatype::Float,
                2,
                uvs,
            ));
        }
        if self.vertex_format.normal {
            attributes.normal = Some(GeometryAttribute::new(
                ComponentDatatype::Float,
                3,
                normals,
            ));
        }

        let center = subdivided_top[0];
        Some(Geometry {
            attributes,
            indices,
            primitive_type: PrimitiveType::Triangles,
            bounding_sphere: Some(BoundingSphere::new(center, 1000.0)),
        })
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct WallOutlineGeometry {
    pub positions: Vec<Cartesian3>,
    pub granularity: f64,
    pub maximum_heights: Option<Vec<f64>>,
    pub minimum_heights: Option<Vec<f64>>,
    pub ellipsoid: Ellipsoid,
}

impl WallOutlineGeometry {
    pub fn new(
        positions: Vec<Cartesian3>,
        granularity: Option<f64>,
        maximum_heights: Option<Vec<f64>>,
        minimum_heights: Option<Vec<f64>>,
        ellipsoid: Option<Ellipsoid>,
    ) -> Self {
        Self {
            positions,
            granularity: granularity.unwrap_or(RADIANS_PER_DEGREE),
            maximum_heights,
            minimum_heights,
            ellipsoid: ellipsoid.unwrap_or(Ellipsoid::WGS84),
        }
    }

    pub fn from_constant_heights(
        positions: Vec<Cartesian3>,
        minimum_height: Option<f64>,
        maximum_height: Option<f64>,
        ellipsoid: Option<Ellipsoid>,
    ) -> Self {
        let len = positions.len();
        let min_h = minimum_height.map(|h| vec![h; len]);
        let max_h = maximum_height.map(|h| vec![h; len]);
        Self::new(positions, None, max_h, min_h, ellipsoid)
    }

    pub fn create_geometry(&self) -> Option<Geometry> {
        let clean = clean_wall_positions(
            &self.positions,
            self.maximum_heights.as_deref(),
            self.minimum_heights.as_deref(),
            self.ellipsoid,
        )?;

        let (subdivided_top, subdivided_bot) = subdivide_wall(
            &clean.positions,
            &clean.top_heights,
            &clean.bottom_heights,
            self.granularity,
            self.ellipsoid,
            false,
        );

        let count = subdivided_top.len();
        if count < 2 {
            return None;
        }

        let mut interleaved_positions = Vec::with_capacity(count * 2 * 3);
        for i in 0..count {
            let bot = subdivided_bot[i];
            let top = subdivided_top[i];

            interleaved_positions.push(bot.x);
            interleaved_positions.push(bot.y);
            interleaved_positions.push(bot.z);

            interleaved_positions.push(top.x);
            interleaved_positions.push(top.y);
            interleaved_positions.push(top.z);
        }

        let num_vertices = count * 2;
        let mut indices = Vec::with_capacity(count * 4);

        for i in (0..(num_vertices - 2)).step_by(2) {
            let ll = i as u32;
            let ul = (i + 1) as u32;
            let lr = (i + 2) as u32;
            let ur = (i + 3) as u32;

            indices.push(ul);
            indices.push(ll);

            indices.push(ul);
            indices.push(ur);

            indices.push(ll);
            indices.push(lr);
        }

        indices.push((num_vertices - 2) as u32);
        indices.push((num_vertices - 1) as u32);

        let attributes = GeometryAttributes {
            position: Some(GeometryAttribute::new(
                ComponentDatatype::Double,
                3,
                interleaved_positions,
            )),
            ..Default::default()
        };

        let center = subdivided_top[0];
        Some(Geometry {
            attributes,
            indices,
            primitive_type: PrimitiveType::Lines,
            bounding_sphere: Some(BoundingSphere::new(center, 1000.0)),
        })
    }
}

// ============================================================================
// Helpers
// ============================================================================

struct CleanedWall {
    positions: Vec<Cartesian3>,
    top_heights: Vec<f64>,
    bottom_heights: Vec<f64>,
}

fn clean_wall_positions(
    positions: &[Cartesian3],
    max_heights: Option<&[f64]>,
    min_heights: Option<&[f64]>,
    ellipsoid: Ellipsoid,
) -> Option<CleanedWall> {
    if positions.len() < 2 {
        return None;
    }
    if let Some(max_h) = max_heights {
        if max_h.len() != positions.len() {
            return None;
        }
    }
    if let Some(min_h) = min_heights {
        if min_h.len() != positions.len() {
            return None;
        }
    }

    let mut cleaned_positions = Vec::new();
    let mut cleaned_top_heights = Vec::new();
    let mut cleaned_bot_heights = Vec::new();

    let mut prev_c: Option<(f64, f64)> = None;

    for (i, &p) in positions.iter().enumerate() {
        let carto = ellipsoid.cartesian_to_cartographic(p.into())?;
        let top_h = max_heights.map_or(carto.height, |h| h[i]);
        let bot_h = min_heights.map_or(0.0, |h| h[i]);

        if let Some((prev_lat, prev_lon)) = prev_c {
            let same_lat = (carto.latitude - prev_lat).abs() < EPSILON10;
            let same_lon = (carto.longitude - prev_lon).abs() < EPSILON10;
            if same_lat && same_lon {
                // Duplicate position in lat/lon: update height if greater
                if let Some(last_h) = cleaned_top_heights.last_mut() {
                    if top_h > *last_h {
                        *last_h = top_h;
                    }
                }
                continue;
            }
        }

        prev_c = Some((carto.latitude, carto.longitude));
        cleaned_positions.push(p);
        cleaned_top_heights.push(top_h);
        cleaned_bot_heights.push(bot_h);
    }

    if cleaned_positions.len() < 2 {
        return None;
    }

    // Check if all heights are identical and 0
    let all_zero = cleaned_top_heights.iter().all(|&h| h.abs() < f64::EPSILON)
        && cleaned_bot_heights.iter().all(|&h| h.abs() < f64::EPSILON);
    if all_zero {
        return None;
    }

    Some(CleanedWall {
        positions: cleaned_positions,
        top_heights: cleaned_top_heights,
        bottom_heights: cleaned_bot_heights,
    })
}

fn subdivide_wall(
    positions: &[Cartesian3],
    top_heights: &[f64],
    bot_heights: &[f64],
    granularity: f64,
    ellipsoid: Ellipsoid,
    duplicate_corners: bool,
) -> (Vec<Cartesian3>, Vec<Cartesian3>) {
    let mut out_top = Vec::new();
    let mut out_bot = Vec::new();

    let max_radius = ellipsoid.maximum_radius();
    let min_dist = chord_length(granularity, max_radius).max(1.0);

    let len = positions.len();
    for i in 0..(len - 1) {
        let p0 = positions[i];
        let p1 = positions[i + 1];
        let h0_top = top_heights[i];
        let h1_top = top_heights[i + 1];
        let h0_bot = bot_heights[i];
        let h1_bot = bot_heights[i + 1];

        let dist = p0.distance(&p1);
        let num_pts = (dist / min_dist).ceil().max(1.0) as usize;

        let carto0 = ellipsoid.cartesian_to_cartographic(p0.into()).unwrap();
        let carto1 = ellipsoid.cartesian_to_cartographic(p1.into()).unwrap();

        let end_idx = if duplicate_corners || i == len - 2 {
            num_pts
        } else {
            num_pts - 1
        };

        for step in 0..=end_idx {
            let t = step as f64 / num_pts as f64;
            let lat = carto0.latitude + (carto1.latitude - carto0.latitude) * t;
            let lon = carto0.longitude + (carto1.longitude - carto0.longitude) * t;

            let cur_top_h = h0_top + (h1_top - h0_top) * t;
            let cur_bot_h = h0_bot + (h1_bot - h0_bot) * t;

            let surface_pos = Cartesian3::from_radians(lon, lat, Some(0.0), Some(&ellipsoid)).unwrap();
            let normal: Cartesian3 = ellipsoid
                .geodetic_surface_normal(surface_pos.into())
                .unwrap_or(Cartesian3::UNIT_Z.into())
                .into();

            out_top.push(surface_pos + normal * cur_top_h);
            out_bot.push(surface_pos + normal * cur_bot_h);
        }
    }

    (out_top, out_bot)
}
