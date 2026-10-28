//! Bounded filled polygon hierarchies: planar 3D caps, WGS84 subdivision, holes and authored ST.
use crate::entities::Entity;
use cesium_formats::{Material, Primitive, Vertex};
use cesium_math::Cartographic;
use glam::{DMat4, DVec2, DVec3};
use serde::{Deserialize, Serialize};
fn yes() -> bool {
    true
}
fn arc() -> String {
    "GEODESIC".into()
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Hierarchy {
    pub positions: Vec<[f64; 3]>,
    #[serde(default)]
    pub holes: Vec<Hierarchy>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TextureHierarchy {
    pub positions: Vec<[f64; 2]>,
    #[serde(default)]
    pub holes: Vec<TextureHierarchy>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct PolygonSurface {
    pub hierarchy: Hierarchy,
    #[serde(default)]
    pub texture_coordinates: Option<TextureHierarchy>,
    #[serde(default)]
    pub height: f64,
    #[serde(default)]
    pub per_position_height: bool,
    #[serde(default)]
    pub extruded_height: Option<f64>,
    #[serde(default = "yes")]
    pub close_top: bool,
    #[serde(default = "yes")]
    pub close_bottom: bool,
    #[serde(default)]
    pub coplanar: bool,
    #[serde(default = "arc")]
    pub arc_type: String,
    #[serde(default)]
    pub st_rotation: f64,
}
fn ecef(p: [f64; 3]) -> Result<DVec3, String> {
    if p.iter().any(|n| !n.is_finite()) || p[0].abs() > 180. || p[1].abs() > 85. || p[2].abs() > 1e7
    {
        return Err("polygon position bounds".into());
    }
    Cartographic::degrees(p[0], p[1], p[2])
        .map(|c| c.to_ecef())
        .map_err(|e| e.to_string())
}
pub fn anchor(s: &PolygonSurface) -> Result<DVec3, String> {
    if s.hierarchy.positions.len() < 3 {
        return Err("polygon needs three positions".into());
    }
    let p = s
        .hierarchy
        .positions
        .iter()
        .map(|p| ecef(*p))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(p.iter().sum::<DVec3>() / p.len() as f64)
}
fn validate(
    h: &Hierarchy,
    uv: Option<&TextureHierarchy>,
    depth: usize,
    count: &mut usize,
) -> Result<(), String> {
    *count += h.positions.len();
    if depth > 8 || *count > 1024 || !(3..=1024).contains(&h.positions.len()) || h.holes.len() > 32
    {
        return Err("polygon hierarchy budget".into());
    }
    for p in &h.positions {
        ecef(*p)?;
    }
    if let Some(t) = uv {
        if t.positions.len() != h.positions.len()
            || t.holes.len() != h.holes.len()
            || t.positions
                .iter()
                .flatten()
                .any(|v| !v.is_finite() || v.abs() > 100.)
        {
            return Err("texture hierarchy must match polygon hierarchy".into());
        }
    }
    for (i, child) in h.holes.iter().enumerate() {
        validate(child, uv.map(|t| &t.holes[i]), depth + 1, count)?;
    }
    Ok(())
}
#[derive(Clone, Copy)]
struct P {
    xyz: DVec3,
    uv: DVec2,
}
struct Builder {
    p: Primitive,
    origin: DVec3,
    rhumb: bool,
    vertex_map: std::collections::HashMap<[u32; 8], u32>,
    normal_override: Option<DVec3>,
}
impl Builder {
    fn triangle(&mut self, a: P, b: P, c: P, normal: DVec3) -> Result<(), String> {
        self.triangle_normals(a, b, c, normal, false)
    }
    fn triangle_normals(
        &mut self,
        a: P,
        mut b: P,
        mut c: P,
        normal: DVec3,
        smooth: bool,
    ) -> Result<(), String> {
        let mut n = (b.xyz - a.xyz).cross(c.xyz - a.xyz);
        if n.length_squared() < 1e-10 {
            return Ok(());
        }
        if n.dot(normal) < 0. {
            std::mem::swap(&mut b, &mut c);
            n = -n;
        }
        let face = n.normalize();
        for v in [a, b, c] {
            let normal = if let Some(normal) = self.normal_override {
                normal
            } else if smooth {
                let c = Cartographic::from_ecef(v.xyz).map_err(|e| e.to_string())?;
                let up = DVec3::new(
                    c.latitude.cos() * c.longitude.cos(),
                    c.latitude.cos() * c.longitude.sin(),
                    c.latitude.sin(),
                );
                up * if up.dot(normal) < 0. { -1. } else { 1. }
            } else {
                face
            };
            let position = (v.xyz - self.origin).as_vec3().to_array();
            let normal = normal.as_vec3().to_array();
            let uv = v.uv.as_vec2().to_array();
            let key = [
                position[0].to_bits(),
                position[1].to_bits(),
                position[2].to_bits(),
                normal[0].to_bits(),
                normal[1].to_bits(),
                normal[2].to_bits(),
                uv[0].to_bits(),
                uv[1].to_bits(),
            ];
            let index = if let Some(index) = self.vertex_map.get(&key) {
                *index
            } else {
                let index = self.p.vertices.len() as u32;
                self.p.vertices.push(Vertex {
                    position,
                    normal,
                    uv,
                    uv1: [0.; 2],
                    tangent: [0.; 4],
                    color: [1.; 4],
                    feature_id: 0,
                });
                self.vertex_map.insert(key, index);
                index
            };
            self.p.indices.push(index);
        }
        if self.p.vertices.len() > 300000 {
            return Err("polygon generated vertex budget".into());
        }
        Ok(())
    }
    fn cap(
        &mut self,
        a: P,
        b: P,
        c: P,
        normal: DVec3,
        curved: Option<f64>,
        depth: usize,
    ) -> Result<(), String> {
        let edges = [
            a.xyz.distance_squared(b.xyz),
            b.xyz.distance_squared(c.xyz),
            c.xyz.distance_squared(a.xyz),
        ];
        let k = (0..3)
            .max_by(|i, j| edges[*i].total_cmp(&edges[*j]))
            .unwrap();
        if let Some(height) = curved.filter(|_| edges[k] > 20000_f64.powi(2) && depth < 16) {
            let (a, b, c) = match k {
                0 => (a, b, c),
                1 => (b, c, a),
                _ => (c, a, b),
            };
            let midpoint = (a.xyz + b.xyz) * 0.5;
            let mut geo = if self.rhumb {
                let a = Cartographic::from_ecef(a.xyz).map_err(|e| e.to_string())?;
                let b = Cartographic::from_ecef(b.xyz).map_err(|e| e.to_string())?;
                let c = rhumb(
                    [a.longitude.to_degrees(), a.latitude.to_degrees(), 0.],
                    [b.longitude.to_degrees(), b.latitude.to_degrees(), 0.],
                    0.5,
                );
                Cartographic::degrees(c[0], c[1], 0.).map_err(|e| e.to_string())?
            } else {
                Cartographic::from_ecef(midpoint).map_err(|e| e.to_string())?
            };
            geo.height = height + 10.;
            let m = P {
                xyz: geo.to_ecef(),
                uv: (a.uv + b.uv) * 0.5,
            };
            self.cap(a, m, c, normal, curved, depth + 1)?;
            self.cap(m, b, c, normal, curved, depth + 1)
        } else {
            self.triangle_normals(a, b, c, normal, curved.is_some())
        }
    }
}
fn frame(points: &[DVec3], origin: DVec3) -> Result<(DVec3, DVec3, DVec3), String> {
    let mut normal = DVec3::ZERO;
    for i in 0..points.len() {
        normal += (points[i] - origin).cross(points[(i + 1) % points.len()] - origin);
    }
    if normal.length_squared() < 1e-8 {
        return Err("degenerate polygon plane".into());
    }
    let mut normal = normal.normalize();
    if normal.dot(origin) < 0. {
        normal = -normal;
    }
    let x = (points[1] - points[0]).normalize();
    let y = normal.cross(x).normalize();
    let x = y.cross(normal).normalize();
    Ok((x, y, normal))
}
// Classical Jacobi covariance axes, matching the pinned coplanar projection basis.
fn principal_frame(points: &[DVec3]) -> Result<(DVec3, DVec3, DVec3), String> {
    use glam::DMat3;
    let mean = points.iter().sum::<DVec3>() / points.len() as f64;
    let mut a = DMat3::ZERO;
    for p in points {
        let v = *p - mean;
        a += DMat3::from_cols(v * v.x, v * v.y, v * v.z) / points.len() as f64;
    }
    let epsilon = a.to_cols_array().iter().map(|v| v * v).sum::<f64>().sqrt() * 1e-20;
    let mut axes = DMat3::IDENTITY;
    for _ in 0..30 {
        let values = a.to_cols_array();
        let pairs = [(1, 2), (0, 2), (0, 1)];
        let &(p, q) = pairs
            .iter()
            .max_by(|(p, q), (r, s)| values[q * 3 + p].abs().total_cmp(&values[s * 3 + r].abs()))
            .unwrap();
        if (pairs
            .iter()
            .map(|(p, q)| 2. * values[q * 3 + p].powi(2))
            .sum::<f64>())
        .sqrt()
            <= epsilon
        {
            break;
        }
        let off = values[q * 3 + p];
        if off.abs() < 1e-15 {
            break;
        }
        let tau = (values[q * 3 + q] - values[p * 3 + p]) / (2. * off);
        let t = if tau < 0. {
            -1. / (-tau + (1. + tau * tau).sqrt())
        } else {
            1. / (tau + (1. + tau * tau).sqrt())
        };
        let c = 1. / (1. + t * t).sqrt();
        let sine = t * c;
        let mut j = DMat3::IDENTITY.to_cols_array();
        j[p * 3 + p] = c;
        j[q * 3 + q] = c;
        j[q * 3 + p] = sine;
        j[p * 3 + q] = -sine;
        let j = DMat3::from_cols_array(&j);
        a = j.transpose() * a * j;
        axes *= j;
    }
    let cols = [axes.x_axis, axes.y_axis, axes.z_axis];
    let mut spans = [0.; 3];
    for (i, axis) in cols.iter().enumerate() {
        let lo = points
            .iter()
            .map(|p| p.dot(*axis))
            .fold(f64::INFINITY, f64::min);
        let hi = points
            .iter()
            .map(|p| p.dot(*axis))
            .fold(f64::NEG_INFINITY, f64::max);
        spans[i] = hi - lo;
    }
    let min = (0..3)
        .min_by(|i, j| spans[*i].total_cmp(&spans[*j]))
        .unwrap();
    let (mut x, y) = match min {
        0 => (cols[1], cols[2]),
        1 => (cols[0], cols[2]),
        _ => (cols[0], cols[1]),
    };
    let mut n = x.cross(y);
    if n.length_squared() < 1e-12 {
        return Err("degenerate coplanar basis".into());
    }
    let geodetic = mean
        / DVec3::new(
            6378137_f64.powi(2),
            6378137_f64.powi(2),
            6356752.314245_f64.powi(2),
        );
    if n.dot(geodetic) < 0. {
        x = -x;
        n = -n;
    }
    Ok((x, y, n.normalize()))
}
fn rhumb(a: [f64; 3], b: [f64; 3], t: f64) -> [f64; 2] {
    let e = (1_f64 / 298.257223563 * (2. - 1. / 298.257223563)).sqrt();
    let iso = |v: f64| {
        let f = v.to_radians();
        (std::f64::consts::FRAC_PI_4 + f / 2.).tan().ln() - e * (e * f.sin()).atanh()
    };
    let target = iso(a[1]) * (1. - t) + iso(b[1]) * t;
    let mut f = a[1].to_radians() * (1. - t) + b[1].to_radians() * t;
    for _ in 0..8 {
        f = 2. * (target + e * (e * f.sin()).atanh()).exp().atan() - std::f64::consts::FRAC_PI_2;
    }
    [
        a[0] + ((b[0] - a[0] + 180.).rem_euclid(360.) - 180.) * t,
        f.to_degrees(),
    ]
}
fn boundary(h: &Hierarchy, s: &PolygonSurface, height: f64) -> Result<Vec<[f64; 3]>, String> {
    let mut out = vec![];
    for i in 0..h.positions.len() {
        let mut a = h.positions[i];
        let mut b = h.positions[(i + 1) % h.positions.len()];
        if !s.per_position_height && !s.coplanar {
            a[2] = height;
            b[2] = height;
        }
        if s.texture_coordinates.is_some() || s.per_position_height || s.coplanar {
            out.push(a);
            continue;
        }
        let line = crate::geodesic::Line::new([a[0], a[1]], [b[0], b[1]])?;
        let n = if s.arc_type == "RHUMB" {
            1
        } else {
            (line.distance / 110000.).ceil().max(1.) as usize
        };
        for j in 0..n {
            let t = j as f64 / n as f64;
            let c = if s.arc_type == "RHUMB" {
                rhumb(a, b, t)
            } else {
                line.at(t)
            };
            out.push([c[0], c[1], height]);
        }
    }
    Ok(out)
}
#[derive(Clone, Copy)]
struct UvFrame {
    x: DVec3,
    y: DVec3,
    normal: DVec3,
    low: DVec2,
    extent: DVec2,
}
fn group(
    b: &mut Builder,
    h: &Hierarchy,
    tex: Option<&TextureHierarchy>,
    s: &PolygonSurface,
    shared: Option<UvFrame>,
) -> Result<(), String> {
    let top = s
        .extruded_height
        .filter(|_| !s.per_position_height)
        .map_or(s.height, |e| e.max(s.height));
    let bottom = s.extruded_height.map(|e| {
        if s.per_position_height {
            e
        } else {
            e.min(s.height)
        }
    });
    let rings = std::iter::once(h)
        .chain(&h.holes)
        .map(|r| boundary(r, s, top))
        .collect::<Result<Vec<_>, _>>()?;
    let points = rings[0]
        .iter()
        .map(|p| ecef(*p))
        .collect::<Result<Vec<_>, _>>()?;
    let uv_frame = if let Some(shared) = shared {
        shared
    } else {
        let (x, y, normal) = if s.coplanar {
            principal_frame(&points)?
        } else {
            frame(&points, b.origin)?
        };
        let (sin, cos) = s.st_rotation.sin_cos();
        let mut low = DVec2::splat(f64::INFINITY);
        let mut high = DVec2::splat(f64::NEG_INFINITY);
        for p in &points {
            let d = *p - b.origin;
            let v = DVec2::new(d.dot(x), d.dot(y));
            let v = DVec2::new(v.x * cos - v.y * sin, v.x * sin + v.y * cos);
            low = low.min(v);
            high = high.max(v);
        }
        UvFrame {
            x,
            y,
            normal,
            low,
            extent: (high - low).max(DVec2::splat(1e-8)),
        }
    };
    let UvFrame {
        x,
        y,
        normal,
        low,
        extent,
    } = uv_frame;
    let origin = b.origin;
    let project = |v: DVec3| {
        let d = v - origin;
        DVec2::new(d.dot(x), d.dot(y))
    };
    let (sin, cos) = s.st_rotation.sin_cos();
    let rotate = |v: DVec2| DVec2::new(v.x * cos - v.y * sin, v.x * sin + v.y * cos);
    let mut verts = vec![];
    let mut flat = vec![];
    let mut holes = vec![];
    for (ri, ring) in rings.iter().enumerate() {
        if ri > 0 {
            holes.push(verts.len());
        }
        let uv = tex.map(|t| if ri == 0 { t } else { &t.holes[ri - 1] });
        for (i, p) in ring.iter().enumerate() {
            let mut c = *p;
            c[2] += if s.coplanar { 0. } else { 10. };
            let xyz = ecef(c)?;
            let plane = project(xyz);
            let st = uv
                .map(|u| DVec2::from_array(u.positions[i]))
                .unwrap_or_else(|| (rotate(plane) - low) / extent);
            flat.extend([plane.x, plane.y]);
            verts.push(P { xyz, uv: st });
        }
    }
    let indices = earcutr::earcut(&flat, &holes, 2).map_err(|e| e.to_string())?;
    if indices.is_empty() {
        return Err("polygon triangulation empty".into());
    }
    let curved = (!s.per_position_height && !s.coplanar).then_some(top);
    for t in indices.as_chunks::<3>().0 {
        if bottom.is_none() || s.close_top {
            b.cap(verts[t[0]], verts[t[1]], verts[t[2]], normal, curved, 0)?;
        }
        if let Some(height) = bottom.filter(|_| s.close_bottom) {
            let mut v = [verts[t[0]], verts[t[1]], verts[t[2]]];
            for v in &mut v {
                let mut c = Cartographic::from_ecef(v.xyz).map_err(|e| e.to_string())?;
                c.height = height + 10.;
                v.xyz = c.to_ecef();
            }
            b.cap(
                v[0],
                v[1],
                v[2],
                -normal,
                (!s.per_position_height && !s.coplanar).then_some(height),
                0,
            )?;
        }
    }
    if let Some(height) = bottom {
        for (ri, ring) in rings.iter().enumerate() {
            let area: f64 = (0..ring.len())
                .map(|i| {
                    let a = project(ecef(ring[i]).unwrap());
                    let c = project(ecef(ring[(i + 1) % ring.len()]).unwrap());
                    a.perp_dot(c)
                })
                .sum();
            for i in 0..ring.len() {
                let j = (i + 1) % ring.len();
                let mut a = ring[i];
                let mut c = ring[j];
                a[2] += 10.;
                c[2] += 10.;
                let ta = ecef(a)?;
                let tc = ecef(c)?;
                a[2] = height + 10.;
                c[2] = height + 10.;
                let ba = ecef(a)?;
                let bc = ecef(c)?;
                let n = (tc - ta).cross((ta + tc) * 0.5).normalize()
                    * if (area > 0.) == (ri == 0) { 1. } else { -1. };
                let p = |xyz, uv| P {
                    xyz,
                    uv: DVec2::from_array(uv),
                };
                b.triangle(p(ta, [0., 1.]), p(ba, [0., 0.]), p(bc, [1., 0.]), n)?;
                b.triangle(p(ta, [0., 1.]), p(bc, [1., 0.]), p(tc, [1., 1.]), n)?;
            }
        }
    }
    // An even-depth descendant is an island with its own immediate holes.
    for (i, hole) in h.holes.iter().enumerate() {
        for (j, island) in hole.holes.iter().enumerate() {
            group(
                b,
                island,
                tex.map(|t| &t.holes[i].holes[j]),
                s,
                Some(uv_frame),
            )?;
        }
    }
    Ok(())
}
pub fn primitive(e: &Entity, s: &PolygonSurface) -> Result<Primitive, String> {
    validate(&s.hierarchy, s.texture_coordinates.as_ref(), 0, &mut 0)?;
    if !s.height.is_finite()
        || s.height.abs() > 1e7
        || s.extruded_height
            .is_some_and(|h| !h.is_finite() || h.abs() > 1e7)
        || !s.st_rotation.is_finite()
        || s.st_rotation.abs() > 10000.
        || !matches!(s.arc_type.as_str(), "GEODESIC" | "RHUMB")
    {
        return Err("polygon options bounds".into());
    }
    if s.coplanar && s.extruded_height.is_some() {
        return Err("coplanar extrusion unsupported".into());
    }
    let mut b = Builder {
        p: Primitive {
            vertices: vec![],
            indices: vec![],
            transform: DMat4::IDENTITY,
            material: Material {
                base_color: e.color,
                metallic: 0.,
                double_sided: true,
                unlit: true,
                ..Default::default()
            },
        },
        origin: anchor(s)?,
        rhumb: s.arc_type == "RHUMB",
        vertex_map: std::collections::HashMap::new(),
        normal_override: if s.coplanar {
            Some(
                principal_frame(
                    &s.hierarchy
                        .positions
                        .iter()
                        .map(|p| ecef(*p))
                        .collect::<Result<Vec<_>, _>>()?,
                )?
                .2,
            )
        } else {
            None
        },
    };
    group(
        &mut b,
        &s.hierarchy,
        s.texture_coordinates.as_ref(),
        s,
        None,
    )?;
    Ok(b.p)
}
pub fn outline_paths(s: &PolygonSurface) -> Result<Vec<Vec<[f64; 3]>>, String> {
    fn visit(
        h: &Hierarchy,
        s: &PolygonSurface,
        out: &mut Vec<Vec<[f64; 3]>>,
    ) -> Result<(), String> {
        let top = s
            .extruded_height
            .filter(|_| !s.per_position_height)
            .map_or(s.height, |e| e.max(s.height));
        let mut ring = boundary(h, s, top)?;
        if s.arc_type == "RHUMB" && !s.per_position_height && !s.coplanar {
            ring.clear();
            for i in 0..h.positions.len() {
                let a = h.positions[i];
                let b = h.positions[(i + 1) % h.positions.len()];
                let line = crate::geodesic::Line::new([a[0], a[1]], [b[0], b[1]])?;
                let n = (line.distance / 10000.).ceil().max(1.) as usize;
                for k in 0..n {
                    let p = rhumb(a, b, k as f64 / n as f64);
                    ring.push([p[0], p[1], top]);
                }
            }
        }
        ring.push(ring[0]);
        out.push(ring.clone());
        if let Some(e) = s.extruded_height {
            let height = if s.per_position_height {
                e
            } else {
                e.min(s.height)
            };
            out.push(ring.iter().map(|p| [p[0], p[1], height]).collect());
            for p in &h.positions {
                let a = if s.per_position_height {
                    *p
                } else {
                    [p[0], p[1], top]
                };
                out.push(vec![a, [p[0], p[1], height]]);
            }
        }
        for hole in &h.holes {
            visit(hole, s, out)?;
        }
        Ok(())
    }
    let mut out = vec![];
    visit(&s.hierarchy, s, &mut out)?;
    Ok(out)
}

pub fn outline(e: &Entity, data: &PolygonSurface) -> Result<Primitive, String> {
    let original = anchor(data)?;
    let mut out: Option<Primitive> = None;
    for path in outline_paths(data)? {
        let mut line = e.clone();
        line.shape = crate::entities::Shape::LineSet {
            paths: vec![path],
            colors: vec![],
            geodesic: !data.per_position_height && !data.coplanar && data.arc_type == "GEODESIC",
            colors_per_vertex: false,
        };
        line.fill = false;
        line.outline = true;
        line.material = None;
        let delta = crate::geographic::anchor(&line.shape)? - original;
        let mut p = crate::line_set::primitive(&line)?;
        for v in &mut p.vertices {
            v.position = (DVec3::from_array(v.position.map(f64::from)) + delta)
                .as_vec3()
                .to_array();
        }
        if let Some(out) = &mut out {
            let offset = out.vertices.len() as u32;
            out.indices
                .extend(p.indices.into_iter().map(|i| i + offset));
            out.vertices.extend(p.vertices);
            if out.vertices.len() > 100000 {
                return Err("polygon outline generated budget".into());
            }
        } else {
            out = Some(p);
        }
    }
    out.ok_or_else(|| "empty polygon outline".into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entities::{self, Shape};
    fn input(id: &str, ext: &str) -> Vec<u8> {
        std::fs::read(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join(format!("../../tests/sandcastle/full/{id}.{ext}")),
        )
        .unwrap()
    }
    fn parse(id: &str) -> Vec<Entity> {
        if id.ends_with("-dev") {
            let base = url::Url::from_file_path(
                std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                    .join(format!("../../tests/sandcastle/full/{id}.primitive.json")),
            )
            .unwrap();
            let raw =
                crate::entity_resources::hydrate(&input(id, "primitive.json"), &base, |u, _| {
                    std::fs::read(u.to_file_path().unwrap()).map_err(|e| e.to_string())
                })
                .unwrap();
            crate::primitive_instances::decode(&raw).unwrap()
        } else if id.starts_with("czml-") {
            entities::czml(&input(id, "czml")).unwrap().0
        } else {
            entities::decode(&input(id, "json")).unwrap()
        }
    }
    #[test]
    fn all_four_original_polygon_contracts() {
        for id in [
            "polygon",
            "polygon-dev",
            "coplanar-polygon-dev",
            "czml-polygon",
        ] {
            let actual = parse(id);
            let expected: Vec<Entity> = serde_json::from_slice(&input(id, "json")).unwrap();
            assert_eq!(actual.len(), expected.len());
            for (a, e) in actual.iter().zip(&expected) {
                assert_eq!(a.id, e.id);
                assert_eq!(
                    serde_json::to_value(&a.shape).unwrap(),
                    serde_json::to_value(&e.shape).unwrap()
                );
                let mesh = a.primitive().unwrap();
                assert!(!mesh.indices.is_empty());
                assert!(mesh.vertices.iter().all(|v| v
                    .position
                    .iter()
                    .chain(&v.normal)
                    .chain(&v.uv)
                    .all(|v| v.is_finite())));
                if a.outline {
                    assert!(crate::outline::primitive(a).unwrap().is_some());
                }
            }
        }
    }
    #[test]
    fn open_caps_and_authored_texture_coordinates() {
        let orange = parse("czml-polygon").remove(3);
        let outline = crate::outline::primitive(&orange).unwrap().unwrap();
        assert_eq!(
            outline.indices.len(),
            24,
            "four straight upper/lower edges and four verticals"
        );
        let mut green = parse("czml-polygon").remove(2);
        let Shape::PolygonSurface { data } = &mut green.shape else {
            panic!()
        };
        assert!(!data.close_top && !data.close_bottom);
        let open = green.primitive().unwrap().indices.len();
        let Shape::PolygonSurface { data } = &mut green.shape else {
            panic!()
        };
        data.close_top = true;
        data.close_bottom = true;
        assert!(green.primitive().unwrap().indices.len() > open);
        let mut textured = parse("polygon").remove(3);
        let mesh = textured.primitive().unwrap();
        let crate::entity_material::SurfaceMaterial::Image { image, .. } =
            textured.material.as_ref().unwrap()
        else {
            panic!()
        };
        let source = crate::entity_material::decode_image(image).unwrap();
        let row = source.image.width as usize * 4;
        let height = source.image.height as usize;
        assert_eq!(
            &mesh.material.textures[0].as_ref().unwrap().image.rgba[..row],
            &source.image.rgba[(height - 1) * row..height * row],
            "authored bottom-origin ST keeps original image upright"
        );

        let Shape::PolygonSurface { data } = &mut textured.shape else {
            panic!()
        };
        let uv = data.texture_coordinates.as_mut().unwrap();
        for st in &uv.positions {
            assert!(mesh
                .vertices
                .iter()
                .any(|v| (v.uv[0] as f64 - st[0]).abs() < 1e-6
                    && (v.uv[1] as f64 - st[1]).abs() < 1e-6));
        }
        uv.positions.pop();
        assert!(textured.primitive().is_err());
    }
    #[test]
    fn independent_coplanar_position_and_rotated_st_reference() {
        let reference: serde_json::Value =
            serde_json::from_slice(&input("polygon-reference", "json")).unwrap();
        for (e, case) in parse("coplanar-polygon-dev")
            .iter()
            .zip(reference["cases"].as_array().unwrap())
        {
            let Shape::PolygonSurface { data } = &e.shape else {
                panic!()
            };
            let origin = anchor(data).unwrap();
            let mesh = e.primitive().unwrap();
            for v in case["vertices"].as_array().unwrap() {
                let p: DVec3 =
                    DVec3::from_array(serde_json::from_value(v["position"].clone()).unwrap());
                let st: [f64; 2] = serde_json::from_value(v["st"].clone()).unwrap();
                let actual = mesh
                    .vertices
                    .iter()
                    .find(|v| {
                        (origin + DVec3::from_array(v.position.map(f64::from))).distance(p) < 0.1
                    })
                    .expect("reference source vertex survives triangulation");
                assert!(
                    (actual.uv[0] as f64 - st[0]).abs() < 1e-5
                        && (actual.uv[1] as f64 - st[1]).abs() < 1e-5,
                    "{} actual {:?}, reference {:?}",
                    e.id,
                    actual.uv,
                    st
                );
            }
        }
    }
    #[test]
    fn rhumb_and_all_point_menu_alternatives() {
        let a = rhumb([-120., 45., 0.], [-80., 45., 0.], 0.5);
        assert!((a[0] + 100.).abs() < 1e-10 && (a[1] - 45.).abs() < 1e-10);
        let line = crate::geodesic::Line::new([-120., 45.], [-80., 45.]).unwrap();
        assert!(line.at(0.5)[1] > 46.);
        let v = crate::entity_variants::decode(&input("points", "variants.json")).unwrap();
        assert_eq!(v.variants.len(), 6);
        assert_eq!(v.variants[3].entities.len(), 3);
        assert_eq!(
            v.variants[4].entities[0].screen.scale_by_distance,
            Some([150., 2., 15000000., 0.5])
        );
        assert!(crate::entity_variants::decode(br#"{"initial":1,"variants":[]}"#).is_err());
    }
}
