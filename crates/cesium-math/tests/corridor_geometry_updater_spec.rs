//! Direct port and verification against CesiumJS `DataSources/CorridorGeometryUpdaterSpec.js` (Apache-2.0).

use std::cell::RefCell;
use std::rc::Rc;

use cesium_math::{
    Cartesian3, Color, ConstantProperty, CornerType, CorridorGeometryUpdater,
    CorridorGraphics, DynProperty, Entity, Event, JulianDate,
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

fn create_basic_corridor() -> Entity {
    let positions = Cartesian3::from_degrees_array(&[0.0, 0.0, 1.0, 0.0, 1.0, 1.0, 0.0, 1.0], None)
        .unwrap();
    let mut corridor = CorridorGraphics::new();
    corridor.positions = Some(Rc::new(ConstantProperty::from_value(positions)));
    corridor.width = Some(Rc::new(ConstantProperty::from_value(1.0)));
    corridor.height = Some(Rc::new(ConstantProperty::from_value(0.0)));

    let mut entity = Entity::new();
    entity.corridor = Some(corridor);
    entity
}

#[test]
fn test_time_varying_positions_causes_geometry_to_be_dynamic() {
    let entity = create_basic_corridor();
    let mut updater = CorridorGeometryUpdater::new(entity.clone(), None).unwrap();
    assert!(!updater.is_dynamic());

    let dynamic_positions = DynamicTestProperty::new(Some(Vec::<Cartesian3>::new()));

    let mut entity2 = entity.clone();
    let mut corridor2 = entity2.corridor.unwrap();
    corridor2.positions = Some(Rc::new(dynamic_positions));
    entity2.corridor = Some(corridor2);

    updater.on_entity_property_changed(&entity2, "corridor");
    assert!(updater.is_dynamic());
}

#[test]
fn test_time_varying_height_and_width_causes_geometry_to_be_dynamic() {
    let entity = create_basic_corridor();
    let mut updater = CorridorGeometryUpdater::new(entity.clone(), None).unwrap();
    assert!(!updater.is_dynamic());

    let dynamic_height = DynamicTestProperty::new(Some(10.0));

    let mut entity2 = entity.clone();
    let mut corridor2 = entity2.corridor.unwrap();
    corridor2.height = Some(Rc::new(dynamic_height));
    entity2.corridor = Some(corridor2);

    updater.on_entity_property_changed(&entity2, "corridor");
    assert!(updater.is_dynamic());

    let dynamic_width = DynamicTestProperty::new(Some(5.0));

    let mut entity3 = entity.clone();
    let mut corridor3 = entity3.corridor.unwrap();
    corridor3.width = Some(Rc::new(dynamic_width));
    entity3.corridor = Some(corridor3);

    let updater3 = CorridorGeometryUpdater::new(entity3, None).unwrap();
    assert!(updater3.is_dynamic());
}

#[test]
fn test_updater_sets_options_and_geometry_instances() {
    let time = JulianDate::now();
    let positions = Cartesian3::from_degrees_array(&[0.0, 0.0, 1.0, 0.0, 1.0, 1.0, 0.0, 1.0], None)
        .unwrap();

    let mut corridor = CorridorGraphics::new();
    corridor.positions = Some(Rc::new(ConstantProperty::from_value(positions.clone())));
    corridor.width = Some(Rc::new(ConstantProperty::from_value(2.0)));
    corridor.height = Some(Rc::new(ConstantProperty::from_value(3.0)));
    corridor.extruded_height = Some(Rc::new(ConstantProperty::from_value(4.0)));
    corridor.granularity = Some(Rc::new(ConstantProperty::from_value(0.1)));
    corridor.corner_type = Some(Rc::new(ConstantProperty::from_value(CornerType::Mitered)));
    corridor.fill = Some(Rc::new(ConstantProperty::from_value(true)));
    corridor.outline = Some(Rc::new(ConstantProperty::from_value(true)));
    corridor.outline_color = Some(Rc::new(ConstantProperty::from_value(Color::RED)));

    let mut entity = Entity::new();
    entity.corridor = Some(corridor);

    let updater = CorridorGeometryUpdater::new(entity, None).unwrap();
    assert!(updater.fill_enabled());
    assert!(updater.outline_enabled());
    assert!(updater.is_closed());

    let options = updater.options();
    assert_eq!(options.positions.as_ref().unwrap(), &positions);
    assert_eq!(options.width, Some(2.0));
    assert_eq!(options.height, Some(3.0));
    assert_eq!(options.extruded_height, Some(4.0));
    assert_eq!(options.granularity, Some(0.1));
    assert_eq!(options.corner_type, Some(CornerType::Mitered));
    assert!(options.offset_attribute.is_none());

    let fill_instance = updater.create_fill_geometry_instance(time).unwrap();
    assert_eq!(fill_instance.geometry.positions, positions);
    assert_eq!(fill_instance.geometry.width, 2.0);
    assert_eq!(fill_instance.geometry.height, 3.0);
    assert_eq!(fill_instance.geometry.extruded_height, Some(4.0));
    assert_eq!(fill_instance.geometry.granularity, 0.1);
    assert_eq!(fill_instance.geometry.corner_type, CornerType::Mitered);
    assert!(fill_instance.attributes.contains_key("show"));

    let outline_instance = updater.create_outline_geometry_instance(time).unwrap();
    assert_eq!(outline_instance.geometry.positions, positions);
    assert_eq!(outline_instance.geometry.width, 2.0);
    assert_eq!(outline_instance.geometry.height, 3.0);
    assert_eq!(outline_instance.geometry.extruded_height, Some(4.0));
    assert_eq!(outline_instance.geometry.granularity, 0.1);
    assert_eq!(outline_instance.geometry.corner_type, CornerType::Mitered);
    assert!(outline_instance.attributes.contains_key("show"));
    assert!(outline_instance.attributes.contains_key("color"));
}

#[test]
fn test_dynamic_updater_sets_properties() {
    let time = JulianDate::now();
    let positions = Cartesian3::from_degrees_array(&[0.0, 0.0, 1.0, 0.0, 1.0, 1.0, 0.0, 1.0], None)
        .unwrap();

    let dyn_positions = DynamicTestProperty::new(Some(positions.clone()));

    let mut corridor = CorridorGraphics::new();
    corridor.positions = Some(Rc::new(dyn_positions));
    corridor.width = Some(Rc::new(ConstantProperty::from_value(6.0)));
    corridor.height = Some(Rc::new(ConstantProperty::from_value(3.0)));
    corridor.extruded_height = Some(Rc::new(ConstantProperty::from_value(2.0)));
    corridor.granularity = Some(Rc::new(ConstantProperty::from_value(2.0)));
    corridor.corner_type = Some(Rc::new(ConstantProperty::from_value(CornerType::Mitered)));

    let mut entity = Entity::new();
    entity.corridor = Some(corridor);

    let updater = CorridorGeometryUpdater::new(entity, None).unwrap();
    assert!(updater.is_dynamic());

    let mut dynamic_updater = updater.create_dynamic_updater().unwrap();
    dynamic_updater.update(time);

    let options = dynamic_updater.options();
    assert_eq!(options.positions.as_ref().unwrap(), &positions);
    assert_eq!(options.height, Some(3.0));
    assert_eq!(options.extruded_height, Some(2.0));
    assert_eq!(options.width, Some(6.0));
    assert_eq!(options.granularity, Some(2.0));
    assert_eq!(options.corner_type, Some(CornerType::Mitered));
    assert!(options.offset_attribute.is_none());
}

#[test]
fn test_geometry_changed_event_is_raised_when_expected() {
    let mut entity = create_basic_corridor();
    let mut updater = CorridorGeometryUpdater::new(entity.clone(), None).unwrap();

    let event_count = Rc::new(std::cell::Cell::new(0));
    let count_clone = event_count.clone();
    updater.geometry_changed.add_listener(move |()| {
        count_clone.set(count_clone.get() + 1);
    });

    // 1. Change positions
    let mut corridor = entity.corridor.clone().unwrap();
    corridor.positions = Some(Rc::new(ConstantProperty::from_value(vec![])));
    entity.corridor = Some(corridor.clone());
    updater.on_entity_property_changed(&entity, "corridor");
    assert_eq!(event_count.get(), 1);

    // 2. Change height
    corridor.height = Some(Rc::new(ConstantProperty::from_value(82.0)));
    entity.corridor = Some(corridor.clone());
    updater.on_entity_property_changed(&entity, "corridor");
    assert_eq!(event_count.get(), 2);

    // 3. Change availability
    updater.on_entity_property_changed(&entity, "availability");
    assert_eq!(event_count.get(), 3);

    // 4. Invalidate geometry by setting positions = None
    corridor.positions = None;
    entity.corridor = Some(corridor.clone());
    updater.on_entity_property_changed(&entity, "corridor");
    assert_eq!(event_count.get(), 4);

    // 5. Changing property while geometry is invalid should not fire event
    corridor.height = None;
    entity.corridor = Some(corridor);
    updater.on_entity_property_changed(&entity, "corridor");
    assert_eq!(event_count.get(), 4);

    // 6. Modifying an unrelated property should not have any effect
    updater.on_entity_property_changed(&entity, "viewFrom");
    assert_eq!(event_count.get(), 4);
}

#[test]
fn test_computes_center() {
    let time = JulianDate::now();
    let entity = create_basic_corridor();
    let updater = CorridorGeometryUpdater::new(entity, None).unwrap();

    let center = updater.compute_center(time).unwrap();
    // [0,0, 1,0, 1,1, 0,1] -> length 4, index 4/2 = 2, which is (1.0, 1.0, 0.0)
    let expected = Cartesian3::from_degrees(1.0, 1.0, Some(0.0), None).unwrap();
    assert!((center.x - expected.x).abs() < 1e-7);
    assert!((center.y - expected.y).abs() < 1e-7);
    assert!((center.z - expected.z).abs() < 1e-7);
}

#[test]
fn test_is_closed_only_when_extruded() {
    let mut entity = create_basic_corridor();
    let updater = CorridorGeometryUpdater::new(entity.clone(), None).unwrap();
    assert!(!updater.is_closed());

    // With extruded_height == height
    let mut corridor = entity.corridor.unwrap();
    corridor.height = Some(Rc::new(ConstantProperty::from_value(10.0)));
    corridor.extruded_height = Some(Rc::new(ConstantProperty::from_value(10.0)));
    entity.corridor = Some(corridor.clone());
    let updater2 = CorridorGeometryUpdater::new(entity.clone(), None).unwrap();
    assert!(!updater2.is_closed());

    // With extruded_height != height
    corridor.extruded_height = Some(Rc::new(ConstantProperty::from_value(20.0)));
    entity.corridor = Some(corridor);
    let updater3 = CorridorGeometryUpdater::new(entity, None).unwrap();
    assert!(updater3.is_closed());
}
