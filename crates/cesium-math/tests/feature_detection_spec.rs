use cesium_math::FeatureDetection;

#[test]
fn test_detects_basic_support() {
    assert!(FeatureDetection::supports_typed_arrays());
    assert!(FeatureDetection::supports_big_int64_array());
    assert!(FeatureDetection::supports_big_uint64_array());
    assert!(FeatureDetection::supports_big_int());
    assert!(FeatureDetection::supports_web_assembly());

    let _ = FeatureDetection::supports_fullscreen();
    let _ = FeatureDetection::supports_web_workers();
}

#[test]
fn test_user_agent_detection_chrome() {
    let ua = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.6099.109 Safari/537.36";
    let info = FeatureDetection::parse_user_agent(ua);

    assert!(info.is_chrome);
    assert!(!info.is_edge);
    assert_eq!(info.chrome_version, Some(vec![120, 0, 6099, 109]));
    assert!(info.is_webkit);
    assert_eq!(info.webkit_version.unwrap().version, vec![537, 36]);
}

#[test]
fn test_user_agent_detection_edge() {
    let ua = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36 Edg/120.0.2210.91";
    let info = FeatureDetection::parse_user_agent(ua);

    assert!(info.is_edge);
    assert!(!info.is_chrome);
    assert_eq!(info.edge_version, Some(vec![120, 0, 2210, 91]));
}

#[test]
fn test_user_agent_detection_safari() {
    let ua = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.2.1 Safari/605.1.15";
    let info = FeatureDetection::parse_user_agent(ua);

    assert!(info.is_safari);
    assert!(!info.is_chrome);
    assert!(!info.is_edge);
    assert_eq!(info.safari_version, Some(vec![17, 2, 1]));
    assert!(info.is_webkit);
    assert_eq!(info.webkit_version.unwrap().version, vec![605, 1, 15]);
}

#[test]
fn test_user_agent_detection_firefox() {
    let ua = "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:121.0) Gecko/20100101 Firefox/121.0";
    let info = FeatureDetection::parse_user_agent(ua);

    assert!(info.is_firefox);
    assert_eq!(info.firefox_version, Some(vec![121, 0]));
}

#[test]
fn test_user_agent_detection_ios() {
    let ua = "Mozilla/5.0 (iPad; CPU OS 17_2 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.2 Mobile/15E148 Safari/604.1";
    let info = FeatureDetection::parse_user_agent(ua);

    assert!(info.is_ipad_or_ios);
}

#[test]
fn test_image_rendering_support() {
    let supported = FeatureDetection::supports_image_rendering_pixelated();
    if supported {
        assert!(FeatureDetection::image_rendering_value().is_some());
    } else {
        assert!(FeatureDetection::image_rendering_value().is_none());
    }
}

#[test]
fn test_webp_support_flow() {
    FeatureDetection::reset_web_p_for_specs();

    // Throws when uninitialized
    assert!(FeatureDetection::supports_web_p().is_err());

    // After initialize
    FeatureDetection::initialize_web_p(true);
    assert!(FeatureDetection::supports_web_p().unwrap());

    FeatureDetection::initialize_web_p(false);
    assert!(!FeatureDetection::supports_web_p().unwrap());
}
