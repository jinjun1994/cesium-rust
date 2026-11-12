use std::cell::{Cell, RefCell};
use std::rc::Rc;

use cesium_math::{
    binary_search, ClockRange, ClockStep, DeveloperError, GregorianDate, JulianDate,
};

use super::{
    clock_view_model::ClockViewModel,
    create_command::{create_command, create_command_with_can_execute, Command},
    toggle_button_view_model::ToggleButtonViewModel,
};

const MONTH_NAMES: [&str; 12] = [
    "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
];

pub const REALTIME_SHUTTLE_RING_ANGLE: f64 = 15.0;
pub const MAX_SHUTTLE_RING_ANGLE: f64 = 105.0;

pub static DEFAULT_TICKS: [f64; 30] = [
    0.001, 0.002, 0.005, 0.01, 0.02, 0.05, 0.1, 0.25, 0.5, 1.0, 2.0, 5.0, 10.0, 15.0, 30.0,
    60.0, 120.0, 300.0, 600.0, 900.0, 1800.0, 3600.0, 7200.0, 14400.0, 21600.0, 43200.0,
    86400.0, 172800.0, 345600.0, 604800.0,
];

pub type TimeFormatter = Rc<dyn Fn(JulianDate, &AnimationViewModel) -> String>;
pub type DateFormatter = Rc<dyn Fn(JulianDate, &AnimationViewModel) -> String>;

fn float_comparator(left: &f64, right: &f64) -> std::cmp::Ordering {
    left.partial_cmp(right).unwrap_or(std::cmp::Ordering::Equal)
}

fn get_typical_multiplier_index(multiplier: f64, ticks: &[f64]) -> usize {
    match binary_search(ticks, &multiplier, float_comparator) {
        Ok(idx) => idx,
        Err(idx) => idx,
    }
}

fn angle_to_multiplier(angle: f64, ticks: &[f64]) -> f64 {
    if angle.abs() <= REALTIME_SHUTTLE_RING_ANGLE {
        return angle / REALTIME_SHUTTLE_RING_ANGLE;
    }
    let minp = REALTIME_SHUTTLE_RING_ANGLE;
    let maxp = MAX_SHUTTLE_RING_ANGLE;
    let minv = 0.0;
    if angle > 0.0 {
        let maxv = ticks[ticks.len() - 1].ln();
        let scale = (maxv - minv) / (maxp - minp);
        (minv + scale * (angle - minp)).exp()
    } else {
        let maxv = (-ticks[0]).ln();
        let scale = (maxv - minv) / (maxp - minp);
        -(minv + scale * (angle.abs() - minp)).exp()
    }
}

fn multiplier_to_angle(multiplier: f64, ticks: &[f64], clock_view_model: &ClockViewModel) -> f64 {
    if clock_view_model.clock_step() == ClockStep::SystemClock {
        return REALTIME_SHUTTLE_RING_ANGLE;
    }
    if multiplier.abs() <= 1.0 {
        return multiplier * REALTIME_SHUTTLE_RING_ANGLE;
    }

    let fastest_multiplier = ticks[ticks.len() - 1];
    let bounded_multiplier = if multiplier > fastest_multiplier {
        fastest_multiplier
    } else if multiplier < -fastest_multiplier {
        -fastest_multiplier
    } else {
        multiplier
    };

    let minp = REALTIME_SHUTTLE_RING_ANGLE;
    let maxp = MAX_SHUTTLE_RING_ANGLE;
    let minv = 0.0;

    if bounded_multiplier > 0.0 {
        let maxv = fastest_multiplier.ln();
        let scale = (maxv - minv) / (maxp - minp);
        (bounded_multiplier.ln() - minv) / scale + minp
    } else {
        let maxv = (-ticks[0]).ln();
        let scale = (maxv - minv) / (maxp - minp);
        -((bounded_multiplier.abs().ln() - minv) / scale + minp)
    }
}

struct InnerState {
    clock_view_model: ClockViewModel,
    all_shuttle_ring_ticks: Vec<f64>,
    sorted_filtered_positive_ticks: Vec<f64>,
    shuttle_ring_dragging: Cell<bool>,
    snap_to_ticks: Cell<bool>,
    date_formatter: RefCell<DateFormatter>,
    time_formatter: RefCell<TimeFormatter>,
}

/// The view model for the `Animation` widget.
#[derive(Clone)]
pub struct AnimationViewModel {
    inner: Rc<InnerState>,
    pause_view_model: ToggleButtonViewModel,
    play_reverse_view_model: ToggleButtonViewModel,
    play_forward_view_model: ToggleButtonViewModel,
    play_realtime_view_model: ToggleButtonViewModel,
    slower_cmd: Command<(), ()>,
    faster_cmd: Command<(), ()>,
}

