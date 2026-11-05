//! PropertyBag implementation matching CesiumJS `DataSources/PropertyBag.js` (Apache-2.0).

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::{DynProperty, Event, EventHelper, JulianDate};

pub type PropertyBagEntry<T> = (String, Option<Rc<dyn DynProperty<T>>>);

/// A Property whose value is a key-value mapping of property names to the computed value of other properties.
#[derive(Clone)]
pub struct PropertyBag<T: Clone + PartialEq + 'static> {
    property_names: Vec<String>,
    properties: HashMap<String, Option<Rc<dyn DynProperty<T>>>>,
    definition_changed: Event<()>,
    event_helper: Rc<RefCell<EventHelper>>,
}

impl<T: Clone + PartialEq + 'static> Default for PropertyBag<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Clone + PartialEq + 'static> PropertyBag<T> {
    pub fn new() -> Self {
        Self {
            property_names: Vec::new(),
            properties: HashMap::new(),
            definition_changed: Event::new(),
            event_helper: Rc::new(RefCell::new(EventHelper::new())),
        }
    }

    pub fn from_entries(entries: Vec<PropertyBagEntry<T>>) -> Self {
        let mut bag = Self::new();
        for (name, prop) in entries {
            bag.add_property(&name, prop);
        }
        bag
    }

    pub fn property_names(&self) -> &[String] {
        &self.property_names
    }

    pub fn has_property(&self, name: &str) -> bool {
        self.properties.contains_key(name)
    }

    pub fn get_property(&self, name: &str) -> Option<Rc<dyn DynProperty<T>>> {
        self.properties.get(name).and_then(|opt| opt.clone())
    }

    pub fn is_constant(&self) -> bool {
        for prop in self.properties.values().flatten() {
            if !prop.is_constant() {
                return false;
            }
        }
        true
    }

    pub fn definition_changed(&self) -> &Event<()> {
        &self.definition_changed
    }

    pub fn add_property(&mut self, name: &str, value: Option<Rc<dyn DynProperty<T>>>) {
        if self.has_property(name) {
            panic!("{name} is already a registered property.");
        }

        self.property_names.push(name.to_string());
        if let Some(ref prop) = value {
            let dc = self.definition_changed.clone();
            self.event_helper.borrow_mut().add(prop.definition_changed(), {
                let ev = dc.clone();
                move |_| ev.raise_event(&())
            });
        }
        self.properties.insert(name.to_string(), value);
        self.definition_changed.raise_event(&());
    }

    pub fn set_property_value(&mut self, name: &str, value: Option<Rc<dyn DynProperty<T>>>) {
        if !self.has_property(name) {
            panic!("{name} is not a registered property.");
        }

        if let Some(ref prop) = value {
            let dc = self.definition_changed.clone();
            self.event_helper.borrow_mut().add(prop.definition_changed(), {
                let ev = dc.clone();
                move |_| ev.raise_event(&())
            });
        }
        self.properties.insert(name.to_string(), value);
        self.definition_changed.raise_event(&());
    }

    pub fn remove_property(&mut self, name: &str) {
        let index = self
            .property_names
            .iter()
            .position(|n| n == name)
            .unwrap_or_else(|| panic!("{name} is not a registered property."));

        self.property_names.remove(index);
        self.properties.remove(name);
        self.definition_changed.raise_event(&());
    }

    pub fn get_value(&self, time: Option<&JulianDate>) -> HashMap<String, Option<T>> {
        let default_time;
        let t = match time {
            Some(time) => time,
            None => {
                default_time = JulianDate::now();
                &default_time
            }
        };

        let mut result = HashMap::with_capacity(self.property_names.len());
        for name in &self.property_names {
            let val = match self.properties.get(name) {
                Some(Some(prop)) => prop.get_value(Some(t)),
                _ => None,
            };
            result.insert(name.clone(), val);
        }
        result
    }

    pub fn equals(&self, other: Option<&Self>) -> bool {
        let other = match other {
            Some(o) => o,
            None => return false,
        };

        if self.property_names.len() != other.property_names.len() {
            return false;
        }

        for name in &self.property_names {
            if !other.has_property(name) {
                return false;
            }
            let a = self.properties.get(name).unwrap();
            let b = other.properties.get(name).unwrap();
            match (a, b) {
                (None, None) => {}
                (Some(pa), Some(pb)) => {
                    if !pa.equals_dyn(&**pb) {
                        return false;
                    }
                }
                _ => return false,
            }
        }
        true
    }
}
