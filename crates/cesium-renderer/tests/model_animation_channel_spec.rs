//! Tests for ModelAnimationChannel matching CesiumJS `Scene/Model/ModelAnimationChannelSpec.js` (Apache-2.0).

use cesium_math::{Cartesian3, EPSILON6, Matrix3, Matrix4, Quaternion};
use cesium_renderer::model_animation_channel::{
    AnimatedPropertyType, AnimationChannel, AnimationOutputValues, AnimationSampler,
    AnimationSpline, AnimationTarget, InterpolationType, ModelAnimation, ModelAnimationChannel,
    ModelRuntimeNode,
};

fn create_mock_channel(
    path: AnimatedPropertyType,
    sampler: AnimationSampler,
) -> AnimationChannel {
    AnimationChannel {
        sampler,
        target: AnimationTarget {
            node: Some(0),
            path,
        },
    }
}

#[test]
fn test_throws_for_missing_arguments() {
    let runtime_node = ModelRuntimeNode::new();
    let runtime_anim = ModelAnimation::default();

    // Missing channel
    assert!(ModelAnimationChannel::new(None, Some(runtime_anim.clone()), Some(&runtime_node)).is_err());

    let sampler = AnimationSampler {
        input: vec![0.0],
        interpolation: InterpolationType::Linear,
        output: AnimationOutputValues::Cartesian3(vec![Cartesian3::ZERO]),
    };
    let channel = create_mock_channel(AnimatedPropertyType::Translation, sampler);

    // Missing runtimeAnimation
    assert!(ModelAnimationChannel::new(Some(channel.clone()), None, Some(&runtime_node)).is_err());

    // Missing runtimeNode
    assert!(ModelAnimationChannel::new(Some(channel), Some(runtime_anim), None).is_err());
}

#[test]
fn test_constructs_constant_spline() {
    let runtime_node = ModelRuntimeNode::new();
    let runtime_anim = ModelAnimation::default();

    let sampler = AnimationSampler {
        input: vec![0.0],
        interpolation: InterpolationType::Linear,
        output: AnimationOutputValues::Cartesian3(vec![Cartesian3::ZERO]),
    };
    let channel = create_mock_channel(AnimatedPropertyType::Translation, sampler);

    let runtime_channel = ModelAnimationChannel::new(Some(channel), Some(runtime_anim), Some(&runtime_node)).unwrap();

    assert_eq!(runtime_channel.splines.len(), 1);
    assert!(matches!(runtime_channel.splines[0], AnimationSpline::ConstantCartesian3(_)));
}

#[test]
fn test_constructs_linear_spline() {
    let runtime_node = ModelRuntimeNode::new();
    let runtime_anim = ModelAnimation::default();

    let times = vec![0.0, 0.25, 0.5, 1.0];
    let translation_points = vec![
        Cartesian3::ZERO,
        Cartesian3::new(1.0, 0.0, 0.0),
        Cartesian3::new(1.0, 1.0, 0.0),
        Cartesian3::new(0.0, 1.0, 0.0),
    ];

    let sampler = AnimationSampler {
        input: times,
        interpolation: InterpolationType::Linear,
        output: AnimationOutputValues::Cartesian3(translation_points),
    };
    let channel = create_mock_channel(AnimatedPropertyType::Translation, sampler);

    let runtime_channel = ModelAnimationChannel::new(Some(channel), Some(runtime_anim), Some(&runtime_node)).unwrap();

    assert_eq!(runtime_channel.splines.len(), 1);
    assert!(matches!(runtime_channel.splines[0], AnimationSpline::LinearCartesian3(_)));
}

#[test]
fn test_constructs_quaternion_spline() {
    let runtime_node = ModelRuntimeNode::new();
    let runtime_anim = ModelAnimation::default();

    let times = vec![0.0, 0.25, 0.5, 1.0];
    let rotation_points = vec![
        Quaternion::IDENTITY,
        Quaternion::new(0.707, 0.0, 0.707, 0.0),
        Quaternion::new(0.0, 0.0, 1.0, 0.0),
        Quaternion::new(0.707, 0.0, -0.707, 0.0),
    ];

    let sampler = AnimationSampler {
        input: times,
        interpolation: InterpolationType::Linear,
        output: AnimationOutputValues::Quaternion(rotation_points),
    };
    let channel = create_mock_channel(AnimatedPropertyType::Rotation, sampler);

    let runtime_channel = ModelAnimationChannel::new(Some(channel), Some(runtime_anim), Some(&runtime_node)).unwrap();

    assert_eq!(runtime_channel.splines.len(), 1);
    assert!(matches!(runtime_channel.splines[0], AnimationSpline::Quaternion(_)));
}

