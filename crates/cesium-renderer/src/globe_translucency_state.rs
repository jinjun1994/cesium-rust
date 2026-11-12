//! GlobeTranslucencyState matching CesiumJS `Scene/GlobeTranslucencyState.js`.

use cesium_math::{Color, NearFarScalar, Rectangle, SceneMode};

use crate::commands::{DrawCommand, Pass};

/// Derived command types generated for globe translucency rendering.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[repr(u8)]
pub enum DerivedCommandType {
    OpaqueFrontFace = 0,
    OpaqueBackFace = 1,
    DepthOnlyFrontFace = 2,
    DepthOnlyBackFace = 3,
    DepthOnlyFrontAndBackFace = 4,
    TranslucentFrontFace = 5,
    TranslucentBackFace = 6,
    TranslucentFrontFaceManualDepthTest = 7,
    TranslucentBackFaceManualDepthTest = 8,
    PickFrontFace = 9,
    PickBackFace = 10,
}

const DERIVED_COMMANDS_MAXIMUM_LENGTH: usize = 11;

const ALL_DERIVED_COMMAND_TYPES: [DerivedCommandType; DERIVED_COMMANDS_MAXIMUM_LENGTH] = [
    DerivedCommandType::OpaqueFrontFace,
    DerivedCommandType::OpaqueBackFace,
    DerivedCommandType::DepthOnlyFrontFace,
    DerivedCommandType::DepthOnlyBackFace,
    DerivedCommandType::DepthOnlyFrontAndBackFace,
    DerivedCommandType::TranslucentFrontFace,
    DerivedCommandType::TranslucentBackFace,
    DerivedCommandType::TranslucentFrontFaceManualDepthTest,
    DerivedCommandType::TranslucentBackFaceManualDepthTest,
    DerivedCommandType::PickFrontFace,
    DerivedCommandType::PickBackFace,
];

/// Globe translucency settings matching CesiumJS `GlobeTranslucency`.
#[derive(Clone, Debug, PartialEq)]
pub struct GlobeTranslucencyConfig {
    pub enabled: bool,
    pub front_face_alpha: f64,
    pub front_face_alpha_by_distance: Option<NearFarScalar>,
    pub back_face_alpha: f64,
    pub back_face_alpha_by_distance: Option<NearFarScalar>,
    pub rectangle: Rectangle,
}

impl Default for GlobeTranslucencyConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            front_face_alpha: 1.0,
            front_face_alpha_by_distance: None,
            back_face_alpha: 1.0,
            back_face_alpha_by_distance: None,
            rectangle: Rectangle::MAXIMUM,
        }
    }
}

/// Scene and globe snapshot used to update `GlobeTranslucencyState`.
#[derive(Clone, Debug, PartialEq)]
pub struct GlobeTranslucencyInput {
    pub globe_present: bool,
    pub globe_show: bool,
    pub base_color: Color,
    pub depth_test_against_terrain: bool,
    pub translucency: GlobeTranslucencyConfig,
    pub camera_underground: bool,
    pub mode: SceneMode,
    pub depth_texture_supported: bool,
}

impl Default for GlobeTranslucencyInput {
    fn default() -> Self {
        Self {
            globe_present: true,
            globe_show: true,
            base_color: Color::WHITE,
            depth_test_against_terrain: false,
            translucency: GlobeTranslucencyConfig::default(),
            camera_underground: false,
            mode: SceneMode::Scene3D,
            depth_texture_supported: true,
        }
    }
}

/// Tracks globe translucency state and derived draw command requirements per frame.
#[derive(Clone, Debug)]
pub struct GlobeTranslucencyState {
    front_face_alpha_by_distance: NearFarScalar,
    back_face_alpha_by_distance: NearFarScalar,
    front_face_translucent: bool,
    back_face_translucent: bool,
    requires_manual_depth_test: bool,
    sun_visible_through_globe: bool,
    environment_visible: bool,
    use_depth_plane: bool,
    number_of_texture_uniforms: usize,
    rectangle: Rectangle,
    derived_command_key: u32,
    derived_commands_dirty: bool,
    derived_command_types: Vec<DerivedCommandType>,
    derived_blend_command_types: Vec<DerivedCommandType>,
    derived_pick_command_types: Vec<DerivedCommandType>,
    derived_command_types_to_update: Vec<DerivedCommandType>,
}

impl Default for GlobeTranslucencyState {
    fn default() -> Self {
        Self::new()
    }
}

