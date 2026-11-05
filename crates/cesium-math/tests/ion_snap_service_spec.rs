use cesium_math::{
    Cartesian4, IonSnapGeometryType, IonSnapHeat, IonSnapMode, IonSnapParentGeometryType,
    IonSnapService, Matrix4,
};

#[test]
fn test_default_snap_aperture() {
    const { assert!(IonSnapService::DEFAULT_SNAP_APERTURE > 0.0) };
}

#[test]
fn test_enums_values() {
    assert_eq!(IonSnapMode::Nearest as i32, 1);
    assert_eq!(IonSnapMode::NearestKeypoint as i32, 2);
    assert_eq!(IonSnapMode::Center as i32, 8);

    assert_eq!(IonSnapHeat::None as i32, 0);
    assert_eq!(IonSnapHeat::NotInRange as i32, 1);
    assert_eq!(IonSnapHeat::InRange as i32, 2);

    assert_eq!(IonSnapGeometryType::None as i32, 0);
    assert_eq!(IonSnapGeometryType::Point as i32, 1);
    assert_eq!(IonSnapGeometryType::Segment as i32, 2);
    assert_eq!(IonSnapGeometryType::Curve as i32, 3);
    assert_eq!(IonSnapGeometryType::Arc as i32, 4);
    assert_eq!(IonSnapGeometryType::Surface as i32, 5);

    assert_eq!(IonSnapParentGeometryType::None as i32, 0);
    assert_eq!(IonSnapParentGeometryType::Wire as i32, 1);
    assert_eq!(IonSnapParentGeometryType::Sheet as i32, 2);
    assert_eq!(IonSnapParentGeometryType::Solid as i32, 3);
    assert_eq!(IonSnapParentGeometryType::Mesh as i32, 4);
    assert_eq!(IonSnapParentGeometryType::Text as i32, 5);
}

#[test]
fn test_compute_world_to_view_maps_ndc_to_screen_pixels() {
    let view_matrix = Matrix4::IDENTITY;
    let projection_matrix = Matrix4::IDENTITY;
    let width = 800.0;
    let height = 600.0;

    let world_to_view =
        IonSnapService::compute_world_to_view(&view_matrix, &projection_matrix, width, height)
            .unwrap();

    let ndc_center = Cartesian4::new(0.0, 0.0, 0.0, 1.0);
    let screen_center = world_to_view.multiply_by_vector(ndc_center);

    assert!((screen_center.x - 400.0).abs() < 1e-4);
    assert!((screen_center.y - 300.0).abs() < 1e-4);
    assert!((screen_center.w - 1.0).abs() < 1e-4);

    // Invalid dimension throws
    assert!(IonSnapService::compute_world_to_view(&view_matrix, &projection_matrix, 0.0, 600.0).is_err());
    assert!(IonSnapService::compute_world_to_view(&view_matrix, &projection_matrix, 800.0, -1.0).is_err());
}

#[test]
fn test_parse_ecef_transform() {
    let rows = [
        [1.0, 0.0, 0.0, 100.0],
        [0.0, 1.0, 0.0, 200.0],
        [0.0, 0.0, 1.0, 300.0],
        [0.0, 0.0, 0.0, 1.0],
    ];

    let mat = IonSnapService::parse_ecef_transform(&rows).unwrap();
    assert_eq!(mat.get_element(0, 3).unwrap(), 100.0);
    assert_eq!(mat.get_element(1, 3).unwrap(), 200.0);
    assert_eq!(mat.get_element(2, 3).unwrap(), 300.0);
    assert_eq!(mat.get_element(3, 3).unwrap(), 1.0);
}