#[test]
fn test_constructs_stepped_spline() {
    let runtime_node = ModelRuntimeNode::new();
    let runtime_anim = ModelAnimation::default();

    let times = vec![0.0, 0.25, 0.5, 1.0];
    let scale_points = vec![
        Cartesian3::new(1.0, 1.0, 1.0),
        Cartesian3::new(1.0, 2.0, 1.0),
        Cartesian3::new(1.0, 1.0, 2.0),
        Cartesian3::new(2.0, 1.0, 1.0),
    ];

    let sampler = AnimationSampler {
        input: times,
        interpolation: InterpolationType::Step,
        output: AnimationOutputValues::Cartesian3(scale_points),
    };
    let channel = create_mock_channel(AnimatedPropertyType::Scale, sampler);

    let runtime_channel = ModelAnimationChannel::new(Some(channel), Some(runtime_anim), Some(&runtime_node)).unwrap();

    assert_eq!(runtime_channel.splines.len(), 1);
    assert!(matches!(runtime_channel.splines[0], AnimationSpline::SteppedCartesian3(_)));
}

#[test]
fn test_constructs_cubic_spline() {
    let runtime_node = ModelRuntimeNode::new();
    let runtime_anim = ModelAnimation::default();

    let cubic_times = vec![0.0, 0.5, 1.0];
    let cubic_points = vec![
        Cartesian3::ZERO,
        Cartesian3::new(1.0, 1.0, 1.0),
        Cartesian3::new(2.0, 2.0, 2.0),
        Cartesian3::new(3.0, 3.0, 3.0),
        Cartesian3::new(4.0, 4.0, 4.0),
        Cartesian3::new(5.0, 5.0, 5.0),
        Cartesian3::new(6.0, 6.0, 6.0),
        Cartesian3::new(7.0, 7.0, 7.0),
        Cartesian3::new(8.0, 8.0, 8.0),
    ];

    let sampler = AnimationSampler {
        input: cubic_times,
        interpolation: InterpolationType::CubicSpline,
        output: AnimationOutputValues::Cartesian3(cubic_points.clone()),
    };
    let channel = create_mock_channel(AnimatedPropertyType::Translation, sampler);

    let runtime_channel = ModelAnimationChannel::new(Some(channel), Some(runtime_anim), Some(&runtime_node)).unwrap();

    assert_eq!(runtime_channel.splines.len(), 1);
    if let AnimationSpline::CubicCartesian3(spline) = &runtime_channel.splines[0] {
        let expected_in_tangents = vec![cubic_points[3], cubic_points[6]];
        let expected_out_tangents = vec![cubic_points[2], cubic_points[5]];
        let expected_points = vec![cubic_points[1], cubic_points[4], cubic_points[7]];

        assert_eq!(spline.in_tangents, expected_in_tangents);
        assert_eq!(spline.out_tangents, expected_out_tangents);
        assert_eq!(spline.points, expected_points);
    } else {
        panic!("Expected CubicCartesian3");
    }
}

#[test]
fn test_constructs_weight_splines() {
    let mut runtime_node = ModelRuntimeNode::new();
    runtime_node.morph_weights = vec![0.0, 0.0];
    let runtime_anim = ModelAnimation::default();

    let times = vec![0.0, 0.25, 0.5, 1.0];
    let weight_points = vec![0.0, 0.0, 0.5, 0.25, 1.0, 0.5, 0.5, 0.25];

    let sampler = AnimationSampler {
        input: times,
        interpolation: InterpolationType::Linear,
        output: AnimationOutputValues::Float(weight_points),
    };
    let channel = create_mock_channel(AnimatedPropertyType::Weights, sampler);

    let runtime_channel = ModelAnimationChannel::new(Some(channel), Some(runtime_anim), Some(&runtime_node)).unwrap();

    assert_eq!(runtime_channel.splines.len(), 2);
    assert!(matches!(runtime_channel.splines[0], AnimationSpline::LinearFloat(_)));
    assert!(matches!(runtime_channel.splines[1], AnimationSpline::LinearFloat(_)));
}

