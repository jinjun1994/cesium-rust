use std::cell::RefCell;
use std::rc::Rc;

use cesium_math::{
    Cartesian3, Clock, ClockStep, JulianDate, Matrix4, SceneMode, ShadowMode,
    TimeIntervalCollection,
};
use cesium_renderer::clipping_plane_collection::ClippingPlaneCollection;
use cesium_renderer::model_3d_tile_content::Cesium3DTileStyle;
use cesium_renderer::point_cloud_shading::PointCloudShading;
use cesium_renderer::scene_primitive::{ContextCapabilities, PrimitiveRenderFrameState};
use cesium_renderer::time_dynamic_point_cloud::{
    TimeDynamicPointCloud, TimeDynamicPointCloudIntervalData, TimeDynamicPointCloudOptions,
};

fn create_test_dates() -> Vec<JulianDate> {
    vec![
        JulianDate::from_iso8601("2018-07-19T15:18:00Z").unwrap(),
        JulianDate::from_iso8601("2018-07-19T15:18:00.5Z").unwrap(),
        JulianDate::from_iso8601("2018-07-19T15:18:01Z").unwrap(),
        JulianDate::from_iso8601("2018-07-19T15:18:01.5Z").unwrap(),
        JulianDate::from_iso8601("2018-07-19T15:18:02Z").unwrap(),
        JulianDate::from_iso8601("2018-07-19T15:18:02.5Z").unwrap(),
    ]
}

fn create_test_transforms() -> Vec<Matrix4> {
    vec![
        Matrix4::from_translation(Cartesian3::new(0.0, 0.0, 0.0)),
        Matrix4::from_translation(Cartesian3::new(10.0, 0.0, 0.0)),
        Matrix4::from_translation(Cartesian3::new(20.0, 0.0, 0.0)),
        Matrix4::from_translation(Cartesian3::new(30.0, 0.0, 0.0)),
        Matrix4::from_translation(Cartesian3::new(40.0, 0.0, 0.0)),
    ]
}

fn create_intervals(
    use_transforms: bool,
    use_draco: bool,
) -> (Vec<JulianDate>, TimeIntervalCollection<TimeDynamicPointCloudIntervalData>) {
    let dates = create_test_dates();
    let transforms = create_test_transforms();

    let folder = if use_transforms {
        "Data/Cesium3DTiles/PointCloud/PointCloudTimeDynamicWithTransform/"
    } else if use_draco {
        "Data/Cesium3DTiles/PointCloud/PointCloudTimeDynamicDraco/"
    } else {
        "Data/Cesium3DTiles/PointCloud/PointCloudTimeDynamic/"
    };

    let uris: Vec<String> = (0..5).map(|i| format!("{}{}.pnts", folder, i)).collect();

    let intervals = TimeIntervalCollection::from_julian_date_array(&dates, |_interval, index| {
        Some(TimeDynamicPointCloudIntervalData {
            uri: uris[index].clone(),
            transform: if use_transforms {
                Some(transforms[index])
            } else {
                None
            },
        })
    });

    (dates, intervals)
}

fn setup_point_cloud(
    use_transforms: bool,
    use_draco: bool,
) -> (
    Rc<RefCell<Clock>>,
    Vec<JulianDate>,
    TimeDynamicPointCloud,
) {
    let (dates, intervals) = create_intervals(use_transforms, use_draco);
    let mut clock = Clock::default();
    clock.set_clock_step(ClockStep::TickDependent);
    clock.set_should_animate(true);
    clock.set_current_time(dates[0]);
    clock.set_multiplier(0.0);
    let clock_rc = Rc::new(RefCell::new(clock));

    let point_cloud = TimeDynamicPointCloud::new(TimeDynamicPointCloudOptions {
        clock: clock_rc.clone(),
        intervals,
        show: Some(true),
        model_matrix: Some(Matrix4::IDENTITY),
        shadows: Some(ShadowMode::Enabled),
        maximum_memory_usage: Some(256.0),
        shading: Some(PointCloudShading::default()),
        style: Some(Cesium3DTileStyle {
            color_expression: Some("color('red')".to_string()),
            show_expression: None,
            point_size_expression: Some("10.0".to_string()),
        }),
        clipping_planes: None,
    })
    .unwrap();

    (clock_rc, dates, point_cloud)
}

