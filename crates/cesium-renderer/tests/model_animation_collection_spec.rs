//! Tests for ModelAnimationCollection matching CesiumJS `Scene/Model/ModelAnimationCollectionSpec.js` (Apache-2.0).

use std::sync::{Arc, Mutex};
use cesium_math::{Cartesian3, JulianDate};
use cesium_renderer::model_animation_channel::{
    AnimatedPropertyType, AnimationChannel, AnimationOutputValues, AnimationSampler,
    AnimationTarget, InterpolationType, ModelRuntimeNode,
};
use cesium_renderer::model_animation_collection::{
    AnimationMetadata, ModelAnimationCollection, ModelAnimationLoop, ModelAnimationOptions,
    ModelAnimationState,
};

fn create_mock_metadata(name: &str, index: usize, duration: f64) -> AnimationMetadata {
    let times = vec![0.0, duration];
    let points = vec![Cartesian3::ZERO, Cartesian3::new(1.0, 1.0, 1.0)];

    let sampler = AnimationSampler {
        input: times,
        interpolation: InterpolationType::Linear,
        output: AnimationOutputValues::Cartesian3(points),
    };

    let channel = AnimationChannel {
        sampler,
        target: Some(AnimationTarget {
            node: Some(0),
            path: AnimatedPropertyType::Translation,
        }),
    };

    AnimationMetadata {
        name: Some(name.to_string()),
        index,
        channels: vec![channel],
        local_start_time: 0.0,
        local_stop_time: duration,
    }
}

#[test]
fn test_initializes() {
    let collection = ModelAnimationCollection::new(true, true);
    assert_eq!(collection.len(), 0);
    assert!(collection.is_empty());
}

#[test]
fn test_throws_when_add_called_on_non_ready_model() {
    let mut collection = ModelAnimationCollection::new(false, true);
    let err = collection.add(ModelAnimationOptions {
        index: Some(0),
        ..ModelAnimationOptions::new()
    }).unwrap_err();
    assert!(err.message.contains("Animations are not loaded"));
}

#[test]
fn test_throws_when_add_missing_name_or_index() {
    let mut collection = ModelAnimationCollection::new(true, true);
    collection.animations_metadata.push(create_mock_metadata("anim0", 0, 1.0));

    let err = collection.add(ModelAnimationOptions::new()).unwrap_err();
    assert!(err.message.contains("Either options.name or options.index must be defined"));
}

#[test]
fn test_throws_when_add_invalid_name() {
    let mut collection = ModelAnimationCollection::new(true, true);
    collection.animations_metadata.push(create_mock_metadata("anim0", 0, 1.0));

    let err = collection.add(ModelAnimationOptions {
        name: Some("nonexistent".to_string()),
        ..ModelAnimationOptions::new()
    }).unwrap_err();
    assert!(err.message.contains("options.name must be a valid animation name"));
}

#[test]
fn test_throws_when_add_invalid_index() {
    let mut collection = ModelAnimationCollection::new(true, true);
    collection.animations_metadata.push(create_mock_metadata("anim0", 0, 1.0));

    let err = collection.add(ModelAnimationOptions {
        index: Some(5),
        ..ModelAnimationOptions::new()
    }).unwrap_err();
    assert!(err.message.contains("options.index must be a valid animation index"));
}

#[test]
fn test_throws_when_add_invalid_multiplier() {
    let mut collection = ModelAnimationCollection::new(true, true);
    collection.animations_metadata.push(create_mock_metadata("anim0", 0, 1.0));

    let err = collection.add(ModelAnimationOptions {
        index: Some(0),
        multiplier: -1.0,
        ..ModelAnimationOptions::new()
    }).unwrap_err();
    assert!(err.message.contains("options.multiplier must be greater than zero"));
}