#[test]
fn test_constructs_cubic_weight_splines() {
    let mut runtime_node = ModelRuntimeNode::new();
    runtime_node.morph_weights = vec![0.0, 0.0];
    let runtime_anim = ModelAnimation::default();

    let cubic_weight_times = vec![0.0, 0.5];
    let cubic_weight_points = vec![
        0.0, 0.0,
        0.0, 0.0,
        0.2, -0.5,
        0.3, 0.0,
        1.0, 0.5,
        0.0, 0.0,
    ];

    let sampler = AnimationSampler {
        input: cubic_weight_times,
        interpolation: InterpolationType::CubicSpline,
        output: AnimationOutputValues::Float(cubic_weight_points),
    };
    let channel = create_mock_channel(AnimatedPropertyType::Weights, sampler);

    let runtime_channel = ModelAnimationChannel::new(Some(channel), Some(runtime_anim), Some(&runtime_node)).unwrap();

    assert_eq!(runtime_channel.splines.len(), 2);

    if let AnimationSpline::CubicFloat(s0) = &runtime_channel.splines[0] {
        assert_eq!(s0.in_tangents, vec![0.3]);
        assert_eq!(s0.points, vec![0.0, 1.0]);
        assert_eq!(s0.out_tangents, vec![0.2]);
    } else {
        panic!("Expected CubicFloat for spline 0");
    }

    if let AnimationSpline::CubicFloat(s1) = &runtime_channel.splines[1] {
        assert_eq!(s1.in_tangents, vec![0.0]);
        assert_eq!(s1.points, vec![0.0, 0.5]);
        assert_eq!(s1.out_tangents, vec![-0.5]);
    } else {
        panic!("Expected CubicFloat for spline 1");
    }
}

#[test]
fn test_animates_node_translation() {
    let mut runtime_node = ModelRuntimeNode::new();
    let runtime_anim = ModelAnimation::default();

    let times = vec![0.0, 0.25, 0.5, 1.0];
    let translation_points = vec![
        Cartesian3::ZERO,
        Cartesian3::new(1.0, 0.0, 0.0),
        Cartesian3::new(1.0, 1.0, 0.0),
        Cartesian3::new(0.0, 1.0, 0.0),
    ];

    let sampler = AnimationSampler {
        input: times.clone(),
        interpolation: InterpolationType::Linear,
        output: AnimationOutputValues::Cartesian3(translation_points.clone()),
    };
    let channel = create_mock_channel(AnimatedPropertyType::Translation, sampler);

    let runtime_channel = ModelAnimationChannel::new(Some(channel), Some(runtime_anim), Some(&runtime_node)).unwrap();

    assert_eq!(runtime_node.translation, Cartesian3::ZERO);
    assert_eq!(runtime_node.transform(), Matrix4::IDENTITY);

    let time = times[1];
    let expected = translation_points[1];

    runtime_channel.animate(&mut runtime_node, time).unwrap();
    assert_eq!(runtime_node.translation, expected);
    assert_eq!(runtime_node.transform(), Matrix4::from_translation(expected));

    let time_mid = (times[1] + times[2]) / 2.0;
    let expected_mid = Cartesian3::lerp(translation_points[1], translation_points[2], 0.5);

    runtime_channel.animate(&mut runtime_node, time_mid).unwrap();
    assert_eq!(runtime_node.translation, expected_mid);
    assert_eq!(runtime_node.transform(), Matrix4::from_translation(expected_mid));
}

