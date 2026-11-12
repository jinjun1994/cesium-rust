//! Knockout compatibility layer matching CesiumJS `Widgets/ThirdParty/knockout.js` (Apache-2.0 / MIT).

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::{Rc, Weak};

/// A subscription token used to unsubscribe.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct SubscriptionToken(pub usize);

type ObservableListener<T> = Rc<dyn Fn(&T)>;

/// Generic observable wrapper with change notifications.
pub struct KnockoutObservable<T> {
    value: RefCell<T>,
    listeners: RefCell<HashMap<usize, ObservableListener<T>>>,
    next_id: RefCell<usize>,
}

impl<T: Clone> KnockoutObservable<T> {
    /// Creates a new observable with the initial value.
    pub fn new(initial: T) -> Self {
        Self {
            value: RefCell::new(initial),
            listeners: RefCell::new(HashMap::new()),
            next_id: RefCell::new(1),
        }
    }

    /// Gets the current value.
    pub fn get(&self) -> T {
        self.value.borrow().clone()
    }

    /// Sets a new value and notifies subscribers if changed.
    pub fn set(&self, new_value: T) {
        *self.value.borrow_mut() = new_value;
        let val = self.value.borrow().clone();
        let listeners: Vec<ObservableListener<T>> = self.listeners.borrow().values().cloned().collect();
        for listener in listeners {
            listener(&val);
        }
    }

    /// Subscribes to value changes.
    pub fn subscribe<F>(&self, callback: F) -> SubscriptionToken
    where
        F: Fn(&T) + 'static,
    {
        let id = *self.next_id.borrow();
        *self.next_id.borrow_mut() += 1;
        self.listeners.borrow_mut().insert(id, Rc::new(callback));
        SubscriptionToken(id)
    }

    /// Unsubscribes a listener by token.
    pub fn unsubscribe(&self, token: SubscriptionToken) {
        self.listeners.borrow_mut().remove(&token.0);
    }
}

/// Dynamic Knockout value representation.
#[derive(Clone)]
pub enum KnockoutValue {
    Undefined,
    Null,
    Bool(bool),
    Number(f64),
    String(String),
    Observable(Rc<KnockoutObservable<KnockoutValue>>),
    Object(Rc<RefCell<HashMap<String, KnockoutValue>>>),
}

impl std::fmt::Debug for KnockoutValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Undefined => write!(f, "Undefined"),
            Self::Null => write!(f, "Null"),
            Self::Bool(b) => write!(f, "Bool({b})"),
            Self::Number(n) => write!(f, "Number({n})"),
            Self::String(s) => write!(f, "String({s:?})"),
            Self::Observable(obs) => write!(f, "Observable({:?})", obs.get()),
            Self::Object(m) => write!(f, "Object({:?})", m.borrow()),
        }
    }
}

impl KnockoutValue {
    /// Unwraps observable value if wrapped.
    pub fn unwrap(&self) -> KnockoutValue {
        match self {
            KnockoutValue::Observable(obs) => obs.get().unwrap(),
            other => other.clone(),
        }
    }

    /// Converts to string if possible.
    pub fn as_str(&self) -> Option<String> {
        match self.unwrap() {
            KnockoutValue::String(s) => Some(s),
            KnockoutValue::Number(n) => Some(if n.fract() == 0.0 {
                format!("{}", n as i64)
            } else {
                format!("{n}")
            }),
            KnockoutValue::Bool(b) => Some(b.to_string()),
            _ => None,
        }
    }
}

/// An object whose properties can be tracked by Knockout.
pub struct TrackableObject {
    pub properties: RefCell<HashMap<String, KnockoutValue>>,
    pub tracked: RefCell<HashMap<String, Rc<KnockoutObservable<KnockoutValue>>>>,
}

impl Default for TrackableObject {
    fn default() -> Self {
        Self::new()
    }
}

impl TrackableObject {
    /// Creates a new trackable object.
    pub fn new() -> Self {
        Self {
            properties: RefCell::new(HashMap::new()),
            tracked: RefCell::new(HashMap::new()),
        }
    }

    /// Sets a property on the object.
    pub fn set_property(&self, name: impl Into<String>, value: KnockoutValue) {
        let name = name.into();
        if let Some(obs) = self.tracked.borrow().get(&name) {
            obs.set(value.clone());
        }
        self.properties.borrow_mut().insert(name, value);
    }

    /// Gets a property value.
    pub fn get_property(&self, name: &str) -> Option<KnockoutValue> {
        self.properties.borrow().get(name).cloned()
    }
}

/// Simulated DOM element for Knockout template binding and testing.
pub struct KnockoutElement {
    pub tag_name: String,
    pub attributes: RefCell<HashMap<String, String>>,
    pub children: RefCell<Vec<Rc<RefCell<KnockoutElement>>>>,
    pub parent: RefCell<Option<Weak<RefCell<KnockoutElement>>>>,
    pub cleanups: RefCell<Vec<Box<dyn Fn()>>>,
}

