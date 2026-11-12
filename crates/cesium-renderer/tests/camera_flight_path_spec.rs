use std::cell::Cell;
use std::rc::Rc;

use cesium_math::{
    equals_epsilon, Cartesian3, Cartographic, EasingFunction, Ellipsoid, GeographicProjection,
    OrthographicOffCenterFrustum, SceneMode, EPSILON12, EPSILON4, EPSILON6, EPSILON7, PI_OVER_FOUR,
    PI_OVER_TWO, TWO_PI,
};
use cesium_renderer::{
    CameraFlightOptions, CameraFlightPath, CameraFrustum, FlightScene,
};
use glam::DVec3;

fn create_orthographic_frustum(scene: &FlightScene) -> CameraFrustum {
    match scene.camera.frustum {
        CameraFrustum::Perspective(p) => {
            let fovy = p.fovy().unwrap_or(p.fov);
            let tan_theta = (0.5 * fovy).tan();
            let top = p.near * tan_theta;
            let right = p.aspect_ratio * top;
            CameraFrustum::OrthographicOffCenter(OrthographicOffCenterFrustum {
                left: -right,
                right,
                bottom: -top,
                top,
                near: p.near,
                far: p.far,
            })
        }
        other => other,
    }
}

#[test]
fn create_animation_throws_without_destination() {
    let mut scene = FlightScene::default();
    assert!(CameraFlightPath::create_tween(&mut scene, CameraFlightOptions::default()).is_err());
}

#[test]
fn creates_an_animation_and_wraps_callbacks() {
    let mut scene = FlightScene::default();
    let completed = Rc::new(Cell::new(false));
    let completed_clone = completed.clone();

    let mut flight = CameraFlightPath::create_tween(
        &mut scene,
        CameraFlightOptions {
            destination: Some(Cartesian3::new(1e9, 1e9, 1e9)),
            duration: Some(5.0),
            complete: Some(Box::new(move || completed_clone.set(true))),
            ..Default::default()
        },
    )
    .unwrap();

    assert_eq!(flight.duration, 5.0);
    assert!(flight.has_update());
    assert!(!scene.enable_inputs);
    flight.complete(&mut scene);
    assert!(completed.get());
    assert!(scene.enable_inputs);
}

#[test]
fn creates_an_animation_in_3d() {
    let mut scene = FlightScene::default();
    let start_position = scene.camera.position;
    let start_heading = scene.camera.heading;
    let start_pitch = scene.camera.pitch;
    let start_roll = scene.camera.roll;

    let end_position = start_position.negate();
    let end_heading = 20.0_f64.to_radians();
    let end_pitch = (-45.0_f64).to_radians();
    let end_roll = TWO_PI;
    let duration = 5.0;

    let mut flight = CameraFlightPath::create_tween(
        &mut scene,
        CameraFlightOptions {
            destination: Some(end_position),
            heading: Some(end_heading),
            pitch: Some(end_pitch),
            roll: Some(end_roll),
            duration: Some(duration),
            ..Default::default()
        },
    )
    .unwrap();

    flight.update(&mut scene, 0.0);
    assert!(Cartesian3::equals_epsilon(
        &scene.camera.position,
        &start_position,
        Some(EPSILON12),
        Some(EPSILON12),
    ));
    assert!(equals_epsilon(scene.camera.heading, start_heading, Some(EPSILON12), None));
    assert!(equals_epsilon(scene.camera.pitch, start_pitch, Some(EPSILON12), None));
    assert!(equals_epsilon(
        cesium_math::negative_pi_to_pi(scene.camera.roll),
        cesium_math::negative_pi_to_pi(start_roll),
        Some(EPSILON12),
        None
    ));

    flight.update(&mut scene, duration);
    assert!(Cartesian3::equals_epsilon(
        &scene.camera.position,
        &end_position,
        Some(EPSILON12),
        Some(EPSILON12),
    ));
    assert!(equals_epsilon(scene.camera.heading, end_heading, Some(EPSILON12), None));
    assert!(equals_epsilon(scene.camera.pitch, end_pitch, Some(EPSILON12), None));
    assert!(equals_epsilon(
        cesium_math::negative_pi_to_pi(scene.camera.roll),
        cesium_math::negative_pi_to_pi(end_roll),
        Some(EPSILON12),
        None
    ));
}

