//! Direct port and verification against CesiumJS `GeometryUpdaterSpec.js` (Apache-2.0).

use cesium_math::{Entity, GeometryUpdater, GeometryUpdaterOptions, JulianDate, TimeInterval, TimeIntervalCollection};

#[test]
fn test_constructor_throws_if_missing_required_options() {
    // Missing entity
    let opts1 = GeometryUpdaterOptions {
        entity: None,
        geometry_property_name: Some("box".to_string()),
        observed_property_names: Some(vec!["availability".to_string(), "box".to_string()]),
    };
    assert!(GeometryUpdater::new(Some(opts1)).is_err());

    // Missing geometryPropertyName
    let opts2 = GeometryUpdaterOptions {
        entity: Some(Entity::new()),
        geometry_property_name: None,
        observed_property_names: Some(vec!["availability".to_string(), "box".to_string()]),
    };
    assert!(GeometryUpdater::new(Some(opts2)).is_err());

    // Missing observedPropertyNames
    let opts3 = GeometryUpdaterOptions {
        entity: Some(Entity::new()),
        geometry_property_name: Some("box".to_string()),
        observed_property_names: None,
    };
    assert!(GeometryUpdater::new(Some(opts3)).is_err());

    // Missing options entirely
    assert!(GeometryUpdater::new(None).is_err());
}

#[test]
fn test_constructor_sets_expected_properties() {
    let entity = Entity::with_id("testEntity");
    let opts = GeometryUpdaterOptions {
        entity: Some(entity.clone()),
        geometry_property_name: Some("box".to_string()),
        observed_property_names: Some(vec!["availability".to_string(), "box".to_string()]),
    };

    let updater = GeometryUpdater::new(Some(opts)).unwrap();
    assert_eq!(updater.id(), "box-testEntity");
    assert_eq!(updater.entity().id, "testEntity");
    assert_eq!(updater.geometry_property_name(), "box");
    assert_eq!(updater.observed_property_names().len(), 2);
    assert!(!updater.fill_enabled());
    assert!(!updater.outline_enabled());
    assert!(!updater.is_dynamic());
    assert!(!updater.is_closed());
    assert_eq!(updater.outline_width(), 1.0);
}

#[test]
fn test_create_dynamic_updater_validations() {
    let entity = Entity::new();
    let opts = GeometryUpdaterOptions {
        entity: Some(entity),
        geometry_property_name: Some("box".to_string()),
        observed_property_names: Some(vec!["availability".to_string(), "box".to_string()]),
    };
    let mut updater = GeometryUpdater::new(Some(opts)).unwrap();

    // Throws if not dynamic
    assert!(updater.create_dynamic_updater(Some(()), Some(())).is_err());

    updater.set_dynamic(true);

    // Throws if primitives undefined
    assert!(updater.create_dynamic_updater(None, Some(())).is_err());

    // Throws if groundPrimitives undefined
    assert!(updater.create_dynamic_updater(Some(()), None).is_err());

    // Succeeds when all provided and dynamic
    assert!(updater.create_dynamic_updater(Some(()), Some(())).is_ok());
}

#[test]
fn test_has_constant_fill_and_outline() {
    let mut entity = Entity::new();
    let opts = GeometryUpdaterOptions {
        entity: Some(entity.clone()),
        geometry_property_name: Some("box".to_string()),
        observed_property_names: Some(vec!["availability".to_string(), "box".to_string()]),
    };
    let mut updater = GeometryUpdater::new(Some(opts)).unwrap();

    // When fill/outline not enabled, always constant
    assert!(updater.has_constant_fill());
    assert!(updater.has_constant_outline());

    updater.set_fill_enabled(true);
    updater.set_outline_enabled(true);

    // With no availability on entity, still constant
    assert!(updater.has_constant_fill());
    assert!(updater.has_constant_outline());

    // When entity has availability interval, constancy depends on interval
    let mut intervals = TimeIntervalCollection::new();
    intervals.add_interval(TimeInterval::new(
        JulianDate::now(),
        JulianDate::now().add_seconds(100.0),
        true,
        true,
    ));
    entity.availability = Some(intervals);

    let opts2 = GeometryUpdaterOptions {
        entity: Some(entity),
        geometry_property_name: Some("box".to_string()),
        observed_property_names: Some(vec!["availability".to_string(), "box".to_string()]),
    };
    let mut updater2 = GeometryUpdater::new(Some(opts2)).unwrap();
    updater2.set_fill_enabled(true);
    updater2.set_outline_enabled(true);

    assert!(!updater2.has_constant_fill());
    assert!(!updater2.has_constant_outline());
}
