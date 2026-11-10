//! ModelAnimationChannel matching CesiumJS `Scene/Model/ModelAnimationChannel.js` (Apache-2.0).

use std::cell::RefCell;
use std::rc::Rc;

use cesium_math::{
    clamp_time, find_time_interval, wrap_time, Cartesian3, ConstantSpline, DeveloperError,
    HermiteSpline, LinearSpline, Matrix4, Quaternion, QuaternionSpline, SteppedSpline,
};

/// The property of a node that an animation channel targets.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AnimatedPropertyType {
    Translation,
    Rotation,
    Scale,
    Weights,
}

/// Interpolation algorithm used by an animation sampler.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InterpolationType {
    Linear,
    Step,
    CubicSpline,
}

/// Animation output values representing keyframe points.
#[derive(Debug, Clone, PartialEq)]
pub enum AnimationOutputValues {
    Cartesian3(Vec<Cartesian3>),
    Quaternion(Vec<Quaternion>),
    Float(Vec<f64>),
}

/// Animation sampler matching glTF animation sampler.
#[derive(Debug, Clone, PartialEq)]
pub struct AnimationSampler {
    pub input: Vec<f64>,
    pub interpolation: InterpolationType,
    pub output: AnimationOutputValues,
}

/// Target node and property path for an animation channel.
#[derive(Debug, Clone, PartialEq)]
pub struct AnimationTarget {
    pub node: Option<usize>,
    pub path: AnimatedPropertyType,
}

/// glTF animation channel component.
#[derive(Debug, Clone, PartialEq)]
pub struct AnimationChannel {
    pub sampler: AnimationSampler,
    pub target: Option<AnimationTarget>,
}

/// Runtime node representing a node in the scene graph being animated.
#[derive(Debug, Clone, PartialEq)]
pub struct ModelRuntimeNode {
    pub translation: Cartesian3,
    pub rotation: Quaternion,
    pub scale: Cartesian3,
    pub morph_weights: Vec<f64>,
    pub user_animated: bool,
    pub original_transform: Matrix4,
    pub transform: Matrix4,
}

impl Default for ModelRuntimeNode {
    fn default() -> Self {
        Self {
            translation: Cartesian3::ZERO,
            rotation: Quaternion::IDENTITY,
            scale: Cartesian3::new(1.0, 1.0, 1.0),
            morph_weights: Vec::new(),
            user_animated: false,
            original_transform: Matrix4::IDENTITY,
            transform: Matrix4::IDENTITY,
        }
    }
}

impl ModelRuntimeNode {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_transform(transform: Matrix4) -> Self {
        Self {
            original_transform: transform,
            transform,
            ..Default::default()
        }
    }

    pub fn computed_transform(&self) -> Matrix4 {
        Matrix4::from_translation_quaternion_rotation_scale(
            self.translation,
            self.rotation,
            self.scale,
        )
    }
}

/// Runtime animation configuration owning animation channels.
#[derive(Debug, Clone, PartialEq)]
pub struct ModelAnimationChannelAnimation {
    pub clamp_animations: bool,
}

impl Default for ModelAnimationChannelAnimation {
    fn default() -> Self {
        Self {
            clamp_animations: true,
        }
    }
}

/// Scalar cubic spline for weights interpolation matching CesiumJS HermiteSpline.
#[derive(Clone, Debug, PartialEq)]
pub struct ScalarHermiteSpline {
    pub times: Vec<f64>,
    pub points: Vec<f64>,
    pub in_tangents: Vec<f64>,
    pub out_tangents: Vec<f64>,
}

impl ScalarHermiteSpline {
    pub fn new(
        times: Vec<f64>,
        points: Vec<f64>,
        in_tangents: Vec<f64>,
        out_tangents: Vec<f64>,
    ) -> Result<Self, DeveloperError> {
        if points.len() < 2 {
            return Err(DeveloperError::new(
                "points.length must be greater than or equal to 2.",
            ));
        }
        if times.len() != points.len() {
            return Err(DeveloperError::new(
                "times.length must be equal to points.length.",
            ));
        }
        if in_tangents.len() != points.len() - 1 || out_tangents.len() != points.len() - 1 {
            return Err(DeveloperError::new(
                "inTangents or outTangents length is not equal to points.length - 1.",
            ));
        }
        Ok(Self {
            times,
            points,
            in_tangents,
            out_tangents,
        })
    }

