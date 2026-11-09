use cesium_math::{Cartesian3, DeveloperError, Matrix4};
use cesium_renderer::{EllipsoidPrimitive, EllipsoidPrimitiveOptions, Material};

#[test]
fn test_gets_default_properties() {
    let mut ellipsoid = EllipsoidPrimitive::default();
    assert!(ellipsoid.show);
    assert_eq!(ellipsoid.center, Cartesian3::ZERO);
    assert!(ellipsoid.radii.is_none());
    assert_eq!(ellipsoid.model_matrix, Matrix4::IDENTITY);
    assert_eq!(
        ellipsoid.material.as_ref().unwrap().material_type,
        Material::COLOR_TYPE
    );
    assert!(!ellipsoid.debug_show_bounding_volume);
    ellipsoid.destroy();
    assert!(ellipsoid.is_destroyed());
}

#[test]
fn test_constructs_with_options() {
    let material = Material::from_type(Material::STRIPE_TYPE).unwrap();
    let mut e = EllipsoidPrimitive::new(EllipsoidPrimitiveOptions {
        center: Some(Cartesian3::new(1.0, 2.0, 3.0)),
        radii: Some(Cartesian3::new(4.0, 5.0, 6.0)),
        model_matrix: Some(Matrix4::from_uniform_scale(2.0)),
        show: Some(false),
        material: Some(material.clone()),
        id: Some("id".to_string()),
        debug_show_bounding_volume: Some(true),
    });

    assert_eq!(e.center, Cartesian3::new(1.0, 2.0, 3.0));
    assert_eq!(e.radii, Some(Cartesian3::new(4.0, 5.0, 6.0)));
    assert_eq!(e.model_matrix, Matrix4::from_uniform_scale(2.0));
    assert!(!e.show);
    assert_eq!(e.material.as_ref().unwrap().material_type, material.material_type);
    assert_eq!(e.id.as_deref(), Some("id"));
    assert!(e.debug_show_bounding_volume);

    e.destroy();
    assert!(e.is_destroyed());
}

#[test]
fn test_renders_with_default_material() {
    let mut ellipsoid = EllipsoidPrimitive::default();
    ellipsoid.radii = Some(Cartesian3::new(1.0, 1.0, 1.0));

    let cmd = ellipsoid.update().expect("should succeed");
    assert!(cmd.is_some());
}

#[test]
fn test_does_not_render_when_show_is_false() {
    let mut ellipsoid = EllipsoidPrimitive::default();
    ellipsoid.radii = Some(Cartesian3::new(1.0, 1.0, 1.0));
    ellipsoid.show = false;

    let cmd = ellipsoid.update().expect("should succeed");
    assert!(cmd.is_none());
}

#[test]
fn test_does_not_render_without_radii() {
    let ellipsoid = EllipsoidPrimitive::default();
    let cmd = ellipsoid.update().expect("should succeed");
    assert!(cmd.is_none());
}

#[test]
fn test_throws_when_rendered_without_material() {
    let mut ellipsoid = EllipsoidPrimitive::default();
    ellipsoid.radii = Some(Cartesian3::new(1.0, 1.0, 1.0));
    ellipsoid.material = None;

    let err = ellipsoid.update().expect_err("should throw without material");
    assert_eq!(err.name(), DeveloperError::NAME);
}

#[test]
fn test_is_destroyed() {
    let mut ellipsoid = EllipsoidPrimitive::default();
    assert!(!ellipsoid.is_destroyed());
    ellipsoid.destroy();
    assert!(ellipsoid.is_destroyed());
}
