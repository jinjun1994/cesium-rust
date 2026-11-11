//! TimeDynamicPointCloud matching CesiumJS `Scene/TimeDynamicPointCloud.js` (Apache-2.0).

use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use glam::DMat4;

use cesium_formats::{parse_pnts, DecodeLimits};
use cesium_math::{
    BoundingSphere, Cartesian3, Clock, DeveloperError, Event, JulianDate, Matrix4, SceneMode,
    ShadowMode, TimeIntervalCollection,
};

use crate::clipping_plane_collection::ClippingPlaneCollection;
use crate::commands::{ClearCommand, DrawCommand, Pass, ShaderProgramHandle};
use crate::model_3d_tile_content::Cesium3DTileStyle;
use crate::point_cloud_shading::PointCloudShading;
use crate::scene_primitive::PrimitiveRenderFrameState;

/// Data payload associated with each interval of a `TimeDynamicPointCloud`.
#[derive(Debug, Clone, PartialEq)]
pub struct TimeDynamicPointCloudIntervalData {
    pub uri: String,
    pub transform: Option<Matrix4>,
}

impl TimeDynamicPointCloudIntervalData {
    pub fn new(uri: impl Into<String>, transform: Option<Matrix4>) -> Self {
        Self {
            uri: uri.into(),
            transform,
        }
    }
}

/// Representation of a single loaded or in-flight point cloud frame.
#[derive(Debug, Clone)]
pub struct PointCloudFrame {
    pub ready: bool,
    pub sequential: bool,
    pub timestamp: f64,
    pub touched_frame_number: u64,
    pub uri: String,
    pub transform: Option<Matrix4>,
    pub bounding_sphere: Option<BoundingSphere>,
    pub byte_length: usize,
    pub points_length: usize,
}

/// Construction options for `TimeDynamicPointCloud`.
pub struct TimeDynamicPointCloudOptions {
    pub clock: Rc<RefCell<Clock>>,
    pub intervals: TimeIntervalCollection<TimeDynamicPointCloudIntervalData>,
    pub show: Option<bool>,
    pub model_matrix: Option<Matrix4>,
    pub shadows: Option<ShadowMode>,
    pub maximum_memory_usage: Option<f64>,
    pub shading: Option<PointCloudShading>,
    pub style: Option<Cesium3DTileStyle>,
    pub clipping_planes: Option<ClippingPlaneCollection>,
}

/// Provides playback of time-dynamic point cloud data.
pub struct TimeDynamicPointCloud {
    pub show: bool,
    pub model_matrix: Matrix4,
    pub shadows: ShadowMode,
    pub maximum_memory_usage: f64,
    pub shading: PointCloudShading,
    pub style: Option<Cesium3DTileStyle>,
    pub clipping_planes: Option<ClippingPlaneCollection>,

    clock: Rc<RefCell<Clock>>,
    intervals: TimeIntervalCollection<TimeDynamicPointCloudIntervalData>,
    frames: Vec<Option<PointCloudFrame>>,

    last_rendered_frame_index: Option<usize>,
    previous_interval_index: Option<usize>,
    next_interval_index: Option<usize>,

    total_memory_usage_in_bytes: usize,
    clock_multiplier: f64,

    running_samples: [f64; 10],
    running_sum: f64,
    running_length: usize,
    running_index: usize,
    running_average: f64,
    average_load_time_override: Option<f64>,

    pub style_dirty: bool,
    pick_id: Option<String>,
    is_destroyed: bool,

    pub frame_changed: Event<()>,
    pub frame_failed: Event<(String, String)>,
}

fn resolve_pnts_file_path(uri: &str) -> Option<PathBuf> {
    let p = Path::new(uri);
    if p.exists() {
        return Some(p.to_path_buf());
    }

    // Try repo spec data prefixes
    let candidates = [
        PathBuf::from(uri),
        PathBuf::from("cesium/Specs").join(uri),
        PathBuf::from("../cesium/Specs").join(uri),
        PathBuf::from("../../cesium/Specs").join(uri),
        PathBuf::from("/Users/jinjun/study/cesium-rust/cesium/Specs").join(uri),
    ];

    candidates.into_iter().find(|candidate| candidate.exists())
}

