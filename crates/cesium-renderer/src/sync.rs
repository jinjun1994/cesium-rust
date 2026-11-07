//! WebGLSync fence synchronization abstraction matching CesiumJS `Renderer/Sync.js` (Apache-2.0).

use cesium_math::{DeveloperError, RuntimeError};

/// Status of the GPU fence sync object.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SyncStatus {
    Unsignaled = 0x9118,
    Signaled = 0x9119,
}

/// GPU fence sync object for synchronizing GPU and CPU activities.
#[derive(Debug)]
pub struct Sync {
    status: SyncStatus,
    is_destroyed: bool,
}

impl Default for Sync {
    fn default() -> Self {
        Self::new()
    }
}

impl Sync {
    /// Creates a new GPU sync object initialized in the Unsignaled state.
    pub fn new() -> Self {
        Self {
            status: SyncStatus::Unsignaled,
            is_destroyed: false,
        }
    }

    /// Queries the current status of the sync object.
    pub fn get_status(&self) -> SyncStatus {
        self.status
    }

    /// Sets the sync status (useful for simulated or GPU fence callback updates).
    pub fn set_status(&mut self, status: SyncStatus) {
        self.status = status;
    }

    /// Signals the sync object, completing the fence.
    pub fn signal(&mut self) {
        self.status = SyncStatus::Signaled;
    }

    /// Returns whether this sync object has been destroyed.
    pub fn is_destroyed(&self) -> bool {
        self.is_destroyed
    }

    /// Destroys this sync object.
    pub fn destroy(&mut self) -> Result<(), DeveloperError> {
        if self.is_destroyed {
            return Err(DeveloperError::new(
                "This object was destroyed, i.e., destroy() was called.",
            ));
        }
        self.is_destroyed = true;
        Ok(())
    }

    /// Incrementally polls the status of the Sync object until signaled.
    ///
    /// The `schedule_func` is repeatedly called with a tick count or callback.
    /// If signaled, resolves successfully; if TTL exhausts, returns `RuntimeError("Wait for signal timeout")`.
    pub fn wait_for_signal<F>(
        &mut self,
        mut poll_action: F,
        ttl: Option<usize>,
    ) -> Result<(), RuntimeError>
    where
        F: FnMut(&mut Self, usize) -> bool, // returns true to continue, or mutates status
    {
        let mut remaining = ttl.unwrap_or(10);

        loop {
            if self.status == SyncStatus::Signaled {
                return Ok(());
            }

            if remaining == 0 {
                return Err(RuntimeError::new("Wait for signal timeout"));
            }

            remaining -= 1;
            let should_continue = poll_action(self, remaining);
            if !should_continue && self.status != SyncStatus::Signaled {
                // If poll action stopped early without signaling, check one more time
                if remaining == 0 && self.status != SyncStatus::Signaled {
                    return Err(RuntimeError::new("Wait for signal timeout"));
                }
            }
        }
    }
}