#[test]
fn test_animates_node_rotation() {
    let mut runtime_node = ModelRuntimeNode::new();
    let runtime_anim = ModelAnimation::default();

    let times = vec![0.0, 0.25, 0.5, 1.0];
    let rotation_points = vec![
        Quaternion::IDENTITY,
        Quaternion::new(0.707, 0.0, 0.707, 0.0),
        Quaternion::new(0.0, 0.0, 1.0, 0.0),
        Quaternion::new(0.707, 0.0, -0.707, 0.0),
    ];

    let sampler = AnimationSampler {
        input: times.clone(),
        interpolation: InterpolationType::Linear,
        output: AnimationOutputValues::Quaternion(rotation_points.clone()),
    };
    let channel = create_mock_channel(AnimatedPropertyType::Rotation, sampler);

    let runtime_channel = ModelAnimationChannel::new(Some(channel), Some(runtime_anim), Some(&runtime_node)).unwrap();

    assert_eq!(runtime_node.rotation, Quaternion::IDENTITY);
    assert_eq!(runtime_node.transform(), Matrix4::IDENTITY);

    let time = times[1];
    let expected = rotation_points[1];
    let expected_matrix = Matrix3::from_quaternion(&expected);

    runtime_channel.animate(&mut runtime_node, time).unwrap();
    assert_eq!(runtime_node.rotation, expected);
    assert!(runtime_node.transform().equals_epsilon(
        &Matrix4::from_rotation_translation(&expected_matrix, Cartesian3::ZERO),
        EPSILON6,
    ));

    let time_mid = (times[1] + times[2]) / 2.0;
    let expected_mid = Quaternion::slerp(&rotation_points[1], &rotation_points[2], 0.5);
    let expected_matrix_mid = Matrix3::from_quaternion(&expected_mid);

    runtime_channel.animate(&mut runtime_node, time_mid).unwrap();
    assert!(runtime_node.rotation.equals_epsilon(&expected_mid, EPSILON6));
    assert!(runtime_node.transform().equals_epsilon(
        &Matrix4::from_rotation_translation(&expected_matrix_mid, Cartesian3::ZERO),
        EPSILON6,
    ));
}

#[test]
fn test_animates_node_scale() {
    let mut runtime_node = ModelRuntimeNode::new();
    let runtime_anim = ModelAnimation::default();

    let times = vec![0.0, 0.25, 0.5, 1.0];
    let scale_points = vec![
        Cartesian3::new(1.0, 1.0, 1.0),
        Cartesian3::new(1.0, 2.0, 1.0),
        Cartesian3::new(1.0, 1.0, 2.0),
        Cartesian3::new(2.0, 1.0, 1.0),
    ];

    let sampler = AnimationSampler {
        input: times.clone(),
        interpolation: InterpolationType::Linear,
        output: AnimationOutputValues::Cartesian3(scale_points.clone()),
    };
    let channel = create_mock_channel(AnimatedPropertyType::Scale, sampler);

    let runtime_channel = ModelAnimationChannel::new(Some(channel), Some(runtime_anim), Some(&runtime_node)).unwrap();

    assert_eq!(runtime_node.scale, Cartesian3::new(1.0, 1.0, 1.0));
    assert_eq!(runtime_node.transform(), Matrix4::IDENTITY);

    let time = times[1];
    let expected = scale_points[1];

    runtime_channel.animate(&mut runtime_node, time).unwrap();
    assert_eq!(runtime_node.scale, expected);
    assert_eq!(runtime_node.transform(), Matrix4::from_scale(expected));

    let time_mid = (times[1] + times[2]) / 2.0;
    let expected_mid = Cartesian3::lerp(scale_points[1], scale_points[2], 0.5);

    runtime_channel.animate(&mut runtime_node, time_mid).unwrap();
    assert_eq!(runtime_node.scale, expected_mid);
    assert_eq!(runtime_node.transform(), Matrix4::from_scale(expected_mid));
}

#[test]
fn test_animates_node_weights() {
    let mut runtime_node = ModelRuntimeNode::new();
    runtime_node.morph_weights = vec![0.0, 0.0];
    let runtime_anim = ModelAnimation::default();

    let times = vec![0.0, 0.25, 0.5, 1.0];
    let weight_points = vec![0.0, 0.0, 0.5, 0.25, 1.0, 0.5, 0.5, 0.25];

    let sampler = AnimationSampler {
        input: times.clone(),
        interpolation: InterpolationType::Linear,
        output: AnimationOutputValues::Float(weight_points.clone()),
    };
    let channel = create_mock_channel(AnimatedPropertyType::Weights, sampler);

    let runtime_channel = ModelAnimationChannel::new(Some(channel), Some(runtime_anim), Some(&runtime_node)).unwrap();

    assert_eq!(runtime_node.morph_weights, vec![0.0, 0.0]);

    let time = times[1];
    runtime_channel.animate(&mut runtime_node, time).unwrap();
    assert_eq!(runtime_node.morph_weights, vec![weight_points[2], weight_points[3]]);

    let time_mid = (times[1] + times[2]) / 2.0;
    runtime_channel.animate(&mut runtime_node, time_mid).unwrap();
    let expected_mid = vec![
        (weight_points[2] + weight_points[4]) / 2.0,
        (weight_points[3] + weight_points[5]) / 2.0,
    ];
    assert_eq!(runtime_node.morph_weights, expected_mid);
}