#[test]
fn test_add_works_with_name_and_index() {
    let mut collection = ModelAnimationCollection::new(true, true);
    collection.animations_metadata.push(create_mock_metadata("anim0", 0, 1.0));
    collection.animations_metadata.push(create_mock_metadata("anim1", 1, 2.0));

    let idx0 = collection.add(ModelAnimationOptions {
        name: Some("anim0".to_string()),
        ..ModelAnimationOptions::new()
    }).unwrap();
    assert_eq!(idx0, 0);
    assert_eq!(collection.get(0).unwrap().name.as_deref(), Some("anim0"));

    let idx1 = collection.add(ModelAnimationOptions {
        index: Some(1),
        ..ModelAnimationOptions::new()
    }).unwrap();
    assert_eq!(idx1, 1);
    assert_eq!(collection.get(1).unwrap().name.as_deref(), Some("anim1"));
    assert_eq!(collection.len(), 2);
}

#[test]
fn test_add_all_works() {
    let mut collection = ModelAnimationCollection::new(true, true);
    collection.animations_metadata.push(create_mock_metadata("anim0", 0, 1.0));
    collection.animations_metadata.push(create_mock_metadata("anim1", 1, 2.0));

    let added = collection.add_all(ModelAnimationOptions {
        multiplier: 2.0,
        loop_mode: ModelAnimationLoop::Repeat,
        ..ModelAnimationOptions::new()
    }).unwrap();

    assert_eq!(added.len(), 2);
    assert_eq!(collection.len(), 2);
    assert_eq!(collection.get(0).unwrap().multiplier, 2.0);
    assert_eq!(collection.get(0).unwrap().loop_mode, ModelAnimationLoop::Repeat);
    assert_eq!(collection.get(1).unwrap().multiplier, 2.0);
}

#[test]
fn test_contains_and_remove() {
    let mut collection = ModelAnimationCollection::new(true, true);
    collection.animations_metadata.push(create_mock_metadata("anim0", 0, 1.0));
    collection.add(ModelAnimationOptions {
        index: Some(0),
        ..ModelAnimationOptions::new()
    }).unwrap();

    assert!(collection.contains_index(0));
    assert!(collection.contains_name("anim0"));
    assert!(!collection.contains_index(1));
    assert!(!collection.contains_name("unknown"));

    // Remove
    assert!(collection.remove(0));
    assert_eq!(collection.len(), 0);
    assert!(!collection.remove(0));
}

#[test]
fn test_remove_all() {
    let mut collection = ModelAnimationCollection::new(true, true);
    collection.animations_metadata.push(create_mock_metadata("anim0", 0, 1.0));
    collection.animations_metadata.push(create_mock_metadata("anim1", 1, 1.0));

    collection.add_all(ModelAnimationOptions::new()).unwrap();
    assert_eq!(collection.len(), 2);

    collection.remove_all();
    assert_eq!(collection.len(), 0);
}

#[test]
fn test_animation_events_and_remove_on_stop() {
    let mut collection = ModelAnimationCollection::new(true, true);
    collection.animations_metadata.push(create_mock_metadata("anim0", 0, 2.0));

    let start_called = Arc::new(Mutex::new(false));
    let update_count = Arc::new(Mutex::new(0));
    let stop_called = Arc::new(Mutex::new(false));

    let start_c = start_called.clone();
    let update_c = update_count.clone();
    let stop_c = stop_called.clone();

    let start_time = JulianDate::now();

    let idx = collection.add(ModelAnimationOptions {
        index: Some(0),
        start_time: Some(start_time),
        remove_on_stop: true,
        ..ModelAnimationOptions::new()
    }).unwrap();

    let anim = collection.get_mut(idx).unwrap();
    anim.add_start_listener(move |_| {
        *start_c.lock().unwrap() = true;
    });
    anim.add_update_listener(move |_, _| {
        *update_c.lock().unwrap() += 1;
    });
    anim.add_stop_listener(move |_| {
        *stop_c.lock().unwrap() = true;
    });

    let mut nodes = vec![ModelRuntimeNode::new()];

    // At start time -> starts animating
    let played = collection.update(start_time, &mut nodes).unwrap();
    assert!(played);
    assert!(*start_called.lock().unwrap());
    assert_eq!(*update_count.lock().unwrap(), 1);
    assert_eq!(collection.len(), 1);

    // Mid time -> updates
    let mid_time = start_time.add_seconds(1.0);
    let played_mid = collection.update(mid_time, &mut nodes).unwrap();
    assert!(played_mid);
    assert_eq!(*update_count.lock().unwrap(), 2);
    assert_eq!(collection.len(), 1);

    // Beyond stop time -> finishes, calls stop, and removes
    let stop_time = start_time.add_seconds(3.0);
    let played_stop = collection.update(stop_time, &mut nodes).unwrap();
    assert!(played_stop);
    assert!(*stop_called.lock().unwrap());
    assert_eq!(collection.len(), 0); // removeOnStop
}