    pub fn evaluate(&self, time: f64) -> Result<f64, DeveloperError> {
        let i = find_time_interval(&self.times, time)?;
        let t0 = self.times[i];
        let t1 = self.times[i + 1];
        let dt = t1 - t0;
        let u = if dt > 0.0 { (time - t0) / dt } else { 0.0 };
        let u2 = u * u;
        let u3 = u2 * u;

        let h00 = 2.0 * u3 - 3.0 * u2 + 1.0;
        let h10 = u3 - 2.0 * u2 + u;
        let h01 = -2.0 * u3 + 3.0 * u2;
        let h11 = u3 - u2;

        let p0 = self.points[i];
        let p1 = self.points[i + 1];
        let m0 = self.out_tangents[i] * dt;
        let m1 = self.in_tangents[i] * dt;

        Ok(h00 * p0 + h10 * m0 + h01 * p1 + h11 * m1)
    }

    pub fn clamp_time(&self, time: f64) -> f64 {
        clamp_time(&self.times, time)
    }

    pub fn wrap_time(&self, time: f64) -> f64 {
        wrap_time(&self.times, time)
    }
}

/// Spline variant used by ModelAnimationChannel.
#[derive(Debug, Clone, PartialEq)]
pub enum AnimationSpline {
    ConstantCartesian3(ConstantSpline<Cartesian3>),
    ConstantQuaternion(ConstantSpline<Quaternion>),
    ConstantFloat(ConstantSpline<f64>),
    LinearCartesian3(LinearSpline<Cartesian3>),
    LinearFloat(LinearSpline<f64>),
    Quaternion(QuaternionSpline),
    SteppedCartesian3(SteppedSpline<Cartesian3>),
    SteppedQuaternion(SteppedSpline<Quaternion>),
    SteppedFloat(SteppedSpline<f64>),
    CubicCartesian3(HermiteSpline),
    CubicFloat(ScalarHermiteSpline),
}

impl AnimationSpline {
    pub fn clamp_time(&self, time: f64) -> f64 {
        match self {
            Self::ConstantCartesian3(s) => s.clamp_time(time),
            Self::ConstantQuaternion(s) => s.clamp_time(time),
            Self::ConstantFloat(s) => s.clamp_time(time),
            Self::LinearCartesian3(s) => s.clamp_time(time),
            Self::LinearFloat(s) => s.clamp_time(time),
            Self::Quaternion(s) => s.clamp_time(time),
            Self::SteppedCartesian3(s) => s.clamp_time(time),
            Self::SteppedQuaternion(s) => s.clamp_time(time),
            Self::SteppedFloat(s) => s.clamp_time(time),
            Self::CubicCartesian3(s) => s.clamp_time(time),
            Self::CubicFloat(s) => s.clamp_time(time),
        }
    }

    pub fn wrap_time(&self, time: f64) -> f64 {
        match self {
            Self::ConstantCartesian3(s) => s.wrap_time(time),
            Self::ConstantQuaternion(s) => s.wrap_time(time),
            Self::ConstantFloat(s) => s.wrap_time(time),
            Self::LinearCartesian3(s) => s.wrap_time(time),
            Self::LinearFloat(s) => s.wrap_time(time),
            Self::Quaternion(s) => s.wrap_time(time),
            Self::SteppedCartesian3(s) => s.wrap_time(time),
            Self::SteppedQuaternion(s) => s.wrap_time(time),
            Self::SteppedFloat(s) => s.wrap_time(time),
            Self::CubicCartesian3(s) => s.wrap_time(time),
            Self::CubicFloat(s) => s.wrap_time(time),
        }
    }

