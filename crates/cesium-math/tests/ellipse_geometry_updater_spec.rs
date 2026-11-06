use std::cell::RefCell;
use std::rc::Rc;

use cesium_math::{
    Cartesian3, ConstantPositionProperty, ConstantProperty, DynProperty, EllipseGeometryUpdater,
    EllipseGraphics, Entity, Event, JulianDate, TimeInterval, TimeIntervalCollection,
};

/// Mock dynamic property simulating time-varying values without is_constant returning true.
struct DynamicTestProperty<T: Clone + 'static> {
    value: RefCell<Option<T>>,
    event: Event<()>,
}

impl<T: Clone + 'static> DynamicTestProperty<T> {
    fn new(val: Option<T>) -> Self {
        Self {
            value: RefCell::new(val),
            event: Event::new(),
        }
    }
}

impl<T: Clone + 'static> DynProperty<T> for DynamicTestProperty<T> {
    fn is_constant(&self) -> bool {
        false
    }

    fn definition_changed(&self) -> &Event<()> {
        &self.event
    }

    fn get_value(&self, _time: Option<&JulianDate>) -> Option<T> {
        self.value.borrow().clone()
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn equals_dyn(&self, _other: &dyn DynProperty<T>) -> bool {
        false
    }
}

fn create_basic_ellipse() -> Entity {
    let mut ellipse = EllipseGraphics::new();
    ellipse.semi_major_axis = Some(Rc::new(ConstantProperty::from_value(2.0)));
    ellipse.semi_minor_axis = Some(Rc::new(ConstantProperty::from_value(1.0)));
    ellipse.height = Some(Rc::new(ConstantProperty::from_value(0.0)));

    let mut entity = Entity::new();
    let pos = Cartesian3::from_degrees(0.0, 0.0, Some(0.0), None).ok();
    entity.position = Some(ConstantPositionProperty::new(pos, None));
    entity.ellipse = Some(ellipse);
    entity
}

#[test]
fn test_no_geometry_available_when_semi_major_axis_is_undefined() {
    let mut entity = create_basic_ellipse();
    let mut updater = EllipseGeometryUpdater::new(entity.clone(), None).unwrap();

    if let Some(ellipse) = &mut entity.ellipse {
        ellipse.semi_major_axis = None;
    }
    updater.on_entity_property_changed(&entity, "ellipse");

    assert!(!updater.fill_enabled());
    assert!(!updater.outline_enabled());
    assert!(!updater.is_dynamic());
}

#[test]
fn test_no_geometry_available_when_semi_minor_axis_is_undefined() {
    let mut entity = create_basic_ellipse();
    let mut updater = EllipseGeometryUpdater::new(entity.clone(), None).unwrap();

    if let Some(ellipse) = &mut entity.ellipse {
        ellipse.semi_minor_axis = None;
    }
    updater.on_entity_property_changed(&entity, "ellipse");

    assert!(!updater.fill_enabled());
    assert!(!updater.outline_enabled());
    assert!(!updater.is_dynamic());
}

