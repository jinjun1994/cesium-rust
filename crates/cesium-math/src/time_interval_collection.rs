//! TimeIntervalCollection matching CesiumJS `Core/TimeIntervalCollection`.
//! Adapted from CesiumJS (Apache-2.0).

use crate::{julian_date::JulianDate, time_interval::TimeInterval};
use std::cmp::Ordering;

#[derive(Clone, Debug, PartialEq)]
pub struct TimeIntervalCollection<T = ()> {
    intervals: Vec<TimeInterval<T>>,
}

impl<T> Default for TimeIntervalCollection<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> TimeIntervalCollection<T> {
    pub const fn new() -> Self {
        Self {
            intervals: Vec::new(),
        }
    }

    pub fn len(&self) -> usize {
        self.intervals.len()
    }

    pub fn is_empty(&self) -> bool {
        self.intervals.is_empty()
    }

    pub fn start(&self) -> Option<JulianDate> {
        self.intervals.first().map(|i| i.start)
    }

    pub fn stop(&self) -> Option<JulianDate> {
        self.intervals.last().map(|i| i.stop)
    }

    pub fn is_start_included(&self) -> bool {
        self.intervals.first().is_some_and(|i| i.is_start_included)
    }

    pub fn is_stop_included(&self) -> bool {
        self.intervals.last().is_some_and(|i| i.is_stop_included)
    }

    pub fn get(&self, index: usize) -> Option<&TimeInterval<T>> {
        self.intervals.get(index)
    }

    pub fn remove_all(&mut self) {
        self.intervals.clear();
    }

    pub fn contains(&self, date: &JulianDate) -> bool {
        self.index_of(date).is_ok()
    }

    pub fn index_of(&self, date: &JulianDate) -> Result<usize, usize> {
        let mut low = 0;
        let mut high = self.intervals.len();

        while low < high {
            let mid = low + (high - low) / 2;
            let interval = &self.intervals[mid];
            let cmp = interval.start.compare(date);

            match cmp {
                Ordering::Equal => {
                    if interval.is_start_included {
                        return Ok(mid);
                    }
                    if mid > 0
                        && self.intervals[mid - 1].stop.equals(date)
                        && self.intervals[mid - 1].is_stop_included
                    {
                        return Ok(mid - 1);
                    }
                    return Err(mid);
                }
                Ordering::Less => {
                    low = mid + 1;
                }
                Ordering::Greater => {
                    high = mid;
                }
            }
        }

        if low > 0 && self.intervals[low - 1].contains(date) {
            Ok(low - 1)
        } else {
            Err(low)
        }
    }

    pub fn find_interval_containing_date(&self, date: &JulianDate) -> Option<&TimeInterval<T>> {
        self.index_of(date).ok().and_then(|idx| self.intervals.get(idx))
    }

    pub fn find_data_for_interval_containing_date(&self, date: &JulianDate) -> Option<&T> {
        self.find_interval_containing_date(date).and_then(|i| i.data.as_ref())
    }

    pub fn find_interval(
        &self,
        start: Option<JulianDate>,
        stop: Option<JulianDate>,
        is_start_included: Option<bool>,
        is_stop_included: Option<bool>,
    ) -> Option<&TimeInterval<T>> {
        self.intervals.iter().find(|interval| {
            start.is_none_or(|s| interval.start.equals(&s))
                && stop.is_none_or(|s| interval.stop.equals(&s))
                && is_start_included.is_none_or(|si| interval.is_start_included == si)
                && is_stop_included.is_none_or(|si| interval.is_stop_included == si)
        })
    }
}

impl<T: Clone + PartialEq> TimeIntervalCollection<T> {
    pub fn from_intervals(intervals: Vec<TimeInterval<T>>) -> Self {
        let mut collection = Self::new();
        for interval in intervals {
            collection.add_interval(interval);
        }
        collection
    }

