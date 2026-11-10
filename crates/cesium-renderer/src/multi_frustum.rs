//! MultiFrustum matching CesiumJS `Scene/View.js` multifrustum logic and `Scene/DebugInspector.js` (Apache-2.0).

use std::collections::HashMap;

use cesium_math::{Color, SceneMode, EPSILON2};
use glam::DVec3;

use crate::commands::{DrawCommand, NUMBER_OF_PASSES};
use crate::frustum_commands::FrustumCommands;
use crate::shader_source::ShaderSource;

/// Command extent representing the distance range a DrawCommand spans along the view direction.
#[derive(Clone, Debug, PartialEq)]
pub struct CommandExtent {
    pub command: DrawCommand,
    pub near: f64,
    pub far: f64,
}

/// Statistics collected when `debug_show_frustums` is active.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DebugFrustumStatistics {
    pub total_commands: usize,
    pub commands_in_frustums: HashMap<i32, usize>,
}

/// Options to initialize `MultiFrustum`.
#[derive(Clone, Debug, Default)]
pub struct MultiFrustumOptions {
    pub far_to_near_ratio: Option<f64>,
    pub logarithmic_depth_far_to_near_ratio: Option<f64>,
    pub near_to_far_distance_2d: Option<f64>,
    pub use_log_depth: Option<bool>,
    pub scene_mode: Option<SceneMode>,
    pub debug_show_frustums: Option<bool>,
}

/// MultiFrustum manager handling frustum splits and command binning.
#[derive(Clone, Debug, PartialEq)]
pub struct MultiFrustum {
    /// Active frustum command bins.
    pub frustum_commands_list: Vec<FrustumCommands>,
    /// Splits of frustums near and far distances.
    pub frustum_splits: Vec<f64>,
    /// Ratio of far to near plane in 3D/CV when logarithmic depth is disabled.
    pub far_to_near_ratio: f64,
    /// Ratio of far to near plane in 3D/CV when logarithmic depth is enabled.
    pub logarithmic_depth_far_to_near_ratio: f64,
    /// Uniform depth size of each frustum in meters for 2D mode.
    pub near_to_far_distance_2d: f64,
    /// Whether logarithmic depth buffer is enabled.
    pub use_log_depth: bool,
    /// Current scene mode (2D, ColumbusView, 3D).
    pub scene_mode: SceneMode,
    /// Whether to debug visualize frustum overlaps.
    pub debug_show_frustums: bool,
    /// Frustum statistics recorded when `debug_show_frustums` is enabled.
    pub debug_frustum_statistics: DebugFrustumStatistics,
}

impl Default for MultiFrustum {
    fn default() -> Self {
        Self::new(MultiFrustumOptions::default())
    }
}

impl MultiFrustum {
    /// Creates a new `MultiFrustum` manager with given options.
    pub fn new(options: MultiFrustumOptions) -> Self {
        Self {
            frustum_commands_list: Vec::new(),
            frustum_splits: Vec::new(),
            far_to_near_ratio: options.far_to_near_ratio.unwrap_or(1000.0),
            logarithmic_depth_far_to_near_ratio: options
                .logarithmic_depth_far_to_near_ratio
                .unwrap_or(1e9),
            near_to_far_distance_2d: options.near_to_far_distance_2d.unwrap_or(1.75e6),
            use_log_depth: options.use_log_depth.unwrap_or(false),
            scene_mode: options.scene_mode.unwrap_or(SceneMode::Scene3D),
            debug_show_frustums: options.debug_show_frustums.unwrap_or(false),
            debug_frustum_statistics: DebugFrustumStatistics::default(),
        }
    }

