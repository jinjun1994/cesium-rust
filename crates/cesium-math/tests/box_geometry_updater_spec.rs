use std::cell::RefCell;
use std::rc::Rc;

use cesium_math::{
    BoxGeometryUpdater, BoxGraphics, Cartesian3, Color, ConstantPositionProperty, ConstantProperty,
    DynProperty, Entity, Event, GeometryOffsetAttribute, HeightReference, JulianDate, TimeInterval,
    TimeIntervalCollection,
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

fn create_basic_box() -> Entity {
    let mut entity = Entity::new();
    let mut box_graphics = BoxGraphics::new();
    box_graphics.dimensions = Some(Rc::new(ConstantProperty::from_value(Cartesian3::new(
        1.0, 2.0, 3.0,
    ))));
    let pos = Cartesian3::from_degrees(0.0, 0.0, Some(0.0), None).ok();
    entity.position = Some(ConstantPositionProperty::new(pos, None));
    entity.box_graphics = Some(box_graphics);
    entity
}

fn create_dynamic_box() -> Entity {
    let mut entity = create_basic_box();
    if let Some(box_graphics) = &mut entity.box_graphics {
        box_graphics.dimensions = Some(Rc::new(DynamicTestProperty::new(Some(Cartesian3::new(
            1.0, 2.0, 3.0,
        )))));
    }
    entity
}

#[test]
fn test_time_varying_dimensions_causes_geometry_to_be_dynamic() {
    let mut entity = create_basic_box();
    let mut updater = BoxGeometryUpdater::new(entity.clone(), None).unwrap();
    assert!(!updater.is_dynamic());

    if let Some(box_graphics) = &mut entity.box_graphics {
        box_graphics.dimensions = Some(Rc::new(DynamicTestProperty::new(Some(Cartesian3::new(
            1.0, 2.0, 3.0,
        )))));
    }
    updater.on_entity_property_changed(&entity, "box");

    assert!(updater.is_dynamic());
}

#[test]
fn test_creates_geometry_with_expected_properties() {
    let mut entity = create_basic_box();
    let dimensions = Cartesian3::new(1.0, 2.0, 3.0);
    let time = JulianDate::now();

    if let Some(box_graphics) = &mut entity.box_graphics {
        box_graphics.outline = Some(Rc::new(ConstantProperty::from_value(true)));
        box_graphics.dimensions = Some(Rc::new(ConstantProperty::from_value(dimensions)));
    }

    let updater = BoxGeometryUpdater::new(entity, None).unwrap();

    let fill_instance = updater.create_fill_geometry_instance(time).unwrap();
    assert_eq!(fill_instance.geometry.maximum, dimensions * 0.5);
    assert_eq!(fill_instance.geometry.offset_attribute, None);

    let outline_instance = updater.create_outline_geometry_instance(time).unwrap();
    assert_eq!(outline_instance.geometry.max(), dimensions * 0.5);
    assert_eq!(outline_instance.geometry.offset_attribute, None);
}

#[test]
fn test_creates_geometry_with_expected_offset_attribute() {
    let mut entity = create_basic_box();
    let time = JulianDate::now();

    if let Some(box_graphics) = &mut entity.box_graphics {
        box_graphics.outline = Some(Rc::new(ConstantProperty::from_value(true)));
        box_graphics.outline_color = Some(Rc::new(ConstantProperty::from_value(Color::BLACK)));
    }

    let mut updater = BoxGeometryUpdater::new(entity.clone(), None).unwrap();

    updater.on_entity_property_changed(&entity, "box");
    let fill = updater.create_fill_geometry_instance(time).unwrap();
    assert_eq!(fill.geometry.offset_attribute, None);
    let outline = updater.create_outline_geometry_instance(time).unwrap();
    assert_eq!(outline.geometry.offset_attribute, None);

    if let Some(box_graphics) = &mut entity.box_graphics {
        box_graphics.height_reference = Some(Rc::new(ConstantProperty::from_value(
            HeightReference::None,
        )));
    }
    updater.on_entity_property_changed(&entity, "box");
    let fill = updater.create_fill_geometry_instance(time).unwrap();
    assert_eq!(fill.geometry.offset_attribute, None);
    let outline = updater.create_outline_geometry_instance(time).unwrap();
    assert_eq!(outline.geometry.offset_attribute, None);

    if let Some(box_graphics) = &mut entity.box_graphics {
        box_graphics.height_reference = Some(Rc::new(ConstantProperty::from_value(
            HeightReference::ClampToGround,
        )));
    }
    updater.on_entity_property_changed(&entity, "box");
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

    if let Some(box_graphics) = &mut entity.box_graphics {
        box_graphics.height_reference = Some(Rc::new(ConstantProperty::from_value(
            HeightReference::RelativeToGround,
        )));
    }
    updater.on_entity_property_changed(&entity, "box");
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
    let entity = create_dynamic_box();
    let updater = BoxGeometryUpdater::new(entity.clone(), None).unwrap();
    let mut dynamic_updater = updater.create_dynamic_updater(Some(()), Some(())).unwrap();

    let now = JulianDate::now();
    dynamic_updater.update(now);

    let expected_dim = entity
        .box_graphics
        .as_ref()
        .unwrap()
        .dimensions
        .as_ref()
        .unwrap()
        .get_value(Some(&now));
    assert_eq!(dynamic_updater.options().dimensions, expected_dim);
    assert_eq!(dynamic_updater.options().offset_attribute, None);
}

#[test]
fn test_geometry_changed_event_raised_when_expected() {
    let mut entity = create_basic_box();
    let mut updater = BoxGeometryUpdater::new(entity.clone(), None).unwrap();

    let change_count = Rc::new(RefCell::new(0usize));
    let change_count_clone = Rc::clone(&change_count);
    updater.geometry_changed.add_listener(move |()| {
        *change_count_clone.borrow_mut() += 1;
    });

    // 1. Modifying dimensions raises event (count == 1)
    if let Some(box_graphics) = &mut entity.box_graphics {
        box_graphics.dimensions = Some(Rc::new(ConstantProperty::from_value(Cartesian3::new(
            10.0, 20.0, 30.0,
        ))));
    }
    updater.on_entity_property_changed(&entity, "box");
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
    if let Some(box_graphics) = &mut entity.box_graphics {
        box_graphics.dimensions = None;
    }
    updater.on_entity_property_changed(&entity, "box");
    assert_eq!(*change_count.borrow(), 3);

    // 4. Since there's no valid geometry, changing another box property should not raise the event (count == 3)
    if let Some(box_graphics) = &mut entity.box_graphics {
        box_graphics.outline_width = None;
    }
    updater.on_entity_property_changed(&entity, "box");
    assert_eq!(*change_count.borrow(), 3);

    // 5. Modifying an unrelated property should not have any effect (count == 3)
    entity.view_from = Some(ConstantProperty::from_value(Cartesian3::UNIT_X));
    updater.on_entity_property_changed(&entity, "viewFrom");
    assert_eq!(*change_count.borrow(), 3);
}

#[test]
fn test_computes_center() {
    let entity = create_basic_box();
    let updater = BoxGeometryUpdater::new(entity.clone(), None).unwrap();
    let time = JulianDate::now();

    assert_eq!(
        updater.compute_center(time),
        entity.position.as_ref().and_then(|p| p.get_value(Some(&time)))
    );
}