impl AnimationViewModel {
    /// Default date formatter mirroring CesiumJS `AnimationViewModel.defaultDateFormatter`.
    pub fn default_date_formatter(date: JulianDate, _view_model: &AnimationViewModel) -> String {
        let gregorian: GregorianDate = date.to_gregorian_date();
        let month_idx = (gregorian.month as usize).saturating_sub(1).min(11);
        format!(
            "{} {} {}",
            MONTH_NAMES[month_idx], gregorian.day, gregorian.year
        )
    }

    /// Default time formatter mirroring CesiumJS `AnimationViewModel.defaultTimeFormatter`.
    pub fn default_time_formatter(date: JulianDate, view_model: &AnimationViewModel) -> String {
        let gregorian: GregorianDate = date.to_gregorian_date();
        let hour = gregorian.hour;
        let minute = gregorian.minute;
        let second = gregorian.second;
        let millisecond = gregorian.millisecond.round() as u32;

        if view_model.clock_view_model().multiplier().abs() < 1.0 {
            format!("{hour:02}:{minute:02}:{second:02}.{millisecond:03}")
        } else {
            format!("{hour:02}:{minute:02}:{second:02} UTC")
        }
    }

    /// Creates a new `AnimationViewModel`.
    pub fn new(clock_view_model: ClockViewModel) -> Self {
        let mut sorted_pos = Vec::from(DEFAULT_TICKS);
        sorted_pos.sort_by(float_comparator);
        sorted_pos.dedup();

        let mut all_ticks = Vec::with_capacity(sorted_pos.len() * 2);
        for &t in sorted_pos.iter().rev() {
            if t != 0.0 {
                all_ticks.push(-t);
            }
        }
        all_ticks.extend(&sorted_pos);

        let inner = Rc::new(InnerState {
            clock_view_model: clock_view_model.clone(),
            all_shuttle_ring_ticks: all_ticks,
            sorted_filtered_positive_ticks: sorted_pos,
            shuttle_ring_dragging: Cell::new(false),
            snap_to_ticks: Cell::new(false),
            date_formatter: RefCell::new(Rc::new(Self::default_date_formatter)),
            time_formatter: RefCell::new(Rc::new(Self::default_time_formatter)),
        });

        // Commands
        let inner_pause = Rc::clone(&inner);
        let pause_cmd = create_command(move |()| {
            let cvm = &inner_pause.clock_view_model;
            if cvm.should_animate() {
                cvm.set_should_animate(false);
            } else if Self::check_can_animate(&inner_pause) {
                cvm.set_should_animate(true);
            }
        });

        let inner_pause_tb = Rc::clone(&inner);
        let pause_view_model = ToggleButtonViewModel::with_static_tooltip(
            pause_cmd,
            move || !Self::check_is_animating(&inner_pause_tb),
            "Pause",
        );

        let inner_reverse = Rc::clone(&inner);
        let play_reverse_cmd = create_command(move |()| {
            let cvm = &inner_reverse.clock_view_model;
            let mult = cvm.multiplier();
            if mult > 0.0 {
                cvm.set_multiplier(-mult);
            }
            cvm.set_should_animate(true);
        });

        let inner_rev_tb = Rc::clone(&inner);
        let play_reverse_view_model = ToggleButtonViewModel::with_static_tooltip(
            play_reverse_cmd,
            move || {
                Self::check_is_animating(&inner_rev_tb)
                    && inner_rev_tb.clock_view_model.multiplier() < 0.0
            },
            "Play Reverse",
        );

        let inner_fwd = Rc::clone(&inner);
        let play_forward_cmd = create_command(move |()| {
            let cvm = &inner_fwd.clock_view_model;
            let mult = cvm.multiplier();
            if mult < 0.0 {
                cvm.set_multiplier(-mult);
            }
            cvm.set_should_animate(true);
        });

        let inner_fwd_tb = Rc::clone(&inner);
        let play_forward_view_model = ToggleButtonViewModel::with_static_tooltip(
            play_forward_cmd,
            move || {
                Self::check_is_animating(&inner_fwd_tb)
                    && inner_fwd_tb.clock_view_model.multiplier() > 0.0
                    && inner_fwd_tb.clock_view_model.clock_step() != ClockStep::SystemClock
            },
            "Play Forward",
        );

        let inner_realtime = Rc::clone(&inner);
        let play_realtime_cmd = create_command_with_can_execute(
            move |()| {
                inner_realtime
                    .clock_view_model
                    .set_clock_step(ClockStep::SystemClock);
            },
            Self::check_is_system_time_available(&inner),
        );

        let inner_rt_tb = Rc::clone(&inner);
        let play_realtime_view_model = ToggleButtonViewModel::new(
            play_realtime_cmd,
            {
                let inner = Rc::clone(&inner_rt_tb);
                move || inner.clock_view_model.clock_step() == ClockStep::SystemClock
            },
            {
                let inner = Rc::clone(&inner_rt_tb);
                move || {
                    if Self::check_is_system_time_available(&inner) {
                        "Today (real-time)".to_string()
                    } else {
                        "Current time not in range".to_string()
                    }
                }
            },
        );

        let inner_slower = Rc::clone(&inner);
        let slower_cmd = create_command(move |()| {
            let cvm = &inner_slower.clock_view_model;
            let mult = cvm.multiplier();
            let idx = get_typical_multiplier_index(mult, &inner_slower.all_shuttle_ring_ticks);
            if idx > 0 {
                cvm.set_multiplier(inner_slower.all_shuttle_ring_ticks[idx - 1]);
            }
        });

        let inner_faster = Rc::clone(&inner);
        let faster_cmd = create_command(move |()| {
            let cvm = &inner_faster.clock_view_model;
            let mult = cvm.multiplier();
            let idx = get_typical_multiplier_index(mult, &inner_faster.all_shuttle_ring_ticks);
            if idx + 1 < inner_faster.all_shuttle_ring_ticks.len() {
                cvm.set_multiplier(inner_faster.all_shuttle_ring_ticks[idx + 1]);
            }
        });

        Self {
            inner,
            pause_view_model,
            play_reverse_view_model,
            play_forward_view_model,
            play_realtime_view_model,
            slower_cmd,
            faster_cmd,
        }
    }

