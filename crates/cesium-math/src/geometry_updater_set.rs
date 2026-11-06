//! Manages a set of GeometryUpdater classes for each entity.
//! Direct port of CesiumJS `DataSources/GeometryUpdaterSet.js` (Apache-2.0).

use std::cell::RefCell;
use std::rc::Rc;

use crate::{
    BoxGeometryUpdater, CorridorGeometryUpdater, CylinderGeometryUpdater, DeveloperError,
    EllipseGeometryUpdater, EllipsoidGeometryUpdater, Entity, Event, EventHelper, ListenerId,
    PlaneGeometryUpdater, PolygonGeometryUpdater, PolylineVolumeGeometryUpdater,
    RectangleGeometryUpdater, WallGeometryUpdater,
};

/// Trait for extensible custom geometry updaters.
pub trait CustomUpdater: std::any::Any {
    /// Gets the event that is raised whenever the underlying geometry changes.
    fn geometry_changed(&self) -> &Event<()>;
    /// Notifies that an entity property has changed.
    fn on_entity_property_changed(&self, entity: &Entity, property_name: &str);
    /// Destroys this updater.
    fn destroy(&self) {}
    /// Custom type tag for identification.
    fn type_name(&self) -> &'static str {
        "CustomUpdater"
    }
}

/// Represents one geometry updater instance in the set.
pub enum GeometryUpdaterInstance {
    Box(BoxGeometryUpdater),
    Cylinder(CylinderGeometryUpdater),
    Corridor(CorridorGeometryUpdater),
    Ellipse(EllipseGeometryUpdater),
    Ellipsoid(EllipsoidGeometryUpdater),
    Plane(PlaneGeometryUpdater),
    Polygon(PolygonGeometryUpdater),
    PolylineVolume(PolylineVolumeGeometryUpdater),
    Rectangle(RectangleGeometryUpdater),
    Wall(WallGeometryUpdater),
    Custom(Rc<dyn CustomUpdater>),
}

impl GeometryUpdaterInstance {
    pub fn geometry_changed(&self) -> &Event<()> {
        match self {
            Self::Box(u) => &u.geometry_changed,
            Self::Cylinder(u) => &u.geometry_changed,
            Self::Corridor(u) => &u.geometry_changed,
            Self::Ellipse(u) => &u.geometry_changed,
            Self::Ellipsoid(u) => &u.geometry_changed,
            Self::Plane(u) => &u.geometry_changed,
            Self::Polygon(u) => &u.geometry_changed,
            Self::PolylineVolume(u) => &u.geometry_changed,
            Self::Rectangle(u) => &u.geometry_changed,
            Self::Wall(u) => &u.geometry_changed,
            Self::Custom(u) => u.geometry_changed(),
        }
    }

    pub fn on_entity_property_changed(&self, entity: &Entity, property_name: &str) {
        if let Self::Custom(u) = self {
            u.on_entity_property_changed(entity, property_name);
        }
    }

    pub fn is_box(&self) -> bool {
        matches!(self, Self::Box(_))
    }

    pub fn is_cylinder(&self) -> bool {
        matches!(self, Self::Cylinder(_))
    }

    pub fn is_corridor(&self) -> bool {
        matches!(self, Self::Corridor(_))
    }

    pub fn is_ellipse(&self) -> bool {
        matches!(self, Self::Ellipse(_))
    }

    pub fn is_ellipsoid(&self) -> bool {
        matches!(self, Self::Ellipsoid(_))
    }

    pub fn is_plane(&self) -> bool {
        matches!(self, Self::Plane(_))
    }

    pub fn is_polygon(&self) -> bool {
        matches!(self, Self::Polygon(_))
    }

    pub fn is_polyline_volume(&self) -> bool {
        matches!(self, Self::PolylineVolume(_))
    }

    pub fn is_rectangle(&self) -> bool {
        matches!(self, Self::Rectangle(_))
    }

    pub fn is_wall(&self) -> bool {
        matches!(self, Self::Wall(_))
    }

    pub fn is_custom(&self) -> bool {
        matches!(self, Self::Custom(_))
    }
}

pub type CustomUpdaterFactory =
    Rc<dyn Fn(&Entity, Option<()>) -> Rc<dyn CustomUpdater>>;

