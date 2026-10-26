//! Offscreen real GPU verification; run with a Vulkan GPU or Mesa adapter.
use cesium_renderer::{attachment, device, instance, Renderer};
use cesium_viewer::{entities, Scene};
use std::time::Duration;
mod common;
use common::pixels;

#[test]
#[ignore = "requires native GPU or Mesa Vulkan"]
fn entities_draw_pick_motion_and_atomic_replacement() {
    pollster::block_on(async {
        let inst = instance();
        let (_, device, queue) = device(&inst, None).await.unwrap();
        let mut r =
            Renderer::new(device, queue, wgpu::TextureFormat::Rgba8UnormSrgb, 768, 512).unwrap();
        let mut s = Scene::new(&mut r, None).unwrap();
        let specs =
            entities::decode(include_bytes!("../../../tests/entities/shapes.json")).unwrap();
        s.set_entities(&mut r, specs).unwrap();
        let target = attachment(
            &r.device,
            768,
            512,
            r.format(),
            wgpu::TextureUsages::RENDER_ATTACHMENT,
        );
        let pick = |s: &Scene, r: &mut Renderer| {
            s.render(
                r,
                &target.create_view(&Default::default()),
                Some((384, 256)),
            )
            .unwrap();
            r.device
                .poll(wgpu::PollType::Wait {
                    submission_index: None,
                    timeout: Some(Duration::from_secs(30)),
                })
                .unwrap();
            r.poll_pick().unwrap()
        };
        for id in ["box", "ellipsoid", "cylinder", "cone", "plane"] {
            assert!(s.focus_entity(id, 40000.));
            let f = pick(&s, &mut r).expect("geometry center should draw and be pickable");
            assert_eq!(s.properties(f).unwrap()["id"], id);
        }
        s.focus_location(110., 25., 180000.).unwrap();
        let preview = pixels(&mut r, &s).unwrap();
        common::save_png(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../docs/entities-preview.png"),
            &preview,
            768,
            512,
            image::ColorType::Rgba8,
        )
        .unwrap();
        let (entities, clock) =
            entities::czml(include_bytes!("../../../tests/entities/motion.czml")).unwrap();
        s.set_entities(&mut r, entities).unwrap();
        s.clock = clock;
        s.tick(0.);
        assert!(s.focus_entity("moving-box", 40000.));
        let f = pick(&s, &mut r).unwrap();
        assert_eq!(s.properties(f).unwrap()["id"], "moving-box");
        let before = pixels(&mut r, &s).unwrap();
        s.clock.current_time += 30.;
        s.tick(0.);
        let after = pixels(&mut r, &s).unwrap();
        let changed = before
            .as_chunks::<4>()
            .0
            .iter()
            .zip(after.as_chunks::<4>().0.iter())
            .filter(|(a, b)| a != b)
            .count();
        assert!(
            changed > 100,
            "sample motion must change framebuffer pixels"
        );
        assert_ne!(pick(&s, &mut r), Some(f));
        assert!(s.focus_entity("moving-box", 40000.));
        assert!(pick(&s, &mut r).is_some());
        let mut invalid =
            entities::decode(include_bytes!("../../../tests/entities/shapes.json")).unwrap();
        invalid[0].color[3] = 1.5;
        assert!(s.set_entities(&mut r, invalid).is_err());
        assert!(
            s.focus_entity("moving-box", 40000.),
            "invalid replacement must preserve previous entities"
        );
        assert!(s.remove_entity("moving-box"));
        assert!(s.properties(f).is_none());
        assert!(pick(&s, &mut r).is_none());
    });
}
#[test]
#[ignore = "requires native GPU or Mesa Vulkan"]
fn geographic_holes_extrusion_visibility_and_upsert() {
    pollster::block_on(async {
        let inst = instance();
        let (_, device, queue) = device(&inst, None).await.unwrap();
        let mut r =
            Renderer::new(device, queue, wgpu::TextureFormat::Rgba8UnormSrgb, 768, 512).unwrap();
        let mut s = Scene::new(&mut r, None).unwrap();
        s.set_entities(
            &mut r,
            entities::decode(include_bytes!("../../../tests/entities/geographic.json")).unwrap(),
        )
        .unwrap();
        let target = attachment(
            &r.device,
            768,
            512,
            r.format(),
            wgpu::TextureUsages::RENDER_ATTACHMENT,
        );
        let pick = |s: &Scene, r: &mut Renderer| {
            s.render(
                r,
                &target.create_view(&Default::default()),
                Some((384, 256)),
            )
            .unwrap();
            r.device
                .poll(wgpu::PollType::Wait {
                    submission_index: None,
                    timeout: Some(Duration::from_secs(30)),
                })
                .unwrap();
            r.poll_pick().unwrap()
        };
        s.focus_location(109.4, 25.2, 80000.).unwrap();
        let camera = s.orbit.camera(768, 512);
        let direction = -camera.eye.normalize();
        let distance = cesium_math::ray_ellipsoid(camera.eye, direction).unwrap();
        let hit = cesium_math::Cartographic::from_ecef(camera.eye + direction * distance).unwrap();
        assert!((hit.latitude.to_degrees() - 25.2).abs() < 1e-8);
        assert!((hit.longitude.to_degrees() - 109.4).abs() < 1e-8);

        assert!(
            pick(&s, &mut r).is_none(),
            "polygon hole must expose non-pickable Earth"
        );
        s.focus_location(109.25, 25.2, 80000.).unwrap();
        let hole = pick(&s, &mut r).unwrap();
        assert_eq!(s.properties(hole).unwrap()["id"], "hole");
        for id in ["rectangle", "ellipse", "wall"] {
            assert!(s.focus_entity(id, 80000.));
            let f = pick(&s, &mut r).unwrap();
            assert_eq!(s.properties(f).unwrap()["id"], id);
        }
        s.focus_entity("ellipse", 60000.);
        let original = pick(&s, &mut r).unwrap();
        assert!(s.show_entity("ellipse", false));
        assert!(pick(&s, &mut r).is_none());
        assert!(s.properties(original).is_none());
        assert!(s.show_entity("ellipse", true));
        assert_eq!(pick(&s, &mut r), Some(original));
        let mut update = s
            .entity_specs()
            .into_iter()
            .find(|e| e.id == "rectangle")
            .unwrap();
        update.name = "Updated rectangle".into();
        s.upsert_entities(&mut r, vec![update.clone()]).unwrap();
        assert_eq!(
            pick(&s, &mut r),
            Some(original),
            "unrelated mesh IDs must remain stable"
        );
        assert_eq!(s.entity_specs().len(), 6);
        update.color[3] = 1.5;
        assert!(s.upsert_entities(&mut r, vec![update]).is_err());
        assert_eq!(pick(&s, &mut r), Some(original));
        s.focus_location(110., 25.25, 230000.).unwrap();
        let preview = pixels(&mut r, &s).unwrap();
        common::save_png(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../docs/geographic-entities-preview.png"),
            &preview,
            768,
            512,
            image::ColorType::Rgba8,
        )
        .unwrap();
        let (data, clock) =
            entities::czml(include_bytes!("../../../tests/entities/geographic.czml")).unwrap();
        s.set_entities(&mut r, data).unwrap();
        s.clock = clock;
        s.tick(0.);
        assert_eq!(s.entity_specs().len(), 4);
        assert!(s.focus_entity("wall", 80000.));
        assert_eq!(
            s.properties(pick(&s, &mut r).unwrap()).unwrap()["id"],
            "wall"
        );
    });
}
#[test]
#[ignore = "requires native GPU or Mesa Vulkan"]
fn entity_material_uvs_transparent_cells_and_czml_wall() {
    pollster::block_on(async {
        let inst = instance();
        let (_, device, queue) = device(&inst, None).await.unwrap();
        let mut r =
            Renderer::new(device, queue, wgpu::TextureFormat::Rgba8UnormSrgb, 768, 512).unwrap();
        let mut s = Scene::new(&mut r, None).unwrap();
        s.set_entities(
            &mut r,
            entities::decode(include_bytes!("../../../tests/entities/materials.json")).unwrap(),
        )
        .unwrap();
        let target = attachment(
            &r.device,
            768,
            512,
            r.format(),
            wgpu::TextureUsages::RENDER_ATTACHMENT,
        );
        let pick = |s: &Scene, r: &mut Renderer| {
            s.render(
                r,
                &target.create_view(&Default::default()),
                Some((384, 256)),
            )
            .unwrap();
            r.device
                .poll(wgpu::PollType::Wait {
                    submission_index: None,
                    timeout: Some(Duration::from_secs(30)),
                })
                .unwrap();
            r.poll_pick().unwrap()
        };
        s.focus_location(109.11, 25.2, 80000.).unwrap();
        assert!(
            pick(&s, &mut r).is_none(),
            "transparent stripe must allow a pick miss through to Earth"
        );
        s.focus_location(109.02, 25.2, 80000.).unwrap();
        let f = pick(&s, &mut r).unwrap();
        assert_eq!(s.properties(f).unwrap()["id"], "stripe");
        s.focus_location(110.225, 25.225, 80000.).unwrap();
        assert!(
            pick(&s, &mut r).is_none(),
            "empty grid cells should be unpickable"
        );
        s.focus_location(110.4, 25.2, 80000.).unwrap();
        assert_eq!(
            s.properties(pick(&s, &mut r).unwrap()).unwrap()["id"],
            "grid"
        );
        s.focus_location(109.8, 25., 230000.).unwrap();
        let preview = pixels(&mut r, &s).unwrap();
        common::save_png(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../docs/materials-preview.png"),
            &preview,
            768,
            512,
            image::ColorType::Rgba8,
        )
        .unwrap();
        let (data, _) =
            entities::czml(include_bytes!("../../../tests/entities/materials.czml")).unwrap();
        assert_eq!(data.len(), 5);
        assert!(data.iter().take(3).all(|e| e.material.is_some()));
        let (data, _) = entities::czml(include_bytes!(
            "../../../tests/entities/sandcastle-wall.czml"
        ))
        .unwrap();
        assert_eq!(data[0].color[3], 150. / 255.);
        s.set_entities(&mut r, data).unwrap();
        assert!(s.focus_entity("wall", 4000000.));
        assert_eq!(
            s.properties(pick(&s, &mut r).unwrap()).unwrap()["id"],
            "wall"
        );
    });
}
#[test]
#[ignore = "requires native GPU or Mesa Vulkan"]
fn images_outlines_fill_toggle_and_lifecycle() {
    pollster::block_on(async {
        let inst = instance();
        let (_, device, queue) = device(&inst, None).await.unwrap();
        let mut r =
            Renderer::new(device, queue, wgpu::TextureFormat::Rgba8UnormSrgb, 768, 512).unwrap();
        let mut s = Scene::new(&mut r, None).unwrap();
        let data =
            entities::decode(include_bytes!("../../../tests/entities/image-outline.json")).unwrap();
        s.set_entities(&mut r, data).unwrap();
        let target = attachment(
            &r.device,
            768,
            512,
            r.format(),
            wgpu::TextureUsages::RENDER_ATTACHMENT,
        );
        let pick = |s: &Scene, r: &mut Renderer, x, y| {
            s.render(r, &target.create_view(&Default::default()), Some((x, y)))
                .unwrap();
            r.device
                .poll(wgpu::PollType::Wait {
                    submission_index: None,
                    timeout: Some(Duration::from_secs(30)),
                })
                .unwrap();
            r.poll_pick().unwrap()
        };
        s.focus_location(109.2, 25.2, 80000.).unwrap();
        assert!(
            pick(&s, &mut r, 384, 256).is_none(),
            "transparent PNG hole must pass through"
        );
        s.focus_location(109.05, 25.2, 80000.).unwrap();
        assert_eq!(
            s.properties(pick(&s, &mut r, 384, 256).unwrap()).unwrap()["id"],
            "image"
        );
        s.focus_location(110.4, 25.2, 80000.).unwrap();
        assert!(
            pick(&s, &mut r, 384, 256).is_none(),
            "outline-only shape has no interior pick"
        );
        s.focus_location(110.2, 25.2, 80000.).unwrap();
        let outline = (255..=257)
            .flat_map(|y| (383..=385).map(move |x| (x, y)))
            .find_map(|(x, y)| pick(&s, &mut r, x, y))
            .expect("outline line should be pickable near boundary");
        assert_eq!(s.properties(outline).unwrap()["id"], "outline-only");
        assert!(s.show_entity("outline-only", false));
        assert!(s.properties(outline).is_none());
        assert!(s.show_entity("outline-only", true));
        let mut image = s
            .entity_specs()
            .into_iter()
            .find(|e| e.id == "image")
            .unwrap();
        if let Some(cesium_viewer::entity_material::SurfaceMaterial::Image {
            transparent, ..
        }) = &mut image.material
        {
            *transparent = false;
        }
        s.upsert_entities(&mut r, vec![image]).unwrap();
        s.focus_location(109.2, 25.2, 80000.).unwrap();
        assert_eq!(
            s.properties(pick(&s, &mut r, 384, 256).unwrap()).unwrap()["id"],
            "image",
            "opaque image mode ignores PNG alpha"
        );
        s.focus_location(109.8, 25.1, 230000.).unwrap();
        let preview = pixels(&mut r, &s).unwrap();
        common::save_png(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../docs/image-outline-preview.png"),
            &preview,
            768,
            512,
            image::ColorType::Rgba8,
        )
        .unwrap();
        assert!(s.remove_entity("outline-only"));
        assert!(s.properties(outline).is_none());
    });
}