#[test]
fn test_time_varying_properties_cause_geometry_to_be_dynamic() {
    let mut entity = create_basic_ellipse();
    let mut updater = EllipseGeometryUpdater::new(entity.clone(), None).unwrap();
    assert!(!updater.is_dynamic());

    // semiMinorAxis
    if let Some(el) = &mut entity.ellipse {
        el.semi_minor_axis = Some(Rc::new(DynamicTestProperty::new(Some(1.0))));
    }
    updater.on_entity_property_changed(&entity, "ellipse");
    assert!(updater.is_dynamic());

    // semiMajorAxis
    let mut entity2 = create_basic_ellipse();
    let mut updater2 = EllipseGeometryUpdater::new(entity2.clone(), None).unwrap();
    if let Some(el) = &mut entity2.ellipse {
        el.semi_major_axis = Some(Rc::new(DynamicTestProperty::new(Some(1.0))));
    }
    updater2.on_entity_property_changed(&entity2, "ellipse");
    assert!(updater2.is_dynamic());

    // rotation
    let mut entity3 = create_basic_ellipse();
    let mut updater3 = EllipseGeometryUpdater::new(entity3.clone(), None).unwrap();
    if let Some(el) = &mut entity3.ellipse {
        el.rotation = Some(Rc::new(DynamicTestProperty::new(Some(1.0))));
    }
    updater3.on_entity_property_changed(&entity3, "ellipse");
    assert!(updater3.is_dynamic());

    // height
    let mut entity4 = create_basic_ellipse();
    let mut updater4 = EllipseGeometryUpdater::new(entity4.clone(), None).unwrap();
    if let Some(el) = &mut entity4.ellipse {
        el.height = Some(Rc::new(DynamicTestProperty::new(Some(1.0))));
    }
    updater4.on_entity_property_changed(&entity4, "ellipse");
    assert!(updater4.is_dynamic());

    // extrudedHeight
    let mut entity5 = create_basic_ellipse();
    let mut updater5 = EllipseGeometryUpdater::new(entity5.clone(), None).unwrap();
    if let Some(el) = &mut entity5.ellipse {
        el.extruded_height = Some(Rc::new(DynamicTestProperty::new(Some(1.0))));
    }
    updater5.on_entity_property_changed(&entity5, "ellipse");
    assert!(updater5.is_dynamic());

    // granularity
    let mut entity6 = create_basic_ellipse();
    let mut updater6 = EllipseGeometryUpdater::new(entity6.clone(), None).unwrap();
    if let Some(el) = &mut entity6.ellipse {
        el.granularity = Some(Rc::new(DynamicTestProperty::new(Some(1.0))));
    }
    updater6.on_entity_property_changed(&entity6, "ellipse");
    assert!(updater6.is_dynamic());

    // stRotation
    let mut entity7 = create_basic_ellipse();
    let mut updater7 = EllipseGeometryUpdater::new(entity7.clone(), None).unwrap();
    if let Some(el) = &mut entity7.ellipse {
        el.st_rotation = Some(Rc::new(DynamicTestProperty::new(Some(1.0))));
    }
    updater7.on_entity_property_changed(&entity7, "ellipse");
    assert!(updater7.is_dynamic());

    // numberOfVerticalLines
    let mut entity8 = create_basic_ellipse();
    let mut updater8 = EllipseGeometryUpdater::new(entity8.clone(), None).unwrap();
    if let Some(el) = &mut entity8.ellipse {
        el.number_of_vertical_lines = Some(Rc::new(DynamicTestProperty::new(Some(16))));
    }
    updater8.on_entity_property_changed(&entity8, "ellipse");
    assert!(updater8.is_dynamic());
}

#[test]
fn test_creates_geometry_with_expected_properties() {
    let center = Cartesian3::new(4.0, 5.0, 6.0);
    let rotation = 1.0;
    let semi_major_axis = 3.0;
    let semi_minor_axis = 2.0;
    let height = 431.0;
    let extruded_height = 123.0;
    let granularity = 0.97;
    let st_rotation = 12.0;
    let number_of_vertical_lines = 15;

    let mut entity = Entity::new();
    entity.position = Some(ConstantPositionProperty::new(Some(center), None));

    let mut ellipse = EllipseGraphics::new();
    ellipse.outline = Some(Rc::new(ConstantProperty::from_value(true)));
    ellipse.number_of_vertical_lines = Some(Rc::new(ConstantProperty::from_value(
        number_of_vertical_lines,
    )));
    ellipse.semi_major_axis = Some(Rc::new(ConstantProperty::from_value(semi_major_axis)));
    ellipse.semi_minor_axis = Some(Rc::new(ConstantProperty::from_value(semi_minor_axis)));
    ellipse.rotation = Some(Rc::new(ConstantProperty::from_value(rotation)));
    ellipse.st_rotation = Some(Rc::new(ConstantProperty::from_value(st_rotation)));
    ellipse.height = Some(Rc::new(ConstantProperty::from_value(height)));
    ellipse.extruded_height = Some(Rc::new(ConstantProperty::from_value(extruded_height)));
    ellipse.granularity = Some(Rc::new(ConstantProperty::from_value(granularity)));
    entity.ellipse = Some(ellipse);

    let updater = EllipseGeometryUpdater::new(entity, None).unwrap();
    let time = JulianDate::now();

    let fill_instance = updater.create_fill_geometry_instance(time).unwrap();
    let geometry = &fill_instance.geometry;
    assert_eq!(geometry.center, center);
    assert_eq!(geometry.semi_major_axis, semi_major_axis);
    assert_eq!(geometry.semi_minor_axis, semi_minor_axis);
    assert_eq!(geometry.rotation, rotation);
    assert_eq!(geometry.st_rotation, st_rotation);
    assert_eq!(geometry.height, height);
    assert_eq!(geometry.extruded_height, Some(extruded_height));
    assert_eq!(geometry.granularity, granularity);
    assert_eq!(geometry.offset_attribute, None);

    let outline_instance = updater.create_outline_geometry_instance(time).unwrap();
    let outline_geometry = &outline_instance.geometry;
    assert_eq!(outline_geometry.center, center);
    assert_eq!(outline_geometry.semi_major_axis, semi_major_axis);
    assert_eq!(outline_geometry.semi_minor_axis, semi_minor_axis);
    assert_eq!(outline_geometry.rotation, rotation);
    assert_eq!(outline_geometry.height, height);
    assert_eq!(outline_geometry.extruded_height, Some(extruded_height));
    assert_eq!(outline_geometry.granularity, granularity);
    assert_eq!(
        outline_geometry.number_of_vertical_lines,
        Some(number_of_vertical_lines as usize)
    );
    assert_eq!(outline_geometry.offset_attribute, None);
}

