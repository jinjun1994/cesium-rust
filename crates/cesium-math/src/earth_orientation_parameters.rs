use std::path::Path;
use serde::{Deserialize, Serialize};

use crate::errors::RuntimeError;
use crate::julian_date::{JulianDate, LeapSecond, TimeStandard};
use crate::time_constants::MODIFIED_JULIAN_DATE_DIFFERENCE;

/// A set of Earth Orientation Parameters (EOP) sampled at a time.
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
pub struct EarthOrientationParametersSample {
    pub x_pole_wander: f64,
    pub y_pole_wander: f64,
    pub x_pole_offset: f64,
    pub y_pole_offset: f64,
    pub ut1_minus_utc: f64,
}

impl EarthOrientationParametersSample {
    pub const ZERO: Self = Self {
        x_pole_wander: 0.0,
        y_pole_wander: 0.0,
        x_pole_offset: 0.0,
        y_pole_offset: 0.0,
        ut1_minus_utc: 0.0,
    };

    pub fn new(
        x_pole_wander: f64,
        y_pole_wander: f64,
        x_pole_offset: f64,
        y_pole_offset: f64,
        ut1_minus_utc: f64,
    ) -> Self {
        Self {
            x_pole_wander,
            y_pole_wander,
            x_pole_offset,
            y_pole_offset,
            ut1_minus_utc,
        }
    }
}

/// Raw serialized EOP data.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EarthOrientationParametersData {
    pub column_names: Vec<String>,
    pub samples: Vec<serde_json::Value>,
}

/// Earth Orientation Parameters (EOP) manager for coordinate frames transforms.
#[derive(Debug, Clone)]
pub struct EarthOrientationParameters {
    dates: Vec<JulianDate>,
    samples: Vec<f64>,
    date_column: usize,
    x_pole_wander_radians_column: usize,
    y_pole_wander_radians_column: usize,
    ut1_minus_utc_seconds_column: usize,
    x_celestial_pole_offset_radians_column: usize,
    y_celestial_pole_offset_radians_column: usize,
    tai_minus_utc_seconds_column: usize,
    column_count: usize,
    last_index: Option<usize>,
    add_new_leap_seconds: bool,
}

impl Default for EarthOrientationParameters {
    fn default() -> Self {
        Self::new(None, true).expect("default zero eop")
    }
}

impl EarthOrientationParameters {
    /// Returns whether new leap seconds found in data will be registered.
    pub fn add_new_leap_seconds(&self) -> bool {
        self.add_new_leap_seconds
    }

