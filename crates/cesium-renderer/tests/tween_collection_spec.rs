use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use cesium_math::{Color, EasingFunction};
use cesium_renderer::tween_collection::{
    TweenAlphaOptions, TweenCollection, TweenMaterial, TweenOffsetOptions, TweenOptions,
    TweenPropertyOptions,
};

#[test]
fn test_add_adds_a_tween() {
    let mut start_object = HashMap::new();
    start_object.insert("value".to_string(), 0.0);
    let mut stop_object = HashMap::new();
    stop_object.insert("value".to_string(), 1.0);

    let updated = Rc::new(RefCell::new(false));
    let completed = Rc::new(RefCell::new(false));
    let cancelled = Rc::new(RefCell::new(false));

    let u_clone = updated.clone();
    let c_clone = completed.clone();
    let x_clone = cancelled.clone();

    let tweens = TweenCollection::new();
    let tween_rc = tweens
        .add(TweenOptions {
            start_object: start_object.clone(),
            stop_object: stop_object.clone(),
            duration: 1.0,
            delay: Some(0.5),
            easing_function: Some(EasingFunction::QuadraticIn),
            update: Some(Box::new(move |_| *u_clone.borrow_mut() = true)),
            complete: Some(Box::new(move || *c_clone.borrow_mut() = true)),
            cancel: Some(Box::new(move || *x_clone.borrow_mut() = true)),
            repeat: None,
        })
        .unwrap();

    let tween = tween_rc.borrow();
    assert_eq!(tween.start_object, start_object);
    assert_eq!(tween.stop_object, stop_object);
    assert_eq!(tween.duration, 1.0);
    assert_eq!(tween.delay, 0.5);
    assert_eq!(tween.easing_function, EasingFunction::QuadraticIn);
    assert!(tween.update.is_some());
    assert!(tween.complete.is_some());
    assert!(tween.cancel.is_some());
}

#[test]
fn test_add_adds_a_tween_with_defaults() {
    let mut start_object = HashMap::new();
    start_object.insert("value".to_string(), 0.0);
    let mut stop_object = HashMap::new();
    stop_object.insert("value".to_string(), 1.0);

    let tweens = TweenCollection::new();
    let tween_rc = tweens
        .add(TweenOptions {
            start_object: start_object.clone(),
            stop_object: stop_object.clone(),
            duration: 1.0,
            delay: None,
            easing_function: None,
            update: None,
            complete: None,
            cancel: None,
            repeat: None,
        })
        .unwrap();

    let tween = tween_rc.borrow();
    assert_eq!(tween.start_object, start_object);
    assert_eq!(tween.stop_object, stop_object);
    assert_eq!(tween.duration, 1.0);
    assert_eq!(tween.delay, 0.0);
    assert_eq!(tween.easing_function, EasingFunction::LinearNone);
    assert!(tween.update.is_none());
    assert!(tween.complete.is_none());
    assert!(tween.cancel.is_none());
}

#[test]
fn test_add_adds_with_duration_of_zero() {
    let completed = Rc::new(RefCell::new(false));
    let c_clone = completed.clone();

    let mut start_object = HashMap::new();
    start_object.insert("v".to_string(), 0.0);
    let mut stop_object = HashMap::new();
    stop_object.insert("v".to_string(), 1.0);

    let tweens = TweenCollection::new();
    let _tween = tweens
        .add(TweenOptions {
            start_object,
            stop_object,
            duration: 0.0,
            delay: None,
            easing_function: None,
            update: None,
            complete: Some(Box::new(move || *c_clone.borrow_mut() = true)),
            cancel: None,
            repeat: None,
        })
        .unwrap();

    assert_eq!(tweens.len(), 0);
    assert!(*completed.borrow());
}

#[test]
fn test_add_validations() {
    let tweens = TweenCollection::new();

    // Empty startObject
    assert!(tweens
        .add(TweenOptions {
            start_object: HashMap::new(),
            stop_object: [("v".to_string(), 1.0)].into_iter().collect(),
            duration: 1.0,
            delay: None,
            easing_function: None,
            update: None,
            complete: None,
            cancel: None,
            repeat: None,
        })
        .is_err());

    // Empty stopObject
    assert!(tweens
        .add(TweenOptions {
            start_object: [("v".to_string(), 0.0)].into_iter().collect(),
            stop_object: HashMap::new(),
            duration: 1.0,
            delay: None,
            easing_function: None,
            update: None,
            complete: None,
            cancel: None,
            repeat: None,
        })
        .is_err());

    // Negative duration
    assert!(tweens
        .add(TweenOptions {
            start_object: [("v".to_string(), 0.0)].into_iter().collect(),
            stop_object: [("v".to_string(), 1.0)].into_iter().collect(),
            duration: -1.0,
            delay: None,
            easing_function: None,
            update: None,
            complete: None,
            cancel: None,
            repeat: None,
        })
        .is_err());
}

