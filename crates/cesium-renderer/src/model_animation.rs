//! ModelAnimation matching CesiumJS `Scene/Model/ModelAnimation.js` (Apache-2.0).

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

use cesium_math::{DeveloperError, JulianDate};
use crate::model_animation_channel::{
    AnimationChannel, ModelAnimationChannel, ModelAnimationChannelAnimation, ModelRuntimeNode,
};

/// Looping mode for a model animation matching CesiumJS `ModelAnimationLoop`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ModelAnimationLoop {
    #[default]
    None,
    Repeat,
    MirroredRepeat,
}

/// The state of an animation matching CesiumJS `ModelAnimationState`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ModelAnimationState {
    #[default]
    Stopped,
    Animating,
}

/// Metadata description of an animation component in a 3D model.
#[derive(Debug, Clone, PartialEq)]
pub struct AnimationMetadata {
    pub name: Option<String>,
    pub index: usize,
    pub channels: Vec<AnimationChannel>,
    pub local_start_time: f64,
    pub local_stop_time: f64,
}

pub type AnimationTimeCallback = Arc<dyn Fn(f64, f64) -> f64 + Send + Sync>;
pub type AnimationStateCallback = Arc<dyn Fn(&str) + Send + Sync>;
pub type AnimationUpdateCallback = Arc<dyn Fn(&str, f64) + Send + Sync>;
pub type AnimationCollectionCallback = Arc<dyn Fn(usize, &str) + Send + Sync>;

/// Options for adding or initializing a `ModelAnimation`.
#[derive(Clone, Default)]
pub struct ModelAnimationOptions {
    pub name: Option<String>,
    pub index: Option<usize>,
    pub start_time: Option<JulianDate>,
    pub delay: f64,
    pub stop_time: Option<JulianDate>,
    pub remove_on_stop: bool,
    pub multiplier: f64,
    pub reverse: bool,
    pub loop_mode: ModelAnimationLoop,
    pub animation_time: Option<AnimationTimeCallback>,
}

impl ModelAnimationOptions {
    pub fn new() -> Self {
        Self {
            name: None,
            index: None,
            start_time: None,
            delay: 0.0,
            stop_time: None,
            remove_on_stop: false,
            multiplier: 1.0,
            reverse: false,
            loop_mode: ModelAnimationLoop::None,
            animation_time: None,
        }
    }
}

/// Model scene graph holding runtime nodes.
#[derive(Clone, Debug, PartialEq, Default)]
pub struct ModelAnimationSceneGraph {
    pub runtime_nodes: Vec<Rc<RefCell<ModelRuntimeNode>>>,
}

impl ModelAnimationSceneGraph {
    pub fn new(runtime_nodes: Vec<Rc<RefCell<ModelRuntimeNode>>>) -> Self {
        Self { runtime_nodes }
    }
}

/// Model hosting animation channels and scene graph.
#[derive(Clone, Debug, PartialEq)]
pub struct ModelAnimationModel {
    pub clamp_animations: bool,
    pub scene_graph: ModelAnimationSceneGraph,
}

impl Default for ModelAnimationModel {
    fn default() -> Self {
        Self {
            clamp_animations: true,
            scene_graph: ModelAnimationSceneGraph::default(),
        }
    }
}

impl ModelAnimationModel {
    pub fn new(clamp_animations: bool, scene_graph: ModelAnimationSceneGraph) -> Self {
        Self {
            clamp_animations,
            scene_graph,
        }
    }
}

/// An active runtime animation instance matching CesiumJS `ModelAnimation`.
#[derive(Clone)]
pub struct ModelAnimation {
    pub model: Option<ModelAnimationModel>,
    pub animation: Option<AnimationMetadata>,
    pub name: Option<String>,
    pub index: usize,
    pub runtime_channels: Vec<ModelAnimationChannel>,
    pub local_start_time: f64,
    pub local_stop_time: f64,
    pub start_time: Option<JulianDate>,
    pub delay: f64,
    pub stop_time: Option<JulianDate>,
    pub remove_on_stop: bool,
    pub multiplier: f64,
    pub reverse: bool,
    pub loop_mode: ModelAnimationLoop,
    pub animation_time: Option<AnimationTimeCallback>,
    pub start_listeners: Vec<AnimationStateCallback>,
    pub update_listeners: Vec<AnimationUpdateCallback>,
    pub stop_listeners: Vec<AnimationStateCallback>,
    pub state: ModelAnimationState,
    pub computed_start_time: Option<JulianDate>,
    pub duration: Option<f64>,
    pub prev_animation_delta: Option<f64>,
}

impl Default for ModelAnimation {
    fn default() -> Self {
        Self {
            model: None,
            animation: None,
            name: None,
            index: 0,
            runtime_channels: Vec::new(),
            local_start_time: 0.0,
            local_stop_time: 0.0,
            start_time: None,
            delay: 0.0,
            stop_time: None,
            remove_on_stop: false,
            multiplier: 1.0,
            reverse: false,
            loop_mode: ModelAnimationLoop::None,
            animation_time: None,
            start_listeners: Vec::new(),
            update_listeners: Vec::new(),
            stop_listeners: Vec::new(),
            state: ModelAnimationState::Stopped,
            computed_start_time: None,
            duration: None,
            prev_animation_delta: None,
        }
    }
}