impl GlobeTranslucencyState {
    /// Creates a new `GlobeTranslucencyState` with opaque defaults.
    pub fn new() -> Self {
        Self {
            front_face_alpha_by_distance: NearFarScalar::new(0.0, 1.0, 0.0, 1.0),
            back_face_alpha_by_distance: NearFarScalar::new(0.0, 1.0, 0.0, 1.0),
            front_face_translucent: false,
            back_face_translucent: false,
            requires_manual_depth_test: false,
            sun_visible_through_globe: false,
            environment_visible: false,
            use_depth_plane: false,
            number_of_texture_uniforms: 0,
            rectangle: Rectangle::MAXIMUM,
            derived_command_key: 0,
            derived_commands_dirty: false,
            derived_command_types: Vec::new(),
            derived_blend_command_types: Vec::new(),
            derived_pick_command_types: Vec::new(),
            derived_command_types_to_update: Vec::new(),
        }
    }

    pub fn front_face_alpha_by_distance(&self) -> &NearFarScalar {
        &self.front_face_alpha_by_distance
    }

    pub fn back_face_alpha_by_distance(&self) -> &NearFarScalar {
        &self.back_face_alpha_by_distance
    }

    pub fn translucent(&self) -> bool {
        self.front_face_translucent
    }

    pub fn back_face_translucent(&self) -> bool {
        self.back_face_translucent
    }

    pub fn requires_manual_depth_test(&self) -> bool {
        self.requires_manual_depth_test
    }

    pub fn sun_visible_through_globe(&self) -> bool {
        self.sun_visible_through_globe
    }

    pub fn environment_visible(&self) -> bool {
        self.environment_visible
    }

    pub fn use_depth_plane(&self) -> bool {
        self.use_depth_plane
    }

    pub fn number_of_texture_uniforms(&self) -> usize {
        self.number_of_texture_uniforms
    }

    pub fn rectangle(&self) -> &Rectangle {
        &self.rectangle
    }

    pub fn derived_commands_dirty(&self) -> bool {
        self.derived_commands_dirty
    }

    pub fn derived_command_types(&self) -> &[DerivedCommandType] {
        &self.derived_command_types
    }

    pub fn derived_blend_command_types(&self) -> &[DerivedCommandType] {
        &self.derived_blend_command_types
    }

    pub fn derived_pick_command_types(&self) -> &[DerivedCommandType] {
        &self.derived_pick_command_types
    }

    pub fn derived_command_types_to_update(&self) -> &[DerivedCommandType] {
        &self.derived_command_types_to_update
    }

    /// Updates all translucency flags and derived command lists from the scene/globe state.
    pub fn update(&mut self, input: &GlobeTranslucencyInput) {
        if !input.globe_present || !input.globe_show {
            self.front_face_translucent = false;
            self.back_face_translucent = false;
            self.requires_manual_depth_test = false;
            self.sun_visible_through_globe = true;
            self.environment_visible = true;
            self.use_depth_plane = false;
            self.number_of_texture_uniforms = 0;
            self.derived_command_types.clear();
            self.derived_blend_command_types.clear();
            self.derived_pick_command_types.clear();
            self.derived_command_types_to_update.clear();
            self.derived_commands_dirty = self.derived_command_key != 0;
            self.derived_command_key = 0;
            return;
        }

        self.front_face_alpha_by_distance = update_alpha_by_distance(
            input.translucency.enabled,
            input.translucency.front_face_alpha,
            input.translucency.front_face_alpha_by_distance.as_ref(),
        );
        self.back_face_alpha_by_distance = update_alpha_by_distance(
            input.translucency.enabled,
            input.translucency.back_face_alpha,
            input.translucency.back_face_alpha_by_distance.as_ref(),
        );

        self.front_face_translucent = is_face_translucent(
            input.translucency.enabled,
            &self.front_face_alpha_by_distance,
            input.base_color,
        );
        self.back_face_translucent = is_face_translucent(
            input.translucency.enabled,
            &self.back_face_alpha_by_distance,
            input.base_color,
        );

        self.requires_manual_depth_test = self.front_face_translucent
            && !self.back_face_translucent
            && !input.depth_test_against_terrain
            && input.mode != SceneMode::Scene2D
            && input.depth_texture_supported;

        self.sun_visible_through_globe = self.front_face_translucent
            && (input.camera_underground || self.back_face_translucent);
        self.environment_visible = !input.camera_underground || self.front_face_translucent;
        self.use_depth_plane = !input.camera_underground && !self.front_face_translucent;

        let mut num_uniforms = 0;
        if self.front_face_translucent {
            num_uniforms += 1;
        }
        if self.requires_manual_depth_test {
            num_uniforms += 1;
        }
        self.number_of_texture_uniforms = num_uniforms;

        self.rectangle = input.translucency.rectangle;
        self.gather_derived_command_requirements(input);
    }

