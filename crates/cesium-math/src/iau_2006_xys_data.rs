use std::path::PathBuf;
use std::sync::mpsc::{channel, Receiver, Sender};
use serde::{Deserialize, Serialize};

use crate::errors::RuntimeError;
use crate::julian_date::{JulianDate, TimeStandard};

/// An IAU 2006 XYS value sampled at a particular time.
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
pub struct Iau2006XysSample {
    pub x: f64,
    pub y: f64,
    pub s: f64,
}

impl Iau2006XysSample {
    pub const ZERO: Self = Self {
        x: 0.0,
        y: 0.0,
        s: 0.0,
    };

    pub const fn new(x: f64, y: f64, s: f64) -> Self {
        Self { x, y, s }
    }

    pub fn equals_epsilon(&self, other: &Self, epsilon: f64) -> bool {
        (self.x - other.x).abs() <= epsilon
            && (self.y - other.y).abs() <= epsilon
            && (self.s - other.s).abs() <= epsilon
    }
}

/// Options for configuring IAU 2006 XYS data.
#[derive(Debug, Clone, Default)]
pub struct Iau2006XysDataOptions {
    pub xys_file_url_template: Option<String>,
    pub interpolation_order: Option<usize>,
    pub sample_zero_julian_ephemeris_date: Option<f64>,
    pub step_size_days: Option<f64>,
    pub samples_per_xys_file: Option<usize>,
    pub total_samples: Option<usize>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct XysChunkJson {
    samples: Vec<f64>,
}

/// A set of IAU2006 XYS data that is used to evaluate the transformation between ICRF and ITRF.
pub struct Iau2006XysData {
    xys_file_url_template: Option<String>,
    interpolation_order: usize,
    sample_zero_julian_ephemeris_date: f64,
    sample_zero_date_tt: JulianDate,
    step_size_days: f64,
    samples_per_xys_file: usize,
    total_samples: usize,
    samples: Vec<f64>,
    chunk_loaded: Vec<bool>,
    chunk_in_progress: Vec<bool>,
    denominators: Vec<f64>,
    x_table: Vec<f64>,
    work: Vec<f64>,
    coef: Vec<f64>,
    #[allow(dead_code)]
    sender: Sender<(usize, Vec<f64>)>,
    receiver: Receiver<(usize, Vec<f64>)>,
}

impl Default for Iau2006XysData {
    fn default() -> Self {
        Self::new(None)
    }
}

impl Iau2006XysData {
    /// Creates a new IAU 2006 XYS data interpolator.
    pub fn new(options: Option<Iau2006XysDataOptions>) -> Self {
        let opts = options.unwrap_or_default();
        let interpolation_order = opts.interpolation_order.unwrap_or(9);
        let sample_zero_julian_ephemeris_date =
            opts.sample_zero_julian_ephemeris_date.unwrap_or(2442396.5);
        let sample_zero_date_tt = JulianDate::new(
            sample_zero_julian_ephemeris_date,
            0.0,
            TimeStandard::TAI,
        );
        let step_size_days = opts.step_size_days.unwrap_or(1.0);
        let samples_per_xys_file = opts.samples_per_xys_file.unwrap_or(1000);
        let total_samples = opts.total_samples.unwrap_or(27426);

        let total_chunks = total_samples.div_ceil(samples_per_xys_file);
        let mut denominators = vec![0.0; interpolation_order + 1];
        let mut x_table = vec![0.0; interpolation_order + 1];

        let step_n = step_size_days.powi(interpolation_order as i32);
        for i in 0..=interpolation_order {
            let mut denom = step_n;
            x_table[i] = i as f64 * step_size_days;
            for j in 0..=interpolation_order {
                if j != i {
                    denom *= (i as f64) - (j as f64);
                }
            }
            denominators[i] = 1.0 / denom;
        }

        let (sender, receiver) = channel();

        Self {
            xys_file_url_template: opts.xys_file_url_template,
            interpolation_order,
            sample_zero_julian_ephemeris_date,
            sample_zero_date_tt,
            step_size_days,
            samples_per_xys_file,
            total_samples,
            samples: vec![0.0; total_samples * 3],
            chunk_loaded: vec![false; total_chunks],
            chunk_in_progress: vec![false; total_chunks],
            denominators,
            x_table,
            work: vec![0.0; interpolation_order + 1],
            coef: vec![0.0; interpolation_order + 1],
            sender,
            receiver,
        }
    }

