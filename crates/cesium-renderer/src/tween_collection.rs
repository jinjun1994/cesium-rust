//! TweenCollection matching CesiumJS `Scene/TweenCollection.js` (Apache-2.0).

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::{Rc, Weak};
use std::sync::atomic::{AtomicU64, Ordering};

use cesium_math::{Color, DeveloperError, EasingFunction};

static NEXT_TWEEN_ID: AtomicU64 = AtomicU64::new(1);

/// Callback invoked on each tween update.
pub type TweenUpdateCallback = Box<dyn FnMut(&HashMap<String, f64>)>;
/// Callback invoked when a tween completes.
pub type TweenCompleteCallback = Box<dyn FnMut()>;
/// Callback invoked when a tween is cancelled.
pub type TweenCancelCallback = Box<dyn FnMut()>;

/// Options for creating a `Tween`.
pub struct TweenOptions {
    pub start_object: HashMap<String, f64>,
    pub stop_object: HashMap<String, f64>,
    pub duration: f64,
    pub delay: Option<f64>,
    pub easing_function: Option<EasingFunction>,
    pub update: Option<TweenUpdateCallback>,
    pub complete: Option<TweenCompleteCallback>,
    pub cancel: Option<TweenCancelCallback>,
    pub repeat: Option<f64>,
}

/// A tween animation interpolating numeric properties over time.
pub struct Tween {
    pub id: u64,
    pub start_object: HashMap<String, f64>,
    pub stop_object: HashMap<String, f64>,
    pub duration: f64,
    pub delay: f64,
    pub easing_function: EasingFunction,

    pub update: Option<TweenUpdateCallback>,
    pub complete: Option<TweenCompleteCallback>,
    pub cancel: Option<TweenCancelCallback>,

    start_time: Option<f64>,
    needs_start: bool,
    repeat: f64,
    collection: Option<Weak<RefCell<TweenCollectionInner>>>,
}

impl Tween {
    /// Cancels this tween if it is still active in its collection.
    pub fn cancel_tween(&mut self) {
        if let Some(ref mut cancel) = self.cancel {
            cancel();
        }
        self.cancel = None;
        if let Some(weak) = &self.collection {
            if let Some(inner) = weak.upgrade() {
                let id = self.id;
                inner.borrow_mut().tweens.retain(|t| {
                    t.try_borrow().map(|b| b.id != id).unwrap_or(false)
                });
            }
        }
    }
}

/// Options for `TweenCollection::add_property`.
pub struct TweenPropertyOptions {
    pub object: Rc<RefCell<HashMap<String, f64>>>,
    pub property: String,
    pub start_value: f64,
    pub stop_value: f64,
    pub duration: Option<f64>,
    pub delay: Option<f64>,
    pub easing_function: Option<EasingFunction>,
    pub update: Option<Box<dyn FnMut(f64)>>,
    pub complete: Option<TweenCompleteCallback>,
    pub cancel: Option<TweenCancelCallback>,
    pub repeat: Option<f64>,
}

/// A material with uniforms for color and offset animations.
#[derive(Debug, Clone, Default)]
pub struct TweenMaterial {
    pub color_uniforms: HashMap<String, Color>,
    pub offset: Option<f64>,
}

/// Options for `TweenCollection::add_alpha`.
pub struct TweenAlphaOptions {
    pub material: Rc<RefCell<TweenMaterial>>,
    pub start_value: Option<f64>,
    pub stop_value: Option<f64>,
    pub duration: Option<f64>,
    pub delay: Option<f64>,
    pub easing_function: Option<EasingFunction>,
    pub update: Option<Box<dyn FnMut(f64)>>,
    pub complete: Option<TweenCompleteCallback>,
    pub cancel: Option<TweenCancelCallback>,
}

/// Options for `TweenCollection::add_offset_increment`.
pub struct TweenOffsetOptions {
    pub material: Rc<RefCell<TweenMaterial>>,
    pub duration: Option<f64>,
    pub delay: Option<f64>,
    pub easing_function: Option<EasingFunction>,
    pub update: Option<Box<dyn FnMut(f64)>>,
    pub cancel: Option<TweenCancelCallback>,
}

#[derive(Default)]
struct TweenCollectionInner {
    tweens: Vec<Rc<RefCell<Tween>>>,
}

impl TweenCollectionInner {
    fn remove_by_id(&mut self, id: u64) -> bool {
        if let Some(pos) = self.tweens.iter().position(|t| {
            t.try_borrow().map(|b| b.id == id).unwrap_or(false)
        }) {
            let tween_rc = self.tweens.remove(pos);
            if let Ok(mut tween) = tween_rc.try_borrow_mut() {
                if let Some(ref mut cancel) = tween.cancel {
                    cancel();
                }
            }
            return true;
        }
        false
    }
}