#[test]
fn creates_an_animation_in_3d_using_custom_ellipsoid() {
    let ellipsoid = Ellipsoid::new(DVec3::splat(1_737_400.0)).unwrap();
    let projection = GeographicProjection::new(ellipsoid).unwrap();
    let mut scene = FlightScene::with_projection(projection);

    let start_position = scene.camera.position;
    let end_position = Cartesian3::from(
        ellipsoid
            .cartographic_to_cartesian(Cartographic::from_degrees(0.0, 0.0, Some(100.0)))
            .unwrap(),
    );

    let duration = 1.0;
    let mut flight = CameraFlightPath::create_tween(
        &mut scene,
        CameraFlightOptions {
            destination: Some(end_position),
            duration: Some(duration),
            ..Default::default()
        },
    )
    .unwrap();

    flight.update(&mut scene, 0.0);
    assert!(Cartesian3::equals_epsilon(
        &scene.camera.position,
        &start_position,
        Some(EPSILON12),
        Some(EPSILON12),
    ));

    flight.update(&mut scene, duration);
    assert!(Cartesian3::equals_epsilon(
        &scene.camera.position,
        &end_position,
        Some(EPSILON7),
        Some(EPSILON7),
    ));
}

#[test]
fn creates_an_animation_in_columbus_view() {
    let mut scene = FlightScene {
        mode: SceneMode::ColumbusView,
        ..Default::default()
    };
    scene.camera.position = Cartesian3::new(0.0, 0.0, 1000.0);
    scene.camera.direction = Cartesian3::UNIT_Z.negate();
    scene.camera.up = Cartesian3::UNIT_Y;
    scene.camera.right = scene.camera.direction.cross(&scene.camera.up);

    let start_position = scene.camera.position;
    let projection = scene.map_projection;
    let destination = start_position
        + Cartesian3::new(
            -6e5 * std::f64::consts::PI,
            6e5 * PI_OVER_FOUR,
            100.0,
        );
    let carto = projection.unproject(DVec3::from(destination)).unwrap();
    let end_position = Cartesian3::from(
        projection
            .ellipsoid()
            .cartographic_to_cartesian(carto)
            .unwrap(),
    );

    let duration = 5.0;
    let mut flight = CameraFlightPath::create_tween(
        &mut scene,
        CameraFlightOptions {
            destination: Some(end_position),
            duration: Some(duration),
            ..Default::default()
        },
    )
    .unwrap();

    flight.update(&mut scene, 0.0);
    assert!(Cartesian3::equals_epsilon(
        &scene.camera.position,
        &start_position,
        Some(EPSILON12),
        Some(EPSILON12),
    ));

    flight.update(&mut scene, duration);
    assert!(Cartesian3::equals_epsilon(
        &scene.camera.position,
        &destination,
        Some(EPSILON4),
        Some(EPSILON4),
    ));
}

#[test]
fn creates_an_animation_in_2d() {
    let mut scene = FlightScene {
        mode: SceneMode::Scene2D,
        ..Default::default()
    };
    scene.camera.position = Cartesian3::new(0.0, 0.0, 1000.0);
    scene.camera.direction = Cartesian3::UNIT_Z.negate();
    scene.camera.up = Cartesian3::UNIT_Y;
    scene.camera.right = scene.camera.direction.cross(&scene.camera.up);
    scene.camera.frustum = create_orthographic_frustum(&scene);

    let start_height = match scene.camera.frustum {
        CameraFrustum::OrthographicOffCenter(f) => f.right - f.left,
        _ => unreachable!(),
    };
    let start_position = scene.camera.position;

    let projection = scene.map_projection;
    let destination = start_position
        + Cartesian3::new(
            -6e6 * std::f64::consts::PI,
            6e6 * PI_OVER_FOUR,
            100.0,
        );
    let carto = projection.unproject(DVec3::from(destination)).unwrap();
    let end_position = Cartesian3::from(
        projection
            .ellipsoid()
            .cartographic_to_cartesian(carto)
            .unwrap(),
    );

    let duration = 5.0;
    let mut flight = CameraFlightPath::create_tween(
        &mut scene,
        CameraFlightOptions {
            destination: Some(end_position),
            duration: Some(duration),
            ..Default::default()
        },
    )
    .unwrap();

    flight.update(&mut scene, 0.0);
    assert!(Cartesian3::equals_epsilon(
        &scene.camera.position,
        &start_position,
        Some(EPSILON12),
        Some(EPSILON12),
    ));
    if let CameraFrustum::OrthographicOffCenter(f) = scene.camera.frustum {
        assert!(equals_epsilon(f.right - f.left, start_height, Some(EPSILON7), None));
    }

    flight.update(&mut scene, duration);
    assert!(equals_epsilon(scene.camera.position.x, destination.x, Some(EPSILON7), None));
    assert!(equals_epsilon(scene.camera.position.y, destination.y, Some(EPSILON7), None));
    assert!(equals_epsilon(scene.camera.position.z, start_position.z, Some(EPSILON7), None));
    if let CameraFrustum::OrthographicOffCenter(f) = scene.camera.frustum {
        assert!(equals_epsilon(f.right - f.left, destination.z, Some(EPSILON7), None));
    }
}

