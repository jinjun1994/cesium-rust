use std::cell::RefCell;
use std::rc::Rc;

use cesium_math::{
    Cartesian2, Cartesian3, ConstantPositionProperty, ConstantProperty, DynProperty, Entity, Event,
    JulianDate, Plane, PlaneGeometryUpdater, PlaneGraphics, TimeInterval, TimeIntervalCollection,
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

fn create_basic_plane() -> Entity {
    let mut plane_graphics = PlaneGraphics::new();
    plane_graphics.plane = Some(Rc::new(ConstantProperty::from_value(
        Plane::new(glam::DVec3::X, 0.0).unwrap(),
    )));
    plane_graphics.dimensions = Some(Rc::new(ConstantProperty::from_value(Cartesian2::new(
        1.0, 2.0,
    ))));

    let mut entity = Entity::new();
    let pos = Cartesian3::from_degrees(0.0, 0.0, Some(0.0), None).ok();
    entity.position = Some(ConstantPositionProperty::new(pos, None));
    entity.plane = Some(plane_graphics);
    entity
}

fn create_dynamic_plane() -> Entity {
    let mut entity = create_basic_plane();
    if let Some(plane_graphics) = &mut entity.plane {
        plane_graphics.plane = Some(Rc::new(DynamicTestProperty::new(Some(
            Plane::new(glam::DVec3::X, 0.0).unwrap(),
        ))));
        plane_graphics.dimensions = Some(Rc::new(DynamicTestProperty::new(Some(Cartesian2::new(
            1.0, 2.0,
        )))));
    }
    entity
}

#[test]
fn test_time_varying_plane_causes_geometry_to_be_dynamic() {
    let mut entity = create_basic_plane();
    let mut updater = PlaneGeometryUpdater::new(entity.clone(), None).unwrap();
    assert!(!updater.is_dynamic());

    if let Some(plane_graphics) = &mut entity.plane {
        plane_graphics.plane = Some(Rc::new(DynamicTestProperty::new(Some(
            Plane::new(glam::DVec3::X, 0.0).unwrap(),
        ))));
    }
    updater.on_entity_property_changed(&entity, "plane");

    assert!(updater.is_dynamic());
}

#[test]
fn test_time_varying_dimensions_causes_geometry_to_be_dynamic() {
    let mut entity = create_basic_plane();
    let mut updater = PlaneGeometryUpdater::new(entity.clone(), None).unwrap();
    assert!(!updater.is_dynamic());

    if let Some(plane_graphics) = &mut entity.plane {
        plane_graphics.dimensions = Some(Rc::new(DynamicTestProperty::new(Some(Cartesian2::new(
            1.0, 2.0,
        )))));
    }
    updater.on_entity_property_changed(&entity, "plane");

    assert!(updater.is_dynamic());
}

#[test]
fn test_dynamic_updater_sets_properties() {
    let entity = create_dynamic_plane();
    let updater = PlaneGeometryUpdater::new(entity.clone(), None).unwrap();
    let mut dynamic_updater = updater.create_dynamic_updater(Some(()), Some(())).unwrap();

    let now = JulianDate::now();
    dynamic_updater.update(now);

    let options = dynamic_updater.options();
    let expected_plane = entity
        .plane
        .as_ref()
        .unwrap()
        .plane
        .as_ref()
        .unwrap()
        .get_value(Some(&now));
    let expected_dim = entity
        .plane
        .as_ref()
        .unwrap()
        .dimensions
        .as_ref()
        .unwrap()
        .get_value(Some(&now));

    assert_eq!(options.plane, expected_plane);
    assert_eq!(options.dimensions, expected_dim);
}

#[test]
fn test_geometry_changed_event_raised_when_expected() {
    let mut entity = create_basic_plane();
    let mut updater = PlaneGeometryUpdater::new(entity.clone(), None).unwrap();

    let change_count = Rc::new(RefCell::new(0usize));
    let change_count_clone = Rc::clone(&change_count);
    updater.geometry_changed.add_listener(move |()| {
        *change_count_clone.borrow_mut() += 1;
    });

    // 1. Modifying dimensions raises event (count == 1)
    if let Some(plane_graphics) = &mut entity.plane {
        plane_graphics.dimensions = Some(Rc::new(ConstantProperty::from_value(Cartesian2::new(
            10.0, 20.0,
        ))));
    }
    updater.on_entity_property_changed(&entity, "plane");
    assert_eq!(*change_count.borrow(), 1);

    // 2. Modifying availability raises event (count == 2)
    let mut intervals = TimeIntervalCollection::new();
    intervals.add_interval(TimeInterval::new(
        JulianDate::now(),
        JulianDate::now().add_seconds(100.0),
        true,
        true,
    ));
    entity.availability = Some(intervals);
    updater.on_entity_property_changed(&entity, "availability");
    assert_eq!(*change_count.borrow(), 2);

    // 3. Removing dimensions raises event (count == 3)
    if let Some(plane_graphics) = &mut entity.plane {
        plane_graphics.dimensions = None;
    }
    updater.on_entity_property_changed(&entity, "plane");
    assert_eq!(*change_count.borrow(), 3);

    // 4. Since there's no valid geometry, changing another property should not raise the event (count == 3)
    if let Some(plane_graphics) = &mut entity.plane {
        plane_graphics.outline_width = None;
    }
    updater.on_entity_property_changed(&entity, "plane");
    assert_eq!(*change_count.borrow(), 3);

    // 5. Modifying an unrelated property should not have any effect (count == 3)
    entity.view_from = Some(ConstantProperty::from_value(Cartesian3::UNIT_X));
    updater.on_entity_property_changed(&entity, "viewFrom");
    assert_eq!(*change_count.borrow(), 3);
}

#[test]
fn test_creates_fill_and_outline_geometry_instances() {
    let mut entity = create_basic_plane();
    let time = JulianDate::now();

    if let Some(plane_graphics) = &mut entity.plane {
        plane_graphics.outline = Some(Rc::new(ConstantProperty::from_value(true)));
    }

    let updater = PlaneGeometryUpdater::new(entity, None).unwrap();
    assert!(!updater.is_closed());

    let fill_instance = updater.create_fill_geometry_instance(time).unwrap();
    assert!(fill_instance.attributes.contains_key("show"));
    assert!(fill_instance.attributes.contains_key("color"));

    let outline_instance = updater.create_outline_geometry_instance(time).unwrap();
    assert!(outline_instance.attributes.contains_key("show"));
    assert!(outline_instance.attributes.contains_key("color"));
}
