//! Render performance metrics, CPU phase timers, and optional GPU timestamp queries.

use std::collections::VecDeque;
#[cfg(not(target_arch = "wasm32"))]
use std::time::{Duration, Instant};
#[cfg(target_arch = "wasm32")]
use web_time::{Duration, Instant};

/// Detailed breakdown of CPU and GPU frame timings.
#[derive(Clone, Debug, Default)]
pub struct FrameStats {
    /// Time spent sorting meshes, evaluating camera distances, and determining draw order.
    pub prepare_duration: Duration,
    /// Time spent packing and writing uniforms to the GPU (batch page uploads).
    pub uniform_upload_duration: Duration,
    /// Time spent recording render passes into command encoders.
    pub pass_encode_duration: Duration,
    /// Time spent submitting command buffers to the GPU queue.
    pub submit_duration: Duration,
    /// Total CPU time for the frame (prepare + uniform + pass + submit).
    pub total_cpu_duration: Duration,
    /// Measured GPU execution time in milliseconds (if timestamp query supported and available).
    pub gpu_time_ms: Option<f64>,
    /// Number of draw calls executed in the frame.
    pub draw_calls: u32,
    /// Number of triangles drawn.
    pub triangles_drawn: u64,
    /// Number of lines drawn.
    pub lines_drawn: u64,
    /// Total number of objects rendered.
    pub object_count: u32,
    /// Number of uniform pages active.
    pub uniform_pages: usize,
    /// Estimated resident mesh buffer memory in bytes.
    pub resident_mesh_bytes: usize,
    /// Estimated resident texture memory in bytes.
    pub resident_texture_bytes: usize,
    /// Estimated uniform buffer memory in bytes.
    pub uniform_buffer_bytes: usize,
}

impl FrameStats {
    pub fn prepare_ms(&self) -> f64 {
        self.prepare_duration.as_secs_f64() * 1000.0
    }
    pub fn uniform_upload_ms(&self) -> f64 {
        self.uniform_upload_duration.as_secs_f64() * 1000.0
    }
    pub fn pass_encode_ms(&self) -> f64 {
        self.pass_encode_duration.as_secs_f64() * 1000.0
    }
    pub fn submit_ms(&self) -> f64 {
        self.submit_duration.as_secs_f64() * 1000.0
    }
    pub fn total_cpu_ms(&self) -> f64 {
        self.total_cpu_duration.as_secs_f64() * 1000.0
    }
}

/// Aggregated statistical summary (p50, p95, p99) over recent frames.
#[derive(Clone, Debug)]
pub struct FrameStatsSummary {
    pub sample_count: usize,
    pub cpu_p50_ms: f64,
    pub cpu_p95_ms: f64,
    pub cpu_p99_ms: f64,
    pub avg_draw_calls: f64,
    pub avg_triangles: f64,
    pub avg_objects: f64,
    pub gpu_timestamp_available: bool,
    pub last_frame: FrameStats,
}

/// Rolling window of frame statistics for diagnostics and latency percentiles.
pub struct FrameStatsHistory {
    max_samples: usize,
    history: VecDeque<FrameStats>,
    gpu_timestamp_supported: bool,
}

impl FrameStatsHistory {
    pub fn new(max_samples: usize, gpu_timestamp_supported: bool) -> Self {
        Self {
            max_samples: max_samples.max(10),
            history: VecDeque::with_capacity(max_samples),
            gpu_timestamp_supported,
        }
    }

    pub fn push(&mut self, stats: FrameStats) {
        if self.history.len() >= self.max_samples {
            self.history.pop_front();
        }
        self.history.push_back(stats);
    }

    pub fn latest(&self) -> Option<&FrameStats> {
        self.history.back()
    }

    pub fn summary(&self) -> Option<FrameStatsSummary> {
        if self.history.is_empty() {
            return None;
        }
        let sample_count = self.history.len();
        let mut cpu_times: Vec<f64> = self.history.iter().map(|s| s.total_cpu_ms()).collect();
        cpu_times.sort_by(|a, b| a.total_cmp(b));

        let p50_idx = ((sample_count as f64 * 0.50).round() as usize).min(sample_count - 1);
        let p95_idx = ((sample_count as f64 * 0.95).round() as usize).min(sample_count - 1);
        let p99_idx = ((sample_count as f64 * 0.99).round() as usize).min(sample_count - 1);

        let total_draw_calls: u32 = self.history.iter().map(|s| s.draw_calls).sum();
        let total_triangles: u64 = self.history.iter().map(|s| s.triangles_drawn).sum();
        let total_objects: u32 = self.history.iter().map(|s| s.object_count).sum();

        Some(FrameStatsSummary {
            sample_count,
            cpu_p50_ms: cpu_times[p50_idx],
            cpu_p95_ms: cpu_times[p95_idx],
            cpu_p99_ms: cpu_times[p99_idx],
            avg_draw_calls: total_draw_calls as f64 / sample_count as f64,
            avg_triangles: total_triangles as f64 / sample_count as f64,
            avg_objects: total_objects as f64 / sample_count as f64,
            gpu_timestamp_available: self.gpu_timestamp_supported,
            last_frame: self.history.back().cloned().unwrap_or_default(),
        })
    }
}

/// Helper timer for measuring sub-frame stages.
pub struct StageTimer {
    start: Instant,
}

impl StageTimer {
    pub fn start() -> Self {
        Self {
            start: Instant::now(),
        }
    }

    pub fn elapsed(&self) -> Duration {
        self.start.elapsed()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stats_summary_percentiles() {
        let mut history = FrameStatsHistory::new(100, false);
        for i in 1..=100 {
            let stats = FrameStats {
                total_cpu_duration: Duration::from_micros(i * 100),
                draw_calls: 10,
                triangles_drawn: 1000,
                object_count: 5,
                ..Default::default()
            };
            history.push(stats);
        }
        let summary = history.summary().unwrap();
        assert_eq!(summary.sample_count, 100);
        assert!((summary.cpu_p50_ms - 5.0).abs() < 0.2);
        assert!((summary.cpu_p95_ms - 9.5).abs() < 0.2);
        assert!((summary.cpu_p99_ms - 9.9).abs() < 0.2);
        assert_eq!(summary.avg_draw_calls, 10.0);
        assert_eq!(summary.avg_triangles, 1000.0);
        assert_eq!(summary.avg_objects, 5.0);
        assert!(!summary.gpu_timestamp_available);
    }
}