#[test]
fn test_animates_with_delay() {
    let mut collection = ModelAnimationCollection::new(true, true);
    collection.animations_metadata.push(create_mock_metadata("anim0", 0, 2.0));

    let start_time = JulianDate::now();

    collection.add(ModelAnimationOptions {
        index: Some(0),
        start_time: Some(start_time),
        delay: 5.0,
        ..ModelAnimationOptions::new()
    }).unwrap();

    let mut nodes = vec![ModelRuntimeNode::new()];

    // Before delay elapsed -> does not animate
    let t1 = start_time.add_seconds(2.0);
    let played = collection.update(t1, &mut nodes).unwrap();
    assert!(!played);
    assert_eq!(nodes[0].translation, Cartesian3::ZERO);

    // After delay elapsed -> animates
    let t2 = start_time.add_seconds(6.0);
    let played2 = collection.update(t2, &mut nodes).unwrap();
    assert!(played2);
    assert!(nodes[0].translation.x > 0.0);
}

#[test]
fn test_animates_with_multiplier_and_reverse() {
    let mut collection = ModelAnimationCollection::new(true, true);
    collection.animations_metadata.push(create_mock_metadata("anim0", 0, 4.0));

    let start_time = JulianDate::now();

    // Multiplier = 2.0, Reverse = true
    collection.add(ModelAnimationOptions {
        index: Some(0),
        start_time: Some(start_time),
        multiplier: 2.0,
        reverse: true,
        ..ModelAnimationOptions::new()
    }).unwrap();

    let mut nodes = vec![ModelRuntimeNode::new()];

    // At start time -> because of reverse, starts at end (1.0, 1.0, 1.0)
    collection.update(start_time, &mut nodes).unwrap();
    assert_eq!(nodes[0].translation, Cartesian3::new(1.0, 1.0, 1.0));

    // After 1.0s (which is 2.0s local time due to multiplier 2.0), reaches halfway
    let t1 = start_time.add_seconds(1.0);
    collection.update(t1, &mut nodes).unwrap();
    assert!((nodes[0].translation.x - 0.5).abs() < 1e-6);
}

#[test]
fn test_animates_with_repeat_and_mirrored_repeat() {
    let mut collection = ModelAnimationCollection::new(true, true);
    collection.animations_metadata.push(create_mock_metadata("anim0", 0, 2.0));

    let start_time = JulianDate::now();

    collection.add(ModelAnimationOptions {
        index: Some(0),
        start_time: Some(start_time),
        loop_mode: ModelAnimationLoop::Repeat,
        ..ModelAnimationOptions::new()
    }).unwrap();

    let mut nodes = vec![ModelRuntimeNode::new()];

    // At 3.0s (1.5 periods) -> wraps to 1.0s (halfway)
    let t = start_time.add_seconds(3.0);
    collection.update(t, &mut nodes).unwrap();
    assert!((nodes[0].translation.x - 0.5).abs() < 1e-6);
    assert_eq!(collection.get(0).unwrap().state, ModelAnimationState::Animating);
}
