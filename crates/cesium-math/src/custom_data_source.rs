//! A DataSource implementation which can be used to manually manage a group of entities.
//! Direct port of CesiumJS `DataSources/CustomDataSource.js`.

use crate::{create_guid, DataSourceClock, EntityCollection, Event};

/// A DataSource implementation which can be used to manually manage a group of entities.
#[derive(Clone)]
pub struct CustomDataSource {
    id: String,
    name: String,
    clock: Option<DataSourceClock>,
    is_loading: bool,
    entities: EntityCollection,
    pub changed_event: Event<()>,
    pub loading_event: Event<bool>,
    pub error_event: Event<String>,
}

impl Default for CustomDataSource {
    fn default() -> Self {
        Self::new("default")
    }
}

impl PartialEq for CustomDataSource {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl CustomDataSource {
    /// Creates a new CustomDataSource with a given name.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            id: create_guid(),
            name: name.into(),
            clock: None,
            is_loading: false,
            entities: EntityCollection::new(),
            changed_event: Event::new(),
            loading_event: Event::new(),
            error_event: Event::new(),
        }
    }

    /// Gets the unique ID of this data source.
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Gets the human-readable name of this instance.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Sets the human-readable name of this instance.
    pub fn set_name(&mut self, name: impl Into<String>) {
        let name_str = name.into();
        if self.name != name_str {
            self.name = name_str;
            self.changed_event.raise_event(&());
        }
    }

    /// Gets the desired clock settings for this data source.
    pub fn clock(&self) -> Option<&DataSourceClock> {
        self.clock.as_ref()
    }

    /// Sets the desired clock settings for this data source.
    pub fn set_clock(&mut self, clock: Option<DataSourceClock>) {
        if self.clock != clock {
            self.clock = clock;
            self.changed_event.raise_event(&());
        }
    }

    /// Gets the entity collection managed by this data source.
    pub fn entities(&self) -> &EntityCollection {
        &self.entities
    }

    /// Gets the mutable entity collection managed by this data source.
    pub fn entities_mut(&mut self) -> &mut EntityCollection {
        &mut self.entities
    }

    /// Gets a value indicating if the data source is currently loading data.
    pub fn is_loading(&self) -> bool {
        self.is_loading
    }

    /// Sets whether the data source is currently loading data.
    pub fn set_loading(&mut self, loading: bool) {
        if self.is_loading != loading {
            self.is_loading = loading;
            self.loading_event.raise_event(&loading);
        }
    }

    /// Gets whether or not this data source should be displayed.
    pub fn show(&self) -> bool {
        self.entities.show()
    }

    /// Sets whether or not this data source should be displayed.
    pub fn set_show(&mut self, show: bool) {
        self.entities.set_show(show);
    }
}