#[test]
fn test_dynamic_updater_sets_properties() {
    let mut ellipse = EllipseGraphics::new();
    ellipse.semi_major_axis = Some(Rc::new(DynamicTestProperty::new(Some(2.0))));
    ellipse.semi_minor_axis = Some(Rc::new(DynamicTestProperty::new(Some(1.0))));
    ellipse.height = Some(Rc::new(DynamicTestProperty::new(Some(1.0))));

    let mut entity = Entity::new();
    let pos = Cartesian3::UNIT_Z;
    entity.position = Some(ConstantPositionProperty::new(Some(pos), None));
    entity.ellipse = Some(ellipse);

    let updater = EllipseGeometryUpdater::new(entity.clone(), None).unwrap();
    let mut dynamic_updater = updater.create_dynamic_updater(Some(()), Some(())).unwrap();

    let now = JulianDate::now();
    dynamic_updater.update(now);

    let options = dynamic_updater.options();
    assert_eq!(options.semi_major_axis, Some(2.0));
    assert_eq!(options.semi_minor_axis, Some(1.0));
    assert_eq!(options.height, Some(1.0));
    assert_eq!(options.offset_attribute, None);
}

#[test]
fn test_geometry_changed_event_raised_when_expected() {
    let mut entity = create_basic_ellipse();
    let mut updater = EllipseGeometryUpdater::new(entity.clone(), None).unwrap();

    let change_count = Rc::new(RefCell::new(0usize));
    let change_count_clone = Rc::clone(&change_count);
    updater.geometry_changed.add_listener(move |()| {
        *change_count_clone.borrow_mut() += 1;
    });

    // 1. Modifying position raises event (count == 1)
    entity.position = Some(ConstantPositionProperty::new(Some(Cartesian3::UNIT_Z), None));
    updater.on_entity_property_changed(&entity, "position");
    assert_eq!(*change_count.borrow(), 1);

    // 2. Modifying semiMajorAxis raises event (count == 2)
    if let Some(el) = &mut entity.ellipse {
        el.semi_major_axis = Some(Rc::new(ConstantProperty::from_value(82.0)));
    }
    updater.on_entity_property_changed(&entity, "ellipse");
    assert_eq!(*change_count.borrow(), 2);

    // 3. Modifying availability raises event (count == 3)
    let mut intervals = TimeIntervalCollection::new();
    intervals.add_interval(TimeInterval::new(
        JulianDate::now(),
        JulianDate::now().add_seconds(100.0),
        true,
        true,
    ));
    entity.availability = Some(intervals);
    updater.on_entity_property_changed(&entity, "availability");
    assert_eq!(*change_count.borrow(), 3);

    // 4. Setting semiMajorAxis to None invalidates geometry and raises event (count == 4)
    if let Some(el) = &mut entity.ellipse {
        el.semi_major_axis = None;
    }
    updater.on_entity_property_changed(&entity, "ellipse");
    assert_eq!(*change_count.borrow(), 4);

    // 5. Since geometry is already invalid, modifying another property does not raise event (count == 4)
    if let Some(el) = &mut entity.ellipse {
        el.semi_minor_axis = None;
    }
    updater.on_entity_property_changed(&entity, "ellipse");
    assert_eq!(*change_count.borrow(), 4);

    // 6. Modifying an unrelated property does not raise event (count == 4)
    entity.view_from = Some(ConstantProperty::from_value(Cartesian3::UNIT_X));
    updater.on_entity_property_changed(&entity, "viewFrom");
    assert_eq!(*change_count.borrow(), 4);

    // 7. Restoring valid properties raises event (count == 5)
    if let Some(el) = &mut entity.ellipse {
        el.semi_major_axis = Some(Rc::new(DynamicTestProperty::new(Some(10.0))));
        el.semi_minor_axis = Some(Rc::new(DynamicTestProperty::new(Some(10.0))));
    }
    updater.on_entity_property_changed(&entity, "ellipse");
    assert_eq!(*change_count.borrow(), 5);
}

#[test]
fn test_computes_center() {
    let entity = create_basic_ellipse();
    let updater = EllipseGeometryUpdater::new(entity.clone(), None).unwrap();
    let time = JulianDate::now();

    assert_eq!(
        updater.compute_center(time),
        entity.position.as_ref().and_then(|p| p.get_value(Some(&time)))
    );
    assert!(!updater.is_closed());
}
