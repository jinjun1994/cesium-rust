//! Direct port and verification against CesiumJS `GeometryUpdaterSetSpec.js` (Apache-2.0).

use std::cell::Cell;
use std::rc::Rc;

use cesium_math::{CustomUpdater, Entity, Event, GeometryUpdaterSet};

#[test]
fn test_has_expected_defaults() {
    let updater_set = GeometryUpdaterSet::new(Entity::new(), None).unwrap();

    assert_eq!(updater_set.updaters.len(), 10);
    assert!(updater_set.updaters[0].is_box());
    assert!(updater_set.updaters[1].is_cylinder());
    assert!(updater_set.updaters[2].is_corridor());
    assert!(updater_set.updaters[3].is_ellipse());
    assert!(updater_set.updaters[4].is_ellipsoid());
    assert!(updater_set.updaters[5].is_plane());
    assert!(updater_set.updaters[6].is_polygon());
    assert!(updater_set.updaters[7].is_polyline_volume());
    assert!(updater_set.updaters[8].is_rectangle());
    assert!(updater_set.updaters[9].is_wall());
}

struct FakeUpdater {
    geometry_changed: Event<()>,
}

impl CustomUpdater for FakeUpdater {
    fn geometry_changed(&self) -> &Event<()> {
        &self.geometry_changed
    }

    fn on_entity_property_changed(&self, _entity: &Entity, _property_name: &str) {}

    fn type_name(&self) -> &'static str {
        "FakeUpdater"
    }
}

#[test]
fn test_registers_new_updater() {
    let fake_factory = Rc::new(|_entity: &Entity, _scene: Option<()>| -> Rc<dyn CustomUpdater> {
        Rc::new(FakeUpdater {
            geometry_changed: Event::new(),
        })
    });

    GeometryUpdaterSet::register_updater("FakeUpdater", fake_factory);

    let updater_set = GeometryUpdaterSet::new(Entity::new(), None).unwrap();
    assert_eq!(updater_set.updaters.len(), 11);
    assert!(updater_set.updaters[10].is_custom());

    GeometryUpdaterSet::unregister_updater("FakeUpdater");
    let updater_set2 = GeometryUpdaterSet::new(Entity::new(), None).unwrap();
    assert_eq!(updater_set2.updaters.len(), 10);
}

#[test]
fn test_geometry_changed_event_and_for_each() {
    let updater_set = GeometryUpdaterSet::new(Entity::new(), None).unwrap();

    let count = Rc::new(Cell::new(0usize));
    let count_clone = Rc::clone(&count);
    let _sub = updater_set.geometry_changed.add_listener(move |_| {
        count_clone.set(count_clone.get() + 1);
    });

    // Trigger an inner updater's geometry_changed
    updater_set.updaters[0].geometry_changed().raise_event(&());
    assert_eq!(count.get(), 1);

    // Test for_each
    let mut visited = 0;
    updater_set.for_each(|_| {
        visited += 1;
    });
    assert_eq!(visited, 10);
}