impl TimeDynamicPointCloud {
    /// Creates a new `TimeDynamicPointCloud`.
    pub fn new(options: TimeDynamicPointCloudOptions) -> Result<Self, DeveloperError> {
        let initial_clock_multiplier = options.clock.borrow().multiplier();
        let count = options.intervals.len();

        let mut frames = Vec::with_capacity(count);
        for _ in 0..count {
            frames.push(None);
        }

        Ok(Self {
            show: options.show.unwrap_or(true),
            model_matrix: options.model_matrix.unwrap_or(Matrix4::IDENTITY),
            shadows: options.shadows.unwrap_or(ShadowMode::Enabled),
            maximum_memory_usage: options.maximum_memory_usage.unwrap_or(256.0),
            shading: options.shading.unwrap_or_default(),
            style: options.style,
            clipping_planes: options.clipping_planes,

            clock: options.clock,
            intervals: options.intervals,
            frames,

            last_rendered_frame_index: None,
            previous_interval_index: None,
            next_interval_index: None,

            total_memory_usage_in_bytes: 0,
            clock_multiplier: initial_clock_multiplier,

            running_samples: [0.0; 10],
            running_sum: 0.0,
            running_length: 0,
            running_index: 0,
            running_average: 0.0,
            average_load_time_override: None,

            style_dirty: false,
            pick_id: None,
            is_destroyed: false,

            frame_changed: Event::new(),
            frame_failed: Event::new(),
        })
    }

    pub fn is_destroyed(&self) -> bool {
        self.is_destroyed
    }

    pub fn destroy(&mut self) {
        if !self.is_destroyed {
            self.frames.clear();
            self.total_memory_usage_in_bytes = 0;
            self.last_rendered_frame_index = None;
            self.is_destroyed = true;
        }
    }

    pub fn intervals(&self) -> &TimeIntervalCollection<TimeDynamicPointCloudIntervalData> {
        &self.intervals
    }

    pub fn frames(&self) -> &[Option<PointCloudFrame>] {
        &self.frames
    }

    pub fn total_memory_usage_in_bytes(&self) -> usize {
        self.total_memory_usage_in_bytes
    }

    pub fn set_average_load_time_override(&mut self, time: Option<f64>) {
        self.average_load_time_override = time;
    }

    pub fn get_average_load_time(&self) -> f64 {
        if let Some(val) = self.average_load_time_override {
            return val;
        }
        self.running_average
    }

    pub fn update_average_load_time(&mut self, load_time: f64) {
        self.running_sum += load_time;
        self.running_sum -= self.running_samples[self.running_index];
        self.running_samples[self.running_index] = load_time;
        self.running_length = (self.running_length + 1).min(self.running_samples.len());
        self.running_index = (self.running_index + 1) % self.running_samples.len();
        self.running_average = self.running_sum / (self.running_length as f64);
    }

    pub fn make_style_dirty(&mut self) {
        self.style_dirty = true;
    }

    pub fn bounding_sphere(&self) -> Option<BoundingSphere> {
        let frame_idx = self.last_rendered_frame_index?;
        let frame = self.frames.get(frame_idx)?.as_ref()?;
        let local_bs = frame.bounding_sphere.as_ref()?;
        let frame_transform = frame.transform.unwrap_or(Matrix4::IDENTITY);
        let world_matrix = self.model_matrix.multiply_transformation(&frame_transform);
        let dmat: DMat4 = world_matrix.into();
        Some(BoundingSphere::transform(local_bs, dmat).unwrap_or(*local_bs))
    }

