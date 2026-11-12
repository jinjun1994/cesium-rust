//! Gaussian Splat Primitive matching CesiumJS `Scene/GaussianSplatPrimitive.js` (Apache-2.0).

use cesium_math::{Cartesian3, PrimitiveType, SceneMode, SplitDirection};
use glam::DMat4;

use crate::commands::{DrawCommand, DrawCommandOptions, Pass};
use crate::gaussian_splat_3d_tile_content::GaussianSplat3DTileContent;

/// Default number of consecutive frames with unchanged tile selection before building a snapshot.
pub const DEFAULT_STABLE_FRAMES: u32 = 2;
/// Maximum frames a pending snapshot can stall before being abandoned.
pub const DEFAULT_MAX_SNAPSHOT_STALL_FRAMES: u32 = 30;
/// Minimum frame interval between steady-state re-sorts.
pub const DEFAULT_SORT_MIN_FRAME_INTERVAL: u32 = 3;
/// Minimum camera direction angle delta (radians, ~0.5 deg) to trigger steady re-sort.
pub const DEFAULT_SORT_MIN_ANGLE_RADIANS: f64 = 0.008_726_646_259_971_648;
/// Minimum camera position delta (meters) to trigger steady re-sort.
pub const DEFAULT_SORT_MIN_POSITION_DELTA: f64 = 1.0;

/// Sorting state of a `GaussianSplatPrimitive`, matching `GaussianSplatSortingState` in CesiumJS.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
#[repr(u8)]
pub enum GaussianSplatSortingState {
    #[default]
    Idle = 0,
    Waiting = 1,
    Sorting = 2,
    Sorted = 3,
    Error = 4,
}

/// Lifecycle state of an aggregated `GaussianSplatSnapshot`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
#[repr(u8)]
pub enum SnapshotState {
    #[default]
    Building = 0,
    TexturePending = 1,
    TextureReady = 2,
    Sorting = 3,
    Ready = 4,
}

/// Aggregated splat data snapshot built from selected `GaussianSplat3DTileContent` tiles.
#[derive(Clone, Debug, PartialEq)]
pub struct GaussianSplatSnapshot {
    pub state: SnapshotState,
    pub num_splats: usize,
    pub positions: Vec<f32>,
    pub rotations: Vec<f32>,
    pub scales: Vec<f32>,
    pub colors: Vec<f32>,
    pub indexes: Vec<u32>,
    pub sh_degree: usize,
    pub model_matrix: DMat4,
    pub stall_frames: u32,
    pub sort_pending: bool,
}

impl GaussianSplatSnapshot {
    /// Builds a snapshot by concatenating selected `GaussianSplat3DTileContent` slices.
    pub fn from_contents(contents: &[&GaussianSplat3DTileContent], model_matrix: DMat4) -> Self {
        let num_splats: usize = contents.iter().map(|c| c.points_length()).sum();
        let mut positions = Vec::with_capacity(num_splats * 3);
        let mut rotations = Vec::with_capacity(num_splats * 4);
        let mut scales = Vec::with_capacity(num_splats * 3);
        let mut colors = Vec::with_capacity(num_splats * 4);
        let mut sh_degree = 0_usize;

        for content in contents {
            positions.extend_from_slice(content.positions());
            rotations.extend_from_slice(content.rotations());
            scales.extend_from_slice(content.scales());
            colors.extend_from_slice(content.colors());
            sh_degree = sh_degree.max(content.sh_degree());
        }

        let indexes: Vec<u32> = (0..num_splats as u32).collect();
        Self {
            state: SnapshotState::TextureReady,
            num_splats,
            positions,
            rotations,
            scales,
            colors,
            indexes,
            sh_degree,
            model_matrix,
            stall_frames: 0,
            sort_pending: false,
        }
    }

