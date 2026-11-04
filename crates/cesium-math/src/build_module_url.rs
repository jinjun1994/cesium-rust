use std::sync::RwLock;

use crate::append_forward_slash::append_forward_slash;
use crate::errors::DeveloperError;

static BASE_URL: RwLock<Option<String>> = RwLock::new(None);

/// Matches a script URL against the Cesium script regex pattern:
/// `((?:.*\/)|^)Cesium\.js(?:\?|\#|$)`
/// Returns the prefix up to `Cesium.js` if matched.
pub fn match_cesium_script_url(url: &str) -> Option<&str> {
    let target = "Cesium.js";
    let pos = url.find(target)?;

    // Preceding part must be either the beginning of the string or end with '/'
    if pos > 0 && !url[..pos].ends_with('/') {
        return None;
    }

    // Following part must be empty, start with '?', or start with '#'
    let after = &url[pos + target.len()..];
    if !after.is_empty() && !after.starts_with('?') && !after.starts_with('#') {
        return None;
    }

    Some(&url[..pos])
}

/// Builds a module URL given a base URL and a relative path.
pub fn build_module_url_from_base_url(base_url: &str, relative_path: &str) -> String {
    let mut base = append_forward_slash(base_url);
    let trimmed_rel = relative_path.trim_start_matches('/');
    base.push_str(trimmed_rel);
    base
}

/// Sets the base URL used by `build_module_url`.
pub fn set_base_url(url: &str) {
    if let Ok(mut lock) = BASE_URL.write() {
        *lock = Some(append_forward_slash(url));
    }
}

/// Gets the currently configured base URL, if any.
pub fn get_base_url() -> Option<String> {
    BASE_URL.read().ok().and_then(|lock| lock.clone())
}

/// Clears the base URL cache.
pub fn clear_base_url() {
    if let Ok(mut lock) = BASE_URL.write() {
        *lock = None;
    }
}

/// Resolves a relative module URL using the configured base URL.
pub fn build_module_url(relative_path: &str) -> Result<String, DeveloperError> {
    let base = get_base_url().ok_or_else(|| {
        DeveloperError::new(
            "Unable to determine Cesium base URL automatically, try setting base url via set_base_url.",
        )
    })?;
    Ok(build_module_url_from_base_url(&base, relative_path))
}
