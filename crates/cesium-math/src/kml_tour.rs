//! KmlTour, KmlTourFlyTo, KmlTourWait, KmlLookAt, KmlCamera matching CesiumJS `DataSources/Kml*.js` (Apache-2.0).

use std::cell::RefCell;
use std::rc::Rc;

use crate::{BoundingSphere, Cartesian3, Event, HeadingPitchRange, HeadingPitchRoll};

/// Camera orientation and position representation for KML `<LookAt>`.
#[derive(Clone, Debug, PartialEq)]
pub struct KmlLookAt {
    pub position: Cartesian3,
    pub heading_pitch_range: HeadingPitchRange,
}

impl KmlLookAt {
    pub fn new(position: Cartesian3, heading_pitch_range: HeadingPitchRange) -> Self {
        Self {
            position,
            heading_pitch_range,
        }
    }
}

/// Camera orientation and position representation for KML `<Camera>`.
#[derive(Clone, Debug, PartialEq)]
pub struct KmlCamera {
    pub position: Cartesian3,
    pub heading_pitch_roll: HeadingPitchRoll,
}

impl KmlCamera {
    pub fn new(position: Cartesian3, heading_pitch_roll: HeadingPitchRoll) -> Self {
        Self {
            position,
            heading_pitch_roll,
        }
    }
}

/// View target for KML tour flight, either a LookAt or Camera.
#[derive(Clone, Debug, PartialEq)]
pub enum KmlTourView {
    LookAt(KmlLookAt),
    Camera(KmlCamera),
}

impl From<KmlLookAt> for KmlTourView {
    fn from(v: KmlLookAt) -> Self {
        Self::LookAt(v)
    }
}

impl From<KmlCamera> for KmlTourView {
    fn from(v: KmlCamera) -> Self {
        Self::Camera(v)
    }
}

/// Easing functions supported by KML tour flight.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EasingFunction {
    LinearNone,
}

/// Options generated for camera flights.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct KmlCameraOptions {
    pub duration: f64,
    pub destination: Option<Cartesian3>,
    pub orientation: Option<HeadingPitchRoll>,
    pub offset: Option<HeadingPitchRange>,
    pub easing_function: Option<EasingFunction>,
    pub has_complete: bool,
}

/// Trait implemented by camera flight controllers (e.g. Scene camera or test mock).
pub trait KmlCameraTarget {
    fn fly_to(&mut self, options: &KmlCameraOptions, on_complete: Box<dyn FnOnce(bool)>);
    fn fly_to_bounding_sphere(
        &mut self,
        bounding_sphere: &BoundingSphere,
        options: &KmlCameraOptions,
        on_complete: Box<dyn FnOnce(bool)>,
    );
    fn cancel_flight(&mut self);
}

/// Callback invoked when a tour entry finishes. Parameter indicates if it was terminated early.
pub type KmlTourDoneCallback = Box<dyn FnOnce(bool)>;

type SharedDoneCallback = Rc<RefCell<Option<KmlTourDoneCallback>>>;

/// Transitions the KmlTour to the next destination.
pub struct KmlTourFlyTo {
    pub duration: f64,
    pub fly_to_mode: String,
    pub view: KmlTourView,
    pub blocking: bool,
    pub active_callback: Option<SharedDoneCallback>,
}

impl KmlTourFlyTo {
    /// Creates a new KmlTourFlyTo transition.
    pub fn new(
        duration: f64,
        fly_to_mode: impl Into<String>,
        view: impl Into<KmlTourView>,
    ) -> Self {
        Self {
            duration,
            fly_to_mode: fly_to_mode.into(),
            view: view.into(),
            blocking: true,
            active_callback: None,
        }
    }

