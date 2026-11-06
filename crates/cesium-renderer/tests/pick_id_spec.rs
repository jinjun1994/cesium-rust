//! Direct port and verification against CesiumJS `PickIdSpec.js` (Apache-2.0).

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use cesium_math::Color;
use cesium_renderer::PickId;

#[derive(Clone, Debug, PartialEq)]
struct MockPickObject {
    key: Option<String>,
    message: Option<String>,
}

#[test]
fn test_creates_pick_ids() {
    let pick_objects = Rc::new(RefCell::new(HashMap::from([
        (
            1,
            MockPickObject {
                key: Some("a".to_string()),
                message: None,
            },
        ),
        (
            2,
            MockPickObject {
                key: Some("b".to_string()),
                message: None,
            },
        ),
        (
            3,
            MockPickObject {
                key: Some("c".to_string()),
                message: None,
            },
        ),
    ])));

    let _pick_ids = [
        PickId::new(Rc::clone(&pick_objects), 1, Color::from_rgba(1)),
        PickId::new(Rc::clone(&pick_objects), 2, Color::from_rgba(2)),
        PickId::new(Rc::clone(&pick_objects), 3, Color::from_rgba(3)),
    ];

    let objs = pick_objects.borrow();
    assert_eq!(objs.len(), 3);
    assert_eq!(objs.get(&1).unwrap().key.as_deref(), Some("a"));
    assert_eq!(objs.get(&2).unwrap().key.as_deref(), Some("b"));
    assert_eq!(objs.get(&3).unwrap().key.as_deref(), Some("c"));
}

#[test]
fn test_destroys_pick_ids() {
    let pick_objects = Rc::new(RefCell::new(HashMap::from([
        (
            1,
            MockPickObject {
                key: Some("a".to_string()),
                message: None,
            },
        ),
        (
            2,
            MockPickObject {
                key: Some("b".to_string()),
                message: None,
            },
        ),
        (
            3,
            MockPickObject {
                key: Some("c".to_string()),
                message: None,
            },
        ),
    ])));

    let pick_ids = [
        PickId::new(Rc::clone(&pick_objects), 1, Color::from_rgba(1)),
        PickId::new(Rc::clone(&pick_objects), 2, Color::from_rgba(2)),
        PickId::new(Rc::clone(&pick_objects), 3, Color::from_rgba(3)),
    ];

    pick_ids[1].destroy();
    assert_eq!(pick_objects.borrow().len(), 2);
    assert!(pick_objects.borrow().contains_key(&1));
    assert!(!pick_objects.borrow().contains_key(&2));
    assert!(pick_objects.borrow().contains_key(&3));

    pick_ids[0].destroy();
    assert_eq!(pick_objects.borrow().len(), 1);
    assert!(pick_objects.borrow().contains_key(&3));

    pick_ids[2].destroy();
    assert!(pick_objects.borrow().is_empty());
}

#[test]
fn test_gets_sets_pick_id_objects() {
    let pick_objects = Rc::new(RefCell::new(HashMap::from([
        (
            1,
            MockPickObject {
                key: Some("a".to_string()),
                message: None,
            },
        ),
        (
            2,
            MockPickObject {
                key: Some("b".to_string()),
                message: None,
            },
        ),
        (
            3,
            MockPickObject {
                key: Some("c".to_string()),
                message: None,
            },
        ),
    ])));

    let pick_ids = [
        PickId::new(Rc::clone(&pick_objects), 1, Color::from_rgba(1)),
        PickId::new(Rc::clone(&pick_objects), 2, Color::from_rgba(2)),
        PickId::new(Rc::clone(&pick_objects), 3, Color::from_rgba(3)),
    ];

    assert_eq!(
        pick_ids[0].object(),
        Some(MockPickObject {
            key: Some("a".to_string()),
            message: None
        })
    );
    assert_eq!(
        pick_ids[1].object(),
        Some(MockPickObject {
            key: Some("b".to_string()),
            message: None
        })
    );
    assert_eq!(
        pick_ids[2].object(),
        Some(MockPickObject {
            key: Some("c".to_string()),
            message: None
        })
    );

    pick_ids[1].set_object(MockPickObject {
        key: None,
        message: Some("hello world".to_string()),
    });

    assert_eq!(
        pick_ids[0].object(),
        Some(MockPickObject {
            key: Some("a".to_string()),
            message: None
        })
    );
    assert_eq!(
        pick_ids[1].object(),
        Some(MockPickObject {
            key: None,
            message: Some("hello world".to_string())
        })
    );
    assert_eq!(
        pick_ids[2].object(),
        Some(MockPickObject {
            key: Some("c".to_string()),
            message: None
        })
    );
}
