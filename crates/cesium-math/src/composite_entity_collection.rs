//! Non-destructively composites multiple EntityCollection instances into a single collection.
//! Direct port of CesiumJS `DataSources/CompositeEntityCollection.js`.

use crate::{create_guid, DeveloperError, Entity, EntityCollection, Event};

/// Non-destructively composites multiple EntityCollection instances into a single collection.
#[derive(Clone)]
pub struct CompositeEntityCollection {
    id: String,
    composite: EntityCollection,
    collections: Vec<EntityCollection>,
    suspend_count: usize,
    should_recomposite: bool,
    pub collection_changed: Event<(Vec<Entity>, Vec<Entity>, Vec<Entity>)>,
}

impl Default for CompositeEntityCollection {
    fn default() -> Self {
        Self::new()
    }
}

impl CompositeEntityCollection {
    /// Creates an empty CompositeEntityCollection.
    pub fn new() -> Self {
        Self {
            id: create_guid(),
            composite: EntityCollection::new(),
            collections: Vec::new(),
            suspend_count: 0,
            should_recomposite: false,
            collection_changed: Event::new(),
        }
    }

    /// Creates a CompositeEntityCollection with the provided initial collections.
    pub fn with_collections(collections: Vec<EntityCollection>) -> Self {
        let mut composite = Self::new();
        for collection in collections {
            let _ = composite.add_collection(collection, None);
        }
        composite
    }

    /// Gets a globally unique identifier for this collection.
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Gets the number of collections in this composite.
    pub fn get_collections_length(&self) -> usize {
        self.collections.len()
    }

    /// Gets the array of Entity instances in the composite collection.
    pub fn values(&self) -> Vec<&Entity> {
        self.composite.values()
    }

    /// Returns true if the provided entity is in this collection, false otherwise.
    pub fn contains(&self, entity: &Entity) -> bool {
        self.composite.contains(entity)
    }

    /// Checks to see if the composite contains a given collection.
    pub fn contains_collection(&self, collection: &EntityCollection) -> bool {
        self.collections.iter().any(|c| c.id() == collection.id())
    }

    /// Determines the index of a given collection in the composite.
    pub fn index_of_collection(&self, collection: &EntityCollection) -> Option<usize> {
        self.collections.iter().position(|c| c.id() == collection.id())
    }

    /// Gets a collection by index from the composite.
    pub fn get_collection(&self, index: usize) -> Option<&EntityCollection> {
        self.collections.get(index)
    }

    /// Gets a mutable collection by index from the composite.
    pub fn get_collection_mut(&mut self, index: usize) -> Option<&mut EntityCollection> {
        self.collections.get_mut(index)
    }

    /// Prevents events from being raised until resume_events is called.
    pub fn suspend_events(&mut self) {
        self.suspend_count += 1;
        self.composite.suspend_events();
    }

    /// Resumes raising events.
    pub fn resume_events(&mut self) -> Result<(), DeveloperError> {
        if self.suspend_count == 0 {
            return Err(DeveloperError::new(
                "resumeEvents can not be called before suspendEvents.",
            ));
        }
        self.suspend_count -= 1;
        self.composite.resume_events()?;
        if self.suspend_count == 0 && self.should_recomposite {
            self.recomposite();
        }
        Ok(())
    }

    /// Adds a collection to the composite.
    pub fn add_collection(
        &mut self,
        collection: EntityCollection,
        index: Option<usize>,
    ) -> Result<(), DeveloperError> {
        if let Some(idx) = index {
            if idx > self.collections.len() {
                return Err(DeveloperError::new(
                    "index must be less than or equal to the number of collections.",
                ));
            }
            self.collections.insert(idx, collection);
        } else {
            self.collections.push(collection);
        }
        self.recomposite();
        Ok(())
    }

    /// Removes a collection from this composite, if present.
    pub fn remove_collection(&mut self, collection: &EntityCollection) -> bool {
        if let Some(pos) = self.index_of_collection(collection) {
            self.collections.remove(pos);
            self.recomposite();
            true
        } else {
            false
        }
    }

    /// Removes all collections from this composite.
    pub fn remove_all_collections(&mut self) {
        self.collections.clear();
        self.recomposite();
    }

    /// Raises a collection up one position in the composite.
    pub fn raise_collection(&mut self, collection: &EntityCollection) -> Result<(), DeveloperError> {
        let index = self
            .index_of_collection(collection)
            .ok_or_else(|| DeveloperError::new("collection is not in this composite."))?;
        if index + 1 < self.collections.len() {
            self.collections.swap(index, index + 1);
            self.recomposite();
        }
        Ok(())
    }

    /// Lowers a collection down one position in the composite.
    pub fn lower_collection(&mut self, collection: &EntityCollection) -> Result<(), DeveloperError> {
        let index = self
            .index_of_collection(collection)
            .ok_or_else(|| DeveloperError::new("collection is not in this composite."))?;
        if index > 0 {
            self.collections.swap(index, index - 1);
            self.recomposite();
        }
        Ok(())
    }

    /// Raises a collection to the top of the composite.
    pub fn raise_collection_to_top(
        &mut self,
        collection: &EntityCollection,
    ) -> Result<(), DeveloperError> {
        let index = self
            .index_of_collection(collection)
            .ok_or_else(|| DeveloperError::new("collection is not in this composite."))?;
        if index + 1 < self.collections.len() {
            let item = self.collections.remove(index);
            self.collections.push(item);
            self.recomposite();
        }
        Ok(())
    }

    /// Lowers a collection to the bottom of the composite.
    pub fn lower_collection_to_bottom(
        &mut self,
        collection: &EntityCollection,
    ) -> Result<(), DeveloperError> {
        let index = self
            .index_of_collection(collection)
            .ok_or_else(|| DeveloperError::new("collection is not in this composite."))?;
        if index > 0 {
            let item = self.collections.remove(index);
            self.collections.insert(0, item);
            self.recomposite();
        }
        Ok(())
    }

    /// Recomposites all collections into the single composite collection.
    pub fn recomposite(&mut self) {
        self.should_recomposite = true;
        if self.suspend_count != 0 {
            return;
        }

        self.composite.suspend_events();
        self.composite.remove_all();

        // Merge from last collection down to first so that later collections take precedence
        for collection in self.collections.iter().rev() {
            for entity in collection.values() {
                if !self.composite.contains_id(&entity.id) {
                    let mut new_entity = Entity::with_id(&entity.id);
                    new_entity.merge(entity);
                    let _ = self.composite.add(new_entity);
                } else if let Some(target) = self.composite.get_by_id_mut(&entity.id) {
                    target.merge(entity);
                }
            }
        }

        let _ = self.composite.resume_events();
        self.should_recomposite = false;
    }
}
