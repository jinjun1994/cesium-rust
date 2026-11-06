//! Direct port and verification against CesiumJS `DataSources/GroundGeometryUpdaterSpec.js` (Apache-2.0).

use cesium_math::{
    GeometryOffsetAttribute, GroundGeometryUpdater, HeightReference, CLAMP_TO_GROUND,
};

#[test]
fn test_get_geometry_height_works_for_none_and_relative() {
    let expected = 30.0;
    let height = expected;
    let mut height_reference = Some(HeightReference::None);
    assert_eq!(
        GroundGeometryUpdater::get_geometry_height(height, height_reference),
        expected
    );

    height_reference = Some(HeightReference::RelativeToGround);
    assert_eq!(
        GroundGeometryUpdater::get_geometry_height(height, height_reference),
        expected
    );
}

#[test]
fn test_get_geometry_height_works_for_clamp_to_ground() {
    let height = 50.0;
    let height_reference = Some(HeightReference::ClampToGround);
    assert_eq!(
        GroundGeometryUpdater::get_geometry_height(height, height_reference),
        0.0
    );
}

#[test]
fn test_get_geometry_extruded_height_works_for_none_and_relative() {
    let expected = 30.0;
    let height = expected;
    let mut height_reference = Some(HeightReference::None);
    assert_eq!(
        GroundGeometryUpdater::get_geometry_extruded_height(height, height_reference),
        expected
    );

    height_reference = Some(HeightReference::RelativeToGround);
    assert_eq!(
        GroundGeometryUpdater::get_geometry_extruded_height(height, height_reference),
        expected
    );
}

#[test]
fn test_get_geometry_extruded_height_works_for_clamp_to_ground() {
    let height = 50.0;
    let height_reference = Some(HeightReference::ClampToGround);
    assert_eq!(
        GroundGeometryUpdater::get_geometry_extruded_height(height, height_reference),
        CLAMP_TO_GROUND
    );
}

#[test]
fn test_compute_geometry_offset_attribute_works() {
    let height = Some(50.0);
    let extruded_height = Some(30.0);
    let mut height_reference = None;
    let mut extruded_height_reference = None;

    let mut result = GroundGeometryUpdater::compute_geometry_offset_attribute(
        height,
        height_reference,
        extruded_height,
        extruded_height_reference,
    );
    assert!(result.is_none());

    height_reference = Some(HeightReference::None);
    extruded_height_reference = Some(HeightReference::None);
    result = GroundGeometryUpdater::compute_geometry_offset_attribute(
        height,
        height_reference,
        extruded_height,
        extruded_height_reference,
    );
    assert!(result.is_none());

    height_reference = Some(HeightReference::None);
    extruded_height_reference = Some(HeightReference::ClampToGround);
    result = GroundGeometryUpdater::compute_geometry_offset_attribute(
        height,
        height_reference,
        extruded_height,
        extruded_height_reference,
    );
    assert!(result.is_none());

    height_reference = Some(HeightReference::None);
    extruded_height_reference = Some(HeightReference::RelativeToGround);
    result = GroundGeometryUpdater::compute_geometry_offset_attribute(
        height,
        height_reference,
        extruded_height,
        extruded_height_reference,
    );
    assert_eq!(result, Some(GeometryOffsetAttribute::Top));

    height_reference = Some(HeightReference::ClampToGround);
    extruded_height_reference = Some(HeightReference::None);
    result = GroundGeometryUpdater::compute_geometry_offset_attribute(
        height,
        height_reference,
        extruded_height,
        extruded_height_reference,
    );
    assert_eq!(result, Some(GeometryOffsetAttribute::Top));

    height_reference = Some(HeightReference::ClampToGround);
    extruded_height_reference = Some(HeightReference::ClampToGround);
    result = GroundGeometryUpdater::compute_geometry_offset_attribute(
        height,
        height_reference,
        extruded_height,
        extruded_height_reference,
    );
    assert_eq!(result, Some(GeometryOffsetAttribute::Top));

    height_reference = Some(HeightReference::ClampToGround);
    extruded_height_reference = Some(HeightReference::RelativeToGround);
    result = GroundGeometryUpdater::compute_geometry_offset_attribute(
        height,
        height_reference,
        extruded_height,
        extruded_height_reference,
    );
    assert_eq!(result, Some(GeometryOffsetAttribute::All));

    height_reference = Some(HeightReference::RelativeToGround);
    extruded_height_reference = Some(HeightReference::None);
    result = GroundGeometryUpdater::compute_geometry_offset_attribute(
        height,
        height_reference,
        extruded_height,
        extruded_height_reference,
    );
    assert_eq!(result, Some(GeometryOffsetAttribute::Top));

    height_reference = Some(HeightReference::RelativeToGround);
    extruded_height_reference = Some(HeightReference::ClampToGround);
    result = GroundGeometryUpdater::compute_geometry_offset_attribute(
        height,
        height_reference,
        extruded_height,
        extruded_height_reference,
    );
    assert_eq!(result, Some(GeometryOffsetAttribute::Top));

    height_reference = Some(HeightReference::RelativeToGround);
    extruded_height_reference = Some(HeightReference::RelativeToGround);
    result = GroundGeometryUpdater::compute_geometry_offset_attribute(
        height,
        height_reference,
        extruded_height,
        extruded_height_reference,
    );
    assert_eq!(result, Some(GeometryOffsetAttribute::All));

    result = GroundGeometryUpdater::compute_geometry_offset_attribute(
        None,
        height_reference,
        None,
        extruded_height_reference,
    );
    assert!(result.is_none());
}

#[test]
fn test_destroy_works() {
    let mut updater = GroundGeometryUpdater::new();
    assert!(!updater.is_destroyed());
    updater.destroy();
    assert!(updater.is_destroyed());
}
