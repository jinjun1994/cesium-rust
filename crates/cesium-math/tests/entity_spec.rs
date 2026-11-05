//! Direct port and verification against CesiumJS `EntitySpec.js` (Apache-2.0).

use std::cell::RefCell;
use std::rc::Rc;

use cesium_math::{
    BillboardGraphics, BoxGraphics, Cartesian3, ConstantPositionProperty, ConstantProperty,
    CorridorGraphics, CylinderGraphics, EllipseGraphics, EllipsoidGraphics, Entity,
    FixedFrameTransforms, JulianDate, LabelGraphics, Matrix3, Matrix4, ModelGraphics, PathGraphics,
    PlaneGraphics, PointGraphics, PolygonGraphics, PolylineGraphics, PolylineVolumeGraphics,
    Quaternion, RectangleGraphics, TimeInterval, TimeIntervalCollection, TrackingReferenceFrame,
    WallGraphics,
};

#[test]
fn test_tracking_reference_frame_values() {
    assert_eq!(TrackingReferenceFrame::Autodetect.value(), 0);
    assert_eq!(TrackingReferenceFrame::Enu.value(), 1);
    assert_eq!(TrackingReferenceFrame::Inertial.value(), 2);
    assert_eq!(TrackingReferenceFrame::Velocity.value(), 3);
}

#[test]
fn test_constructor_sets_expected_properties() {
    let entity = Entity::new();
    assert!(!entity.id.is_empty());
    assert!(entity.name.is_none());
    assert!(entity.billboard.is_none());
    assert!(entity.box_graphics.is_none());
    assert!(entity.corridor.is_none());
    assert!(entity.cylinder.is_none());
    assert!(entity.description.is_none());
    assert!(entity.ellipse.is_none());
    assert!(entity.ellipsoid.is_none());
    assert!(entity.label.is_none());
    assert!(entity.model.is_none());
    assert!(entity.orientation.is_none());
    assert!(entity.path.is_none());
    assert!(entity.plane.is_none());
    assert!(entity.point.is_none());
    assert!(entity.polygon.is_none());
    assert!(entity.polyline.is_none());
    assert!(entity.polyline_volume.is_none());
    assert!(entity.position.is_none());
    assert!(entity.rectangle.is_none());
    assert!(entity.view_from.is_none());
    assert!(entity.wall.is_none());
    assert_eq!(
        entity.tracking_reference_frame,
        TrackingReferenceFrame::Autodetect
    );
    assert!(entity.show);
    assert!(entity.is_showing());

    let mut configured = Entity::with_id("someId");
    configured.name = Some("bob".to_string());
    configured.show = false;
    configured.tracking_reference_frame = TrackingReferenceFrame::Inertial;
    configured.availability = Some(TimeIntervalCollection::new());
    let parent_entity = Rc::new(RefCell::new(Entity::new()));
    configured.parent = Some(parent_entity.clone());

    configured.billboard = Some(BillboardGraphics::default());
    configured.box_graphics = Some(BoxGraphics::default());
    configured.corridor = Some(CorridorGraphics::default());
    configured.cylinder = Some(CylinderGraphics::default());
    configured.description = Some(ConstantProperty::new(Some("description".to_string())));
    configured.ellipse = Some(EllipseGraphics::default());
    configured.ellipsoid = Some(EllipsoidGraphics::default());
    configured.label = Some(LabelGraphics::default());
    configured.model = Some(ModelGraphics::default());
    configured.orientation = Some(ConstantProperty::new(Some(Quaternion::new(1.0, 2.0, 3.0, 4.0))));
    let mut path = PathGraphics::default();
    path.relative_to = Some(Rc::new(ConstantProperty::new(Some("someEntityId".to_string()))));
    configured.path = Some(path);
    configured.plane = Some(PlaneGraphics::default());
    configured.point = Some(PointGraphics::default());
    configured.polygon = Some(PolygonGraphics::default());
    configured.polyline = Some(PolylineGraphics::default());
    configured.polyline_volume = Some(PolylineVolumeGraphics::default());
    configured.position = Some(ConstantPositionProperty::new(
        Some(Cartesian3::new(5.0, 6.0, 7.0)),
        None,
    ));
    configured.rectangle = Some(RectangleGraphics::default());
    configured.view_from = Some(ConstantProperty::new(Some(Cartesian3::new(8.0, 9.0, 10.0))));
    configured.wall = Some(WallGraphics::default());

    assert_eq!(configured.id, "someId");
    assert_eq!(configured.name.as_deref(), Some("bob"));
    assert!(!configured.show);
    assert!(!configured.is_showing());
    assert_eq!(
        configured.tracking_reference_frame,
        TrackingReferenceFrame::Inertial
    );
    assert!(configured.availability.is_some());
    assert!(configured.parent.is_some());

    assert!(configured.billboard.is_some());
    assert!(configured.box_graphics.is_some());
    assert!(configured.corridor.is_some());
    assert!(configured.cylinder.is_some());
    assert!(configured.description.is_some());
    assert!(configured.ellipse.is_some());
    assert!(configured.ellipsoid.is_some());
    assert!(configured.label.is_some());
    assert!(configured.model.is_some());
    assert!(configured.orientation.is_some());
    assert!(configured.path.is_some());
    assert_eq!(
        configured
            .path
            .as_ref()
            .unwrap()
            .relative_to
            .as_ref()
            .unwrap()
            .get_value(Some(&JulianDate::now()))
            .as_deref(),
        Some("someEntityId")
    );
    assert!(configured.plane.is_some());
    assert!(configured.point.is_some());
    assert!(configured.polygon.is_some());
    assert!(configured.polyline.is_some());
    assert!(configured.polyline_volume.is_some());
    assert!(configured.position.is_some());
    assert!(configured.rectangle.is_some());
    assert!(configured.view_from.is_some());
    assert!(configured.wall.is_some());
}

