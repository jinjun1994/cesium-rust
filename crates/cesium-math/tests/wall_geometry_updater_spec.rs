//! Direct port and verification against CesiumJS `DataSources/WallGeometryUpdaterSpec.js` (Apache-2.0).

use std::cell::RefCell;
use std::rc::Rc;

use cesium_math::{
    Cartesian3, Color, ConstantProperty, DynProperty, Entity, Event, JulianDate,
    WallGeometryUpdater, WallGraphics,
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

fn create_basic_wall() -> Entity {
    let positions = Cartesian3::from_radians_array_heights(
        &[0.0, 0.0, 1.0, 1.0, 0.0, 1.0, 1.0, 1.0, 1.0, 0.0, 1.0, 1.0],
        None,
    )
    .unwrap();

    let mut wall = WallGraphics::new();
    wall.positions = Some(Rc::new(ConstantProperty::from_value(positions)));

    let mut entity = Entity::new();
    entity.wall = Some(wall);
    entity
}

#[test]
fn test_time_varying_positions_causes_geometry_to_be_dynamic() {
    let entity = create_basic_wall();
    let mut updater = WallGeometryUpdater::new(entity.clone(), None).unwrap();
    assert!(!updater.is_dynamic());

    let dynamic_positions = DynamicTestProperty::new(Some(Vec::<Cartesian3>::new()));

    let mut entity2 = entity.clone();
    let mut wall2 = entity2.wall.unwrap();
    wall2.positions = Some(Rc::new(dynamic_positions));
    entity2.wall = Some(wall2);

    updater.on_entity_property_changed(&entity2, "wall");
    assert!(updater.is_dynamic());
}

#[test]
fn test_time_varying_minimum_and_maximum_heights_causes_geometry_to_be_dynamic() {
    let entity = create_basic_wall();
    let mut updater = WallGeometryUpdater::new(entity.clone(), None).unwrap();
    assert!(!updater.is_dynamic());

    let dynamic_min_heights = DynamicTestProperty::new(Some(vec![0.0, 1.0, 2.0, 3.0]));
    let mut entity2 = entity.clone();
    let mut wall2 = entity2.wall.unwrap();
    wall2.minimum_heights = Some(Rc::new(dynamic_min_heights));
    entity2.wall = Some(wall2);

    updater.on_entity_property_changed(&entity2, "wall");
    assert!(updater.is_dynamic());

    let dynamic_max_heights = DynamicTestProperty::new(Some(vec![4.0, 5.0, 6.0, 7.0]));
    let mut entity3 = entity.clone();
    let mut wall3 = entity3.wall.unwrap();
    wall3.maximum_heights = Some(Rc::new(dynamic_max_heights));
    entity3.wall = Some(wall3);

    let updater3 = WallGeometryUpdater::new(entity3.clone(), None).unwrap();
    assert!(updater3.is_dynamic());

    let dynamic_granularity = DynamicTestProperty::new(Some(0.5));
    let mut entity4 = entity.clone();
    let mut wall4 = entity4.wall.unwrap();
    wall4.granularity = Some(Rc::new(dynamic_granularity));
    entity4.wall = Some(wall4);

    let updater4 = WallGeometryUpdater::new(entity4, None).unwrap();
    assert!(updater4.is_dynamic());
}

#[test]
fn test_creates_geometry_with_expected_properties() {
    let time = JulianDate::now();
    let entity = create_basic_wall();

    let min_heights = vec![0.0, 1.0, 2.0, 3.0];
    let max_heights = vec![4.0, 5.0, 6.0, 7.0];
    let granularity = 0.97;

    let mut wall = entity.wall.clone().unwrap();
    wall.outline = Some(Rc::new(ConstantProperty::from_value(true)));
    wall.minimum_heights = Some(Rc::new(ConstantProperty::from_value(min_heights.clone())));
    wall.maximum_heights = Some(Rc::new(ConstantProperty::from_value(max_heights.clone())));
    wall.granularity = Some(Rc::new(ConstantProperty::from_value(granularity)));

    let mut entity = entity;
    entity.wall = Some(wall);

    let updater = WallGeometryUpdater::new(entity, None).unwrap();
    assert!(updater.fill_enabled());
    assert!(updater.outline_enabled());
    assert!(!updater.is_closed());

    let fill_instance = updater.create_fill_geometry_instance(time).unwrap();
    assert_eq!(fill_instance.geometry.granularity, granularity);
    assert_eq!(
        fill_instance.geometry.minimum_heights.as_ref().unwrap(),
        &min_heights
    );
    assert_eq!(
        fill_instance.geometry.maximum_heights.as_ref().unwrap(),
        &max_heights
    );
    assert!(fill_instance.attributes.contains_key("show"));

    let outline_instance = updater.create_outline_geometry_instance(time).unwrap();
    assert_eq!(outline_instance.geometry.granularity, granularity);
    assert_eq!(
        outline_instance.geometry.minimum_heights.as_ref().unwrap(),
        &min_heights
    );
    assert_eq!(
        outline_instance.geometry.maximum_heights.as_ref().unwrap(),
        &max_heights
    );
    assert!(outline_instance.attributes.contains_key("show"));
    assert!(outline_instance.attributes.contains_key("color"));
}

#[test]
fn test_dynamic_updater_sets_properties() {
    let time = JulianDate::now();
    let positions = Cartesian3::from_radians_array(&[0.0, 0.0, 1.0, 0.0, 1.0, 1.0, 0.0, 1.0], None)
        .unwrap();

    let dyn_positions = DynamicTestProperty::new(Some(positions.clone()));
    let dyn_min_heights = DynamicTestProperty::new(Some(vec![1.0, 2.0, 3.0, 4.0]));
    let dyn_max_heights = DynamicTestProperty::new(Some(vec![2.0, 3.0, 4.0, 5.0]));
    let dyn_granularity = DynamicTestProperty::new(Some(1.0));

    let mut wall = WallGraphics::new();
    wall.positions = Some(Rc::new(dyn_positions));
    wall.show = Some(Rc::new(ConstantProperty::from_value(true)));
    wall.minimum_heights = Some(Rc::new(dyn_min_heights));
    wall.maximum_heights = Some(Rc::new(dyn_max_heights));
    wall.granularity = Some(Rc::new(dyn_granularity));
    wall.fill = Some(Rc::new(ConstantProperty::from_value(true)));
    wall.outline = Some(Rc::new(ConstantProperty::from_value(true)));
    wall.outline_color = Some(Rc::new(ConstantProperty::from_value(Color::RED)));

    let mut entity = Entity::new();
    entity.wall = Some(wall);

    let updater = WallGeometryUpdater::new(entity, None).unwrap();
    assert!(updater.is_dynamic());

    let mut dynamic_updater = updater.create_dynamic_updater().unwrap();
    dynamic_updater.update(time);

    let options = dynamic_updater.options();
    assert_eq!(options.positions.as_ref().unwrap(), &positions);
    assert_eq!(
        options.minimum_heights.as_ref().unwrap(),
        &vec![1.0, 2.0, 3.0, 4.0]
    );
    assert_eq!(
        options.maximum_heights.as_ref().unwrap(),
        &vec![2.0, 3.0, 4.0, 5.0]
    );
    assert_eq!(options.granularity, Some(1.0));
}

#[test]
fn test_geometry_changed_event_is_raised_when_expected() {
    let mut entity = create_basic_wall();
    let mut updater = WallGeometryUpdater::new(entity.clone(), None).unwrap();

    let event_count = Rc::new(std::cell::Cell::new(0));
    let count_clone = event_count.clone();
    updater.geometry_changed.add_listener(move |()| {
        count_clone.set(count_clone.get() + 1);
    });

    // 1. Change positions
    let mut wall = entity.wall.clone().unwrap();
    wall.positions = Some(Rc::new(ConstantProperty::from_value(vec![])));
    entity.wall = Some(wall.clone());
    updater.on_entity_property_changed(&entity, "wall");
    assert_eq!(event_count.get(), 1);

    // 2. Change granularity
    wall.granularity = Some(Rc::new(ConstantProperty::from_value(82.0)));
    entity.wall = Some(wall.clone());
    updater.on_entity_property_changed(&entity, "wall");
    assert_eq!(event_count.get(), 2);

    // 3. Change availability
    updater.on_entity_property_changed(&entity, "availability");
    assert_eq!(event_count.get(), 3);

    // 4. Invalidate geometry by setting positions = None
    wall.positions = None;
    entity.wall = Some(wall.clone());
    updater.on_entity_property_changed(&entity, "wall");
    assert_eq!(event_count.get(), 4);

    // 5. Changing property while geometry is invalid should not fire event
    wall.granularity = None;
    entity.wall = Some(wall);
    updater.on_entity_property_changed(&entity, "wall");
    assert_eq!(event_count.get(), 4);

    // 6. Modifying an unrelated property should not have any effect
    updater.on_entity_property_changed(&entity, "viewFrom");
    assert_eq!(event_count.get(), 4);
}

#[test]
fn test_is_closed_always_false() {
    let entity = create_basic_wall();
    let updater = WallGeometryUpdater::new(entity, None).unwrap();
    assert!(!updater.is_closed());
}
