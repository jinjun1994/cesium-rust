//! Real one-pixel line-list outlines; no triangle wireframe or metre-width substitution.
use crate::entities::{Entity, Shape};
use cesium_formats::{Material, Primitive, Vertex};
use cesium_math::Cartographic;
use glam::{DMat4, DVec3};
const STEP: f64 = 0.1;
pub fn primitive(e: &Entity) -> Result<Option<Primitive>, String> {
    if !e.outline {
        return Ok(None);
    }
    if matches!(e.shape, Shape::LineSet { .. }) {
        return e.primitive().map(Some);
    }
    // Public callers can construct Entity directly; validate before indexing any rings.
    e.primitive()?;
    if let Shape::PolygonSurface { data } = &e.shape {
        return crate::polygon_surface::outline(e, data).map(Some);
    }
    if e.outline_width != 1. {
        return Err("only one-pixel outlineWidth is supported".into());
    }
    let mut p = Primitive {
        vertices: vec![],
        indices: vec![],
        transform: DMat4::IDENTITY,
        material: Material {
            base_color: crate::entity_material::linear(e.outline_color),
            alpha_blend: e.outline_color[3] < 1.,
            unlit: true,
            metallic: 0.,
            double_sided: true,
            ..Default::default()
        },
    };
    let origin = if e.shape.standalone() {
        crate::geographic::anchor(&e.shape)?
    } else if e.shape.geographic() {
        let c = e.position_degrees.ok_or("outline center")?;
        Cartographic::degrees(c[0], c[1], c[2])
            .map_err(|e| e.to_string())?
            .to_ecef()
    } else {
        DVec3::ZERO
    };
    let mut line = |a: DVec3, b: DVec3| -> Result<(), String> {
        if a.distance_squared(b) < 1e-12 {
            return Ok(());
        }
        if p.vertices.len() >= 100000 {
            return Err("outline vertex budget".into());
        }
        let start = p.vertices.len() as u32;
        for v in [a, b] {
            p.vertices.push(Vertex {
                position: v.as_vec3().to_array(),
                normal: [0., 0., 1.],
                uv: [0.; 2],
                uv1: [0.; 2],
                tangent: [0.; 4],
                color: [1.; 4],
                feature_id: 0,
            });
        }
        p.indices.extend([start, start + 1]);
        Ok(())
    };
    let geo = |c: [f64; 3]| {
        Cartographic::degrees(c[0], c[1], c[2] + 10.5)
            .map(|c| c.to_ecef() - origin)
            .map_err(|e| e.to_string())
    };
    let mut path = |coords: &[[f64; 3]], closed: bool| -> Result<(), String> {
        let mut coords = coords.to_vec();
        if closed && coords.first() != coords.last() {
            coords.push(coords[0]);
        }
        for pair in coords.windows(2) {
            let a = pair[0];
            let mut b = pair[1];
            b[0] = a[0] + (b[0] - a[0] + 180.).rem_euclid(360.) - 180.;
            let n = (((b[0] - a[0]).abs().max((b[1] - a[1]).abs()) / STEP).ceil() as usize).max(1);
            let point = |t: f64| {
                geo([
                    a[0] + (b[0] - a[0]) * t,
                    a[1] + (b[1] - a[1]) * t,
                    a[2] + (b[2] - a[2]) * t,
                ])
            };
            for i in 0..n {
                line(
                    point(i as f64 / n as f64)?,
                    point((i + 1) as f64 / n as f64)?,
                )?;
            }
        }
        Ok(())
    };
    match &e.shape {
        Shape::WidePolyline { .. } => return Err("wide polyline uses material outline".into()),
        Shape::Corridor {
            extruded_height, ..
        } => {
            let boundary = crate::corridor::boundary(&e.shape)?;
            path(&boundary.ring, true)?;
            if let Some(h) = extruded_height {
                let bottom: Vec<_> = boundary.ring.iter().map(|c| [c[0], c[1], *h]).collect();
                path(&bottom, true)?;
                for c in boundary.vertical {
                    path(&[c, [c[0], c[1], *h]], false)?;
                }
            }
        }
        Shape::PolygonSurface { data } => {
            for points in crate::polygon_surface::outline_paths(data)? {
                path(&points, false)?;
            }
        }
        Shape::PolylineVolume {
            positions,
            shape_positions,
            corner_type,
        } => {
            return crate::polyline_volume::outline(e, positions, shape_positions, *corner_type)
                .map(Some);
        }
        Shape::LineSet { .. } => return e.primitive().map(Some),
        Shape::Polygon {
            positions,
            holes,
            height,
            extruded_height,
            ..
        } => {
            let mut pos = positions.clone();
            if let Some(h) = height {
                for v in &mut pos {
                    if v[2] == 0. {
                        v[2] = *h;
                    }
                }
            }
            for ring in std::iter::once(&pos).chain(holes) {
                path(ring, true)?;
                if let Some(h) = extruded_height {
                    let bottom: Vec<_> = ring.iter().map(|c| [c[0], c[1], *h]).collect();
                    path(&bottom, true)?;
                    for c in ring {
                        path(&[*c, [c[0], c[1], *h]], false)?;
                    }
                }
            }
        }
        Shape::Rectangle {
            coordinates: [w, s, e, n],
            height,
            extruded_height,
            rotation,
            ..
        } => {
            let is_full = (*w == -180. && *e == 180.) || (*e - *w).abs() >= 359.999;
            let width = if is_full {
                360.
            } else {
                (e - w).rem_euclid(360.)
            };
            let center_lon = crate::geographic::unwrap(w + width / 2., 0.);
            let center_lat = (s + n) / 2.;
            let mut ring = Vec::new();
            if is_full || width > 180. {
                let steps = 36;
                for i in 0..steps {
                    let lon = *w + (width * i as f64) / steps as f64;
                    ring.push([lon, *s, *height]);
                }
                for i in 0..steps {
                    let lon = *e - (width * i as f64) / steps as f64;
                    ring.push([lon, *n, *height]);
                }
            } else {
                ring = vec![
                    [*w, *s, *height],
                    [*e, *s, *height],
                    [*e, *n, *height],
                    [*w, *n, *height],
                ];
            }
            if rotation.abs() > 1e-8 {
                let cos_r = rotation.cos();
                let sin_r = rotation.sin();
                for pt in &mut ring {
                    let d_lon = pt[0] - center_lon;
                    let d_lat = pt[1] - center_lat;
                    pt[0] = center_lon + d_lon * cos_r - d_lat * sin_r;
                    pt[1] = center_lat + d_lon * sin_r + d_lat * cos_r;
                }
            }
            path(&ring, true)?;
            if let Some(h) = extruded_height {
                let bottom: Vec<[f64; 3]> = ring.iter().map(|c| [c[0], c[1], *h]).collect();
                path(&bottom, true)?;
                for (a, b) in ring.iter().zip(&bottom) {
                    path(&[*a, *b], false)?;
                }
            }
        }
        Shape::Ellipse {
            semi_major,
            semi_minor,
            rotation,
            extruded_height,
            height,
            vertical_lines,
            ..
        } => {
            let ring = crate::geographic::ellipse_ring(
                e.position_degrees.ok_or("ellipse center")?,
                *semi_major,
                *semi_minor,
                *rotation,
                *height,
            )?;
            path(&ring, true)?;
            if let Some(h) = extruded_height {
                let bottom: Vec<_> = ring.iter().map(|c| [c[0], c[1], *h]).collect();
                path(&bottom, true)?;
                let count = vertical_lines.unwrap_or(16) as usize;
                if count > 256 {
                    return Err("ellipse vertical line limit".into());
                }
                for line in 0..count {
                    let i = line * ring.len() / count;
                    path(&[ring[i], bottom[i]], false)?;
                }
            }
        }
        Shape::Wall {
            positions,
            minimum_heights,
            maximum_heights,
        } => {
            let top: Vec<_> = positions
                .iter()
                .enumerate()
                .map(|(i, c)| [c[0], c[1], maximum_heights.get(i).copied().unwrap_or(c[2])])
                .collect();
            let bottom: Vec<_> = positions
                .iter()
                .enumerate()
                .map(|(i, c)| [c[0], c[1], minimum_heights.get(i).copied().unwrap_or(0.)])
                .collect();
            for track in [&top, &bottom] {
                for pair in track.windows(2) {
                    let a = pair[0];
                    let b = pair[1];
                    let arc = crate::geodesic::Line::new([a[0], a[1]], [b[0], b[1]])?;
                    let count = (arc.distance / (cesium_math::WGS84_A * 0.1f64.to_radians()))
                        .ceil()
                        .max(1.) as usize;
                    if count > 1800 {
                        return Err("wall outline span limit".into());
                    }
                    let points: Vec<_> = (0..=count)
                        .map(|i| {
                            let t = i as f64 / count as f64;
                            let p = arc.at(t);
                            [p[0], p[1], a[2] + (b[2] - a[2]) * t]
                        })
                        .collect();
                    path(&points, false)?;
                }
            }
            for (a, b) in top.iter().zip(&bottom) {
                path(&[*a, *b], false)?;
            }
        }
        Shape::Point { .. } | Shape::Billboard { .. } | Shape::Label { .. } => {
            return Err("screen graphics use their own outline fields".into())
        }
        Shape::Polyline { .. } => return Err("polyline outlines are not supported".into()),
        Shape::Box { dimensions, .. } => {
            let d = DVec3::from_array(*dimensions) * 0.5;
            let v = [
                DVec3::new(-d.x, -d.y, -d.z),
                DVec3::new(d.x, -d.y, -d.z),
                DVec3::new(d.x, d.y, -d.z),
                DVec3::new(-d.x, d.y, -d.z),
                DVec3::new(-d.x, -d.y, d.z),
                DVec3::new(d.x, -d.y, d.z),
                DVec3::new(d.x, d.y, d.z),
                DVec3::new(-d.x, d.y, d.z),
            ];
            for [a, b] in [
                [0, 1],
                [1, 2],
                [2, 3],
                [3, 0],
                [4, 5],
                [5, 6],
                [6, 7],
                [7, 4],
                [0, 4],
                [1, 5],
                [2, 6],
                [3, 7],
            ] {
                line(v[a], v[b])?;
            }
        }
        Shape::Model { scale, .. } => {
            let d = DVec3::new(8.0, 14.0, 6.0) * (scale.max(0.1) * 0.5);
            let v = [
                DVec3::new(-d.x, -d.y, -d.z),
                DVec3::new(d.x, -d.y, -d.z),
                DVec3::new(d.x, d.y, -d.z),
                DVec3::new(-d.x, d.y, -d.z),
                DVec3::new(-d.x, -d.y, d.z),
                DVec3::new(d.x, -d.y, d.z),
                DVec3::new(d.x, d.y, d.z),
                DVec3::new(-d.x, d.y, d.z),
            ];
            for [a, b] in [
                [0, 1],
                [1, 2],
                [2, 3],
                [3, 0],
                [4, 5],
                [5, 6],
                [6, 7],
                [7, 4],
                [0, 4],
                [1, 5],
                [2, 6],
                [3, 7],
            ] {
                line(v[a], v[b])?;
            }
        }
        Shape::Frustum {
            fov,
            aspect_ratio,
            near,
            far,
        } => {
            let half_fov = fov * 0.5;
            let tan_fov = half_fov.tan();
            let y_near = near * tan_fov;
            let x_near = y_near * aspect_ratio;
            let y_far = far * tan_fov;
            let x_far = y_far * aspect_ratio;
            let v = [
                DVec3::new(-x_near, -y_near, -near),
                DVec3::new(x_near, -y_near, -near),
                DVec3::new(x_near, y_near, -near),
                DVec3::new(-x_near, y_near, -near),
                DVec3::new(-x_far, -y_far, -far),
                DVec3::new(x_far, -y_far, -far),
                DVec3::new(x_far, y_far, -far),
                DVec3::new(-x_far, y_far, -far),
            ];
            for [a, b] in [
                [0, 1],
                [1, 2],
                [2, 3],
                [3, 0],
                [4, 5],
                [5, 6],
                [6, 7],
                [7, 4],
                [0, 4],
                [1, 5],
                [2, 6],
                [3, 7],
            ] {
                line(v[a], v[b])?;
            }
        }
        Shape::Plane {
            dimensions,
            normal,
            distance,
        } => {
            let v = crate::plane::corners(*dimensions, *normal, *distance);
            for i in 0..4 {
                line(v[i], v[(i + 1) % 4])?;
            }
        }
        Shape::Cylinder {
            length,
            top_radius,
            bottom_radius,
            ..
        } => {
            for (radius, z) in [(*top_radius, length / 2.), (*bottom_radius, -length / 2.)] {
                for i in 0..128 {
                    let a = i as f64 * std::f64::consts::TAU / 128.;
                    let b = (i + 1) as f64 * std::f64::consts::TAU / 128.;
                    line(
                        DVec3::new(radius * a.cos(), radius * a.sin(), z),
                        DVec3::new(radius * b.cos(), radius * b.sin(), z),
                    )?;
                }
            }
            for i in 0..16 {
                let a = i as f64 * std::f64::consts::TAU / 16.;
                line(
                    DVec3::new(top_radius * a.cos(), top_radius * a.sin(), length / 2.),
                    DVec3::new(
                        bottom_radius * a.cos(),
                        bottom_radius * a.sin(),
                        -length / 2.,
                    ),
                )?;
            }
        }
        Shape::Ellipsoid {
            radii,
            inner_radii,
            minimum_clock,
            maximum_clock,
            minimum_cone,
            maximum_cone,
            stacks,
            slices,
            ..
        } => {
            let min_clock = minimum_clock.unwrap_or(0.0);
            let max_clock = maximum_clock.unwrap_or(std::f64::consts::TAU);
            let min_cone = minimum_cone.unwrap_or(0.0);
            let max_cone = maximum_cone.unwrap_or(std::f64::consts::PI);

            let raw_stacks = stacks.unwrap_or(10);
            let raw_slices = slices.unwrap_or(8);

            let mut slice_partitions = ((raw_slices + 1) as f64 * (max_clock - min_clock).abs()
                / std::f64::consts::TAU)
                .round() as u32;
            let mut stack_partitions = ((raw_stacks + 1) as f64 * (max_cone - min_cone).abs()
                / std::f64::consts::PI)
                .round() as u32;

            if slice_partitions < 2 {
                slice_partitions = 2;
            }
            if stack_partitions < 2 {
                stack_partitions = 2;
            }

            let r_outer = DVec3::from_array(*radii);
            let r_inner = inner_radii.map(DVec3::from_array);
            let has_inner_surface = r_inner.is_some_and(|r| r != r_outer);

            let subdivisions = 128usize;

            // Latitude lines
            for i in 0..stack_partitions {
                let phi =
                    min_cone + (i as f64 * (max_cone - min_cone)) / ((stack_partitions - 1) as f64);
                let sin_phi = phi.sin();
                let cos_phi = phi.cos();
                for j in 0..subdivisions - 1 {
                    let theta0 = min_clock
                        + (j as f64 * (max_clock - min_clock)) / ((subdivisions - 1) as f64);
                    let theta1 = min_clock
                        + ((j + 1) as f64 * (max_clock - min_clock)) / ((subdivisions - 1) as f64);
                    let p0 = DVec3::new(
                        r_outer.x * sin_phi * theta0.cos(),
                        r_outer.y * sin_phi * theta0.sin(),
                        r_outer.z * cos_phi,
                    );
                    let p1 = DVec3::new(
                        r_outer.x * sin_phi * theta1.cos(),
                        r_outer.y * sin_phi * theta1.sin(),
                        r_outer.z * cos_phi,
                    );
                    line(p0, p1)?;
                    if has_inner_surface {
                        let rin = r_inner.unwrap();
                        let p_in0 = DVec3::new(
                            rin.x * sin_phi * theta0.cos(),
                            rin.y * sin_phi * theta0.sin(),
                            rin.z * cos_phi,
                        );
                        let p_in1 = DVec3::new(
                            rin.x * sin_phi * theta1.cos(),
                            rin.y * sin_phi * theta1.sin(),
                            rin.z * cos_phi,
                        );
                        line(p_in0, p_in1)?;
                    }
                }
            }

            // Longitude lines
            for i in 0..slice_partitions {
                let theta = min_clock
                    + (i as f64 * (max_clock - min_clock)) / ((slice_partitions - 1) as f64);
                let cos_theta = theta.cos();
                let sin_theta = theta.sin();
                for j in 0..subdivisions - 1 {
                    let phi0 =
                        min_cone + (j as f64 * (max_cone - min_cone)) / ((subdivisions - 1) as f64);
                    let phi1 = min_cone
                        + ((j + 1) as f64 * (max_cone - min_cone)) / ((subdivisions - 1) as f64);
                    let p0 = DVec3::new(
                        r_outer.x * phi0.sin() * cos_theta,
                        r_outer.y * phi0.sin() * sin_theta,
                        r_outer.z * phi0.cos(),
                    );
                    let p1 = DVec3::new(
                        r_outer.x * phi1.sin() * cos_theta,
                        r_outer.y * phi1.sin() * sin_theta,
                        r_outer.z * phi1.cos(),
                    );
                    line(p0, p1)?;
                    if has_inner_surface {
                        let rin = r_inner.unwrap();
                        let p_in0 = DVec3::new(
                            rin.x * phi0.sin() * cos_theta,
                            rin.y * phi0.sin() * sin_theta,
                            rin.z * phi0.cos(),
                        );
                        let p_in1 = DVec3::new(
                            rin.x * phi1.sin() * cos_theta,
                            rin.y * phi1.sin() * sin_theta,
                            rin.z * phi1.cos(),
                        );
                        line(p_in0, p_in1)?;
                    }
                }
            }

            // Cap lines for open cones
            if has_inner_surface {
                let rin = r_inner.unwrap();
                if min_cone > 0.0 {
                    for i in 0..slice_partitions {
                        let theta = min_clock
                            + (i as f64 * (max_clock - min_clock))
                                / ((slice_partitions - 1) as f64);
                        let p_out = DVec3::new(
                            r_outer.x * min_cone.sin() * theta.cos(),
                            r_outer.y * min_cone.sin() * theta.sin(),
                            r_outer.z * min_cone.cos(),
                        );
                        let p_in = DVec3::new(
                            rin.x * min_cone.sin() * theta.cos(),
                            rin.y * min_cone.sin() * theta.sin(),
                            rin.z * min_cone.cos(),
                        );
                        line(p_out, p_in)?;
                    }
                }
                if max_cone < std::f64::consts::PI {
                    for i in 0..slice_partitions {
                        let theta = min_clock
                            + (i as f64 * (max_clock - min_clock))
                                / ((slice_partitions - 1) as f64);
                        let p_out = DVec3::new(
                            r_outer.x * max_cone.sin() * theta.cos(),
                            r_outer.y * max_cone.sin() * theta.sin(),
                            r_outer.z * max_cone.cos(),
                        );
                        let p_in = DVec3::new(
                            rin.x * max_cone.sin() * theta.cos(),
                            rin.y * max_cone.sin() * theta.sin(),
                            rin.z * max_cone.cos(),
                        );
                        line(p_out, p_in)?;
                    }
                }
            }
        }
    }
    if p.indices.is_empty() {
        return Err("empty outline geometry".into());
    }
    Ok(Some(p))
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn analytic_edges_and_czml_flags() {
        let value = serde_json::json!([{"id":"box","positionDegrees":[0,0,100],"shape":{"type":"box","dimensions":[10,20,30]},"fill":false,"outline":true}]);
        let e = crate::entities::decode(&serde_json::to_vec(&value).unwrap()).unwrap();
        let p = primitive(&e[0]).unwrap().unwrap();
        assert_eq!(p.indices.len(), 24);
        assert!(p.indices.iter().all(|i| (*i as usize) < p.vertices.len()));
        assert!(!e[0].fill);
        let mut e = e[0].clone();
        e.outline_width = 2.;
        assert!(primitive(&e).is_err());
    }
    #[test]
    fn geographic_holes_extrusion_and_wire_meshes() {
        let mut data: serde_json::Value =
            serde_json::from_slice(include_bytes!("../../../tests/entities/geographic.json"))
                .unwrap();
        data.as_array_mut().unwrap().remove(0);
        for e in data.as_array_mut().unwrap() {
            e["outline"] = serde_json::json!(true);
        }
        let entities = crate::entities::decode(&serde_json::to_vec(&data).unwrap()).unwrap();
        for e in entities {
            let p = primitive(&e).unwrap().unwrap();
            assert!(!p.indices.is_empty());
            assert!(p
                .vertices
                .iter()
                .flat_map(|v| v.position)
                .all(f32::is_finite));
        }
    }
}
