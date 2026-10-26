//! Swept 3D polyline volume geometry implementation.
//! Follows Cesium PolylineVolumeGeometry and PolylineVolumeOutlineGeometry:
//! sweeps a 2D cross-section along an ECEF polyline on the WGS84 ellipsoid
//! with ROUNDED, MITERED, or BEVELED corner topology and triangulated end caps.

use crate::entities::{CornerType, Entity};
use cesium_formats::{Material, Primitive, Vertex};
use cesium_math::Cartographic;
use glam::{DMat4, DQuat, DVec3};

fn cart(p: [f64; 3]) -> Result<DVec3, String> {
    if p.iter().any(|v| !v.is_finite()) || p[0].abs() > 180. || p[1].abs() > 85. || p[2].abs() > 1e7
    {
        return Err("polyline volume coordinates out of bounds".into());
    }
    Cartographic::degrees(p[0], p[1], p[2])
        .map(|c| c.to_ecef())
        .map_err(|e| e.to_string())
}

fn geodetic_surface_normal(p: DVec3) -> DVec3 {
    let a = cesium_math::WGS84_A;
    let b = a * (1. - 1. / 298.257223563);
    (p / DVec3::new(a * a, a * a, b * b)).normalize()
}

pub fn anchor(positions: &[[f64; 3]]) -> Result<DVec3, String> {
    let p0 = positions.first().ok_or("at least one position required")?;
    Cartographic::degrees(p0[0], p0[1], p0[2])
        .map(|c| c.to_ecef())
        .map_err(|e| e.to_string())
}

struct Slice {
    center: DVec3,
    left: DVec3,
    up: DVec3,
    scale_x: f64,
}

impl Slice {
    fn vertex(&self, pt: [f64; 2]) -> DVec3 {
        self.center + self.left * (pt[0] * self.scale_x) + self.up * pt[1]
    }
}

fn clean_shape(shape_positions: &[[f64; 2]]) -> Result<(Vec<[f64; 2]>, f64, f64), String> {
    if shape_positions.len() < 3 || shape_positions.len() > 1024 {
        return Err("shape positions must have 3 to 1024 points".into());
    }
    for p in shape_positions {
        if !p[0].is_finite() || !p[1].is_finite() || p[0].abs() > 1e7 || p[1].abs() > 1e7 {
            return Err("shape position values out of bounds".into());
        }
    }
    let mut shape: Vec<[f64; 2]> = Vec::with_capacity(shape_positions.len());
    for &pt in shape_positions {
        if shape.is_empty()
            || (pt[0] - shape.last().unwrap()[0]).abs() > 1e-4
            || (pt[1] - shape.last().unwrap()[1]).abs() > 1e-4
        {
            shape.push(pt);
        }
    }
    if shape.len() > 2
        && (shape[0][0] - shape.last().unwrap()[0]).abs() < 1e-4
        && (shape[0][1] - shape.last().unwrap()[1]).abs() < 1e-4
    {
        shape.pop();
    }
    if shape.len() < 3 {
        return Err("non-degenerate 2D shape required".into());
    }

    let min_x = shape.iter().map(|p| p[0]).fold(f64::INFINITY, f64::min);
    let max_x = shape.iter().map(|p| p[0]).fold(f64::NEG_INFINITY, f64::max);
    let min_y = shape.iter().map(|p| p[1]).fold(f64::INFINITY, f64::min);
    let max_y = shape.iter().map(|p| p[1]).fold(f64::NEG_INFINITY, f64::max);
    let cx = (min_x + max_x) * 0.5;
    let cy = (min_y + max_y) * 0.5;
    let width = (max_x - min_x).max(1.0);
    let height = (max_y - min_y).max(1.0);

    let centered: Vec<[f64; 2]> = shape.iter().map(|p| [p[0] - cx, p[1] - cy]).collect();
    Ok((centered, width, height))
}

