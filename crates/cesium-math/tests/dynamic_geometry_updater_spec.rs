//! Direct port and verification against CesiumJS `DataSources/DynamicGeometryUpdaterSpec.js` (Apache-2.0).

use cesium_math::{
    BoxGraphics, DynamicGeometryUpdater, Entity, GeometryUpdater, GeometryUpdaterOptions,
};

#[test]
fn test_constructor_throws_with_no_updater() {
    let dummy_primitives = ();
    let dummy_ground_primitives = ();
    let result = DynamicGeometryUpdater::new(
        None::<&GeometryUpdater>,
        Some(&dummy_primitives),
        Some(&dummy_ground_primitives),
    );
    assert!(result.is_err());
}

#[test]
fn test_constructor_throws_with_no_primitives() {
    let mut entity = Entity::new();
    entity.box_graphics = Some(BoxGraphics::new());
    let updater = GeometryUpdater::new(Some(GeometryUpdaterOptions {
        entity: Some(entity),
        geometry_property_name: Some("box".to_string()),
        observed_property_names: Some(vec!["availability".to_string(), "box".to_string()]),
    }))
    .unwrap();

    let dummy_ground_primitives = ();
    let result = DynamicGeometryUpdater::new(
        Some(&updater),
        None::<&()>,
        Some(&dummy_ground_primitives),
    );
    assert!(result.is_err());
}

#[test]
fn test_constructor_throws_with_no_ground_primitives() {
    let mut entity = Entity::new();
    entity.box_graphics = Some(BoxGraphics::new());
    let updater = GeometryUpdater::new(Some(GeometryUpdaterOptions {
        entity: Some(entity),
        geometry_property_name: Some("box".to_string()),
        observed_property_names: Some(vec!["availability".to_string(), "box".to_string()]),
    }))
    .unwrap();

    let dummy_primitives = ();
    let result = DynamicGeometryUpdater::new(
        Some(&updater),
        Some(&dummy_primitives),
        None::<&()>,
    );
    assert!(result.is_err());
}

#[test]
fn test_update_throws_with_no_time() {
    let mut entity = Entity::new();
    entity.box_graphics = Some(BoxGraphics::new());
    let updater = GeometryUpdater::new(Some(GeometryUpdaterOptions {
        entity: Some(entity),
        geometry_property_name: Some("box".to_string()),
        observed_property_names: Some(vec!["availability".to_string(), "box".to_string()]),
    }))
    .unwrap();

    let dummy_primitives = ();
    let dummy_ground_primitives = ();
    let mut dynamic_updater = DynamicGeometryUpdater::new(
        Some(&updater),
        Some(&dummy_primitives),
        Some(&dummy_ground_primitives),
    )
    .unwrap();

    assert!(dynamic_updater.update(None).is_err());
}
