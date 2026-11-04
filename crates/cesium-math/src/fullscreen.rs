use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::RwLock;

static SUPPORTS_FULLSCREEN: AtomicBool = AtomicBool::new(true);
static IS_FULLSCREEN: AtomicBool = AtomicBool::new(false);
static IS_ENABLED: AtomicBool = AtomicBool::new(true);
static FULLSCREEN_ELEMENT: RwLock<Option<String>> = RwLock::new(None);

/// Browser-independent functions for working with the standard fullscreen API.
pub struct Fullscreen;

impl Fullscreen {
    pub fn supports_fullscreen() -> bool {
        SUPPORTS_FULLSCREEN.load(Ordering::SeqCst)
    }

    pub fn set_supports_fullscreen_for_specs(supported: bool) {
        SUPPORTS_FULLSCREEN.store(supported, Ordering::SeqCst);
    }

    pub fn is_enabled() -> Option<bool> {
        if !Self::supports_fullscreen() {
            return None;
        }
        Some(IS_ENABLED.load(Ordering::SeqCst))
    }

    pub fn set_enabled_for_specs(enabled: bool) {
        IS_ENABLED.store(enabled, Ordering::SeqCst);
    }

    pub fn element() -> Option<Option<String>> {
        if !Self::supports_fullscreen() {
            return None;
        }
        let guard = FULLSCREEN_ELEMENT.read().unwrap();
        Some(guard.clone())
    }

    pub fn is_fullscreen() -> Option<bool> {
        if !Self::supports_fullscreen() {
            return None;
        }
        Some(IS_FULLSCREEN.load(Ordering::SeqCst))
    }

    pub fn change_event_name() -> Option<&'static str> {
        if !Self::supports_fullscreen() {
            return None;
        }
        Some("fullscreenchange")
    }

    pub fn error_event_name() -> Option<&'static str> {
        if !Self::supports_fullscreen() {
            return None;
        }
        Some("fullscreenerror")
    }

    pub fn request_fullscreen(element_id: Option<&str>) {
        if !Self::supports_fullscreen() {
            return;
        }
        IS_FULLSCREEN.store(true, Ordering::SeqCst);
        let mut guard = FULLSCREEN_ELEMENT.write().unwrap();
        *guard = element_id.map(|s| s.to_string());
    }

    pub fn exit_fullscreen() {
        if !Self::supports_fullscreen() {
            return;
        }
        IS_FULLSCREEN.store(false, Ordering::SeqCst);
        let mut guard = FULLSCREEN_ELEMENT.write().unwrap();
        *guard = None;
    }

    pub fn reset_for_specs() {
        SUPPORTS_FULLSCREEN.store(true, Ordering::SeqCst);
        IS_FULLSCREEN.store(false, Ordering::SeqCst);
        IS_ENABLED.store(true, Ordering::SeqCst);
        let mut guard = FULLSCREEN_ELEMENT.write().unwrap();
        *guard = None;
    }
}
