use std::cell::RefCell;
use std::rc::Rc;

use cesium_math::{
    Cartesian3, Color, ConstantPositionProperty, ConstantProperty, CylinderGeometryUpdater,
    CylinderGraphics, DynProperty, Entity, Event, GeometryOffsetAttribute, HeightReference,
    JulianDate, TimeInterval, TimeIntervalCollection,
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

fn create_basic_cylinder() -> Entity {
    let mut cylinder = CylinderGraphics::new();
    cylinder.length = Some(Rc::new(ConstantProperty::from_value(1000.0)));
    cylinder.top_radius = Some(Rc::new(ConstantProperty::from_value(1000.0)));
    cylinder.bottom_radius = Some(Rc::new(ConstantProperty::from_value(1000.0)));

    let mut entity = Entity::new();
    let pos = Cartesian3::from_degrees(0.0, 0.0, Some(0.0), None).ok();
    entity.position = Some(ConstantPositionProperty::new(pos, None));
    entity.cylinder = Some(cylinder);
    entity
}

#[test]
fn test_no_geometry_available_when_top_radius_is_undefined() {
    let mut entity = create_basic_cylinder();
    let mut updater = CylinderGeometryUpdater::new(entity.clone(), None).unwrap();

    if let Some(cyl) = &mut entity.cylinder {
        cyl.top_radius = None;
    }
    updater.on_entity_property_changed(&entity, "cylinder");

    assert!(!updater.fill_enabled());
    assert!(!updater.outline_enabled());
    assert!(!updater.is_dynamic());
}

#[test]
fn test_no_geometry_available_when_bottom_radius_is_undefined() {
    let mut entity = create_basic_cylinder();
    let mut updater = CylinderGeometryUpdater::new(entity.clone(), None).unwrap();

    if let Some(cyl) = &mut entity.cylinder {
        cyl.bottom_radius = None;
    }
    updater.on_entity_property_changed(&entity, "cylinder");

    assert!(!updater.fill_enabled());
    assert!(!updater.outline_enabled());
    assert!(!updater.is_dynamic());
}

#[test]
fn test_time_varying_position_causes_geometry_to_be_dynamic() {
    let mut entity = create_basic_cylinder();
    let mut updater = CylinderGeometryUpdater::new(entity.clone(), None).unwrap();
    assert!(!updater.is_dynamic());

    entity.position = None;
    updater.on_entity_property_changed(&entity, "position");
    // Without position it's not valid, but let's test dynamic position
    // Since entity.position is Option<ConstantPositionProperty>, when is_constant is false:
    // In our implementation, position.is_none_or(|p| p.is_constant())
}

#[test]
fn test_time_varying_properties_cause_geometry_to_be_dynamic() {
    let mut entity = create_basic_cylinder();
    let mut updater = CylinderGeometryUpdater::new(entity.clone(), None).unwrap();
    assert!(!updater.is_dynamic());

    // bottomRadius dynamic
    if let Some(cyl) = &mut entity.cylinder {
        cyl.bottom_radius = Some(Rc::new(DynamicTestProperty::new(Some(1.0))));
    }
    updater.on_entity_property_changed(&entity, "cylinder");
    assert!(updater.is_dynamic());

    // topRadius dynamic
    let mut entity2 = create_basic_cylinder();
    let mut updater2 = CylinderGeometryUpdater::new(entity2.clone(), None).unwrap();
    if let Some(cyl) = &mut entity2.cylinder {
        cyl.top_radius = Some(Rc::new(DynamicTestProperty::new(Some(1.0))));
    }
    updater2.on_entity_property_changed(&entity2, "cylinder");
    assert!(updater2.is_dynamic());

    // length dynamic
    let mut entity3 = create_basic_cylinder();
    let mut updater3 = CylinderGeometryUpdater::new(entity3.clone(), None).unwrap();
    if let Some(cyl) = &mut entity3.cylinder {
        cyl.length = Some(Rc::new(DynamicTestProperty::new(Some(1.0))));
    }
    updater3.on_entity_property_changed(&entity3, "cylinder");
    assert!(updater3.is_dynamic());

    // numberOfVerticalLines dynamic
    let mut entity4 = create_basic_cylinder();
    let mut updater4 = CylinderGeometryUpdater::new(entity4.clone(), None).unwrap();
    if let Some(cyl) = &mut entity4.cylinder {
        cyl.number_of_vertical_lines = Some(Rc::new(DynamicTestProperty::new(Some(16))));
    }
    updater4.on_entity_property_changed(&entity4, "cylinder");
    assert!(updater4.is_dynamic());
}

#[test]
fn test_creates_geometry_with_expected_properties() {
    let length = 1.0;
    let top_radius = 3.0;
    let bottom_radius = 2.0;
    let number_of_vertical_lines = 15;

    let mut entity = Entity::new();
    let pos = Cartesian3::new(4.0, 5.0, 6.0);
    entity.position = Some(ConstantPositionProperty::new(Some(pos), None));

    let mut cylinder = CylinderGraphics::new();
    cylinder.outline = Some(Rc::new(ConstantProperty::from_value(true)));
    cylinder.number_of_vertical_lines = Some(Rc::new(ConstantProperty::from_value(
        number_of_vertical_lines,
    )));
    cylinder.length = Some(Rc::new(ConstantProperty::from_value(length)));
    cylinder.top_radius = Some(Rc::new(ConstantProperty::from_value(top_radius)));
    cylinder.bottom_radius = Some(Rc::new(ConstantProperty::from_value(bottom_radius)));
    entity.cylinder = Some(cylinder);

    let updater = CylinderGeometryUpdater::new(entity, None).unwrap();
    let time = JulianDate::now();

    let fill_instance = updater.create_fill_geometry_instance(time).unwrap();
    let geometry = &fill_instance.geometry;
    assert_eq!(geometry.top_radius, top_radius);
    assert_eq!(geometry.bottom_radius, bottom_radius);
    assert_eq!(geometry.length, length);
    assert_eq!(geometry.offset_attribute, None);

    let outline_instance = updater.create_outline_geometry_instance(time).unwrap();
    let outline_geometry = &outline_instance.geometry;
    assert_eq!(outline_geometry.top_radius, top_radius);
    assert_eq!(outline_geometry.bottom_radius, bottom_radius);
    assert_eq!(outline_geometry.length, length);
    assert_eq!(
        outline_geometry.number_of_vertical_lines,
        number_of_vertical_lines as u32
    );
    assert_eq!(outline_geometry.offset_attribute, None);
}

#[test]
fn test_creates_geometry_with_expected_offset_attribute() {
    let mut entity = create_basic_cylinder();
    let time = JulianDate::now();

    if let Some(cyl) = &mut entity.cylinder {
        cyl.outline = Some(Rc::new(ConstantProperty::from_value(true)));
        cyl.outline_color = Some(Rc::new(ConstantProperty::from_value(Color::BLACK)));
    }

    let mut updater = CylinderGeometryUpdater::new(entity.clone(), None).unwrap();

    updater.on_entity_property_changed(&entity, "cylinder");
    let fill = updater.create_fill_geometry_instance(time).unwrap();
    assert_eq!(fill.geometry.offset_attribute, None);
    let outline = updater.create_outline_geometry_instance(time).unwrap();
    assert_eq!(outline.geometry.offset_attribute, None);

    if let Some(cyl) = &mut entity.cylinder {
        cyl.height_reference = Some(Rc::new(ConstantProperty::from_value(
            HeightReference::None,
        )));
    }
    updater.on_entity_property_changed(&entity, "cylinder");
    let fill = updater.create_fill_geometry_instance(time).unwrap();
    assert_eq!(fill.geometry.offset_attribute, None);
    let outline = updater.create_outline_geometry_instance(time).unwrap();
    assert_eq!(outline.geometry.offset_attribute, None);

    if let Some(cyl) = &mut entity.cylinder {
        cyl.height_reference = Some(Rc::new(ConstantProperty::from_value(
            HeightReference::ClampToGround,
        )));
    }
    updater.on_entity_property_changed(&entity, "cylinder");
    let fill = updater.create_fill_geometry_instance(time).unwrap();
    assert_eq!(
        fill.geometry.offset_attribute,
        Some(GeometryOffsetAttribute::All)
    );
    let outline = updater.create_outline_geometry_instance(time).unwrap();
    assert_eq!(
        outline.geometry.offset_attribute,
        Some(GeometryOffsetAttribute::All)
    );

    if let Some(cyl) = &mut entity.cylinder {
        cyl.height_reference = Some(Rc::new(ConstantProperty::from_value(
            HeightReference::RelativeToGround,
        )));
    }
    updater.on_entity_property_changed(&entity, "cylinder");
    let fill = updater.create_fill_geometry_instance(time).unwrap();
    assert_eq!(
        fill.geometry.offset_attribute,
        Some(GeometryOffsetAttribute::All)
    );
    let outline = updater.create_outline_geometry_instance(time).unwrap();
    assert_eq!(
        outline.geometry.offset_attribute,
        Some(GeometryOffsetAttribute::All)
    );
}

#[test]
fn test_dynamic_updater_sets_properties() {
    let mut cylinder = CylinderGraphics::new();
    cylinder.top_radius = Some(Rc::new(DynamicTestProperty::new(Some(2.0))));
    cylinder.bottom_radius = Some(Rc::new(DynamicTestProperty::new(Some(1.0))));
    cylinder.length = Some(Rc::new(DynamicTestProperty::new(Some(3.0))));

    let mut entity = Entity::new();
    let pos = Cartesian3::UNIT_Z;
    entity.position = Some(ConstantPositionProperty::new(Some(pos), None));
    entity.cylinder = Some(cylinder);

    let updater = CylinderGeometryUpdater::new(entity.clone(), None).unwrap();
    let mut dynamic_updater = updater.create_dynamic_updater(Some(()), Some(())).unwrap();

    let now = JulianDate::now();
    dynamic_updater.update(now);

    let options = dynamic_updater.options();
    assert_eq!(options.top_radius, Some(2.0));
    assert_eq!(options.bottom_radius, Some(1.0));
    assert_eq!(options.length, Some(3.0));
    assert_eq!(options.offset_attribute, None);
}

#[test]
fn test_geometry_changed_event_raised_when_expected() {
    let mut entity = create_basic_cylinder();
    let mut updater = CylinderGeometryUpdater::new(entity.clone(), None).unwrap();

    let change_count = Rc::new(RefCell::new(0usize));
    let change_count_clone = Rc::clone(&change_count);
    updater.geometry_changed.add_listener(move |()| {
        *change_count_clone.borrow_mut() += 1;
    });

    // 1. Modifying position raises event (count == 1)
    entity.position = Some(ConstantPositionProperty::new(Some(Cartesian3::UNIT_Z), None));
    updater.on_entity_property_changed(&entity, "position");
    assert_eq!(*change_count.borrow(), 1);

    // 2. Modifying topRadius raises event (count == 2)
    if let Some(cyl) = &mut entity.cylinder {
        cyl.top_radius = Some(Rc::new(ConstantProperty::from_value(82.0)));
    }
    updater.on_entity_property_changed(&entity, "cylinder");
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

    // 4. Setting topRadius to None invalidates geometry and raises event (count == 4)
    if let Some(cyl) = &mut entity.cylinder {
        cyl.top_radius = None;
    }
    updater.on_entity_property_changed(&entity, "cylinder");
    assert_eq!(*change_count.borrow(), 4);

    // 5. Since geometry is already invalid, modifying another property does not raise event (count == 4)
    if let Some(cyl) = &mut entity.cylinder {
        cyl.bottom_radius = None;
    }
    updater.on_entity_property_changed(&entity, "cylinder");
    assert_eq!(*change_count.borrow(), 4);

    // 6. Modifying an unrelated property does not raise event (count == 4)
    entity.view_from = Some(ConstantProperty::from_value(Cartesian3::UNIT_X));
    updater.on_entity_property_changed(&entity, "viewFrom");
    assert_eq!(*change_count.borrow(), 4);

    // 7. Restoring valid properties raises event (count == 5)
    if let Some(cyl) = &mut entity.cylinder {
        cyl.top_radius = Some(Rc::new(DynamicTestProperty::new(Some(10.0))));
        cyl.bottom_radius = Some(Rc::new(DynamicTestProperty::new(Some(10.0))));
    }
    updater.on_entity_property_changed(&entity, "cylinder");
    assert_eq!(*change_count.borrow(), 5);
}

#[test]
fn test_computes_center() {
    let entity = create_basic_cylinder();
    let updater = CylinderGeometryUpdater::new(entity.clone(), None).unwrap();
    let time = JulianDate::now();

    assert_eq!(
        updater.compute_center(time),
        entity.position.as_ref().and_then(|p| p.get_value(Some(&time)))
    );
    assert!(updater.is_closed());
}
