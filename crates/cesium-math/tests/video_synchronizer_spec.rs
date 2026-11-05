use cesium_math::{JulianDate, VideoElementMock, VideoSynchronizer, VideoSynchronizerOptions};

#[test]
fn test_default_construct() {
    let mut synchronizer = VideoSynchronizer::default();
    assert_eq!(synchronizer.epoch, JulianDate::MINIMUM_VALUE);
    assert_eq!(synchronizer.tolerance, 1.0);
    assert!(!synchronizer.is_destroyed());

    synchronizer.destroy();
    assert!(synchronizer.is_destroyed());
}

#[test]
fn test_construct_with_options() {
    let epoch = JulianDate::from_iso8601("2015-11-01T00:00:00Z").unwrap();
    let tolerance = 0.15;

    let mut synchronizer = VideoSynchronizer::new(Some(VideoSynchronizerOptions {
        epoch: Some(epoch),
        tolerance: Some(tolerance),
    }));

    assert_eq!(synchronizer.epoch, epoch);
    assert_eq!(synchronizer.tolerance, tolerance);
    assert!(!synchronizer.is_destroyed());

    synchronizer.destroy();
    assert!(synchronizer.is_destroyed());
}

#[test]
fn test_syncs_time_when_looping() {
    let epoch = JulianDate::from_iso8601("2015-11-01T00:00:00Z").unwrap();
    let duration = 50.0;
    let is_looping = true;

    // Time at epoch
    let t0 = epoch;
    let desired0 = VideoSynchronizer::compute_desired_time(&t0, &epoch, duration, is_looping);
    assert!((desired0 - 0.0).abs() < 1e-4);

    // Time +10s
    let t10 = epoch.add_seconds(10.0);
    let desired10 = VideoSynchronizer::compute_desired_time(&t10, &epoch, duration, is_looping);
    assert!((desired10 - 10.0).abs() < 1e-4);

    // Time +60s (exceeds duration 50s, loop to 10s)
    let t60 = epoch.add_seconds(60.0);
    let desired60 = VideoSynchronizer::compute_desired_time(&t60, &epoch, duration, is_looping);
    assert!((desired60 - 10.0).abs() < 1e-4);

    // Time -1s (wraps backward to 49s)
    let t_minus_1 = epoch.add_seconds(-1.0);
    let desired_minus_1 = VideoSynchronizer::compute_desired_time(&t_minus_1, &epoch, duration, is_looping);
    assert!((desired_minus_1 - 49.0).abs() < 1e-4);
}

#[test]
fn test_syncs_time_when_not_looping() {
    let epoch = JulianDate::from_iso8601("2015-11-01T00:00:00Z").unwrap();
    let duration = 50.0;
    let is_looping = false;

    // Time at epoch
    let t0 = epoch;
    let desired0 = VideoSynchronizer::compute_desired_time(&t0, &epoch, duration, is_looping);
    assert!((desired0 - 0.0).abs() < 1e-4);

    // Time +10s
    let t10 = epoch.add_seconds(10.0);
    let desired10 = VideoSynchronizer::compute_desired_time(&t10, &epoch, duration, is_looping);
    assert!((desired10 - 10.0).abs() < 1e-4);

    // Time +60s (clamped to duration 50s)
    let t60 = epoch.add_seconds(60.0);
    let desired60 = VideoSynchronizer::compute_desired_time(&t60, &epoch, duration, is_looping);
    assert!((desired60 - 50.0).abs() < 1e-4);

    // Time -1s (clamped to 0s)
    let t_minus_1 = epoch.add_seconds(-1.0);
    let desired_minus_1 = VideoSynchronizer::compute_desired_time(&t_minus_1, &epoch, duration, is_looping);
    assert!((desired_minus_1 - 0.0).abs() < 1e-4);
}

#[test]
fn test_plays_and_pauses_based_on_clock() {
    let epoch = JulianDate::from_iso8601("2015-11-01T00:00:00Z").unwrap();
    let synchronizer = VideoSynchronizer::new(Some(VideoSynchronizerOptions {
        epoch: Some(epoch),
        ..Default::default()
    }));

    let mut element = VideoElementMock {
        paused: false,
        ..Default::default()
    };

    // should_animate = false -> pause
    synchronizer.sync_tick(&epoch, false, &mut element);
    assert_eq!(element.pause_calls, 1);
    assert!(element.paused);

    // should_animate = true -> play
    synchronizer.sync_tick(&epoch, true, &mut element);
    assert_eq!(element.play_calls, 1);
    assert!(!element.paused);

    // should_animate = false -> pause
    synchronizer.sync_tick(&epoch, false, &mut element);
    assert_eq!(element.pause_calls, 2);
    assert!(element.paused);
}
