//! An observable collection of Entity instances where each entity has a unique id.
//! Direct port of CesiumJS `DataSources/EntityCollection.js`.

use std::collections::HashMap;

use crate::{
    create_guid, DeveloperError, Entity, Event, Iso8601, TimeInterval,
};

/// An observable collection of Entity instances where each entity has a unique id.
#[derive(Clone)]
pub struct EntityCollection {
    id: String,
    entities: HashMap<String, Entity>,
    ordered_ids: Vec<String>,
    added_entities: HashMap<String, Entity>,
    removed_entities: HashMap<String, Entity>,
    changed_entities: HashMap<String, Entity>,
    suspend_count: usize,
    show: bool,
    firing: bool,
    refire: bool,
    pub collection_changed: Event<(Vec<Entity>, Vec<Entity>, Vec<Entity>)>,
}

impl Default for EntityCollection {
    fn default() -> Self {
        Self::new()
    }
}

impl EntityCollection {
    /// Creates a new EntityCollection with a unique ID.
    pub fn new() -> Self {
        Self {
            id: create_guid(),
            entities: HashMap::new(),
            ordered_ids: Vec::new(),
            added_entities: HashMap::new(),
            removed_entities: HashMap::new(),
            changed_entities: HashMap::new(),
            suspend_count: 0,
            show: true,
            firing: false,
            refire: false,
            collection_changed: Event::new(),
        }
    }

    /// Gets a globally unique identifier for this collection.
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Gets whether or not this entity collection should be displayed.
    pub fn show(&self) -> bool {
        self.show
    }

    /// Sets whether or not this entity collection should be displayed.
    pub fn set_show(&mut self, show: bool) {
        if self.show == show {
            return;
        }
        self.suspend_events();
        self.show = show;
        // Mark all entities as changed
        for (id, entity) in &self.entities {
            if !self.added_entities.contains_key(id) {
                self.changed_entities.insert(id.clone(), entity.clone());
            }
        }
        let _ = self.resume_events();
    }

    /// Prevents collectionChanged events from being raised until resume_events is called.
    pub fn suspend_events(&mut self) {
        self.suspend_count += 1;
    }

    /// Resumes raising collectionChanged events immediately when an item is added or removed.
    pub fn resume_events(&mut self) -> Result<(), DeveloperError> {
        if self.suspend_count == 0 {
            return Err(DeveloperError::new(
                "resumeEvents can not be called before suspendEvents.",
            ));
        }
        self.suspend_count -= 1;
        self.fire_changed_event();
        Ok(())
    }

    fn fire_changed_event(&mut self) {
        if self.firing {
            self.refire = true;
            return;
        }

        if self.suspend_count == 0
            && (!self.added_entities.is_empty()
                || !self.removed_entities.is_empty()
                || !self.changed_entities.is_empty())
        {
            self.firing = true;
            loop {
                self.refire = false;
                let added: Vec<Entity> = self.added_entities.values().cloned().collect();
                let removed: Vec<Entity> = self.removed_entities.values().cloned().collect();
                let changed: Vec<Entity> = self.changed_entities.values().cloned().collect();

                self.added_entities.clear();
                self.removed_entities.clear();
                self.changed_entities.clear();

                self.collection_changed
                    .raise_event(&(added, removed, changed));

                if !self.refire {
                    break;
                }
            }
            self.firing = false;
        }
    }

    /// Gets the array of Entity instances in the collection.
    pub fn values(&self) -> Vec<&Entity> {
        self.ordered_ids
            .iter()
            .filter_map(|id| self.entities.get(id))
            .collect()
    }

    /// Gets the number of entities in the collection.
    pub fn len(&self) -> usize {
        self.entities.len()
    }

    /// Returns true if the collection is empty.
    pub fn is_empty(&self) -> bool {
        self.entities.is_empty()
    }

    /// Returns true if the provided entity is in this collection, false otherwise.
    pub fn contains(&self, entity: &Entity) -> bool {
        self.entities.contains_key(&entity.id)
    }

    /// Returns true if an entity with the given id is in this collection.
    pub fn contains_id(&self, id: &str) -> bool {
        self.entities.contains_key(id)
    }