#[test]
fn uses_fly_over_longitude_and_weight() {
    let mut scene = FlightScene::default();
    let ellipsoid = scene.map_projection.ellipsoid();
    let start = Cartesian3::from(
        ellipsoid
            .cartographic_to_cartesian(Cartographic::from_degrees(10.0, 45.0, Some(1000.0)))
            .unwrap(),
    );
    let end = Cartesian3::from(
        ellipsoid
            .cartographic_to_cartesian(Cartographic::from_degrees(20.0, 45.0, Some(1000.0)))
            .unwrap(),
    );

    scene.camera.position = start;
    let mut over_lon = CameraFlightPath::create_tween(
        &mut scene,
        CameraFlightOptions {
            destination: Some(end),
            duration: Some(1.0),
            fly_over_longitude: Some(0.0_f64.to_radians()),
            ..Default::default()
        },
    )
    .unwrap();

    over_lon.update(&mut scene, 0.3);
    let lon_over = ellipsoid
        .cartesian_to_cartographic(DVec3::from(scene.camera.position))
        .unwrap()
        .longitude
        .to_degrees();
    assert!(lon_over < 10.0);

    scene.camera.position = start;
    let mut direct = CameraFlightPath::create_tween(
        &mut scene,
        CameraFlightOptions {
            destination: Some(end),
            duration: Some(1.0),
            ..Default::default()
        },
    )
    .unwrap();
    direct.update(&mut scene, 0.3);
    let lon_direct = ellipsoid
        .cartesian_to_cartographic(DVec3::from(scene.camera.position))
        .unwrap()
        .longitude
        .to_degrees();
    assert!(lon_direct > 10.0 && lon_direct < 20.0);

    // flyOverLongitudeWeight test
    let end50 = Cartesian3::from(
        ellipsoid
            .cartographic_to_cartesian(Cartographic::from_degrees(50.0, 45.0, Some(1000.0)))
            .unwrap(),
    );
    scene.camera.position = start;
    let mut big_weight = CameraFlightPath::create_tween(
        &mut scene,
        CameraFlightOptions {
            destination: Some(end50),
            duration: Some(1.0),
            fly_over_longitude: Some(0.0_f64.to_radians()),
            fly_over_longitude_weight: Some(20.0),
            ..Default::default()
        },
    )
    .unwrap();
    big_weight.update(&mut scene, 0.3);
    let lon_big = ellipsoid
        .cartesian_to_cartographic(DVec3::from(scene.camera.position))
        .unwrap()
        .longitude
        .to_degrees();
    assert!(lon_big < 10.0);

    scene.camera.position = start;
    let mut small_weight = CameraFlightPath::create_tween(
        &mut scene,
        CameraFlightOptions {
            destination: Some(end50),
            duration: Some(1.0),
            fly_over_longitude: Some(0.0_f64.to_radians()),
            fly_over_longitude_weight: Some(2.0),
            ..Default::default()
        },
    )
    .unwrap();
    small_weight.update(&mut scene, 0.3);
    let lon_small = ellipsoid
        .cartesian_to_cartographic(DVec3::from(scene.camera.position))
        .unwrap()
        .longitude
        .to_degrees();
    assert!(lon_small > 10.0 && lon_small < 50.0);
}

