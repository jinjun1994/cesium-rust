//! Direct port and verification against CesiumJS `Renderer/loadCubeMapSpec.js` (Apache-2.0).

use cesium_renderer::{load_cube_map, Context, ContextOptions, CubeMapUrls};

fn create_test_context() -> Context {
    Context::new(ContextOptions {
        width: Some(1),
        height: Some(1),
        ..Default::default()
    })
    .unwrap()
}

#[test]
fn test_loads_a_cube_map() {
    let context = create_test_context();
    let urls = CubeMapUrls {
        positive_x: Some("./Data/Images/Green.png".to_string()),
        negative_x: Some("./Data/Images/Blue.png".to_string()),
        positive_y: Some("./Data/Images/Green.png".to_string()),
        negative_y: Some("./Data/Images/Blue.png".to_string()),
        positive_z: Some("./Data/Images/Green.png".to_string()),
        negative_z: Some("./Data/Images/Blue.png".to_string()),
    };

    let cm = load_cube_map(Some(&context), Some(urls)).unwrap();
    assert_eq!(cm.width, 1);
    assert_eq!(cm.height, 1);

    // +X is green
    let c_px = cm.sample_direction([1.0, 0.0, 0.0]);
    assert_eq!(c_px.red, 0.0);
    assert_eq!(c_px.green, 1.0);
    assert_eq!(c_px.blue, 0.0);

    // -X is blue
    let c_nx = cm.sample_direction([-1.0, 0.0, 0.0]);
    assert_eq!(c_nx.red, 0.0);
    assert_eq!(c_nx.green, 0.0);
    assert_eq!(c_nx.blue, 1.0);

    // +Y is green
    let c_py = cm.sample_direction([0.0, 1.0, 0.0]);
    assert_eq!(c_py.red, 0.0);
    assert_eq!(c_py.green, 1.0);
    assert_eq!(c_py.blue, 0.0);

    // -Y is blue
    let c_ny = cm.sample_direction([0.0, -1.0, 0.0]);
    assert_eq!(c_ny.red, 0.0);
    assert_eq!(c_ny.green, 0.0);
    assert_eq!(c_ny.blue, 1.0);

    // +Z is green
    let c_pz = cm.sample_direction([0.0, 0.0, 1.0]);
    assert_eq!(c_pz.red, 0.0);
    assert_eq!(c_pz.green, 1.0);
    assert_eq!(c_pz.blue, 0.0);

    // -Z is blue
    let c_nz = cm.sample_direction([0.0, 0.0, -1.0]);
    assert_eq!(c_nz.red, 0.0);
    assert_eq!(c_nz.green, 0.0);
    assert_eq!(c_nz.blue, 1.0);
}

#[test]
fn test_calls_error_function_when_positive_x_does_not_exist() {
    let context = create_test_context();
    let urls = CubeMapUrls {
        positive_x: Some("does_not_exist.png".to_string()),
        negative_x: Some("./Data/Images/Blue.png".to_string()),
        positive_y: Some("./Data/Images/Green.png".to_string()),
        negative_y: Some("./Data/Images/Blue.png".to_string()),
        positive_z: Some("./Data/Images/Green.png".to_string()),
        negative_z: Some("./Data/Images/Blue.png".to_string()),
    };
    assert!(load_cube_map(Some(&context), Some(urls)).is_err());
}

#[test]
fn test_calls_error_function_when_negative_x_does_not_exist() {
    let context = create_test_context();
    let urls = CubeMapUrls {
        positive_x: Some("./Data/Images/Green.png".to_string()),
        negative_x: Some("does_not_exist.png".to_string()),
        positive_y: Some("./Data/Images/Green.png".to_string()),
        negative_y: Some("./Data/Images/Blue.png".to_string()),
        positive_z: Some("./Data/Images/Green.png".to_string()),
        negative_z: Some("./Data/Images/Blue.png".to_string()),
    };
    assert!(load_cube_map(Some(&context), Some(urls)).is_err());
}

#[test]
fn test_calls_error_function_when_positive_y_does_not_exist() {
    let context = create_test_context();
    let urls = CubeMapUrls {
        positive_x: Some("./Data/Images/Green.png".to_string()),
        negative_x: Some("./Data/Images/Blue.png".to_string()),
        positive_y: Some("does_not_exist.png".to_string()),
        negative_y: Some("./Data/Images/Blue.png".to_string()),
        positive_z: Some("./Data/Images/Green.png".to_string()),
        negative_z: Some("./Data/Images/Blue.png".to_string()),
    };
    assert!(load_cube_map(Some(&context), Some(urls)).is_err());
}

