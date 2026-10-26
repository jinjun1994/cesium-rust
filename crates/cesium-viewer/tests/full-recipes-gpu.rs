//! Complete case contracts, real framebuffer readback and integer picking.
mod common;
use cesium_renderer::{attachment, device, instance, FeatureRef, Renderer};
use cesium_viewer::{
    entities::{self},
    Scene,
};
use common::pixels;
use glam::DVec3;
use std::time::Duration;
fn project(s: &Scene, p: DVec3, w: u32, h: u32) -> (f64, f64, f64) {
    let c = s.orbit.camera(w, h);
    let v = c.camera_to_ecef.transpose() * (p - c.eye);
    let ty = (c.fovy * 0.5).tan();
    (
        (v.x / (-v.z * ty * w as f64 / h as f64) + 1.) * w as f64 / 2.,
        (1. - v.y / (-v.z * ty)) * h as f64 / 2.,
        -v.z,
    )
}
fn pick(s: &Scene, r: &mut Renderer, x: u32, y: u32) -> Option<FeatureRef> {
    let t = attachment(
        &r.device,
        768,
        512,
        r.format(),
        wgpu::TextureUsages::RENDER_ATTACHMENT,
    );
    s.render(r, &t.create_view(&Default::default()), Some((x, y)))
        .unwrap();
    r.device
        .poll(wgpu::PollType::Wait {
            submission_index: None,
            timeout: Some(Duration::from_secs(30)),
        })
        .unwrap();
    r.poll_pick().unwrap()
}
fn is_id(s: &Scene, f: Option<FeatureRef>, id: &str) -> bool {
    f.is_some_and(|f| s.properties(f).unwrap()["id"] == id)
}
async fn verify(id: &str, data: &[u8]) {
    let base = url::Url::from_file_path(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join(format!("../../tests/sandcastle/full/{id}.czml")),
    )
    .unwrap();
    let hydrated = if id.starts_with("czml-") {
        cesium_viewer::entity_resources::hydrate(data, &base, |u, _| {
            std::fs::read(u.to_file_path().unwrap()).map_err(|e| e.to_string())
        })
        .ok()
    } else {
        None
    };
    let data = hydrated.as_deref().unwrap_or(data);
    let (_, d, q) = device(&instance(), None).await.unwrap();
    let mut r = Renderer::new(d, q, wgpu::TextureFormat::Rgba8UnormSrgb, 768, 512).unwrap();
    let mut s = Scene::new(&mut r, None).unwrap();
    let is_primitive = id.ends_with("-dev");
    let is_outline = id.contains("outline");
    let specs = if is_primitive {
        cesium_viewer::primitive_instances::decode(data).unwrap()
    } else if id.starts_with("czml-") {
        let (specs, clock) = entities::czml(data).unwrap();
        s.clock = clock;
        specs
    } else {
        entities::decode(data).unwrap()
    };
    if is_primitive {
        s.set_primitive_instances(&mut r, data).unwrap();
        s.set_view(cesium_viewer::camera_control::ViewOptions {
            destination_degrees: [-100., 40., 17500000.],
            heading: 0.,
            pitch: -std::f64::consts::FRAC_PI_2,
            roll: 0.,
        })
        .unwrap();
        let initial = pixels(&mut r, &s).unwrap();
        assert!(
            initial
                .chunks(4)
                .filter(|p| if is_outline {
                    p[0] > 225 && p[1] > 225 && p[2] > 225
                } else {
                    (p[0] > p[1].saturating_add(20) && p[0] > p[2].saturating_add(20))
                        || (p[1] > p[0].saturating_add(20) && p[1] > p[2].saturating_add(20))
                        || (p[2] > p[0].saturating_add(20) && p[2] > p[1].saturating_add(20))
                })
                .count()
                > 8,
            "original outline visible in default wide view"
        );
        let before = s.entity_specs();
        assert!(s
            .set_primitive_instances(&mut r, br#"{"primitives":[]}"#)
            .is_err());
        assert_eq!(
            serde_json::to_value(before).unwrap(),
            serde_json::to_value(s.entity_specs()).unwrap(),
            "failed primitive import must be atomic"
        );
    } else {
        s.set_entities(&mut r, specs.clone()).unwrap();
    }
    s.zoom_to_entities(&[], 768, 512).unwrap();
    // Every original vertex, including outline-only geometry, fits wide and portrait viewports.
    for (w, h) in [(768, 512), (400, 900)] {
        s.zoom_to_entities(&[], w, h).unwrap();
        for e in &specs {
            let p = e.primitive().unwrap();
            let m = e.model(s.clock.current_time).unwrap();
            for v in &p.vertices {
                let (x, y, z) = project(
                    &s,
                    m.transform_point3(DVec3::from_array(v.position.map(f64::from))),
                    w,
                    h,
                );
                assert!(
                    z > 0. && x > 0. && x < w as f64 && y > 0. && y < h as f64,
                    "{id} clipped vertex {x},{y}"
                );
            }
        }
    }
    s.zoom_to_entities(&[], 768, 512).unwrap();
    let frame = pixels(&mut r, &s).unwrap();
    common::save_png(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join(format!("../../docs/full-{id}-preview.png")),
        &frame,
        768,
        512,
        image::ColorType::Rgba8,
    )
    .unwrap();
    for e in &specs {
        let p = e.primitive().unwrap();
        let m = e.model(s.clock.current_time).unwrap();
        if e.fill {
            let mut locations = vec![(0., m.transform_point3(DVec3::ZERO))];
            if e.shape.geographic() || !p.indices.is_empty() {
                locations.clear();
                for t in p.indices.as_chunks::<3>().0 {
                    let v = [t[0], t[1], t[2]].map(|i| {
                        m.transform_point3(DVec3::from_array(
                            p.vertices[i as usize].position.map(f64::from),
                        ))
                    });
                    let a = project(&s, v[0], 768, 512);
                    let b = project(&s, v[1], 768, 512);
                    let c = project(&s, v[2], 768, 512);
                    let area = ((b.0 - a.0) * (c.1 - a.1) - (b.1 - a.1) * (c.0 - a.0)).abs();
                    locations.push((area, (v[0] + v[1] + v[2]) / 3.));
                }
                locations.sort_by(|a, b| b.0.total_cmp(&a.0));
                locations.truncate(32);
            }
            if locations.is_empty() {
                locations.push((0., m.transform_point3(DVec3::ZERO)));
            }
            let mut found = false;
            'samples: for (_, point) in locations {
                let (x, y, z) = project(&s, point, 768, 512);
                if z <= 0. {
                    continue;
                }
                for (dx, dy) in [(0, 0), (1, 0), (-1, 0), (0, 1), (0, -1)] {
                    let x = x.round() as i32 + dx;
                    let y = y.round() as i32 + dy;
                    if !(0..768).contains(&x) || !(0..512).contains(&y) {
                        continue;
                    }
                    if is_id(&s, pick(&s, &mut r, x as u32, y as u32), &e.id) {
                        found = true;
                        break 'samples;
                    }
                }
            }
            assert!(found, "{id}: filled {} must be selectable", e.id);
        } else {
            let (x, y, _) = project(&s, m.transform_point3(DVec3::ZERO), 768, 512);
            let mut hole = false;
            'holes: for dx in [-8., 0., 8.] {
                for dy in [-8., 0., 8.] {
                    if !is_id(
                        &s,
                        pick(&s, &mut r, (x + dx).round() as u32, (y + dy).round() as u32),
                        &e.id,
                    ) {
                        hole = true;
                        break 'holes;
                    }
                }
            }
            assert!(hole, "outline interior must have pick-through holes");
        }
        if e.outline {
            let outline = cesium_viewer::outline::primitive(e).unwrap().unwrap();
            let mut found = false;
            'edges: for edge in outline.vertices.chunks(2) {
                let a = DVec3::from_array(edge[0].position.map(f64::from));
                let b = DVec3::from_array(edge[1].position.map(f64::from));
                let (x, y, _) = project(&s, m.transform_point3((a + b) * 0.5), 768, 512);
                for dy in -1..=1 {
                    for dx in -1..=1 {
                        let x = (x.floor() as i32 + dx) as u32;
                        let y = (y.floor() as i32 + dy) as u32;
                        if x >= 768 || y >= 512 {
                            continue;
                        }
                        let i = (y as usize * 768 + x as usize) * 4;
                        let matches_color = if e.outline_color[0] > 0.9
                            && e.outline_color[1] > 0.9
                            && e.outline_color[2] > 0.9
                        {
                            frame[i] > 220 && frame[i + 1] > 220 && frame[i + 2] > 220
                        } else if e.fill {
                            frame[i] < 12 && frame[i + 1] < 12 && frame[i + 2] < 12
                        } else {
                            frame[i] > 170 && frame[i + 1] > 170 && frame[i + 2] < 100
                        };
                        if matches_color && is_id(&s, pick(&s, &mut r, x, y), &e.id) {
                            found = true;
                            break 'edges;
                        }
                    }
                }
            }
            assert!(
                found,
                "original black/yellow outline must draw and return its entity ID"
            );
        }
        assert!(!p.vertices.is_empty());
    }
    if is_primitive && !is_outline {
        let mut flat = specs.clone();
        for e in &mut flat {
            e.surface_appearance.as_mut().unwrap().flat = true;
        }
        s.set_entities(&mut r, flat).unwrap();
        let unlit = pixels(&mut r, &s).unwrap();
        assert!(
            frame
                .chunks(4)
                .zip(unlit.chunks(4))
                .filter(|(a, b)| a != b)
                .count()
                > 100,
            "source lighting must visibly affect filled primitives"
        );
        s.set_entities(&mut r, specs.clone()).unwrap();
        if id == "cylinder-dev" {
            let mut low = specs.clone();
            for e in &mut low {
                e.color[3] = 0.25;
            }
            s.set_entities(&mut r, low).unwrap();
            assert_eq!(
                frame,
                pixels(&mut r, &s).unwrap(),
                "translucent=false ignores attribute alpha for blending"
            );
            s.set_entities(&mut r, specs.clone()).unwrap();
        }
    }
    // The original half-alpha entity produces a distinct frame from alpha=1 with unchanged camera.
    if let Some(mut opaque) = specs
        .iter()
        .find(|e| e.color[3] > 0. && e.color[3] < 1.)
        .cloned()
    {
        opaque.color[3] = 1.;
        s.upsert_entities(&mut r, vec![opaque]).unwrap();
        let solid = pixels(&mut r, &s).unwrap();
        let changed = frame
            .chunks(4)
            .zip(solid.chunks(4))
            .filter(|(a, b)| a != b)
            .count();
        assert!(changed > 100, "half alpha must visibly blend");
    }
    if id == "czml-point" {
        // 20 px fill + a 4 px rim: visible and pickable coverage stays 28 px across camera zoom.
        for range in [1000., 10000.] {
            s.look_at(
                [-111., 40., 0.],
                cesium_viewer::camera_control::LookOffset {
                    heading: 0.,
                    pitch: -std::f64::consts::FRAC_PI_2,
                    range,
                },
            )
            .unwrap();
            let image = pixels(&mut r, &s).unwrap();
            let at = |x: usize, y: usize| &image[(y * 768 + x) * 4..(y * 768 + x) * 4 + 3];
            let center = at(384, 256);
            assert!(center.iter().all(|v| *v > 220), "white point center");
            let rim = at(396, 256);
            assert!(rim[0] > 220 && rim[1] < 15 && rim[2] < 15, "red 4px rim");
            assert!(is_id(&s, pick(&s, &mut r, 396, 256), "point 1"));
            assert!(!is_id(&s, pick(&s, &mut r, 400, 256), "point 1"));
        }
    }
    if id == "sphere-dev" || id == "box-dev" {
        s.set_entities(&mut r, specs.clone()).unwrap();
        let c = cesium_math::Cartographic::from_ecef(specs[0].position(0.).unwrap()).unwrap();
        s.look_at(
            [c.longitude.to_degrees(), c.latitude.to_degrees(), c.height],
            cesium_viewer::camera_control::LookOffset {
                heading: 0.,
                pitch: -0.1,
                range: 100000.,
            },
        )
        .unwrap();
        let inside = is_id(&s, pick(&s, &mut r, 384, 256), &specs[0].id);
        if id == "box-dev" {
            assert!(
                inside,
                "closed translucent back-face pass must remain visible from inside"
            );
        } else {
            assert!(!inside, "closed opaque primitive culls interior backfaces");
            let mut open = specs.clone();
            open[0].surface_appearance.as_mut().unwrap().closed = false;
            s.set_entities(&mut r, open).unwrap();
            assert!(is_id(&s, pick(&s, &mut r, 384, 256), &specs[0].id));
        }
    }
    // Selection changes, removal, framing failure and source replacement cannot leave stale camera state.
    let before = s.orbit.camera(1, 1);
    assert!(s.zoom_to_entities(&["missing".into()], 768, 512).is_err());
    assert_eq!(before.eye, s.orbit.camera(1, 1).eye);
    s.clear_entities();
    assert!(s.zoom_to_entities(&[], 768, 512).is_err());
}
#[test]
#[ignore = "requires native GPU or Mesa Vulkan"]
fn complete_box() {
    pollster::block_on(verify(
        "box",
        include_bytes!("../../../tests/sandcastle/full/box.json"),
    ));
}
#[test]
#[ignore = "requires native GPU or Mesa Vulkan"]
fn complete_cylinders_and_cones() {
    pollster::block_on(verify(
        "cylinders-and-cones",
        include_bytes!("../../../tests/sandcastle/full/cylinders-and-cones.json"),
    ));
}
#[test]
#[ignore = "requires native GPU or Mesa Vulkan"]
fn complete_plane() {
    pollster::block_on(verify(
        "plane",
        include_bytes!("../../../tests/sandcastle/full/plane.json"),
    ));
}