#[test]
fn test_renders_in_3d_2d_and_columbus_view() {
    let (clock, _dates, mut pc) = setup_point_cloud(false, false);

    for mode in [SceneMode::Scene3D, SceneMode::Scene2D, SceneMode::ColumbusView] {
        let mut frame_state = PrimitiveRenderFrameState {
            mode,
            frame_number: 1,
            time: Some(clock.borrow().current_time()),
            ..Default::default()
        };

        pc.update(&mut frame_state);
        assert_eq!(frame_state.command_list.len(), 1);
        assert_eq!(
            frame_state.command_list[0].primitive_type,
            cesium_math::PrimitiveType::Points
        );
    }
}

#[test]
fn test_gets_bounding_sphere_of_rendered_frame() {
    let (clock, dates, mut pc) = setup_point_cloud(true, false);
    assert!(pc.bounding_sphere().is_none());

    let mut frame_state = PrimitiveRenderFrameState {
        mode: SceneMode::Scene3D,
        frame_number: 1,
        time: Some(dates[0]),
        ..Default::default()
    };

    pc.update(&mut frame_state);
    let bs0 = pc.bounding_sphere().expect("Frame 0 has bounding sphere");

    clock.borrow_mut().set_current_time(dates[1]);
    frame_state.frame_number = 2;
    frame_state.time = Some(dates[1]);
    frame_state.command_list.clear();
    pc.update(&mut frame_state);

    let bs1 = pc.bounding_sphere().expect("Frame 1 has bounding sphere");
    assert!(!bs0.equals(Some(&bs1)));
}

#[test]
fn test_sets_show_and_model_matrix() {
    let (clock, _dates, mut pc) = setup_point_cloud(false, false);

    let mut frame_state = PrimitiveRenderFrameState {
        mode: SceneMode::Scene3D,
        frame_number: 1,
        time: Some(clock.borrow().current_time()),
        ..Default::default()
    };

    pc.show = false;
    pc.update(&mut frame_state);
    assert_eq!(frame_state.command_list.len(), 0);

    pc.show = true;
    pc.model_matrix = Matrix4::from_translation(Cartesian3::new(100.0, 200.0, 300.0));
    pc.update(&mut frame_state);
    assert_eq!(frame_state.command_list.len(), 1);
}

#[test]
fn test_sets_shadows() {
    let (clock, _dates, mut pc) = setup_point_cloud(false, false);

    let mut frame_state = PrimitiveRenderFrameState {
        mode: SceneMode::Scene3D,
        frame_number: 1,
        time: Some(clock.borrow().current_time()),
        ..Default::default()
    };

    pc.shadows = ShadowMode::Disabled;
    pc.update(&mut frame_state);
    assert!(!frame_state.command_list[0].cast_shadows);
    assert!(!frame_state.command_list[0].receive_shadows);

    pc.shadows = ShadowMode::Enabled;
    frame_state.command_list.clear();
    frame_state.frame_number = 2;
    pc.update(&mut frame_state);
    assert!(frame_state.command_list[0].cast_shadows);
    assert!(frame_state.command_list[0].receive_shadows);
}