    pub fn evaluate_cartesian3(&self, time: f64) -> Result<Cartesian3, DeveloperError> {
        match self {
            Self::ConstantCartesian3(s) => Ok(s.evaluate(time)),
            Self::LinearCartesian3(s) => s.evaluate(time),
            Self::SteppedCartesian3(s) => s.evaluate(time),
            Self::CubicCartesian3(s) => s.evaluate(time),
            _ => Err(DeveloperError::new("Spline is not a Cartesian3 spline")),
        }
    }

    pub fn evaluate_quaternion(&self, time: f64) -> Result<Quaternion, DeveloperError> {
        match self {
            Self::ConstantQuaternion(s) => Ok(s.evaluate(time)),
            Self::Quaternion(s) => s.evaluate(time),
            Self::SteppedQuaternion(s) => s.evaluate(time),
            _ => Err(DeveloperError::new("Spline is not a Quaternion spline")),
        }
    }

    pub fn evaluate_float(&self, time: f64) -> Result<f64, DeveloperError> {
        match self {
            Self::ConstantFloat(s) => Ok(s.evaluate(time)),
            Self::LinearFloat(s) => s.evaluate(time),
            Self::SteppedFloat(s) => s.evaluate(time),
            Self::CubicFloat(s) => s.evaluate(time),
            _ => Err(DeveloperError::new("Spline is not a Float spline")),
        }
    }
}

/// Runtime animation channel matching CesiumJS `ModelAnimationChannel`.
#[derive(Debug, Clone, PartialEq)]
pub struct ModelAnimationChannel {
    pub channel: AnimationChannel,
    pub runtime_animation: ModelAnimationChannelAnimation,
    pub splines: Vec<AnimationSpline>,
    pub path: AnimatedPropertyType,
    pub runtime_node: Option<Rc<RefCell<ModelRuntimeNode>>>,
}

impl ModelAnimationChannel {
    /// Creates a new `ModelAnimationChannel`.
    pub fn new(
        channel: Option<AnimationChannel>,
        runtime_animation: Option<ModelAnimationChannelAnimation>,
        runtime_node: Option<&ModelRuntimeNode>,
    ) -> Result<Self, DeveloperError> {
        let channel = channel.ok_or_else(|| DeveloperError::new("options.channel is required."))?;
        let runtime_animation = runtime_animation.ok_or_else(|| DeveloperError::new("options.runtimeAnimation is required."))?;
        let runtime_node = runtime_node.ok_or_else(|| DeveloperError::new("options.runtimeNode is required."))?;

        let sampler = &channel.sampler;
        let times = &sampler.input;
        let interpolation = sampler.interpolation;
        let target = channel.target.as_ref().ok_or_else(|| DeveloperError::new("options.channel.target is required."))?;
        let path = target.path;

        let count = if path == AnimatedPropertyType::Weights {
            runtime_node.morph_weights.len().max(1)
        } else {
            1
        };

        let splines = Self::create_splines(times, &sampler.output, interpolation, path, count)?;

        Ok(Self {
            channel,
            runtime_animation,
            splines,
            path,
            runtime_node: None,
        })
    }

    /// Creates a new `ModelAnimationChannel` with an attached shared runtime node cell.
    pub fn new_with_node_rc(
        channel: Option<AnimationChannel>,
        runtime_animation: Option<ModelAnimationChannelAnimation>,
        runtime_node: Option<Rc<RefCell<ModelRuntimeNode>>>,
    ) -> Result<Self, DeveloperError> {
        let node_cell = runtime_node.as_ref().ok_or_else(|| DeveloperError::new("options.runtimeNode is required."))?;
        let mut inst = {
            let borrowed = node_cell.borrow();
            Self::new(channel, runtime_animation, Some(&borrowed))?
        };
        inst.runtime_node = runtime_node;
        Ok(inst)
    }

    /// Evaluates the channel spline and animates the attached runtime node.
    pub fn animate_node(&self, time: f64) -> Result<(), DeveloperError> {
        if let Some(node) = &self.runtime_node {
            let mut borrowed = node.borrow_mut();
            self.animate(&mut borrowed, time)?;
        }
        Ok(())
    }