    /// Gets an entity with the specified id.
    pub fn get_by_id(&self, id: &str) -> Option<&Entity> {
        self.entities.get(id)
    }

    /// Gets a mutable entity with the specified id.
    pub fn get_by_id_mut(&mut self, id: &str) -> Option<&mut Entity> {
        self.entities.get_mut(id)
    }

    /// Notifies the collection that an entity's definition has changed.
    pub fn notify_entity_changed(&mut self, entity_id: &str) {
        if let Some(entity) = self.entities.get(entity_id) {
            let entity_clone = entity.clone();
            if !self.added_entities.contains_key(entity_id) {
                self.changed_entities
                    .insert(entity_id.to_string(), entity_clone);
            }
            self.fire_changed_event();
        }
    }

    /// Add an entity to the collection.
    pub fn add(&mut self, entity: Entity) -> Result<&mut Entity, DeveloperError> {
        let id = entity.id.clone();
        if self.entities.contains_key(&id) {
            return Err(DeveloperError::new(format!(
                "An entity with id {id} already exists in this collection."
            )));
        }

        self.entities.insert(id.clone(), entity.clone());
        self.ordered_ids.push(id.clone());

        if self.removed_entities.remove(&id).is_none() {
            self.added_entities.insert(id.clone(), entity);
        }

        self.fire_changed_event();
        Ok(self.entities.get_mut(&id).unwrap())
    }

    /// Gets an entity with the specified id or creates it and adds it to the collection if it does not exist.
    pub fn get_or_create_entity(&mut self, id: &str) -> Result<&mut Entity, DeveloperError> {
        if id.is_empty() {
            return Err(DeveloperError::new("id is required."));
        }
        if !self.entities.contains_key(id) {
            let entity = Entity::with_id(id);
            self.add(entity)?;
        }
        Ok(self.entities.get_mut(id).unwrap())
    }

    /// Removes an entity from the collection.
    pub fn remove(&mut self, entity: &Entity) -> bool {
        self.remove_by_id(&entity.id)
    }

    /// Removes an entity with the provided id from the collection.
    pub fn remove_by_id(&mut self, id: &str) -> bool {
        if id.is_empty() || !self.entities.contains_key(id) {
            return false;
        }

        let entity = self.entities.remove(id).unwrap();
        if let Some(pos) = self.ordered_ids.iter().position(|x| x == id) {
            self.ordered_ids.remove(pos);
        }

        if self.added_entities.remove(id).is_none() {
            self.removed_entities.insert(id.to_string(), entity);
            self.changed_entities.remove(id);
        }

        self.fire_changed_event();
        true
    }

    /// Removes all Entities from the collection.
    pub fn remove_all(&mut self) {
        if self.entities.is_empty() {
            return;
        }
        for (id, entity) in self.entities.drain() {
            if self.added_entities.remove(&id).is_none() {
                self.removed_entities.insert(id, entity);
            }
        }
        self.ordered_ids.clear();
        self.changed_entities.clear();
        self.fire_changed_event();
    }

    /// Computes the maximum availability of the entities in the collection.
    pub fn compute_availability(&self) -> TimeInterval {
        let min_iso = Iso8601::minimum_value();
        let max_iso = Iso8601::maximum_value();
        let mut start_time = max_iso;
        let mut stop_time = min_iso;

        for entity in self.entities.values() {
            if let Some(ref availability) = entity.availability {
                if let Some(start) = availability.start() {
                    if start.compare(&start_time) == std::cmp::Ordering::Less && !start.equals(&min_iso) {
                        start_time = start;
                    }
                }
                if let Some(stop) = availability.stop() {
                    if stop.compare(&stop_time) == std::cmp::Ordering::Greater && !stop.equals(&max_iso) {
                        stop_time = stop;
                    }
                }
            }
        }

        if max_iso.equals(&start_time) {
            start_time = min_iso;
        }
        if min_iso.equals(&stop_time) {
            stop_time = max_iso;
        }

        TimeInterval::new(start_time, stop_time, true, true)
    }
}
