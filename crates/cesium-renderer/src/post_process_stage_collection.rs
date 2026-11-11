use std::rc::Rc;

use cesium_math::DeveloperError;

use crate::post_process_stage::PostProcessStage;
use crate::post_process_stage_composite::{PostProcessStageComposite, PostProcessStageNode};
use crate::post_process_stage_library::PostProcessStageLibrary;
use crate::texture::{Texture, TextureOptions};

/// Tonemapping algorithm when rendering with high dynamic range.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub enum Tonemapper {
    Reinhard,
    ModifiedReinhard,
    Filmic,
    Aces,
    #[default]
    PbrNeutral,
}

impl std::str::FromStr for Tonemapper {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "REINHARD" => Ok(Self::Reinhard),
            "MODIFIED_REINHARD" => Ok(Self::ModifiedReinhard),
            "FILMIC" => Ok(Self::Filmic),
            "ACES" => Ok(Self::Aces),
            "PBR_NEUTRAL" => Ok(Self::PbrNeutral),
            _ => Err(()),
        }
    }
}

impl Tonemapper {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Reinhard => "REINHARD",
            Self::ModifiedReinhard => "MODIFIED_REINHARD",
            Self::Filmic => "FILMIC",
            Self::Aces => "ACES",
            Self::PbrNeutral => "PBR_NEUTRAL",
        }
    }

    pub fn validate(s: &str) -> bool {
        s.parse::<Self>().is_ok()
    }
}

/// A collection of PostProcessStages and/or PostProcessStageComposites.
#[derive(Clone, Debug)]
pub struct PostProcessStageCollection {
    pub fxaa: PostProcessStage,
    pub ambient_occlusion: PostProcessStageComposite,
    pub bloom: PostProcessStageComposite,
    pub auto_exposure: PostProcessStageComposite,
    pub tonemapping: PostProcessStage,
    pub tonemapper: Tonemapper,
    pub exposure: f64,
    pub stages: Vec<PostProcessStageNode>,
    pub output_texture: Option<Rc<Texture>>,
}

impl Default for PostProcessStageCollection {
    fn default() -> Self {
        Self::new()
    }
}

impl PostProcessStageCollection {
    /// Creates a new PostProcessStageCollection.
    pub fn new() -> Self {
        let mut fxaa = PostProcessStageLibrary::create_fxaa_stage();
        fxaa.enabled = false;

        let mut ambient_occlusion = PostProcessStageLibrary::create_ambient_occlusion_stage();
        ambient_occlusion.enabled = false;

        let mut bloom = PostProcessStageLibrary::create_bloom_stage();
        bloom.enabled = false;

        let mut auto_exposure = PostProcessStageLibrary::create_auto_exposure_stage();
        auto_exposure.enabled = false;

        let tonemapper = Tonemapper::PbrNeutral;
        let mut tonemapping = PostProcessStageLibrary::create_pbr_neutral_tonemapping_stage(None);
        tonemapping.enabled = false;

        Self {
            fxaa,
            ambient_occlusion,
            bloom,
            auto_exposure,
            tonemapping,
            tonemapper,
            exposure: 1.0,
            stages: Vec::new(),
            output_texture: None,
        }
    }

    /// Sets the tonemapper and creates the corresponding tonemapping stage.
    pub fn set_tonemapper(&mut self, tonemapper: Tonemapper) {
        self.tonemapper = tonemapper;
        let enabled = self.tonemapping.enabled;
        let mut stage = match tonemapper {
            Tonemapper::Reinhard => {
                PostProcessStageLibrary::create_reinhard_tonemapping_stage(None)
            }
            Tonemapper::ModifiedReinhard => {
                PostProcessStageLibrary::create_modified_reinhard_tonemapping_stage(None)
            }
            Tonemapper::Filmic => PostProcessStageLibrary::create_filmic_tonemapping_stage(None),
            Tonemapper::Aces => PostProcessStageLibrary::create_aces_tonemapping_stage(None),
            Tonemapper::PbrNeutral => {
                PostProcessStageLibrary::create_pbr_neutral_tonemapping_stage(None)
            }
        };
        stage.enabled = enabled;
        self.tonemapping = stage;
    }

    /// Determines if all enabled post-process stages are ready to be executed.
    pub fn ready(&self) -> bool {
        let mut has_active = false;
        if self.fxaa.enabled {
            has_active = true;
            if !self.fxaa.ready {
                return false;
            }
        }
        if self.ambient_occlusion.enabled {
            has_active = true;
            if !self.ambient_occlusion.ready {
                return false;
            }
        }
        if self.bloom.enabled {
            has_active = true;
            if !self.bloom.ready {
                return false;
            }
        }
        if self.tonemapping.enabled {
            has_active = true;
            if !self.tonemapping.ready {
                return false;
            }
        }
        for s in &self.stages {
            if s.is_enabled() {
                has_active = true;
                if !s.is_ready() {
                    return false;
                }
            }
        }
        has_active
    }