fn compute_slices(
    positions: &[[f64; 3]],
    shape_positions: &[[f64; 2]],
    corner_type: CornerType,
) -> Result<(Vec<Slice>, Vec<[f64; 2]>), String> {
    if positions.len() < 2 || positions.len() > 64 {
        return Err("polyline volume positions must contain 2 to 64 points".into());
    }
    let (centered_shape, width, _height) = clean_shape(shape_positions)?;
    let half_width = width * 0.5;

    let pts: Vec<DVec3> = positions
        .iter()
        .map(|&p| cart(p))
        .collect::<Result<Vec<_>, _>>()?;

    if pts.windows(2).any(|w| w[0].distance_squared(w[1]) < 1e-4) {
        return Err("distinct polyline positions required".into());
    }

    let mut slices: Vec<Slice> = Vec::new();
    let num_pts = pts.len();

    for seg in 0..num_pts - 1 {
        let p0_deg = positions[seg];
        let p1_deg = positions[seg + 1];
        let p0 = pts[seg];
        let p1 = pts[seg + 1];

        let forward = (p1 - p0).normalize();

        let deg_dist = ((p1_deg[0] - p0_deg[0]).powi(2) + (p1_deg[1] - p0_deg[1]).powi(2)).sqrt();
        let subdivisions = (deg_dist / 1.0).ceil().max(1.0) as usize;

        let start_step = if seg == 0 { 0 } else { 1 };
        let end_step = if seg == num_pts - 2 {
            subdivisions
        } else {
            subdivisions - 1
        };

        for step in start_step..=end_step {
            let t = step as f64 / subdivisions as f64;
            let lon = p0_deg[0] + (p1_deg[0] - p0_deg[0]) * t;
            let lat = p0_deg[1] + (p1_deg[1] - p0_deg[1]) * t;
            let h = p0_deg[2] + (p1_deg[2] - p0_deg[2]) * t;
            let center = cart([lon, lat, h])?;

            let up = geodetic_surface_normal(center);
            let left = up.cross(forward).normalize();
            let true_up = left.cross(forward).normalize();

            slices.push(Slice {
                center,
                left,
                up: true_up,
                scale_x: 1.0,
            });
        }

        if seg < num_pts - 2 {
            let p_prev = pts[seg];
            let p_curr = pts[seg + 1];
            let p_next = pts[seg + 2];

            let f_in = (p_curr - p_prev).normalize();
            let f_out = (p_next - p_curr).normalize();
            let n = geodetic_surface_normal(p_curr);

            let proj_in = (f_in - n * f_in.dot(n)).normalize();
            let proj_out = (f_out - n * f_out.dot(n)).normalize();

            let backward = -proj_in;
            let forward_dir = proj_out;

            let dot = backward.dot(forward_dir);
            if (1.0 - dot.abs()) > 1e-6 {
                let corner_dir = (forward_dir + backward).normalize();
                let sin_half = corner_dir.cross(backward).length().max(0.25);
                let scalar = (1.0 / sin_half).min(4.0);

                let left_in = n.cross(f_in).normalize();
                let up_in = left_in.cross(f_in).normalize();
                let left_out = n.cross(f_out).normalize();
                let up_out = left_out.cross(f_out).normalize();

                match corner_type {
                    CornerType::Mitered => {
                        let miter_left = corner_dir;
                        let miter_up = n;
                        slices.push(Slice {
                            center: p_curr,
                            left: miter_left,
                            up: miter_up,
                            scale_x: scalar,
                        });
                    }
                    CornerType::Beveled => {
                        let offset = (half_width * (scalar - 1.0).max(0.1)).min(half_width * 1.5);
                        let center_a = p_curr - f_in * offset;
                        let center_b = p_curr + f_out * offset;
                        slices.push(Slice {
                            center: center_a,
                            left: left_in,
                            up: up_in,
                            scale_x: 1.0,
                        });
                        slices.push(Slice {
                            center: center_b,
                            left: left_out,
                            up: up_out,
                            scale_x: 1.0,
                        });
                    }
                    CornerType::Rounded => {
                        let angle = (left_in.dot(left_out).clamp(-1.0, 1.0)).acos();
                        let n_sub = (angle / 10.0_f64.to_radians()).ceil().max(2.0) as usize;
                        let rot_axis = if backward.cross(forward_dir).dot(n) >= 0.0 {
                            -n
                        } else {
                            n
                        };
                        for i in 0..=n_sub {
                            let frac = i as f64 / n_sub as f64;
                            let q = DQuat::from_axis_angle(rot_axis, angle * frac);
                            let cur_left = q * left_in;
                            let cur_up = q * up_in;
                            slices.push(Slice {
                                center: p_curr,
                                left: cur_left.normalize(),
                                up: cur_up.normalize(),
                                scale_x: 1.0,
                            });
                        }
                    }
                }
            }
        }
    }

    Ok((slices, centered_shape))
}