    /// Returns the column index of modifiedJulianDateUtc.
    pub fn date_column(&self) -> usize {
        self.date_column
    }
    pub fn new(
        data: Option<EarthOrientationParametersData>,
        add_new_leap_seconds: bool,
    ) -> Result<Self, RuntimeError> {
        let eop_data = data.unwrap_or_else(|| EarthOrientationParametersData {
            column_names: vec![
                "dateIso8601".to_string(),
                "modifiedJulianDateUtc".to_string(),
                "xPoleWanderRadians".to_string(),
                "yPoleWanderRadians".to_string(),
                "ut1MinusUtcSeconds".to_string(),
                "lengthOfDayCorrectionSeconds".to_string(),
                "xCelestialPoleOffsetRadians".to_string(),
                "yCelestialPoleOffsetRadians".to_string(),
                "taiMinusUtcSeconds".to_string(),
            ],
            samples: Vec::new(),
        });

        let find_col = |name: &str| -> Result<usize, RuntimeError> {
            eop_data
                .column_names
                .iter()
                .position(|c| c == name)
                .ok_or_else(|| {
                    RuntimeError::new(
                        "Error in loaded EOP data: The columnNames property must include modifiedJulianDateUtc, xPoleWanderRadians, yPoleWanderRadians, ut1MinusUtcSeconds, xCelestialPoleOffsetRadians, yCelestialPoleOffsetRadians, and taiMinusUtcSeconds columns",
                    )
                })
        };

        let date_col = find_col("modifiedJulianDateUtc")?;
        let x_wander_col = find_col("xPoleWanderRadians")?;
        let y_wander_col = find_col("yPoleWanderRadians")?;
        let ut1_col = find_col("ut1MinusUtcSeconds")?;
        let x_offset_col = find_col("xCelestialPoleOffsetRadians")?;
        let y_offset_col = find_col("yCelestialPoleOffsetRadians")?;
        let tai_col = find_col("taiMinusUtcSeconds")?;

        let column_count = eop_data.column_names.len();
        let sample_len = eop_data.samples.len();

        let mut dates = Vec::new();
        let mut numeric_samples = Vec::with_capacity(sample_len);
        for val in &eop_data.samples {
            numeric_samples.push(val.as_f64().unwrap_or(0.0));
        }

        let mut last_tai_minus_utc: Option<f64> = None;

        let mut i = 0;
        while i < sample_len {
            let mjd = numeric_samples[i + date_col];
            let tai_minus_utc = numeric_samples[i + tai_col];
            let day = mjd + MODIFIED_JULIAN_DATE_DIFFERENCE;
            let date = JulianDate::new(day, tai_minus_utc, TimeStandard::TAI);
            dates.push(date);

            if add_new_leap_seconds {
                if let Some(last_tai) = last_tai_minus_utc {
                    if (tai_minus_utc - last_tai).abs() > 1e-6 {
                        JulianDate::add_leap_second(LeapSecond::new(date, tai_minus_utc));
                    }
                }
                last_tai_minus_utc = Some(tai_minus_utc);
            }

            i += column_count;
        }

        Ok(Self {
            dates,
            samples: numeric_samples,
            date_column: date_col,
            x_pole_wander_radians_column: x_wander_col,
            y_pole_wander_radians_column: y_wander_col,
            ut1_minus_utc_seconds_column: ut1_col,
            x_celestial_pole_offset_radians_column: x_offset_col,
            y_celestial_pole_offset_radians_column: y_offset_col,
            tai_minus_utc_seconds_column: tai_col,
            column_count,
            last_index: None,
            add_new_leap_seconds,
        })
    }

    /// Loads and parses EOP from JSON string.
    pub fn from_json(json_str: &str, add_new_leap_seconds: bool) -> Result<Self, RuntimeError> {
        let value: serde_json::Value = serde_json::from_str(json_str)
            .map_err(|e| RuntimeError::new(format!("Failed to parse EOP JSON: {}", e)))?;

        let column_names_val = value.get("columnNames").ok_or_else(|| {
            RuntimeError::new("Error in loaded EOP data: The columnNames property is required.")
        })?;
        let samples_val = value.get("samples").ok_or_else(|| {
            RuntimeError::new("Error in loaded EOP data: The samples property is required.")
        })?;

        let column_names: Vec<String> = serde_json::from_value(column_names_val.clone())
            .map_err(|_| RuntimeError::new("Invalid columnNames array"))?;
        let samples: Vec<serde_json::Value> = serde_json::from_value(samples_val.clone())
            .map_err(|_| RuntimeError::new("Invalid samples array"))?;

        Self::new(
            Some(EarthOrientationParametersData {
                column_names,
                samples,
            }),
            add_new_leap_seconds,
        )
    }

    /// Loads EOP from a file path.
    pub fn from_file(path: impl AsRef<Path>, add_new_leap_seconds: bool) -> Result<Self, RuntimeError> {
        let path_ref = path.as_ref();
        let content = std::fs::read_to_string(path_ref).map_err(|_| {
            RuntimeError::new(format!(
                "An error occurred while retrieving the EOP data from the URL {}.",
                path_ref.display()
            ))
        })?;
        Self::from_json(&content, add_new_leap_seconds)
    }

