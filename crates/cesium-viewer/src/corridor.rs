//! Cesium reference equations: Apache-2.0, Cesium GS; license in tests/sandcastle/reference-license.md.
//! Bounded corridor construction following the pinned Cesium core tangent-offset/corner equations.
//! Center arcs use the existing WGS84 inverse/direct solver; cap tessellation remains shared.
use crate::entities::{CornerType, Shape};
use cesium_math::Cartographic;
use glam::{DQuat, DVec3};
pub struct Boundary {
    pub ring: Vec<[f64; 3]>,
    pub vertical: Vec<[f64; 3]>,
}
fn cart(p: [f64; 3]) -> Result<DVec3, String> {
    if p.iter().any(|v| !v.is_finite()) || p[0].abs() > 180. || p[1].abs() > 85. || p[2].abs() > 1e6
    {
        return Err("bounded corridor coordinates required".into());
    }
    Ok(Cartographic::degrees(p[0], p[1], 0.)
        .map_err(|e| e.to_string())?
        .to_ecef())
}
fn geo(p: DVec3, height: f64) -> Result<[f64; 3], String> {
    let c = Cartographic::from_ecef(p).map_err(|e| e.to_string())?;
    Ok([c.longitude.to_degrees(), c.latitude.to_degrees(), height])
}
fn normal(p: DVec3) -> DVec3 {
    let a = cesium_math::WGS84_A;
    let b = a * (1. - 1. / 298.257223563);
    (p / DVec3::new(a * a, a * a, b * b)).normalize()
}
fn shifted_arc(
    start: DVec3,
    end: DVec3,
    left: DVec3,
    width: f64,
    right: &mut Vec<DVec3>,
    lhs: &mut Vec<DVec3>,
) -> Result<(), String> {
    let a = geo(start, 0.)?;
    let b = geo(end, 0.)?;
    let arc = crate::geodesic::Line::new([a[0], a[1]], [b[0], b[1]])?;
    let count = (arc.distance / (cesium_math::WGS84_A * 0.1_f64.to_radians()))
        .ceil()
        .max(1.) as usize;
    if count > 1800 {
        return Err("corridor segment span limit".into());
    }
    let skip = usize::from(!right.is_empty());
    for i in skip..=count {
        let [lon, lat] = arc.at(i as f64 / count as f64);
        let p = cart([lon, lat, 0.])?;
        right.push(p - left * width);
        lhs.push(p + left * width);
    }
    Ok(())
}
fn rounded(
    pivot: DVec3,
    start: DVec3,
    end: DVec3,
    left_outside: bool,
    beveled: bool,
) -> Vec<DVec3> {
    let angle = (start - pivot)
        .normalize()
        .dot((end - pivot).normalize())
        .clamp(-1., 1.)
        .acos();
    let n = if beveled {
        1
    } else {
        (angle / 5_f64.to_radians()).ceil() as usize + 1
    };
    let axis = if left_outside {
        -pivot.normalize()
    } else {
        pivot.normalize()
    };
    (1..=n)
        .map(|i| DQuat::from_axis_angle(axis, angle * i as f64 / n as f64) * start)
        .collect()
}
pub fn boundary(shape: &Shape) -> Result<Boundary, String> {
    let Shape::Corridor {
        positions,
        width,
        height,
        extruded_height,
        corner_type,
    } = shape
    else {
        return Err("corridor required".into());
    };
    if !(2..=32).contains(&positions.len())
        || !width.is_finite()
        || !(1.0..=1000000.).contains(width)
        || !height.is_finite()
        || height.abs() > 1e6
        || extruded_height.is_some_and(|h| !h.is_finite() || h.abs() > 1e6)
    {
        return Err("corridor positions/width/height limit".into());
    }
    let points = positions
        .iter()
        .map(|p| cart(*p))
        .collect::<Result<Vec<_>, _>>()?;
    if points.windows(2).any(|p| p[0].distance(p[1]) < 1.) {
        return Err("distinct corridor positions required".into());
    }
    let half = *width / 2.;
    let mut forward = (points[1] - points[0]).normalize();
    let mut left = normal(points[0]).cross(forward).normalize();
    let mut backward = -forward;
    let mut previous = points[0];
    let mut right = vec![];
    let mut lhs = vec![];
    let mut walls = vec![];
    for i in 1..points.len() - 1 {
        let p = points[i];
        let n = normal(p);
        forward = (points[i + 1] - p).normalize();
        let fp = (forward - n * forward.dot(n)).normalize();
        let bp = (backward - n * backward.dot(n)).normalize();
        let dot = fp.dot(bp);
        if dot.abs() > 1. - 1e-7 {
            if dot > 0. {
                return Err("corridor reversal unsupported".into());
            }
            continue;
        }
        let d = (forward + backward).normalize();
        let direction = (d - n * d.dot(n)).normalize();
        let scalar = half / direction.cross(backward).length().max(0.25);
        let corner = direction * scalar;
        let left_outside = backward.cross(forward).dot(n) >= 0.;
        let inside = p + corner;
        let sign = if left_outside { 1. } else { -1. };
        let center = inside + left * (sign * half);
        let start = inside + left * (sign * 2. * half);
        shifted_arc(previous, center, left, half, &mut right, &mut lhs)?;
        // One shared inside vertex, avoiding tiny cracks between independently projected offset arcs.
        if left_outside {
            *right.last_mut().unwrap() = inside;
        } else {
            *lhs.last_mut().unwrap() = inside;
        }
        walls.push(inside);
        left = n.cross(forward).normalize();
        let end = inside + left * (sign * 2. * half);
        previous = inside + left * (sign * half);
        let outside = if left_outside { &mut lhs } else { &mut right };
        let corner_points = match corner_type {
            CornerType::Mitered => vec![p - corner, end],
            CornerType::Rounded => rounded(inside, start, end, left_outside, false),
            CornerType::Beveled => rounded(inside, start, end, left_outside, true),
        };
        walls.push(corner_points[corner_points.len() / 2]);
        if *corner_type == CornerType::Beveled {
            walls.push(start);
            walls.push(end);
        }
        outside.extend(corner_points);
        backward = -forward;
    }
    shifted_arc(
        previous,
        *points.last().unwrap(),
        left,
        half,
        &mut right,
        &mut lhs,
    )?;
    walls.extend([
        right[0],
        lhs[0],
        *right.last().unwrap(),
        *lhs.last().unwrap(),
    ]);
    let mut ring = right.clone();
    if *corner_type == CornerType::Rounded {
        let cap = rounded(
            (*right.last().unwrap() + *lhs.last().unwrap()) / 2.,
            *right.last().unwrap(),
            *lhs.last().unwrap(),
            false,
            false,
        );
        walls.push(cap[cap.len() / 2]);
        ring.extend(cap);
    }
    ring.extend(lhs.iter().rev().copied());
    if *corner_type == CornerType::Rounded {
        let cap = rounded((right[0] + lhs[0]) / 2., lhs[0], right[0], false, false);
        walls.push(cap[cap.len() / 2]);
        ring.extend(cap);
    }
    let mut coordinates = Vec::new();
    for p in ring {
        let c = geo(p, *height)?;
        if coordinates
            .last()
            .is_none_or(|last: &[f64; 3]| (last[0] - c[0]).abs().max((last[1] - c[1]).abs()) > 1e-9)
        {
            coordinates.push(c);
        }
    }
    if coordinates.len() > 1023 {
        return Err("corridor boundary budget".into());
    }
    if coordinates.first() != coordinates.last() {
        coordinates.push(coordinates[0]);
    }
    Ok(Boundary {
        ring: coordinates,
        vertical: walls
            .into_iter()
            .map(|p| geo(p, *height))
            .collect::<Result<Vec<_>, _>>()?,
    })
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::entities;
    use serde_json::Value;
    fn fixture(id: &str, ext: &str) -> Vec<u8> {
        std::fs::read(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join(format!("../../tests/sandcastle/full/{id}.{ext}")),
        )
        .unwrap()
    }
    #[test]
    fn full_case_source_parameters_and_geometry() {
        for id in [
            "circle-dev",
            "corridor",
            "corridor-dev",
            "corridor-outline-dev",
        ] {
            let actual = if id == "corridor" {
                entities::decode(&fixture(id, "json")).unwrap()
            } else {
                crate::primitive_instances::decode(&fixture(id, "primitive.json")).unwrap()
            };
            let expected: Vec<entities::Entity> =
                serde_json::from_slice(&fixture(id, "json")).unwrap();
            assert_eq!(
                serde_json::to_value(&actual).unwrap(),
                serde_json::to_value(expected).unwrap(),
                "{id}"
            );
            for e in actual {
                assert!(!e.primitive().unwrap().vertices.is_empty(), "{id}");
                if e.outline {
                    assert!(crate::outline::primitive(&e).unwrap().is_some());
                }
            }
        }
    }
    #[test]
    fn boundary_tracks_independent_cesium_geometry() {
        let reference: Value =
            serde_json::from_slice(&fixture("corridor-reference", "json")).unwrap();
        assert_eq!(
            reference["corridorLibrarySha256"],
            "d4e040e851045cff862e8f69a74b0907878e41cf70cc6be5aed9c931e8ec25d8"
        );
        for c in reference["cases"].as_array().unwrap() {
            let shape: Shape = serde_json::from_value(c["shape"].clone()).unwrap();
            let b = boundary(&shape).unwrap();
            let vertices: Vec<[f64; 3]> = serde_json::from_value(c["positions"].clone()).unwrap();
            let indices: Vec<usize> = serde_json::from_value(c["indices"].clone()).unwrap();
            let distance = |p: DVec3, a: DVec3, b: DVec3| {
                let d = b - a;
                p.distance(a + d * ((p - a).dot(d) / d.length_squared()).clamp(0., 1.))
            };
            let mut worst = 0_f64;
            for p in &b.ring {
                let p = cart(*p).unwrap();
                let d = indices
                    .as_chunks::<2>()
                    .0
                    .iter()
                    .map(|[a, b]| {
                        distance(
                            p,
                            DVec3::from_array(vertices[*a]),
                            DVec3::from_array(vertices[*b]),
                        )
                    })
                    .fold(f64::INFINITY, f64::min);
                worst = worst.max(d);
            }
            let ours: Vec<_> = b.ring.iter().map(|p| cart(*p).unwrap()).collect();
            for p in &vertices {
                let p = DVec3::from_array(*p);
                let d = ours
                    .windows(2)
                    .map(|v| distance(p, v[0], v[1]))
                    .fold(f64::INFINITY, f64::min);
                worst = worst.max(d);
            }
            eprintln!(
                "reference {:?}: maximum boundary deviation {worst:.3} m",
                c["shape"]["cornerType"]
            );
            assert!(
                worst < 300.,
                "{:?}: distance {worst}m from independent reference boundary",
                c["shape"]["cornerType"]
            );
            assert_eq!(b.ring.first(), b.ring.last());
            if let Shape::Corridor {
                corner_type: CornerType::Mitered,
                ..
            } = shape
            {
                assert_eq!(b.vertical.len(), 6);
            }
        }
    }
    #[test]
    fn rejects_degenerate_width_positions_and_corners() {
        let mut e = entities::decode(&fixture("corridor", "json"))
            .unwrap()
            .remove(0);
        for width in [0., -1., 1000001., f64::INFINITY] {
            if let Shape::Corridor { width: w, .. } = &mut e.shape {
                *w = width;
            }
            assert!(e.primitive().is_err());
        }
        let mut e = entities::decode(&fixture("corridor", "json"))
            .unwrap()
            .remove(0);
        if let Shape::Corridor { positions, .. } = &mut e.shape {
            positions[1] = positions[0];
        }
        assert!(e.primitive().is_err());
    }
}
