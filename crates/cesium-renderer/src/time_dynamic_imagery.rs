//! TimeDynamicImagery matching CesiumJS `Scene/TimeDynamicImagery.js` (Apache-2.0).

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::{Rc, Weak};

use cesium_math::{
    Clock, DeveloperError, ListenerId, Request, RequestType, TimeInterval,
    TimeIntervalCollection,
};

/// A request entry recorded for an imagery tile in an interval.
#[derive(Clone)]
pub struct TileRequestEntry {
    pub key: String,
    pub priority_function: Option<Rc<dyn Fn() -> f64>>,
}

impl std::fmt::Debug for TileRequestEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TileRequestEntry")
            .field("key", &self.key)
            .field("has_priority_fn", &self.priority_function.is_some())
            .finish()
    }
}

/// Cached tile in a specific interval.
#[derive(Clone, Debug)]
pub struct CachedTile<TImage: Clone + 'static> {
    pub image: TImage,
    pub request: Request,
}

/// Callback signature for requesting imagery tile in a time interval.
pub type RequestImageFunction<T, TImage> =
    Rc<dyn Fn(u32, u32, u32, &Request, &TimeInterval<T>) -> Option<TImage>>;

/// Callback signature for reloading imagery when interval changes.
pub type ReloadFunction = Rc<dyn Fn()>;

/// Options for initializing `TimeDynamicImagery`.
pub struct TimeDynamicImageryOptions<T: Clone + 'static = (), TImage: Clone + 'static = ()> {
    pub clock: Option<Rc<RefCell<Clock>>>,
    pub times: Option<TimeIntervalCollection<T>>,
    pub request_image_function: Option<RequestImageFunction<T, TImage>>,
    pub reload_function: Option<ReloadFunction>,
}

impl<T: Clone + 'static, TImage: Clone + 'static> Default for TimeDynamicImageryOptions<T, TImage> {
    fn default() -> Self {
        Self {
            clock: None,
            times: None,
            request_image_function: None,
            reload_function: None,
        }
    }
}

pub struct TimeDynamicImageryInner<T: Clone + 'static = (), TImage: Clone + 'static = ()> {
    pub clock: Rc<RefCell<Clock>>,
    pub times: TimeIntervalCollection<T>,
    pub request_image_function: RequestImageFunction<T, TImage>,
    pub reload_function: ReloadFunction,
    pub current_interval_index: Option<usize>,
    pub tile_cache: HashMap<usize, HashMap<String, CachedTile<TImage>>>,
    pub tiles_requested_for_interval: Vec<TileRequestEntry>,
}

impl<T: Clone + 'static, TImage: Clone + 'static> TimeDynamicImageryInner<T, TImage> {
    pub fn current_interval(&self) -> Option<TimeInterval<T>> {
        self.current_interval_index.and_then(|idx| self.times.get(idx).cloned())
    }

    pub fn get_from_cache(
        &mut self,
        x: u32,
        y: u32,
        level: u32,
        request: Option<&mut Request>,
    ) -> Option<TImage> {
        let key = format!("{x}-{y}-{level}");
        let idx = self.current_interval_index?;
        let interval_cache = self.tile_cache.get_mut(&idx)?;
        let cached = interval_cache.remove(&key)?;
        if let Some(req) = request {
            req.state = cached.request.state;
        }
        Some(cached.image)
    }

    pub fn check_approaching_interval(
        &mut self,
        x: u32,
        y: u32,
        level: u32,
        request: &Request,
    ) {
        let key = format!("{x}-{y}-{level}");
        let approaching_interval = {
            let clock_borrow = self.clock.borrow();
            get_approaching_interval(&clock_borrow, &self.times)
        };

        let mut added = false;
        if let Some(ref interval) = approaching_interval {
            added = self.add_to_cache(
                &key,
                x,
                y,
                level,
                interval,
                request.priority_function.clone(),
            );
        }

        if !added {
            self.tiles_requested_for_interval.push(TileRequestEntry {
                key,
                priority_function: request.priority_function.clone(),
            });
        }

        if self.tiles_requested_for_interval.len() >= 512 {
            self.tiles_requested_for_interval.drain(0..256);
        }
    }

