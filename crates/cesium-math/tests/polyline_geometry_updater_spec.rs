//! Direct port and verification against CesiumJS `DataSources/PolylineGeometryUpdaterSpec.js` (Apache-2.0).

use std::cell::RefCell;
use std::rc::Rc;

use cesium_math::{
    ArcType, BoundingSphere, BoundingSphereState, Cartesian3, Color, ColorMaterialProperty,
    ConstantProperty, DynProperty, Entity, Event, JulianDate, PolylineGeometryUpdater,
    PolylineGraphics,
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

fn create_basic_polyline() -> Entity {
    let positions = Cartesian3::from_radians_array(&[0.0, 0.0, 1.0, 0.0, 1.0, 1.0, 0.0, 1.0], None)
        .unwrap();
    let mut polyline = PolylineGraphics::new();
    polyline.positions = Some(Rc::new(ConstantProperty::from_value(positions)));

    let mut entity = Entity::new();
    entity.polyline = Some(polyline);
    entity
}

#[test]
fn test_constructor_sets_expected_defaults() {
    let entity = Entity::new();
    let updater = PolylineGeometryUpdater::new(entity.clone(), None).unwrap();

    assert_eq!(updater.entity().id, entity.id);
    assert!(!updater.is_closed());
    assert!(!updater.fill_enabled());
    assert!(updater.fill_material_property().is_none());
    assert!(updater.depth_fail_material_property().is_none());
    assert!(!updater.outline_enabled());
    assert!(updater.has_constant_fill());
    assert!(updater.has_constant_outline());
    assert!(updater.shadows_property().is_none());
    assert!(updater.distance_display_condition_property().is_none());
    assert!(!updater.is_dynamic());
    assert!(!updater.clamp_to_ground());
    assert!(updater.arc_type().is_none());
    assert_eq!(updater.z_index(), 0);

    let time = JulianDate::now();
    assert!(!updater.is_outline_visible(time));
    assert!(!updater.is_filled(time));
}

#[test]
fn test_no_geometry_when_polyline_undefined_or_not_shown() {
    let mut entity = create_basic_polyline();
    let mut updater = PolylineGeometryUpdater::new(entity.clone(), None).unwrap();
    assert!(updater.fill_enabled());

    entity.polyline = None;
    updater.on_entity_property_changed(&entity, "polyline");
    assert!(!updater.fill_enabled());
    assert!(!updater.outline_enabled());
    assert!(!updater.is_dynamic());

    let mut entity2 = create_basic_polyline();
    let mut poly = entity2.polyline.clone().unwrap();
    poly.show = Some(Rc::new(ConstantProperty::from_value(false)));
    entity2.polyline = Some(poly);
    let updater2 = PolylineGeometryUpdater::new(entity2, None).unwrap();
    assert!(!updater2.fill_enabled());
}

#[test]
fn test_values_correct_when_using_default_graphics() {
    let entity = create_basic_polyline();
    let updater = PolylineGeometryUpdater::new(entity, None).unwrap();

    assert!(!updater.is_closed());
    assert!(updater.fill_enabled());
    assert!(updater.fill_material_property().is_some());
    assert!(updater.depth_fail_material_property().is_none());
    assert!(!updater.outline_enabled());
    assert!(updater.has_constant_fill());
    assert!(updater.has_constant_outline());
    assert!(updater.shadows_property().is_some());
    assert!(updater.distance_display_condition_property().is_some());
    assert!(!updater.is_dynamic());
    assert!(!updater.clamp_to_ground());
    assert!(updater.arc_type().is_none());
    assert_eq!(updater.z_index(), 0);
}

#[test]
fn test_polyline_material_and_depth_fail_exposed() {
    let mut entity = create_basic_polyline();
    let mut poly = entity.polyline.clone().unwrap();
    let color_mat = Rc::new(ColorMaterialProperty::from_color(Color::RED));
    poly.material = Some(color_mat.clone());
    let depth_mat = Rc::new(ColorMaterialProperty::from_color(Color::BLUE));
    poly.depth_fail_material = Some(depth_mat.clone());
    entity.polyline = Some(poly);

    let updater = PolylineGeometryUpdater::new(entity, None).unwrap();
    assert!(updater.fill_material_property().is_some());
    assert!(updater.depth_fail_material_property().is_some());
}

#[test]
fn test_time_varying_positions_and_width_causes_geometry_to_be_dynamic() {
    let entity = create_basic_polyline();
    let mut updater = PolylineGeometryUpdater::new(entity.clone(), None).unwrap();
    assert!(!updater.is_dynamic());

    let dynamic_positions = DynamicTestProperty::new(Some(Vec::<Cartesian3>::new()));

    let mut entity2 = entity.clone();
    let mut poly2 = entity2.polyline.unwrap();
    poly2.positions = Some(Rc::new(dynamic_positions));
    entity2.polyline = Some(poly2);

    updater.on_entity_property_changed(&entity2, "polyline");
    assert!(updater.is_dynamic());

    let dynamic_width = DynamicTestProperty::new(Some(5.0));
    let mut entity3 = entity.clone();
    let mut poly3 = entity3.polyline.unwrap();
    poly3.width = Some(Rc::new(dynamic_width));
    entity3.polyline = Some(poly3);

    let updater3 = PolylineGeometryUpdater::new(entity3, None).unwrap();
    assert!(updater3.is_dynamic());
}

#[test]
fn test_creates_fill_geometry_instance() {
    let time = JulianDate::now();
    let positions = Cartesian3::from_radians_array(&[0.0, 0.0, 1.0, 0.0, 1.0, 1.0, 0.0, 1.0], None)
        .unwrap();

    let mut poly = PolylineGraphics::new();
    poly.positions = Some(Rc::new(ConstantProperty::from_value(positions.clone())));
    poly.width = Some(Rc::new(ConstantProperty::from_value(4.0)));
    poly.arc_type = Some(Rc::new(ConstantProperty::from_value(ArcType::Rhumb)));
    poly.granularity = Some(Rc::new(ConstantProperty::from_value(0.1)));

    let mut entity = Entity::new();
    entity.polyline = Some(poly);

    let updater = PolylineGeometryUpdater::new(entity, None).unwrap();
    let instance = updater.create_fill_geometry_instance(time).unwrap();

    assert_eq!(instance.geometry.positions, positions);
    assert_eq!(instance.geometry.width, 4.0);
    assert_eq!(instance.geometry.arc_type, ArcType::Rhumb);
    assert_eq!(instance.geometry.granularity, 0.1);
    assert!(instance.attributes.contains_key("show"));
    assert!(instance.attributes.contains_key("color"));
}

#[test]
fn test_dynamic_updater_sets_properties_and_computes_bounding_sphere() {
    let time = JulianDate::now();
    let positions = Cartesian3::from_radians_array(&[0.0, 0.0, 1.0, 0.0, 1.0, 1.0, 0.0, 1.0], None)
        .unwrap();

    let dyn_positions = DynamicTestProperty::new(Some(positions.clone()));
    let dyn_width = DynamicTestProperty::new(Some(3.0));

    let mut poly = PolylineGraphics::new();
    poly.positions = Some(Rc::new(dyn_positions));
    poly.width = Some(Rc::new(dyn_width));
    poly.arc_type = Some(Rc::new(ConstantProperty::from_value(ArcType::Geodesic)));

    let mut entity = Entity::new();
    entity.polyline = Some(poly);

    let updater = PolylineGeometryUpdater::new(entity, None).unwrap();
    assert!(updater.is_dynamic());

    let mut dynamic_updater = updater.create_dynamic_updater().unwrap();
    dynamic_updater.update(time);

    let options = dynamic_updater.options();
    assert_eq!(options.positions.as_ref().unwrap(), &positions);
    assert_eq!(options.width, Some(3.0));
    assert_eq!(options.arc_type, Some(ArcType::Geodesic));

    let mut bs = BoundingSphere::default();
    let state = dynamic_updater.get_bounding_sphere(Some(&mut bs)).unwrap();
    assert_eq!(state, BoundingSphereState::Done);
    assert!(bs.radius > 0.0);

    // Throws on missing result
    assert!(dynamic_updater.get_bounding_sphere(None).is_err());
}

#[test]
fn test_geometry_changed_event_is_raised_when_expected() {
    let mut entity = create_basic_polyline();
    let mut updater = PolylineGeometryUpdater::new(entity.clone(), None).unwrap();

    let event_count = Rc::new(std::cell::Cell::new(0));
    let count_clone = event_count.clone();
    updater.geometry_changed.add_listener(move |()| {
        count_clone.set(count_clone.get() + 1);
    });

    // 1. Change positions
    let mut poly = entity.polyline.clone().unwrap();
    poly.positions = Some(Rc::new(ConstantProperty::from_value(vec![])));
    entity.polyline = Some(poly.clone());
    updater.on_entity_property_changed(&entity, "polyline");
    assert_eq!(event_count.get(), 1);

    // 2. Change width
    poly.width = Some(Rc::new(ConstantProperty::from_value(82.0)));
    entity.polyline = Some(poly.clone());
    updater.on_entity_property_changed(&entity, "polyline");
    assert_eq!(event_count.get(), 2);

    // 3. Change availability
    updater.on_entity_property_changed(&entity, "availability");
    assert_eq!(event_count.get(), 3);

    // 4. Invalidate geometry by setting positions = None
    poly.positions = None;
    entity.polyline = Some(poly.clone());
    updater.on_entity_property_changed(&entity, "polyline");
    assert_eq!(event_count.get(), 4);

    // 5. Changing property while geometry is invalid should not fire event
    poly.width = None;
    entity.polyline = Some(poly);
    updater.on_entity_property_changed(&entity, "polyline");
    assert_eq!(event_count.get(), 4);

    // 6. Modifying an unrelated property should not have any effect
    updater.on_entity_property_changed(&entity, "viewFrom");
    assert_eq!(event_count.get(), 4);
}
