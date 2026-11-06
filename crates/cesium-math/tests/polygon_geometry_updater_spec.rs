//! Direct port and verification against CesiumJS `DataSources/PolygonGeometryUpdaterSpec.js` (Apache-2.0).

use std::cell::RefCell;
use std::rc::Rc;

use cesium_math::{
    ArcType, Cartesian3, Color, ConstantProperty, DynProperty, Ellipsoid, Entity,
    Event, JulianDate, PolygonGeometryUpdater, PolygonGraphics, PolygonHierarchy,
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

fn create_basic_polygon() -> Entity {
    let positions = Cartesian3::from_radians_array(
        &[-1.0, -1.0, 1.0, -1.0, 1.0, 1.0, -1.0, 1.0],
        None,
    )
    .unwrap();

    let mut polygon = PolygonGraphics::new();
    polygon.hierarchy = Some(Rc::new(ConstantProperty::from_value(
        PolygonHierarchy::new(positions),
    )));
    polygon.height = Some(Rc::new(ConstantProperty::from_value(0.0)));

    let mut entity = Entity::new();
    entity.polygon = Some(polygon);
    entity
}

#[test]
fn test_properly_detects_closed_geometry() {
    let mut entity = create_basic_polygon();
    let mut updater = PolygonGeometryUpdater::new(entity.clone(), None).unwrap();
    assert!(!updater.is_closed());

    let mut poly = entity.polygon.clone().unwrap();
    poly.extruded_height = Some(Rc::new(ConstantProperty::from_value(1000.0)));
    entity.polygon = Some(poly.clone());

    updater.on_entity_property_changed(&entity, "polygon");
    assert!(updater.is_closed());

    // With close_top = false
    poly.close_top = Some(Rc::new(ConstantProperty::from_value(false)));
    entity.polygon = Some(poly);
    updater.on_entity_property_changed(&entity, "polygon");
    assert!(!updater.is_closed());
}

#[test]
fn test_time_varying_hierarchy_causes_geometry_to_be_dynamic() {
    let entity = create_basic_polygon();
    let mut updater = PolygonGeometryUpdater::new(entity.clone(), None).unwrap();
    assert!(!updater.is_dynamic());

    let dynamic_hierarchy = DynamicTestProperty::new(Some(PolygonHierarchy::default()));

    let mut entity2 = entity.clone();
    let mut poly2 = entity2.polygon.unwrap();
    poly2.hierarchy = Some(Rc::new(dynamic_hierarchy));
    entity2.polygon = Some(poly2);

    updater.on_entity_property_changed(&entity2, "polygon");
    assert!(updater.is_dynamic());
}

#[test]
fn test_time_varying_height_causes_geometry_to_be_dynamic() {
    let entity = create_basic_polygon();
    let mut updater = PolygonGeometryUpdater::new(entity.clone(), None).unwrap();
    assert!(!updater.is_dynamic());

    let dynamic_height = DynamicTestProperty::new(Some(10.0));
    let mut entity2 = entity.clone();
    let mut poly2 = entity2.polygon.unwrap();
    poly2.height = Some(Rc::new(dynamic_height));
    entity2.polygon = Some(poly2);

    updater.on_entity_property_changed(&entity2, "polygon");
    assert!(updater.is_dynamic());
}

#[test]
fn test_creates_geometry_with_expected_properties() {
    let time = JulianDate::now();
    let entity = create_basic_polygon();

    let mut poly = entity.polygon.clone().unwrap();
    poly.outline = Some(Rc::new(ConstantProperty::from_value(true)));
    poly.height = Some(Rc::new(ConstantProperty::from_value(100.0)));
    poly.extruded_height = Some(Rc::new(ConstantProperty::from_value(200.0)));
    poly.granularity = Some(Rc::new(ConstantProperty::from_value(0.01)));
    poly.st_rotation = Some(Rc::new(ConstantProperty::from_value(0.5)));
    poly.per_position_height = Some(Rc::new(ConstantProperty::from_value(false)));
    poly.close_top = Some(Rc::new(ConstantProperty::from_value(true)));
    poly.close_bottom = Some(Rc::new(ConstantProperty::from_value(true)));
    poly.arc_type = Some(Rc::new(ConstantProperty::from_value(ArcType::Rhumb)));

    let mut entity = entity;
    entity.polygon = Some(poly);

    let updater = PolygonGeometryUpdater::new(entity, None).unwrap();
    assert!(updater.fill_enabled());
    assert!(updater.outline_enabled());
    assert!(updater.is_closed());

    let fill_instance = updater.create_fill_geometry_instance(time).unwrap();
    assert_eq!(fill_instance.geometry.height(), 200.0);
    assert_eq!(fill_instance.geometry.extruded_height(), 100.0);
    assert_eq!(fill_instance.geometry.granularity(), 0.01);
    assert_eq!(fill_instance.geometry.st_rotation(), 0.5);
    assert!(!fill_instance.geometry.per_position_height());
    assert!(fill_instance.geometry.close_top());
    assert!(fill_instance.geometry.close_bottom());
    assert_eq!(fill_instance.geometry.arc_type(), ArcType::Rhumb);
    assert!(fill_instance.geometry.offset_attribute().is_none());
    assert!(fill_instance.attributes.contains_key("show"));

    let outline_instance = updater.create_outline_geometry_instance(time).unwrap();
    assert_eq!(outline_instance.geometry.height(), 200.0);
    assert_eq!(outline_instance.geometry.extruded_height(), 100.0);
    assert_eq!(outline_instance.geometry.granularity(), 0.01);
    assert_eq!(outline_instance.geometry.arc_type(), ArcType::Rhumb);
    assert!(!outline_instance.geometry.per_position_height());
    assert!(outline_instance.geometry.offset_attribute().is_none());
    assert!(outline_instance.attributes.contains_key("show"));
    assert!(outline_instance.attributes.contains_key("color"));
}

#[test]
fn test_dynamic_updater_sets_properties() {
    let time = JulianDate::now();
    let hierarchy = PolygonHierarchy::default();

    let dyn_hier = DynamicTestProperty::new(Some(hierarchy.clone()));
    let dyn_height = DynamicTestProperty::new(Some(3.0));
    let dyn_extruded_height = DynamicTestProperty::new(Some(2.0));
    let dyn_granularity = DynamicTestProperty::new(Some(2.0));
    let dyn_st_rotation = DynamicTestProperty::new(Some(1.0));
    let dyn_close_top = DynamicTestProperty::new(Some(false));
    let dyn_close_bottom = DynamicTestProperty::new(Some(false));
    let dyn_arc_type = DynamicTestProperty::new(Some(ArcType::Rhumb));

    let mut poly = PolygonGraphics::new();
    poly.hierarchy = Some(Rc::new(dyn_hier));
    poly.show = Some(Rc::new(ConstantProperty::from_value(true)));
    poly.height = Some(Rc::new(dyn_height));
    poly.extruded_height = Some(Rc::new(dyn_extruded_height));
    poly.granularity = Some(Rc::new(dyn_granularity));
    poly.st_rotation = Some(Rc::new(dyn_st_rotation));
    poly.close_top = Some(Rc::new(dyn_close_top));
    poly.close_bottom = Some(Rc::new(dyn_close_bottom));
    poly.arc_type = Some(Rc::new(dyn_arc_type));
    poly.fill = Some(Rc::new(ConstantProperty::from_value(true)));
    poly.outline = Some(Rc::new(ConstantProperty::from_value(true)));
    poly.outline_color = Some(Rc::new(ConstantProperty::from_value(Color::RED)));

    let mut entity = Entity::new();
    entity.polygon = Some(poly);

    let updater = PolygonGeometryUpdater::new(entity, None).unwrap();
    assert!(updater.is_dynamic());

    let mut dynamic_updater = updater.create_dynamic_updater().unwrap();
    dynamic_updater.update(time);

    let options = dynamic_updater.options();
    assert_eq!(options.polygon_hierarchy, Some(hierarchy));
    assert_eq!(options.height, Some(3.0));
    assert_eq!(options.extruded_height, Some(2.0));
    assert_eq!(options.granularity, Some(2.0));
    assert_eq!(options.st_rotation, Some(1.0));
    assert_eq!(options.close_top, Some(false));
    assert_eq!(options.close_bottom, Some(false));
    assert_eq!(options.arc_type, Some(ArcType::Rhumb));
    assert!(options.offset_attribute.is_none());
}

#[test]
fn test_geometry_changed_event_is_raised_when_expected() {
    let mut entity = create_basic_polygon();
    let mut updater = PolygonGeometryUpdater::new(entity.clone(), None).unwrap();

    let event_count = Rc::new(std::cell::Cell::new(0));
    let count_clone = event_count.clone();
    updater.geometry_changed.add_listener(move |()| {
        count_clone.set(count_clone.get() + 1);
    });

    // 1. Change hierarchy
    let mut poly = entity.polygon.clone().unwrap();
    poly.hierarchy = Some(Rc::new(ConstantProperty::from_value(
        PolygonHierarchy::default(),
    )));
    entity.polygon = Some(poly.clone());
    updater.on_entity_property_changed(&entity, "polygon");
    assert_eq!(event_count.get(), 1);

    // 2. Change height
    poly.height = Some(Rc::new(ConstantProperty::from_value(82.0)));
    entity.polygon = Some(poly.clone());
    updater.on_entity_property_changed(&entity, "polygon");
    assert_eq!(event_count.get(), 2);

    // 3. Change availability
    updater.on_entity_property_changed(&entity, "availability");
    assert_eq!(event_count.get(), 3);

    // 4. Invalidate geometry by setting hierarchy = None
    poly.hierarchy = None;
    entity.polygon = Some(poly.clone());
    updater.on_entity_property_changed(&entity, "polygon");
    assert_eq!(event_count.get(), 4);

    // 5. Changing property while geometry is invalid should not fire event
    poly.height = None;
    entity.polygon = Some(poly);
    updater.on_entity_property_changed(&entity, "polygon");
    assert_eq!(event_count.get(), 4);

    // 6. Modifying an unrelated property should not have any effect
    updater.on_entity_property_changed(&entity, "viewFrom");
    assert_eq!(event_count.get(), 4);
}

#[test]
fn test_per_position_height_sets_on_terrain_to_false() {
    let mut entity = create_basic_polygon();
    let mut poly = entity.polygon.clone().unwrap();
    poly.height = None;
    poly.extruded_height = None;
    poly.fill = Some(Rc::new(ConstantProperty::from_value(true)));
    poly.per_position_height = Some(Rc::new(ConstantProperty::from_value(true)));
    entity.polygon = Some(poly);

    let updater = PolygonGeometryUpdater::new(entity, None).unwrap();
    assert!(!updater.on_terrain());
}

#[test]
fn test_computes_center() {
    let time = JulianDate::now();
    let entity = create_basic_polygon();
    let updater = PolygonGeometryUpdater::new(entity, None).unwrap();

    let center = updater.compute_center(time).unwrap();
    let scaled = Ellipsoid::WGS84.scale_to_geodetic_surface(center.into()).unwrap();
    let expected = Cartesian3::from_degrees(0.0, 0.0, Some(0.0), None).unwrap();

    assert!((scaled.x - expected.x).abs() < 1e-7);
    assert!((scaled.y - expected.y).abs() < 1e-7);
    assert!((scaled.z - expected.z).abs() < 1e-7);
}
