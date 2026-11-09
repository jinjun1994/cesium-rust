use cesium_math::{Cartesian2, Cartesian3, Credit, Matrix4};
use cesium_renderer::{EquirectangularPanorama, EquirectangularPanoramaOptions};

#[test]
fn test_throws_if_image_not_provided() {
    let options = EquirectangularPanoramaOptions::default();
    let result = EquirectangularPanorama::new(options);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.message.contains("options.image is required"));
}

#[test]
fn test_can_be_constructed_with_minimal_options() {
    let panorama = EquirectangularPanorama::new(EquirectangularPanoramaOptions {
        image: Some("test.png".to_string()),
        ..Default::default()
    })
    .expect("Construction should succeed");

    assert_eq!(panorama.image(), "test.png");
    assert_eq!(panorama.transform(), &Matrix4::IDENTITY);
    assert!(panorama.credit().is_none());
    assert!(panorama.show());
    assert!(!panorama.is_destroyed());
}

#[test]
fn test_respects_user_provided_transform() {
    let transform = Matrix4::from_translation(Cartesian3::new(1.0, 2.0, 3.0));
    let panorama = EquirectangularPanorama::new(EquirectangularPanoramaOptions {
        image: Some("test.png".to_string()),
        transform: Some(transform),
        ..Default::default()
    })
    .expect("Construction should succeed");

    assert_eq!(panorama.transform(), &transform);
}

#[test]
fn test_handles_credit_object() {
    let credit = Credit::new("Custom Credit", None);
    let panorama = EquirectangularPanorama::new(EquirectangularPanoramaOptions {
        image: Some("test.png".to_string()),
        credit: Some(credit.clone()),
        ..Default::default()
    })
    .expect("Construction should succeed");

    assert_eq!(panorama.credit(), Some(&credit));
}

#[test]
fn test_handles_credit_string() {
    let credit = Credit::new("Test Credit", None);
    let panorama = EquirectangularPanorama::new(EquirectangularPanoramaOptions {
        image: Some("test.png".to_string()),
        credit: Some(credit),
        ..Default::default()
    })
    .expect("Construction should succeed");

    assert!(panorama.credit().is_some());
    assert!(panorama.credit().unwrap().html().contains("Test Credit"));
}

#[test]
fn test_supports_horizontal_and_vertical_repeat() {
    let panorama = EquirectangularPanorama::new(EquirectangularPanoramaOptions {
        image: Some("test.png".to_string()),
        repeat_horizontal: Some(2.0),
        repeat_vertical: Some(3.0),
        ..Default::default()
    })
    .expect("Construction should succeed");

    let uniforms = &panorama.primitive().appearance.material.uniforms;
    assert_eq!(uniforms.repeat, Cartesian2::new(-2.0, 3.0));
}

#[test]
fn test_update_and_destroy() {
    let mut panorama = EquirectangularPanorama::new(EquirectangularPanoramaOptions {
        image: Some("test.png".to_string()),
        ..Default::default()
    })
    .expect("Construction should succeed");

    // 2D mode returns None
    let cmd_2d = panorama.update(false, true).expect("update 2D");
    assert!(cmd_2d.is_none());

    // 3D mode returns DrawCommand
    let cmd_3d = panorama.update(true, true).expect("update 3D");
    assert!(cmd_3d.is_some());

    // show = false returns None
    panorama.set_show(false);
    assert!(!panorama.show());
    assert!(!panorama.primitive().show);
    let cmd_hidden = panorama.update(true, true).expect("update hidden");
    assert!(cmd_hidden.is_none());

    // Destroy
    assert!(!panorama.is_destroyed());
    panorama.destroy();
    assert!(panorama.is_destroyed());
    assert!(panorama.primitive().is_destroyed());

    // Calling update after destroy throws DeveloperError
    assert!(panorama.update(true, true).is_err());
}