#[test]
fn test_honors_maximum_memory_usage() {
    let (clock, dates, mut pc) = setup_point_cloud(false, false);

    // Preload all 5 frames
    for i in 0..5 {
        pc.load_frame(i, 1);
    }

    assert_eq!(pc.frames().len(), 5);
    let total_before = pc.total_memory_usage_in_bytes();
    assert!(total_before > 0);

    // Set maximum memory usage to 0 MB
    pc.maximum_memory_usage = 0.0;

    // Render frame 0 at frame_number 2
    clock.borrow_mut().set_current_time(dates[0]);
    let mut frame_state = PrimitiveRenderFrameState {
        mode: SceneMode::Scene3D,
        frame_number: 2,
        time: Some(dates[0]),
        ..Default::default()
    };

    pc.update(&mut frame_state);

    // Frame 0 was touched at frame_number 2, so frames 1..5 should be unloaded
    assert!(pc.frames()[0].is_some());
    for i in 1..5 {
        assert!(pc.frames()[i].is_none());
    }
    assert_eq!(
        pc.total_memory_usage_in_bytes(),
        pc.frames()[0].as_ref().unwrap().byte_length
    );
}

#[test]
fn test_enables_eye_dome_lighting() {
    let (clock, _dates, mut pc) = setup_point_cloud(false, false);

    let mut frame_state = PrimitiveRenderFrameState {
        mode: SceneMode::Scene3D,
        frame_number: 1,
        time: Some(clock.borrow().current_time()),
        context_capabilities: Some(ContextCapabilities {
            draw_buffers: true,
            fragment_depth: true,
            floating_point_texture: true,
        }),
        ..Default::default()
    };

    pc.shading.attenuation = true;
    pc.shading.eye_dome_lighting = true;
    pc.update(&mut frame_state);

    // 1 hijacked draw command + 1 screen blend quad + 1 clear command
    assert_eq!(frame_state.command_list.len(), 3);
}

#[test]
fn test_sets_style_and_make_style_dirty() {
    let (_clock, _dates, mut pc) = setup_point_cloud(false, false);

    pc.style = Some(Cesium3DTileStyle {
        color_expression: Some("color('blue')".to_string()),
        show_expression: None,
        point_size_expression: Some("15.0".to_string()),
    });
    assert!(!pc.style_dirty);

    pc.make_style_dirty();
    assert!(pc.style_dirty);
}

#[test]
fn test_sets_clipping_planes() {
    let (_clock, _dates, mut pc) = setup_point_cloud(false, false);

    let planes = ClippingPlaneCollection::default();
    pc.clipping_planes = Some(planes);
    assert!(pc.clipping_planes.is_some());
}

#[test]
fn test_does_not_render_during_morph_or_out_of_range() {
    let (clock, dates, mut pc) = setup_point_cloud(false, false);

    // Morphing
    let mut frame_state = PrimitiveRenderFrameState {
        mode: SceneMode::Morphing,
        frame_number: 1,
        time: Some(dates[0]),
        ..Default::default()
    };
    pc.update(&mut frame_state);
    assert_eq!(frame_state.command_list.len(), 0);

    // Out of time range (before start date)
    frame_state.mode = SceneMode::Scene3D;
    let before_date = dates[0].add_seconds(-10.0);
    clock.borrow_mut().set_current_time(before_date);
    frame_state.time = Some(before_date);
    pc.update(&mut frame_state);
    assert_eq!(frame_state.command_list.len(), 0);

    // Out of time range (after end date)
    let after_date = dates[5].add_seconds(10.0);
    clock.borrow_mut().set_current_time(after_date);
    frame_state.time = Some(after_date);
    pc.update(&mut frame_state);
    assert_eq!(frame_state.command_list.len(), 0);
}

#[test]
fn test_renders_frames_using_draco_compression() {
    let (clock, _dates, mut pc) = setup_point_cloud(false, true);

    let mut frame_state = PrimitiveRenderFrameState {
        mode: SceneMode::Scene3D,
        frame_number: 1,
        time: Some(clock.borrow().current_time()),
        ..Default::default()
    };

    pc.update(&mut frame_state);
    assert_eq!(frame_state.command_list.len(), 1);
    assert!(pc.frames()[0].is_some());
}