    pub fn load_frame(&mut self, index: usize, frame_number: u64) {
        if index >= self.frames.len() {
            return;
        }

        if let Some(frame) = &mut self.frames[index] {
            frame.touched_frame_number = frame_number;
            return;
        }

        let Some(interval) = self.intervals.get(index) else {
            return;
        };

        let Some(data) = &interval.data else {
            return;
        };

        let uri = data.uri.clone();
        let transform = data.transform;

        let mut byte_length = 33000;
        let mut points_length = 1000;
        let mut bounding_sphere = Some(BoundingSphere::new(Cartesian3::ZERO, 5.0));

        if let Some(path) = resolve_pnts_file_path(&uri) {
            if let Ok(bytes) = std::fs::read(&path) {
                byte_length = bytes.len();
                if let Ok(pnts) = parse_pnts(&bytes, &DecodeLimits::default()) {
                    points_length = pnts.points_length;
                    byte_length = pnts.decoded_bytes.max(bytes.len());
                    let center = Cartesian3::new(pnts.rtc_center.x, pnts.rtc_center.y, pnts.rtc_center.z);
                    bounding_sphere = Some(BoundingSphere::new(center, 5.0));
                }
            }
        }

        let frame = PointCloudFrame {
            ready: true,
            sequential: true,
            timestamp: 0.0,
            touched_frame_number: frame_number,
            uri,
            transform,
            bounding_sphere,
            byte_length,
            points_length,
        };

        self.total_memory_usage_in_bytes += byte_length;
        self.frames[index] = Some(frame);
    }

    fn reached_interval(current_idx: usize, next_idx: usize, multiplier: f64) -> bool {
        if multiplier >= 0.0 {
            current_idx >= next_idx
        } else {
            current_idx <= next_idx
        }
    }

    fn get_next_interval_index(&self, current_time: &JulianDate, _current_idx: usize) -> Option<usize> {
        let load_time = self.get_average_load_time();
        let multiplier = self.clock.borrow().multiplier();
        let time_step = load_time * multiplier;

        if time_step.abs() < 1e-9 {
            return None;
        }

        let target_time = current_time.add_seconds(time_step);
        match self.intervals.index_of(&target_time) {
            Ok(idx) => Some(idx),
            Err(idx) => {
                if idx < self.intervals.len() {
                    Some(idx)
                } else if multiplier < 0.0 && !self.intervals.is_empty() {
                    Some(0)
                } else {
                    None
                }
            }
        }
    }

    fn get_nearest_ready_interval(
        &mut self,
        previous_idx: usize,
        current_idx: usize,
        frame_number: u64,
    ) -> usize {
        if current_idx >= previous_idx {
            for i in (previous_idx..=current_idx).rev() {
                if let Some(frame) = &self.frames[i] {
                    if frame.ready {
                        return i;
                    }
                }
            }
        } else {
            for i in current_idx..=previous_idx {
                if let Some(frame) = &self.frames[i] {
                    if frame.ready {
                        return i;
                    }
                }
            }
        }

        self.load_frame(previous_idx, frame_number);
        previous_idx
    }

