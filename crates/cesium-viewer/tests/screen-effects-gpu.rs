//! Independent GPU tests for the five 0.0.15 effects.
mod common;
use cesium_renderer::{attachment, device, instance, FeatureRef, Renderer};
use cesium_viewer::{
    entities::{self, Entity},
    label_appearance::LabelStyle,
    Scene,
};
use common::pixels;
use std::time::Duration;
async fn setup() -> (Renderer, Scene) {
    let (_, d, q) = device(&instance(), None).await.unwrap();
    let mut r = Renderer::new(d, q, wgpu::TextureFormat::Rgba8UnormSrgb, 768, 512).unwrap();
    let s = Scene::new(&mut r, None).unwrap();
    (r, s)
}
fn specs(bytes: &[u8]) -> Vec<Entity> {
    entities::decode(bytes).unwrap()
}
fn pick(s: &Scene, r: &mut Renderer, x: u32, y: u32) -> Option<FeatureRef> {
    let target = attachment(
        &r.device,
        768,
        512,
        r.format(),
        wgpu::TextureUsages::RENDER_ATTACHMENT,
    );
    s.render(r, &target.create_view(&Default::default()), Some((x, y)))
        .unwrap();
    r.device
        .poll(wgpu::PollType::Wait {
            submission_index: None,
            timeout: Some(Duration::from_secs(30)),
        })
        .unwrap();
    r.poll_pick().unwrap()
}
#[test]
#[ignore = "requires native GPU or Mesa Vulkan"]
fn label_background_color_and_pick_rectangle() {
    pollster::block_on(async {
        let (mut r, mut s) = setup().await;
        let mut e = specs(include_bytes!(
            "../../../tests/entities/label-background.json"
        ))
        .remove(0);
        let p = e.primitive().unwrap();
        let image = &p.material.textures[0].as_ref().unwrap().image;
        let x = (384. - image.width as f64 / 2. + 3.).round() as u32;
        let y = (256. - image.height as f64 / 2. + 3.).round() as u32;
        s.set_entities(&mut r, vec![e.clone()]).unwrap();
        s.focus_entity(&e.id, 40000.);
        assert_eq!(
            s.properties(pick(&s, &mut r, x, y).unwrap()).unwrap()["id"],
            e.id
        );
        let frame = pixels(&mut r, &s).unwrap();
        let i = (y as usize * 768 + x as usize) * 4;
        assert!(
            frame[i + 2] > frame[i] + 60,
            "blue background is independent of white fill"
        );
        if let entities::Shape::Label { appearance, .. } = &mut e.shape {
            appearance.show_background = false;
        }
        s.upsert_entities(&mut r, vec![e]).unwrap();
        assert!(
            pick(&s, &mut r, x, y).is_none(),
            "background toggles independently"
        );
    });
}
#[test]
#[ignore = "requires native GPU or Mesa Vulkan"]
fn label_outline_color_hollow_ink_and_pick() {
    pollster::block_on(async {
        let (mut r, mut s) = setup().await;
        let mut e = specs(include_bytes!("../../../tests/entities/label-outline.json")).remove(0);
        s.set_entities(&mut r, vec![e.clone()]).unwrap();
        s.focus_entity(&e.id, 40000.);
        let frame = pixels(&mut r, &s).unwrap();
        let (x, y) = (200..310)
            .flat_map(|y| (200..568).map(move |x| (x, y)))
            .find(|&(x, y)| {
                let i = (y * 768 + x) * 4;
                frame[i] > 200 && frame[i + 1] > 140 && frame[i + 2] < 100
            })
            .unwrap();
        assert_eq!(
            s.properties(pick(&s, &mut r, x as u32, y as u32).unwrap())
                .unwrap()["id"],
            e.id
        );
        if let entities::Shape::Label {
            text,
            font_size,
            appearance,
            ..
        } = &mut e.shape
        {
            *text = "O".into();
            *font_size = 64.;
            appearance.style = LabelStyle::Outline;
        }
        let base = cesium_viewer::label::raster("O", 64.).unwrap();
        let bw = base.width as usize;
        let bh = base.height as usize;
        let ink = (1..bh - 1)
            .flat_map(|y| (1..bw - 1).map(move |x| (x, y)))
            .find(|&(x, y)| {
                (-1..=1).all(|dy| {
                    (-1..=1).all(|dx| {
                        base.texture.image.rgba[(((y as isize + dy) as usize * bw
                            + (x as isize + dx) as usize)
                            * 4)
                            + 3]
                            == 255
                    })
                })
            })
            .unwrap();
        let p = e.primitive().unwrap();
        let t = &p.material.textures[0].as_ref().unwrap().image;
        let x = (384. - t.width as f64 / 2. + ink.0 as f64 + 4. + 0.5).floor() as u32;
        let y = (256. - t.height as f64 / 2. + ink.1 as f64 + 4. + 0.5).floor() as u32;
        s.upsert_entities(&mut r, vec![e]).unwrap();
        assert!(
            pick(&s, &mut r, x, y).is_none(),
            "OUTLINE removes the original solid ink interior"
        );
    });
}
#[test]
#[ignore = "requires native GPU or Mesa Vulkan"]
fn billboard_rotation_preserves_pixel_offset() {
    pollster::block_on(async {
        let (mut r, mut s) = setup().await;
        let mut e = specs(include_bytes!(
            "../../../tests/entities/billboard-rotation.json"
        ))
        .remove(0);
        e.screen.rotation = 0.;
        s.set_entities(&mut r, vec![e.clone()]).unwrap();
        s.focus_entity(&e.id, 40000.);
        assert!(pick(&s, &mut r, 454, 256).is_some());
        assert!(pick(&s, &mut r, 414, 296).is_none());
        e.screen.rotation = std::f64::consts::FRAC_PI_2;
        s.upsert_entities(&mut r, vec![e]).unwrap();
        assert!(pick(&s, &mut r, 454, 256).is_none());
        assert!(
            pick(&s, &mut r, 414, 296).is_some(),
            "rectangle turns vertical around its fixed 30px offset"
        );
    });
}
#[test]
#[ignore = "requires native GPU or Mesa Vulkan"]
fn distance_pixel_offset_moves_without_resizing() {
    pollster::block_on(async {
        let (mut r, mut s) = setup().await;
        let e = specs(include_bytes!(
            "../../../tests/entities/pixel-offset-distance.json"
        ))
        .remove(0);
        s.set_entities(&mut r, vec![e.clone()]).unwrap();
        s.focus_entity(&e.id, 20000.);
        assert!(pick(&s, &mut r, 442, 276).is_some());
        assert!(pick(&s, &mut r, 522, 316).is_none());
        s.focus_entity(&e.id, 220000.);
        assert!(pick(&s, &mut r, 442, 276).is_none());
        assert!(pick(&s, &mut r, 522, 316).is_some());
        assert!(
            pick(&s, &mut r, 531, 316).is_none(),
            "image width stays 48px at scaled offset"
        );
    });
}
#[test]
#[ignore = "requires native GPU or Mesa Vulkan"]
fn depth_distance_overlay_color_pick_and_transparent_holes() {
    pollster::block_on(async {
        let (mut r, mut s) = setup().await;
        let data = specs(include_bytes!(
            "../../../tests/entities/depth-test-distance.json"
        ));
        let mut e = data[1].clone();
        s.set_entities(&mut r, data).unwrap();
        s.focus_entity(&e.id, 40000.);
        s.orbit.pitch = std::f64::consts::FRAC_PI_2;
        assert_eq!(
            s.properties(pick(&s, &mut r, 384, 256).unwrap()).unwrap()["id"],
            "occluder"
        );
        s.focus_entity(&e.id, 100000.);
        s.orbit.pitch = std::f64::consts::FRAC_PI_2;
        assert_eq!(
            s.properties(pick(&s, &mut r, 384, 256).unwrap()).unwrap()["id"],
            e.id
        );
        let mut front = e.clone();
        front.id = "front".into();
        front.position_degrees = Some([110., 25., 44000.]);
        front.color = [0., 1., 0., 1.];
        front.shape = entities::Shape::Point {
            pixel_size: 80.,
            outline_width: 0.,
            outline_color: [0., 0., 0., 1.],
        };
        front.screen.disable_depth_test_distance = 0.;
        s.upsert_entities(&mut r, vec![front]).unwrap();
        assert_eq!(
            s.properties(pick(&s, &mut r, 384, 256).unwrap()).unwrap()["id"],
            e.id,
            "overlay must be last in color and pick passes"
        );
        let frame = pixels(&mut r, &s).unwrap();
        let i = (256 * 768 + 384) * 4;
        assert!(frame[i] > 220 && frame[i + 1] < 120);
        let image = specs(include_bytes!("../../../tests/entities/screen.json")).remove(1);
        e.shape = image.shape;
        e.color = [1.; 4];
        s.upsert_entities(&mut r, vec![e.clone()]).unwrap();
        assert_eq!(
            s.properties(pick(&s, &mut r, 384, 256).unwrap()).unwrap()["id"],
            "front",
            "transparent overlay hole must preserve foreground pick"
        );
        assert_eq!(
            s.properties(pick(&s, &mut r, 410, 256).unwrap()).unwrap()["id"],
            e.id
        );
        e.screen.disable_depth_test_distance = 0.;
        s.remove_entity("front");
        s.upsert_entities(&mut r, vec![e]).unwrap();
        assert_eq!(
            s.properties(pick(&s, &mut r, 410, 256).unwrap()).unwrap()["id"],
            "occluder"
        );
        let base = url::Url::parse("file:///tests/screen-effects.czml").unwrap();
        let hydrated = cesium_viewer::entity_resources::hydrate(
            include_bytes!("../../../tests/entities/screen-effects.czml"),
            &base,
            |_, _| Ok(include_bytes!("../../../tests/entities/image-test.png").to_vec()),
        )
        .unwrap();
        let (specs, clock) = entities::czml(&hydrated).unwrap();
        s.clock = clock;
        s.set_entities(&mut r, specs).unwrap();
        s.focus_location(110., 25., 100000.).unwrap();
        common::save_png(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../docs/screen-effects-preview.png"),
            &pixels(&mut r, &s).unwrap(),
            768,
            512,
            image::ColorType::Rgba8,
        )
        .unwrap();
    });
}