    fn gather_derived_command_requirements(&mut self, input: &GlobeTranslucencyInput) {
        self.derived_command_types = self.compute_derived_types(input, false, false);
        self.derived_blend_command_types = self.compute_derived_types(input, true, false);
        self.derived_pick_command_types = self.compute_derived_types(input, false, true);

        let mut key: u32 = 0;
        for &ty in &self.derived_command_types {
            key |= 1 << (ty as u8);
        }
        for &ty in &self.derived_blend_command_types {
            key |= 1 << (ty as u8);
        }
        for &ty in &self.derived_pick_command_types {
            key |= 1 << (ty as u8);
        }

        self.derived_command_types_to_update.clear();
        for ty in ALL_DERIVED_COMMAND_TYPES {
            if (key & (1 << (ty as u8))) > 0 {
                self.derived_command_types_to_update.push(ty);
            }
        }

        self.derived_commands_dirty = key != self.derived_command_key;
        self.derived_command_key = key;
    }

    fn compute_derived_types(
        &self,
        input: &GlobeTranslucencyInput,
        is_blend_command: bool,
        is_pick_command: bool,
    ) -> Vec<DerivedCommandType> {
        let mut types = Vec::new();
        if !self.front_face_translucent {
            return types;
        }

        let translucent_front = if is_pick_command {
            DerivedCommandType::PickFrontFace
        } else if self.requires_manual_depth_test {
            DerivedCommandType::TranslucentFrontFaceManualDepthTest
        } else {
            DerivedCommandType::TranslucentFrontFace
        };

        let translucent_back = if is_pick_command {
            DerivedCommandType::PickBackFace
        } else if self.requires_manual_depth_test {
            DerivedCommandType::TranslucentBackFaceManualDepthTest
        } else {
            DerivedCommandType::TranslucentBackFace
        };

        if input.mode == SceneMode::Scene2D {
            types.push(DerivedCommandType::DepthOnlyFrontFace);
            types.push(translucent_front);
            return types;
        }

        if self.back_face_translucent {
            if !is_blend_command {
                types.push(DerivedCommandType::DepthOnlyFrontAndBackFace);
            }
            if input.camera_underground {
                types.push(translucent_front);
                types.push(translucent_back);
            } else {
                types.push(translucent_back);
                types.push(translucent_front);
            }
        } else if input.camera_underground {
            if !is_blend_command {
                types.push(DerivedCommandType::DepthOnlyBackFace);
            }
            types.push(DerivedCommandType::OpaqueFrontFace);
            types.push(translucent_back);
        } else {
            if !is_blend_command {
                types.push(DerivedCommandType::DepthOnlyFrontFace);
            }
            types.push(DerivedCommandType::OpaqueBackFace);
            types.push(translucent_front);
        }

        types
    }

    /// Produces derived `DrawCommand` instances for a base globe tile command.
    pub fn derive_commands(
        &self,
        base_command: &DrawCommand,
        is_blend_command: bool,
        is_pick_command: bool,
    ) -> Vec<(DerivedCommandType, DrawCommand)> {
        let types = if is_pick_command {
            &self.derived_pick_command_types
        } else if is_blend_command {
            &self.derived_blend_command_types
        } else {
            &self.derived_command_types
        };

        types
            .iter()
            .map(|&ty| {
                let mut cmd = base_command.clone();
                cmd.pass = Some(match ty {
                    DerivedCommandType::OpaqueFrontFace
                    | DerivedCommandType::OpaqueBackFace
                    | DerivedCommandType::DepthOnlyFrontFace
                    | DerivedCommandType::DepthOnlyBackFace
                    | DerivedCommandType::DepthOnlyFrontAndBackFace => Pass::Globe,
                    DerivedCommandType::TranslucentFrontFace
                    | DerivedCommandType::TranslucentBackFace
                    | DerivedCommandType::TranslucentFrontFaceManualDepthTest
                    | DerivedCommandType::TranslucentBackFaceManualDepthTest
                    | DerivedCommandType::PickFrontFace
                    | DerivedCommandType::PickBackFace => Pass::Translucent,
                });
                (ty, cmd)
            })
            .collect()
    }
}

fn update_alpha_by_distance(
    enabled: bool,
    alpha: f64,
    alpha_by_distance: Option<&NearFarScalar>,
) -> NearFarScalar {
    if !enabled {
        return NearFarScalar::new(0.0, 1.0, 0.0, 1.0);
    }

    let Some(src) = alpha_by_distance else {
        return NearFarScalar::new(0.0, alpha, 0.0, alpha);
    };

    NearFarScalar::new(
        src.near,
        src.near_value * alpha,
        src.far,
        src.far_value * alpha,
    )
}

fn is_face_translucent(
    translucency_enabled: bool,
    alpha_by_distance: &NearFarScalar,
    base_color: Color,
) -> bool {
    translucency_enabled
        && (base_color.alpha < 1.0
            || alpha_by_distance.near_value < 1.0
            || alpha_by_distance.far_value < 1.0)
}