pub fn primitive(
    e: &Entity,
    positions: &[[f64; 3]],
    shape_positions: &[[f64; 2]],
    corner_type: CornerType,
) -> Result<Primitive, String> {
    let origin = anchor(positions)?;
    let (slices, shape) = compute_slices(positions, shape_positions, corner_type)?;
    if slices.len() < 2 {
        return Err("at least two slices required for polyline volume".into());
    }

    let k = shape.len();
    let num_slices = slices.len();

    let flat: Vec<f64> = shape.iter().flat_map(|p| [p[0], p[1]]).collect();
    let cap_indices = earcutr::earcut(&flat, &[], 2).map_err(|_| "earcut triangulation failed")?;

    let mut vertices: Vec<Vertex> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();

    let start_slice = &slices[0];
    let start_n = -start_slice.left.cross(start_slice.up).normalize();
    let start_base = vertices.len() as u32;
    for &pt in &shape {
        let wp = start_slice.vertex(pt) - origin;
        vertices.push(Vertex {
            position: wp.as_vec3().to_array(),
            normal: start_n.as_vec3().to_array(),
            uv: [0.0, 0.0],
            uv1: [0.0, 0.0],
            color: [1.0; 4],
            tangent: [1.0, 0.0, 0.0, 1.0],
            feature_id: 0,
        });
    }
    for chunk in cap_indices.as_chunks::<3>().0 {
        indices.extend([
            start_base + chunk[0] as u32,
            start_base + chunk[2] as u32,
            start_base + chunk[1] as u32,
        ]);
    }

    for s in 0..num_slices - 1 {
        let s0 = &slices[s];
        let s1 = &slices[s + 1];
        let u0 = s as f32 / (num_slices - 1) as f32;
        let u1 = (s + 1) as f32 / (num_slices - 1) as f32;

        for j in 0..k {
            let next_j = (j + 1) % k;
            let p00 = s0.vertex(shape[j]) - origin;
            let p01 = s0.vertex(shape[next_j]) - origin;
            let p11 = s1.vertex(shape[next_j]) - origin;
            let p10 = s1.vertex(shape[j]) - origin;

            let face_n = (p01 - p00).cross(p10 - p00).normalize();
            let v0 = j as f32 / k as f32;
            let v1 = (j + 1) as f32 / k as f32;

            let base = vertices.len() as u32;
            vertices.push(Vertex {
                position: p00.as_vec3().to_array(),
                normal: face_n.as_vec3().to_array(),
                uv: [u0, v0],
                uv1: [0.0, 0.0],
                color: [1.0; 4],
                tangent: [1.0, 0.0, 0.0, 1.0],
                feature_id: 0,
            });
            vertices.push(Vertex {
                position: p01.as_vec3().to_array(),
                normal: face_n.as_vec3().to_array(),
                uv: [u0, v1],
                uv1: [0.0, 0.0],
                color: [1.0; 4],
                tangent: [1.0, 0.0, 0.0, 1.0],
                feature_id: 0,
            });
            vertices.push(Vertex {
                position: p11.as_vec3().to_array(),
                normal: face_n.as_vec3().to_array(),
                uv: [u1, v1],
                uv1: [0.0, 0.0],
                color: [1.0; 4],
                tangent: [1.0, 0.0, 0.0, 1.0],
                feature_id: 0,
            });
            vertices.push(Vertex {
                position: p10.as_vec3().to_array(),
                normal: face_n.as_vec3().to_array(),
                uv: [u1, v0],
                uv1: [0.0, 0.0],
                color: [1.0; 4],
                tangent: [1.0, 0.0, 0.0, 1.0],
                feature_id: 0,
            });

            indices.extend([base, base + 1, base + 2, base, base + 2, base + 3]);
        }
    }

    let end_slice = &slices[num_slices - 1];
    let end_n = end_slice.left.cross(end_slice.up).normalize();
    let end_base = vertices.len() as u32;
    for &pt in &shape {
        let wp = end_slice.vertex(pt) - origin;
        vertices.push(Vertex {
            position: wp.as_vec3().to_array(),
            normal: end_n.as_vec3().to_array(),
            uv: [1.0, 1.0],
            uv1: [0.0, 0.0],
            color: [1.0; 4],
            tangent: [1.0, 0.0, 0.0, 1.0],
            feature_id: 0,
        });
    }
    for chunk in cap_indices.as_chunks::<3>().0 {
        indices.extend([
            end_base + chunk[0] as u32,
            end_base + chunk[1] as u32,
            end_base + chunk[2] as u32,
        ]);
    }

    let mut p = Primitive {
        vertices,
        indices,
        transform: DMat4::from_translation(origin),
        material: Material {
            base_color: e.color,
            alpha_blend: e.color[3] < 1.0,
            metallic: 0.0,
            double_sided: true,
            ..Default::default()
        },
    };

    crate::entity_material::apply(e, &mut p)?;
    e.apply_surface_appearance(&mut p);
    Ok(p)
}

