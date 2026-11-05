use std::cell::RefCell;
use std::rc::Rc;

/// A token representing a registered listener with an Event.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ListenerId(usize);

type ListenerCallback<T> = Rc<dyn Fn(&T)>;

#[derive(Clone)]
struct ListenerEntry<T> {
    id: ListenerId,
    callback: ListenerCallback<T>,
}

struct EventInner<T> {
    listeners: Vec<ListenerEntry<T>>,
    to_add: Vec<ListenerEntry<T>>,
    to_remove: Vec<ListenerId>,
    invoking: bool,
    next_id: usize,
}

/// A generic utility class for managing subscribers for a particular event.
#[derive(Clone)]
pub struct Event<T = ()> {
    inner: Rc<RefCell<EventInner<T>>>,
}

impl<T> std::fmt::Debug for Event<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Event(...)")
    }
}

impl<T: 'static> Default for Event<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: 'static> Event<T> {
    /// Creates a new empty Event.
    pub fn new() -> Self {
        Self {
            inner: Rc::new(RefCell::new(EventInner {
                listeners: Vec::new(),
                to_add: Vec::new(),
                to_remove: Vec::new(),
                invoking: false,
                next_id: 1,
            })),
        }
    }

    /// Number of listeners currently registered.
    pub fn number_of_listeners(&self) -> usize {
        let inner = self.inner.borrow();
        let current_count = inner.listeners.len() + inner.to_add.len();
        let remove_count = inner.to_remove.len();
        current_count.saturating_sub(remove_count)
    }

    /// Registers a callback function to be executed whenever the event is raised.
    /// Returns a unique `ListenerId` for subsequent unregistration.
    pub fn add_listener<F>(&self, callback: F) -> ListenerId
    where
        F: Fn(&T) + 'static,
    {
        let mut inner = self.inner.borrow_mut();
        let id = ListenerId(inner.next_id);
        inner.next_id += 1;

        let entry = ListenerEntry {
            id,
            callback: Rc::new(callback),
        };

        if inner.invoking {
            inner.to_add.push(entry);
        } else {
            inner.listeners.push(entry);
        }

        id
    }

    /// Unregisters a previously registered callback.
    /// Returns true if the listener was removed, false otherwise.
    pub fn remove_listener(&self, id: ListenerId) -> bool {
        let mut inner = self.inner.borrow_mut();

        if inner.to_remove.contains(&id) {
            return false;
        }

        let in_listeners = inner.listeners.iter().any(|e| e.id == id);
        let in_to_add = inner.to_add.iter().any(|e| e.id == id);

        if !in_listeners && !in_to_add {
            return false;
        }

        if inner.invoking {
            inner.to_remove.push(id);
        } else {
            inner.listeners.retain(|e| e.id != id);
            inner.to_add.retain(|e| e.id != id);
        }

        true
    }

    /// Raises the event by calling each registered listener with the supplied argument.
    pub fn raise_event(&self, arg: &T) {
        {
            let mut inner = self.inner.borrow_mut();
            inner.invoking = true;
        }

        let listener_count = self.inner.borrow().listeners.len();
        for i in 0..listener_count {
            let callback_to_run = {
                let inner = self.inner.borrow();
                if let Some(entry) = inner.listeners.get(i) {
                    if inner.to_remove.contains(&entry.id) {
                        None
                    } else {
                        Some(Rc::clone(&entry.callback))
                    }
                } else {
                    None
                }
            }; // inner borrow drops here!

            if let Some(cb) = callback_to_run {
                cb(arg);
            }
        }

        let mut inner = self.inner.borrow_mut();
        inner.invoking = false;

        // Apply removals
        let removals = std::mem::take(&mut inner.to_remove);
        for id in removals {
            inner.listeners.retain(|e| e.id != id);
            inner.to_add.retain(|e| e.id != id);
        }

        // Apply additions
        let mut additions = std::mem::take(&mut inner.to_add);
        inner.listeners.append(&mut additions);
    }
}
