//! Tests for ModelAnimation matching CesiumJS `Scene/Model/ModelAnimationSpec.js` (Apache-2.0).

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;

use cesium_math::{Cartesian3, JulianDate, Quaternion, TimeStandard};
use cesium_renderer::model_animation::{
    AnimationMetadata, ModelAnimation, ModelAnimationLoop, ModelAnimationModel,
    ModelAnimationOptions, ModelAnimationSceneGraph,
};
use cesium_renderer::model_animation_channel::{
    AnimatedPropertyType, AnimationChannel, AnimationOutputValues, AnimationSampler,
    AnimationTarget, InterpolationType, ModelRuntimeNode,
};

fn create_mock_channel(
    path: AnimatedPropertyType,
    sampler: AnimationSampler,
    node_index: usize,
) -> AnimationChannel {
    AnimationChannel {
        sampler,
        target: Some(AnimationTarget {
            node: Some(node_index),
            path,
        }),
    }
}

fn create_test_context() -> (
    Rc<RefCell<ModelRuntimeNode>>,
    ModelAnimationModel,
    AnimationSampler,
    AnimationSampler,
) {
    let runtime_node = Rc::new(RefCell::new(ModelRuntimeNode::new()));
    let scene_graph = ModelAnimationSceneGraph::new(vec![runtime_node.clone()]);
    let model = ModelAnimationModel::new(true, scene_graph);

    let translation_sampler = AnimationSampler {
        input: vec![0.0, 0.5, 1.0],
        interpolation: InterpolationType::Linear,
        output: AnimationOutputValues::Cartesian3(vec![
            Cartesian3::ZERO,
            Cartesian3::new(1.0, 2.0, 3.0),
            Cartesian3::new(4.0, 5.0, 6.0),
        ]),
    };

    let rotation_sampler = AnimationSampler {
        input: vec![0.25, 0.75, 1.25],
        interpolation: InterpolationType::Step,
        output: AnimationOutputValues::Quaternion(vec![
            Quaternion::IDENTITY,
            Quaternion::new(0.0, 0.0, 0.707, -0.707),
            Quaternion::IDENTITY,
        ]),
    };

    (runtime_node, model, translation_sampler, rotation_sampler)
}

#[test]
fn test_initializes() {
    let (_node, model, translation_sampler, rotation_sampler) = create_test_context();

    let animation = AnimationMetadata {
        channels: vec![
            create_mock_channel(AnimatedPropertyType::Translation, translation_sampler, 0),
            create_mock_channel(AnimatedPropertyType::Rotation, rotation_sampler, 0),
        ],
        name: Some("Sample Animation".to_string()),
        index: 0,
        local_start_time: 0.0,
        local_stop_time: 1.25,
    };

    let runtime_animation = ModelAnimation::new(&model, &animation, None).unwrap();

    assert_eq!(runtime_animation.animation.as_ref(), Some(&animation));
    assert_eq!(runtime_animation.name.as_deref(), Some("Sample Animation"));
    assert!(runtime_animation.model.is_some());

    let channels = &runtime_animation.runtime_channels;
    assert_eq!(channels.len(), 2);
    assert_eq!(channels[0].path, AnimatedPropertyType::Translation);
    assert_eq!(channels[1].path, AnimatedPropertyType::Rotation);

    assert_eq!(runtime_animation.local_start_time, 0.0);
    assert!((runtime_animation.local_stop_time - 1.25).abs() < 1e-6);

    assert!(runtime_animation.start_time.is_none());
    assert_eq!(runtime_animation.delay, 0.0);
    assert!(runtime_animation.stop_time.is_none());
    assert!(!runtime_animation.remove_on_stop);
    assert_eq!(runtime_animation.multiplier, 1.0);
    assert!(!runtime_animation.reverse);
    assert_eq!(runtime_animation.loop_mode, ModelAnimationLoop::None);
}

#[test]
fn test_initializes_with_options() {
    let (_node, model, translation_sampler, rotation_sampler) = create_test_context();

    let animation = AnimationMetadata {
        channels: vec![
            create_mock_channel(AnimatedPropertyType::Translation, translation_sampler, 0),
            create_mock_channel(AnimatedPropertyType::Rotation, rotation_sampler, 0),
        ],
        name: Some("Sample Animation".to_string()),
        index: 0,
        local_start_time: 0.0,
        local_stop_time: 1.25,
    };

    let start_time = JulianDate::new(2456658.0, 43200.0, TimeStandard::UTC);
    let stop_time = JulianDate::new(2456658.0, 43290.0, TimeStandard::UTC);

    let options = ModelAnimationOptions {
        start_time: Some(start_time),
        delay: 5.0,
        stop_time: Some(stop_time),
        multiplier: 0.5,
        reverse: true,
        loop_mode: ModelAnimationLoop::Repeat,
        remove_on_stop: true,
        ..Default::default()
    };

    let runtime_animation = ModelAnimation::new(&model, &animation, Some(options)).unwrap();

    assert_eq!(runtime_animation.animation.as_ref(), Some(&animation));
    assert_eq!(runtime_animation.name.as_deref(), Some("Sample Animation"));
    assert!(runtime_animation.model.is_some());

    let channels = &runtime_animation.runtime_channels;
    assert_eq!(channels.len(), 2);
    assert_eq!(channels[0].path, AnimatedPropertyType::Translation);
    assert_eq!(channels[1].path, AnimatedPropertyType::Rotation);

    assert_eq!(runtime_animation.local_start_time, 0.0);
    assert!((runtime_animation.local_stop_time - 1.25).abs() < 1e-6);

    assert_eq!(runtime_animation.start_time, Some(start_time));
    assert_eq!(runtime_animation.delay, 5.0);
    assert_eq!(runtime_animation.stop_time, Some(stop_time));
    assert!(runtime_animation.remove_on_stop);
    assert_eq!(runtime_animation.multiplier, 0.5);
    assert!(runtime_animation.reverse);
    assert_eq!(runtime_animation.loop_mode, ModelAnimationLoop::Repeat);
}

