use cesium_renderer::sky_box::{SkyBox, SkyBoxFaceSource, SkyBoxOptions, SkyBoxSources};

fn make_sources_all_url(path: &str) -> SkyBoxSources {
    SkyBoxSources {
        positive_x: Some(SkyBoxFaceSource::from(path)),
        negative_x: Some(SkyBoxFaceSource::from(path)),
        positive_y: Some(SkyBoxFaceSource::from(path)),
        negative_y: Some(SkyBoxFaceSource::from(path)),
        positive_z: Some(SkyBoxFaceSource::from(path)),
        negative_z: Some(SkyBoxFaceSource::from(path)),
    }
}

fn make_sources_all_images(bytes: &[u8]) -> SkyBoxSources {
    SkyBoxSources {
        positive_x: Some(SkyBoxFaceSource::from(bytes.to_vec())),
        negative_x: Some(SkyBoxFaceSource::from(bytes.to_vec())),
        positive_y: Some(SkyBoxFaceSource::from(bytes.to_vec())),
        negative_y: Some(SkyBoxFaceSource::from(bytes.to_vec())),
        positive_z: Some(SkyBoxFaceSource::from(bytes.to_vec())),
        negative_z: Some(SkyBoxFaceSource::from(bytes.to_vec())),
    }
}

#[test]
fn test_draws_sky_box_from_images_and_urls() {
    let mut sky_box_urls = SkyBox::new(SkyBoxOptions {
        sources: Some(make_sources_all_url("Blue.png")),
        show: Some(true),
    });

    let cmd1 = sky_box_urls
        .update(true, true)
        .expect("update should succeed");
    assert!(cmd1.is_some());

    let mut sky_box_images = SkyBox::new(SkyBoxOptions {
        sources: Some(make_sources_all_images(&[0, 0, 255, 255])),
        show: Some(true),
    });

    let cmd2 = sky_box_images
        .update(true, true)
        .expect("update should succeed");
    assert!(cmd2.is_some());
}

#[test]
fn test_does_not_render_when_show_is_false() {
    let mut sky_box = SkyBox::new(SkyBoxOptions {
        sources: Some(make_sources_all_url("Blue.png")),
        show: Some(false),
    });

    assert_eq!(sky_box.update(true, true).unwrap(), None);

    sky_box.show = true;
    assert!(sky_box.update(true, true).unwrap().is_some());

    sky_box.show = false;
    assert_eq!(sky_box.update(true, true).unwrap(), None);
}

#[test]
fn test_does_not_render_in_2d_or_without_render_pass() {
    let mut sky_box = SkyBox::new(SkyBoxOptions {
        sources: Some(make_sources_all_url("Blue.png")),
        show: Some(true),
    });

    // In 2D
    assert_eq!(sky_box.update(false, true).unwrap(), None);

    // In 3D without render pass
    assert_eq!(sky_box.update(true, false).unwrap(), None);
}

#[test]
fn test_gets_constructor_options() {
    let sky_box = SkyBox::new(SkyBoxOptions {
        sources: Some(SkyBoxSources {
            positive_x: Some(SkyBoxFaceSource::from("positiveX.png")),
            negative_x: Some(SkyBoxFaceSource::from("negativeX.png")),
            positive_y: Some(SkyBoxFaceSource::from("positiveY.png")),
            negative_y: Some(SkyBoxFaceSource::from("negativeY.png")),
            positive_z: Some(SkyBoxFaceSource::from("positiveZ.png")),
            negative_z: Some(SkyBoxFaceSource::from("negativeZ.png")),
        }),
        show: Some(false),
    });

    let sources = sky_box.sources.as_ref().unwrap();
    assert_eq!(sources.positive_x.as_ref().unwrap().as_url(), Some("positiveX.png"));
    assert_eq!(sources.negative_x.as_ref().unwrap().as_url(), Some("negativeX.png"));
    assert_eq!(sources.positive_y.as_ref().unwrap().as_url(), Some("positiveY.png"));
    assert_eq!(sources.negative_y.as_ref().unwrap().as_url(), Some("negativeY.png"));
    assert_eq!(sources.positive_z.as_ref().unwrap().as_url(), Some("positiveZ.png"));
    assert_eq!(sources.negative_z.as_ref().unwrap().as_url(), Some("negativeZ.png"));
    assert!(!sky_box.show);
}

#[test]
fn test_is_destroyed() {
    let mut sky_box = SkyBox::new(SkyBoxOptions {
        sources: Some(make_sources_all_url("Blue.png")),
        show: Some(true),
    });

    assert!(!sky_box.is_destroyed());
    sky_box.destroy();
    assert!(sky_box.is_destroyed());

    assert!(sky_box.update(true, true).is_err());
}

#[test]
fn test_throws_when_face_missing() {
    let mut sky_box = SkyBox::new(SkyBoxOptions {
        sources: Some(SkyBoxSources {
            positive_x: None,
            negative_x: Some(SkyBoxFaceSource::from("Blue.png")),
            positive_y: Some(SkyBoxFaceSource::from("Blue.png")),
            negative_y: Some(SkyBoxFaceSource::from("Blue.png")),
            positive_z: Some(SkyBoxFaceSource::from("Blue.png")),
            negative_z: Some(SkyBoxFaceSource::from("Blue.png")),
        }),
        show: Some(true),
    });

    let res = sky_box.update(true, true);
    assert!(res.is_err());
    assert!(res.unwrap_err().message.contains("Missing positiveX"));
}

#[test]
fn test_throws_when_face_type_mismatched() {
    let mut sky_box = SkyBox::new(SkyBoxOptions {
        sources: Some(SkyBoxSources {
            positive_x: Some(SkyBoxFaceSource::from(vec![1, 2, 3])),
            negative_x: Some(SkyBoxFaceSource::from("Blue.png")),
            positive_y: Some(SkyBoxFaceSource::from("Blue.png")),
            negative_y: Some(SkyBoxFaceSource::from("Blue.png")),
            positive_z: Some(SkyBoxFaceSource::from("Blue.png")),
            negative_z: Some(SkyBoxFaceSource::from("Blue.png")),
        }),
        show: Some(true),
    });

    let res = sky_box.update(true, true);
    assert!(res.is_err());
    assert!(res.unwrap_err().message.contains("properties must all be the same type"));
}
