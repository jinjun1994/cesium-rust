//! JulianDate and leap second time systems matching CesiumJS `Core/JulianDate`, `Core/LeapSecond`, `Core/TimeStandard`, and `Core/GregorianDate`.
//! Adapted from CesiumJS (Apache-2.0).

#![allow(
    clippy::too_many_arguments,
    clippy::comparison_chain
)]

use crate::{time_constants::*, MathError};
use std::cmp::Ordering;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TimeStandard {
    UTC = 0,
    TAI = 1,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LeapSecond {
    pub julian_date: JulianDate,
    pub offset: f64,
}

impl LeapSecond {
    pub const fn new(julian_date: JulianDate, offset: f64) -> Self {
        Self {
            julian_date,
            offset,
        }
    }
}

pub static LEAP_SECONDS: [LeapSecond; 28] = [
    LeapSecond::new(JulianDate::new_tai(2441317, 43210.0), 10.0), // January 1, 1972 00:00:00 UTC
    LeapSecond::new(JulianDate::new_tai(2441499, 43211.0), 11.0), // July 1, 1972 00:00:00 UTC
    LeapSecond::new(JulianDate::new_tai(2441683, 43212.0), 12.0), // January 1, 1973 00:00:00 UTC
    LeapSecond::new(JulianDate::new_tai(2442048, 43213.0), 13.0), // January 1, 1974 00:00:00 UTC
    LeapSecond::new(JulianDate::new_tai(2442413, 43214.0), 14.0), // January 1, 1975 00:00:00 UTC
    LeapSecond::new(JulianDate::new_tai(2442778, 43215.0), 15.0), // January 1, 1976 00:00:00 UTC
    LeapSecond::new(JulianDate::new_tai(2443144, 43216.0), 16.0), // January 1, 1977 00:00:00 UTC
    LeapSecond::new(JulianDate::new_tai(2443509, 43217.0), 17.0), // January 1, 1978 00:00:00 UTC
    LeapSecond::new(JulianDate::new_tai(2443874, 43218.0), 18.0), // January 1, 1979 00:00:00 UTC
    LeapSecond::new(JulianDate::new_tai(2444239, 43219.0), 19.0), // January 1, 1980 00:00:00 UTC
    LeapSecond::new(JulianDate::new_tai(2444786, 43220.0), 20.0), // July 1, 1981 00:00:00 UTC
    LeapSecond::new(JulianDate::new_tai(2445151, 43221.0), 21.0), // July 1, 1982 00:00:00 UTC
    LeapSecond::new(JulianDate::new_tai(2445516, 43222.0), 22.0), // July 1, 1983 00:00:00 UTC
    LeapSecond::new(JulianDate::new_tai(2446247, 43223.0), 23.0), // July 1, 1985 00:00:00 UTC
    LeapSecond::new(JulianDate::new_tai(2447161, 43224.0), 24.0), // January 1, 1988 00:00:00 UTC
    LeapSecond::new(JulianDate::new_tai(2447892, 43225.0), 25.0), // January 1, 1990 00:00:00 UTC
    LeapSecond::new(JulianDate::new_tai(2448257, 43226.0), 26.0), // January 1, 1991 00:00:00 UTC
    LeapSecond::new(JulianDate::new_tai(2448804, 43227.0), 27.0), // July 1, 1992 00:00:00 UTC
    LeapSecond::new(JulianDate::new_tai(2449169, 43228.0), 28.0), // July 1, 1993 00:00:00 UTC
    LeapSecond::new(JulianDate::new_tai(2449534, 43229.0), 29.0), // July 1, 1994 00:00:00 UTC
    LeapSecond::new(JulianDate::new_tai(2450083, 43230.0), 30.0), // January 1, 1996 00:00:00 UTC
    LeapSecond::new(JulianDate::new_tai(2450630, 43231.0), 31.0), // July 1, 1997 00:00:00 UTC
    LeapSecond::new(JulianDate::new_tai(2451179, 43232.0), 32.0), // January 1, 1999 00:00:00 UTC
    LeapSecond::new(JulianDate::new_tai(2453736, 43233.0), 33.0), // January 1, 2006 00:00:00 UTC
    LeapSecond::new(JulianDate::new_tai(2454832, 43234.0), 34.0), // January 1, 2009 00:00:00 UTC
    LeapSecond::new(JulianDate::new_tai(2456109, 43235.0), 35.0), // July 1, 2012 00:00:00 UTC
    LeapSecond::new(JulianDate::new_tai(2457204, 43236.0), 36.0), // July 1, 2015 00:00:00 UTC
    LeapSecond::new(JulianDate::new_tai(2457754, 43237.0), 37.0), // January 1, 2017 00:00:00 UTC
];

fn binary_search_leap_second(date: &JulianDate) -> Result<usize, usize> {
    LEAP_SECONDS.binary_search_by(|ls| ls.julian_date.compare(date))
}

fn convert_utc_to_tai(jd: JulianDate) -> JulianDate {
    let mut index = match binary_search_leap_second(&jd) {
        Ok(i) => i,
        Err(i) => i,
    };
    if index >= LEAP_SECONDS.len() {
        index = LEAP_SECONDS.len() - 1;
    }
    let mut offset = LEAP_SECONDS[index].offset;
    if index > 0 {
        let diff = LEAP_SECONDS[index].julian_date.seconds_difference(&jd);
        if diff > offset {
            index -= 1;
            offset = LEAP_SECONDS[index].offset;
        }
    }
    jd.add_seconds(offset)
}

fn convert_tai_to_utc(jd: &JulianDate) -> Option<JulianDate> {
    let index = match binary_search_leap_second(jd) {
        Ok(i) => i,
        Err(i) => i,
    };
    if index == 0 {
        return Some(jd.add_seconds(-LEAP_SECONDS[0].offset));
    }
    if index >= LEAP_SECONDS.len() {
        return Some(jd.add_seconds(-LEAP_SECONDS[LEAP_SECONDS.len() - 1].offset));
    }
    let diff = LEAP_SECONDS[index].julian_date.seconds_difference(jd);
    if diff == 0.0 {
        return Some(jd.add_seconds(-LEAP_SECONDS[index].offset));
    }
    if diff <= 1.0 {
        // during leap second moment
        return None;
    }
    Some(jd.add_seconds(-LEAP_SECONDS[index - 1].offset))
}

fn normalize_components(mut whole_days: i32, mut seconds_of_day: f64) -> (i32, f64) {
    let extra_days = (seconds_of_day / SECONDS_PER_DAY).floor() as i32;
    whole_days += extra_days;
    seconds_of_day -= extra_days as f64 * SECONDS_PER_DAY;
    if seconds_of_day < 0.0 {
        whole_days -= 1;
        seconds_of_day += SECONDS_PER_DAY;
    }
    (whole_days, seconds_of_day)
}

fn compute_julian_date_components(
    year: i32,
    month: i32,
    day: i32,
    mut hour: i32,
    minute: i32,
    second: i32,
    millisecond: f64,
) -> (i32, f64) {
    let a = (month - 14) / 12;
    let b = year + 4800 + a;
    let mut day_number = ((1461 * b) / 4) + ((367 * (month - 2 - 12 * a)) / 12)
        - ((3 * ((b + 100) / 100)) / 4)
        + day
        - 32075;

    hour -= 12;
    if hour < 0 {
        hour += 24;
    }
    let seconds_of_day = second as f64
        + (hour as f64 * SECONDS_PER_HOUR
            + minute as f64 * SECONDS_PER_MINUTE
            + millisecond * SECONDS_PER_MILLISECOND);

    if seconds_of_day >= 43200.0 {
        day_number -= 1;
    }
    (day_number, seconds_of_day)
}

// ============================================================================
// GregorianDate
// ============================================================================

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GregorianDate {
    pub year: i32,
    pub month: u8,
    pub day: u8,
    pub hour: u8,
    pub minute: u8,
    pub second: u8,
    pub millisecond: f64,
    pub is_leap_second: bool,
}

impl Default for GregorianDate {
    fn default() -> Self {
        Self {
            year: 1,
            month: 1,
            day: 1,
            hour: 0,
            minute: 0,
            second: 0,
            millisecond: 0.0,
            is_leap_second: false,
        }
    }
}

// ============================================================================
// JulianDate
// ============================================================================

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct JulianDate {
    pub day_number: i32,
    pub seconds_of_day: f64,
}

impl Default for JulianDate {
    fn default() -> Self {
        Self {
            day_number: 0,
            seconds_of_day: 10.0,
        }
    }
}

impl JulianDate {
    #[inline]
    pub const fn new_tai(day_number: i32, seconds_of_day: f64) -> Self {
        Self {
            day_number,
            seconds_of_day,
        }
    }

    pub fn new(day_number: f64, mut seconds_of_day: f64, standard: TimeStandard) -> Self {
        let whole_days = day_number.floor() as i32;
        let frac_day = day_number - whole_days as f64;
        seconds_of_day += frac_day * SECONDS_PER_DAY;

        let (d, s) = normalize_components(whole_days, seconds_of_day);
        let jd = Self {
            day_number: d,
            seconds_of_day: s,
        };
        match standard {
            TimeStandard::UTC => convert_utc_to_tai(jd),
            TimeStandard::TAI => jd,
        }
    }

    pub fn from_utc_components(
        year: i32,
        month: i32,
        day: i32,
        hour: i32,
        minute: i32,
        second: i32,
        millisecond: f64,
    ) -> Self {
        let (day_number, seconds_of_day) =
            compute_julian_date_components(year, month, day, hour, minute, second, millisecond);
        Self::new(day_number as f64, seconds_of_day, TimeStandard::UTC)
    }

    pub fn from_tai_components(
        year: i32,
        month: i32,
        day: i32,
        hour: i32,
        minute: i32,
        second: i32,
        millisecond: f64,
    ) -> Self {
        let (day_number, seconds_of_day) =
            compute_julian_date_components(year, month, day, hour, minute, second, millisecond);
        Self::new(day_number as f64, seconds_of_day, TimeStandard::TAI)
    }

    pub fn to_gregorian_date(&self) -> GregorianDate {
        let mut is_leap_second = false;
        let utc = match convert_tai_to_utc(self) {
            Some(u) => u,
            None => {
                let minus_one = self.add_seconds(-1.0);
                is_leap_second = true;
                convert_tai_to_utc(&minus_one).unwrap_or(minus_one)
            }
        };

        let mut julian_day_number = utc.day_number;
        let seconds_of_day = utc.seconds_of_day;

        if seconds_of_day >= 43200.0 {
            julian_day_number += 1;
        }

        let mut l = julian_day_number + 68569;
        let n = (4 * l) / 146097;
        l -= (146097 * n + 3) / 4;
        let i = (4000 * (l + 1)) / 1461001;
        l = l - ((1461 * i) / 4) + 31;
        let j = (80 * l) / 2447;
        let day = l - ((2447 * j) / 80);
        l = j / 11;
        let month = j + 2 - 12 * l;
        let year = 100 * (n - 49) + i + l;

        let mut hour = (seconds_of_day / SECONDS_PER_HOUR) as i32;
        let mut rem_seconds = seconds_of_day - hour as f64 * SECONDS_PER_HOUR;
        let minute = (rem_seconds / SECONDS_PER_MINUTE) as i32;
        rem_seconds -= minute as f64 * SECONDS_PER_MINUTE;
        let mut second = rem_seconds.floor() as i32;
        let millisecond = (rem_seconds - second as f64) / SECONDS_PER_MILLISECOND;

        hour += 12;
        if hour > 23 {
            hour -= 24;
        }
        if is_leap_second {
            second += 1;
        }

        GregorianDate {
            year,
            month: month as u8,
            day: day as u8,
            hour: hour as u8,
            minute: minute as u8,
            second: second as u8,
            millisecond,
            is_leap_second,
        }
    }

    #[inline]
    pub fn add_seconds(&self, seconds: f64) -> Self {
        let (d, s) = normalize_components(self.day_number, self.seconds_of_day + seconds);
        Self {
            day_number: d,
            seconds_of_day: s,
        }
    }

    #[inline]
    pub fn add_minutes(&self, minutes: f64) -> Self {
        self.add_seconds(minutes * SECONDS_PER_MINUTE)
    }

    #[inline]
    pub fn add_hours(&self, hours: f64) -> Self {
        self.add_seconds(hours * SECONDS_PER_HOUR)
    }

    #[inline]
    pub fn add_days(&self, days: f64) -> Self {
        self.add_seconds(days * SECONDS_PER_DAY)
    }

    #[inline]
    pub fn seconds_difference(&self, other: &Self) -> f64 {
        let day_diff = (self.day_number - other.day_number) as f64;
        day_diff * SECONDS_PER_DAY + (self.seconds_of_day - other.seconds_of_day)
    }

    #[inline]
    pub fn days_difference(&self, other: &Self) -> f64 {
        self.seconds_difference(other) / SECONDS_PER_DAY
    }

    #[inline]
    pub fn total_days(&self) -> f64 {
        self.day_number as f64 + (self.seconds_of_day / SECONDS_PER_DAY)
    }

    #[inline]
    pub fn compare(&self, other: &Self) -> Ordering {
        match self.day_number.cmp(&other.day_number) {
            Ordering::Equal => self
                .seconds_of_day
                .partial_cmp(&other.seconds_of_day)
                .unwrap_or(Ordering::Equal),
            ord => ord,
        }
    }

    #[inline]
    pub fn equals(&self, other: &Self) -> bool {
        self.day_number == other.day_number && self.seconds_of_day == other.seconds_of_day
    }

    #[inline]
    pub fn equals_epsilon(&self, other: &Self, epsilon: f64) -> bool {
        self.seconds_difference(other).abs() <= epsilon
    }

    #[inline]
    pub fn less_than(&self, other: &Self) -> bool {
        self.compare(other) == Ordering::Less
    }

    #[inline]
    pub fn less_than_or_equals(&self, other: &Self) -> bool {
        self.compare(other) != Ordering::Greater
    }

    #[inline]
    pub fn greater_than(&self, other: &Self) -> bool {
        self.compare(other) == Ordering::Greater
    }

    #[inline]
    pub fn greater_than_or_equals(&self, other: &Self) -> bool {
        self.compare(other) != Ordering::Less
    }

    /// Parse simple ISO 8601 strings, e.g. "2000-01-01T12:00:00Z" or "1980-08-01T00:00:00Z"
    pub fn from_iso8601(iso: &str) -> Result<Self, MathError> {
        let s = iso.trim();
        // Support format like YYYY-MM-DDTHH:MM:SSZ or YYYY-MM-DD
        let parts: Vec<&str> = s.split('T').collect();
        if parts.is_empty() {
            return Err(MathError);
        }
        let date_parts: Vec<&str> = parts[0].split('-').collect();
        if date_parts.len() != 3 {
            return Err(MathError);
        }
        let year: i32 = date_parts[0].parse().map_err(|_| MathError)?;
        let month: i32 = date_parts[1].parse().map_err(|_| MathError)?;
        let day: i32 = date_parts[2].parse().map_err(|_| MathError)?;

        let mut hour = 0;
        let mut minute = 0;
        let mut second = 0;
        let mut millisecond = 0.0;

        if parts.len() > 1 {
            let time_str = parts[1].trim_end_matches('Z');
            let time_parts: Vec<&str> = time_str.split(':').collect();
            if !time_parts.is_empty() {
                hour = time_parts[0].parse().map_err(|_| MathError)?;
            }
            if time_parts.len() > 1 {
                minute = time_parts[1].parse().map_err(|_| MathError)?;
            }
            if time_parts.len() > 2 {
                let sec_str = time_parts[2];
                if let Some(dot_idx) = sec_str.find('.') {
                    second = sec_str[..dot_idx].parse().map_err(|_| MathError)?;
                    let frac: f64 = format!("0.{}", &sec_str[dot_idx + 1..])
                        .parse()
                        .map_err(|_| MathError)?;
                    millisecond = frac * 1000.0;
                } else {
                    second = sec_str.parse().map_err(|_| MathError)?;
                }
            }
        }

        Ok(Self::from_utc_components(
            year,
            month,
            day,
            hour,
            minute,
            second,
            millisecond,
        ))
    }

    pub fn to_iso8601(&self) -> String {
        let g = self.to_gregorian_date();
        let ms = g.millisecond.round() as u32;
        if ms > 0 {
            format!(
                "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}.{:03}Z",
                g.year, g.month, g.day, g.hour, g.minute, g.second, ms
            )
        } else {
            format!(
                "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
                g.year, g.month, g.day, g.hour, g.minute, g.second
            )
        }
    }
}