    /// Returns options for `Camera.fly_to` or `Camera.fly_to_bounding_sphere`.
    pub fn get_camera_options(
        &self,
        camera_options: Option<&KmlCameraOptions>,
    ) -> KmlCameraOptions {
        let mut options = KmlCameraOptions {
            duration: self.duration,
            destination: None,
            orientation: None,
            offset: None,
            easing_function: if self.fly_to_mode == "smooth" {
                Some(EasingFunction::LinearNone)
            } else {
                None
            },
            has_complete: self.active_callback.is_some(),
        };

        match &self.view {
            KmlTourView::Camera(cam) => {
                options.destination = Some(cam.position);
                options.orientation = Some(cam.heading_pitch_roll);
            }
            KmlTourView::LookAt(look_at) => {
                options.offset = Some(look_at.heading_pitch_range);
            }
        }

        if let Some(extra) = camera_options {
            if extra.duration > 0.0 {
                options.duration = extra.duration;
            }
            if extra.destination.is_some() {
                options.destination = extra.destination;
            }
            if extra.orientation.is_some() {
                options.orientation = extra.orientation;
            }
            if extra.offset.is_some() {
                options.offset = extra.offset;
            }
            if extra.easing_function.is_some() {
                options.easing_function = extra.easing_function;
            }
        }

        options
    }

    /// Sets an explicit active callback.
    pub fn set_active_callback<F: FnOnce(bool) + 'static>(&mut self, cb: F) {
        self.active_callback = Some(Rc::new(RefCell::new(Some(Box::new(cb)))));
    }

    /// Invokes the registered complete callback.
    pub fn invoke_complete(&mut self, terminated: bool) {
        if let Some(cb_cell) = self.active_callback.take() {
            if let Some(cb) = cb_cell.borrow_mut().take() {
                cb(terminated);
            }
        }
    }

    /// Plays this playlist entry.
    pub fn play(
        &mut self,
        done: Option<Box<dyn FnOnce(bool)>>,
        camera: &mut dyn KmlCameraTarget,
        camera_options: Option<&KmlCameraOptions>,
    ) {
        let cb_cell = Rc::new(RefCell::new(done));
        self.active_callback = Some(Rc::clone(&cb_cell));
        let options = self.get_camera_options(camera_options);

        let on_complete = Box::new(move |terminated: bool| {
            if let Some(cb) = cb_cell.borrow_mut().take() {
                cb(terminated);
            }
        });

        match &self.view {
            KmlTourView::Camera(_) => {
                camera.fly_to(&options, on_complete);
            }
            KmlTourView::LookAt(look_at) => {
                let sphere = BoundingSphere::new(look_at.position, 0.0);
                camera.fly_to_bounding_sphere(&sphere, &options, on_complete);
            }
        }
    }

    /// Stops execution of current entry.
    pub fn stop(&mut self, camera: Option<&mut dyn KmlCameraTarget>) {
        if let Some(cam) = camera {
            cam.cancel_flight();
        }
        self.invoke_complete(true);
    }
}

/// Pauses the KmlTour for a given number of seconds.
pub struct KmlTourWait {
    pub duration: f64,
    pub blocking: bool,
    pub active_callback: Option<SharedDoneCallback>,
}

impl KmlTourWait {
    /// Creates a new KmlTourWait entry.
    pub fn new(duration: f64) -> Self {
        Self {
            duration,
            blocking: true,
            active_callback: None,
        }
    }

    /// Plays this wait entry.
    pub fn play(&mut self, done: Box<dyn FnOnce(bool)>) {
        self.active_callback = Some(Rc::new(RefCell::new(Some(done))));
    }

    /// Triggers normal completion of the wait timer.
    pub fn trigger_done(&mut self) {
        if let Some(cb_cell) = self.active_callback.take() {
            if let Some(cb) = cb_cell.borrow_mut().take() {
                cb(false);
            }
        }
    }

    /// Stops execution of current entry.
    pub fn stop(&mut self) {
        if let Some(cb_cell) = self.active_callback.take() {
            if let Some(cb) = cb_cell.borrow_mut().take() {
                cb(true);
            }
        }
    }
}

/// Playlist entry in a KmlTour.
pub enum KmlTourEntry {
    FlyTo(KmlTourFlyTo),
    Wait(KmlTourWait),
}

impl From<KmlTourFlyTo> for KmlTourEntry {
    fn from(f: KmlTourFlyTo) -> Self {
        Self::FlyTo(f)
    }
}

impl From<KmlTourWait> for KmlTourEntry {
    fn from(w: KmlTourWait) -> Self {
        Self::Wait(w)
    }
}

impl KmlTourEntry {
    pub fn is_fly_to(&self) -> bool {
        matches!(self, Self::FlyTo(_))
    }

    pub fn is_wait(&self) -> bool {
        matches!(self, Self::Wait(_))
    }

    pub fn as_fly_to(&self) -> Option<&KmlTourFlyTo> {
        match self {
            Self::FlyTo(f) => Some(f),
            _ => None,
        }
    }