    fn add_to_cache(
        &mut self,
        key: &str,
        x: u32,
        y: u32,
        level: u32,
        interval: &TimeInterval<T>,
        priority_function: Option<Rc<dyn Fn() -> f64>>,
    ) -> bool {
        let index = match self.times.index_of(&interval.start) {
            Ok(idx) => idx,
            Err(_) => return false,
        };

        if let Some(cache) = self.tile_cache.get(&index) {
            if cache.contains_key(key) {
                return true;
            }
        }

        let request = Request {
            url: None,
            priority: 0.0,
            throttle: false,
            throttle_by_server: true,
            request_type: RequestType::Imagery,
            server_key: None,
            state: cesium_math::RequestState::Received,
            cancelled: false,
            priority_function,
            cancel_function: None,
        };

        if let Some(image) = (self.request_image_function)(x, y, level, &request, interval) {
            let interval_cache = self.tile_cache.entry(index).or_default();
            interval_cache.insert(
                key.to_string(),
                CachedTile {
                    image,
                    request,
                },
            );
            true
        } else {
            false
        }
    }

    pub fn clock_on_tick(&mut self, clock: &Clock) {
        let time = clock.current_time();
        let index = self.times.index_of(&time).ok();

        if index != self.current_interval_index {
            // Cancel all outstanding requests and clear out caches not from current time interval
            if let Some(old_idx) = self.current_interval_index {
                if let Some(mut current_cache) = self.tile_cache.remove(&old_idx) {
                    for (_k, mut v) in current_cache.drain() {
                        v.request.cancel();
                    }
                }
            }
            self.tiles_requested_for_interval.clear();
            self.current_interval_index = index;
            (self.reload_function)();
            return;
        }

        let approaching_interval = get_approaching_interval(clock, &self.times);
        if let Some(interval) = approaching_interval {
            let mut success = true;
            while success {
                if self.tiles_requested_for_interval.is_empty() {
                    break;
                }
                let tile = self.tiles_requested_for_interval.pop().unwrap();
                if let Some((x, y, level)) = parse_key(&tile.key) {
                    success = self.add_to_cache(
                        &tile.key,
                        x,
                        y,
                        level,
                        &interval,
                        tile.priority_function.clone(),
                    );
                    if !success {
                        self.tiles_requested_for_interval.push(tile);
                    }
                }
            }
        }
    }
}

/// Provides functionality for ImageryProviders that have time dynamic imagery.
pub struct TimeDynamicImagery<T: Clone + 'static = (), TImage: Clone + 'static = ()> {
    inner: Rc<RefCell<TimeDynamicImageryInner<T, TImage>>>,
    listener_id: Option<ListenerId>,
}

impl<T: Clone + 'static, TImage: Clone + 'static> TimeDynamicImagery<T, TImage> {
    /// Constructs a `TimeDynamicImagery` instance.
    pub fn new(options: TimeDynamicImageryOptions<T, TImage>) -> Result<Self, DeveloperError> {
        let clock = options
            .clock
            .ok_or_else(|| DeveloperError::new("options.clock is required."))?;
        let times = options
            .times
            .ok_or_else(|| DeveloperError::new("options.times is required."))?;
        let request_image_function = options
            .request_image_function
            .ok_or_else(|| DeveloperError::new("options.requestImageFunction is required."))?;
        let reload_function = options
            .reload_function
            .ok_or_else(|| DeveloperError::new("options.reloadFunction is required."))?;

        let inner = Rc::new(RefCell::new(TimeDynamicImageryInner {
            clock: clock.clone(),
            times,
            request_image_function,
            reload_function,
            current_interval_index: None,
            tile_cache: HashMap::new(),
            tiles_requested_for_interval: Vec::new(),
        }));

        let weak_inner: Weak<RefCell<TimeDynamicImageryInner<T, TImage>>> = Rc::downgrade(&inner);
        let listener_id = clock.borrow_mut().on_tick.add_listener(move |c| {
            if let Some(inner_arc) = weak_inner.upgrade() {
                inner_arc.borrow_mut().clock_on_tick(c);
            }
        });

        // Initial tick call to set the current interval and trigger initial reload
        let clock_borrow = clock.borrow();
        inner.borrow_mut().clock_on_tick(&clock_borrow);

        Ok(Self {
            inner,
            listener_id: Some(listener_id),
        })
    }

