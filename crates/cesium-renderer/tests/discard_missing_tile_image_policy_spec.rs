use cesium_math::Cartesian2;
use cesium_renderer::{
    DiscardMissingError, DiscardMissingTileImagePolicy, DiscardMissingTileImagePolicyOptions,
};

#[test]
fn test_throws_if_missing_image_url_is_not_provided() {
    let err = DiscardMissingTileImagePolicy::new(DiscardMissingTileImagePolicyOptions {
        missing_image_url: None,
        pixels_to_check: Some(vec![Cartesian2::new(0.0, 0.0)]),
        disable_check_if_all_pixels_are_transparent: false,
        preloaded_image: None,
    })
    .expect_err("should require missing_image_url");

    assert_eq!(err, DiscardMissingError::MissingImageUrlRequired);
}

#[test]
fn test_throws_if_pixels_to_check_is_not_provided() {
    let err = DiscardMissingTileImagePolicy::new(DiscardMissingTileImagePolicyOptions {
        missing_image_url: Some("http://some.host.invalid/missingImage.png".to_string()),
        pixels_to_check: None,
        disable_check_if_all_pixels_are_transparent: false,
        preloaded_image: None,
    })
    .expect_err("should require pixels_to_check");

    assert_eq!(err, DiscardMissingError::PixelsToCheckRequired);
}

#[test]
fn test_discards_image_identical_to_missing_image() {
    // 2x2 red image
    let red_image = vec![
        255, 0, 0, 255, 255, 0, 0, 255,
        255, 0, 0, 255, 255, 0, 0, 255,
    ];
    // 2x2 green image
    let green_image = vec![
        0, 255, 0, 255, 0, 255, 0, 255,
        0, 255, 0, 255, 0, 255, 0, 255,
    ];

    let policy = DiscardMissingTileImagePolicy::new(DiscardMissingTileImagePolicyOptions {
        missing_image_url: Some("Data/Images/Red16x16.png".to_string()),
        pixels_to_check: Some(vec![Cartesian2::new(0.0, 0.0)]),
        disable_check_if_all_pixels_are_transparent: false,
        preloaded_image: Some((2, 2, red_image.clone())),
    })
    .expect("should construct");

    assert!(policy.is_ready());
    assert!(policy.should_discard_image(2, 2, &red_image).unwrap());
    assert!(!policy.should_discard_image(2, 2, &green_image).unwrap());
}

#[test]
fn test_discards_transparent_when_check_not_disabled() {
    let transparent_image = vec![0, 0, 0, 0];

    let policy = DiscardMissingTileImagePolicy::new(DiscardMissingTileImagePolicyOptions {
        missing_image_url: Some("Data/Images/Transparent.png".to_string()),
        pixels_to_check: Some(vec![Cartesian2::new(0.0, 0.0)]),
        disable_check_if_all_pixels_are_transparent: false,
        preloaded_image: Some((1, 1, transparent_image.clone())),
    })
    .expect("should construct");

    assert!(policy.is_ready());
    assert!(policy
        .should_discard_image(1, 1, &transparent_image)
        .unwrap());
}

#[test]
fn test_does_not_discard_when_all_pixels_transparent_and_flag_set() {
    let transparent_image = vec![0, 0, 0, 0];

    let policy = DiscardMissingTileImagePolicy::new(DiscardMissingTileImagePolicyOptions {
        missing_image_url: Some("Data/Images/Transparent.png".to_string()),
        pixels_to_check: Some(vec![Cartesian2::new(0.0, 0.0)]),
        disable_check_if_all_pixels_are_transparent: true,
        preloaded_image: Some((1, 1, transparent_image.clone())),
    })
    .expect("should construct");

    assert!(policy.is_ready());
    // Should not discard because check is disabled
    assert!(!policy
        .should_discard_image(1, 1, &transparent_image)
        .unwrap());
}

#[test]
fn test_throws_if_called_before_policy_is_ready() {
    let policy = DiscardMissingTileImagePolicy::new(DiscardMissingTileImagePolicyOptions {
        missing_image_url: Some("Data/Images/Transparent.png".to_string()),
        pixels_to_check: Some(vec![Cartesian2::new(0.0, 0.0)]),
        disable_check_if_all_pixels_are_transparent: true,
        preloaded_image: None,
    })
    .expect("should construct");

    assert!(!policy.is_ready());
    let err = policy
        .should_discard_image(1, 1, &[0, 0, 0, 0])
        .expect_err("should reject when not ready");
    assert_eq!(err, DiscardMissingError::NotReady);
}