    pub fn as_wait(&self) -> Option<&KmlTourWait> {
        match self {
            Self::Wait(w) => Some(w),
            _ => None,
        }
    }

    pub fn as_fly_to_mut(&mut self) -> Option<&mut KmlTourFlyTo> {
        match self {
            Self::FlyTo(f) => Some(f),
            _ => None,
        }
    }

    pub fn as_wait_mut(&mut self) -> Option<&mut KmlTourWait> {
        match self {
            Self::Wait(w) => Some(w),
            _ => None,
        }
    }

    pub fn stop(&mut self, camera: Option<&mut dyn KmlCameraTarget>) {
        match self {
            Self::FlyTo(f) => f.stop(camera),
            Self::Wait(w) => w.stop(),
        }
    }
}

/// Describes a KmlTour guiding the camera through a playlist of flyTo and wait entries.
pub struct KmlTour {
    pub id: String,
    pub name: String,
    pub playlist_index: usize,
    pub playlist: Vec<KmlTourEntry>,
    pub tour_start: Event<()>,
    pub tour_end: Event<bool>,           // terminated
    pub entry_start: Event<usize>,       // playlist_index
    pub entry_end: Event<(usize, bool)>, // (playlist_index, terminated)
    active_entries: Vec<usize>,
}

impl KmlTour {
    /// Creates a new KmlTour.
    pub fn new(name: impl Into<String>, id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            playlist_index: 0,
            playlist: Vec::new(),
            tour_start: Event::new(),
            tour_end: Event::new(),
            entry_start: Event::new(),
            entry_end: Event::new(),
            active_entries: Vec::new(),
        }
    }

    /// Adds an entry to this tour playlist.
    pub fn add_playlist_entry(&mut self, entry: impl Into<KmlTourEntry>) {
        self.playlist.push(entry.into());
    }

    /// Plays this tour from the beginning.
    pub fn play(
        &mut self,
        camera: &mut dyn KmlCameraTarget,
        camera_options: Option<&KmlCameraOptions>,
    ) {
        self.tour_start.raise_event(&());
        self.playlist_index = 0;
        self.play_current_entry(camera, camera_options);
    }

    fn play_current_entry(
        &mut self,
        camera: &mut dyn KmlCameraTarget,
        camera_options: Option<&KmlCameraOptions>,
    ) {
        if self.playlist_index >= self.playlist.len() {
            self.playlist_index = 0;
            self.cancel_all_active_entries(Some(camera));
            self.tour_end.raise_event(&false);
            return;
        }

        let idx = self.playlist_index;
        self.active_entries.push(idx);
        self.entry_start.raise_event(&idx);

        match &mut self.playlist[idx] {
            KmlTourEntry::FlyTo(fly_to) => {
                fly_to.play(None, camera, camera_options);
            }
            KmlTourEntry::Wait(_) => {
                // Wait timer will be completed via step_wait or complete_wait
            }
        }
    }

    /// Advances to the next entry after completing the current entry.
    pub fn step_next(
        &mut self,
        terminated: bool,
        camera: &mut dyn KmlCameraTarget,
        camera_options: Option<&KmlCameraOptions>,
    ) {
        let idx = self.playlist_index;
        self.active_entries.retain(|&i| i != idx);
        self.entry_end.raise_event(&(idx, terminated));

        if terminated {
            self.playlist_index = 0;
            self.cancel_all_active_entries(Some(camera));
            self.tour_end.raise_event(&true);
        } else {
            self.playlist_index += 1;
            self.play_current_entry(camera, camera_options);
        }
    }

    /// Stops currently playing tour.
    pub fn stop(&mut self, camera: Option<&mut dyn KmlCameraTarget>) {
        let cur = self.playlist_index;
        if cur < self.playlist.len() {
            self.entry_end.raise_event(&(cur, true));
        }
        self.cancel_all_active_entries(camera);
        self.playlist_index = 0;
        self.tour_end.raise_event(&true);
    }

    fn cancel_all_active_entries(&mut self, camera: Option<&mut dyn KmlCameraTarget>) {
        if let Some(cam) = camera {
            cam.cancel_flight();
        }
        for idx in self.active_entries.drain(..) {
            if idx < self.playlist.len() {
                self.playlist[idx].stop(None);
            }
        }
    }
}