    pub fn add_interval(&mut self, mut interval: TimeInterval<T>) {
        if interval.is_empty() {
            return;
        }

        if self.intervals.is_empty()
            || interval.start.compare(&self.intervals.last().unwrap().stop) == Ordering::Greater
        {
            self.intervals.push(interval);
            return;
        }

        let mut index = self.intervals
            .binary_search_by(|it| it.start.compare(&interval.start))
            .unwrap_or_else(|idx| idx);

        if index > 0
            && interval.is_start_included
            && self.intervals[index - 1].is_start_included
            && self.intervals[index - 1].start.equals(&interval.start)
        {
            index -= 1;
        } else if index < self.intervals.len()
            && !interval.is_start_included
            && self.intervals[index].is_start_included
            && self.intervals[index].start.equals(&interval.start)
        {
            index += 1;
        }

        if index > 0 {
            let prev = &self.intervals[index - 1];
            let cmp = prev.stop.compare(&interval.start);
            let overlaps = cmp == Ordering::Greater
                || (cmp == Ordering::Equal && (prev.is_stop_included || interval.is_start_included));

            if overlaps {
                if prev.data == interval.data {
                    let new_stop = if interval.stop.compare(&prev.stop) == Ordering::Greater {
                        interval.stop
                    } else {
                        prev.stop
                    };
                    let new_is_stop_included = if interval.stop.compare(&prev.stop) == Ordering::Greater {
                        interval.is_stop_included
                    } else {
                        prev.is_stop_included
                            || (interval.stop.equals(&prev.stop) && interval.is_stop_included)
                    };
                    interval = TimeInterval {
                        start: prev.start,
                        stop: new_stop,
                        is_start_included: prev.is_start_included,
                        is_stop_included: new_is_stop_included,
                        data: interval.data,
                    };
                    self.intervals.remove(index - 1);
                    index -= 1;
                } else {
                    let prev_stop_cmp = prev.stop.compare(&interval.stop);
                    let split = prev_stop_cmp == Ordering::Greater
                        || (prev_stop_cmp == Ordering::Equal
                            && prev.is_stop_included
                            && !interval.is_stop_included);

                    if split {
                        let second_piece = TimeInterval {
                            start: interval.stop,
                            stop: prev.stop,
                            is_start_included: !interval.is_stop_included,
                            is_stop_included: prev.is_stop_included,
                            data: prev.data.clone(),
                        };
                        self.intervals.insert(index, second_piece);
                    }

                    self.intervals[index - 1] = TimeInterval {
                        start: self.intervals[index - 1].start,
                        stop: interval.start,
                        is_start_included: self.intervals[index - 1].is_start_included,
                        is_stop_included: !interval.is_start_included,
                        data: self.intervals[index - 1].data.clone(),
                    };
                }
            }
        }

        while index < self.intervals.len() {
            let curr = &self.intervals[index];
            let cmp = interval.stop.compare(&curr.start);
            let overlaps = cmp == Ordering::Greater
                || (cmp == Ordering::Equal && (interval.is_stop_included || curr.is_start_included));

            if overlaps {
                if curr.data == interval.data {
                    let new_stop = if curr.stop.compare(&interval.stop) == Ordering::Greater {
                        curr.stop
                    } else {
                        interval.stop
                    };
                    let new_is_stop_included = if curr.stop.compare(&interval.stop) == Ordering::Greater {
                        curr.is_stop_included
                    } else {
                        interval.is_stop_included
                    };
                    interval = TimeInterval {
                        start: interval.start,
                        stop: new_stop,
                        is_start_included: interval.is_start_included,
                        is_stop_included: new_is_stop_included,
                        data: interval.data,
                    };
                    self.intervals.remove(index);
                } else {
                    self.intervals[index] = TimeInterval {
                        start: interval.stop,
                        stop: curr.stop,
                        is_start_included: !interval.is_stop_included,
                        is_stop_included: curr.is_stop_included,
                        data: curr.data.clone(),
                    };

                    if self.intervals[index].is_empty() {
                        self.intervals.remove(index);
                    } else {
                        break;
                    }
                }
            } else {
                break;
            }
        }

        self.intervals.insert(index, interval);
    }

    pub fn remove_interval<U>(&mut self, interval: &TimeInterval<U>) -> bool {
        if interval.is_empty() {
            return false;
        }

        let index = self.intervals
            .binary_search_by(|it| it.start.compare(&interval.start))
            .unwrap_or_else(|idx| idx);

        let mut result = false;

        if index > 0 {
            let prev = &self.intervals[index - 1];
            let cmp = prev.stop.compare(&interval.start);
            let overlaps = cmp == Ordering::Greater
                || (cmp == Ordering::Equal && prev.is_stop_included && interval.is_start_included);

            if overlaps {
                result = true;
                let prev_stop_cmp = prev.stop.compare(&interval.stop);
                let split = prev_stop_cmp == Ordering::Greater
                    || (prev_stop_cmp == Ordering::Equal
                        && prev.is_stop_included
                        && !interval.is_stop_included);

                if split {
                    let second_piece = TimeInterval {
                        start: interval.stop,
                        stop: prev.stop,
                        is_start_included: !interval.is_stop_included,
                        is_stop_included: prev.is_stop_included,
                        data: prev.data.clone(),
                    };
                    self.intervals.insert(index, second_piece);
                }

                self.intervals[index - 1] = TimeInterval {
                    start: self.intervals[index - 1].start,
                    stop: interval.start,
                    is_start_included: self.intervals[index - 1].is_start_included,
                    is_stop_included: !interval.is_start_included,
                    data: self.intervals[index - 1].data.clone(),
                };
            }
        }

        while index < self.intervals.len() {
            let curr = &self.intervals[index];
            let cmp = interval.stop.compare(&curr.start);
            let overlaps = cmp == Ordering::Greater
                || (cmp == Ordering::Equal && interval.is_stop_included && curr.is_start_included);

            if overlaps {
                result = true;
                let curr_stop_cmp = interval.stop.compare(&curr.stop);
                if curr_stop_cmp == Ordering::Greater
                    || (curr_stop_cmp == Ordering::Equal && interval.is_stop_included)
                {
                    self.intervals.remove(index);
                } else {
                    self.intervals[index] = TimeInterval {
                        start: interval.stop,
                        stop: curr.stop,
                        is_start_included: !interval.is_stop_included,
                        is_stop_included: curr.is_stop_included,
                        data: curr.data.clone(),
                    };
                    break;
                }
            } else {
                break;
            }
        }

        result
    }
}