    /// Sorts splat indices back-to-front along the camera view direction.
    pub fn sort_back_to_front(&mut self, camera_position: Cartesian3, camera_direction: Cartesian3) {
        if self.num_splats <= 1 {
            self.state = SnapshotState::Ready;
            self.sort_pending = false;
            return;
        }

        let dir_x = camera_direction.x;
        let dir_y = camera_direction.y;
        let dir_z = camera_direction.z;
        let cam_x = camera_position.x;
        let cam_y = camera_position.y;
        let cam_z = camera_position.z;

        let mut depths = Vec::with_capacity(self.num_splats);
        for (idx, chunk) in self.positions.as_chunks::<3>().0.iter().enumerate() {
            let dx = chunk[0] as f64 - cam_x;
            let dy = chunk[1] as f64 - cam_y;
            let dz = chunk[2] as f64 - cam_z;
            let depth = dx * dir_x + dy * dir_y + dz * dir_z;
            depths.push((idx as u32, depth));
        }

        // Back-to-front: largest depth first.
        depths.sort_by(|a, b| b.1.total_cmp(&a.1));
        self.indexes.clear();
        self.indexes.extend(depths.into_iter().map(|(idx, _)| idx));
        self.state = SnapshotState::Ready;
        self.sort_pending = false;
    }
}

/// Per-frame camera and pass context passed to `GaussianSplatPrimitive::update`.
#[derive(Clone, Debug, PartialEq)]
pub struct GaussianSplatFrameContext {
    pub frame_number: u64,
    pub camera_position: Cartesian3,
    pub camera_direction: Cartesian3,
    pub passes_render: bool,
    pub scene_mode: SceneMode,
}

impl Default for GaussianSplatFrameContext {
    fn default() -> Self {
        Self {
            frame_number: 0,
            camera_position: Cartesian3::ZERO,
            camera_direction: Cartesian3::new(0.0, 0.0, -1.0),
            passes_render: true,
            scene_mode: SceneMode::Scene3D,
        }
    }
}

/// Options for constructing a `GaussianSplatPrimitive`.
#[derive(Clone, Debug, PartialEq)]
pub struct GaussianSplatPrimitiveOptions {
    pub show: bool,
    pub debug_show_bounding_volume: bool,
    pub split_direction: SplitDirection,
    pub model_matrix: DMat4,
    pub stable_frames_threshold: u32,
    pub max_snapshot_stall_frames: u32,
    pub sort_min_frame_interval: u32,
    pub sort_min_angle_radians: f64,
    pub sort_min_position_delta: f64,
    pub sorter_available: bool,
}

impl Default for GaussianSplatPrimitiveOptions {
    fn default() -> Self {
        Self {
            show: true,
            debug_show_bounding_volume: false,
            split_direction: SplitDirection::None,
            model_matrix: DMat4::IDENTITY,
            stable_frames_threshold: DEFAULT_STABLE_FRAMES,
            max_snapshot_stall_frames: DEFAULT_MAX_SNAPSHOT_STALL_FRAMES,
            sort_min_frame_interval: DEFAULT_SORT_MIN_FRAME_INTERVAL,
            sort_min_angle_radians: DEFAULT_SORT_MIN_ANGLE_RADIANS,
            sort_min_position_delta: DEFAULT_SORT_MIN_POSITION_DELTA,
            sorter_available: true,
        }
    }
}

/// Primitive that aggregates selected Gaussian Splat tiles into sorted GPU-ready snapshots.
#[derive(Clone, Debug)]
pub struct GaussianSplatPrimitive {
    pub show: bool,
    pub debug_show_bounding_volume: bool,
    pub split_direction: SplitDirection,
    pub model_matrix: DMat4,
    pub stable_frames_threshold: u32,
    pub max_snapshot_stall_frames: u32,
    pub sort_min_frame_interval: u32,
    pub sort_min_angle_radians: f64,
    pub sort_min_position_delta: f64,
    pub sorter_available: bool,
    is_stable: bool,
    stable_frames: u32,
    selected_tile_count: usize,
    last_selected_tile_count: usize,
    has_tileset_dirty: bool,
    gaussian_splat_sorting_state: GaussianSplatSortingState,
    active_snapshot: Option<GaussianSplatSnapshot>,
    pending_snapshot: Option<GaussianSplatSnapshot>,
    staged_contents: Vec<GaussianSplat3DTileContent>,
    last_sort_frame: Option<u64>,
    last_sort_camera_position: Option<Cartesian3>,
    last_sort_camera_direction: Option<Cartesian3>,
    draw_commands: Vec<DrawCommand>,
    destroyed: bool,
}

impl Default for GaussianSplatPrimitive {
    fn default() -> Self {
        Self::new(GaussianSplatPrimitiveOptions::default())
    }
}

