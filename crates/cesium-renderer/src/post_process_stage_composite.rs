use std::collections::HashMap;
use std::rc::Rc;
use std::sync::atomic::{AtomicU64, Ordering};

use cesium_math::DeveloperError;

use crate::post_process_stage::{PostProcessStage, PostProcessUniformValue};
use crate::texture::Texture;

static NEXT_COMPOSITE_ID: AtomicU64 = AtomicU64::new(1);

/// An element contained within a composite, which can either be a leaf stage or a nested composite.
#[derive(Clone, Debug)]
pub enum PostProcessStageNode {
    Stage(PostProcessStage),
    Composite(PostProcessStageComposite),
}

impl PostProcessStageNode {
    pub fn name(&self) -> &str {
        match self {
            Self::Stage(s) => &s.name,
            Self::Composite(c) => &c.name,
        }
    }

    pub fn is_destroyed(&self) -> bool {
        match self {
            Self::Stage(s) => s.is_destroyed(),
            Self::Composite(c) => c.is_destroyed(),
        }
    }

    pub fn destroy(&mut self) {
        match self {
            Self::Stage(s) => s.destroy(),
            Self::Composite(c) => c.destroy(),
        }
    }

    pub fn is_ready(&self) -> bool {
        match self {
            Self::Stage(s) => s.ready,
            Self::Composite(c) => c.ready,
        }
    }

    pub fn is_enabled(&self) -> bool {
        match self {
            Self::Stage(s) => s.enabled,
            Self::Composite(c) => c.enabled,
        }
    }

    pub fn as_stage(&self) -> Option<&PostProcessStage> {
        match self {
            Self::Stage(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_stage_mut(&mut self) -> Option<&mut PostProcessStage> {
        match self {
            Self::Stage(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_composite(&self) -> Option<&PostProcessStageComposite> {
        match self {
            Self::Composite(c) => Some(c),
            _ => None,
        }
    }

    pub fn as_composite_mut(&mut self) -> Option<&mut PostProcessStageComposite> {
        match self {
            Self::Composite(c) => Some(c),
            _ => None,
        }
    }
}

/// Options for constructing a PostProcessStageComposite.
#[derive(Clone, Debug)]
pub struct PostProcessStageCompositeOptions {
    pub stages: Vec<PostProcessStageNode>,
    pub input_previous_stage_texture: Option<bool>,
    pub name: Option<String>,
    pub uniforms: Option<HashMap<String, PostProcessUniformValue>>,
}

impl PostProcessStageCompositeOptions {
    pub fn new(stages: Vec<PostProcessStageNode>) -> Self {
        Self {
            stages,
            input_previous_stage_texture: None,
            name: None,
            uniforms: None,
        }
    }
}

/// A collection of PostProcessStages or other composites that execute together logically.
#[derive(Clone, Debug)]
pub struct PostProcessStageComposite {
    pub stages: Vec<PostProcessStageNode>,
    pub input_previous_stage_texture: bool,
    pub name: String,
    pub uniforms: Option<HashMap<String, PostProcessUniformValue>>,
    pub enabled: bool,
    pub ready: bool,
    pub output_texture: Option<Rc<Texture>>,
    is_destroyed: bool,
}

impl PostProcessStageComposite {
    /// Creates a new composite of post process stages.
    pub fn new(options: PostProcessStageCompositeOptions) -> Result<Self, DeveloperError> {
        if options.stages.is_empty() {
            return Err(DeveloperError::new(
                "options.stages.length must be greater than 0.",
            ));
        }

        let input_previous_stage_texture = options.input_previous_stage_texture.unwrap_or(true);
        let name = options.name.unwrap_or_else(|| {
            let id = NEXT_COMPOSITE_ID.fetch_add(1, Ordering::Relaxed);
            format!("composite_{id}")
        });

        Ok(Self {
            stages: options.stages,
            input_previous_stage_texture,
            name,
            uniforms: options.uniforms,
            enabled: true,
            ready: false,
            output_texture: None,
            is_destroyed: false,
        })
    }

    /// Number of direct child stages/composites.
    pub fn len(&self) -> usize {
        self.stages.len()
    }

    /// Whether stages list is empty.
    pub fn is_empty(&self) -> bool {
        self.stages.is_empty()
    }

    /// Returns a reference to the stage at the given index.
    pub fn get(&self, index: usize) -> Result<&PostProcessStageNode, DeveloperError> {
        self.stages
            .get(index)
            .ok_or_else(|| DeveloperError::new("Index out of bounds."))
    }

    /// Returns a mutable reference to the stage at the given index.
    pub fn get_mut(&mut self, index: usize) -> Result<&mut PostProcessStageNode, DeveloperError> {
        self.stages
            .get_mut(index)
            .ok_or_else(|| DeveloperError::new("Index out of bounds."))
    }

    /// Determines if all stages in this composite are ready.
    pub fn check_ready(&mut self) -> bool {
        let all_ready = self.stages.iter().all(|s| s.is_ready());
        self.ready = all_ready;
        all_ready
    }

    /// Destroys this composite and all contained stages.
    pub fn destroy(&mut self) {
        self.is_destroyed = true;
        for stage in &mut self.stages {
            stage.destroy();
        }
        self.output_texture = None;
    }

    /// Returns true if destroyed.
    pub fn is_destroyed(&self) -> bool {
        self.is_destroyed
    }
}
