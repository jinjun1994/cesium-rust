use cesium_math::{Cartesian3, Color, Matrix4};
use cesium_renderer::{DebugModelMatrixPrimitive, DebugModelMatrixPrimitiveOptions};

#[test]
fn test_gets_default_properties() {
    let mut p = DebugModelMatrixPrimitive::default();
    assert_eq!(p.length, 10000000.0);
    assert_eq!(p.width, 2.0);
    assert_eq!(p.model_matrix, Matrix4::IDENTITY);
    assert!(p.show);
    assert!(p.id.is_none());
    p.destroy();
    assert!(p.is_destroyed());
}

#[test]
fn test_constructs_with_options() {
    let mut p = DebugModelMatrixPrimitive::new(DebugModelMatrixPrimitiveOptions {
        length: Some(10.0),
        width: Some(1.0),
        model_matrix: Some(Matrix4::from_uniform_scale(2.0)),
        show: Some(false),
        id: Some("id".to_string()),
    });

    assert_eq!(p.length, 10.0);
    assert_eq!(p.width, 1.0);
    assert_eq!(p.model_matrix, Matrix4::from_uniform_scale(2.0));
    assert!(!p.show);
    assert_eq!(p.id.as_deref(), Some("id"));
    p.destroy();
}

#[test]
fn test_computes_axes_and_scales() {
    let p = DebugModelMatrixPrimitive::new(DebugModelMatrixPrimitiveOptions {
        length: Some(50.0),
        model_matrix: Some(Matrix4::IDENTITY),
        ..Default::default()
    });

    let axes = p.compute_axes().expect("should compute axes");
    assert_eq!(axes.len(), 3);

    // X axis
    assert_eq!(axes[0].origin, Cartesian3::ZERO);
    assert_eq!(axes[0].direction, Cartesian3::UNIT_X);
    assert_eq!(axes[0].color, Color::RED);
    assert_eq!(axes[0].scaled_matrix, Matrix4::from_uniform_scale(50.0));

    // Y axis
    assert_eq!(axes[1].origin, Cartesian3::ZERO);
    assert_eq!(axes[1].direction, Cartesian3::UNIT_Y);
    assert_eq!(axes[1].color, Color::GREEN);

    // Z axis
    assert_eq!(axes[2].origin, Cartesian3::ZERO);
    assert_eq!(axes[2].direction, Cartesian3::UNIT_Z);
    assert_eq!(axes[2].color, Color::BLUE);
}

#[test]
fn test_is_destroyed() {
    let mut p = DebugModelMatrixPrimitive::default();
    assert!(!p.is_destroyed());
    p.destroy();
    assert!(p.is_destroyed());
}
