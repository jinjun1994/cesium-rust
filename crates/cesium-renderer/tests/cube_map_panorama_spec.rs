use cesium_math::{DeveloperError, Matrix4};
use cesium_renderer::{CubeMapPanorama, CubeMapPanoramaOptions, CubeMapUrls};

fn make_valid_sources() -> CubeMapUrls {
    CubeMapUrls {
        positive_x: Some("px.png".to_string()),
        negative_x: Some("nx.png".to_string()),
        positive_y: Some("py.png".to_string()),
        negative_y: Some("ny.png".to_string()),
        positive_z: Some("pz.png".to_string()),
        negative_z: Some("nz.png".to_string()),
    }
}

#[test]
fn test_constructs_with_default_values() {
    let panorama = CubeMapPanorama::new(CubeMapPanoramaOptions {
        sources: Some(make_valid_sources()),
        ..Default::default()
    });

    assert!(panorama.show);
    assert_eq!(panorama.transform, None);
    assert!(!panorama.is_destroyed());
}

#[test]
fn test_stores_transform_if_provided() {
    let transform = Matrix4::IDENTITY;
    let panorama = CubeMapPanorama::new(CubeMapPanoramaOptions {
        sources: Some(make_valid_sources()),
        transform: Some(transform),
        ..Default::default()
    });

    assert_eq!(panorama.transform, Some(transform));
}

#[test]
fn test_creates_credit_from_string() {
    let panorama = CubeMapPanorama::new(CubeMapPanoramaOptions {
        sources: Some(make_valid_sources()),
        credit: Some("Test Credit".to_string()),
        ..Default::default()
    });

    assert_eq!(panorama.credit, Some("Test Credit".to_string()));
}

#[test]
fn test_does_not_update_when_show_is_false() {
    let mut panorama = CubeMapPanorama::new(CubeMapPanoramaOptions {
        sources: Some(make_valid_sources()),
        show: Some(false),
        ..Default::default()
    });

    let result = panorama.update(true, true).expect("should succeed");
    assert!(result.is_none());
}

#[test]
fn test_does_not_update_in_non_3d_modes() {
    let mut panorama = CubeMapPanorama::new(CubeMapPanoramaOptions {
        sources: Some(make_valid_sources()),
        ..Default::default()
    });

    // 2D mode (is_3d = false)
    let result = panorama.update(false, true).expect("should succeed");
    assert!(result.is_none());
}

#[test]
fn test_throws_if_sources_are_missing_faces() {
    let mut panorama = CubeMapPanorama::new(CubeMapPanoramaOptions {
        sources: Some(CubeMapUrls {
            positive_x: Some("px.png".to_string()),
            ..Default::default()
        }),
        ..Default::default()
    });

    let err = panorama
        .update(true, true)
        .expect_err("should throw DeveloperError");
    assert_eq!(err.name(), DeveloperError::NAME);
}

#[test]
fn test_destroys_resources() {
    let mut panorama = CubeMapPanorama::new(CubeMapPanoramaOptions {
        sources: Some(make_valid_sources()),
        ..Default::default()
    });

    panorama.destroy();
    assert!(panorama.is_destroyed());
}
