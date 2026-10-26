//! Device lifecycle, error routing, and the recovery state machine.
//!
//! State transition model:
//! ```text
//! Running
//!   │  (device lost / uncaptured error)
//!   ▼
//!  Lost ──► Recreating ──► Rehydrating ──► Running
//!   │           │               │
//!   └───────────┴───────────────┴────────► Failed
//! ```

use std::sync::mpsc;
use std::sync::Arc;

/// Life cycle status of the underlying GPU device.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DeviceStatus {
    /// Device is healthy, processing frames normally.
    Running,
    /// Device lost detected; pending GPU commands and memory mapping are invalid.
    Lost { reason: String, message: String },
    /// Recreating adapter, device, queue, surface configurations, and baseline pipelines.
    Recreating,
    /// Device recreated; scene and viewer are rehydrating CPU-resident meshes and textures.
    Rehydrating,
    /// Unrecoverable failure occurred during initialization or recreation.
    Failed(String),
}

/// Tracks device epoch, asynchronous error channels, and recovery transitions.
pub struct DeviceContext {
    pub status: DeviceStatus,
    pub epoch: u64,
    errors_rx: mpsc::Receiver<String>,
    error_tx: mpsc::Sender<String>,
    consecutive_failures: u32,
}

impl DeviceContext {
    pub fn new(device: &wgpu::Device) -> Self {
        let (error_tx, errors_rx) = mpsc::channel();
        let lost_tx = error_tx.clone();
        device.set_device_lost_callback(move |reason, message| {
            let _ = lost_tx.send(format!("device lost: {reason:?}: {message}"));
        });
        let uncaptured_tx = error_tx.clone();
        device.on_uncaptured_error(Arc::new(move |error: wgpu::Error| {
            let _ = uncaptured_tx.send(format!("uncaptured: {error}"));
        }));

        Self {
            status: DeviceStatus::Running,
            epoch: 0,
            errors_rx,
            error_tx,
            consecutive_failures: 0,
        }
    }

    /// Re-binds callbacks on a newly created device after recreation.
    pub fn rebind_callbacks(&mut self, device: &wgpu::Device) {
        let lost_tx = self.error_tx.clone();
        device.set_device_lost_callback(move |reason, message| {
            let _ = lost_tx.send(format!("device lost: {reason:?}: {message}"));
        });
        let uncaptured_tx = self.error_tx.clone();
        device.on_uncaptured_error(Arc::new(move |error: wgpu::Error| {
            let _ = uncaptured_tx.send(format!("uncaptured: {error}"));
        }));
    }

    /// Polls pending asynchronous device errors and updates status if lost.
    pub fn poll_health(&mut self) -> Result<(), String> {
        while let Ok(msg) = self.errors_rx.try_recv() {
            if msg.starts_with("device lost:") {
                let parts: Vec<&str> = msg.splitn(3, ':').collect();
                let reason = parts.get(1).unwrap_or(&"Unknown").trim().to_string();
                let message = parts.get(2).unwrap_or(&"").trim().to_string();
                self.status = DeviceStatus::Lost { reason, message };
                self.epoch = self.epoch.wrapping_add(1);
            } else if matches!(self.status, DeviceStatus::Running) {
                // Non-fatal or uncaptured error recorded
                return Err(msg);
            }
        }
        match &self.status {
            DeviceStatus::Running => Ok(()),
            DeviceStatus::Lost { reason, message } => {
                Err(format!("device is lost ({reason}): {message}"))
            }
            DeviceStatus::Recreating => Err("device is currently recreating".into()),
            DeviceStatus::Rehydrating => Err("device is currently rehydrating resources".into()),
            DeviceStatus::Failed(e) => Err(format!("device recovery failed: {e}")),
        }
    }

    /// Marks the device as lost explicitly (e.g. from surface acquisition failure).
    pub fn mark_lost(&mut self, reason: impl Into<String>, message: impl Into<String>) {
        self.status = DeviceStatus::Lost {
            reason: reason.into(),
            message: message.into(),
        };
        self.epoch = self.epoch.wrapping_add(1);
    }

    /// Initiates recreation transition.
    pub fn start_recreating(&mut self) {
        self.status = DeviceStatus::Recreating;
        self.epoch = self.epoch.wrapping_add(1);
    }

    /// Advances to rehydrating state after base pipelines and layouts are created.
    pub fn advance_to_rehydrating(&mut self) {
        self.status = DeviceStatus::Rehydrating;
    }

    /// Completes the recovery cycle and marks device as running.
    pub fn complete_recovery(&mut self) {
        self.status = DeviceStatus::Running;
        self.consecutive_failures = 0;
    }

    /// Records failure and handles retry backoff limit.
    pub fn mark_failed(&mut self, error: impl Into<String>) {
        self.consecutive_failures = self.consecutive_failures.saturating_add(1);
        let err_str = error.into();
        self.status = DeviceStatus::Failed(err_str);
    }

    pub fn is_healthy(&self) -> bool {
        matches!(self.status, DeviceStatus::Running)
    }

    pub fn consecutive_failures(&self) -> u32 {
        self.consecutive_failures
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn state_machine_transitions() {
        let (tx, rx) = mpsc::channel();
        let mut ctx = DeviceContext {
            status: DeviceStatus::Running,
            epoch: 0,
            errors_rx: rx,
            error_tx: tx,
            consecutive_failures: 0,
        };

        assert!(ctx.is_healthy());
        assert_eq!(ctx.epoch, 0);

        // Mark lost
        ctx.mark_lost("DeviceRemoved", "GPU unplugged");
        assert_eq!(
            ctx.status,
            DeviceStatus::Lost {
                reason: "DeviceRemoved".into(),
                message: "GPU unplugged".into()
            }
        );
        assert_eq!(ctx.epoch, 1);
        assert!(!ctx.is_healthy());

        // Start recreating
        ctx.start_recreating();
        assert_eq!(ctx.status, DeviceStatus::Recreating);
        assert_eq!(ctx.epoch, 2);

        // Advance to rehydrating
        ctx.advance_to_rehydrating();
        assert_eq!(ctx.status, DeviceStatus::Rehydrating);

        // Complete recovery
        ctx.complete_recovery();
        assert_eq!(ctx.status, DeviceStatus::Running);
        assert!(ctx.is_healthy());

        // Failure path
        ctx.mark_failed("AdapterNotFound");
        assert_eq!(ctx.status, DeviceStatus::Failed("AdapterNotFound".into()));
        assert_eq!(ctx.consecutive_failures(), 1);
    }
}