#[test]
fn test_calls_error_function_when_negative_y_does_not_exist() {
    let context = create_test_context();
    let urls = CubeMapUrls {
        positive_x: Some("./Data/Images/Green.png".to_string()),
        negative_x: Some("./Data/Images/Blue.png".to_string()),
        positive_y: Some("./Data/Images/Green.png".to_string()),
        negative_y: Some("does_not_exist.png".to_string()),
        positive_z: Some("./Data/Images/Green.png".to_string()),
        negative_z: Some("./Data/Images/Blue.png".to_string()),
    };
    assert!(load_cube_map(Some(&context), Some(urls)).is_err());
}

#[test]
fn test_calls_error_function_when_positive_z_does_not_exist() {
    let context = create_test_context();
    let urls = CubeMapUrls {
        positive_x: Some("./Data/Images/Green.png".to_string()),
        negative_x: Some("./Data/Images/Blue.png".to_string()),
        positive_y: Some("./Data/Images/Green.png".to_string()),
        negative_y: Some("./Data/Images/Blue.png".to_string()),
        positive_z: Some("does_not_exist.png".to_string()),
        negative_z: Some("./Data/Images/Blue.png".to_string()),
    };
    assert!(load_cube_map(Some(&context), Some(urls)).is_err());
}

#[test]
fn test_calls_error_function_when_negative_z_does_not_exist() {
    let context = create_test_context();
    let urls = CubeMapUrls {
        positive_x: Some("./Data/Images/Green.png".to_string()),
        negative_x: Some("./Data/Images/Blue.png".to_string()),
        positive_y: Some("./Data/Images/Green.png".to_string()),
        negative_y: Some("./Data/Images/Blue.png".to_string()),
        positive_z: Some("./Data/Images/Green.png".to_string()),
        negative_z: Some("does_not_exist.png".to_string()),
    };
    assert!(load_cube_map(Some(&context), Some(urls)).is_err());
}

#[test]
fn test_throws_without_a_context() {
    let urls = CubeMapUrls {
        positive_x: Some("px".to_string()),
        negative_x: Some("nx".to_string()),
        positive_y: Some("py".to_string()),
        negative_y: Some("ny".to_string()),
        positive_z: Some("pz".to_string()),
        negative_z: Some("nz".to_string()),
    };
    assert!(load_cube_map(None, Some(urls)).is_err());
}

#[test]
fn test_throws_without_urls() {
    let context = create_test_context();
    assert!(load_cube_map(Some(&context), None).is_err());
}

#[test]
fn test_throws_without_positive_x() {
    let context = create_test_context();
    let urls = CubeMapUrls {
        positive_x: None,
        negative_x: Some("nx".to_string()),
        positive_y: Some("py".to_string()),
        negative_y: Some("ny".to_string()),
        positive_z: Some("pz".to_string()),
        negative_z: Some("nz".to_string()),
    };
    assert!(load_cube_map(Some(&context), Some(urls)).is_err());
}

#[test]
fn test_throws_without_negative_x() {
    let context = create_test_context();
    let urls = CubeMapUrls {
        positive_x: Some("px".to_string()),
        negative_x: None,
        positive_y: Some("py".to_string()),
        negative_y: Some("ny".to_string()),
        positive_z: Some("pz".to_string()),
        negative_z: Some("nz".to_string()),
    };
    assert!(load_cube_map(Some(&context), Some(urls)).is_err());
}

#[test]
fn test_throws_without_positive_y() {
    let context = create_test_context();
    let urls = CubeMapUrls {
        positive_x: Some("px".to_string()),
        negative_x: Some("nx".to_string()),
        positive_y: None,
        negative_y: Some("ny".to_string()),
        positive_z: Some("pz".to_string()),
        negative_z: Some("nz".to_string()),
    };
    assert!(load_cube_map(Some(&context), Some(urls)).is_err());
}

#[test]
fn test_throws_without_negative_y() {
    let context = create_test_context();
    let urls = CubeMapUrls {
        positive_x: Some("px".to_string()),
        negative_x: Some("nx".to_string()),
        positive_y: Some("py".to_string()),
        negative_y: None,
        positive_z: Some("pz".to_string()),
        negative_z: Some("nz".to_string()),
    };
    assert!(load_cube_map(Some(&context), Some(urls)).is_err());
}

#[test]
fn test_throws_without_positive_z() {
    let context = create_test_context();
    let urls = CubeMapUrls {
        positive_x: Some("px".to_string()),
        negative_x: Some("nx".to_string()),
        positive_y: Some("py".to_string()),
        negative_y: Some("ny".to_string()),
        positive_z: None,
        negative_z: Some("nz".to_string()),
    };
    assert!(load_cube_map(Some(&context), Some(urls)).is_err());
}

#[test]
fn test_throws_without_negative_z() {
    let context = create_test_context();
    let urls = CubeMapUrls {
        positive_x: Some("px".to_string()),
        negative_x: Some("nx".to_string()),
        positive_y: Some("py".to_string()),
        negative_y: Some("ny".to_string()),
        positive_z: Some("pz".to_string()),
        negative_z: None,
    };
    assert!(load_cube_map(Some(&context), Some(urls)).is_err());
}