    fn create_cubic_spline_cartesian3(times: &[f64], points: &[Cartesian3]) -> Result<HermiteSpline, DeveloperError> {
        let mut cubic_points = Vec::new();
        let mut in_tangents = Vec::new();
        let mut out_tangents = Vec::new();

        let length = points.len();
        let mut i = 0;
        while i < length {
            in_tangents.push(points[i]);
            cubic_points.push(points[i + 1]);
            out_tangents.push(points[i + 2]);
            i += 3;
        }

        if !in_tangents.is_empty() {
            in_tangents.remove(0);
        }
        if !out_tangents.is_empty() {
            out_tangents.pop();
        }

        HermiteSpline::new(times.to_vec(), cubic_points, in_tangents, out_tangents)
    }

    fn create_splines(
        times: &[f64],
        points: &AnimationOutputValues,
        interpolation: InterpolationType,
        path: AnimatedPropertyType,
        count: usize,
    ) -> Result<Vec<AnimationSpline>, DeveloperError> {
        let mut splines = Vec::new();

        if path == AnimatedPropertyType::Weights {
            let float_points = match points {
                AnimationOutputValues::Float(pts) => pts,
                _ => return Err(DeveloperError::new("Weights require float output points.")),
            };

            let points_length = float_points.len();
            let output_length = points_length / count;

            for target_index in 0..count {
                let mut output = vec![0.0; output_length];

                let mut points_index = target_index;
                if interpolation == InterpolationType::CubicSpline {
                    let mut i = 0;
                    while i < output_length {
                        output[i] = float_points[points_index];
                        output[i + 1] = float_points[points_index + count];
                        output[i + 2] = float_points[points_index + 2 * count];
                        points_index += count * 3;
                        i += 3;
                    }
                } else {
                    for val in output.iter_mut().take(output_length) {
                        *val = float_points[points_index];
                        points_index += count;
                    }
                }

                if times.len() == 1 && output.len() == 1 {
                    splines.push(AnimationSpline::ConstantFloat(ConstantSpline::new(output[0])));
                } else {
                    match interpolation {
                        InterpolationType::Step => {
                            let s = SteppedSpline::new(times.to_vec(), output)?;
                            splines.push(AnimationSpline::SteppedFloat(s));
                        }
                        InterpolationType::Linear => {
                            let s = LinearSpline::new(times.to_vec(), output)?;
                            splines.push(AnimationSpline::LinearFloat(s));
                        }
                        InterpolationType::CubicSpline => {
                            let mut cubic_pts = Vec::new();
                            let mut in_tans = Vec::new();
                            let mut out_tans = Vec::new();

                            let mut i = 0;
                            while i < output.len() {
                                in_tans.push(output[i]);
                                cubic_pts.push(output[i + 1]);
                                out_tans.push(output[i + 2]);
                                i += 3;
                            }
                            if !in_tans.is_empty() {
                                in_tans.remove(0);
                            }
                            if !out_tans.is_empty() {
                                out_tans.pop();
                            }
                            let s = ScalarHermiteSpline::new(times.to_vec(), cubic_pts, in_tans, out_tans)?;
                            splines.push(AnimationSpline::CubicFloat(s));
                        }
                    }
                }
            }
        } else {
            match points {
                AnimationOutputValues::Cartesian3(pts) => {
                    if times.len() == 1 && pts.len() == 1 {
                        splines.push(AnimationSpline::ConstantCartesian3(ConstantSpline::new(pts[0])));
                    } else {
                        match interpolation {
                            InterpolationType::Step => {
                                let s = SteppedSpline::new(times.to_vec(), pts.clone())?;
                                splines.push(AnimationSpline::SteppedCartesian3(s));
                            }
                            InterpolationType::Linear => {
                                let s = LinearSpline::new(times.to_vec(), pts.clone())?;
                                splines.push(AnimationSpline::LinearCartesian3(s));
                            }
                            InterpolationType::CubicSpline => {
                                let s = Self::create_cubic_spline_cartesian3(times, pts)?;
                                splines.push(AnimationSpline::CubicCartesian3(s));
                            }
                        }
                    }
                }
                AnimationOutputValues::Quaternion(pts) => {
                    if times.len() == 1 && pts.len() == 1 {
                        splines.push(AnimationSpline::ConstantQuaternion(ConstantSpline::new(pts[0])));
                    } else {
                        match interpolation {
                            InterpolationType::Step => {
                                let s = SteppedSpline::new(times.to_vec(), pts.clone())?;
                                splines.push(AnimationSpline::SteppedQuaternion(s));
                            }
                            InterpolationType::Linear => {
                                let s = QuaternionSpline::new(times.to_vec(), pts.clone())?;
                                splines.push(AnimationSpline::Quaternion(s));
                            }
                            InterpolationType::CubicSpline => {
                                let s = QuaternionSpline::new(times.to_vec(), pts.clone())?;
                                splines.push(AnimationSpline::Quaternion(s));
                            }
                        }
                    }
                }
                AnimationOutputValues::Float(pts) => {
                    if times.len() == 1 && pts.len() == 1 {
                        splines.push(AnimationSpline::ConstantFloat(ConstantSpline::new(pts[0])));
                    } else {
                        match interpolation {
                            InterpolationType::Step => {
                                let s = SteppedSpline::new(times.to_vec(), pts.clone())?;
                                splines.push(AnimationSpline::SteppedFloat(s));
                            }
                            InterpolationType::Linear => {
                                let s = LinearSpline::new(times.to_vec(), pts.clone())?;
                                splines.push(AnimationSpline::LinearFloat(s));
                            }
                            InterpolationType::CubicSpline => {
                                let mut cubic_pts = Vec::new();
                                let mut in_tans = Vec::new();
                                let mut out_tans = Vec::new();
                                let mut i = 0;
                                while i < pts.len() {
                                    in_tans.push(pts[i]);
                                    cubic_pts.push(pts[i + 1]);
                                    out_tans.push(pts[i + 2]);
                                    i += 3;
                                }
                                if !in_tans.is_empty() {
                                    in_tans.remove(0);
                                }
                                if !out_tans.is_empty() {
                                    out_tans.pop();
                                }
                                let s = ScalarHermiteSpline::new(times.to_vec(), cubic_pts, in_tans, out_tans)?;
                                splines.push(AnimationSpline::CubicFloat(s));
                            }
                        }
                    }
                }
            }
        }

        Ok(splines)
    }

