use std::cell::RefCell;
use std::rc::Rc;

use cesium_math::{Clock, ClockRange, ClockStep, JulianDate, ListenerId};

struct ClockViewModelState {
    clock: Rc<RefCell<Clock>>,
    system_time: JulianDate,
    start_time: JulianDate,
    stop_time: JulianDate,
    current_time: JulianDate,
    multiplier: f64,
    clock_step: ClockStep,
    clock_range: ClockRange,
    can_animate: bool,
    should_animate: bool,
    listener_id: Option<ListenerId>,
    is_destroyed: bool,
}

impl ClockViewModelState {
    fn sync_from_clock_ref(&mut self, clock: &Clock) {
        self.system_time = JulianDate::now();
        self.start_time = clock.start_time();
        self.stop_time = clock.stop_time();
        self.current_time = clock.current_time();
        self.multiplier = clock.multiplier();
        self.clock_step = clock.clock_step();
        self.clock_range = clock.clock_range();
        self.can_animate = clock.can_animate;
        self.should_animate = clock.should_animate();
    }
}

/// A view model which exposes a `Clock` for user interfaces and keeps observable state in sync.
#[derive(Clone)]
pub struct ClockViewModel {
    inner: Rc<RefCell<ClockViewModelState>>,
}

impl Default for ClockViewModel {
    fn default() -> Self {
        Self::new(None)
    }
}

impl ClockViewModel {
    /// Creates a new `ClockViewModel` wrapping either the provided clock or a default Clock.
    pub fn new(clock: Option<Rc<RefCell<Clock>>>) -> Self {
        let clock = clock.unwrap_or_else(|| Rc::new(RefCell::new(Clock::default())));

        let (
            system_time,
            start_time,
            stop_time,
            current_time,
            multiplier,
            clock_step,
            clock_range,
            can_animate,
            should_animate,
        ) = {
            let c = clock.borrow();
            (
                JulianDate::now(),
                c.start_time(),
                c.stop_time(),
                c.current_time(),
                c.multiplier(),
                c.clock_step(),
                c.clock_range(),
                c.can_animate,
                c.should_animate(),
            )
        };

        let state = Rc::new(RefCell::new(ClockViewModelState {
            clock: Rc::clone(&clock),
            system_time,
            start_time,
            stop_time,
            current_time,
            multiplier,
            clock_step,
            clock_range,
            can_animate,
            should_animate,
            listener_id: None,
            is_destroyed: false,
        }));

        let weak = Rc::downgrade(&state);
        let id = clock.borrow().on_tick.add_listener(move |c: &Clock| {
            if let Some(strong) = weak.upgrade() {
                let mut inner = strong.borrow_mut();
                if !inner.is_destroyed {
                    inner.sync_from_clock_ref(c);
                }
            }
        });

        state.borrow_mut().listener_id = Some(id);

        Self { inner: state }
    }

    /// Gets the underlying clock reference.
    pub fn clock(&self) -> Rc<RefCell<Clock>> {
        Rc::clone(&self.inner.borrow().clock)
    }

    /// Gets the current system time.
    pub fn system_time(&self) -> JulianDate {
        self.inner.borrow().system_time
    }

    /// Gets the clock start time.
    pub fn start_time(&self) -> JulianDate {
        self.inner.borrow().start_time
    }

    /// Sets the clock start time and synchronizes the view model.
    pub fn set_start_time(&self, value: JulianDate) {
        self.inner.borrow().clock.borrow_mut().set_start_time(value);
        self.synchronize();
    }

    /// Gets the clock stop time.
    pub fn stop_time(&self) -> JulianDate {
        self.inner.borrow().stop_time
    }

    /// Sets the clock stop time and synchronizes the view model.
    pub fn set_stop_time(&self, value: JulianDate) {
        self.inner.borrow().clock.borrow_mut().set_stop_time(value);
        self.synchronize();
    }

    /// Gets the current time.
    pub fn current_time(&self) -> JulianDate {
        self.inner.borrow().current_time
    }

    /// Sets the current time and synchronizes the view model.
    pub fn set_current_time(&self, value: JulianDate) {
        self.inner.borrow().clock.borrow_mut().set_current_time(value);
        self.synchronize();
    }

    /// Gets the multiplier.
    pub fn multiplier(&self) -> f64 {
        self.inner.borrow().multiplier
    }

    /// Sets the multiplier and synchronizes the view model.
    pub fn set_multiplier(&self, value: f64) {
        self.inner.borrow().clock.borrow_mut().set_multiplier(value);
        self.synchronize();
    }

    /// Gets the clock step setting.
    pub fn clock_step(&self) -> ClockStep {
        self.inner.borrow().clock_step
    }

    /// Sets the clock step setting and synchronizes the view model.
    pub fn set_clock_step(&self, value: ClockStep) {
        self.inner.borrow().clock.borrow_mut().set_clock_step(value);
        self.synchronize();
    }

    /// Gets the clock range setting.
    pub fn clock_range(&self) -> ClockRange {
        self.inner.borrow().clock_range
    }

    /// Sets the clock range setting and synchronizes the view model.
    pub fn set_clock_range(&self, value: ClockRange) {
        self.inner.borrow().clock.borrow_mut().set_clock_range(value);
        self.synchronize();
    }

    /// Gets whether the clock can animate.
    pub fn can_animate(&self) -> bool {
        self.inner.borrow().can_animate
    }

    /// Sets whether the clock can animate and synchronizes the view model.
    pub fn set_can_animate(&self, value: bool) {
        self.inner.borrow().clock.borrow_mut().can_animate = value;
        self.synchronize();
    }

    /// Gets whether the clock should animate.
    pub fn should_animate(&self) -> bool {
        self.inner.borrow().should_animate
    }

    /// Sets whether the clock should animate and synchronizes the view model.
    pub fn set_should_animate(&self, value: bool) {
        self.inner
            .borrow()
            .clock
            .borrow_mut()
            .set_should_animate(value);
        self.synchronize();
    }

    /// Updates the view model with the contents of the underlying clock.
    pub fn synchronize(&self) {
        let mut inner = self.inner.borrow_mut();
        if inner.is_destroyed {
            return;
        }
        let c_ref = inner.clock.borrow().clone();
        inner.sync_from_clock_ref(&c_ref);
    }

    /// Returns whether the view model has been destroyed.
    pub fn is_destroyed(&self) -> bool {
        self.inner.borrow().is_destroyed
    }

    /// Destroys the view model and unregisters internal listeners.
    pub fn destroy(&self) {
        let mut inner = self.inner.borrow_mut();
        if inner.is_destroyed {
            return;
        }
        if let Some(id) = inner.listener_id.take() {
            inner.clock.borrow().on_tick.remove_listener(id);
        }
        inner.is_destroyed = true;
    }
}