#[test]
fn test_initializes_without_invalid_channels() {
    let (_node, model, translation_sampler, rotation_sampler) = create_test_context();

    let animation = AnimationMetadata {
        channels: vec![
            create_mock_channel(AnimatedPropertyType::Translation, translation_sampler, 0),
            AnimationChannel {
                sampler: rotation_sampler,
                target: None,
            },
        ],
        name: Some("Sample Animation".to_string()),
        index: 0,
        local_start_time: 0.0,
        local_stop_time: 1.0,
    };

    let runtime_animation = ModelAnimation::new(&model, &animation, None).unwrap();

    assert_eq!(runtime_animation.animation.as_ref(), Some(&animation));
    assert_eq!(runtime_animation.name.as_deref(), Some("Sample Animation"));
    assert!(runtime_animation.model.is_some());

    let channels = &runtime_animation.runtime_channels;
    assert_eq!(channels.len(), 1);
    assert_eq!(channels[0].path, AnimatedPropertyType::Translation);

    assert_eq!(runtime_animation.local_start_time, 0.0);
    assert_eq!(runtime_animation.local_stop_time, 1.0);
}

#[test]
fn test_animates() {
    let (runtime_node, model, translation_sampler, rotation_sampler) = create_test_context();

    let animation = AnimationMetadata {
        channels: vec![
            create_mock_channel(AnimatedPropertyType::Translation, translation_sampler, 0),
            create_mock_channel(AnimatedPropertyType::Rotation, rotation_sampler, 0),
        ],
        name: Some("Sample Animation".to_string()),
        index: 0,
        local_start_time: 0.0,
        local_stop_time: 1.25,
    };

    let runtime_animation = ModelAnimation::new(&model, &animation, None).unwrap();

    assert_eq!(runtime_node.borrow().translation, Cartesian3::ZERO);
    assert_eq!(runtime_node.borrow().rotation, Quaternion::IDENTITY);

    runtime_animation.animate(0.5).unwrap();

    assert_eq!(
        runtime_node.borrow().translation,
        Cartesian3::new(1.0, 2.0, 3.0)
    );
    assert_eq!(runtime_node.borrow().rotation, Quaternion::IDENTITY);

    runtime_animation.animate(1.0).unwrap();

    assert_eq!(
        runtime_node.borrow().translation,
        Cartesian3::new(4.0, 5.0, 6.0)
    );
    assert_eq!(
        runtime_node.borrow().rotation,
        Quaternion::new(0.0, 0.0, 0.707, -0.707)
    );
}

#[test]
fn test_animation_events_and_callbacks() {
    let (_node, model, translation_sampler, _rotation_sampler) = create_test_context();

    let animation = AnimationMetadata {
        channels: vec![create_mock_channel(
            AnimatedPropertyType::Translation,
            translation_sampler,
            0,
        )],
        name: Some("Sample Animation".to_string()),
        index: 0,
        local_start_time: 0.0,
        local_stop_time: 1.0,
    };

    let mut runtime_animation = ModelAnimation::new(&model, &animation, None).unwrap();

    let started = Arc::new(AtomicBool::new(false));
    let s_clone = started.clone();
    runtime_animation.add_start_listener(move |_| {
        s_clone.store(true, Ordering::SeqCst);
    });

    let update_count = Arc::new(AtomicUsize::new(0));
    let u_clone = update_count.clone();
    runtime_animation.add_update_listener(move |_, _| {
        u_clone.fetch_add(1, Ordering::SeqCst);
    });

    let stopped = Arc::new(AtomicBool::new(false));
    let st_clone = stopped.clone();
    runtime_animation.add_stop_listener(move |_| {
        st_clone.store(true, Ordering::SeqCst);
    });

    runtime_animation.raise_start_event();
    assert!(started.load(Ordering::SeqCst));

    runtime_animation.raise_update_event(0.5);
    assert_eq!(update_count.load(Ordering::SeqCst), 1);

    runtime_animation.raise_stop_event();
    assert!(stopped.load(Ordering::SeqCst));
}