#[test]
fn test_constructor_creates_unique_id() {
    let object = Entity::new();
    let object2 = Entity::new();
    assert!(!object.id.is_empty());
    assert!(!object2.id.is_empty());
    assert_ne!(object.id, object2.id);
}

#[test]
fn test_is_available() {
    let mut entity = Entity::new();
    assert!(entity.is_available(&JulianDate::now()));

    let start = JulianDate::from_iso8601("2000-01-01T00:00:00Z").unwrap();
    let stop = JulianDate::from_iso8601("2001-01-01T00:00:00Z").unwrap();
    let interval = TimeInterval::new(start, stop, true, true);
    let mut intervals = TimeIntervalCollection::new();
    intervals.add_interval(interval);

    entity.availability = Some(intervals);
    assert!(!entity.is_available(&start.add_seconds(-1.0)));
    assert!(entity.is_available(&start));
    assert!(entity.is_available(&stop));
    assert!(!entity.is_available(&stop.add_seconds(1.0)));
}

#[test]
fn test_merge_availability_and_properties() {
    let mut entity = Entity::new();
    let start = JulianDate::from_iso8601("2000-01-01T00:00:00Z").unwrap();
    let stop = JulianDate::from_iso8601("2001-01-01T00:00:00Z").unwrap();
    let interval = TimeInterval::new(start, stop, true, true);
    let mut intervals = TimeIntervalCollection::new();
    intervals.add_interval(interval);
    entity.availability = Some(intervals);

    let mut entity2 = Entity::new();
    let interval2 = TimeInterval::new(start.add_seconds(100.0), stop, true, true);
    let mut intervals2 = TimeIntervalCollection::new();
    intervals2.add_interval(interval2);
    entity2.availability = Some(intervals2);

    entity.merge(&entity2);
    // Availability should not be overwritten
    assert!(entity.is_available(&start));

    // Custom properties merging
    let mut source = Entity::with_id("source");
    source.add_property("customProp").unwrap();
    source
        .properties
        .set_property_value("customProp", Some(Rc::new(ConstantProperty::new(Some(serde_json::json!("fizzbuzz"))))));

    let mut target = Entity::with_id("target");
    assert!(!target.has_property("customProp"));
    target.merge(&source);
    assert!(target.has_property("customProp"));
    let prop = target.properties.get_property("customProp").unwrap();
    assert_eq!(
        prop.get_value(None),
        Some(serde_json::json!("fizzbuzz"))
    );
}

