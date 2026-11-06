//! Direct port and verification against CesiumJS `DataSources/RectangleGeometryUpdaterSpec.js` (Apache-2.0).

use std::cell::RefCell;
use std::rc::Rc;

use cesium_math::{
    Cartesian3, ConstantProperty, DynProperty, Entity, Event, GeoRectangle,
    JulianDate, RectangleGeometryUpdater, RectangleGraphics,
};

/// Dynamic property simulating time-varying values without is_constant returning true.
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

fn create_basic_rectangle() -> Entity {
    let mut rectangle = RectangleGraphics::new();
    rectangle.coordinates = Some(Rc::new(ConstantProperty::from_value(GeoRectangle::new(
        -1.0, -1.0, 1.0, 1.0,
    ))));
    rectangle.height = Some(Rc::new(ConstantProperty::from_value(0.0)));

    let mut entity = Entity::new();
    entity.rectangle = Some(rectangle);
    entity
}

#[test]
fn test_properly_detects_closed_geometry() {
    let mut entity = create_basic_rectangle();
    let mut updater = RectangleGeometryUpdater::new(entity.clone(), None).unwrap();
    assert!(!updater.is_closed());

    let mut rect = entity.rectangle.clone().unwrap();
    rect.extruded_height = Some(Rc::new(ConstantProperty::from_value(1000.0)));
    entity.rectangle = Some(rect);

    updater.on_entity_property_changed(&entity, "rectangle");
    assert!(updater.is_closed());
}

#[test]
fn test_time_varying_coordinates_and_height_causes_geometry_to_be_dynamic() {
    let entity = create_basic_rectangle();
    let mut updater = RectangleGeometryUpdater::new(entity.clone(), None).unwrap();
    assert!(!updater.is_dynamic());

    let dynamic_coords = DynamicTestProperty::new(Some(GeoRectangle::new(0.0, 0.0, 1.0, 1.0)));
    let mut entity2 = entity.clone();
    let mut rect2 = entity2.rectangle.unwrap();
    rect2.coordinates = Some(Rc::new(dynamic_coords));
    entity2.rectangle = Some(rect2);

    updater.on_entity_property_changed(&entity2, "rectangle");
    assert!(updater.is_dynamic());

    let dynamic_height = DynamicTestProperty::new(Some(10.0));
    let mut entity3 = entity.clone();
    let mut rect3 = entity3.rectangle.unwrap();
    rect3.height = Some(Rc::new(dynamic_height));
    entity3.rectangle = Some(rect3);

    let updater3 = RectangleGeometryUpdater::new(entity3, None).unwrap();
    assert!(updater3.is_dynamic());
}

#[test]
fn test_creates_geometry_with_expected_properties() {
    let time = JulianDate::now();
    let entity = create_basic_rectangle();

    let mut rect = entity.rectangle.clone().unwrap();
    rect.outline = Some(Rc::new(ConstantProperty::from_value(true)));
    rect.rotation = Some(Rc::new(ConstantProperty::from_value(0.5)));
    rect.st_rotation = Some(Rc::new(ConstantProperty::from_value(0.25)));
    rect.height = Some(Rc::new(ConstantProperty::from_value(100.0)));
    rect.extruded_height = Some(Rc::new(ConstantProperty::from_value(200.0)));
    rect.granularity = Some(Rc::new(ConstantProperty::from_value(0.01)));

    let mut entity = entity;
    entity.rectangle = Some(rect);

    let updater = RectangleGeometryUpdater::new(entity, None).unwrap();
    assert!(updater.fill_enabled());
    assert!(updater.outline_enabled());
    assert!(updater.is_closed());

    let fill_instance = updater.create_fill_geometry_instance(time).unwrap();
    assert_eq!(fill_instance.geometry.rotation(), 0.5);
    assert_eq!(fill_instance.geometry.st_rotation(), 0.25);
    assert_eq!(fill_instance.geometry.surface_height(), 200.0);
    assert_eq!(fill_instance.geometry.extruded_height(), 100.0);
    assert_eq!(fill_instance.geometry.granularity(), 0.01);
    assert!(fill_instance.geometry.offset_attribute().is_none());
    assert!(fill_instance.attributes.contains_key("show"));

    let outline_instance = updater.create_outline_geometry_instance(time).unwrap();
    assert_eq!(outline_instance.geometry.rotation(), 0.5);
    assert_eq!(outline_instance.geometry.surface_height(), 200.0);
    assert_eq!(outline_instance.geometry.extruded_height(), 100.0);
    assert_eq!(outline_instance.geometry.granularity(), 0.01);
    assert!(outline_instance.geometry.offset_attribute().is_none());
    assert!(outline_instance.attributes.contains_key("show"));
    assert!(outline_instance.attributes.contains_key("color"));
}