    fn check_can_animate(inner: &InnerState) -> bool {
        let cvm = &inner.clock_view_model;
        let clock_range = cvm.clock_range();

        if inner.shuttle_ring_dragging.get() || clock_range == ClockRange::Unbounded {
            return true;
        }

        let multiplier = cvm.multiplier();
        let current_time = cvm.current_time();
        let start_time = cvm.start_time();

        let result = if clock_range == ClockRange::LoopStop {
            current_time > start_time || (current_time == start_time && multiplier > 0.0)
        } else {
            let stop_time = cvm.stop_time();
            (current_time > start_time && current_time < stop_time)
                || (current_time == start_time && multiplier > 0.0)
                || (current_time == stop_time && multiplier < 0.0)
        };

        if !result {
            cvm.set_should_animate(false);
        }
        result
    }

    fn check_is_system_time_available(inner: &InnerState) -> bool {
        let cvm = &inner.clock_view_model;
        if cvm.clock_range() == ClockRange::Unbounded {
            return true;
        }
        let system_time = cvm.system_time();
        system_time >= cvm.start_time() && system_time <= cvm.stop_time()
    }

    fn check_is_animating(inner: &InnerState) -> bool {
        inner.clock_view_model.should_animate()
            && (Self::check_can_animate(inner) || inner.shuttle_ring_dragging.get())
    }

    /// Gets the underlying clock view model.
    pub fn clock_view_model(&self) -> &ClockViewModel {
        &self.inner.clock_view_model
    }

    /// Gets whether the shuttle ring is currently being dragged.
    pub fn shuttle_ring_dragging(&self) -> bool {
        self.inner.shuttle_ring_dragging.get()
    }

    /// Sets whether the shuttle ring is currently being dragged.
    pub fn set_shuttle_ring_dragging(&self, dragging: bool) {
        self.inner.shuttle_ring_dragging.set(dragging);
    }

    /// Gets whether dragging the shuttle ring snaps to tick values.
    pub fn snap_to_ticks(&self) -> bool {
        self.inner.snap_to_ticks.get()
    }

    /// Sets whether dragging the shuttle ring snaps to tick values.
    pub fn set_snap_to_ticks(&self, snap: bool) {
        self.inner.snap_to_ticks.set(snap);
    }

    /// Gets formatted time string label.
    pub fn time_label(&self) -> String {
        let formatter = self.inner.time_formatter.borrow().clone();
        formatter(self.inner.clock_view_model.current_time(), self)
    }

    /// Gets formatted date string label.
    pub fn date_label(&self) -> String {
        let formatter = self.inner.date_formatter.borrow().clone();
        formatter(self.inner.clock_view_model.current_time(), self)
    }

    /// Sets a custom time formatter.
    pub fn set_time_formatter<F>(&self, formatter: F)
    where
        F: Fn(JulianDate, &AnimationViewModel) -> String + 'static,
    {
        *self.inner.time_formatter.borrow_mut() = Rc::new(formatter);
    }

    /// Sets a custom date formatter.
    pub fn set_date_formatter<F>(&self, formatter: F)
    where
        F: Fn(JulianDate, &AnimationViewModel) -> String + 'static,
    {
        *self.inner.date_formatter.borrow_mut() = Rc::new(formatter);
    }