    pub fn interpolation_order(&self) -> usize {
        self.interpolation_order
    }

    pub fn step_size_days(&self) -> f64 {
        self.step_size_days
    }

    pub fn sample_zero_julian_ephemeris_date(&self) -> f64 {
        self.sample_zero_julian_ephemeris_date
    }

    pub fn total_samples(&self) -> usize {
        self.total_samples
    }

    pub fn samples_per_xys_file(&self) -> usize {
        self.samples_per_xys_file
    }

    pub fn xys_file_url_template(&self) -> Option<&str> {
        self.xys_file_url_template.as_deref()
    }

    fn get_days_since_epoch(&self, day_tt: i32, second_tt: f64) -> f64 {
        let date_tt = JulianDate::new_tai(day_tt, second_tt);
        date_tt.days_difference(&self.sample_zero_date_tt)
    }

    /// Preloads XYS data for a specified date range.
    pub fn preload(
        &mut self,
        start_day_tt: i32,
        start_second_tt: f64,
        stop_day_tt: i32,
        stop_second_tt: f64,
    ) {
        let start_days = self.get_days_since_epoch(start_day_tt, start_second_tt);
        let stop_days = self.get_days_since_epoch(stop_day_tt, stop_second_tt);

        let mut start_index = (start_days / self.step_size_days
            - (self.interpolation_order as f64) / 2.0)
            .floor() as i64;
        if start_index < 0 {
            start_index = 0;
        }

        let mut stop_index = (stop_days / self.step_size_days
            - (self.interpolation_order as f64) / 2.0)
            .floor() as i64
            + (self.interpolation_order as i64);
        if stop_index >= self.total_samples as i64 {
            stop_index = (self.total_samples as i64) - 1;
        }

        let start_chunk = (start_index as usize) / self.samples_per_xys_file;
        let stop_chunk = (stop_index as usize) / self.samples_per_xys_file;

        for chunk_idx in start_chunk..=stop_chunk {
            self.request_xys_chunk(chunk_idx);
        }
    }

    /// Updates chunk data directly from a sample array.
    pub fn update_chunk_data(&mut self, chunk_index: usize, samples: &[f64]) {
        if chunk_index >= self.chunk_loaded.len() {
            return;
        }
        self.chunk_in_progress[chunk_index] = false;
        let start_index = chunk_index * self.samples_per_xys_file * 3;
        let end_index = (start_index + samples.len()).min(self.samples.len());
        let copy_len = end_index.saturating_sub(start_index);
        self.samples[start_index..start_index + copy_len]
            .copy_from_slice(&samples[..copy_len]);
        self.chunk_loaded[chunk_index] = true;
    }

    /// Synchronously loads a chunk from disk.
    pub fn load_chunk(&mut self, chunk_index: usize) -> Result<(), RuntimeError> {
        let path = find_chunk_file(self.xys_file_url_template.as_deref(), chunk_index).ok_or_else(|| {
            RuntimeError::new(format!(
                "Could not find IAU2006 XYS chunk {} on disk",
                chunk_index
            ))
        })?;

        let content = std::fs::read_to_string(&path).map_err(|e| {
            RuntimeError::new(format!(
                "Failed to read IAU2006 XYS chunk from {}: {}",
                path.display(),
                e
            ))
        })?;

        let parsed: XysChunkJson = serde_json::from_str(&content).map_err(|e| {
            RuntimeError::new(format!("Failed to parse IAU2006 XYS JSON: {}", e))
        })?;

        self.update_chunk_data(chunk_index, &parsed.samples);
        Ok(())
    }

    fn drain_messages(&mut self) {
        while let Ok((idx, samples)) = self.receiver.try_recv() {
            self.update_chunk_data(idx, &samples);
        }
    }

