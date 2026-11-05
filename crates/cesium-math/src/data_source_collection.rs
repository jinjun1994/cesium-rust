//! A collection of DataSource instances.
//! Direct port of CesiumJS `DataSources/DataSourceCollection.js`.

use crate::{CustomDataSource, DeveloperError, Event};

/// A collection of DataSource instances.
#[derive(Clone, Default)]
pub struct DataSourceCollection {
    data_sources: Vec<CustomDataSource>,
    pub data_source_added: Event<CustomDataSource>,
    pub data_source_removed: Event<CustomDataSource>,
    pub data_source_moved: Event<(CustomDataSource, usize, usize)>,
}

impl DataSourceCollection {
    /// Creates a new empty DataSourceCollection.
    pub fn new() -> Self {
        Self {
            data_sources: Vec::new(),
            data_source_added: Event::new(),
            data_source_removed: Event::new(),
            data_source_moved: Event::new(),
        }
    }

    /// Gets the number of data sources in this collection.
    pub fn len(&self) -> usize {
        self.data_sources.len()
    }

    /// Returns true if the collection is empty.
    pub fn is_empty(&self) -> bool {
        self.data_sources.is_empty()
    }

    /// Checks to see if the collection contains a given data source.
    pub fn contains(&self, data_source: &CustomDataSource) -> bool {
        self.data_sources.iter().any(|ds| ds.id() == data_source.id())
    }

    /// Checks to see if the collection contains a data source with the given ID.
    pub fn contains_id(&self, id: &str) -> bool {
        self.data_sources.iter().any(|ds| ds.id() == id)
    }

    /// Determines the index of a given data source in the collection.
    pub fn index_of(&self, data_source: &CustomDataSource) -> Option<usize> {
        self.data_sources
            .iter()
            .position(|ds| ds.id() == data_source.id())
    }

    /// Gets a data source by index from the collection.
    pub fn get(&self, index: usize) -> Option<&CustomDataSource> {
        self.data_sources.get(index)
    }

    /// Gets a mutable data source by index from the collection.
    pub fn get_mut(&mut self, index: usize) -> Option<&mut CustomDataSource> {
        self.data_sources.get_mut(index)
    }

    /// Gets data sources by name from the collection.
    pub fn get_by_name(&self, name: &str) -> Vec<&CustomDataSource> {
        self.data_sources
            .iter()
            .filter(|ds| ds.name() == name)
            .collect()
    }

    /// Adds a data source to the collection.
    pub fn add(&mut self, data_source: CustomDataSource) -> &mut CustomDataSource {
        let clone_for_event = data_source.clone();
        self.data_sources.push(data_source);
        self.data_source_added.raise_event(&clone_for_event);
        self.data_sources.last_mut().unwrap()
    }

    /// Removes a data source from this collection, if present.
    pub fn remove(&mut self, data_source: &CustomDataSource) -> bool {
        self.remove_by_id(data_source.id())
    }

    /// Removes a data source with the provided ID from this collection.
    pub fn remove_by_id(&mut self, id: &str) -> bool {
        if let Some(pos) = self.data_sources.iter().position(|ds| ds.id() == id) {
            let removed = self.data_sources.remove(pos);
            self.data_source_removed.raise_event(&removed);
            true
        } else {
            false
        }
    }

    /// Removes all data sources from this collection.
    pub fn remove_all(&mut self) {
        let removed_list: Vec<CustomDataSource> = std::mem::take(&mut self.data_sources);
        for item in removed_list {
            self.data_source_removed.raise_event(&item);
        }
    }

    /// Raises a data source up one position in the collection.
    pub fn raise(&mut self, data_source: &CustomDataSource) -> Result<(), DeveloperError> {
        let index = self
            .index_of(data_source)
            .ok_or_else(|| DeveloperError::new("dataSource is not in this collection."))?;
        if index + 1 < self.data_sources.len() {
            self.data_sources.swap(index, index + 1);
            let ds = self.data_sources[index + 1].clone();
            self.data_source_moved.raise_event(&(ds, index + 1, index));
        }
        Ok(())
    }

    /// Lowers a data source down one position in the collection.
    pub fn lower(&mut self, data_source: &CustomDataSource) -> Result<(), DeveloperError> {
        let index = self
            .index_of(data_source)
            .ok_or_else(|| DeveloperError::new("dataSource is not in this collection."))?;
        if index > 0 {
            self.data_sources.swap(index, index - 1);
            let ds = self.data_sources[index - 1].clone();
            self.data_source_moved.raise_event(&(ds, index - 1, index));
        }
        Ok(())
    }

    /// Raises a data source to the top of the collection.
    pub fn raise_to_top(&mut self, data_source: &CustomDataSource) -> Result<(), DeveloperError> {
        let index = self
            .index_of(data_source)
            .ok_or_else(|| DeveloperError::new("dataSource is not in this collection."))?;
        if index + 1 < self.data_sources.len() {
            let item = self.data_sources.remove(index);
            self.data_sources.push(item);
            let new_index = self.data_sources.len() - 1;
            let ds = self.data_sources[new_index].clone();
            self.data_source_moved.raise_event(&(ds, new_index, index));
        }
        Ok(())
    }

    /// Lowers a data source to the bottom of the collection.
    pub fn lower_to_bottom(&mut self, data_source: &CustomDataSource) -> Result<(), DeveloperError> {
        let index = self
            .index_of(data_source)
            .ok_or_else(|| DeveloperError::new("dataSource is not in this collection."))?;
        if index > 0 {
            let item = self.data_sources.remove(index);
            self.data_sources.insert(0, item);
            let ds = self.data_sources[0].clone();
            self.data_source_moved.raise_event(&(ds, 0, index));
        }
        Ok(())
    }
}
