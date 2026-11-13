//! ModelAnimationCollection matching CesiumJS `Scene/Model/ModelAnimationCollection.js` (Apache-2.0).

use cesium_math::{clamp, DeveloperError, JulianDate};
use crate::model_animation_channel::ModelRuntimeNode;
pub use crate::model_animation::{
    AnimationCollectionCallback, AnimationMetadata, AnimationStateCallback, AnimationTimeCallback,
    AnimationUpdateCallback, ModelAnimation, ModelAnimationLoop, ModelAnimationOptions,
    ModelAnimationState,
};

/// Alias for compatibility with collection-specific code.
pub type RuntimeModelAnimation = ModelAnimation;

/// A collection of active model animations matching CesiumJS `ModelAnimationCollection`.
#[derive(Clone, Default)]
pub struct ModelAnimationCollection {
    pub model_ready: bool,
    pub model_clamp_animations: bool,
    pub animate_while_paused: bool,
    pub animations_metadata: Vec<AnimationMetadata>,
    pub runtime_animations: Vec<RuntimeModelAnimation>,
    pub previous_time: Option<JulianDate>,
    pub animation_added_listeners: Vec<AnimationCollectionCallback>,
    pub animation_removed_listeners: Vec<AnimationCollectionCallback>,
}

impl std::fmt::Debug for ModelAnimationCollection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ModelAnimationCollection")
            .field("model_ready", &self.model_ready)
            .field("model_clamp_animations", &self.model_clamp_animations)
            .field("animate_while_paused", &self.animate_while_paused)
            .field("animations_metadata", &self.animations_metadata)
            .field("runtime_animations_count", &self.runtime_animations.len())
            .field("previous_time", &self.previous_time)
            .finish()
    }
}

impl ModelAnimationCollection {
    pub fn new(model_ready: bool, model_clamp_animations: bool) -> Self {
        Self {
            model_ready,
            model_clamp_animations,
            animate_while_paused: false,
            animations_metadata: Vec::new(),
            runtime_animations: Vec::new(),
            previous_time: None,
            animation_added_listeners: Vec::new(),
            animation_removed_listeners: Vec::new(),
        }
    }

    pub fn len(&self) -> usize {
        self.runtime_animations.len()
    }

    pub fn is_empty(&self) -> bool {
        self.runtime_animations.is_empty()
    }

    pub fn get(&self, index: usize) -> Result<&RuntimeModelAnimation, DeveloperError> {
        if index >= self.runtime_animations.len() {
            return Err(DeveloperError::new(
                "index must be valid within the range of the collection",
            ));
        }
        Ok(&self.runtime_animations[index])
    }

    pub fn get_mut(&mut self, index: usize) -> Result<&mut RuntimeModelAnimation, DeveloperError> {
        if index >= self.runtime_animations.len() {
            return Err(DeveloperError::new(
                "index must be valid within the range of the collection",
            ));
        }
        Ok(&mut self.runtime_animations[index])
    }

    pub fn contains_index(&self, index: usize) -> bool {
        self.runtime_animations.iter().any(|a| a.index == index)
    }

    pub fn contains_name(&self, name: &str) -> bool {
        self.runtime_animations.iter().any(|a| a.name.as_deref() == Some(name))
    }

    pub fn add(&mut self, options: ModelAnimationOptions) -> Result<usize, DeveloperError> {
        if !self.model_ready {
            return Err(DeveloperError::new(
                "Animations are not loaded.  Wait for Model.ready to be true.",
            ));
        }

        if options.name.is_none() && options.index.is_none() {
            return Err(DeveloperError::new(
                "Either options.name or options.index must be defined.",
            ));
        }

        if options.multiplier <= 0.0 {
            return Err(DeveloperError::new(
                "options.multiplier must be greater than zero.",
            ));
        }

        let meta_index = if let Some(idx) = options.index {
            if idx >= self.animations_metadata.len() {
                return Err(DeveloperError::new(
                    "options.index must be a valid animation index.",
                ));
            }
            idx
        } else if let Some(ref name) = options.name {
            self.animations_metadata
                .iter()
                .position(|m| m.name.as_deref() == Some(name))
                .ok_or_else(|| {
                    DeveloperError::new("options.name must be a valid animation name.")
                })?
        } else {
            return Err(DeveloperError::new("options must specify name or index."));
        };

        let metadata = &self.animations_metadata[meta_index];
        let anim = ModelAnimation::from_metadata(metadata, options, self.model_clamp_animations)?;
        let name_str = anim.name.clone().unwrap_or_default();
        let anim_index = anim.index;

        self.runtime_animations.push(anim);
        let new_idx = self.runtime_animations.len() - 1;

        for listener in &self.animation_added_listeners {
            listener(anim_index, &name_str);
        }

        Ok(new_idx)
    }

