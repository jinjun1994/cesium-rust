use crate::DeveloperError;
use std::collections::HashSet;
use std::sync::Mutex;

static WARNINGS: Mutex<Option<HashSet<String>>> = Mutex::new(None);

/// Logs a one-time message if it has not been logged before.
pub fn one_time_warning(identifier: &str, message: Option<&str>) -> Result<bool, DeveloperError> {
    if identifier.is_empty() {
        return Err(DeveloperError::new("identifier is required."));
    }
    let mut lock = WARNINGS.lock().unwrap();
    let set = lock.get_or_insert_with(HashSet::new);
    if set.insert(identifier.to_string()) {
        let msg = message.unwrap_or(identifier);
        eprintln!("[Cesium Warning]: {}", msg);
        Ok(true)
    } else {
        Ok(false)
    }
}

/// Logs a deprecation warning if it has not been logged before.
pub fn deprecation_warning(identifier: &str, message: &str) -> Result<bool, DeveloperError> {
    if identifier.is_empty() || message.is_empty() {
        return Err(DeveloperError::new("identifier and message are required."));
    }
    one_time_warning(identifier, Some(message))
}

/// Resets recorded warnings (useful for testing).
pub fn reset_warnings() {
    let mut lock = WARNINGS.lock().unwrap();
    if let Some(set) = lock.as_mut() {
        set.clear();
    }
}
