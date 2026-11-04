use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::RwLock;

use crate::errors::DeveloperError;

static WEBP_INITIALIZED: AtomicBool = AtomicBool::new(false);
static WEBP_SUPPORTED: AtomicBool = AtomicBool::new(false);
static CUSTOM_USER_AGENT: RwLock<Option<String>> = RwLock::new(None);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WebkitVersion {
    pub version: Vec<u32>,
    pub is_nightly: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct UserAgentInfo {
    pub is_chrome: bool,
    pub chrome_version: Option<Vec<u32>>,
    pub is_safari: bool,
    pub safari_version: Option<Vec<u32>>,
    pub is_webkit: bool,
    pub webkit_version: Option<WebkitVersion>,
    pub is_edge: bool,
    pub edge_version: Option<Vec<u32>>,
    pub is_firefox: bool,
    pub firefox_version: Option<Vec<u32>>,
    pub is_ipad_or_ios: bool,
}

pub struct FeatureDetection;

impl FeatureDetection {
    pub fn supports_fullscreen() -> bool {
        true
    }

    pub fn supports_web_workers() -> bool {
        true
    }

    pub fn supports_typed_arrays() -> bool {
        true
    }

    pub fn supports_big_int64_array() -> bool {
        true
    }

    pub fn supports_big_uint64_array() -> bool {
        true
    }

    pub fn supports_big_int() -> bool {
        true
    }

    pub fn supports_web_assembly() -> bool {
        true
    }

    pub fn set_user_agent_for_specs(user_agent: Option<&str>) {
        let mut ua_lock = CUSTOM_USER_AGENT.write().unwrap();
        *ua_lock = user_agent.map(|s| s.to_string());
    }

    pub fn current_user_agent() -> String {
        if let Ok(guard) = CUSTOM_USER_AGENT.read() {
            if let Some(ref ua) = *guard {
                return ua.clone();
            }
        }
        String::new()
    }

    #[allow(clippy::field_reassign_with_default)]
    pub fn parse_user_agent(ua: &str) -> UserAgentInfo {
        let mut info = UserAgentInfo::default();

        info.is_ipad_or_ios = ua.contains("iPhone") || ua.contains("iPad") || ua.contains("iPod");

        // Edge (Edg/xx.xx)
        if let Some(pos) = ua.find(" Edg/") {
            info.is_edge = true;
            let slice = &ua[pos + 5..];
            info.edge_version = Some(extract_version(slice));
        }

        // Chrome (Chrome/xx.xx)
        if !info.is_edge {
            if let Some(pos) = ua.find(" Chrome/") {
                info.is_chrome = true;
                let slice = &ua[pos + 8..];
                info.chrome_version = Some(extract_version(slice));
            }
        }

        // Safari
        if !info.is_chrome && !info.is_edge && ua.contains(" Safari/") {
            if let Some(pos) = ua.find(" Version/") {
                info.is_safari = true;
                let slice = &ua[pos + 9..];
                info.safari_version = Some(extract_version(slice));
            }
        }

        // WebKit
        if let Some(pos) = ua.find(" AppleWebKit/") {
            info.is_webkit = true;
            let slice = &ua[pos + 13..];
            let end = slice.find(' ').unwrap_or(slice.len());
            let token = &slice[..end];
            let is_nightly = token.ends_with('+');
            let version_str = token.trim_end_matches('+');
            info.webkit_version = Some(WebkitVersion {
                version: extract_version(version_str),
                is_nightly,
            });
        }

        // Firefox
        if let Some(pos) = ua.find("Firefox/") {
            info.is_firefox = true;
            let slice = &ua[pos + 8..];
            info.firefox_version = Some(extract_version(slice));
        }

        info
    }

    pub fn is_chrome() -> bool {
        Self::parse_user_agent(&Self::current_user_agent()).is_chrome
    }

    pub fn chrome_version() -> Option<Vec<u32>> {
        Self::parse_user_agent(&Self::current_user_agent()).chrome_version
    }

    pub fn is_safari() -> bool {
        Self::parse_user_agent(&Self::current_user_agent()).is_safari
    }

    pub fn safari_version() -> Option<Vec<u32>> {
        Self::parse_user_agent(&Self::current_user_agent()).safari_version
    }

    pub fn is_webkit() -> bool {
        Self::parse_user_agent(&Self::current_user_agent()).is_webkit
    }

    pub fn webkit_version() -> Option<WebkitVersion> {
        Self::parse_user_agent(&Self::current_user_agent()).webkit_version
    }

    pub fn is_edge() -> bool {
        Self::parse_user_agent(&Self::current_user_agent()).is_edge
    }

    pub fn edge_version() -> Option<Vec<u32>> {
        Self::parse_user_agent(&Self::current_user_agent()).edge_version
    }

    pub fn is_firefox() -> bool {
        Self::parse_user_agent(&Self::current_user_agent()).is_firefox
    }

    pub fn firefox_version() -> Option<Vec<u32>> {
        Self::parse_user_agent(&Self::current_user_agent()).firefox_version
    }

    pub fn is_ipad_or_ios() -> bool {
        Self::parse_user_agent(&Self::current_user_agent()).is_ipad_or_ios
    }

    pub fn supports_image_rendering_pixelated() -> bool {
        true
    }

    pub fn image_rendering_value() -> Option<&'static str> {
        if Self::supports_image_rendering_pixelated() {
            Some("pixelated")
        } else {
            None
        }
    }

    pub fn reset_web_p_for_specs() {
        WEBP_INITIALIZED.store(false, Ordering::SeqCst);
        WEBP_SUPPORTED.store(false, Ordering::SeqCst);
    }

    pub fn initialize_web_p(supported: bool) {
        WEBP_SUPPORTED.store(supported, Ordering::SeqCst);
        WEBP_INITIALIZED.store(true, Ordering::SeqCst);
    }

    pub fn supports_web_p() -> Result<bool, DeveloperError> {
        if !WEBP_INITIALIZED.load(Ordering::SeqCst) {
            return Err(DeveloperError::new(
                "You must call FeatureDetection.supportsWebP.initialize and wait for the promise to resolve before calling FeatureDetection.supportsWebP",
            ));
        }
        Ok(WEBP_SUPPORTED.load(Ordering::SeqCst))
    }
}

fn extract_version(slice: &str) -> Vec<u32> {
    let mut parts = Vec::new();
    let end = slice.find(|c: char| !c.is_ascii_digit() && c != '.').unwrap_or(slice.len());
    let ver_str = &slice[..end];
    for p in ver_str.split('.') {
        if let Ok(num) = p.parse::<u32>() {
            parts.push(num);
        }
    }
    parts
}
