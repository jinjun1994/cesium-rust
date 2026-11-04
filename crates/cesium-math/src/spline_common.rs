//! Common spline utilities matching CesiumJS `Core/Spline`.
//! Adapted from CesiumJS (Apache-2.0).

use crate::errors::DeveloperError;

pub fn find_time_interval(times: &[f64], time: f64) -> Result<usize, DeveloperError> {
    let len = times.len();
    if len < 2 {
        return Err(DeveloperError::new("times must contain at least 2 points."));
    }
    if time < times[0] || time > times[len - 1] {
        return Err(DeveloperError::new("time is out of range."));
    }

    for i in 0..(len - 1) {
        if time >= times[i] && time < times[i + 1] {
            return Ok(i);
        }
    }
    Ok(len - 2)
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
