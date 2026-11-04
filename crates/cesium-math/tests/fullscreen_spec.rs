use cesium_math::Fullscreen;

#[test]
fn test_fullscreen_supported_flow() {
    Fullscreen::reset_for_specs();

    assert!(Fullscreen::supports_fullscreen());
    assert_eq!(Fullscreen::is_enabled(), Some(true));
    assert_eq!(Fullscreen::element(), Some(None));
    assert_eq!(Fullscreen::is_fullscreen(), Some(false));
    assert_eq!(Fullscreen::change_event_name(), Some("fullscreenchange"));
    assert_eq!(Fullscreen::error_event_name(), Some("fullscreenerror"));

    // Request fullscreen
    Fullscreen::request_fullscreen(Some("canvas"));
    assert_eq!(Fullscreen::is_fullscreen(), Some(true));
    assert_eq!(Fullscreen::element(), Some(Some("canvas".to_string())));

    // Exit fullscreen
    Fullscreen::exit_fullscreen();
    assert_eq!(Fullscreen::is_fullscreen(), Some(false));
    assert_eq!(Fullscreen::element(), Some(None));
}

#[test]
fn test_fullscreen_unsupported_flow() {
    Fullscreen::reset_for_specs();
    Fullscreen::set_supports_fullscreen_for_specs(false);

    assert!(!Fullscreen::supports_fullscreen());
    assert_eq!(Fullscreen::is_enabled(), None);
    assert_eq!(Fullscreen::element(), None);
    assert_eq!(Fullscreen::is_fullscreen(), None);
    assert_eq!(Fullscreen::change_event_name(), None);
    assert_eq!(Fullscreen::error_event_name(), None);

    // No-op when unsupported
    Fullscreen::request_fullscreen(Some("canvas"));
    assert_eq!(Fullscreen::is_fullscreen(), None);

    Fullscreen::exit_fullscreen();
    assert_eq!(Fullscreen::is_fullscreen(), None);
}