pub fn outline(
    e: &Entity,
    positions: &[[f64; 3]],
    shape_positions: &[[f64; 2]],
    corner_type: CornerType,
) -> Result<Primitive, String> {
    let origin = anchor(positions)?;
    let (slices, shape) = compute_slices(positions, shape_positions, corner_type)?;
    if slices.len() < 2 {
        return Err("at least two slices required for outline".into());
    }

    let k = shape.len();
    let num_slices = slices.len();

    let mut vertices: Vec<Vertex> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();

    let mut add_line = |a: DVec3, b: DVec3| {
        let base = vertices.len() as u32;
        vertices.push(Vertex {
            position: (a - origin).as_vec3().to_array(),
            normal: [0.0, 0.0, 1.0],
            uv: [0.0; 2],
            uv1: [0.0; 2],
            tangent: [0.0; 4],
            color: [1.0; 4],
            feature_id: 0,
        });
        vertices.push(Vertex {
            position: (b - origin).as_vec3().to_array(),
            normal: [0.0, 0.0, 1.0],
            uv: [0.0; 2],
            uv1: [0.0; 2],
            tangent: [0.0; 4],
            color: [1.0; 4],
            feature_id: 0,
        });
        indices.extend([base, base + 1]);
    };

    let s_start = &slices[0];
    for j in 0..k {
        let p0 = s_start.vertex(shape[j]);
        let p1 = s_start.vertex(shape[(j + 1) % k]);
        add_line(p0, p1);
    }

    let s_end = &slices[num_slices - 1];
    for j in 0..k {
        let p0 = s_end.vertex(shape[j]);
        let p1 = s_end.vertex(shape[(j + 1) % k]);
        add_line(p0, p1);
    }

    for s in 0..num_slices - 1 {
        let s0 = &slices[s];
        let s1 = &slices[s + 1];
        for &pt in &shape {
            let p0 = s0.vertex(pt);
            let p1 = s1.vertex(pt);
            add_line(p0, p1);
        }
    }

    Ok(Primitive {
        vertices,
        indices,
        transform: DMat4::from_translation(origin),
        material: Material {
            base_color: crate::entity_material::linear(e.outline_color),
            alpha_blend: e.outline_color[3] < 1.0,
            unlit: true,
            metallic: 0.0,
            double_sided: true,
            ..Default::default()
        },
    })
}

#[cfg(test)]
mod tests {
    use crate::{entities, outline, primitive_instances};

    #[test]
    fn batch29_polyline_volume_all_four_cases() {
        // 1. Entities: polyline-volume
        let entities = entities::decode(include_bytes!(
            "../../../tests/sandcastle/full/polyline-volume.json"
        ))
        .unwrap();
        assert_eq!(entities.len(), 3);
        for e in &entities {
            let prim = e.primitive().unwrap();
            assert!(!prim.vertices.is_empty());
            assert!(!prim.indices.is_empty());
            if e.outline {
                let out = outline::primitive(e).unwrap().unwrap();
                assert!(!out.vertices.is_empty());
                assert!(!out.indices.is_empty());
            }
        }

        // 2. Primitives: polyline-volume-dev
        let dev_entities = primitive_instances::decode(include_bytes!(
            "../../../tests/sandcastle/full/polyline-volume-dev.primitive.json"
        ))
        .unwrap();
        assert_eq!(dev_entities.len(), 3);
        for e in &dev_entities {
            let prim = e.primitive().unwrap();
            assert!(!prim.vertices.is_empty());
            assert!(!prim.indices.is_empty());
        }

        // 3. Outlines: polyline-volume-outline-dev
        let outline_entities = primitive_instances::decode(include_bytes!(
            "../../../tests/sandcastle/full/polyline-volume-outline-dev.primitive.json"
        ))
        .unwrap();
        assert_eq!(outline_entities.len(), 2);
        for e in &outline_entities {
            assert!(e.outline);
            let out = outline::primitive(e).unwrap().unwrap();
            assert!(!out.vertices.is_empty());
            assert!(!out.indices.is_empty());
        }

        // 4. CZML: czml-polyline-volume
        let (czml_entities, _) = entities::czml(include_bytes!(
            "../../../tests/sandcastle/full/czml-polyline-volume.czml"
        ))
        .unwrap();
        assert_eq!(czml_entities.len(), 2);
        assert_eq!(czml_entities[0].id, "greenBox");
        assert!(czml_entities[0].outline);
        let p_box = czml_entities[0].primitive().unwrap();
        assert!(!p_box.vertices.is_empty());
        let o_box = outline::primitive(&czml_entities[0]).unwrap().unwrap();
        assert!(!o_box.vertices.is_empty());

        assert_eq!(czml_entities[1].id, "blueStar");
        let p_star = czml_entities[1].primitive().unwrap();
        assert!(!p_star.vertices.is_empty());
    }
}