#[test]
fn test_compute_model_matrix() {
    let mut entity = Entity::new();
    let time = JulianDate::now();
    assert!(entity.compute_model_matrix(&time).is_none());

    let position = Cartesian3::new(123456.0, 654321.0, 123456.0);
    entity.position = Some(ConstantPositionProperty::new(Some(position), None));

    // When orientation is None, returns ENU
    let enu_expected = FixedFrameTransforms::east_north_up_to_fixed_frame(position, None).unwrap();
    let model_matrix = entity.compute_model_matrix(&time).unwrap();
    assert_eq!(model_matrix, enu_expected);

    // With result buffer
    let mut result = Matrix4::IDENTITY;
    let res = entity
        .compute_model_matrix_with_result(&time, &mut result)
        .unwrap();
    assert_eq!(res, enu_expected);
    assert_eq!(result, enu_expected);

    // When orientation is defined
    let orientation = Quaternion::new(1.0, 2.0, 3.0, 4.0).normalize();
    entity.orientation = Some(ConstantProperty::new(Some(orientation)));

    let rot = Matrix3::from_quaternion(&orientation);
    let expected = Matrix4::from_rotation_translation(&rot, position);
    let matrix_with_orientation = entity.compute_model_matrix(&time).unwrap();
    assert_eq!(matrix_with_orientation, expected);
}

#[test]
fn test_add_remove_custom_properties() {
    let mut entity = Entity::new();
    assert!(!entity.has_property("bob"));
    assert!(!entity.property_names().contains(&"bob".to_string()));

    entity.add_property("bob").unwrap();
    assert!(entity.has_property("bob"));
    assert!(entity.property_names().contains(&"bob".to_string()));

    entity.remove_property("bob").unwrap();
    assert!(!entity.has_property("bob"));
    assert!(!entity.property_names().contains(&"bob".to_string()));

    // Can re-add
    entity.add_property("bob").unwrap();
    assert!(entity.has_property("bob"));

    // Throw on duplicate
    assert!(entity.add_property("bob").is_err());

    // Throw on non-existent remove
    entity.remove_property("bob").unwrap();
    assert!(entity.remove_property("bob").is_err());

    // Reserved properties rejection
    assert!(entity.add_property("merge").is_err());
    assert!(entity.remove_property("merge").is_err());
    assert!(entity.add_property("name").is_err());
    assert!(entity.remove_property("name").is_err());
}

#[test]
fn test_is_showing_hierarchy() {
    let mut entity = Entity::new();
    assert!(entity.is_showing());

    entity.show = false;
    assert!(!entity.is_showing());

    entity.show = true;
    assert!(entity.is_showing());

    // Parent hierarchy
    let parent = Rc::new(RefCell::new(Entity::new()));
    entity.parent = Some(parent.clone());

    assert!(entity.is_showing());

    parent.borrow_mut().show = false;
    assert!(!entity.is_showing());

    parent.borrow_mut().show = true;
    assert!(entity.is_showing());

    // Grandparent hierarchy
    let grandparent = Rc::new(RefCell::new(Entity::new()));
    parent.borrow_mut().parent = Some(grandparent.clone());

    assert!(entity.is_showing());

    grandparent.borrow_mut().show = false;
    assert!(!entity.is_showing());

    grandparent.borrow_mut().show = true;
    assert!(entity.is_showing());

    // Replacing parent with a hidden parent
    let hidden_parent = Rc::new(RefCell::new(Entity::new()));
    hidden_parent.borrow_mut().show = false;
    entity.parent = Some(hidden_parent);
    assert!(!entity.is_showing());

    // Removing parent restores visibility
    entity.parent = None;
    assert!(entity.is_showing());
}
