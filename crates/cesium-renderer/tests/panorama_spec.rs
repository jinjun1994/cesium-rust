//! PanoramaSpec matching CesiumJS `Specs/Scene/PanoramaSpec.js` (Apache-2.0).

use cesium_math::{Cartesian3, Credit, Matrix4};
use cesium_renderer::cube_map::CubeMapUrls;
use cesium_renderer::cube_map_panorama::{CubeMapPanorama, CubeMapPanoramaOptions};
use cesium_renderer::equirectangular_panorama::{
    EquirectangularPanorama, EquirectangularPanoramaOptions,
};
use cesium_renderer::panorama::{Panorama, PanoramaTrait};

#[test]
fn test_cannot_be_instantiated() {
    let result = Panorama::new();
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.message.contains(Panorama::INSTANTIATION_ERROR_MESSAGE));
}

#[test]
fn test_abstract_properties_throw_developer_error() {
    let panorama = Panorama;

    // show throws DeveloperError
    let show_res = panorama.show();
    assert!(show_res.is_err());
    assert!(show_res
        .unwrap_err()
        .message
        .contains(Panorama::INSTANTIATION_ERROR_MESSAGE));

    // transform throws DeveloperError
    let transform_res = panorama.transform();
    assert!(transform_res.is_err());
    assert!(transform_res
        .unwrap_err()
        .message
        .contains(Panorama::INSTANTIATION_ERROR_MESSAGE));

    // credit throws DeveloperError
    let credit_res = panorama.credit();
    assert!(credit_res.is_err());
    assert!(credit_res
        .unwrap_err()
        .message
        .contains(Panorama::INSTANTIATION_ERROR_MESSAGE));
}

#[test]
fn test_concrete_implementations_implement_panorama_trait() {
    // CubeMapPanorama
    let mut cube_pano = CubeMapPanorama::new(CubeMapPanoramaOptions {
        sources: Some(CubeMapUrls {
            positive_x: Some("px.png".to_string()),
            negative_x: Some("nx.png".to_string()),
            positive_y: Some("py.png".to_string()),
            negative_y: Some("ny.png".to_string()),
            positive_z: Some("pz.png".to_string()),
            negative_z: Some("nz.png".to_string()),
        }),
        transform: Some(Matrix4::from_translation(Cartesian3::new(1.0, 2.0, 3.0))),
        show: Some(true),
        credit: Some("CubeMap credit".to_string()),
    });

    assert!(cube_pano.show());
    cube_pano.set_show(false);
    assert!(!cube_pano.show());
    assert!(cube_pano.transform().is_some());
    assert!(!cube_pano.is_destroyed());
    cube_pano.destroy();
    assert!(cube_pano.is_destroyed());

    // EquirectangularPanorama
    let custom_credit = Credit::new("Equi credit", None);
    let mut equi_pano = EquirectangularPanorama::new(EquirectangularPanoramaOptions {
        image: Some("test.png".to_string()),
        transform: Some(Matrix4::IDENTITY),
        credit: Some(custom_credit.clone()),
        show: Some(true),
        ..Default::default()
    })
    .unwrap();

    assert!(equi_pano.show());
    equi_pano.set_show(false);
    assert!(!equi_pano.show());
    assert_eq!(equi_pano.transform(), &Matrix4::IDENTITY);
    assert_eq!(PanoramaTrait::transform(&equi_pano), Some(&Matrix4::IDENTITY));
    assert_eq!(equi_pano.credit(), Some(&custom_credit));
    assert!(!equi_pano.is_destroyed());
    equi_pano.destroy();
    assert!(equi_pano.is_destroyed());
}