/// A collection of tweens for animating properties.
#[derive(Clone, Default)]
pub struct TweenCollection {
    inner: Rc<RefCell<TweenCollectionInner>>,
}

impl TweenCollection {
    /// Creates a new empty `TweenCollection`.
    pub fn new() -> Self {
        Self {
            inner: Rc::new(RefCell::new(TweenCollectionInner::default())),
        }
    }

    /// Number of active tweens in the collection.
    pub fn len(&self) -> usize {
        self.inner.borrow().tweens.len()
    }

    /// Returns true if there are no tweens.
    pub fn is_empty(&self) -> bool {
        self.inner.borrow().tweens.is_empty()
    }

    /// Adds a tween to the collection.
    pub fn add(&self, mut options: TweenOptions) -> Result<Rc<RefCell<Tween>>, DeveloperError> {
        if options.start_object.is_empty() || options.stop_object.is_empty() {
            return Err(DeveloperError::new(
                "options.startObject and options.stopObject are required.",
            ));
        }

        if options.duration < 0.0 {
            return Err(DeveloperError::new(
                "options.duration is required and must be positive.",
            ));
        }

        let delay = options.delay.unwrap_or(0.0);
        let easing = options.easing_function.unwrap_or(EasingFunction::LinearNone);
        let repeat = options.repeat.unwrap_or(0.0);

        if options.duration == 0.0 {
            if let Some(ref mut complete) = options.complete {
                complete();
            }
            let tween = Tween {
                id: NEXT_TWEEN_ID.fetch_add(1, Ordering::SeqCst),
                start_object: options.start_object,
                stop_object: options.stop_object,
                duration: 0.0,
                delay,
                easing_function: easing,
                update: options.update,
                complete: options.complete,
                cancel: options.cancel,
                start_time: None,
                needs_start: false,
                repeat,
                collection: None,
            };
            return Ok(Rc::new(RefCell::new(tween)));
        }

        let id = NEXT_TWEEN_ID.fetch_add(1, Ordering::SeqCst);
        let tween = Rc::new(RefCell::new(Tween {
            id,
            start_object: options.start_object,
            stop_object: options.stop_object,
            duration: options.duration,
            delay,
            easing_function: easing,
            update: options.update,
            complete: options.complete,
            cancel: options.cancel,
            start_time: None,
            needs_start: true,
            repeat,
            collection: Some(Rc::downgrade(&self.inner)),
        }));

        self.inner.borrow_mut().tweens.push(tween.clone());
        Ok(tween)
    }

    /// Adds a tween for animating a single scalar property on an object.
    pub fn add_property(
        &self,
        options: TweenPropertyOptions,
    ) -> Result<Rc<RefCell<Tween>>, DeveloperError> {
        let prop = options.property;
        if !options.object.borrow().contains_key(&prop) {
            return Err(DeveloperError::new(
                "options.object must have the specified property.",
            ));
        }

        let mut start_object = HashMap::new();
        start_object.insert(prop.clone(), options.start_value);

        let mut stop_object = HashMap::new();
        stop_object.insert(prop.clone(), options.stop_value);

        let object_clone = options.object.clone();
        let prop_name = prop.clone();
        let mut user_update = options.update;

        let update_cb: TweenUpdateCallback = Box::new(move |val: &HashMap<String, f64>| {
            if let Some(&v) = val.get(&prop_name) {
                object_clone.borrow_mut().insert(prop_name.clone(), v);
                if let Some(ref mut u) = user_update {
                    u(v);
                }
            }
        });

        self.add(TweenOptions {
            start_object,
            stop_object,
            duration: options.duration.unwrap_or(3.0),
            delay: options.delay,
            easing_function: options.easing_function,
            update: Some(update_cb),
            complete: options.complete,
            cancel: options.cancel,
            repeat: options.repeat,
        })
    }

    /// Animates the alpha component of all color uniforms in a material.
    pub fn add_alpha(&self, options: TweenAlphaOptions) -> Result<Rc<RefCell<Tween>>, DeveloperError> {
        if options.material.borrow().color_uniforms.is_empty() {
            return Err(DeveloperError::new(
                "material has no properties with alpha components.",
            ));
        }

        let start_val = options.start_value.unwrap_or(0.0);
        let stop_val = options.stop_value.unwrap_or(1.0);

        let mut start_object = HashMap::new();
        start_object.insert("alpha".to_string(), start_val);

        let mut stop_object = HashMap::new();
        stop_object.insert("alpha".to_string(), stop_val);

        let mat_rc = options.material.clone();
        let mut user_update = options.update;

        let update_cb: TweenUpdateCallback = Box::new(move |val: &HashMap<String, f64>| {
            if let Some(&alpha) = val.get("alpha") {
                for color in mat_rc.borrow_mut().color_uniforms.values_mut() {
                    color.alpha = alpha as f32;
                }
                if let Some(ref mut u) = user_update {
                    u(alpha);
                }
            }
        });

        self.add(TweenOptions {
            start_object,
            stop_object,
            duration: options.duration.unwrap_or(3.0),
            delay: options.delay,
            easing_function: options.easing_function,
            update: Some(update_cb),
            complete: options.complete,
            cancel: options.cancel,
            repeat: None,
        })
    }

