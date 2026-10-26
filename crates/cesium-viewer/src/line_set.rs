//! Bounded WGS84 geodesic or straight ECEF line lists with independent segment colors.
use crate::entities::{Entity, Shape};
use cesium_formats::{Material, Primitive, Vertex};
use cesium_math::Cartographic;
use glam::{DMat4, DVec3};
pub fn primitive(e: &Entity) -> Result<Primitive, String> {
    let Shape::LineSet {
        paths,
        colors,
        geodesic,
        colors_per_vertex,
    } = &e.shape
    else {
        return Err("line set required".into());
    };
    if e.fill
        || !e.outline
        || paths.is_empty()
        || paths.len() > 128
        || (!colors.is_empty() && colors.len() != paths.len())
    {
        return Err("bounded outline-only paths/colors required".into());
    }
    let origin = crate::geographic::anchor(&e.shape)?;
    let mut p = Primitive {
        vertices: vec![],
        indices: vec![],
        transform: DMat4::IDENTITY,
        material: Material {
            base_color: crate::entity_material::linear(e.outline_color),
            unlit: true,
            double_sided: true,
            alpha_blend: e.outline_color[3] < 1.,
            ..Default::default()
        },
    };
    let coord = |c: [f64; 3]| -> Result<DVec3, String> {
        if c.iter().any(|v| !v.is_finite())
            || c[0].abs() > 180.
            || c[1].abs() > 85.
            || c[2].abs() > 2e6
        {
            return Err("bounded line coordinates required".into());
        }
        Ok(Cartographic::degrees(c[0], c[1], c[2] + 10.5)
            .map_err(|e| e.to_string())?
            .to_ecef())
    };
    for (pi, path) in paths.iter().enumerate() {
        if !(2..=1024).contains(&path.len()) {
            return Err("line path position count".into());
        }
        let cs = colors.get(pi);
        if cs.is_some_and(|c| {
            c.len() != path.len() && (*colors_per_vertex || c.len() != path.len() - 1)
        }) {
            return Err("line color count".into());
        }
        if cs.is_some_and(|c| {
            c.iter()
                .flatten()
                .any(|v| !v.is_finite() || !(0.0..=1.).contains(v))
        }) {
            return Err("finite normalized line colors required".into());
        }
        for (i, pair) in path.windows(2).enumerate() {
            let a = coord(pair[0])?;
            let b = coord(pair[1])?;
            if a.distance(b) < 1e-6 {
                return Err("distinct line positions required".into());
            }
            let arc = if *geodesic {
                Some(crate::geodesic::Line::new(
                    [pair[0][0], pair[0][1]],
                    [pair[1][0], pair[1][1]],
                )?)
            } else {
                None
            };
            let n = arc
                .map_or(1, |a| {
                    (a.distance / (cesium_math::WGS84_A * 0.1_f64.to_radians())).ceil() as usize
                })
                .max(1);
            if n > 1800 || p.vertices.len() + n * 2 > 100000 {
                return Err("line vertex budget".into());
            }
            for k in 0..n {
                for t in [k as f64 / n as f64, (k + 1) as f64 / n as f64] {
                    let v = if let Some(arc) = arc {
                        let [lon, lat] = arc.at(t);
                        coord([lon, lat, pair[0][2] + (pair[1][2] - pair[0][2]) * t])?
                    } else {
                        a.lerp(b, t)
                    };
                    let c = cs.map_or([1.; 4], |c| {
                        let x = c[i];
                        if *colors_per_vertex {
                            std::array::from_fn(|j| x[j] + (c[i + 1][j] - x[j]) * t as f32)
                        } else {
                            x
                        }
                    });
                    p.indices.push(p.vertices.len() as u32);
                    p.vertices.push(Vertex {
                        position: (v - origin).as_vec3().to_array(),
                        normal: [0., 0., 1.],
                        uv: [0.; 2],
                        uv1: [0.; 2],
                        tangent: [0.; 4],
                        feature_id: 0,
                        color: crate::entity_material::linear(c),
                    });
                    p.material.alpha_blend |= c[3] < 1.;
                }
            }
        }
    }
    Ok(p)
}