    /// Updates frustum bins based on near and far bounds, matching CesiumJS `View.js#updateFrustums`.
    pub fn update_frustums(
        &mut self,
        camera_near: f64,
        camera_far: f64,
        camera_position_z: f64,
        scene_near: f64,
        scene_far: f64,
    ) {
        let far_to_near_ratio = if self.use_log_depth {
            self.logarithmic_depth_far_to_near_ratio
        } else {
            self.far_to_near_ratio
        };

        let is_2d = self.scene_mode == SceneMode::Scene2D;

        // Extend the far plane slightly further to prevent geometry clipping against the far plane.
        let mut far = scene_far * (1.0 + EPSILON2);
        let mut near = scene_near;

        // The computed near plane must be between the user defined near and far planes.
        // The computed far plane must be between the user defined far and computed near.
        near = near.clamp(camera_near, camera_far);
        far = far.clamp(near, camera_far);

        let num_frustums = if is_2d {
            // The multifrustum for 2D is uniformly distributed.
            far = far.min(camera_position_z + self.near_to_far_distance_2d);
            near = near.min(far);
            let count = ((far - near).max(1.0) / self.near_to_far_distance_2d).ceil();
            if count.is_nan() || count < 1.0 {
                1
            } else {
                count as usize
            }
        } else {
            // The multifrustum for 3D/CV is non-uniformly distributed.
            if near <= 0.0 || far <= near || far_to_near_ratio <= 1.0 {
                1
            } else {
                let count = ((far / near).ln() / far_to_near_ratio.ln()).ceil();
                if count.is_nan() || count < 1.0 {
                    1
                } else {
                    count as usize
                }
            }
        };

        let num_frustums = num_frustums.max(1);
        self.frustum_commands_list.clear();

        for m in 0..num_frustums {
            let (cur_near, cur_far) = if is_2d {
                let cur_near = (far - self.near_to_far_distance_2d)
                    .min(near + (m as f64) * self.near_to_far_distance_2d);
                let cur_far = far.min(cur_near + self.near_to_far_distance_2d);
                (cur_near, cur_far)
            } else {
                let cur_near = near.max(far_to_near_ratio.powi(m as i32) * near);
                let cur_far = far.min(far_to_near_ratio * cur_near);
                (cur_near, cur_far)
            };
            self.frustum_commands_list
                .push(FrustumCommands::new(Some(cur_near), Some(cur_far)));
        }

        // Update frustum splits
        self.frustum_splits.clear();
        for f in &self.frustum_commands_list {
            self.frustum_splits.push(f.near);
        }
        if let Some(last) = self.frustum_commands_list.last() {
            self.frustum_splits.push(last.far);
        }
    }

    /// Inserts a command into appropriate frustum bins based on its depth range, matching CesiumJS `View.js#insertIntoBin`.
    pub fn insert_into_bin(&mut self, mut command_extent: CommandExtent) {
        if self.debug_show_frustums {
            command_extent.command.debug_overlapping_frustums = 0;
        }

        let num_frustums = self.frustum_commands_list.len();
        for i in 0..num_frustums {
            let frustum_far = self.frustum_commands_list[i].far;
            if command_extent.near > frustum_far {
                continue;
            }

            let frustum_near = self.frustum_commands_list[i].near;
            if command_extent.far < frustum_near {
                break;
            }

            let pass = command_extent.command.pass.map_or(0, |p| p as usize);
            if pass < NUMBER_OF_PASSES {
                let frustum = &mut self.frustum_commands_list[i];
                let index = frustum.indices[pass];
                frustum.indices[pass] += 1;
                if index < frustum.commands[pass].len() {
                    frustum.commands[pass][index] = command_extent.command.clone();
                } else {
                    frustum.commands[pass].push(command_extent.command.clone());
                }
            }

            if self.debug_show_frustums {
                command_extent.command.debug_overlapping_frustums |= 1 << i;
            }

            if command_extent.command.execute_in_closest_frustum {
                break;
            }
        }

        if self.debug_show_frustums {
            let mask = command_extent.command.debug_overlapping_frustums;
            *self
                .debug_frustum_statistics
                .commands_in_frustums
                .entry(mask)
                .or_insert(0) += 1;
            self.debug_frustum_statistics.total_commands += 1;
        }
    }