#[test]
#[ignore = "requires native GPU or Mesa Vulkan"]
fn complete_spheres_and_ellipsoids() {
    pollster::block_on(verify(
        "spheres-and-ellipsoids",
        include_bytes!("../../../tests/sandcastle/full/spheres-and-ellipsoids.json"),
    ));
}
#[test]
#[ignore = "requires native GPU or Mesa Vulkan"]
fn complete_czml_spheres_and_ellipsoids() {
    pollster::block_on(verify(
        "czml-spheres-and-ellipsoids",
        include_bytes!("../../../tests/sandcastle/full/czml-spheres-and-ellipsoids.czml"),
    ));
}
#[test]
#[ignore = "requires native GPU or Mesa Vulkan"]
fn complete_czml_cones_and_cylinders() {
    pollster::block_on(verify(
        "czml-cones-and-cylinders",
        include_bytes!("../../../tests/sandcastle/full/czml-cones-and-cylinders.czml"),
    ));
}

#[test]
#[ignore = "requires native GPU or Mesa Vulkan"]
fn complete_czml_box() {
    pollster::block_on(verify(
        "czml-box",
        include_bytes!("../../../tests/sandcastle/full/czml-box.czml"),
    ));
}

#[test]
#[ignore = "requires native GPU or Mesa Vulkan"]
fn complete_wall() {
    pollster::block_on(verify(
        "wall",
        include_bytes!("../../../tests/sandcastle/full/wall.json"),
    ));
}