#[test]
fn test_picks() {
    let (clock, _dates, mut pc) = setup_point_cloud(false, false);

    let mut frame_state = PrimitiveRenderFrameState {
        mode: SceneMode::Scene3D,
        frame_number: 1,
        time: Some(clock.borrow().current_time()),
        ..Default::default()
    };

    pc.update(&mut frame_state);
    assert_eq!(
        frame_state.command_list[0].pick_id.as_deref(),
        Some("TimeDynamicPointCloudPickId")
    );
}

#[test]
fn test_prefetches_and_skips_frames_based_on_multiplier() {
    let (clock, dates, mut pc) = setup_point_cloud(false, false);
    pc.set_average_load_time_override(Some(0.5));

    let mut frame_state = PrimitiveRenderFrameState {
        mode: SceneMode::Scene3D,
        frame_number: 1,
        time: Some(dates[0]),
        ..Default::default()
    };

    // Render frame 0 with multiplier 1.0 (step = 0.5s) -> preloads frame 1
    clock.borrow_mut().set_multiplier(1.0);
    pc.update(&mut frame_state);
    assert!(pc.frames()[0].is_some());
    assert!(pc.frames()[1].is_some());

    // Switch multiplier to 4.0 (step = 2.0s) -> preloads frame 4
    clock.borrow_mut().set_multiplier(4.0);
    frame_state.frame_number = 2;
    pc.update(&mut frame_state);
    assert!(pc.frames()[4].is_some());
}

#[test]
fn test_works_with_negative_clock_multiplier() {
    let (clock, dates, mut pc) = setup_point_cloud(false, false);
    pc.set_average_load_time_override(Some(2.0));

    // Jump to frame 4
    clock.borrow_mut().set_current_time(dates[4]);
    clock.borrow_mut().set_multiplier(-0.6);

    let mut frame_state = PrimitiveRenderFrameState {
        mode: SceneMode::Scene3D,
        frame_number: 1,
        time: Some(dates[4]),
        ..Default::default()
    };

    pc.update(&mut frame_state);
    assert!(pc.frames()[4].is_some());
    // Preloads frame 1 at 2.0s - 1.2s = 0.8s
    assert!(pc.frames()[1].is_some());
}

#[test]
fn test_raises_frame_changed_event() {
    let (clock, dates, mut pc) = setup_point_cloud(false, false);

    let event_count = Rc::new(RefCell::new(0));
    let ec_clone = event_count.clone();
    let listener_id = pc.frame_changed.add_listener(move |_| {
        *ec_clone.borrow_mut() += 1;
    });

    let mut frame_state = PrimitiveRenderFrameState {
        mode: SceneMode::Scene3D,
        frame_number: 1,
        time: Some(dates[0]),
        ..Default::default()
    };

    pc.update(&mut frame_state);
    assert_eq!(*event_count.borrow(), 1);

    // Same frame -> does not re-raise
    frame_state.frame_number = 2;
    pc.update(&mut frame_state);
    assert_eq!(*event_count.borrow(), 1);

    // Switch to frame 1
    clock.borrow_mut().set_current_time(dates[1]);
    frame_state.frame_number = 3;
    frame_state.time = Some(dates[1]);
    pc.update(&mut frame_state);
    assert_eq!(*event_count.borrow(), 2);

    pc.frame_changed.remove_listener(listener_id);
}

#[test]
fn test_destroys() {
    let (clock, _dates, mut pc) = setup_point_cloud(false, false);

    let mut frame_state = PrimitiveRenderFrameState {
        mode: SceneMode::Scene3D,
        frame_number: 1,
        time: Some(clock.borrow().current_time()),
        ..Default::default()
    };

    pc.update(&mut frame_state);
    assert!(!pc.is_destroyed());
    assert!(pc.total_memory_usage_in_bytes() > 0);

    pc.destroy();
    assert!(pc.is_destroyed());
    assert_eq!(pc.total_memory_usage_in_bytes(), 0);
    assert!(pc.frames().is_empty());
}
