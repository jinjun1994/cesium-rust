//! Five independent offscreen verifications, one for each 0.0.14 feature.
mod common;
use cesium_renderer::{attachment, device, instance, FeatureRef, Renderer};
use cesium_viewer::{
    entities::{self, Entity},
    screen_options::{HorizontalOrigin as H, VerticalOrigin as V},
    Scene,
};
use common::pixels;
use std::time::Duration;
async fn setup() -> (Renderer, Scene) {
    let (_, device, queue) = device(&instance(), None).await.unwrap();
    let mut r =
        Renderer::new(device, queue, wgpu::TextureFormat::Rgba8UnormSrgb, 768, 512).unwrap();
    let s = Scene::new(&mut r, None).unwrap();
    (r, s)
}
fn point() -> Entity {
    entities::decode(include_bytes!("../../../tests/entities/screen.json"))
        .unwrap()
        .remove(0)
}
fn billboard() -> Entity {
    entities::decode(include_bytes!("../../../tests/entities/screen.json"))
        .unwrap()
        .remove(1)
}
fn label() -> Entity {
    entities::decode(include_bytes!("../../../tests/entities/labels.json"))
        .unwrap()
        .remove(0)
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
fn red_diameter(p: &[u8]) -> usize {
    (280..488)
        .filter(|&x| {
            let i = (256 * 768 + x) * 4;
            p[i] > 220 && p[i + 1] < 120 && p[i + 2] < 120
        })
        .count()
}
#[test]
#[ignore = "requires native GPU or Mesa Vulkan"]
fn label_chinese_text_alpha_pick_and_lifecycle() {
    pollster::block_on(async {
        let (mut r, mut s) = setup().await;
        let e = label();
        s.set_entities(&mut r, vec![e.clone()]).unwrap();
        s.focus_entity(&e.id, 40000.);
        let frame = pixels(&mut r, &s).unwrap();
        let opaque = (200..310)
            .flat_map(|y| (240..528).map(move |x| (x, y)))
            .find(|&(x, y)| {
                let i = (y * 768 + x) * 4;
                frame[i..i + 3].iter().all(|v| *v > 245)
            })
            .unwrap();
        let hit = pick(&s, &mut r, opaque.0 as u32, opaque.1 as u32).unwrap();
        assert_eq!(s.properties(hit).unwrap()["id"], e.id);
        assert!(
            pick(&s, &mut r, 240, 200).is_none(),
            "empty text rectangle region must pass through"
        );
        s.show_entity(&e.id, false);
        assert!(pick(&s, &mut r, opaque.0 as u32, opaque.1 as u32).is_none());
        assert!(s.properties(hit).is_none());
        s.show_entity(&e.id, true);
        let mut bad = e.clone();
        if let entities::Shape::Label { text, .. } = &mut bad.shape {
            *text = "😀".into();
        }
        assert!(s.upsert_entities(&mut r, vec![bad]).is_err());
        assert_eq!(s.entity_specs()[0].id, e.id);
        s.remove_entity(&e.id);
        assert!(s.properties(hit).is_none());
    });
}
#[test]
#[ignore = "requires native GPU or Mesa Vulkan"]
fn distance_display_condition_draw_and_pick() {
    pollster::block_on(async {
        let (mut r, mut s) = setup().await;
        let mut e = point();
        e.screen.distance_display_condition = Some([30000., 80000.]);
        s.set_entities(&mut r, vec![e]).unwrap();
        for (range, visible) in [
            (40000., true),
            (15000., false),
            (160000., false),
            (50000., true),
        ] {
            s.focus_entity("point", range);
            assert_eq!(pick(&s, &mut r, 384, 256).is_some(), visible);
            assert_eq!(red_diameter(&pixels(&mut r, &s).unwrap()) > 0, visible);
        }
    });
}
#[test]
#[ignore = "requires native GPU or Mesa Vulkan"]
fn distance_scale_pixel_diameter_and_fixed_offset() {
    pollster::block_on(async {
        let (mut r, mut s) = setup().await;
        let mut e = point();
        e.screen.scale_by_distance = Some([40000., 2., 160000., 0.5]);
        s.set_entities(&mut r, vec![e]).unwrap();
        for (range, expected) in [(20000., 64), (220000., 16)] {
            s.focus_entity("point", range);
            assert_eq!(red_diameter(&pixels(&mut r, &s).unwrap()), expected);
            assert!(pick(&s, &mut r, 384, 256).is_some());
        }
        let mut e = billboard();
        if let entities::Shape::Billboard {
            width,
            height,
            pixel_offset,
            ..
        } = &mut e.shape
        {
            *width = 48.;
            *height = 48.;
            *pixel_offset = [40., 20.];
        }
        e.screen.scale_by_distance = Some([40000., 2., 160000., 0.5]);
        s.set_entities(&mut r, vec![e]).unwrap();
        s.focus_entity("billboard", 20000.);
        assert!(
            pick(&s, &mut r, 380, 276).is_some(),
            "48px image expands to 96px around fixed 40px offset"
        );
        assert!(pick(&s, &mut r, 475, 276).is_none());
        assert!(pick(&s, &mut r, 424, 232).is_some());
        assert!(
            pick(&s, &mut r, 424, 224).is_none(),
            "20px vertical offset must not double"
        );
    });
}
#[test]
#[ignore = "requires native GPU or Mesa Vulkan"]
fn distance_translucency_blend_and_zero_alpha_pick() {
    pollster::block_on(async {
        let (mut r, mut s) = setup().await;
        let mut e = point();
        e.screen.translucency_by_distance = Some([40000., 1., 160000., 0.]);
        s.set_entities(&mut r, vec![e.clone()]).unwrap();
        s.focus_entity("point", 20000.);
        assert!(pick(&s, &mut r, 384, 256).is_some());
        s.focus_entity("point", 100000.);
        let faded = pixels(&mut r, &s).unwrap();
        assert!(pick(&s, &mut r, 384, 256).is_some());
        s.show_entity("point", false);
        let background = pixels(&mut r, &s).unwrap();
        s.show_entity("point", true);
        let mut opaque = e.clone();
        opaque.screen.translucency_by_distance = None;
        s.upsert_entities(&mut r, vec![opaque]).unwrap();
        let full = pixels(&mut r, &s).unwrap();
        let i = (256 * 768 + 384) * 4;
        assert!(
            faded[i] > background[i] && faded[i] < full[i],
            "distance alpha must blend in framebuffer"
        );
        s.upsert_entities(&mut r, vec![e]).unwrap();
        s.focus_entity("point", 220000.);
        assert!(pick(&s, &mut r, 384, 256).is_none());
        assert_eq!(red_diameter(&pixels(&mut r, &s).unwrap()), 0);
    });
}
#[test]
#[ignore = "requires native GPU or Mesa Vulkan"]
fn origins_billboard_quadrants_and_label_baseline() {
    pollster::block_on(async {
        let (mut r, mut s) = setup().await;
        let mut e = billboard();
        e.screen.horizontal_origin = H::Left;
        e.screen.vertical_origin = V::Top;
        s.set_entities(&mut r, vec![e.clone()]).unwrap();
        s.focus_entity("billboard", 40000.);
        assert!(pick(&s, &mut r, 394, 266).is_some());
        assert!(pick(&s, &mut r, 374, 246).is_none());
        e.screen.horizontal_origin = H::Right;
        e.screen.vertical_origin = V::Bottom;
        s.upsert_entities(&mut r, vec![e]).unwrap();
        assert!(pick(&s, &mut r, 374, 246).is_some());
        assert!(pick(&s, &mut r, 394, 266).is_none());
        let mut e = label();
        if let entities::Shape::Label {
            text, font_size, ..
        } = &mut e.shape
        {
            *text = "gyp".into();
            *font_size = 48.;
        }
        e.screen.horizontal_origin = H::Left;
        e.screen.vertical_origin = V::Baseline;
        s.set_entities(&mut r, vec![e.clone()]).unwrap();
        s.focus_entity(&e.id, 40000.);
        let frame = pixels(&mut r, &s).unwrap();
        let rows = (180..330)
            .filter(|&y| {
                (384..520).any(|x| {
                    let i = (y * 768 + x) * 4;
                    frame[i..i + 3].iter().all(|v| *v > 240)
                })
            })
            .collect::<Vec<_>>();
        assert!(
            *rows.first().unwrap() < 256 && *rows.last().unwrap() > 256,
            "baseline must put descenders below anchor"
        );
        let base = url::Url::parse("file:///tests/label-distance.czml").unwrap();
        let data = cesium_viewer::entity_resources::hydrate(
            include_bytes!("../../../tests/entities/label-distance.czml"),
            &base,
            |_, _| Ok(include_bytes!("../../../tests/entities/image-test.png").to_vec()),
        )
        .unwrap();
        let (specs, clock) = entities::czml(&data).unwrap();
        s.clock = clock;
        s.set_entities(&mut r, specs).unwrap();
        s.focus_location(110., 25., 56000.).unwrap();
        common::save_png(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../docs/label-distance-preview.png"),
            &pixels(&mut r, &s).unwrap(),
            768,
            512,
            image::ColorType::Rgba8,
        )
        .unwrap();
    });
}