impl ModelAnimation {
    /// Creates a new `ModelAnimation` from a model, animation metadata, and options.
    pub fn new(
        model: &ModelAnimationModel,
        animation: &AnimationMetadata,
        options: Option<ModelAnimationOptions>,
    ) -> Result<Self, DeveloperError> {
        let options = options.unwrap_or_default();
        let multiplier = if options.multiplier <= 0.0 {
            1.0
        } else {
            options.multiplier
        };

        let mut local_start_time = f64::MAX;
        let mut local_stop_time = -f64::MAX;
        let mut runtime_channels = Vec::new();

        let channel_config = ModelAnimationChannelAnimation {
            clamp_animations: model.clamp_animations,
        };

        for channel in &animation.channels {
            let target = match &channel.target {
                Some(t) => t,
                None => continue,
            };

            let node_index = match target.node {
                Some(idx) => idx,
                None => continue,
            };

            if node_index >= model.scene_graph.runtime_nodes.len() {
                continue;
            }

            let runtime_node = model.scene_graph.runtime_nodes[node_index].clone();

            let runtime_channel = ModelAnimationChannel::new_with_node_rc(
                Some(channel.clone()),
                Some(channel_config.clone()),
                Some(runtime_node),
            )?;

            let times = &channel.sampler.input;
            if !times.is_empty() {
                local_start_time = local_start_time.min(times[0]);
                local_stop_time = local_stop_time.max(times[times.len() - 1]);
            }

            runtime_channels.push(runtime_channel);
        }

        if runtime_channels.is_empty() {
            local_start_time = 0.0;
            local_stop_time = 0.0;
        }

        Ok(Self {
            model: Some(model.clone()),
            animation: Some(animation.clone()),
            name: animation.name.clone(),
            index: animation.index,
            runtime_channels,
            local_start_time,
            local_stop_time,
            start_time: options.start_time,
            delay: options.delay,
            stop_time: options.stop_time,
            remove_on_stop: options.remove_on_stop,
            multiplier,
            reverse: options.reverse,
            loop_mode: options.loop_mode,
            animation_time: options.animation_time,
            start_listeners: Vec::new(),
            update_listeners: Vec::new(),
            stop_listeners: Vec::new(),
            state: ModelAnimationState::Stopped,
            computed_start_time: None,
            duration: None,
            prev_animation_delta: None,
        })
    }

    /// Creates an animation from metadata and collection options without requiring a full model scene graph.
    pub fn from_metadata(
        metadata: &AnimationMetadata,
        options: ModelAnimationOptions,
        model_clamp_animations: bool,
    ) -> Result<Self, DeveloperError> {
        let multiplier = if options.multiplier <= 0.0 {
            1.0
        } else {
            options.multiplier
        };

        let mut runtime_channels = Vec::new();
        let dummy_runtime_node = ModelRuntimeNode::new();
        let channel_config = ModelAnimationChannelAnimation {
            clamp_animations: model_clamp_animations,
        };

        for ch in &metadata.channels {
            let rc = ModelAnimationChannel::new(
                Some(ch.clone()),
                Some(channel_config.clone()),
                Some(&dummy_runtime_node),
            )?;
            runtime_channels.push(rc);
        }

        Ok(Self {
            model: None,
            animation: Some(metadata.clone()),
            name: metadata.name.clone(),
            index: metadata.index,
            runtime_channels,
            local_start_time: metadata.local_start_time,
            local_stop_time: metadata.local_stop_time,
            start_time: options.start_time,
            delay: options.delay,
            stop_time: options.stop_time,
            remove_on_stop: options.remove_on_stop,
            multiplier,
            reverse: options.reverse,
            loop_mode: options.loop_mode,
            animation_time: options.animation_time,
            start_listeners: Vec::new(),
            update_listeners: Vec::new(),
            stop_listeners: Vec::new(),
            state: ModelAnimationState::Stopped,
            computed_start_time: None,
            duration: None,
            prev_animation_delta: None,
        })
    }

    /// Evaluates all channels and advances attached runtime nodes.
    pub fn animate(&self, time: f64) -> Result<(), DeveloperError> {
        for channel in &self.runtime_channels {
            channel.animate_node(time)?;
        }
        Ok(())
    }

    /// Evaluates all channels and advances a specific runtime node.
    pub fn animate_node(
        &self,
        runtime_node: &mut ModelRuntimeNode,
        time: f64,
    ) -> Result<(), DeveloperError> {
        for channel in &self.runtime_channels {
            channel.animate(runtime_node, time)?;
        }
        Ok(())
    }

    /// Raises the `start` event to registered listeners.
    pub fn raise_start_event(&self) {
        let name = self.name.as_deref().unwrap_or_default();
        for listener in &self.start_listeners {
            listener(name);
        }
    }

    /// Raises the `update` event to registered listeners with animation time.
    pub fn raise_update_event(&self, time: f64) {
        let name = self.name.as_deref().unwrap_or_default();
        for listener in &self.update_listeners {
            listener(name, time);
        }
    }

    /// Raises the `stop` event to registered listeners.
    pub fn raise_stop_event(&self) {
        let name = self.name.as_deref().unwrap_or_default();
        for listener in &self.stop_listeners {
            listener(name);
        }
    }

    /// Adds a callback invoked when animation starts.
    pub fn add_start_listener<F>(&mut self, f: F)
    where
        F: Fn(&str) + Send + Sync + 'static,
    {
        self.start_listeners.push(Arc::new(f));
    }

    /// Adds a callback invoked each frame animation updates.
    pub fn add_update_listener<F>(&mut self, f: F)
    where
        F: Fn(&str, f64) + Send + Sync + 'static,
    {
        self.update_listeners.push(Arc::new(f));
    }

    /// Adds a callback invoked when animation stops.
    pub fn add_stop_listener<F>(&mut self, f: F)
    where
        F: Fn(&str) + Send + Sync + 'static,
    {
        self.stop_listeners.push(Arc::new(f));
    }
}