    /// Returns the number of user stages in the collection.
    pub fn len(&self) -> usize {
        self.stages.len()
    }

    /// Whether there are no user stages.
    pub fn is_empty(&self) -> bool {
        self.stages.is_empty()
    }

    /// Adds a stage or composite to the collection.
    pub fn add(&mut self, stage: PostProcessStageNode) -> Result<PostProcessStageNode, DeveloperError> {
        let name = stage.name();
        if self.contains(name) {
            return Err(DeveloperError::new(
                "The post-process stage is already in the collection.",
            ));
        }
        self.stages.push(stage.clone());
        Ok(stage)
    }

    /// Checks if a stage with the given name is present in the collection.
    pub fn contains(&self, name: &str) -> bool {
        self.stages.iter().any(|s| s.name() == name)
    }

    /// Checks if a stage node is present in the collection.
    pub fn contains_stage(&self, stage: &PostProcessStageNode) -> bool {
        self.contains(stage.name())
    }

    /// Removes a stage from the collection by name and destroys it.
    pub fn remove(&mut self, name: &str) -> bool {
        if let Some(pos) = self.stages.iter().position(|s| s.name() == name) {
            let mut stage = self.stages.remove(pos);
            stage.destroy();
            true
        } else {
            false
        }
    }

    /// Removes a stage from the collection and destroys it.
    pub fn remove_stage(&mut self, stage: &PostProcessStageNode) -> bool {
        self.remove(stage.name())
    }

    /// Removes all user stages from the collection and destroys them.
    pub fn remove_all(&mut self) {
        for mut s in self.stages.drain(..) {
            s.destroy();
        }
    }

    /// Gets the stage at the given index.
    pub fn get(&self, index: usize) -> Result<&PostProcessStageNode, DeveloperError> {
        self.stages
            .get(index)
            .ok_or_else(|| DeveloperError::new("Index out of bounds."))
    }

    /// Gets a mutable reference to the stage at the given index.
    pub fn get_mut(&mut self, index: usize) -> Result<&mut PostProcessStageNode, DeveloperError> {
        self.stages
            .get_mut(index)
            .ok_or_else(|| DeveloperError::new("Index out of bounds."))
    }

    /// Finds a stage by name (searching built-in stages and user stages, and their children).
    pub fn get_stage_by_name(&self, name: &str) -> Option<&PostProcessStageNode> {
        self.stages.iter().find(|s| s.name() == name)
    }

    /// Gets the output texture for a given stage by name.
    pub fn get_output_texture(&self, name: &str) -> Option<Rc<Texture>> {
        // Builtin FXAA / AO / Bloom
        if self.fxaa.name == name && self.fxaa.enabled {
            if let Some(tex) = &self.fxaa.output_texture {
                return Some(tex.clone());
            }
            // If user has stages, FXAA shares output with the last executed stage
            if let Some(last) = self.stages.last() {
                match last {
                    PostProcessStageNode::Stage(s) => {
                        if let Some(tex) = &s.output_texture {
                            return Some(tex.clone());
                        }
                    }
                    PostProcessStageNode::Composite(c) => {
                        if let Some(tex) = &c.output_texture {
                            return Some(tex.clone());
                        }
                    }
                }
            }
            return Some(Rc::new(Texture::new(TextureOptions::default())));
        }
        if self.ambient_occlusion.name == name && self.ambient_occlusion.enabled {
            return self.ambient_occlusion.output_texture.clone().or_else(|| {
                Some(Rc::new(Texture::new(TextureOptions::default())))
            });
        }
        if self.bloom.name == name && self.bloom.enabled {
            return self.bloom.output_texture.clone().or_else(|| {
                Some(Rc::new(Texture::new(TextureOptions::default())))
            });
        }

        // Search user stages
        if let Some(node) = self.stages.iter().find(|s| s.name() == name) {
            match node {
                PostProcessStageNode::Stage(s) => s.output_texture.clone().or_else(|| {
                    Some(Rc::new(Texture::new(TextureOptions::default())))
                }),
                PostProcessStageNode::Composite(c) => c.output_texture.clone().or_else(|| {
                    Some(Rc::new(Texture::new(TextureOptions::default())))
                }),
            }
        } else {
            None
        }
    }
}
