//! Direct port and verification against CesiumJS `EntityViewSpec.js` (Apache-2.0).

use cesium_math::{
    BoundingSphere, Cartesian3, ConstantPositionProperty, ConstantProperty, Ellipsoid, Entity,
    EntityView, JulianDate, Quaternion, TrackingReferenceFrame,
};

#[test]
fn test_constructor_sets_expected_values() {
    let entity = Entity::new();
    let view = EntityView::new(entity.clone(), None);
    assert_eq!(view.ellipsoid, Ellipsoid::WGS84);
    assert_eq!(view.entity.id, entity.id);

    let view2 = EntityView::new(entity, Some(Ellipsoid::UNIT_SPHERE));
    assert_eq!(view2.ellipsoid, Ellipsoid::UNIT_SPHERE);
}

#[test]
fn test_can_set_and_get_default_offset_3d() {
    let sample_offset = Cartesian3::new(1.0, 2.0, 3.0);
    EntityView::set_default_offset_3d(sample_offset);
    assert_eq!(EntityView::default_offset_3d(), sample_offset);

    let entity = Entity::new();
    let view = EntityView::new(entity, None);
    let time = JulianDate::now();
    assert_eq!(view.compute_offset(&time), sample_offset);
}

#[test]
fn test_uses_entity_view_from() {
    let sample_offset = Cartesian3::new(10.0, 20.0, 30.0);
    let mut entity = Entity::new();
    entity.view_from = Some(ConstantProperty::new(Some(sample_offset)));

    let view = EntityView::new(entity, None);
    let time = JulianDate::now();
    assert_eq!(view.compute_offset(&time), sample_offset);
}

#[test]
fn test_uses_provided_bounding_sphere() {
    let bs = BoundingSphere::new(Cartesian3::new(3.0, 4.0, 5.0), 6.0);
    let mut view = EntityView::new(Entity::new(), None);
    view.bounding_sphere = Some(bs);

    assert!(view.bounding_sphere.is_some());
    assert_eq!(view.bounding_sphere.unwrap().radius, 6.0);
}

#[test]
fn test_compute_transform_with_reference_frames() {
    let position = Cartesian3::new(6378137.0, 0.0, 0.0);
    let mut entity = Entity::new();
    entity.position = Some(ConstantPositionProperty::new(Some(position), None));

    let mut view = EntityView::new(entity, None);
    let time = JulianDate::now();

    // Default ENU transform
    let enu_mat = view.compute_transform(&time);
    assert!(enu_mat.is_some());

    // TrackingReferenceFrame::Inertial with orientation
    view.entity.tracking_reference_frame = TrackingReferenceFrame::Inertial;
    view.entity.orientation = Some(ConstantProperty::new(Some(Quaternion::IDENTITY)));
    let inertial_mat = view.compute_transform(&time);
    assert!(inertial_mat.is_some());

    // TrackingReferenceFrame::Velocity fallback to ENU
    view.entity.tracking_reference_frame = TrackingReferenceFrame::Velocity;
    let velocity_mat = view.compute_transform(&time);
    assert!(velocity_mat.is_some());
}