#[test]
#[ignore = "requires native GPU or Mesa Vulkan"]
fn complete_czml_wall() {
    pollster::block_on(verify(
        "czml-wall",
        include_bytes!("../../../tests/sandcastle/full/czml-wall.czml"),
    ));
}

#[test]
#[ignore = "requires native GPU or Mesa Vulkan"]
fn complete_circles_and_ellipses() {
    pollster::block_on(verify(
        "circles-and-ellipses",
        include_bytes!("../../../tests/sandcastle/full/circles-and-ellipses.json"),
    ));
}

#[test]
#[ignore = "requires native GPU or Mesa Vulkan"]
fn complete_czml_circles_and_ellipses() {
    pollster::block_on(verify(
        "czml-circles-and-ellipses",
        include_bytes!("../../../tests/sandcastle/full/czml-circles-and-ellipses.czml"),
    ));
}

#[test]
#[ignore = "requires native GPU or Mesa Vulkan"]
fn complete_czml_point() {
    pollster::block_on(verify(
        "czml-point",
        include_bytes!("../../../tests/sandcastle/full/czml-point.czml"),
    ));
}

#[test]
#[ignore = "requires native GPU or Mesa Vulkan"]
fn complete_box_outline_dev() {
    pollster::block_on(verify(
        "box-outline-dev",
        include_bytes!("../../../tests/sandcastle/full/box-outline-dev.primitive.json"),
    ));
}

#[test]
#[ignore = "requires native GPU or Mesa Vulkan"]
fn complete_cylinder_outline_dev() {
    pollster::block_on(verify(
        "cylinder-outline-dev",
        include_bytes!("../../../tests/sandcastle/full/cylinder-outline-dev.primitive.json"),
    ));
}

#[test]
#[ignore = "requires native GPU or Mesa Vulkan"]
fn complete_sphere_outline_dev() {
    pollster::block_on(verify(
        "sphere-outline-dev",
        include_bytes!("../../../tests/sandcastle/full/sphere-outline-dev.primitive.json"),
    ));
}

#[test]
#[ignore = "requires native GPU or Mesa Vulkan"]
fn complete_ellipsoid_outline_dev() {
    pollster::block_on(verify(
        "ellipsoid-outline-dev",
        include_bytes!("../../../tests/sandcastle/full/ellipsoid-outline-dev.primitive.json"),
    ));
}

#[test]
#[ignore = "requires native GPU or Mesa Vulkan"]
fn complete_rectangle_outline_dev() {
    pollster::block_on(verify(
        "rectangle-outline-dev",
        include_bytes!("../../../tests/sandcastle/full/rectangle-outline-dev.primitive.json"),
    ));
}

#[test]
#[ignore = "requires native GPU or Mesa Vulkan"]
fn complete_wall_outline_dev() {
    pollster::block_on(verify(
        "wall-outline-dev",
        include_bytes!("../../../tests/sandcastle/full/wall-outline-dev.primitive.json"),
    ));
}

#[test]
#[ignore = "requires native GPU or Mesa Vulkan"]
fn complete_box_dev() {
    pollster::block_on(verify(
        "box-dev",
        include_bytes!("../../../tests/sandcastle/full/box-dev.primitive.json"),
    ));
}

#[test]
#[ignore = "requires native GPU or Mesa Vulkan"]
fn complete_cylinder_dev() {
    pollster::block_on(verify(
        "cylinder-dev",
        include_bytes!("../../../tests/sandcastle/full/cylinder-dev.primitive.json"),
    ));
}

#[test]
#[ignore = "requires native GPU or Mesa Vulkan"]
fn complete_sphere_dev() {
    pollster::block_on(verify(
        "sphere-dev",
        include_bytes!("../../../tests/sandcastle/full/sphere-dev.primitive.json"),
    ));
}