    fn request_xys_chunk(&mut self, chunk_index: usize) {
        if chunk_index >= self.chunk_loaded.len() {
            return;
        }
        if self.chunk_loaded[chunk_index] || self.chunk_in_progress[chunk_index] {
            return;
        }

        self.chunk_in_progress[chunk_index] = true;

        #[cfg(not(target_arch = "wasm32"))]
        {
            let template = self.xys_file_url_template.clone();
            let sender = self.sender.clone();
            std::thread::spawn(move || {
                if let Some(path) = find_chunk_file(template.as_deref(), chunk_index) {
                    if let Ok(content) = std::fs::read_to_string(&path) {
                        if let Ok(parsed) = serde_json::from_str::<XysChunkJson>(&content) {
                            let _ = sender.send((chunk_index, parsed.samples));
                        }
                    }
                }
            });
        }
    }

    /// Computes the XYS values for a given date by interpolating.
    ///
    /// If required chunk data is not yet loaded, returns `None` and requests the chunks in the background.
    pub fn compute_xys_radians(
        &mut self,
        day_tt: i32,
        second_tt: f64,
    ) -> Option<Iau2006XysSample> {
        self.drain_messages();

        let days_since_epoch = self.get_days_since_epoch(day_tt, second_tt);
        if days_since_epoch < 0.0 {
            // Prior to epoch of data
            return None;
        }

        let center_index = (days_since_epoch / self.step_size_days).floor() as usize;
        if center_index >= self.total_samples {
            // After last sample in data
            return None;
        }

        let degree = self.interpolation_order;
        let mut first_index = (center_index as isize) - ((degree as isize) / 2);
        if first_index < 0 {
            first_index = 0;
        }
        let mut last_index = (first_index as usize) + degree;
        if last_index >= self.total_samples {
            last_index = self.total_samples - 1;
            first_index = (last_index as isize) - (degree as isize);
            if first_index < 0 {
                first_index = 0;
            }
        }
        let first_index = first_index as usize;

        let first_chunk = first_index / self.samples_per_xys_file;
        let last_chunk = last_index / self.samples_per_xys_file;

        let mut data_missing = false;
        if !self.chunk_loaded.get(first_chunk).copied().unwrap_or(false) {
            self.request_xys_chunk(first_chunk);
            data_missing = true;
        }
        if !self.chunk_loaded.get(last_chunk).copied().unwrap_or(false) {
            self.request_xys_chunk(last_chunk);
            data_missing = true;
        }

        if data_missing {
            return None;
        }

        let x = days_since_epoch - (first_index as f64) * self.step_size_days;

        for i in 0..=degree {
            self.work[i] = x - self.x_table[i];
        }

        let mut result = Iau2006XysSample::ZERO;

        for i in 0..=degree {
            self.coef[i] = 1.0;
            for j in 0..=degree {
                if j != i {
                    self.coef[i] *= self.work[j];
                }
            }
            self.coef[i] *= self.denominators[i];

            let sample_index = (first_index + i) * 3;
            result.x += self.coef[i] * self.samples[sample_index];
            result.y += self.coef[i] * self.samples[sample_index + 1];
            result.s += self.coef[i] * self.samples[sample_index + 2];
        }

        Some(result)
    }
}

fn find_chunk_file(template: Option<&str>, chunk_index: usize) -> Option<PathBuf> {
    if let Some(tmpl) = template {
        let path_str = tmpl.replace("{0}", &chunk_index.to_string());
        let p = PathBuf::from(&path_str);
        if p.exists() {
            return Some(p);
        }
    }

    // Try relative asset path
    let rel = format!("Assets/IAU2006_XYS/IAU2006_XYS_{}.json", chunk_index);
    let candidates = [
        PathBuf::from(&rel),
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()?
            .parent()?
            .parent()?
            .join("cesium/packages/engine/Source")
            .join(&rel),
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()?
            .parent()?
            .join("cesium/packages/engine/Source")
            .join(&rel),
    ];

    candidates.into_iter().find(|candidate| candidate.exists())
}