#[test]
fn test_cancel_tween_and_remove() {
    let cancel_count = Rc::new(RefCell::new(0));
    let cc_clone = cancel_count.clone();

    let tweens = TweenCollection::new();
    let tween = tweens
        .add(TweenOptions {
            start_object: [("v".to_string(), 0.0)].into_iter().collect(),
            stop_object: [("v".to_string(), 1.0)].into_iter().collect(),
            duration: 1.0,
            delay: None,
            easing_function: None,
            update: None,
            complete: None,
            cancel: Some(Box::new(move || *cc_clone.borrow_mut() += 1)),
            repeat: None,
        })
        .unwrap();

    assert_eq!(tweens.len(), 1);
    assert!(tweens.contains(&tween));

    tween.borrow_mut().cancel_tween();
    assert_eq!(*cancel_count.borrow(), 1);
    assert_eq!(tweens.len(), 0);
    assert!(!tweens.contains(&tween));

    // remove already removed returns false
    assert!(!tweens.remove(&tween));
}

#[test]
fn test_remove_all() {
    let cancel_count = Rc::new(RefCell::new(0));
    let cc1 = cancel_count.clone();
    let cc2 = cancel_count.clone();

    let tweens = TweenCollection::new();
    tweens
        .add(TweenOptions {
            start_object: [("v".to_string(), 0.0)].into_iter().collect(),
            stop_object: [("v".to_string(), 1.0)].into_iter().collect(),
            duration: 1.0,
            delay: None,
            easing_function: None,
            update: None,
            complete: None,
            cancel: Some(Box::new(move || *cc1.borrow_mut() += 1)),
            repeat: None,
        })
        .unwrap();

    tweens
        .add(TweenOptions {
            start_object: [("v".to_string(), 0.0)].into_iter().collect(),
            stop_object: [("v".to_string(), 1.0)].into_iter().collect(),
            duration: 1.0,
            delay: None,
            easing_function: None,
            update: None,
            complete: None,
            cancel: Some(Box::new(move || *cc2.borrow_mut() += 1)),
            repeat: None,
        })
        .unwrap();

    assert_eq!(tweens.len(), 2);
    tweens.remove_all();
    assert_eq!(tweens.len(), 0);
    assert_eq!(*cancel_count.borrow(), 2);
}

#[test]
fn test_get() {
    let tweens = TweenCollection::new();
    let t0 = tweens
        .add(TweenOptions {
            start_object: [("v".to_string(), 0.0)].into_iter().collect(),
            stop_object: [("v".to_string(), 1.0)].into_iter().collect(),
            duration: 1.0,
            delay: None,
            easing_function: None,
            update: None,
            complete: None,
            cancel: None,
            repeat: None,
        })
        .unwrap();
    let t1 = tweens
        .add(TweenOptions {
            start_object: [("v".to_string(), 0.0)].into_iter().collect(),
            stop_object: [("v".to_string(), 1.0)].into_iter().collect(),
            duration: 1.0,
            delay: None,
            easing_function: None,
            update: None,
            complete: None,
            cancel: None,
            repeat: None,
        })
        .unwrap();

    assert_eq!(tweens.get(0).unwrap().borrow().id, t0.borrow().id);
    assert_eq!(tweens.get(1).unwrap().borrow().id, t1.borrow().id);
    assert!(tweens.get(2).is_err());
}

