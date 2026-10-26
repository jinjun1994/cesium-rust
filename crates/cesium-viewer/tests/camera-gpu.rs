//! Render/readback and integer-pick checks for the bounded camera SDK.
mod common;
use cesium_renderer::{attachment, device, instance, FeatureRef, Renderer};
use cesium_viewer::{
    camera_control::{LookOffset, ViewOptions},
    entities::{self, Entity},
    Scene,
};
use common::pixels;
use std::time::Duration;
fn view(lon: f64) -> ViewOptions {
    ViewOptions {
        destination_degrees: [lon, 25.1, 96000.],
        heading: 0.,
        pitch: -std::f64::consts::FRAC_PI_2,
        roll: 0.,
    }
}
async fn setup() -> (Renderer, Scene) {
    let (_, d, q) = device(&instance(), None).await.unwrap();
    let mut r = Renderer::new(d, q, wgpu::TextureFormat::Rgba8UnormSrgb, 768, 512).unwrap();
    let mut s = Scene::new(&mut r, None).unwrap();
    let e = entities::decode(include_bytes!("../../../tests/entities/screen.json"))
        .unwrap()
        .remove(0);
    s.set_entities(&mut r, vec![e]).unwrap();
    (r, s)
}
fn pick(s: &Scene, r: &mut Renderer) -> Option<FeatureRef> {
    let t = attachment(
        &r.device,
        768,
        512,
        r.format(),
        wgpu::TextureUsages::RENDER_ATTACHMENT,
    );
    s.render(r, &t.create_view(&Default::default()), Some((384, 256)))
        .unwrap();
    r.device
        .poll(wgpu::PollType::Wait {
            submission_index: None,
            timeout: Some(Duration::from_secs(30)),
        })
        .unwrap();
    r.poll_pick().unwrap()
}
fn center(s: &Scene, r: &mut Renderer, id: &str) {
    assert_eq!(s.properties(pick(s, r).unwrap()).unwrap()["id"], id);
}
#[test]
#[ignore = "requires native GPU or Mesa Vulkan"]
fn set_view_pose_matches_visible_and_pick_center() {
    pollster::block_on(async {
        let (mut r, mut s) = setup().await;
        s.set_view(view(109.7)).unwrap();
        center(&s, &mut r, "point");
        let a = pixels(&mut r, &s).unwrap();
        s.set_view(view(110.1)).unwrap();
        assert!(pick(&s, &mut r).is_none());
        assert_ne!(a, pixels(&mut r, &s).unwrap());
        let before = s.orbit.camera(1, 1);
        let mut bad = view(110.);
        bad.destination_degrees[1] = 91.;
        assert!(s.set_view(bad).is_err());
        assert_eq!(before.eye, s.orbit.camera(1, 1).eye);
    });
}
#[test]
#[ignore = "requires native GPU or Mesa Vulkan"]
fn fly_to_advances_with_paused_entity_clock_and_exact_end() {
    pollster::block_on(async {
        let (mut r, mut s) = setup().await;
        s.set_view(view(109.3)).unwrap();
        s.clock.should_animate = false;
        let time = s.clock.current_time;
        let a = pixels(&mut r, &s).unwrap();
        s.fly_to(view(109.7), 2.).unwrap();
        s.tick(1.);
        assert_eq!(time, s.clock.current_time);
        assert_eq!(s.camera_state()["flightProgress"], 0.5);
        assert_ne!(a, pixels(&mut r, &s).unwrap());
        s.tick(1.);
        assert_eq!(s.camera_state()["flightStatus"], "completed");
        assert_eq!(s.orbit.camera(1, 1).eye, view(109.7).pose().unwrap().eye);
        center(&s, &mut r, "point");
        common::save_png(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/camera-preview.png"),
            &pixels(&mut r, &s).unwrap(),
            768,
            512,
            image::ColorType::Rgba8,
        )
        .unwrap();
    });
}
#[test]
#[ignore = "requires native GPU or Mesa Vulkan"]
fn cancellation_replacement_atomicity_and_manual_takeover() {
    pollster::block_on(async {
        let (mut r, mut s) = setup().await;
        s.set_view(view(109.3)).unwrap();
        let id = s.fly_to(view(109.7), 4.).unwrap();
        s.tick(1.);
        let at = s.orbit.camera(1, 1);
        assert!(s.fly_to(view(110.), -1.).is_err());
        assert_eq!(s.camera_state()["flightId"], id);
        assert_eq!(s.camera_state()["flightStatus"], "flying");
        assert!(s.cancel_flight());
        assert!(!s.cancel_flight());
        let a = pixels(&mut r, &s).unwrap();
        s.tick(10.);
        assert_eq!(at.eye, s.orbit.camera(1, 1).eye);
        assert_eq!(a, pixels(&mut r, &s).unwrap());
        assert!(s.fly_to(view(110.), 4.).unwrap() > id);
        assert_eq!(at.eye, s.orbit.camera(1, 1).eye);
        s.camera_drag(12., 3.);
        let manual = s.orbit.camera(1, 1);
        s.tick(10.);
        assert_eq!(manual.camera_to_ecef, s.orbit.camera(1, 1).camera_to_ecef);
        assert_eq!(s.camera_state()["flightStatus"], "cancelled");
        s.fly_to(view(109.7), 0.).unwrap();
        center(&s, &mut r, "point");
        s.fly_to(view(110.), 5.).unwrap();
        s.camera_zoom(1.);
        assert_eq!(s.camera_state()["flightStatus"], "cancelled");
    });
}
#[test]
#[ignore = "requires native GPU or Mesa Vulkan"]
fn tracked_entity_motion_visibility_removal_and_availability() {
    pollster::block_on(async {
        let (mut r, mut s) = setup().await;
        let base = url::Url::parse("file:///tests/entities/screen-motion.czml").unwrap();
        let data = cesium_viewer::entity_resources::hydrate(
            include_bytes!("../../../tests/entities/screen-motion.czml"),
            &base,
            |_, _| Ok(include_bytes!("../../../tests/entities/image-test.png").to_vec()),
        )
        .unwrap();
        let (specs, clock) = entities::czml(&data).unwrap();
        s.clock = clock;
        s.set_entities(&mut r, specs).unwrap();
        let offset = LookOffset {
            heading: 0.,
            pitch: -1.2,
            range: 40000.,
        };
        s.track_entity("point", offset).unwrap();
        center(&s, &mut r, "point");
        let start = s.orbit.camera(1, 1).eye;
        s.clock.current_time += 30.;
        s.tick(0.);
        assert!(start.distance(s.orbit.camera(1, 1).eye) > 30000.);
        center(&s, &mut r, "point");
        s.show_entity("point", false);
        assert!(s.camera_state()["trackedEntity"].is_null());
        assert!(s.track_entity("point", offset).is_err());
        s.show_entity("point", true);
        assert!(s.camera_state()["trackedEntity"].is_null());
        s.track_entity("point", offset).unwrap();
        s.remove_entity("point");
        assert!(s.camera_state()["trackedEntity"].is_null());
        let mut e: Entity = entities::decode(include_bytes!("../../../tests/entities/screen.json"))
            .unwrap()
            .remove(0);
        e.availability = Some([s.clock.current_time, s.clock.current_time + 1.]);
        s.upsert_entities(&mut r, vec![e]).unwrap();
        s.track_entity("point", offset).unwrap();
        s.clock.current_time += 2.;
        s.tick(0.);
        assert!(s.camera_state()["trackedEntity"].is_null());
        s.clock.current_time -= 2.;
        s.track_entity("point", offset).unwrap();
        s.clear_entities();
        assert!(s.camera_state()["trackedEntity"].is_null());
    });
}
#[test]
#[ignore = "requires native GPU or Mesa Vulkan"]
fn look_at_fixed_anchor_range_and_orbit_takeover() {
    pollster::block_on(async {
        let (mut r, mut s) = setup().await;
        let target = [109.7, 25.1, 16000.];
        let offset = LookOffset {
            heading: 0.6,
            pitch: -0.8,
            range: 80000.,
        };
        s.look_at(target, offset).unwrap();
        center(&s, &mut r, "point");
        let anchor = cesium_math::Cartographic::degrees(target[0], target[1], target[2])
            .unwrap()
            .to_ecef();
        let before = s.orbit.camera(1, 1);
        assert!((before.eye.distance(anchor) - 80000.).abs() < 1e-5);
        s.tick(3.);
        assert_eq!(before.eye, s.orbit.camera(1, 1).eye);
        let mut e = entities::decode(include_bytes!("../../../tests/entities/screen.json"))
            .unwrap()
            .remove(0);
        e.position_degrees = Some([110.1, 25.1, 16000.]);
        s.upsert_entities(&mut r, vec![e]).unwrap();
        s.tick(1.);
        assert!(pick(&s, &mut r).is_none());
        assert_eq!(before.eye, s.orbit.camera(1, 1).eye);
        s.camera_drag(30., 8.);
        assert_ne!(before.eye, s.orbit.camera(1, 1).eye);
        assert!((s.orbit.camera(1, 1).eye.distance(anchor) - 80000.).abs() < 1e-5);
        assert_eq!(s.camera_state()["mode"], "orbit");
    });
}