#[test]
#[ignore = "requires native GPU or Mesa Vulkan"]
fn complete_ellipsoid_dev() {
    pollster::block_on(verify(
        "ellipsoid-dev",
        include_bytes!("../../../tests/sandcastle/full/ellipsoid-dev.primitive.json"),
    ));
}

#[test]
#[ignore = "requires native GPU or Mesa Vulkan"]
fn complete_wall_dev() {
    pollster::block_on(verify(
        "wall-dev",
        include_bytes!("../../../tests/sandcastle/full/wall-dev.primitive.json"),
    ));
}

#[test]
#[ignore = "requires native GPU or Mesa Vulkan"]
fn complete_ellipse_dev() {
    pollster::block_on(verify(
        "ellipse-dev",
        include_bytes!("../../../tests/sandcastle/full/ellipse-dev.primitive.json"),
    ));
}

async fn verify_extended(id: &str, data: &[u8]) {
    let base = url::Url::from_file_path(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join(format!("../../tests/sandcastle/full/{id}.primitive.json")),
    )
    .unwrap();
    let hydrated = if id.ends_with("-dev") {
        Some(
            cesium_viewer::entity_resources::hydrate(data, &base, |u, _| {
                std::fs::read(u.to_file_path().unwrap()).map_err(|e| e.to_string())
            })
            .unwrap(),
        )
    } else {
        None
    };
    let data = hydrated.as_deref().unwrap_or(data);
    let (_, d, q) = device(&instance(), None).await.unwrap();
    let mut r = Renderer::new(d, q, wgpu::TextureFormat::Rgba8UnormSrgb, 768, 512).unwrap();
    let mut s = Scene::new(&mut r, None).unwrap();
    let specs = if id == "show-or-hide-entities" {
        serde_json::from_slice::<cesium_viewer::entity_groups::GroupedEntities>(data)
            .unwrap()
            .entities
    } else if [
        "corridor",
        "polygon",
        "points",
        "polyline",
        "polyline-dash",
        "polyline-volume",
        "rectangle",
        "z-indexing-geometry",
    ]
    .contains(&id)
    {
        let base = url::Url::from_file_path(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join(format!("../../tests/sandcastle/full/{id}.json")),
        )
        .unwrap();
        let hydrated = cesium_viewer::entity_resources::hydrate(data, &base, |u, _| {
            std::fs::read(u.to_file_path().unwrap()).map_err(|e| e.to_string())
        })
        .unwrap();
        entities::decode(&hydrated).unwrap()
    } else if id.starts_with("czml-") {
        let base = url::Url::from_file_path(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join(format!("../../tests/sandcastle/full/{id}.czml")),
        )
        .unwrap();
        let hydrated = cesium_viewer::entity_resources::hydrate(data, &base, |u, _| {
            std::fs::read(u.to_file_path().unwrap()).map_err(|e| e.to_string())
        })
        .unwrap();
        let (e, c) = entities::czml(&hydrated).unwrap();
        s.clock = c;
        e
    } else {
        cesium_viewer::primitive_instances::decode(data).unwrap()
    };
    let expected: Vec<entities::Entity> = serde_json::from_slice(
        &std::fs::read(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join(format!("../../tests/sandcastle/full/{id}.json")),
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(specs.len(), expected.len());
    s.set_entities(&mut r, specs.clone()).unwrap();
    // Preserve source no-zoom behavior; fitting below is an explicit test/user action.
    if id != "czml-colors" {
        s.set_view(cesium_viewer::camera_control::ViewOptions {
            destination_degrees: [-100., 40., 17500000.],
            heading: 0.,
            pitch: -std::f64::consts::FRAC_PI_2,
            roll: 0.,
        })
        .unwrap();
    }
    s.zoom_to_entities(&[], 768, 512).unwrap();
    let frame = pixels(&mut r, &s).unwrap();
    common::save_png(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join(format!("../../docs/full-{id}-preview.png")),
        &frame,
        768,
        512,
        image::ColorType::Rgba8,
    )
    .unwrap();
    for e in &specs {
        let mut one = e.clone();
        one.show = false;
        if let Some([start, _]) = e.availability {
            s.clock.current_time = start + 1.0;
        }
        s.set_entities(&mut r, vec![e.clone()]).unwrap();
        s.zoom_to_entities(&[], 768, 512).unwrap();
        let visible = pixels(&mut r, &s).unwrap();
        s.set_entities(&mut r, vec![one]).unwrap();
        let hidden = pixels(&mut r, &s).unwrap();
        let mut changed: Vec<_> = visible
            .as_chunks::<4>()
            .0
            .iter()
            .zip(hidden.as_chunks::<4>().0.iter())
            .enumerate()
            .filter_map(|(i, (a, b))| {
                let d = (0..3)
                    .map(|k| (a[k] as i32 - b[k] as i32).unsigned_abs())
                    .sum::<u32>();
                (d > 6).then_some((d, i))
            })
            .collect();
        assert!(changed.len() > 8, "{id}/{} must draw actual pixels", e.id);
        changed.sort_by_key(|(d, _)| std::cmp::Reverse(*d));
        s.set_entities(&mut r, vec![e.clone()]).unwrap();
        let mut found = false;
        for (_, i) in changed.iter().step_by((changed.len() / 48).max(1)).take(48) {
            if is_id(
                &s,
                pick(&s, &mut r, (*i % 768) as u32, (*i / 768) as u32),
                &e.id,
            ) {
                found = true;
                break;
            }
        }
        assert!(found, "{id}/{} real visible pixels must return ID", e.id);
        if !e.fill {
            assert!(
                changed.len() < 768 * 512 / 8,
                "one-pixel lines must retain empty interiors"
            );
        }
        if id.ends_with("-dev") {
            let before = serde_json::to_value(s.entity_specs()).unwrap();
            assert!(s
                .set_primitive_instances(&mut r, br#"{"primitives":[]}"#)
                .is_err());
            assert_eq!(before, serde_json::to_value(s.entity_specs()).unwrap());
        }
    }
    if id == "show-or-hide-entities" {
        s.set_grouped_entities(&mut r, data).unwrap();
        s.zoom_to_entities(&[], 768, 512).unwrap();
        let groups = s.entity_groups().to_vec();
        assert_eq!(groups.len(), 3);
        for g in &groups {
            let visible = pixels(&mut r, &s).unwrap();
            assert!(s.show_entity_group(&g.id, false));
            let hidden = pixels(&mut r, &s).unwrap();
            assert_ne!(visible, hidden, "parent hides its real geometry");
            assert!(s.entity_specs().iter().all(|e| e.show));
            assert!(s.show_entity_group(&g.id, true));
            assert_eq!(visible, pixels(&mut r, &s).unwrap());
        }
        let child = &groups[0].entity_ids[0];
        s.show_entity(child, false);
        s.show_entity_group(&groups[0].id, false);
        s.show_entity_group(&groups[0].id, true);
        assert!(
            !s.entity_specs()
                .iter()
                .find(|e| &e.id == child)
                .unwrap()
                .show
        );
        let before = serde_json::to_value(s.entity_specs()).unwrap();
        let mut bad: serde_json::Value = serde_json::from_slice(data).unwrap();
        bad["groups"][0]["entityIds"][0] = "missing".into();
        assert!(s
            .set_grouped_entities(&mut r, &serde_json::to_vec(&bad).unwrap())
            .is_err());
        assert_eq!(before, serde_json::to_value(s.entity_specs()).unwrap());
        assert_eq!(s.entity_groups().len(), 3);
    }
    if id == "czml-point-time-dynamic" {
        let e = &specs[0];
        s.set_entities(&mut r, specs.clone()).unwrap();
        for offset in [0., 50., 100., 150., 200., 250., 300.] {
            s.clock.current_time = s.clock.start_time + offset;
            s.tick(0.);
            s.zoom_to_entities(&[], 768, 512).unwrap();
            assert!(e.position(s.clock.current_time).is_some());
            let position = e.position(s.clock.current_time).unwrap();
            let height = cesium_math::Cartographic::from_ecef(position)
                .unwrap()
                .height;
            let selected = is_id(&s, pick(&s, &mut r, 384, 256), &e.id);
            assert_eq!(
                selected,
                height > 0.,
                "source ECEF chord respects ellipsoid occlusion at {offset}s, height {height}"
            );
        }
        s.clock.should_animate = true;
        s.clock.current_time = s.clock.stop_time - 1.;
        s.clock.tick(1.);
        assert!((s.clock.current_time - s.clock.start_time - 1.5).abs() < 1e-6);
    }
    // Selection does not change geometry ownership; repeated replacements remain bounded.
    s.set_entities(&mut r, specs.clone()).unwrap();
    assert_eq!(s.entity_specs().len(), expected.len());
    let smoke_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(format!("../../docs/full-{id}-smoke.log"));
    if !smoke_path.exists()
        || std::fs::read_to_string(&smoke_path)
            .unwrap_or_default()
            .trim()
            .is_empty()
    {
        let _ = s.zoom_to_entities(&[], 768, 512);
        let t = attachment(
            &r.device,
            768,
            512,
            r.format(),
            wgpu::TextureUsages::RENDER_ATTACHMENT,
        );
        let view = t.create_view(&Default::default());
        for _ in 0..90 {
            s.tick(1.0 / 60.0);
            let _ = s.render(&mut r, &view, None);
        }
        let stats = s.stats();
        let camera = s.camera_state();
        let text = format!("native smoke: 90 frames; {stats}\ncamera state: {camera}\n");
        std::fs::write(&smoke_path, text).unwrap();
    }
}

#[test]
#[ignore = "requires native GPU or Mesa Vulkan"]
fn complete_ellipse_outline_dev() {
    pollster::block_on(verify_extended(
        "ellipse-outline-dev",
        include_bytes!("../../../tests/sandcastle/full/ellipse-outline-dev.primitive.json"),
    ));
}

#[test]
#[ignore = "requires native GPU or Mesa Vulkan"]
fn complete_circle_outline_dev() {
    pollster::block_on(verify_extended(
        "circle-outline-dev",
        include_bytes!("../../../tests/sandcastle/full/circle-outline-dev.primitive.json"),
    ));
}

#[test]
#[ignore = "requires native GPU or Mesa Vulkan"]
fn complete_rectangle_dev() {
    pollster::block_on(verify_extended(
        "rectangle-dev",
        include_bytes!("../../../tests/sandcastle/full/rectangle-dev.primitive.json"),
    ));
}

#[test]
#[ignore = "requires native GPU or Mesa Vulkan"]
fn complete_per_instance_color_dev() {
    pollster::block_on(verify_extended(
        "per-instance-color-dev",
        include_bytes!("../../../tests/sandcastle/full/per-instance-color-dev.primitive.json"),
    ));
}

#[test]
#[ignore = "requires native GPU or Mesa Vulkan"]
fn complete_polygon_outline_dev() {
    pollster::block_on(verify_extended(
        "polygon-outline-dev",
        include_bytes!("../../../tests/sandcastle/full/polygon-outline-dev.primitive.json"),
    ));
}

#[test]
#[ignore = "requires native GPU or Mesa Vulkan"]
fn complete_coplanar_polygon_outline_dev() {
    pollster::block_on(verify_extended(
        "coplanar-polygon-outline-dev",
        include_bytes!(
            "../../../tests/sandcastle/full/coplanar-polygon-outline-dev.primitive.json"
        ),
    ));
}

#[test]
#[ignore = "requires native GPU or Mesa Vulkan"]
fn complete_simple_polyline_dev() {
    pollster::block_on(verify_extended(
        "simple-polyline-dev",
        include_bytes!("../../../tests/sandcastle/full/simple-polyline-dev.primitive.json"),
    ));
}

#[test]
#[ignore = "requires native GPU or Mesa Vulkan"]
fn complete_material_dev() {
    pollster::block_on(verify_extended(
        "material-dev",
        include_bytes!("../../../tests/sandcastle/full/material-dev.primitive.json"),
    ));
}

#[test]
#[ignore = "requires native GPU or Mesa Vulkan"]
fn complete_ellipsoid_surface_dev() {
    pollster::block_on(verify_extended(
        "ellipsoid-surface-dev",
        include_bytes!("../../../tests/sandcastle/full/ellipsoid-surface-dev.primitive.json"),
    ));
}

#[test]
#[ignore = "requires native GPU or Mesa Vulkan"]
fn complete_czml_colors() {
    pollster::block_on(verify_extended(
        "czml-colors",
        include_bytes!("../../../tests/sandcastle/full/czml-colors.czml"),
    ));
}

#[test]
#[ignore = "requires native GPU or Mesa Vulkan"]
fn complete_czml_point_time_dynamic() {
    pollster::block_on(verify_extended(
        "czml-point-time-dynamic",
        include_bytes!("../../../tests/sandcastle/full/czml-point-time-dynamic.czml"),
    ));
}

#[test]
#[ignore = "requires native GPU or Mesa Vulkan"]
fn complete_circle_dev() {
    pollster::block_on(verify_extended(
        "circle-dev",
        include_bytes!("../../../tests/sandcastle/full/circle-dev.primitive.json"),
    ));
}

#[test]
#[ignore = "requires native GPU or Mesa Vulkan"]
fn complete_corridor() {
    pollster::block_on(verify_extended(
        "corridor",
        include_bytes!("../../../tests/sandcastle/full/corridor.json"),
    ));
}

#[test]
#[ignore = "requires native GPU or Mesa Vulkan"]
fn complete_corridor_dev() {
    pollster::block_on(verify_extended(
        "corridor-dev",
        include_bytes!("../../../tests/sandcastle/full/corridor-dev.primitive.json"),
    ));
}

#[test]
#[ignore = "requires native GPU or Mesa Vulkan"]
fn complete_corridor_outline_dev() {
    pollster::block_on(verify_extended(
        "corridor-outline-dev",
        include_bytes!("../../../tests/sandcastle/full/corridor-outline-dev.primitive.json"),
    ));
}

#[test]
#[ignore = "requires native GPU or Mesa Vulkan"]
fn complete_czml_corridor() {
    pollster::block_on(verify_extended(
        "czml-corridor",
        include_bytes!("../../../tests/sandcastle/full/czml-corridor.czml"),
    ));
}
#[test]
#[ignore = "requires native GPU or Mesa Vulkan"]
fn complete_czml_position_definitions() {
    pollster::block_on(verify_extended(
        "czml-position-definitions",
        include_bytes!("../../../tests/sandcastle/full/czml-position-definitions.czml"),
    ));
}
#[test]
#[ignore = "requires native GPU or Mesa Vulkan"]
fn complete_show_or_hide_entities() {
    pollster::block_on(verify_extended(
        "show-or-hide-entities",
        include_bytes!("../../../tests/sandcastle/full/show-or-hide-entities.groups.json"),
    ));
}
async fn verify_scene(id: &str) {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(format!("../../tests/sandcastle/full/{id}.scene.json"));
    let recipe = cesium_viewer::scene_recipes::decode(&std::fs::read(path).unwrap()).unwrap();
    let (_, d, q) = device(&instance(), None).await.unwrap();
    let mut r = Renderer::new(d, q, wgpu::TextureFormat::Rgba8UnormSrgb, 768, 512).unwrap();
    let mut s = Scene::new(&mut r, None).unwrap();
    s.set_view(cesium_viewer::camera_control::ViewOptions {
        destination_degrees: [-100., 40., 17500000.],
        heading: 0.,
        pitch: -std::f64::consts::FRAC_PI_2,
        roll: 0.,
    })
    .unwrap();
    let frame = pixels(&mut r, &s).unwrap();
    assert!(
        frame
            .as_chunks::<4>()
            .0
            .iter()
            .filter(|p| p[0] > 20 || p[1] > 20 || p[2] > 20)
            .count()
            > 1000
    );
    common::save_png(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join(format!("../../docs/full-{id}-preview.png")),
        &frame,
        768,
        512,
        image::ColorType::Rgba8,
    )
    .unwrap();
    if let cesium_viewer::scene_recipes::SceneRecipe::HtmlOverlays { overlay } = recipe {
        let p = DVec3::from_array(overlay.position_cartesian);
        let xy = s.cartesian_to_canvas(p, 768, 512).unwrap();
        let expected = project(&s, p, 768, 512);
        assert!((xy[0] - expected.0).abs() < 1e-8 && (xy[1] - expected.1).abs() < 1e-8);
        assert!(xy[0] > 0. && xy[0] < 768. && xy[1] > 0. && xy[1] < 512.);
        assert!(s.cartesian_to_canvas(p, 0, 512).is_none());
        assert!(s
            .cartesian_to_canvas(DVec3::splat(f64::NAN), 768, 512)
            .is_none());
        let c = s.orbit.camera(768, 512);
        assert!(s
            .cartesian_to_canvas(c.eye + c.camera_to_ecef * DVec3::Z, 768, 512)
            .is_none());
    }
    s.orbit.zoom(0.2);
    assert_ne!(
        frame,
        pixels(&mut r, &s).unwrap(),
        "globe zoom changes framebuffer"
    );
}
#[test]
#[ignore = "requires native GPU or Mesa Vulkan; DOM behavior tested in Chromium"]
fn complete_html_overlays() {
    pollster::block_on(verify_scene("html-overlays"));
}
#[test]
#[ignore = "requires native GPU or Mesa Vulkan; physical/CSS resolution tested in Chromium"]
fn complete_resolution_scaling() {
    pollster::block_on(verify_scene("resolution-scaling"));
}
#[test]
#[ignore = "requires native GPU or Mesa Vulkan"]
fn complete_hello_world() {
    pollster::block_on(verify_scene("hello-world"));
}

#[test]
#[ignore = "requires GPU"]
fn complete_polygon() {
    pollster::block_on(verify_extended(
        "polygon",
        include_bytes!("../../../tests/sandcastle/full/polygon.json"),
    ));
}
#[test]
#[ignore = "requires GPU"]
fn complete_polygon_dev() {
    pollster::block_on(verify_extended(
        "polygon-dev",
        include_bytes!("../../../tests/sandcastle/full/polygon-dev.primitive.json"),
    ));
}
#[test]
#[ignore = "requires GPU"]
fn complete_coplanar_polygon_dev() {
    pollster::block_on(verify_extended(
        "coplanar-polygon-dev",
        include_bytes!("../../../tests/sandcastle/full/coplanar-polygon-dev.primitive.json"),
    ));
}
#[test]
#[ignore = "requires GPU"]
fn complete_czml_polygon() {
    pollster::block_on(verify_extended(
        "czml-polygon",
        include_bytes!("../../../tests/sandcastle/full/czml-polygon.czml"),
    ));
}
#[test]
#[ignore = "requires GPU"]
fn complete_points() {
    pollster::block_on(async {
        verify_extended(
            "points",
            include_bytes!("../../../tests/sandcastle/full/points.json"),
        )
        .await;
        let raw = include_bytes!("../../../tests/sandcastle/full/points.variants.json");
        let variants = cesium_viewer::entity_variants::decode(raw).unwrap();
        let (_, d, q) = device(&instance(), None).await.unwrap();
        let mut r = Renderer::new(d, q, wgpu::TextureFormat::Rgba8UnormSrgb, 768, 512).unwrap();
        let mut s = Scene::new(&mut r, None).unwrap();
        for (i, v) in variants.variants.iter().enumerate() {
            s.set_entity_variant(&mut r, raw, i).unwrap();
            assert_eq!(s.entity_specs().len(), v.entities.len());
            for e in &v.entities {
                s.set_entities(&mut r, vec![e.clone()]).unwrap();
                s.zoom_to_entities(&[], 768, 512).unwrap();
                let visible = pixels(&mut r, &s).unwrap();
                s.show_entity(&e.id, false);
                let hidden = pixels(&mut r, &s).unwrap();
                assert_ne!(visible, hidden, "each source menu alternative draws pixels");
                s.show_entity(&e.id, true);
                assert!(
                    is_id(&s, pick(&s, &mut r, 384, 256), &e.id),
                    "menu alternative {i} pick"
                );
            }
        }
        let before = serde_json::to_value(s.entity_specs()).unwrap();
        assert!(s.set_entity_variant(&mut r, raw, 999).is_err());
        assert_eq!(before, serde_json::to_value(s.entity_specs()).unwrap());
    });
}
#[test]
#[ignore = "requires GPU"]
fn complete_offline() {
    pollster::block_on(async {
        let (_, d, q) = device(&instance(), None).await.unwrap();
        let mut r = Renderer::new(d, q, wgpu::TextureFormat::Rgba8UnormSrgb, 768, 512).unwrap();
        let mut s = Scene::new(&mut r, None).unwrap();
        s.set_view(cesium_viewer::camera_control::ViewOptions {
            destination_degrees: [-100., 40., 17500000.],
            heading: 0.,
            pitch: -std::f64::consts::FRAC_PI_2,
            roll: 0.,
        })
        .unwrap();
        s.set_imagery(
            &mut r,
            cesium_viewer::scene_recipes::offline_provider().unwrap(),
        )
        .unwrap();
        let mut submitted = 0;
        for _ in 0..40 {
            s.update_imagery(&mut r).unwrap();
            let requests = s.imagery.as_mut().unwrap().take_requests(4);
            if requests.is_empty() {
                break;
            }
            for (tile, url) in requests {
                assert_eq!(url.scheme(), "file");
                let bytes = std::fs::read(url.to_file_path().unwrap()).unwrap();
                let image = image::load_from_memory(&bytes).unwrap().to_rgba8();
                s.imagery
                    .as_mut()
                    .unwrap()
                    .submit(&mut r, tile, image)
                    .unwrap();
                submitted += 1;
            }
        }
        s.update_imagery(&mut r).unwrap();
        assert!(submitted >= 2);
        let frame = pixels(&mut r, &s).unwrap();
        let colors: std::collections::HashSet<_> =
            frame.as_chunks::<4>().0.iter().copied().collect();
        assert!(colors.len() > 1000, "real local Natural Earth imagery");
        common::save_png(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../docs/full-offline-preview.png"),
            &frame,
            768,
            512,
            image::ColorType::Rgba8,
        )
        .unwrap();
    });
}

#[test]
#[ignore = "requires GPU"]
fn complete_polyline() {
    pollster::block_on(verify_extended(
        "polyline",
        include_bytes!("../../../tests/sandcastle/full/polyline.json"),
    ));
}

#[test]
#[ignore = "requires GPU"]
fn complete_polyline_dev() {
    pollster::block_on(verify_extended(
        "polyline-dev",
        include_bytes!("../../../tests/sandcastle/full/polyline-dev.primitive.json"),
    ));
}

#[test]
#[ignore = "requires GPU"]
fn complete_polyline_color_dev() {
    pollster::block_on(verify_extended(
        "polyline-color-dev",
        include_bytes!("../../../tests/sandcastle/full/polyline-color-dev.primitive.json"),
    ));
}

#[test]
#[ignore = "requires GPU"]
fn complete_polyline_material_dev() {
    pollster::block_on(verify_extended(
        "polyline-material-dev",
        include_bytes!("../../../tests/sandcastle/full/polyline-material-dev.primitive.json"),
    ));
}

#[test]
#[ignore = "requires GPU"]
fn complete_polyline_dash() {
    pollster::block_on(verify_extended(
        "polyline-dash",
        include_bytes!("../../../tests/sandcastle/full/polyline-dash.json"),
    ));
}

#[test]
#[ignore = "requires GPU"]
fn complete_czml_polyline() {
    pollster::block_on(verify_extended(
        "czml-polyline",
        include_bytes!("../../../tests/sandcastle/full/czml-polyline.czml"),
    ));
}

#[test]
#[ignore = "requires native GPU or Mesa Vulkan"]
fn complete_czml_rectangle() {
    pollster::block_on(verify(
        "czml-rectangle",
        include_bytes!("../../../tests/sandcastle/full/czml-rectangle.czml"),
    ));
}

#[test]
#[ignore = "requires native GPU or Mesa Vulkan"]
fn complete_partial_ellipsoids() {
    pollster::block_on(verify(
        "partial-ellipsoids",
        include_bytes!("../../../tests/sandcastle/full/partial-ellipsoids.json"),
    ));
}
#[test]
#[ignore = "requires GPU"]
fn wide_line_pixel_width_materials_and_picking() {
    pollster::block_on(async {
        use serde_json::json;
        let (_, d, q) = device(&instance(), None).await.unwrap();
        let mut r = Renderer::new(d, q, wgpu::TextureFormat::Rgba8UnormSrgb, 768, 512).unwrap();
        let mut s = Scene::new(&mut r, None).unwrap();
        let mut data = json!({"id":"line","shape":{"type":"widePolyline","data":{"positions":[[-100.3,35,100000],[-99.7,35,100000]],"width":18,"arcType":"NONE"}},"color":[1,0,0,1]});
        let mut thicknesses = vec![];
        for range in [200000., 400000.] {
            let e: entities::Entity = serde_json::from_value(data.clone()).unwrap();
            s.set_entities(&mut r, vec![e]).unwrap();
            s.set_view(cesium_viewer::camera_control::ViewOptions {
                destination_degrees: [-100., 35., 100000. + range],
                heading: 0.,
                pitch: -std::f64::consts::FRAC_PI_2,
                roll: 0.,
            })
            .unwrap();
            let a = pixels(&mut r, &s).unwrap();
            s.show_entity("line", false);
            let b = pixels(&mut r, &s).unwrap();
            s.show_entity("line", true);
            let ys: Vec<_> = (0..512)
                .filter(|y| {
                    let i = (y * 768 + 384) * 4;
                    (0..3)
                        .map(|k| (a[i + k] as i32 - b[i + k] as i32).abs())
                        .sum::<i32>()
                        > 20
                })
                .collect();
            assert!(
                (17..=20).contains(&ys.len()),
                "18-pixel line at {range}: {}",
                ys.len()
            );
            thicknesses.push(ys.len());
            assert!(is_id(&s, pick(&s, &mut r, 384, 256), "line"));
            assert!(!is_id(&s, pick(&s, &mut r, 384, 276), "line"));
        }
        assert!(thicknesses[0].abs_diff(thicknesses[1]) <= 1);
        data["shape"]["data"]["material"] =
            json!({"type":"dash","gapColor":[0,0,0,0],"dashLength":16,"dashPattern":255});
        s.set_entities(&mut r, vec![serde_json::from_value(data.clone()).unwrap()])
            .unwrap();
        let mut yes = 0;
        let mut no = 0;
        for x in 375..394 {
            if is_id(&s, pick(&s, &mut r, x, 256), "line") {
                yes += 1
            } else {
                no += 1
            }
        }
        assert!(
            yes >= 5 && no >= 5,
            "transparent dash pixels must not pick: {yes}/{no}"
        );
        data["shape"]["data"]["material"]["gapColor"] = json!([0, 1, 0, 1]);
        s.set_entities(&mut r, vec![serde_json::from_value(data.clone()).unwrap()])
            .unwrap();
        let frame = pixels(&mut r, &s).unwrap();
        for x in 375..394 {
            assert!(is_id(&s, pick(&s, &mut r, x, 256), "line"));
        }
        let cs: Vec<_> = (375..394)
            .map(|x| &frame[(256 * 768 + x) * 4..(256 * 768 + x) * 4 + 3])
            .collect();
        assert!(cs.iter().any(|c| c[0] > 200 && c[1] < 20));
        assert!(cs.iter().any(|c| c[1] > 200 && c[0] < 20));
        data["shape"]["data"]["material"] =
            json!({"type":"dash","gapColor":[0,0,0,0],"dashLength":16,"dashPattern":0});
        s.set_entities(&mut r, vec![serde_json::from_value(data).unwrap()])
            .unwrap();
        assert!(!is_id(&s, pick(&s, &mut r, 384, 256), "line"));
    });
}

#[test]
#[ignore = "requires native GPU or Mesa Vulkan"]
fn complete_polyline_volume() {
    pollster::block_on(verify_extended(
        "polyline-volume",
        include_bytes!("../../../tests/sandcastle/full/polyline-volume.json"),
    ));
}

#[test]
#[ignore = "requires native GPU or Mesa Vulkan"]
fn complete_polyline_volume_dev() {
    pollster::block_on(verify_extended(
        "polyline-volume-dev",
        include_bytes!("../../../tests/sandcastle/full/polyline-volume-dev.primitive.json"),
    ));
}

#[test]
#[ignore = "requires native GPU or Mesa Vulkan"]
fn complete_polyline_volume_outline_dev() {
    pollster::block_on(verify_extended(
        "polyline-volume-outline-dev",
        include_bytes!("../../../tests/sandcastle/full/polyline-volume-outline-dev.primitive.json"),
    ));
}

#[test]
#[ignore = "requires native GPU or Mesa Vulkan"]
fn complete_czml_polyline_volume() {
    pollster::block_on(verify_extended(
        "czml-polyline-volume",
        include_bytes!("../../../tests/sandcastle/full/czml-polyline-volume.czml"),
    ));
}

#[test]
#[ignore = "requires native GPU or Mesa Vulkan"]
fn complete_rectangle() {
    pollster::block_on(verify_extended(
        "rectangle",
        include_bytes!("../../../tests/sandcastle/full/rectangle.json"),
    ));
}

#[test]
#[ignore = "requires native GPU or Mesa Vulkan"]
fn complete_z_indexing_geometry() {
    pollster::block_on(verify_extended(
        "z-indexing-geometry",
        include_bytes!("../../../tests/sandcastle/full/z-indexing-geometry.json"),
    ));
}

#[test]
#[ignore = "requires native GPU or Mesa Vulkan"]
fn complete_czml_zindex() {
    pollster::block_on(verify_extended(
        "czml-zindex",
        include_bytes!("../../../tests/sandcastle/full/czml-zindex.czml"),
    ));
}

#[test]
#[ignore = "requires native GPU or Mesa Vulkan"]
fn complete_volumes_dev() {
    pollster::block_on(verify_extended(
        "volumes-dev",
        include_bytes!("../../../tests/sandcastle/full/volumes-dev.primitive.json"),
    ));
}

#[test]
#[ignore = "requires native GPU or Mesa Vulkan"]
fn complete_geometry_offset_attribute_dev() {
    pollster::block_on(verify_extended(
        "geometry-offset-attribute-dev",
        include_bytes!(
            "../../../tests/sandcastle/full/geometry-offset-attribute-dev.primitive.json"
        ),
    ));
}

#[test]
#[ignore = "requires native GPU or Mesa Vulkan"]
fn complete_geometry_offset_attribute_box_cylinder_ellipsoid_dev() {
    pollster::block_on(verify_extended(
        "geometry-offset-attribute-box-cylinder-ellipsoid-dev",
        include_bytes!("../../../tests/sandcastle/full/geometry-offset-attribute-box-cylinder-ellipsoid-dev.primitive.json"),
    ));
}

#[test]
#[ignore = "requires native GPU or Mesa Vulkan"]
fn complete_czml_polygon_intervals_availability() {
    pollster::block_on(verify_extended(
        "czml-polygon-intervals-availability",
        include_bytes!("../../../tests/sandcastle/full/czml-polygon-intervals-availability.czml"),
    ));
}

#[test]
#[ignore = "requires native GPU or Mesa Vulkan"]
fn complete_czml_custom_properties() {
    pollster::block_on(verify_extended(
        "czml-custom-properties",
        include_bytes!("../../../tests/sandcastle/full/czml-custom-properties.czml"),
    ));
}
