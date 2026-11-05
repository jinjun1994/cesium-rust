use std::collections::HashSet;
use std::sync::RwLock;

use crate::DeveloperError;

static GLOBAL_SERVERS: RwLock<Option<HashSet<String>>> = RwLock::new(None);

/// A registry that tracks all servers that are trusted.
#[derive(Debug, Clone, Default)]
pub struct TrustedServers {
    servers: HashSet<String>,
}

impl TrustedServers {
    pub fn new() -> Self {
        Self {
            servers: HashSet::new(),
        }
    }

    /// Adds a trusted server to the registry.
    pub fn add(&mut self, host: &str, port: u16) -> Result<(), DeveloperError> {
        let trimmed = host.trim();
        if trimmed.is_empty() {
            return Err(DeveloperError::new("host is required."));
        }
        if port == 0 {
            return Err(DeveloperError::new("port is required to be greater than 0."));
        }

        let authority = format!("{}:{}", trimmed.to_ascii_lowercase(), port);
        self.servers.insert(authority);
        Ok(())
    }

    /// Removes a trusted server from the registry.
    pub fn remove(&mut self, host: &str, port: u16) -> Result<(), DeveloperError> {
        let trimmed = host.trim();
        if trimmed.is_empty() {
            return Err(DeveloperError::new("host is required."));
        }
        if port == 0 {
            return Err(DeveloperError::new("port is required to be greater than 0."));
        }

        let authority = format!("{}:{}", trimmed.to_ascii_lowercase(), port);
        self.servers.remove(&authority);
        Ok(())
    }

    /// Tests whether a URL points to a trusted server.
    pub fn contains(&self, url: &str) -> Result<bool, DeveloperError> {
        if url.trim().is_empty() {
            return Err(DeveloperError::new("url is required."));
        }

        let authority = match get_authority(url) {
            Some(auth) => auth,
            None => return Ok(false),
        };

        Ok(self.servers.contains(&authority))
    }

    /// Clears the registry.
    pub fn clear(&mut self) {
        self.servers.clear();
    }

    // Global singleton methods matching Cesium's static API

    pub fn add_global(host: &str, port: u16) -> Result<(), DeveloperError> {
        let trimmed = host.trim();
        if trimmed.is_empty() {
            return Err(DeveloperError::new("host is required."));
        }
        if port == 0 {
            return Err(DeveloperError::new("port is required to be greater than 0."));
        }

        let authority = format!("{}:{}", trimmed.to_ascii_lowercase(), port);
        let mut guard = GLOBAL_SERVERS.write().unwrap();
        let servers = guard.get_or_insert_with(HashSet::new);
        servers.insert(authority);
        Ok(())
    }

    pub fn remove_global(host: &str, port: u16) -> Result<(), DeveloperError> {
        let trimmed = host.trim();
        if trimmed.is_empty() {
            return Err(DeveloperError::new("host is required."));
        }
        if port == 0 {
            return Err(DeveloperError::new("port is required to be greater than 0."));
        }

        let authority = format!("{}:{}", trimmed.to_ascii_lowercase(), port);
        let mut guard = GLOBAL_SERVERS.write().unwrap();
        if let Some(servers) = guard.as_mut() {
            servers.remove(&authority);
        }
        Ok(())
    }

    pub fn contains_global(url: &str) -> Result<bool, DeveloperError> {
        if url.trim().is_empty() {
            return Err(DeveloperError::new("url is required."));
        }

        let authority = match get_authority(url) {
            Some(auth) => auth,
            None => return Ok(false),
        };

        let guard = GLOBAL_SERVERS.read().unwrap();
        if let Some(servers) = guard.as_ref() {
            Ok(servers.contains(&authority))
        } else {
            Ok(false)
        }
    }

    pub fn clear_global() {
        let mut guard = GLOBAL_SERVERS.write().unwrap();
        if let Some(servers) = guard.as_mut() {
            servers.clear();
        }
    }
}

fn get_authority(url: &str) -> Option<String> {
    let trimmed = url.trim();

    let (scheme, rest) = if let Some(stripped) = trimmed.strip_prefix("https://") {
        ("https", stripped)
    } else if let Some(stripped) = trimmed.strip_prefix("http://") {
        ("http", stripped)
    } else {
        let stripped = trimmed.strip_prefix("//")?;
        ("http", stripped)
    };

    // Strip path, query, fragment
    let host_and_port = rest.split(['/', '?', '#']).next()?;

    // Strip credentials username:password@
    let host_and_port = if let Some(idx) = host_and_port.rfind('@') {
        &host_and_port[idx + 1..]
    } else {
        host_and_port
    };

    if host_and_port.is_empty() {
        return None;
    }

    let authority = if host_and_port.contains(':') {
        host_and_port.to_ascii_lowercase()
    } else if scheme == "http" {
        format!("{}:80", host_and_port.to_ascii_lowercase())
    } else if scheme == "https" {
        format!("{}:443", host_and_port.to_ascii_lowercase())
    } else {
        return None;
    };

    Some(authority)
}
