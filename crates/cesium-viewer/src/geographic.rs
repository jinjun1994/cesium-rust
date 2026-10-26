//! Geographic entity tessellation, using the bounded GeoJSON polygon/ribbon decoder.
use crate::entities::{Entity, Shape};
use cesium_formats::{Material, Primitive, Vertex};
use cesium_math::Cartographic;
use glam::{DMat4, DVec3};
use serde_json::json;
const OFFSET: f64 = 10.;
const STEP: f64 = 0.1;
fn coordinate(p: [f64; 3]) -> Result<DVec3, String> {
    if p[0].abs() > 180. || p[1].abs() > 85. || p[2].abs() > 2_000_000. {
        return Err(
            "geographic geometry supports longitude ±180°, latitude ±85°, height ±2,000,000m"
                .into(),
        );
    }
    Ok(Cartographic::degrees(p[0], p[1], p[2])
        .map_err(|e| e.to_string())?
        .to_ecef())
}
fn positions(p: &[[f64; 3]], minimum: usize) -> Result<(), String> {
    if p.len() < minimum || p.len() > 1024 {
        return Err("geographic geometry position count".into());
    }
    for p in p {
        coordinate(*p)?;
    }
    Ok(())
}
fn ring(p: &[[f64; 3]]) -> Result<Vec<[f64; 3]>, String> {
    positions(p, 3)?;
    let mut p = p.to_vec();
    if p.first() != p.last() {
        p.push(p[0]);
    }
    Ok(p)
}
pub fn unwrap(lon: f64, origin: f64) -> f64 {
    origin + (lon - origin + 180.).rem_euclid(360.) - 180.
}
fn mean(p: &[[f64; 3]]) -> Result<DVec3, String> {
    positions(p, 2)?;
    let n = p.len() as f64;
    coordinate([
        unwrap(p.iter().map(|v| unwrap(v[0], p[0][0])).sum::<f64>() / n, 0.),
        p.iter().map(|v| v[1]).sum::<f64>() / n,
        p.iter().map(|v| v[2]).sum::<f64>() / n,
    ])
}
pub fn rectangle(
    coordinates: [f64; 4],
    height: f64,
    rotation: f64,
) -> Result<Vec<[f64; 3]>, String> {
    let [w, s, e, n] = coordinates;
    for v in coordinates {
        if !v.is_finite() {
            return Err("finite rectangle coordinates".into());
        }
    }
    let width = (e - w).rem_euclid(360.);
    if w.abs() > 180.
        || e.abs() > 180.
        || s >= n
        || s.abs() > 85.
        || n.abs() > 85.
        || width <= 0.
        || width > 180.
    {
        return Err(
            "rectangle must be ordered, nonempty, <=180° wide and within ±85° latitude".into(),
        );
    }
    if !rotation.is_finite() || rotation.abs() > 1000. {
        return Err("finite rectangle rotation required".into());
    }
    let center_lon = unwrap(w + width / 2., 0.);
    let center_lat = (s + n) / 2.;
    let mut corners = vec![
        [w, s, height],
        [e, s, height],
        [e, n, height],
        [w, n, height],
        [w, s, height],
    ];
    if rotation.abs() > 1e-8 {
        let cos_r = rotation.cos();
        let sin_r = rotation.sin();
        for pt in &mut corners[0..4] {
            let d_lon = pt[0] - center_lon;
            let d_lat = pt[1] - center_lat;
            pt[0] = center_lon + d_lon * cos_r - d_lat * sin_r;
            pt[1] = center_lat + d_lon * sin_r + d_lat * cos_r;
        }
        corners[4] = corners[0];
    }
    Ok(corners)
}
pub fn anchor(shape: &Shape) -> Result<DVec3, String> {
    match shape {
        Shape::PolygonSurface { data } => crate::polygon_surface::anchor(data),
        Shape::PolylineVolume { positions, .. } => crate::polyline_volume::anchor(positions),
        Shape::WidePolyline { data } => mean(&data.positions),
        Shape::Corridor { positions, .. } => mean(positions),
        Shape::LineSet { paths, .. } => mean(&paths.iter().flatten().copied().collect::<Vec<_>>()),
        Shape::Polyline { positions, .. } | Shape::Polygon { positions, .. } => mean(positions),
        Shape::Wall {
            positions: p,
            minimum_heights,
            maximum_heights,
        } => {
            positions(p, 2)?;
            heights(p, minimum_heights, maximum_heights)?;
            let mut centers = p.clone();
            for (i, v) in centers.iter_mut().enumerate() {
                v[2] = (minimum_heights.get(i).copied().unwrap_or(0.)
                    + maximum_heights.get(i).copied().unwrap_or(v[2]))
                    / 2.;
            }
            mean(&centers)
        }
        Shape::Rectangle {
            coordinates,
            height,
            rotation,
            ..
        } => {
            rectangle(*coordinates, *height, *rotation)?;
            let [w, s, e, n] = *coordinates;
            coordinate([
                unwrap(w + (e - w).rem_euclid(360.) / 2., 0.),
                (s + n) / 2.,
                *height,
            ])
        }
        _ => Err("geometry has an explicit entity center".into()),
    }
}
fn heights(p: &[[f64; 3]], min: &[f64], max: &[f64]) -> Result<(), String> {
    if !min.is_empty() && min.len() != p.len() || !max.is_empty() && max.len() != p.len() {
        return Err("wall height arrays must match position count".into());
    }
    for (i, v) in p.iter().enumerate() {
        let low = min.get(i).copied().unwrap_or(0.);
        let high = max.get(i).copied().unwrap_or(v[2]);
        if !low.is_finite()
            || !high.is_finite()
            || low.abs() > 2e6
            || high.abs() > 2e6
            || low >= high
        {
            return Err("finite increasing wall heights required".into());
        }
    }
    Ok(())
}
fn empty(color: [f32; 4]) -> Primitive {
    Primitive {
        vertices: vec![],
        indices: vec![],
        transform: DMat4::IDENTITY,
        material: Material {
            base_color: color,
            metallic: 0.,
            double_sided: true,
            unlit: true,
            ..Default::default()
        },
    }
}
fn merge(out: &mut Primitive, parts: Vec<Primitive>, origin: DVec3) -> Result<(), String> {
    for mut p in parts {
        let start = out.vertices.len() as u32;
        for v in &mut p.vertices {
            v.position = (p
                .transform
                .transform_point3(DVec3::from_array(v.position.map(f64::from)))
                - origin)
                .as_vec3()
                .to_array();
            v.feature_id = 0;
        }
        out.vertices.extend(p.vertices);
        out.indices.extend(p.indices.into_iter().map(|i| i + start));
        budget(out)?;
    }
    Ok(())
}
fn budget(p: &Primitive) -> Result<(), String> {
    if p.vertices.len() > 300000 || p.indices.len() > 1800000 || p.gpu_bytes() > 32 * 1024 * 1024 {
        return Err("geographic entity geometry budget".into());
    }
    Ok(())
}
fn triangle(p: &mut Primitive, a: DVec3, b: DVec3, c: DVec3) -> Result<(), String> {
    let n = (b - a)
        .cross(c - a)
        .try_normalize()
        .ok_or("degenerate wall segment")?;
    let start = p.vertices.len() as u32;
    for v in [a, b, c] {
        p.vertices.push(Vertex {
            position: v.as_vec3().to_array(),
            normal: n.as_vec3().to_array(),
            uv: [0.; 2],
            uv1: [0.; 2],
            color: [1.; 4],
            feature_id: 0,
            tangent: [0.; 4],
        });
    }
    p.indices.extend([start, start + 1, start + 2]);
    budget(p)
}
fn walls(
    out: &mut Primitive,
    p: &[[f64; 3]],
    min: &[f64],
    max: &[f64],
    origin: DVec3,
    curved: bool,
) -> Result<(), String> {
    positions(p, 2)?;
    heights(p, min, max)?;
    for (i, pair) in p.windows(2).enumerate() {
        let a = pair[0];
        let mut b = pair[1];
        b[0] = unwrap(b[0], a[0]);
        let arc = if curved {
            Some(crate::geodesic::Line::new(
                [a[0], a[1]],
                [unwrap(b[0], 0.), b[1]],
            )?)
        } else {
            None
        };
        let steps = arc
            .map(|a| (a.distance / (cesium_math::WGS84_A * STEP.to_radians())).ceil() as usize)
            .unwrap_or_else(|| {
                ((b[0] - a[0]).abs().max((b[1] - a[1]).abs()) / STEP).ceil() as usize
            })
            .max(1);
        if steps > 1800 {
            return Err("wall span limit".into());
        }
        let lo = [
            min.get(i).copied().unwrap_or(0.),
            min.get(i + 1).copied().unwrap_or(0.),
        ];
        let hi = [
            max.get(i).copied().unwrap_or(a[2]),
            max.get(i + 1).copied().unwrap_or(b[2]),
        ];
        if a[0] == b[0] && a[1] == b[1] {
            return Err("duplicate wall position".into());
        }
        let vertex = |t: f64, height: f64| {
            let [lon, lat] = arc.map(|arc| arc.at(t)).unwrap_or([
                unwrap(a[0] + (b[0] - a[0]) * t, 0.),
                a[1] + (b[1] - a[1]) * t,
            ]);
            coordinate([lon, lat, height + OFFSET]).map(|v| v - origin)
        };
        for k in 0..steps {
            let t = k as f64 / steps as f64;
            let u = (k + 1) as f64 / steps as f64;
            let a = vertex(t, lo[0] + (lo[1] - lo[0]) * t)?;
            let b = vertex(u, lo[0] + (lo[1] - lo[0]) * u)?;
            let c = vertex(u, hi[0] + (hi[1] - hi[0]) * u)?;
            let d = vertex(t, hi[0] + (hi[1] - hi[0]) * t)?;
            triangle(out, a, b, c)?;
            triangle(out, a, c, d)?;
        }
    }
    Ok(())
}
fn surface(
    out: &mut Primitive,
    rings: Vec<Vec<[f64; 3]>>,
    extruded: Option<f64>,
    origin: DVec3,
    color: [f32; 4],
) -> Result<(), String> {
    let decode = |rings: &Vec<Vec<[f64; 3]>>| {
        let bytes = serde_json::to_vec(&json!({"type":"Polygon","coordinates":rings}))
            .map_err(|e| e.to_string())?;
        crate::geojson::decode(
            &bytes,
            &crate::geojson::Style {
                fill_color: [color[0], color[1], color[2], 1.],
                surface_offset: OFFSET,
                segment_degrees: STEP,
                ..Default::default()
            },
            &Default::default(),
        )
    };
    let top = decode(&rings)?;
    merge(out, top.primitives, origin)?;
    if let Some(height) = extruded {
        if !height.is_finite() || height.abs() > 1e6 {
            return Err("extruded height limit".into());
        }
        if rings.iter().flatten().any(|v| v[2] != rings[0][0][2]) {
            return Err("extrusion requires a constant-height polygon".into());
        }
        let initial = rings[0][0][2];
        if height == initial {
            return Err("extrusion height equals polygon height".into());
        }
        let lower = height.min(initial);
        let upper = height.max(initial);
        let mut bottom = rings.clone();
        for v in bottom.iter_mut().flatten() {
            v[2] = height;
        }
        let mut cap = decode(&bottom)?;
        for p in &mut cap.primitives {
            for t in p.indices.as_chunks_mut::<3>().0 {
                t.swap(1, 2);
            }
        }
        merge(out, cap.primitives, origin)?;
        for ring in &rings {
            walls(
                out,
                ring,
                &vec![lower; ring.len()],
                &vec![upper; ring.len()],
                origin,
                false,
            )?;
        }
    }
    Ok(())
}
/// Shared fill/outline ellipse boundary following the pinned core quaternion construction.
pub fn ellipse_ring(
    center: [f64; 3],
    major: f64,
    minor: f64,
    rotation: f64,
    height: Option<f64>,
) -> Result<Vec<[f64; 3]>, String> {
    if !major.is_finite()
        || !minor.is_finite()
        || !rotation.is_finite()
        || minor < 1.
        || major < minor
        || major > 1e6
    {
        return Err("ellipse axes 1..1,000,000m and finite rotation required".into());
    }
    let surface = coordinate([center[0], center[1], 0.])?;
    let unit = surface.normalize();
    let east = DVec3::Z.cross(unit).normalize();
    let north = unit.cross(east);
    let mag = surface.length();
    let h = height.unwrap_or(center[2]);
    coordinate([center[0], center[1], h])?;
    let segments = ((std::f64::consts::TAU * major / (cesium_math::WGS84_A * STEP.to_radians()))
        .ceil() as usize)
        .div_ceil(4)
        .max(48)
        * 4;
    (0..segments)
        .map(|i| {
            let theta = i as f64 * std::f64::consts::TAU / segments as f64;
            let az = theta + rotation;
            let axis = east * az.cos() + north * az.sin();
            let radius = major * minor
                / ((major * theta.cos()).powi(2) + (minor * theta.sin()).powi(2)).sqrt();
            let point = glam::DQuat::from_axis_angle(axis, radius / mag) * unit * mag;
            let geo = Cartographic::from_ecef(point).map_err(|e| e.to_string())?;
            let p = [geo.longitude.to_degrees(), geo.latitude.to_degrees(), h];
            coordinate(p)?;
            Ok(p)
        })
        .collect()
}
fn ellipse_cap(
    out: &mut Primitive,
    ring: &[[f64; 3]],
    center: [f64; 3],
    major: f64,
    height: f64,
    origin: DVec3,
    flip: bool,
) -> Result<(), String> {
    let base = out.vertices.len() as u32;
    let count = ring.len();
    let steps = (major / (cesium_math::WGS84_A * STEP.to_radians()))
        .ceil()
        .max(1.) as usize;
    let mut add = |lon: f64, lat: f64| -> Result<(), String> {
        let p = Cartographic::degrees(unwrap(lon, 0.), lat, height + OFFSET)
            .map_err(|e| e.to_string())?;
        out.vertices.push(Vertex {
            position: (p.to_ecef() - origin).as_vec3().to_array(),
            normal: (p.enu_to_ecef().z_axis.truncate() * if flip { -1. } else { 1. })
                .as_vec3()
                .to_array(),
            uv: [0.; 2],
            uv1: [0.; 2],
            tangent: [0.; 4],
            color: [1.; 4],
            feature_id: 0,
        });
        Ok(())
    };
    add(center[0], center[1])?;
    for k in 1..=steps {
        let t = k as f64 / steps as f64;
        for p in ring {
            add(
                center[0] + (unwrap(p[0], center[0]) - center[0]) * t,
                center[1] + (p[1] - center[1]) * t,
            )?;
        }
    }
    let mut tri = |a: u32, b: u32, c: u32| {
        out.indices.extend(if flip { [a, c, b] } else { [a, b, c] });
    };
    for i in 0..count {
        tri(
            base,
            base + 1 + i as u32,
            base + 1 + ((i + 1) % count) as u32,
        );
    }
    for k in 1..steps {
        let inside = base + 1 + ((k - 1) * count) as u32;
        let outside = inside + count as u32;
        for i in 0..count {
            let next = (i + 1) % count;
            tri(inside + i as u32, outside + i as u32, outside + next as u32);
            tri(
                inside + i as u32,
                outside + next as u32,
                inside + next as u32,
            );
        }
    }
    budget(out)
}
pub fn primitive(e: &Entity) -> Result<Primitive, String> {
    let origin = if e.shape.standalone() {
        anchor(&e.shape)?
    } else {
        let p = e.position_degrees.ok_or("ellipse center required")?;
        coordinate(p)?
    };
    let mut out = empty(e.color);
    match &e.shape {
        Shape::Corridor {
            height,
            extruded_height,
            ..
        } => {
            let mut ring = crate::corridor::boundary(&e.shape)?.ring;
            let top = extruded_height.map_or(*height, |h| h.max(*height));
            for p in &mut ring {
                p[2] = top;
            }
            let bottom = extruded_height
                .filter(|h| *h != *height)
                .map(|h| h.min(*height));
            surface(&mut out, vec![ring], bottom, origin, e.color)?;
        }
        Shape::PolygonSurface { data } => return crate::polygon_surface::primitive(e, data),
        Shape::LineSet { .. } => {
            return crate::line_set::primitive(e);
        }
        Shape::Polyline {
            positions: p,
            width,
        } => {
            positions(p, 2)?;
            let bytes = serde_json::to_vec(&json!({"type":"LineString","coordinates":p}))
                .map_err(|e| e.to_string())?;
            let d = crate::geojson::decode(
                &bytes,
                &crate::geojson::Style {
                    line_color: [e.color[0], e.color[1], e.color[2], 1.],
                    line_width: *width,
                    surface_offset: OFFSET,
                    segment_degrees: STEP,
                    ..Default::default()
                },
                &Default::default(),
            )?;
            merge(&mut out, d.primitives, origin)?;
        }
        Shape::Polygon {
            positions,
            holes,
            extruded_height,
        } => {
            if holes.len() > 32 {
                return Err("polygon hole count limit".into());
            }
            let mut rings = vec![ring(positions)?];
            for hole in holes {
                rings.push(ring(hole)?);
            }
            surface(&mut out, rings, *extruded_height, origin, e.color)?;
        }
        Shape::Rectangle {
            coordinates,
            height,
            extruded_height,
            rotation,
            ..
        } => surface(
            &mut out,
            vec![rectangle(*coordinates, *height, *rotation)?],
            *extruded_height,
            origin,
            e.color,
        )?,
        Shape::Ellipse {
            semi_major,
            semi_minor,
            rotation,
            extruded_height,
            height,
            ..
        } => {
            let p = ellipse_ring(
                e.position_degrees.ok_or("ellipse center")?,
                *semi_major,
                *semi_minor,
                *rotation,
                *height,
            )?;
            let center = e.position_degrees.ok_or("ellipse center")?;
            let level = height.unwrap_or(center[2]);
            ellipse_cap(
                &mut out,
                &p,
                center,
                *semi_major,
                level,
                origin,
                extruded_height.is_some_and(|h| h > level),
            )?;
            if let Some(extruded) = extruded_height {
                if !extruded.is_finite() || extruded.abs() > 1e6 || *extruded == level {
                    return Err("bounded distinct ellipse extrusion required".into());
                }
                ellipse_cap(
                    &mut out,
                    &p,
                    center,
                    *semi_major,
                    *extruded,
                    origin,
                    *extruded < level,
                )?;
                let closed = ring(&p)?;
                walls(
                    &mut out,
                    &closed,
                    &vec![level.min(*extruded); closed.len()],
                    &vec![level.max(*extruded); closed.len()],
                    origin,
                    false,
                )?;
            }
        }
        Shape::Wall {
            positions,
            minimum_heights,
            maximum_heights,
        } => walls(
            &mut out,
            positions,
            minimum_heights,
            maximum_heights,
            origin,
            true,
        )?,
        _ => return Err("not a geographic entity".into()),
    }
    if out.indices.is_empty() {
        return Err("empty geographic geometry".into());
    }
    budget(&out)?;
    Ok(out)
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::entities;
    #[test]
    fn shapes_have_finite_rtc_meshes_and_extrusion_faces() {
        let data =
            entities::decode(include_bytes!("../../../tests/entities/geographic.json")).unwrap();
        assert_eq!(data.len(), 6);
        for e in data {
            let p = e.primitive().unwrap();
            assert!(!p.indices.is_empty());
            assert!(p.vertices.iter().all(|v| v
                .position
                .iter()
                .chain(v.normal.iter())
                .all(|n| n.is_finite())));
            assert!(p.indices.iter().all(|i| (*i as usize) < p.vertices.len()));
            let model = e.model(0.).unwrap();
            assert!(model.w_axis.truncate().length() > 6e6);
            if e.id == "rectangle" {
                let heights: Vec<_> = p
                    .vertices
                    .iter()
                    .map(|v| {
                        Cartographic::from_ecef(
                            model.transform_point3(DVec3::from_array(v.position.map(f64::from))),
                        )
                        .unwrap()
                        .height
                    })
                    .collect();
                assert!(heights.iter().any(|h| (*h - 1010.).abs() < 0.1));
                assert!(heights.iter().any(|h| (*h - 20010.).abs() < 0.1));
            }
        }
    }
    #[test]
    fn geographic_czml_and_rejected_unsupported_properties() {
        let (data, _) =
            entities::czml(include_bytes!("../../../tests/entities/geographic.czml")).unwrap();
        assert_eq!(data.len(), 4);
        assert!(matches!(data[0].shape, Shape::Polygon { .. }));
        assert_eq!(data[2].position_degrees.unwrap()[2], 0.);
        assert!(matches!(
            data[2].shape,
            Shape::Ellipse {
                vertical_lines: None,
                height: Some(6000.),
                ..
            }
        ));
        let mut v: serde_json::Value =
            serde_json::from_slice(include_bytes!("../../../tests/entities/geographic.czml"))
                .unwrap();
        v[1]["polygon"]["outline"] = json!(true);
        v[1]["polygon"]["outlineWidth"] = json!(0);
        assert!(entities::czml(&serde_json::to_vec(&v).unwrap())
            .unwrap_err()
            .contains("outline"));
        v[1]["polygon"].as_object_mut().unwrap().remove("outline");
        v[1]["polygon"]
            .as_object_mut()
            .unwrap()
            .remove("outlineWidth");
        v[1]["polygon"]["show"] = json!({"boolean":false});
        let (data, _) = entities::czml(&serde_json::to_vec(&v).unwrap()).unwrap();
        assert!(data[0].position(0.).is_none());
    }
    #[test]
    fn dateline_rectangle_and_invalid_geometry() {
        let v = json!([{"id":"dateline","shape":{"type":"rectangle","coordinatesDegrees":[179.,-1.,-179.,1.],"height":5000.}}]);
        let data = entities::decode(&serde_json::to_vec(&v).unwrap()).unwrap();
        let e = &data[0];
        assert!(e.position(0.).unwrap().x < -6e6);
        let p = e.primitive().unwrap();
        assert!(p.vertices.iter().all(|v| v.position[1].abs() < 250000.));
        for shape in [
            json!({"type":"rectangle","coordinatesDegrees":[0,1,1,0]}),
            json!({"type":"wall","positionsDegrees":[[0,0,100],[1,0,100]],"minimumHeights":[50]}),
            json!({"type":"wall","positionsDegrees":[[0,0,100],[0,0,100]]}),
            json!({"type":"polygon","positionsDegrees":[[0,0,100],[1,1,100],[0,1,100],[1,0,100]]}),
        ] {
            assert!(entities::decode(
                &serde_json::to_vec(&json!([{"id":"invalid","shape":shape}])).unwrap()
            )
            .is_err());
        }
    }
}
