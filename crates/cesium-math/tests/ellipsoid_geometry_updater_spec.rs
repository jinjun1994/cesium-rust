use std::cell::RefCell;
use std::rc::Rc;

use cesium_math::{
    Cartesian3, Color, ConstantPositionProperty, ConstantProperty, DynProperty,
    EllipsoidGeometryUpdater, EllipsoidGraphics, Entity, Event, GeometryOffsetAttribute,
    HeightReference, JulianDate, TimeInterval, TimeIntervalCollection,
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

fn create_basic_ellipsoid() -> Entity {
    let mut ellipsoid = EllipsoidGraphics::new();
    ellipsoid.radii = Some(Rc::new(ConstantProperty::from_value(Cartesian3::new(
        1.0, 2.0, 3.0,
    ))));

    let mut entity = Entity::new();
    let pos = Cartesian3::from_degrees(0.0, 0.0, Some(0.0), None).ok();
    entity.position = Some(ConstantPositionProperty::new(pos, None));
    entity.ellipsoid = Some(ellipsoid);
    entity
}

#[test]
fn test_no_geometry_available_when_radii_is_undefined() {
    let mut entity = create_basic_ellipsoid();
    let mut updater = EllipsoidGeometryUpdater::new(entity.clone(), None).unwrap();

    if let Some(ell) = &mut entity.ellipsoid {
        ell.radii = None;
    }
    updater.on_entity_property_changed(&entity, "ellipsoid");

    assert!(!updater.fill_enabled());
    assert!(!updater.outline_enabled());
    assert!(!updater.is_dynamic());
}

#[test]
fn test_time_varying_properties_cause_geometry_to_be_dynamic() {
    let mut entity = create_basic_ellipsoid();
    let mut updater = EllipsoidGeometryUpdater::new(entity.clone(), None).unwrap();
    assert!(!updater.is_dynamic());

    // radii
    if let Some(ell) = &mut entity.ellipsoid {
        ell.radii = Some(Rc::new(DynamicTestProperty::new(Some(Cartesian3::new(
            1.0, 2.0, 3.0,
        )))));
    }
    updater.on_entity_property_changed(&entity, "ellipsoid");
    assert!(updater.is_dynamic());

    // stackPartitions
    let mut entity2 = create_basic_ellipsoid();
    let mut updater2 = EllipsoidGeometryUpdater::new(entity2.clone(), None).unwrap();
    if let Some(ell) = &mut entity2.ellipsoid {
        ell.stack_partitions = Some(Rc::new(DynamicTestProperty::new(Some(32))));
    }
    updater2.on_entity_property_changed(&entity2, "ellipsoid");
    assert!(updater2.is_dynamic());

    // slicePartitions
    let mut entity3 = create_basic_ellipsoid();
    let mut updater3 = EllipsoidGeometryUpdater::new(entity3.clone(), None).unwrap();
    if let Some(ell) = &mut entity3.ellipsoid {
        ell.slice_partitions = Some(Rc::new(DynamicTestProperty::new(Some(64))));
    }
    updater3.on_entity_property_changed(&entity3, "ellipsoid");
    assert!(updater3.is_dynamic());

    // innerRadii
    let mut entity4 = create_basic_ellipsoid();
    let mut updater4 = EllipsoidGeometryUpdater::new(entity4.clone(), None).unwrap();
    if let Some(ell) = &mut entity4.ellipsoid {
        ell.inner_radii = Some(Rc::new(DynamicTestProperty::new(Some(Cartesian3::new(
            0.5, 1.0, 1.5,
        )))));
    }
    updater4.on_entity_property_changed(&entity4, "ellipsoid");
    assert!(updater4.is_dynamic());
}

#[test]
fn test_creates_geometry_with_expected_properties() {
    let radii = Cartesian3::new(1.0, 2.0, 3.0);
    let inner_radii = Cartesian3::new(0.5, 1.0, 1.5);
    let min_clock = 1.0;
    let max_clock = 2.0;
    let min_cone = 0.5;
    let max_cone = 1.5;
    let stack_partitions = 32;
    let slice_partitions = 64;
    let subdivisions = 15;

    let mut entity = Entity::new();
    let pos = Cartesian3::new(4.0, 5.0, 6.0);
    entity.position = Some(ConstantPositionProperty::new(Some(pos), None));

    let mut ellipsoid = EllipsoidGraphics::new();
    ellipsoid.outline = Some(Rc::new(ConstantProperty::from_value(true)));
    ellipsoid.radii = Some(Rc::new(ConstantProperty::from_value(radii)));
    ellipsoid.stack_partitions = Some(Rc::new(ConstantProperty::from_value(stack_partitions)));
    ellipsoid.slice_partitions = Some(Rc::new(ConstantProperty::from_value(slice_partitions)));
    ellipsoid.inner_radii = Some(Rc::new(ConstantProperty::from_value(inner_radii)));
    ellipsoid.minimum_clock = Some(Rc::new(ConstantProperty::from_value(min_clock)));
    ellipsoid.maximum_clock = Some(Rc::new(ConstantProperty::from_value(max_clock)));
    ellipsoid.minimum_cone = Some(Rc::new(ConstantProperty::from_value(min_cone)));
    ellipsoid.maximum_cone = Some(Rc::new(ConstantProperty::from_value(max_cone)));
    ellipsoid.subdivisions = Some(Rc::new(ConstantProperty::from_value(subdivisions)));
    entity.ellipsoid = Some(ellipsoid);

    let updater = EllipsoidGeometryUpdater::new(entity, None).unwrap();
    let time = JulianDate::now();

    let fill_instance = updater.create_fill_geometry_instance(time).unwrap();
    let geometry = &fill_instance.geometry;
    assert_eq!(geometry.radii, radii);
    assert_eq!(geometry.inner_radii, Some(inner_radii));
    assert_eq!(geometry.minimum_clock, min_clock);
    assert_eq!(geometry.maximum_clock, max_clock);
    assert_eq!(geometry.minimum_cone, min_cone);
    assert_eq!(geometry.maximum_cone, max_cone);
    assert_eq!(geometry.stack_partitions, stack_partitions);
    assert_eq!(geometry.slice_partitions, slice_partitions);
    assert_eq!(geometry.offset_attribute, None);

    let outline_instance = updater.create_outline_geometry_instance(time).unwrap();
    let outline_geometry = &outline_instance.geometry;
    assert_eq!(outline_geometry.radii, radii);
    assert_eq!(outline_geometry.inner_radii, Some(inner_radii));
    assert_eq!(outline_geometry.minimum_clock, min_clock);
    assert_eq!(outline_geometry.maximum_clock, max_clock);
    assert_eq!(outline_geometry.minimum_cone, min_cone);
    assert_eq!(outline_geometry.maximum_cone, max_cone);
    assert_eq!(outline_geometry.stack_partitions, stack_partitions);
    assert_eq!(outline_geometry.slice_partitions, slice_partitions);
    assert_eq!(outline_geometry.subdivisions, subdivisions);
    assert_eq!(outline_geometry.offset_attribute, None);
}

#[test]
fn test_creates_geometry_with_expected_offset_attribute() {
    let mut entity = create_basic_ellipsoid();
    let time = JulianDate::now();

    if let Some(ell) = &mut entity.ellipsoid {
        ell.outline = Some(Rc::new(ConstantProperty::from_value(true)));
        ell.outline_color = Some(Rc::new(ConstantProperty::from_value(Color::BLACK)));
    }

    let mut updater = EllipsoidGeometryUpdater::new(entity.clone(), None).unwrap();

    updater.on_entity_property_changed(&entity, "ellipsoid");
    let fill = updater.create_fill_geometry_instance(time).unwrap();
    assert_eq!(fill.geometry.offset_attribute, None);
    let outline = updater.create_outline_geometry_instance(time).unwrap();
    assert_eq!(outline.geometry.offset_attribute, None);

    if let Some(ell) = &mut entity.ellipsoid {
        ell.height_reference = Some(Rc::new(ConstantProperty::from_value(
            HeightReference::None,
        )));
    }
    updater.on_entity_property_changed(&entity, "ellipsoid");
    let fill = updater.create_fill_geometry_instance(time).unwrap();
    assert_eq!(fill.geometry.offset_attribute, None);
    let outline = updater.create_outline_geometry_instance(time).unwrap();
    assert_eq!(outline.geometry.offset_attribute, None);

    if let Some(ell) = &mut entity.ellipsoid {
        ell.height_reference = Some(Rc::new(ConstantProperty::from_value(
            HeightReference::ClampToGround,
        )));
    }
    updater.on_entity_property_changed(&entity, "ellipsoid");
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

    if let Some(ell) = &mut entity.ellipsoid {
        ell.height_reference = Some(Rc::new(ConstantProperty::from_value(
            HeightReference::RelativeToGround,
        )));
    }
    updater.on_entity_property_changed(&entity, "ellipsoid");
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
    let mut ellipsoid = EllipsoidGraphics::new();
    ellipsoid.radii = Some(Rc::new(DynamicTestProperty::new(Some(Cartesian3::new(
        1.0, 2.0, 3.0,
    )))));
    ellipsoid.inner_radii = Some(Rc::new(DynamicTestProperty::new(Some(Cartesian3::new(
        0.5, 1.0, 1.5,
    )))));

    let mut entity = Entity::new();
    let pos = Cartesian3::UNIT_Z;
    entity.position = Some(ConstantPositionProperty::new(Some(pos), None));
    entity.ellipsoid = Some(ellipsoid);

    let updater = EllipsoidGeometryUpdater::new(entity.clone(), None).unwrap();
    let mut dynamic_updater = updater.create_dynamic_updater(Some(()), Some(())).unwrap();

    let now = JulianDate::now();
    dynamic_updater.update(now);

    let options = dynamic_updater.options();
    assert_eq!(options.radii, Some(Cartesian3::new(1.0, 2.0, 3.0)));
    assert_eq!(options.inner_radii, Some(Cartesian3::new(0.5, 1.0, 1.5)));
    assert_eq!(options.offset_attribute, None);
}

#[test]
fn test_geometry_changed_event_raised_when_expected() {
    let mut entity = create_basic_ellipsoid();
    let mut updater = EllipsoidGeometryUpdater::new(entity.clone(), None).unwrap();

    let change_count = Rc::new(RefCell::new(0usize));
    let change_count_clone = Rc::clone(&change_count);
    updater.geometry_changed.add_listener(move |()| {
        *change_count_clone.borrow_mut() += 1;
    });

    // 1. Modifying position raises event (count == 1)
    entity.position = Some(ConstantPositionProperty::new(Some(Cartesian3::UNIT_Z), None));
    updater.on_entity_property_changed(&entity, "position");
    assert_eq!(*change_count.borrow(), 1);

    // 2. Modifying radii raises event (count == 2)
    if let Some(ell) = &mut entity.ellipsoid {
        ell.radii = Some(Rc::new(ConstantProperty::from_value(Cartesian3::new(
            10.0, 20.0, 30.0,
        ))));
    }
    updater.on_entity_property_changed(&entity, "ellipsoid");
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

    // 4. Setting radii to None invalidates geometry and raises event (count == 4)
    if let Some(ell) = &mut entity.ellipsoid {
        ell.radii = None;
    }
    updater.on_entity_property_changed(&entity, "ellipsoid");
    assert_eq!(*change_count.borrow(), 4);

    // 5. Modifying an unrelated property does not raise event (count == 4)
    entity.view_from = Some(ConstantProperty::from_value(Cartesian3::UNIT_X));
    updater.on_entity_property_changed(&entity, "viewFrom");
    assert_eq!(*change_count.borrow(), 4);

    // 6. Restoring valid properties raises event (count == 5)
    if let Some(ell) = &mut entity.ellipsoid {
        ell.radii = Some(Rc::new(DynamicTestProperty::new(Some(Cartesian3::new(
            1.0, 2.0, 3.0,
        )))));
    }
    updater.on_entity_property_changed(&entity, "ellipsoid");
    assert_eq!(*change_count.borrow(), 5);
}

#[test]
fn test_computes_center() {
    let entity = create_basic_ellipsoid();
    let updater = EllipsoidGeometryUpdater::new(entity.clone(), None).unwrap();
    let time = JulianDate::now();

    assert_eq!(
        updater.compute_center(time),
        entity.position.as_ref().and_then(|p| p.get_value(Some(&time)))
    );
    assert!(updater.is_closed());
}
