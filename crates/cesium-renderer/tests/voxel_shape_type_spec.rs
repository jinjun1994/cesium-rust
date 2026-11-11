use std::str::FromStr;

use cesium_renderer::{
    VoxelBoxShape, VoxelCylinderShape, VoxelEllipsoidShape, VoxelShapeType,
};

#[test]
fn test_voxel_shape_type_get_min_bounds_works() {
    assert_eq!(
        VoxelShapeType::get_min_bounds(VoxelShapeType::Box),
        VoxelBoxShape::DEFAULT_MIN_BOUNDS
    );
    assert_eq!(
        VoxelShapeType::get_min_bounds(VoxelShapeType::Ellipsoid),
        VoxelEllipsoidShape::DEFAULT_MIN_BOUNDS
    );
    assert_eq!(
        VoxelShapeType::get_min_bounds(VoxelShapeType::Cylinder),
        VoxelCylinderShape::DEFAULT_MIN_BOUNDS
    );

    assert_eq!(
        VoxelShapeType::get_min_bounds_from_str(VoxelShapeType::BOX).unwrap(),
        VoxelBoxShape::DEFAULT_MIN_BOUNDS
    );
    assert_eq!(
        VoxelShapeType::get_min_bounds_from_str(VoxelShapeType::ELLIPSOID).unwrap(),
        VoxelEllipsoidShape::DEFAULT_MIN_BOUNDS
    );
    assert_eq!(
        VoxelShapeType::get_min_bounds_from_str(VoxelShapeType::CYLINDER).unwrap(),
        VoxelCylinderShape::DEFAULT_MIN_BOUNDS
    );
}

#[test]
fn test_voxel_shape_type_get_min_bounds_throws_for_invalid_type() {
    assert!(VoxelShapeType::get_min_bounds_from_str("NOT_A_SHAPE_TYPE").is_err());
}

#[test]
fn test_voxel_shape_type_get_max_bounds_works() {
    assert_eq!(
        VoxelShapeType::get_max_bounds(VoxelShapeType::Box),
        VoxelBoxShape::DEFAULT_MAX_BOUNDS
    );
    assert_eq!(
        VoxelShapeType::get_max_bounds(VoxelShapeType::Ellipsoid),
        VoxelEllipsoidShape::DEFAULT_MAX_BOUNDS
    );
    assert_eq!(
        VoxelShapeType::get_max_bounds(VoxelShapeType::Cylinder),
        VoxelCylinderShape::DEFAULT_MAX_BOUNDS
    );

    assert_eq!(
        VoxelShapeType::get_max_bounds_from_str(VoxelShapeType::BOX).unwrap(),
        VoxelBoxShape::DEFAULT_MAX_BOUNDS
    );
    assert_eq!(
        VoxelShapeType::get_max_bounds_from_str(VoxelShapeType::ELLIPSOID).unwrap(),
        VoxelEllipsoidShape::DEFAULT_MAX_BOUNDS
    );
    assert_eq!(
        VoxelShapeType::get_max_bounds_from_str(VoxelShapeType::CYLINDER).unwrap(),
        VoxelCylinderShape::DEFAULT_MAX_BOUNDS
    );
}

#[test]
fn test_voxel_shape_type_get_max_bounds_throws_for_invalid_type() {
    assert!(VoxelShapeType::get_max_bounds_from_str("NOT_A_SHAPE_TYPE").is_err());
}

#[test]
fn test_voxel_shape_type_create_shape_works() {
    let box_shape = VoxelShapeType::create_shape(VoxelShapeType::Box);
    let ellipsoid_shape = VoxelShapeType::create_shape(VoxelShapeType::Ellipsoid);
    let cylinder_shape = VoxelShapeType::create_shape(VoxelShapeType::Cylinder);

    let from_box_str = VoxelShapeType::create_shape_from_str(VoxelShapeType::BOX).unwrap();
    let from_ellipsoid_str =
        VoxelShapeType::create_shape_from_str(VoxelShapeType::ELLIPSOID).unwrap();
    let from_cylinder_str =
        VoxelShapeType::create_shape_from_str(VoxelShapeType::CYLINDER).unwrap();

    assert_eq!(box_shape.shape_type(), from_box_str.shape_type());
    assert_eq!(
        ellipsoid_shape.shape_type(),
        from_ellipsoid_str.shape_type()
    );
    assert_eq!(
        cylinder_shape.shape_type(),
        from_cylinder_str.shape_type()
    );
}

#[test]
fn test_voxel_shape_type_create_shape_throws_for_invalid_type() {
    assert!(VoxelShapeType::create_shape_from_str("NOT_A_SHAPE_TYPE").is_err());
}

#[test]
fn test_voxel_shape_type_from_str_and_display() {
    assert_eq!(
        VoxelShapeType::from_str("BOX").unwrap(),
        VoxelShapeType::Box
    );
    assert_eq!(
        VoxelShapeType::from_str("ELLIPSOID").unwrap(),
        VoxelShapeType::Ellipsoid
    );
    assert_eq!(
        VoxelShapeType::from_str("CYLINDER").unwrap(),
        VoxelShapeType::Cylinder
    );
    assert_eq!(VoxelShapeType::Box.to_string(), "BOX");
    assert_eq!(VoxelShapeType::Ellipsoid.to_string(), "ELLIPSOID");
    assert_eq!(VoxelShapeType::Cylinder.to_string(), "CYLINDER");
}