#[test]
fn test_clamps_animations() {
    let mut runtime_node = ModelRuntimeNode::new();
    let runtime_anim = ModelAnimation {
        clamp_animations: true,
    };

    let times = vec![0.0, 0.25, 0.5, 1.0];
    let translation_points = vec![
        Cartesian3::ZERO,
        Cartesian3::new(1.0, 0.0, 0.0),
        Cartesian3::new(1.0, 1.0, 0.0),
        Cartesian3::new(0.0, 1.0, 0.0),
    ];

    let sampler = AnimationSampler {
        input: times,
        interpolation: InterpolationType::Linear,
        output: AnimationOutputValues::Cartesian3(translation_points.clone()),
    };
    let channel = create_mock_channel(AnimatedPropertyType::Translation, sampler);

    let runtime_channel = ModelAnimationChannel::new(Some(channel), Some(runtime_anim), Some(&runtime_node)).unwrap();

    // Time > max time
    runtime_channel.animate(&mut runtime_node, 10.0).unwrap();
    assert_eq!(runtime_node.translation, translation_points[3]);

    // Time < min time
    runtime_channel.animate(&mut runtime_node, -10.0).unwrap();
    assert_eq!(runtime_node.translation, translation_points[0]);
}

#[test]
fn test_wraps_animations() {
    let mut runtime_node = ModelRuntimeNode::new();
    let runtime_anim = ModelAnimation {
        clamp_animations: false,
    };

    let times = vec![0.0, 0.25, 0.5, 1.0];
    let translation_points = vec![
        Cartesian3::ZERO,
        Cartesian3::new(1.0, 0.0, 0.0),
        Cartesian3::new(1.0, 1.0, 0.0),
        Cartesian3::new(0.0, 1.0, 0.0),
    ];

    let sampler = AnimationSampler {
        input: times,
        interpolation: InterpolationType::Linear,
        output: AnimationOutputValues::Cartesian3(translation_points.clone()),
    };
    let channel = create_mock_channel(AnimatedPropertyType::Translation, sampler);

    let runtime_channel = ModelAnimationChannel::new(Some(channel), Some(runtime_anim), Some(&runtime_node)).unwrap();

    // 1.25 wraps to 0.25
    runtime_channel.animate(&mut runtime_node, 1.25).unwrap();
    assert_eq!(runtime_node.translation, translation_points[1]);

    // -0.5 wraps to 0.5
    runtime_channel.animate(&mut runtime_node, -0.5).unwrap();
    assert_eq!(runtime_node.translation, translation_points[2]);
}

#[test]
fn test_does_not_modify_user_animated_node() {
    let mut runtime_node = ModelRuntimeNode::new();
    let runtime_anim = ModelAnimation::default();

    let times = vec![0.0, 0.25, 0.5, 1.0];
    let translation_points = vec![
        Cartesian3::ZERO,
        Cartesian3::new(1.0, 0.0, 0.0),
        Cartesian3::new(1.0, 1.0, 0.0),
        Cartesian3::new(0.0, 1.0, 0.0),
    ];

    let sampler = AnimationSampler {
        input: times,
        interpolation: InterpolationType::Linear,
        output: AnimationOutputValues::Cartesian3(translation_points),
    };
    let channel = create_mock_channel(AnimatedPropertyType::Translation, sampler);

    let runtime_channel = ModelAnimationChannel::new(Some(channel), Some(runtime_anim), Some(&runtime_node)).unwrap();

    runtime_node.user_animated = true;
    runtime_node.translation = Cartesian3::new(5.0, 5.0, 5.0);

    runtime_channel.animate(&mut runtime_node, 10.0).unwrap();
    assert_eq!(runtime_node.translation, Cartesian3::new(5.0, 5.0, 5.0));
}