    /// Evaluates the channel at the given local time and modifies the runtime node.
    pub fn animate(&self, runtime_node: &mut ModelRuntimeNode, time: f64) -> Result<(), DeveloperError> {
        let splines = &self.splines;
        let clamp = self.runtime_animation.clamp_animations;

        if self.path == AnimatedPropertyType::Weights {
            let length = runtime_node.morph_weights.len().min(splines.len());
            for (i, spline) in splines.iter().enumerate().take(length) {
                let local_time = if clamp {
                    spline.clamp_time(time)
                } else {
                    spline.wrap_time(time)
                };
                runtime_node.morph_weights[i] = spline.evaluate_float(local_time)?;
            }
        } else if runtime_node.user_animated {
            // If the node is being animated externally, ignore the glTF animation.
            return Ok(());
        } else if let Some(spline) = splines.first() {
            let local_time = if clamp {
                spline.clamp_time(time)
            } else {
                spline.wrap_time(time)
            };

            match self.path {
                AnimatedPropertyType::Translation => {
                    runtime_node.translation = spline.evaluate_cartesian3(local_time)?;
                }
                AnimatedPropertyType::Scale => {
                    runtime_node.scale = spline.evaluate_cartesian3(local_time)?;
                }
                AnimatedPropertyType::Rotation => {
                    runtime_node.rotation = spline.evaluate_quaternion(local_time)?;
                }
                AnimatedPropertyType::Weights => {}
            }
        }

        Ok(())
    }
}