    /// Animates offset uniform incrementing endlessly.
    pub fn add_offset_increment(
        &self,
        options: TweenOffsetOptions,
    ) -> Result<Rc<RefCell<Tween>>, DeveloperError> {
        let initial_offset = options
            .material
            .borrow()
            .offset
            .ok_or_else(|| DeveloperError::new("material.uniforms must have an offset property."))?;

        let mut start_object = HashMap::new();
        start_object.insert("offset".to_string(), initial_offset);

        let mut stop_object = HashMap::new();
        stop_object.insert("offset".to_string(), initial_offset + 1.0);

        let mat_rc = options.material.clone();
        let mut user_update = options.update;

        let update_cb: TweenUpdateCallback = Box::new(move |val: &HashMap<String, f64>| {
            if let Some(&offset) = val.get("offset") {
                mat_rc.borrow_mut().offset = Some(offset);
                if let Some(ref mut u) = user_update {
                    u(offset);
                }
            }
        });

        self.add(TweenOptions {
            start_object,
            stop_object,
            duration: options.duration.unwrap_or(3.0),
            delay: options.delay,
            easing_function: options.easing_function,
            update: Some(update_cb),
            complete: None,
            cancel: options.cancel,
            repeat: Some(f64::INFINITY),
        })
    }

    /// Removes a tween from the collection, calling its cancel callback if defined.
    pub fn remove(&self, tween: &Rc<RefCell<Tween>>) -> bool {
        let id = tween.borrow().id;
        self.inner.borrow_mut().remove_by_id(id)
    }

    /// Removes all tweens, calling cancel on each.
    pub fn remove_all(&self) {
        let mut inner = self.inner.borrow_mut();
        for tween_rc in &inner.tweens {
            let mut tween = tween_rc.borrow_mut();
            if let Some(ref mut cancel) = tween.cancel {
                cancel();
            }
        }
        inner.tweens.clear();
    }

    /// Returns true if this collection contains the given tween.
    pub fn contains(&self, tween: &Rc<RefCell<Tween>>) -> bool {
        let id = tween.borrow().id;
        self.inner
            .borrow()
            .tweens
            .iter()
            .any(|t| t.try_borrow().map(|b| b.id == id).unwrap_or(false))
    }

    /// Gets the tween at the given zero-based index.
    pub fn get(&self, index: usize) -> Result<Rc<RefCell<Tween>>, DeveloperError> {
        let inner = self.inner.borrow();
        inner
            .tweens
            .get(index)
            .cloned()
            .ok_or_else(|| DeveloperError::new("index is required and must be within bounds."))
    }

    /// Updates the active tweens with the given time in seconds.
    pub fn update(&self, time: f64) {
        let mut inner = self.inner.borrow_mut();
        let mut i = 0;

        while i < inner.tweens.len() {
            let tween_rc = inner.tweens[i].clone();
            let mut tween = tween_rc.borrow_mut();

            if tween.needs_start {
                tween.needs_start = false;
                tween.start_time = Some(time + tween.delay);
            }

            let start_time = tween.start_time.unwrap_or(time);
            if time < start_time {
                i += 1;
                continue;
            }

            let elapsed = time - start_time;
            let normalized = (elapsed / tween.duration).clamp(0.0, 1.0);
            let factor = tween.easing_function.evaluate(normalized);

            let mut current_vals = HashMap::new();
            for (k, start_v) in &tween.start_object {
                let stop_v = tween.stop_object.get(k).copied().unwrap_or(*start_v);
                let current_v = *start_v + (stop_v - *start_v) * factor;
                current_vals.insert(k.clone(), current_v);
            }

            // Sync start_object values in place as CesiumJS does
            for (k, v) in &current_vals {
                tween.start_object.insert(k.clone(), *v);
            }

            if let Some(ref mut update_cb) = tween.update {
                update_cb(&current_vals);
            }

            if normalized >= 1.0 {
                if tween.repeat > 0.0 {
                    tween.start_time = Some(time);
                    i += 1;
                } else {
                    if let Some(ref mut complete) = tween.complete {
                        complete();
                    }
                    drop(tween);
                    inner.tweens.remove(i);
                }
            } else {
                i += 1;
            }
        }
    }
}