#[test]
fn test_update_animates_tween() {
    let last_val = Rc::new(RefCell::new(0.0));
    let completed = Rc::new(RefCell::new(false));

    let lv_clone = last_val.clone();
    let comp_clone = completed.clone();

    let tweens = TweenCollection::new();
    let tween = tweens
        .add(TweenOptions {
            start_object: [("value".to_string(), 0.0)].into_iter().collect(),
            stop_object: [("value".to_string(), 1.0)].into_iter().collect(),
            duration: 1.0,
            delay: None,
            easing_function: Some(EasingFunction::LinearNone),
            update: Some(Box::new(move |vals| {
                *lv_clone.borrow_mut() = *vals.get("value").unwrap();
            })),
            complete: Some(Box::new(move || *comp_clone.borrow_mut() = true)),
            cancel: None,
            repeat: None,
        })
        .unwrap();

    assert_eq!(tweens.len(), 1);

    tweens.update(0.0);
    assert_eq!(*last_val.borrow(), 0.0);
    assert_eq!(*tween.borrow().start_object.get("value").unwrap(), 0.0);

    tweens.update(0.5);
    assert_eq!(*last_val.borrow(), 0.5);
    assert_eq!(*tween.borrow().start_object.get("value").unwrap(), 0.5);

    tweens.update(1.0);
    assert_eq!(*last_val.borrow(), 1.0);
    assert!(*completed.borrow());
    assert_eq!(tweens.len(), 0);
}

#[test]
fn test_add_property() {
    let object = Rc::new(RefCell::new(HashMap::new()));
    object.borrow_mut().insert("property".to_string(), 0.0);

    let tweens = TweenCollection::new();
    tweens
        .add_property(TweenPropertyOptions {
            object: object.clone(),
            property: "property".to_string(),
            start_value: 0.0,
            stop_value: 1.0,
            duration: Some(1.0),
            delay: None,
            easing_function: None,
            update: None,
            complete: None,
            cancel: None,
            repeat: None,
        })
        .unwrap();

    tweens.update(0.0);
    tweens.update(0.5);
    assert_eq!(*object.borrow().get("property").unwrap(), 0.5);

    // Error on nonexistent property
    assert!(tweens
        .add_property(TweenPropertyOptions {
            object: object.clone(),
            property: "nonexistent".to_string(),
            start_value: 0.0,
            stop_value: 1.0,
            duration: Some(1.0),
            delay: None,
            easing_function: None,
            update: None,
            complete: None,
            cancel: None,
            repeat: None,
        })
        .is_err());
}

#[test]
fn test_add_alpha() {
    let mut uniforms = HashMap::new();
    uniforms.insert("lightColor".to_string(), Color::new(1.0, 1.0, 1.0, 0.0));
    uniforms.insert("darkColor".to_string(), Color::new(0.0, 0.0, 0.0, 0.0));

    let material = Rc::new(RefCell::new(TweenMaterial {
        color_uniforms: uniforms,
        offset: None,
    }));

    let tweens = TweenCollection::new();
    tweens
        .add_alpha(TweenAlphaOptions {
            material: material.clone(),
            start_value: Some(0.0),
            stop_value: Some(1.0),
            duration: Some(1.0),
            delay: None,
            easing_function: None,
            update: None,
            complete: None,
            cancel: None,
        })
        .unwrap();

    tweens.update(0.0);
    tweens.update(0.5);

    let mat = material.borrow();
    assert_eq!(mat.color_uniforms.get("lightColor").unwrap().alpha, 0.5);
    assert_eq!(mat.color_uniforms.get("darkColor").unwrap().alpha, 0.5);

    // Throws without color uniforms
    let empty_material = Rc::new(RefCell::new(TweenMaterial::default()));
    assert!(tweens
        .add_alpha(TweenAlphaOptions {
            material: empty_material,
            start_value: None,
            stop_value: None,
            duration: Some(1.0),
            delay: None,
            easing_function: None,
            update: None,
            complete: None,
            cancel: None,
        })
        .is_err());
}

#[test]
fn test_add_offset_increment() {
    let material = Rc::new(RefCell::new(TweenMaterial {
        color_uniforms: HashMap::new(),
        offset: Some(0.0),
    }));

    let tweens = TweenCollection::new();
    tweens
        .add_offset_increment(TweenOffsetOptions {
            material: material.clone(),
            duration: Some(1.0),
            delay: None,
            easing_function: None,
            update: None,
            cancel: None,
        })
        .unwrap();

    tweens.update(0.0);
    tweens.update(0.5);
    assert_eq!(material.borrow().offset, Some(0.5));

    // Throws without offset uniform
    let no_offset_material = Rc::new(RefCell::new(TweenMaterial::default()));
    assert!(tweens
        .add_offset_increment(TweenOffsetOptions {
            material: no_offset_material,
            duration: Some(1.0),
            delay: None,
            easing_function: None,
            update: None,
            cancel: None,
        })
        .is_err());
}