impl KnockoutElement {
    /// Creates a new element.
    pub fn new(tag_name: impl Into<String>) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Self {
            tag_name: tag_name.into(),
            attributes: RefCell::new(HashMap::new()),
            children: RefCell::new(Vec::new()),
            parent: RefCell::new(None),
            cleanups: RefCell::new(Vec::new()),
        }))
    }

    /// Sets an attribute.
    pub fn set_attribute(&self, name: impl Into<String>, value: impl Into<String>) {
        self.attributes.borrow_mut().insert(name.into(), value.into());
    }

    /// Gets an attribute.
    pub fn get_attribute(&self, name: &str) -> Option<String> {
        self.attributes.borrow().get(name).cloned()
    }

    /// Appends a child element.
    pub fn append_child(parent_rc: &Rc<RefCell<Self>>, child_rc: &Rc<RefCell<Self>>) {
        child_rc.borrow_mut().parent = RefCell::new(Some(Rc::downgrade(parent_rc)));
        parent_rc.borrow_mut().children.borrow_mut().push(Rc::clone(child_rc));
    }

    /// Removes a child element.
    pub fn remove_child(parent_rc: &Rc<RefCell<Self>>, child_rc: &Rc<RefCell<Self>>) {
        let parent = parent_rc.borrow_mut();
        let mut children = parent.children.borrow_mut();
        if let Some(pos) = children.iter().position(|c| Rc::ptr_eq(c, child_rc)) {
            let removed = children.remove(pos);
            removed.borrow_mut().parent = RefCell::new(None);
        }
    }

    /// Clears children.
    pub fn clear_children(&self) {
        for child in self.children.borrow_mut().drain(..) {
            child.borrow_mut().parent = RefCell::new(None);
        }
    }

    /// Searches descendant elements matching basic CSS selector.
    pub fn query_selector(
        element: &Rc<RefCell<Self>>,
        selector: &str,
    ) -> Option<Rc<RefCell<Self>>> {
        let selector = selector.trim();

        // 1. "svg > path"
        if let Some((parent_sel, child_sel)) = selector.split_once('>') {
            let p_sel = parent_sel.trim();
            let c_sel = child_sel.trim();
            if let Some(matched_parent) = Self::query_selector(element, p_sel) {
                for child in matched_parent.borrow().children.borrow().iter() {
                    if matches_selector(child, c_sel) {
                        return Some(Rc::clone(child));
                    }
                }
            }
            return None;
        }

        // Direct check
        for child in element.borrow().children.borrow().iter() {
            if matches_selector(child, selector) {
                return Some(Rc::clone(child));
            }
            if let Some(sub) = Self::query_selector(child, selector) {
                return Some(sub);
            }
        }
        None
    }
}

fn matches_selector(el: &Rc<RefCell<KnockoutElement>>, selector: &str) -> bool {
    let selector = selector.trim();
    if selector.contains('.') {
        let parts: Vec<&str> = selector.split('.').collect();
        let tag = parts[0];
        let class = parts[1];
        if !tag.is_empty() && !el.borrow().tag_name.eq_ignore_ascii_case(tag) {
            return false;
        }
        if let Some(classes) = el.borrow().get_attribute("class") {
            return classes.split_whitespace().any(|c| c == class);
        }
        return false;
    }

    el.borrow().tag_name.eq_ignore_ascii_case(selector)
}

/// Knockout helper functions.
pub struct Knockout;

impl Knockout {
    /// Creates a new observable value.
    pub fn observable(val: KnockoutValue) -> Rc<KnockoutObservable<KnockoutValue>> {
        Rc::new(KnockoutObservable::new(val))
    }

    /// Tracks all properties of an object.
    pub fn track(obj: &Rc<RefCell<TrackableObject>>) {
        let properties = obj.borrow().properties.borrow().clone();
        let binding = obj.borrow();
        let mut tracked = binding.tracked.borrow_mut();

        for (k, v) in properties {
            tracked
                .entry(k)
                .or_insert_with(|| Rc::new(KnockoutObservable::new(v)));
        }
    }

    /// Gets the observable for a property if tracked.
    pub fn get_observable(
        obj: &Rc<RefCell<TrackableObject>>,
        prop_name: &str,
    ) -> Option<Rc<KnockoutObservable<KnockoutValue>>> {
        obj.borrow().tracked.borrow().get(prop_name).cloned()
    }

    /// Cleans an element by running registered cleanups and removing children.
    pub fn clean_node(element: &Rc<RefCell<KnockoutElement>>) {
        let cleanups: Vec<Box<dyn Fn()>> =
            element.borrow_mut().cleanups.borrow_mut().drain(..).collect();
        for cleanup in cleanups {
            cleanup();
        }
        element.borrow().clear_children();
    }
}