    /// Creates potentially visible set by updating frustums and binning commands, matching `View.prototype.createPotentiallyVisibleSet`.
    pub fn create_potentially_visible_set(
        &mut self,
        camera_pos: DVec3,
        camera_dir: DVec3,
        camera_near: f64,
        camera_far: f64,
        camera_position_z: f64,
        commands: &[DrawCommand],
    ) {
        if self.debug_show_frustums {
            self.debug_frustum_statistics = DebugFrustumStatistics::default();
        }

        let mut scene_near = f64::MAX;
        let mut scene_far = f64::MIN;
        let mut extents = Vec::with_capacity(commands.len());

        for command in commands {
            let (cmd_near, cmd_far) = if let Some(bs) = command.bounding_volume {
                let (dn, df) = bs.compute_plane_distances(camera_pos, camera_dir);
                (dn, df)
            } else if let Some(obb) = command.oriented_bounding_box {
                let interval = obb.compute_plane_distances(camera_pos, camera_dir);
                (interval.start, interval.stop)
            } else {
                // If command has no bounding volume we use camera's near and far planes
                (camera_near, camera_far)
            };

            scene_near = scene_near.min(cmd_near);
            scene_far = scene_far.max(cmd_far);

            extents.push(CommandExtent {
                command: command.clone(),
                near: cmd_near,
                far: cmd_far,
            });
        }

        if extents.is_empty() || scene_near > scene_far {
            scene_near = camera_near;
            scene_far = camera_far;
        }

        self.update_frustums(
            camera_near,
            camera_far,
            camera_position_z,
            scene_near,
            scene_far,
        );

        for extent in extents {
            self.insert_into_bin(extent);
        }
    }

    /// Transforms a fragment shader source to include debugShowFrustums debug instrumentation,
    /// matching CesiumJS `DebugInspector.js#createDebugShowFrustumsShaderProgram`.
    pub fn create_debug_show_frustums_fragment_shader(fragment_shader_source: &str) -> String {
        let replaced = ShaderSource::replace_main(fragment_shader_source, "czm_Debug_main");
        format!(
            "{replaced}\n\
            uniform vec3 debugShowCommandsColor;\n\
            uniform vec3 debugShowFrustumsColor;\n\
            void main() {{\n\
                czm_Debug_main();\n\
                out_FragColor.rgb *= debugShowCommandsColor;\n\
                out_FragColor.rgb *= debugShowFrustumsColor;\n\
            }}\n"
        )
    }

    /// Computes the debug show frustums color based on the overlapping frustums bitmask.
    /// Bit 0: red, Bit 1: green, Bit 2: blue.
    pub fn compute_debug_frustums_color(overlapping_frustums_mask: i32) -> Color {
        let r = if overlapping_frustums_mask & (1 << 0) != 0 {
            1.0
        } else {
            0.0
        };
        let g = if overlapping_frustums_mask & (1 << 1) != 0 {
            1.0
        } else {
            0.0
        };
        let b = if overlapping_frustums_mask & (1 << 2) != 0 {
            1.0
        } else {
            0.0
        };
        Color::new(r, g, b, 1.0)
    }

    /// Evaluates the rendered RGBA output from the multi-frustum commands back-to-front (from farthest frustum to closest).
    /// Used for spec assertions matching CesiumJS test rendering behavior.
    pub fn simulate_rendered_rgba(&self, command_colors: &HashMap<String, Color>) -> [u8; 4] {
        // Framebuffer clear color is default Color::BLACK (0, 0, 0, 1.0) matching CesiumJS createScene
        let mut accum_r = 0.0f32;
        let mut accum_g = 0.0f32;
        let mut accum_b = 0.0f32;
        let mut accum_a = 1.0f32;

        // Render from farthest frustum to closest frustum (CesiumJS multi-frustum rendering order)
        for frustum in self.frustum_commands_list.iter().rev() {
            for pass_cmds in &frustum.commands {
                for cmd in pass_cmds {
                    let owner_id = cmd.owner.as_deref().unwrap_or("");
                    if let Some(color) = command_colors.get(owner_id) {
                        let alpha = color.alpha;
                        if alpha > 0.0 {
                            // Standard 'over' alpha blending
                            accum_r = color.red * alpha + accum_r * (1.0 - alpha);
                            accum_g = color.green * alpha + accum_g * (1.0 - alpha);
                            accum_b = color.blue * alpha + accum_b * (1.0 - alpha);
                            accum_a = alpha + accum_a * (1.0 - alpha);
                        }
                    }
                }
            }
        }

        [
            (accum_r * 255.0).round().clamp(0.0, 255.0) as u8,
            (accum_g * 255.0).round().clamp(0.0, 255.0) as u8,
            (accum_b * 255.0).round().clamp(0.0, 255.0) as u8,
            (accum_a * 255.0).round().clamp(0.0, 255.0) as u8,
        ]
    }
}