impl GaussianSplatPrimitive {
    /// Creates a new `GaussianSplatPrimitive` with the given options.
    pub fn new(options: GaussianSplatPrimitiveOptions) -> Self {
        Self {
            show: options.show,
            debug_show_bounding_volume: options.debug_show_bounding_volume,
            split_direction: options.split_direction,
            model_matrix: options.model_matrix,
            stable_frames_threshold: options.stable_frames_threshold,
            max_snapshot_stall_frames: options.max_snapshot_stall_frames,
            sort_min_frame_interval: options.sort_min_frame_interval,
            sort_min_angle_radians: options.sort_min_angle_radians,
            sort_min_position_delta: options.sort_min_position_delta,
            sorter_available: options.sorter_available,
            is_stable: false,
            stable_frames: 0,
            selected_tile_count: 0,
            last_selected_tile_count: 0,
            has_tileset_dirty: false,
            gaussian_splat_sorting_state: GaussianSplatSortingState::Idle,
            active_snapshot: None,
            pending_snapshot: None,
            staged_contents: Vec::new(),
            last_sort_frame: None,
            last_sort_camera_position: None,
            last_sort_camera_direction: None,
            draw_commands: Vec::new(),
            destroyed: false,
        }
    }

    /// Returns whether the primitive has been destroyed.
    pub fn is_destroyed(&self) -> bool {
        self.destroyed
    }

    /// Returns whether tile selection has stabilized across consecutive frames.
    pub fn is_stable(&self) -> bool {
        self.is_stable
    }

    /// Returns the current sorting state.
    pub fn sorting_state(&self) -> GaussianSplatSortingState {
        self.gaussian_splat_sorting_state
    }

    /// Returns the active (rendered) snapshot if present.
    pub fn active_snapshot(&self) -> Option<&GaussianSplatSnapshot> {
        self.active_snapshot.as_ref()
    }

    /// Returns the pending snapshot being prepared/sorted if present.
    pub fn pending_snapshot(&self) -> Option<&GaussianSplatSnapshot> {
        self.pending_snapshot.as_ref()
    }

    /// Returns the draw commands emitted during the most recent `update`.
    pub fn draw_commands(&self) -> &[DrawCommand] {
        &self.draw_commands
    }

    /// Stages selected tile contents for snapshot aggregation and marks the tileset dirty.
    pub fn set_tiles_content(&mut self, contents: Vec<GaussianSplat3DTileContent>) {
        self.selected_tile_count = contents.len();
        self.staged_contents = contents;
        self.has_tileset_dirty = true;
        self.is_stable = false;
        self.stable_frames = 0;
    }