    pub fn add_all(&mut self, options: ModelAnimationOptions) -> Result<Vec<usize>, DeveloperError> {
        if !self.model_ready {
            return Err(DeveloperError::new(
                "Animations are not loaded. Wait for the Model.ready to return true.",
            ));
        }

        if options.multiplier <= 0.0 {
            return Err(DeveloperError::new(
                "options.multiplier must be greater than zero.",
            ));
        }

        let mut added_indices = Vec::new();
        let count = self.animations_metadata.len();
        for i in 0..count {
            let opt = ModelAnimationOptions {
                name: None,
                index: Some(i),
                start_time: options.start_time,
                delay: options.delay,
                stop_time: options.stop_time,
                remove_on_stop: options.remove_on_stop,
                multiplier: options.multiplier,
                reverse: options.reverse,
                loop_mode: options.loop_mode,
                animation_time: None,
            };
            let idx = self.add(opt)?;
            added_indices.push(idx);
        }

        Ok(added_indices)
    }

    pub fn remove(&mut self, index: usize) -> bool {
        if index >= self.runtime_animations.len() {
            return false;
        }

        let anim = self.runtime_animations.remove(index);
        let name_str = anim.name.unwrap_or_default();
        for listener in &self.animation_removed_listeners {
            listener(anim.index, &name_str);
        }
        true
    }

    pub fn remove_all(&mut self) {
        let removed = std::mem::take(&mut self.runtime_animations);
        for anim in removed {
            let name_str = anim.name.unwrap_or_default();
            for listener in &self.animation_removed_listeners {
                listener(anim.index, &name_str);
            }
        }
    }

    pub fn update(
        &mut self,
        current_time: JulianDate,
        runtime_nodes: &mut [ModelRuntimeNode],
    ) -> Result<bool, DeveloperError> {
        let length = self.runtime_animations.len();
        if length == 0 {
            self.previous_time = None;
            return Ok(false);
        }

        if !self.animate_while_paused && self.previous_time == Some(current_time) {
            return Ok(false);
        }
        self.previous_time = Some(current_time);

        let mut animation_occurred = false;
        let mut to_remove_indices = Vec::new();

        for (i, runtime_animation) in self.runtime_animations.iter_mut().enumerate() {
            if runtime_animation.computed_start_time.is_none() {
                let base_start = runtime_animation.start_time.unwrap_or(current_time);
                runtime_animation.computed_start_time = Some(base_start.add_seconds(runtime_animation.delay));
            }

            if runtime_animation.duration.is_none() {
                runtime_animation.duration = Some(
                    runtime_animation.local_stop_time * (1.0 / runtime_animation.multiplier),
                );
            }

            let start_time = runtime_animation.computed_start_time.unwrap();
            let duration = runtime_animation.duration.unwrap();
            let stop_time = runtime_animation.stop_time;

            let past_start_time = start_time <= current_time;
            let reached_stop_time = stop_time.is_some_and(|st| current_time > st);

            let mut delta = 0.0;
            if duration != 0.0 {
                let effective_time = if reached_stop_time { stop_time.unwrap() } else { current_time };
                let seconds = effective_time.seconds_difference(&start_time);
                delta = if let Some(ref custom_anim_time) = runtime_animation.animation_time {
                    custom_anim_time(duration, seconds)
                } else {
                    seconds / duration
                };
            }

            let repeat = runtime_animation.loop_mode == ModelAnimationLoop::Repeat
                || runtime_animation.loop_mode == ModelAnimationLoop::MirroredRepeat;

            let play = (past_start_time || (repeat && runtime_animation.start_time.is_none()))
                && (delta <= 1.0 || repeat)
                && !reached_stop_time;

            if Some(delta) == runtime_animation.prev_animation_delta {
                let animation_stopped = runtime_animation.state == ModelAnimationState::Stopped;
                if play != animation_stopped {
                    continue;
                }
            }
            runtime_animation.prev_animation_delta = Some(delta);

            if play || runtime_animation.state == ModelAnimationState::Animating {
                if play && runtime_animation.state == ModelAnimationState::Stopped {
                    runtime_animation.state = ModelAnimationState::Animating;
                    let name = runtime_animation.name.clone().unwrap_or_default();
                    for listener in &runtime_animation.start_listeners {
                        listener(&name);
                    }
                }

                if runtime_animation.loop_mode == ModelAnimationLoop::Repeat {
                    delta -= delta.floor();
                } else if runtime_animation.loop_mode == ModelAnimationLoop::MirroredRepeat {
                    let floor = delta.floor();
                    let fract = delta - floor;
                    delta = if (floor as i64) % 2 == 1 { 1.0 - fract } else { fract };
                }

                if runtime_animation.reverse {
                    delta = 1.0 - delta;
                }

                let mut local_animation_time = delta * duration * runtime_animation.multiplier;
                local_animation_time = clamp(
                    local_animation_time,
                    runtime_animation.local_start_time,
                    runtime_animation.local_stop_time,
                );

                for node in runtime_nodes.iter_mut() {
                    runtime_animation.animate_node(node, local_animation_time)?;
                }

                let name = runtime_animation.name.clone().unwrap_or_default();
                for listener in &runtime_animation.update_listeners {
                    listener(&name, local_animation_time);
                }
                animation_occurred = true;

                if !play {
                    runtime_animation.state = ModelAnimationState::Stopped;
                    for listener in &runtime_animation.stop_listeners {
                        listener(&name);
                    }

                    if runtime_animation.remove_on_stop {
                        to_remove_indices.push(i);
                    }
                }
            }
        }

        // Remove animations in reverse order
        for &idx in to_remove_indices.iter().rev() {
            self.remove(idx);
        }

        Ok(animation_occurred)
    }
}