#[test]
#[ignore = "requires native GPU or Mesa Vulkan"]
fn screen_pixels_zoom_depth_and_pick_lifecycle() {
    pollster::block_on(async {
        let (_, device, queue) = device(&instance(), None).await.unwrap();
        let mut r =
            Renderer::new(device, queue, wgpu::TextureFormat::Rgba8UnormSrgb, 768, 512).unwrap();
        let mut s = Scene::new(&mut r, None).unwrap();
        let all = entities::decode(include_bytes!("../../../tests/entities/screen.json")).unwrap();
        s.set_entities(&mut r, vec![all[0].clone()]).unwrap();
        let target = attachment(
            &r.device,
            768,
            512,
            r.format(),
            wgpu::TextureUsages::RENDER_ATTACHMENT,
        );
        let pick = |s: &Scene, r: &mut Renderer, x, y| {
            s.render(r, &target.create_view(&Default::default()), Some((x, y)))
                .unwrap();
            r.device
                .poll(wgpu::PollType::Wait {
                    submission_index: None,
                    timeout: Some(Duration::from_secs(30)),
                })
                .unwrap();
            r.poll_pick().unwrap()
        };
        for range in [40000., 400000.] {
            assert!(s.focus_entity("point", range));
            let data = pixels(&mut r, &s).unwrap();
            let red = |x: usize| {
                let i = (256 * 768 + x) * 4;
                data[i] > 220 && data[i + 1] < 120 && data[i + 2] < 120
            };
            assert_eq!(
                (350..420).filter(|&x| red(x)).count(),
                32,
                "point fill must stay 32 pixels at either distance"
            );
            assert_eq!(
                s.properties(pick(&s, &mut r, 384, 256).unwrap()).unwrap()["id"],
                "point"
            );
            assert!(
                pick(&s, &mut r, 402, 274).is_none(),
                "circle quad corners must not pick"
            );
            assert_eq!(
                s.properties(pick(&s, &mut r, 402, 256).unwrap()).unwrap()["id"],
                "point",
                "outline must pick parent"
            );
        }
        let old = pick(&s, &mut r, 384, 256).unwrap();
        assert!(s.show_entity("point", false));
        assert!(s.properties(old).is_none());
        assert!(pick(&s, &mut r, 384, 256).is_none());
        s.show_entity("point", true);
        let mut back = all[0].clone();
        back.id = "back".into();
        back.position_degrees = Some([-70.3, -25.1, 16000.]);
        s.upsert_entities(&mut r, vec![back]).unwrap();
        s.show_entity("point", false);
        assert!(
            pick(&s, &mut r, 384, 256).is_none(),
            "globe must occlude a far-side point"
        );
        s.set_entities(&mut r, vec![all[1].clone()]).unwrap();
        s.focus_entity("billboard", 40000.);
        assert!(
            pick(&s, &mut r, 384, 256).is_none(),
            "image transparent center must pass through"
        );
        let hit = pick(&s, &mut r, 408, 256).unwrap();
        assert_eq!(s.properties(hit).unwrap()["id"], "billboard");
        assert!(
            pick(&s, &mut r, 417, 256).is_none(),
            "64px image has bounded width"
        );
        s.set_entities(&mut r, vec![all[2].clone()]).unwrap();
        s.focus_entity("offset", 40000.);
        assert!(
            pick(&s, &mut r, 384, 256).is_none(),
            "pixelOffset displaces image from anchor"
        );
        assert_eq!(
            s.properties(pick(&s, &mut r, 444, 276).unwrap()).unwrap()["id"],
            "offset"
        );
        s.set_entities(&mut r, all).unwrap();
        s.focus_location(109.9, 25.1, 100000.).unwrap();
        common::save_png(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/screen-preview.png"),
            &pixels(&mut r, &s).unwrap(),
            768,
            512,
            image::ColorType::Rgba8,
        )
        .unwrap();
        s.remove_entity("offset");
        assert!(s.properties(hit).is_none());
        let base = url::Url::parse("file:///samples/screen-motion.czml").unwrap();
        let hydrated = cesium_viewer::entity_resources::hydrate(
            include_bytes!("../../../tests/entities/screen-motion.czml"),
            &base,
            |_, _| Ok(include_bytes!("../../../tests/entities/image-test.png").to_vec()),
        )
        .unwrap();
        let (specs, clock) = entities::czml(&hydrated).unwrap();
        s.clock = clock;
        s.set_entities(&mut r, specs).unwrap();
        s.tick(0.);
        s.focus_entity("point", 40000.);
        assert!(pick(&s, &mut r, 384, 256).is_some());
        let before = pixels(&mut r, &s).unwrap();
        s.clock.current_time += 30.;
        s.tick(0.);
        assert_ne!(before, pixels(&mut r, &s).unwrap());
        assert!(pick(&s, &mut r, 384, 256).is_none());
        assert!(s.focus_entity("point", 40000.));
        assert_eq!(
            s.properties(pick(&s, &mut r, 384, 256).unwrap()).unwrap()["id"],
            "point"
        );
    });
}
