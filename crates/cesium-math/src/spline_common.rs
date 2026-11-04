//! Common spline utilities matching CesiumJS `Core/Spline`.
//! Adapted from CesiumJS (Apache-2.0).

use crate::errors::DeveloperError;

pub fn find_time_interval_with_start(
    times: &[f64],
    time: f64,
    start_index: Option<usize>,
) -> Result<usize, DeveloperError> {
    let length = times.len();
    if length < 2 {
        return Err(DeveloperError::new("times must contain at least 2 points."));
    }
    if time < times[0] || time > times[length - 1] {
        return Err(DeveloperError::new("time is out of range."));
    }

    let start = start_index.unwrap_or(0).min(length - 1);

    if time >= times[start] {
        if start + 1 < length && time < times[start + 1] {
            return Ok(start);
        } else if start + 2 < length && time < times[start + 2] {
            return Ok(start + 1);
        }
    } else if start >= 1 && time >= times[start - 1] {
        return Ok(start - 1);
    }

    if time > times[start] {
        for i in start..(length - 1) {
            if time >= times[i] && time < times[i + 1] {
                return Ok(i);
            }
        }
    } else {
        for i in (0..start).rev() {
            if time >= times[i] && time < times[i + 1] {
                return Ok(i);
            }
        }
    }

    Ok(length - 2)
}

pub fn find_time_interval(times: &[f64], time: f64) -> Result<usize, DeveloperError> {
    find_time_interval_with_start(times, time, None)
}

pub fn wrap_time(times: &[f64], mut time: f64) -> f64 {
    if times.len() < 2 {
        return time;
    }
    let time_start = times[0];
    let time_end = times[times.len() - 1];
    let time_stretch = time_end - time_start;
    if time_stretch <= 0.0 {
        return time_start;
    }

    if time < time_start {
        let divs = ((time_start - time) / time_stretch).floor() + 1.0;
        time += divs * time_stretch;
    }
    if time > time_end {
        let divs = ((time - time_end) / time_stretch).floor() + 1.0;
        time -= divs * time_stretch;
    }
    time
}

pub fn clamp_time(times: &[f64], time: f64) -> f64 {
    if times.is_empty() {
        return time;
    }
    time.clamp(times[0], times[times.len() - 1])
}
