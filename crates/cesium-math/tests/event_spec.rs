use cesium_math::Event;
use std::cell::RefCell;
use std::rc::Rc;

#[test]
fn test_event_basic_add_raise_remove() {
    let event = Event::<i32>::new();
    let called = Rc::new(RefCell::new(Vec::new()));

    let called_clone = Rc::clone(&called);
    let id = event.add_listener(move |val| {
        called_clone.borrow_mut().push(*val);
    });

    event.raise_event(&123);
    assert_eq!(*called.borrow(), vec![123]);

    called.borrow_mut().clear();
    assert!(event.remove_listener(id));

    event.raise_event(&456);
    assert!(called.borrow().is_empty());
}

#[test]
fn test_can_remove_from_within_a_callback() {
    let event = Event::<()>::new();
    let event_clone = event.clone();

    let remove_id = Rc::new(RefCell::new(None));
    let remove_id_clone = Rc::clone(&remove_id);

    let id1 = event.add_listener(|_| {});
    let id2 = event.add_listener(move |_| {
        if let Some(target) = *remove_id_clone.borrow() {
            event_clone.remove_listener(target);
        }
    });
    *remove_id.borrow_mut() = Some(id2);
    let id3 = event.add_listener(|_| {});

    assert_eq!(event.number_of_listeners(), 3);
    event.raise_event(&());
    assert_eq!(event.number_of_listeners(), 2);

    assert!(event.remove_listener(id1));
    assert!(event.remove_listener(id3));
    assert_eq!(event.number_of_listeners(), 0);
}

#[test]
fn test_can_add_a_listener_from_within_a_callback() {
    let event = Event::<()>::new();
    let event_clone = event.clone();

    let added_called = Rc::new(RefCell::new(false));
    let added_called_clone = Rc::clone(&added_called);

    event.add_listener(move |_| {
        let ac = Rc::clone(&added_called_clone);
        event_clone.add_listener(move |_| {
            *ac.borrow_mut() = true;
        });
    });

    assert_eq!(event.number_of_listeners(), 1);
    event.raise_event(&());
    // The added listener was deferred to next raise
    assert_eq!(event.number_of_listeners(), 2);
    assert!(!*added_called.borrow());

    event.raise_event(&());
    assert!(*added_called.borrow());
}

#[test]
fn test_can_remove_multiple_listeners_within_callback() {
    let event = Event::<()>::new();
    let mut ids = Vec::new();

    for _ in 0..10 {
        let ev = event.clone();
        let target_id = Rc::new(RefCell::new(None));
        let tid = Rc::clone(&target_id);
        let id = event.add_listener(move |_| {
            if let Some(i) = *tid.borrow() {
                ev.remove_listener(i);
            }
        });
        *target_id.borrow_mut() = Some(id);
        ids.push(id);
    }

    assert_eq!(event.number_of_listeners(), 10);
    event.raise_event(&());
    assert_eq!(event.number_of_listeners(), 0);
}

#[test]
fn test_number_of_listeners_and_remove_indicator() {
    let event = Event::<()>::new();
    assert_eq!(event.number_of_listeners(), 0);

    let id1 = event.add_listener(|_| {});
    assert_eq!(event.number_of_listeners(), 1);

    let id2 = event.add_listener(|_| {});
    assert_eq!(event.number_of_listeners(), 2);

    assert!(event.remove_listener(id2));
    assert_eq!(event.number_of_listeners(), 1);

    // Removing again returns false
    assert!(!event.remove_listener(id2));
    assert_eq!(event.number_of_listeners(), 1);

    assert!(event.remove_listener(id1));
    assert_eq!(event.number_of_listeners(), 0);
}

#[test]
fn test_works_with_no_listeners() {
    let event = Event::<i32>::new();
    event.raise_event(&123);
    assert_eq!(event.number_of_listeners(), 0);
}