thread_local! {
    static REGISTERED_UPDATERS: RefCell<Vec<(&'static str, CustomUpdaterFactory)>> =
        const { RefCell::new(Vec::new()) };
}

/// Manages a set of updater classes for the GeometryVisualizer for each entity.
pub struct GeometryUpdaterSet {
    pub entity: Entity,
    pub updaters: Vec<GeometryUpdaterInstance>,
    pub geometry_changed: Event<()>,
    event_helper: EventHelper,
    entity_subscription: Option<ListenerId>,
}

impl GeometryUpdaterSet {
    /// Creates a new GeometryUpdaterSet with all default and registered updaters.
    pub fn new(entity: Entity, scene: Option<()>) -> Result<Self, DeveloperError> {
        let mut updaters = Vec::with_capacity(10);
        let geometry_changed = Event::new();
        let mut event_helper = EventHelper::new();

        // 1. BoxGeometryUpdater
        updaters.push(GeometryUpdaterInstance::Box(BoxGeometryUpdater::new(
            entity.clone(),
            scene,
        )?));

        // 2. CylinderGeometryUpdater
        updaters.push(GeometryUpdaterInstance::Cylinder(
            CylinderGeometryUpdater::new(entity.clone(), scene)?,
        ));

        // 3. CorridorGeometryUpdater
        updaters.push(GeometryUpdaterInstance::Corridor(
            CorridorGeometryUpdater::new(entity.clone(), scene)?,
        ));

        // 4. EllipseGeometryUpdater
        updaters.push(GeometryUpdaterInstance::Ellipse(
            EllipseGeometryUpdater::new(entity.clone(), scene)?,
        ));

        // 5. EllipsoidGeometryUpdater
        updaters.push(GeometryUpdaterInstance::Ellipsoid(
            EllipsoidGeometryUpdater::new(entity.clone(), scene)?,
        ));

        // 6. PlaneGeometryUpdater
        updaters.push(GeometryUpdaterInstance::Plane(
            PlaneGeometryUpdater::new(entity.clone(), scene)?,
        ));

        // 7. PolygonGeometryUpdater
        updaters.push(GeometryUpdaterInstance::Polygon(
            PolygonGeometryUpdater::new(entity.clone(), scene)?,
        ));

        // 8. PolylineVolumeGeometryUpdater
        updaters.push(GeometryUpdaterInstance::PolylineVolume(
            PolylineVolumeGeometryUpdater::new(entity.clone(), scene)?,
        ));

        // 9. RectangleGeometryUpdater
        updaters.push(GeometryUpdaterInstance::Rectangle(
            RectangleGeometryUpdater::new(entity.clone(), scene)?,
        ));

        // 10. WallGeometryUpdater
        updaters.push(GeometryUpdaterInstance::Wall(WallGeometryUpdater::new(
            entity.clone(),
            scene,
        )?));

        // Custom registered updaters
        REGISTERED_UPDATERS.with(|reg| {
            for (_, factory) in reg.borrow().iter() {
                let custom = factory(&entity, scene);
                updaters.push(GeometryUpdaterInstance::Custom(custom));
            }
        });

        // Hook up geometry_changed forwarding
        for updater in &updaters {
            let gc = geometry_changed.clone();
            event_helper.add(updater.geometry_changed(), move |_| {
                gc.raise_event(&());
            });
        }

        let entity_sub = entity.definition_changed.add_listener(move |_| {});

        Ok(Self {
            entity,
            updaters,
            geometry_changed,
            event_helper,
            entity_subscription: Some(entity_sub),
        })
    }

    /// Add the provided updater factory to the default list of updaters if not already included.
    pub fn register_updater(name: &'static str, factory: CustomUpdaterFactory) {
        REGISTERED_UPDATERS.with(|reg| {
            let mut reg = reg.borrow_mut();
            if !reg.iter().any(|(n, _)| *n == name) {
                reg.push((name, factory));
            }
        });
    }

    /// Remove the provided updater from the default list of updaters if included.
    pub fn unregister_updater(name: &'static str) {
        REGISTERED_UPDATERS.with(|reg| {
            reg.borrow_mut().retain(|(n, _)| *n != name);
        });
    }

    /// Gets the list of updaters in this set.
    pub fn updaters(&self) -> &[GeometryUpdaterInstance] {
        &self.updaters
    }

    /// Iterates over each updater in the set.
    pub fn for_each<F: FnMut(&GeometryUpdaterInstance)>(&self, mut callback: F) {
        for updater in &self.updaters {
            callback(updater);
        }
    }

    /// Destroys the updater set, unhooking all event listeners.
    pub fn destroy(&mut self) {
        self.event_helper.remove_all();
        self.entity_subscription = None;
        self.updaters.clear();
    }
}

impl Drop for GeometryUpdaterSet {
    fn drop(&mut self) {
        self.destroy();
    }
}