    pub fn clock(&self) -> Rc<RefCell<Clock>> {
        self.inner.borrow().clock.clone()
    }

    pub fn set_clock(&mut self, new_clock: Rc<RefCell<Clock>>) {
        let old_clock = self.inner.borrow().clock.clone();
        if let Some(id) = self.listener_id.take() {
            old_clock.borrow_mut().on_tick.remove_listener(id);
        }

        self.inner.borrow_mut().clock = new_clock.clone();

        let weak_inner = Rc::downgrade(&self.inner);
        let listener_id = new_clock.borrow_mut().on_tick.add_listener(move |c| {
            if let Some(inner_arc) = weak_inner.upgrade() {
                inner_arc.borrow_mut().clock_on_tick(c);
            }
        });
        self.listener_id = Some(listener_id);

        let clock_borrow = new_clock.borrow();
        self.inner.borrow_mut().clock_on_tick(&clock_borrow);
        (self.inner.borrow().reload_function)();
    }

    pub fn times(&self) -> TimeIntervalCollection<T> {
        self.inner.borrow().times.clone()
    }

    pub fn set_times(&mut self, times: TimeIntervalCollection<T>) {
        self.inner.borrow_mut().times = times;
        let clock = self.inner.borrow().clock.clone();
        let clock_borrow = clock.borrow();
        self.inner.borrow_mut().clock_on_tick(&clock_borrow);
        (self.inner.borrow().reload_function)();
    }

    pub fn current_interval_index(&self) -> Option<usize> {
        self.inner.borrow().current_interval_index
    }

    pub fn current_interval(&self) -> Option<TimeInterval<T>> {
        self.inner.borrow().current_interval()
    }

    pub fn tile_cache(&self) -> HashMap<usize, HashMap<String, CachedTile<TImage>>> {
        self.inner.borrow().tile_cache.clone()
    }

    pub fn tiles_requested_for_interval(&self) -> Vec<TileRequestEntry> {
        self.inner.borrow().tiles_requested_for_interval.clone()
    }

    pub fn check_approaching_interval(
        &mut self,
        x: u32,
        y: u32,
        level: u32,
        request: &Request,
    ) {
        self.inner
            .borrow_mut()
            .check_approaching_interval(x, y, level, request);
    }

    pub fn get_from_cache(
        &mut self,
        x: u32,
        y: u32,
        level: u32,
        request: Option<&mut Request>,
    ) -> Option<TImage> {
        self.inner
            .borrow_mut()
            .get_from_cache(x, y, level, request)
    }
}

impl<T: Clone + 'static, TImage: Clone + 'static> Drop for TimeDynamicImagery<T, TImage> {
    fn drop(&mut self) {
        if let Some(id) = self.listener_id.take() {
            self.inner.borrow().clock.borrow_mut().on_tick.remove_listener(id);
        }
    }
}

fn parse_key(key: &str) -> Option<(u32, u32, u32)> {
    let mut parts = key.split('-');
    let x = parts.next()?.parse().ok()?;
    let y = parts.next()?.parse().ok()?;
    let level = parts.next()?.parse().ok()?;
    if parts.next().is_none() {
        Some((x, y, level))
    } else {
        None
    }
}

fn get_approaching_interval<T: Clone + 'static>(
    clock: &Clock,
    times: &TimeIntervalCollection<T>,
) -> Option<TimeInterval<T>> {
    let time = clock.current_time();
    let is_animating = clock.can_animate && clock.should_animate();
    let multiplier = clock.multiplier();

    if !is_animating && multiplier != 0.0 {
        return None;
    }

    let mut index = match times.index_of(&time) {
        Ok(idx) => idx as isize,
        Err(_) => return None,
    };

    let interval = times.get(index as usize)?;

    let seconds = if multiplier > 0.0 {
        // Animating forward
        let sec = interval.stop.seconds_difference(&time);
        index += 1;
        sec / multiplier
    } else if multiplier < 0.0 {
        // Animating backward
        let sec = interval.start.seconds_difference(&time);
        index -= 1;
        sec / multiplier
    } else {
        return None;
    };

    if index >= 0 && (index as usize) < times.len() && seconds <= 5.0 {
        times.get(index as usize).cloned()
    } else {
        None
    }
}