#[test]
fn adjusts_pitch_above_pitch_adjust_height_and_respects_maximum_height() {
    let mut scene = FlightScene::default();
    let ellipsoid = scene.map_projection.ellipsoid();
    let start_pos = Cartesian3::from(
        ellipsoid
            .cartographic_to_cartesian(Cartographic::from_degrees(-20.0, 0.0, Some(1000.0)))
            .unwrap(),
    );
    let end_pos = Cartesian3::from(
        ellipsoid
            .cartographic_to_cartesian(Cartographic::from_degrees(60.0, 0.0, Some(2000.0)))
            .unwrap(),
    );
    let duration = 5.0;

    scene.camera.set_view(
        Some(start_pos),
        Some(0.0),
        Some((-15.0_f64).to_radians()),
        Some(0.0),
    );
    let start_pitch = scene.camera.pitch;
    let end_pitch = (-45.0_f64).to_radians();

    let mut flight = CameraFlightPath::create_tween(
        &mut scene,
        CameraFlightOptions {
            destination: Some(end_pos),
            pitch: Some(end_pitch),
            duration: Some(duration),
            pitch_adjust_height: Some(2000.0),
            ..Default::default()
        },
    )
    .unwrap();

    flight.update(&mut scene, 0.0);
    assert!(equals_epsilon(scene.camera.pitch, start_pitch, Some(EPSILON6), None));

    flight.update(&mut scene, duration);
    assert!(equals_epsilon(scene.camera.pitch, end_pitch, Some(EPSILON6), None));

    flight.update(&mut scene, duration / 2.0);
    assert!(equals_epsilon(scene.camera.pitch, -PI_OVER_TWO, Some(EPSILON4), None));

    // maximumHeight check
    let mut max_h = f64::NEG_INFINITY;
    for i in 0..=5 {
        flight.update(&mut scene, i as f64);
        max_h = max_h.max(scene.camera.position_cartographic().height);
    }
    let cap = max_h * 0.5;
    scene.camera.set_view(Some(start_pos), None, None, None);
    let mut capped = CameraFlightPath::create_tween(
        &mut scene,
        CameraFlightOptions {
            destination: Some(end_pos),
            duration: Some(duration),
            maximum_height: Some(cap),
            ..Default::default()
        },
    )
    .unwrap();
    for i in 0..=5 {
        capped.update(&mut scene, i as f64);
        assert!(scene.camera.position_cartographic().height < cap);
    }
}

#[test]
fn fly_over_longitude_is_smooth_and_handles_morphing_and_zero_duration() {
    let mut scene = FlightScene::default();
    let ellipsoid = scene.map_projection.ellipsoid();
    let start = Cartesian3::from(
        ellipsoid
            .cartographic_to_cartesian(Cartographic::from_degrees(10.0, 45.0, Some(1000.0)))
            .unwrap(),
    );
    let end = Cartesian3::from(
        ellipsoid
            .cartographic_to_cartesian(Cartographic::from_degrees(20.0, 45.0, Some(1000.0)))
            .unwrap(),
    );
    scene.camera.position = start;

    let duration = 100.0;
    let mut flight = CameraFlightPath::create_tween(
        &mut scene,
        CameraFlightOptions {
            destination: Some(end),
            duration: Some(duration),
            fly_over_longitude: Some(0.0),
            ..Default::default()
        },
    )
    .unwrap();

    let mut prev_lon = 10.0;
    let mut crossed_date_line = 0;
    for t in 1..100 {
        flight.update(&mut scene, t as f64);
        let lon = ellipsoid
            .cartesian_to_cartographic(DVec3::from(scene.camera.position))
            .unwrap()
            .longitude
            .to_degrees();
        let mut d = lon - prev_lon;
        if d > 0.0 {
            assert!(prev_lon < -90.0);
            crossed_date_line += 1;
            d -= 360.0;
        }
        prev_lon = lon;
        assert!(d < 0.0);
    }
    assert_eq!(crossed_date_line, 1);

    // Morphing returns empty flight
    scene.mode = SceneMode::Morphing;
    let morph_flight = CameraFlightPath::create_tween(
        &mut scene,
        CameraFlightOptions {
            destination: Some(end),
            ..Default::default()
        },
    )
    .unwrap();
    assert!(morph_flight.is_empty);

    // Duration <= 0 applies destination on complete
    scene.mode = SceneMode::Scene3D;
    scene.camera.position = start;
    let mut zero_flight = CameraFlightPath::create_tween(
        &mut scene,
        CameraFlightOptions {
            destination: Some(end),
            duration: Some(0.0),
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(zero_flight.easing_function, EasingFunction::LinearNone);
    zero_flight.complete(&mut scene);
    assert!(Cartesian3::equals_epsilon(
        &scene.camera.position,
        &end,
        Some(EPSILON7),
        Some(EPSILON7),
    ));
}