    /// Computes interpolated Earth orientation parameters for a given date.
    pub fn compute(&mut self, date: JulianDate) -> EarthOrientationParametersSample {
        if self.samples.is_empty() {
            return EarthOrientationParametersSample::ZERO;
        }

        let dates = &self.dates;
        let mut before;
        let after;

        if let Some(last) = self.last_index {
            let prev_date = dates[last];
            let next_date = dates.get(last + 1).copied();
            let is_after_prev = prev_date.compare(&date).is_le();
            let is_after_last_sample = next_date.is_none();
            let is_before_next = is_after_last_sample || next_date.unwrap().compare(&date).is_ge();

            if is_after_prev && is_before_next {
                before = last;
                if let Some(nxt) = next_date {
                    if nxt.equals(&date) {
                        before += 1;
                    }
                }
                after = before + 1;
                return self.interpolate(date, before, after);
            }
        }

        match dates.binary_search_by(|d| d.compare(&date)) {
            Ok(mut idx) => {
                if idx + 1 < dates.len() && dates[idx + 1].equals(&date) {
                    idx += 1;
                }
                before = idx;
                self.last_index = Some(before);
                self.interpolate(date, before, before)
            }
            Err(idx) => {
                let aft = idx;
                before = aft.saturating_sub(1);
                self.last_index = Some(before);
                self.interpolate(date, before, aft)
            }
        }
    }

    fn fill_from_index(&self, index: usize, result: &mut EarthOrientationParametersSample) {
        let start = index * self.column_count;
        result.x_pole_wander = self.samples[start + self.x_pole_wander_radians_column];
        result.y_pole_wander = self.samples[start + self.y_pole_wander_radians_column];
        result.x_pole_offset = self.samples[start + self.x_celestial_pole_offset_radians_column];
        result.y_pole_offset = self.samples[start + self.y_celestial_pole_offset_radians_column];
        result.ut1_minus_utc = self.samples[start + self.ut1_minus_utc_seconds_column];
    }

    fn interpolate(&self, date: JulianDate, before: usize, after: usize) -> EarthOrientationParametersSample {
        let dates = &self.dates;
        let mut result = EarthOrientationParametersSample::ZERO;

        if after > dates.len() - 1 {
            return result;
        }

        let before_date = dates[before];
        let after_date = dates[after];

        if before_date.equals(&after_date) || date.equals(&before_date) {
            self.fill_from_index(before, &mut result);
            return result;
        } else if date.equals(&after_date) {
            self.fill_from_index(after, &mut result);
            return result;
        }

        let diff_after_before = after_date.seconds_difference(&before_date);
        let factor = if diff_after_before.abs() > 1e-12 {
            date.seconds_difference(&before_date) / diff_after_before
        } else {
            0.0
        };

        let start_before = before * self.column_count;
        let start_after = after * self.column_count;

        let mut before_ut1 = self.samples[start_before + self.ut1_minus_utc_seconds_column];
        let mut after_ut1 = self.samples[start_after + self.ut1_minus_utc_seconds_column];

        let offset_diff = after_ut1 - before_ut1;
        if offset_diff > 0.5 || offset_diff < -0.5 {
            let before_tai = self.samples[start_before + self.tai_minus_utc_seconds_column];
            let after_tai = self.samples[start_after + self.tai_minus_utc_seconds_column];
            if (before_tai - after_tai).abs() > 1e-6 {
                if after_date.equals(&date) {
                    // At the end of the leap second, use the jump value
                    before_ut1 = after_ut1;
                } else {
                    after_ut1 -= after_tai - before_tai;
                }
            }
        }

        result.x_pole_wander = linear_interp(
            factor,
            self.samples[start_before + self.x_pole_wander_radians_column],
            self.samples[start_after + self.x_pole_wander_radians_column],
        );
        result.y_pole_wander = linear_interp(
            factor,
            self.samples[start_before + self.y_pole_wander_radians_column],
            self.samples[start_after + self.y_pole_wander_radians_column],
        );
        result.x_pole_offset = linear_interp(
            factor,
            self.samples[start_before + self.x_celestial_pole_offset_radians_column],
            self.samples[start_after + self.x_celestial_pole_offset_radians_column],
        );
        result.y_pole_offset = linear_interp(
            factor,
            self.samples[start_before + self.y_celestial_pole_offset_radians_column],
            self.samples[start_after + self.y_celestial_pole_offset_radians_column],
        );
        result.ut1_minus_utc = linear_interp(factor, before_ut1, after_ut1);

        result
    }
}

fn linear_interp(dx: f64, y1: f64, y2: f64) -> f64 {
    y1 + dx * (y2 - y1)
}