    /// Updates stability, snapshot lifecycle, sorting, and draw commands for a frame.
    pub fn update(&mut self, frame: &GaussianSplatFrameContext) -> &[DrawCommand] {
        self.draw_commands.clear();

        if self.destroyed || !self.show || !frame.passes_render {
            return &self.draw_commands;
        }

        if frame.scene_mode == SceneMode::Morphing {
            return &self.draw_commands;
        }

        if self.selected_tile_count == self.last_selected_tile_count && !self.has_tileset_dirty {
            self.stable_frames = self.stable_frames.saturating_add(1);
            if self.stable_frames >= self.stable_frames_threshold {
                self.is_stable = true;
            }
        } else {
            self.last_selected_tile_count = self.selected_tile_count;
            self.stable_frames = 0;
            self.is_stable = false;
            self.has_tileset_dirty = false;
        }

        if self.is_stable && self.pending_snapshot.is_none() && !self.staged_contents.is_empty() {
            let refs: Vec<&GaussianSplat3DTileContent> = self.staged_contents.iter().collect();
            self.pending_snapshot =
                Some(GaussianSplatSnapshot::from_contents(&refs, self.model_matrix));
            self.staged_contents.clear();
        }

        if let Some(mut pending) = self.pending_snapshot.take() {
            match pending.state {
                SnapshotState::Building | SnapshotState::TexturePending => {
                    pending.stall_frames = pending.stall_frames.saturating_add(1);
                    if pending.stall_frames <= self.max_snapshot_stall_frames {
                        self.pending_snapshot = Some(pending);
                    }
                }
                SnapshotState::TextureReady => {
                    if self.sorter_available {
                        pending.state = SnapshotState::Sorting;
                        self.gaussian_splat_sorting_state = GaussianSplatSortingState::Sorting;
                        pending.sort_back_to_front(frame.camera_position, frame.camera_direction);
                        self.gaussian_splat_sorting_state = GaussianSplatSortingState::Sorted;
                        self.last_sort_frame = Some(frame.frame_number);
                        self.last_sort_camera_position = Some(frame.camera_position);
                        self.last_sort_camera_direction = Some(frame.camera_direction);
                        self.active_snapshot = Some(pending);
                    } else {
                        // Sorter unavailable: keep pending in TextureReady so next frame can retry.
                        pending.stall_frames = pending.stall_frames.saturating_add(1);
                        self.gaussian_splat_sorting_state = GaussianSplatSortingState::Waiting;
                        if pending.stall_frames <= self.max_snapshot_stall_frames {
                            self.pending_snapshot = Some(pending);
                        }
                    }
                }
                SnapshotState::Sorting => {
                    if self.sorter_available {
                        pending.sort_back_to_front(frame.camera_position, frame.camera_direction);
                        self.gaussian_splat_sorting_state = GaussianSplatSortingState::Sorted;
                        self.last_sort_frame = Some(frame.frame_number);
                        self.last_sort_camera_position = Some(frame.camera_position);
                        self.last_sort_camera_direction = Some(frame.camera_direction);
                        self.active_snapshot = Some(pending);
                    } else {
                        self.pending_snapshot = Some(pending);
                    }
                }
                SnapshotState::Ready => {
                    self.active_snapshot = Some(pending);
                }
            }
        } else if self.needs_steady_sort(frame) && self.sorter_available {
            if let Some(active) = self.active_snapshot.as_mut() {
                self.gaussian_splat_sorting_state = GaussianSplatSortingState::Sorting;
                active.sort_back_to_front(frame.camera_position, frame.camera_direction);
                self.gaussian_splat_sorting_state = GaussianSplatSortingState::Sorted;
                self.last_sort_frame = Some(frame.frame_number);
                self.last_sort_camera_position = Some(frame.camera_position);
                self.last_sort_camera_direction = Some(frame.camera_direction);
            }
        }

        if let Some(active) = &self.active_snapshot {
            if active.state == SnapshotState::Ready && active.num_splats > 0 {
                self.draw_commands.push(DrawCommand::new(DrawCommandOptions {
                    model_matrix: Some(self.model_matrix),
                    primitive_type: Some(PrimitiveType::Triangles),
                    count: Some(6),
                    instance_count: Some(active.num_splats),
                    pass: Some(Pass::GaussianSplats),
                    owner: Some("GaussianSplatPrimitive".to_string()),
                    debug_show_bounding_volume: Some(self.debug_show_bounding_volume),
                    ..Default::default()
                }));
            }
        }

        &self.draw_commands
    }

    /// Returns whether steady-state camera movement warrants re-sorting the active snapshot.
    pub fn needs_steady_sort(&self, frame: &GaussianSplatFrameContext) -> bool {
        let Some(active) = &self.active_snapshot else {
            return false;
        };
        if active.state != SnapshotState::Ready || active.num_splats <= 1 {
            return false;
        }

        if let Some(last_frame) = self.last_sort_frame {
            if frame.frame_number.saturating_sub(last_frame) < u64::from(self.sort_min_frame_interval)
            {
                return false;
            }
        }

        if let Some(last_pos) = self.last_sort_camera_position {
            let delta = (frame.camera_position - last_pos).magnitude();
            if delta >= self.sort_min_position_delta {
                return true;
            }
        } else {
            return true;
        }

        if let Some(last_dir) = self.last_sort_camera_direction {
            let a = last_dir.normalize();
            let b = frame.camera_direction.normalize();
            let dot = a.dot(&b).clamp(-1.0, 1.0);
            let angle = dot.acos();
            if angle >= self.sort_min_angle_radians {
                return true;
            }
        }

        false
    }

    /// Destroys the primitive and releases snapshot state.
    pub fn destroy(&mut self) {
        self.active_snapshot = None;
        self.pending_snapshot = None;
        self.staged_contents.clear();
        self.draw_commands.clear();
        self.gaussian_splat_sorting_state = GaussianSplatSortingState::Idle;
        self.destroyed = true;
    }
}
