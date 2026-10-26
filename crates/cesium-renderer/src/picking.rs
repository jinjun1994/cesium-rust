//! GPU integer ID attachment, async 1x1 readback, and FeatureRef resolution.

use std::sync::mpsc;

/// Stable identifier of a picked feature within a mesh.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct FeatureRef {
    pub mesh: u64,
    pub feature: u32,
}

/// Contiguous range of feature IDs allocated to a specific mesh in a single frame.
#[derive(Clone, Copy, Debug)]
pub struct PickRange {
    pub base: u32,
    pub count: u32,
    pub mesh: u64,
}

/// Asynchronous readback task waiting for GPU buffer mapping.
pub struct PendingPick {
    pub buffer: wgpu::Buffer,
    pub receiver: mpsc::Receiver<Result<(), wgpu::BufferAsyncError>>,
    pub ranges: Vec<PickRange>,
    pub epoch: u64,
}

impl PendingPick {
    /// Polls the buffer mapping status and extracts the picked FeatureRef.
    pub fn poll(self, current_epoch: u64) -> (Option<Option<FeatureRef>>, Option<Self>) {
        let status = match self.receiver.try_recv() {
            Ok(v) => v,
            Err(mpsc::TryRecvError::Empty) => return (None, Some(self)),
            Err(_) => Err(wgpu::BufferAsyncError),
        };

        if status.is_err() {
            return (Some(None), None);
        }

        let view = match self.buffer.slice(..).get_mapped_range() {
            Ok(v) => v,
            Err(_) => return (Some(None), None),
        };

        let id = u32::from_le_bytes([view[0], view[1], view[2], view[3]]);
        drop(view);
        self.buffer.unmap();

        if self.epoch != current_epoch {
            // Discard readback if viewport or device epoch changed in between
            return (Some(None), None);
        }

        let result = self
            .ranges
            .into_iter()
            .find(|r| id >= r.base && id - r.base < r.count)
            .map(|r| FeatureRef {
                mesh: r.mesh,
                feature: id - r.base,
            });

        (Some(result), None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn feature_ref_properties() {
        let f = FeatureRef {
            mesh: 42,
            feature: 7,
        };
        assert_eq!(f.mesh, 42);
        assert_eq!(f.feature, 7);
    }
}