#[test]
fn test_dynamic_updater_sets_properties() {
    let time = JulianDate::now();
    let coords = GeoRectangle::new(0.0, 0.0, 1.0, 1.0);

    let dyn_coords = DynamicTestProperty::new(Some(coords));
    let dyn_height = DynamicTestProperty::new(Some(3.0));
    let dyn_extruded_height = DynamicTestProperty::new(Some(2.0));
    let dyn_granularity = DynamicTestProperty::new(Some(2.0));
    let dyn_st_rotation = DynamicTestProperty::new(Some(1.0));

    let mut rect = RectangleGraphics::new();
    rect.coordinates = Some(Rc::new(dyn_coords));
    rect.show = Some(Rc::new(ConstantProperty::from_value(true)));
    rect.height = Some(Rc::new(dyn_height));
    rect.extruded_height = Some(Rc::new(dyn_extruded_height));
    rect.granularity = Some(Rc::new(dyn_granularity));
    rect.st_rotation = Some(Rc::new(dyn_st_rotation));
    rect.fill = Some(Rc::new(ConstantProperty::from_value(true)));
    rect.outline = Some(Rc::new(ConstantProperty::from_value(true)));

    let mut entity = Entity::new();
    entity.rectangle = Some(rect);

    let updater = RectangleGeometryUpdater::new(entity, None).unwrap();
    assert!(updater.is_dynamic());

    let mut dynamic_updater = updater.create_dynamic_updater().unwrap();
    dynamic_updater.update(time);

    let options = dynamic_updater.options();
    assert_eq!(options.rectangle, Some(coords));
    assert_eq!(options.height, Some(3.0));
    assert_eq!(options.extruded_height, Some(2.0));
    assert_eq!(options.granularity, Some(2.0));
    assert_eq!(options.st_rotation, Some(1.0));
    assert!(options.offset_attribute.is_none());
}

#[test]
fn test_geometry_changed_event_is_raised_when_expected() {
    let mut entity = create_basic_rectangle();
    let mut updater = RectangleGeometryUpdater::new(entity.clone(), None).unwrap();

    let event_count = Rc::new(std::cell::Cell::new(0));
    let count_clone = event_count.clone();
    updater.geometry_changed.add_listener(move |()| {
        count_clone.set(count_clone.get() + 1);
    });

    // 1. Change height
    let mut rect = entity.rectangle.clone().unwrap();
    rect.height = Some(Rc::new(ConstantProperty::from_value(82.0)));
    entity.rectangle = Some(rect.clone());
    updater.on_entity_property_changed(&entity, "rectangle");
    assert_eq!(event_count.get(), 1);

    // 2. Change availability
    updater.on_entity_property_changed(&entity, "availability");
    assert_eq!(event_count.get(), 2);

    // 3. Invalidate geometry by setting coordinates = None
    rect.coordinates = None;
    entity.rectangle = Some(rect.clone());
    updater.on_entity_property_changed(&entity, "rectangle");
    assert_eq!(event_count.get(), 3);

    // 4. Changing property while geometry is invalid should not fire event
    rect.height = None;
    entity.rectangle = Some(rect);
    updater.on_entity_property_changed(&entity, "rectangle");
    assert_eq!(event_count.get(), 3);

    // 5. Modifying an unrelated property should not have any effect
    updater.on_entity_property_changed(&entity, "viewFrom");
    assert_eq!(event_count.get(), 3);
}

#[test]
fn test_computes_center() {
    let time = JulianDate::now();
    let entity = create_basic_rectangle();
    let updater = RectangleGeometryUpdater::new(entity, None).unwrap();

    let center = updater.compute_center(time).unwrap();
    let expected = Cartesian3::from_degrees(0.0, 0.0, Some(0.0), None).unwrap();
    assert!((center.x - expected.x).abs() < 1e-7);
    assert!((center.y - expected.y).abs() < 1e-7);
    assert!((center.z - expected.z).abs() < 1e-7);
}