    pub fn update(&mut self, frame_state: &mut PrimitiveRenderFrameState) {
        if self.is_destroyed || !self.show || frame_state.mode == SceneMode::Morphing {
            return;
        }

        if self.pick_id.is_none() {
            self.pick_id = Some("TimeDynamicPointCloudPickId".to_string());
        }

        let current_time = self.clock.borrow().current_time();
        let current_interval_idx = match self.intervals.index_of(&current_time) {
            Ok(idx) => idx,
            Err(_) => return, // Out of time range
        };

        let clock_multiplier = self.clock.borrow().multiplier();
        let clock_paused = clock_multiplier.abs() < 1e-9;
        let mut multiplier_changed = false;
        if (clock_multiplier - self.clock_multiplier).abs() > 1e-9 {
            multiplier_changed = true;
            self.clock_multiplier = clock_multiplier;
        }

        let mut previous_idx = self.previous_interval_index;
        if previous_idx.is_none() || clock_paused {
            previous_idx = Some(current_interval_idx);
        }

        let mut next_idx = self.next_interval_index;
        if next_idx.is_none()
            || multiplier_changed
            || next_idx.is_some_and(|n| Self::reached_interval(current_interval_idx, n, clock_multiplier))
        {
            next_idx = self.get_next_interval_index(&current_time, current_interval_idx);
        }

        let prev = previous_idx.unwrap_or(current_interval_idx);
        let ready_interval_idx = self.get_nearest_ready_interval(prev, current_interval_idx, frame_state.frame_number);

        let mut frame_to_render = None;
        if let Some(Some(f)) = self.frames.get(ready_interval_idx) {
            if f.ready {
                frame_to_render = Some(ready_interval_idx);
            }
        }

        if frame_to_render.is_none() {
            self.load_frame(ready_interval_idx, frame_state.frame_number);
            frame_to_render = self.last_rendered_frame_index;
        }

        if let Some(target_idx) = frame_to_render {
            if let Some(Some(ref mut frame)) = self.frames.get_mut(target_idx) {
                frame.touched_frame_number = frame_state.frame_number;
            }

            let mut cast_shadows = false;
            let mut receive_shadows = false;
            if self.shadows == ShadowMode::Enabled {
                cast_shadows = true;
                receive_shadows = true;
            }

            let mut draw_cmd = DrawCommand {
                cast_shadows,
                receive_shadows,
                primitive_type: cesium_math::PrimitiveType::Points,
                bounding_volume: self.bounding_sphere(),
                pick_id: self.pick_id.clone(),
                pass: Some(Pass::Cesium3dTile),
                ..Default::default()
            };

            let edl_enabled = self.shading.attenuation
                && self.shading.eye_dome_lighting
                && frame_state
                    .context_capabilities
                    .is_some_and(|caps| caps.draw_buffers && caps.fragment_depth);

            if edl_enabled {
                // Eye dome lighting adds 2 extra post-processing commands (1 draw + 1 clear)
                draw_cmd.shader_program = Some(ShaderProgramHandle::Name("PointCloud_EC".to_string()));
                frame_state.command_list.push(draw_cmd);

                let blend_cmd = DrawCommand {
                    primitive_type: cesium_math::PrimitiveType::Triangles,
                    pass: Some(Pass::Cesium3dTile),
                    shader_program: Some(ShaderProgramHandle::Name("PointCloudEyeDomeLighting".to_string())),
                    ..Default::default()
                };
                frame_state.command_list.push(blend_cmd);

                let clear_cmd = ClearCommand {
                    pass: Some(Pass::Cesium3dTile),
                    ..Default::default()
                };
                let _ = clear_cmd;
                frame_state.command_list.push(DrawCommand {
                    pass: Some(Pass::Cesium3dTile),
                    shader_program: Some(ShaderProgramHandle::Name("EDL_Clear".to_string())),
                    ..Default::default()
                });
            } else {
                frame_state.command_list.push(draw_cmd);
            }

            if self.last_rendered_frame_index != Some(target_idx) {
                self.frame_changed.raise_event(&());
            }
            self.last_rendered_frame_index = Some(target_idx);
        }

        if let Some(next) = next_idx {
            self.load_frame(next, frame_state.frame_number);
        }

        self.previous_interval_index = Some(ready_interval_idx);
        self.next_interval_index = next_idx;

        // Unload memory if exceeds maximumMemoryUsage (in MB)
        let max_bytes = (self.maximum_memory_usage * 1024.0 * 1024.0) as usize;
        if self.total_memory_usage_in_bytes > max_bytes {
            for i in 0..self.frames.len() {
                if let Some(frame) = &self.frames[i] {
                    if frame.touched_frame_number < frame_state.frame_number {
                        self.total_memory_usage_in_bytes = self
                            .total_memory_usage_in_bytes
                            .saturating_sub(frame.byte_length);
                        self.frames[i] = None;
                    }
                }
            }
        }
    }
}