    /// Gets formatted multiplier label.
    pub fn multiplier_label(&self) -> String {
        let cvm = &self.inner.clock_view_model;
        if cvm.clock_step() == ClockStep::SystemClock {
            return "Today".to_string();
        }

        let multiplier = cvm.multiplier();
        if multiplier.fract() == 0.0 {
            format!("{multiplier:.0}x")
        } else {
            let s = format!("{multiplier:.3}");
            let trimmed = s.trim_end_matches('0').trim_end_matches('.');
            format!("{trimmed}x")
        }
    }

    /// Gets current shuttle ring angle.
    pub fn shuttle_ring_angle(&self) -> f64 {
        multiplier_to_angle(
            self.inner.clock_view_model.multiplier(),
            &self.inner.all_shuttle_ring_ticks,
            &self.inner.clock_view_model,
        )
    }

    /// Sets shuttle ring angle and updates multiplier accordingly.
    pub fn set_shuttle_ring_angle(&self, angle: f64) {
        let angle = angle.clamp(-MAX_SHUTTLE_RING_ANGLE, MAX_SHUTTLE_RING_ANGLE);
        let ticks = &self.inner.all_shuttle_ring_ticks;
        let cvm = &self.inner.clock_view_model;
        cvm.set_clock_step(ClockStep::SystemClockMultiplier);

        if angle.abs() == MAX_SHUTTLE_RING_ANGLE {
            cvm.set_multiplier(if angle > 0.0 {
                ticks[ticks.len() - 1]
            } else {
                ticks[0]
            });
            return;
        }

        let mut multiplier = angle_to_multiplier(angle, ticks);
        if self.snap_to_ticks() {
            let idx = get_typical_multiplier_index(multiplier, ticks);
            multiplier = ticks[idx.min(ticks.len() - 1)];
        } else if multiplier != 0.0 {
            let pos = multiplier.abs();
            if pos > 100.0 {
                let num_digits = (pos.round() as u64).to_string().len().saturating_sub(2);
                let divisor = 10f64.powi(num_digits as i32);
                multiplier = (multiplier / divisor).round() * divisor;
            } else if pos > REALTIME_SHUTTLE_RING_ANGLE {
                multiplier = multiplier.round();
            } else if pos > 1.0 {
                multiplier = (multiplier * 10.0).round() / 10.0;
            } else if pos > 0.0 {
                multiplier = (multiplier * 100.0).round() / 100.0;
            }
        }
        cvm.set_multiplier(multiplier);
    }

    /// Gets a copy of positive shuttle ring ticks.
    pub fn get_shuttle_ring_ticks(&self) -> Vec<f64> {
        self.inner.sorted_filtered_positive_ticks.clone()
    }

    /// Sets positive shuttle ring ticks.
    pub fn set_shuttle_ring_ticks(&self, positive_ticks: &[f64]) -> Result<(), DeveloperError> {
        if positive_ticks.is_empty() {
            return Err(DeveloperError::new("positiveTicks cannot be empty."));
        }
        let mut sorted = positive_ticks.to_vec();
        sorted.sort_by(float_comparator);
        sorted.dedup();

        let mut all_ticks = Vec::with_capacity(sorted.len() * 2);
        for &t in sorted.iter().rev() {
            if t != 0.0 {
                all_ticks.push(-t);
            }
        }
        all_ticks.extend(&sorted);

        // Update inner
        let inner_ptr = Rc::as_ptr(&self.inner) as *mut InnerState;
        unsafe {
            (*inner_ptr).sorted_filtered_positive_ticks = sorted;
            (*inner_ptr).all_shuttle_ring_ticks = all_ticks;
        }
        Ok(())
    }

    /// Gets pause toggle button view model.
    pub fn pause_view_model(&self) -> &ToggleButtonViewModel {
        &self.pause_view_model
    }

    /// Gets reverse toggle button view model.
    pub fn play_reverse_view_model(&self) -> &ToggleButtonViewModel {
        &self.play_reverse_view_model
    }

    /// Gets forward toggle button view model.
    pub fn play_forward_view_model(&self) -> &ToggleButtonViewModel {
        &self.play_forward_view_model
    }

    /// Gets realtime toggle button view model.
    pub fn play_realtime_view_model(&self) -> &ToggleButtonViewModel {
        &self.play_realtime_view_model
    }

    /// Gets slower command.
    pub fn slower(&self) -> &Command<(), ()> {
        &self.slower_cmd
    }

    /// Gets faster command.
    pub fn faster(&self) -> &Command<(), ()> {
        &self.faster_cmd
    }
}
