//! Direct port and verification against CesiumJS `DataSources/PolylineVolumeGeometryUpdaterSpec.js` (Apache-2.0).

use std::cell::RefCell;
use std::rc::Rc;

use cesium_math::{
    Cartesian2, Cartesian3, ConstantProperty, CornerType, DynProperty, Entity, Event,
    JulianDate, PolylineVolumeGeometryUpdater, PolylineVolumeGraphics,
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

fn create_sample_shape() -> Vec<Cartesian2> {
    vec![
        Cartesian2::new(-1.0, -1.0),
        Cartesian2::new(1.0, -1.0),
        Cartesian2::new(1.0, 1.0),
        Cartesian2::new(1.0, -1.0),
    ]
}

fn create_basic_polyline_volume() -> Entity {
    let positions = Cartesian3::from_degrees_array(
        &[0.0, 0.0, 1.0, 0.0, 1.0, 1.0, 0.0, 1.0],
        None,
    )
    .unwrap();

    let mut polyline_volume = PolylineVolumeGraphics::new();
    polyline_volume.positions = Some(Rc::new(ConstantProperty::from_value(positions)));
    polyline_volume.shape = Some(Rc::new(ConstantProperty::from_value(create_sample_shape())));

    let mut entity = Entity::new();
    entity.polyline_volume = Some(polyline_volume);
    entity
}

#[test]
fn test_time_varying_positions_causes_geometry_to_be_dynamic() {
    let entity = create_basic_polyline_volume();
    let mut updater = PolylineVolumeGeometryUpdater::new(entity.clone(), None).unwrap();
    assert!(!updater.is_dynamic());

    let dynamic_positions = DynamicTestProperty::new(Some(Vec::<Cartesian3>::new()));

    let mut entity2 = entity.clone();
    let mut vol2 = entity2.polyline_volume.unwrap();
    vol2.positions = Some(Rc::new(dynamic_positions));
    entity2.polyline_volume = Some(vol2);

    updater.on_entity_property_changed(&entity2, "polylineVolume");
    assert!(updater.is_dynamic());
}

#[test]
fn test_time_varying_shape_causes_geometry_to_be_dynamic() {
    let entity = create_basic_polyline_volume();
    let mut updater = PolylineVolumeGeometryUpdater::new(entity.clone(), None).unwrap();
    assert!(!updater.is_dynamic());

    let dynamic_shape = DynamicTestProperty::new(Some(create_sample_shape()));

    let mut entity2 = entity.clone();
    let mut vol2 = entity2.polyline_volume.unwrap();
    vol2.shape = Some(Rc::new(dynamic_shape));
    entity2.polyline_volume = Some(vol2);

    updater.on_entity_property_changed(&entity2, "polylineVolume");
    assert!(updater.is_dynamic());
}

#[test]
fn test_creates_geometry_with_expected_properties() {
    let time = JulianDate::now();
    let shape = create_sample_shape();
    let granularity = 0.97;
    let corner_type = CornerType::Mitered;

    let entity = create_basic_polyline_volume();
    let mut vol = entity.polyline_volume.clone().unwrap();
    vol.outline = Some(Rc::new(ConstantProperty::from_value(true)));
    vol.corner_type = Some(Rc::new(ConstantProperty::from_value(corner_type)));
    vol.shape = Some(Rc::new(ConstantProperty::from_value(shape.clone())));
    vol.granularity = Some(Rc::new(ConstantProperty::from_value(granularity)));

    let mut entity = entity;
    entity.polyline_volume = Some(vol);

    let updater = PolylineVolumeGeometryUpdater::new(entity, None).unwrap();
    assert!(updater.fill_enabled());
    assert!(updater.outline_enabled());
    assert!(!updater.is_closed());

    let fill_instance = updater.create_fill_geometry_instance(time).unwrap();
    assert_eq!(fill_instance.geometry.shape_positions, shape);
    assert_eq!(fill_instance.geometry.granularity, granularity);
    assert!(fill_instance.attributes.contains_key("show"));

    let outline_instance = updater.create_outline_geometry_instance(time).unwrap();
    assert_eq!(outline_instance.geometry.shape_positions, shape);
    assert_eq!(outline_instance.geometry.granularity, granularity);
    assert_eq!(outline_instance.geometry.corner_type, corner_type);
    assert!(outline_instance.attributes.contains_key("show"));
    assert!(outline_instance.attributes.contains_key("color"));
}

#[test]
fn test_dynamic_updater_sets_properties() {
    let time = JulianDate::now();
    let positions = Cartesian3::from_radians_array(&[0.0, 0.0, 1.0, 0.0, 1.0, 1.0, 0.0, 1.0], None)
        .unwrap();
    let shape = create_sample_shape();

    let dyn_positions = DynamicTestProperty::new(Some(positions.clone()));
    let dyn_shape = DynamicTestProperty::new(Some(shape.clone()));
    let dyn_granularity = DynamicTestProperty::new(Some(2.0));
    let dyn_corner_type = DynamicTestProperty::new(Some(CornerType::Mitered));

    let mut vol = PolylineVolumeGraphics::new();
    vol.positions = Some(Rc::new(dyn_positions));
    vol.show = Some(Rc::new(ConstantProperty::from_value(true)));
    vol.shape = Some(Rc::new(dyn_shape));
    vol.outline = Some(Rc::new(ConstantProperty::from_value(true)));
    vol.fill = Some(Rc::new(ConstantProperty::from_value(true)));
    vol.granularity = Some(Rc::new(dyn_granularity));
    vol.corner_type = Some(Rc::new(dyn_corner_type));

    let mut entity = Entity::new();
    entity.polyline_volume = Some(vol);

    let updater = PolylineVolumeGeometryUpdater::new(entity, None).unwrap();
    assert!(updater.is_dynamic());

    let mut dynamic_updater = updater.create_dynamic_updater().unwrap();
    dynamic_updater.update(time);

    let options = dynamic_updater.options();
    assert_eq!(options.polyline_positions.as_ref().unwrap(), &positions);
    assert_eq!(options.shape_positions.as_ref().unwrap(), &shape);
    assert_eq!(options.granularity, Some(2.0));
    assert_eq!(options.corner_type, Some(CornerType::Mitered));
}

#[test]
fn test_geometry_changed_event_is_raised_when_expected() {
    let mut entity = create_basic_polyline_volume();
    let mut updater = PolylineVolumeGeometryUpdater::new(entity.clone(), None).unwrap();

    let event_count = Rc::new(std::cell::Cell::new(0));
    let count_clone = event_count.clone();
    updater.geometry_changed.add_listener(move |()| {
        count_clone.set(count_clone.get() + 1);
    });

    // 1. Change positions
    let mut vol = entity.polyline_volume.clone().unwrap();
    vol.positions = Some(Rc::new(ConstantProperty::from_value(vec![])));
    entity.polyline_volume = Some(vol.clone());
    updater.on_entity_property_changed(&entity, "polylineVolume");
    assert_eq!(event_count.get(), 1);

    // 2. Change shape
    vol.shape = Some(Rc::new(ConstantProperty::from_value(create_sample_shape())));
    entity.polyline_volume = Some(vol.clone());
    updater.on_entity_property_changed(&entity, "polylineVolume");
    assert_eq!(event_count.get(), 2);

    // 3. Change availability
    updater.on_entity_property_changed(&entity, "availability");
    assert_eq!(event_count.get(), 3);

    // 4. Invalidate geometry by setting positions = None
    vol.positions = None;
    entity.polyline_volume = Some(vol.clone());
    updater.on_entity_property_changed(&entity, "polylineVolume");
    assert_eq!(event_count.get(), 4);

    // 5. Changing property while geometry is invalid should not fire event
    vol.shape = None;
    entity.polyline_volume = Some(vol);
    updater.on_entity_property_changed(&entity, "polylineVolume");
    assert_eq!(event_count.get(), 4);

    // 6. Modifying an unrelated property should not have any effect
    updater.on_entity_property_changed(&entity, "viewFrom");
    assert_eq!(event_count.get(), 4);
}
