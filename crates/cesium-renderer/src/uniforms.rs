//! GPU Uniform layouts and paged dynamic uniform buffer allocator.
//!
//! Replaces the hard 4096 object limitation with dynamic paging and single-pass batch writes.

use bytemuck::{Pod, Zeroable};
use std::num::NonZeroU64;

#[repr(C, align(16))]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct GlobalUniform {
    pub projection: [[f32; 4]; 4],
    pub sun_direction_eye: [f32; 4],
    pub viewport_exposure: [f32; 4],
}

#[repr(C, align(16))]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct ObjectUniform {
    pub model_view_relative: [[f32; 4]; 4],
    pub normal_col0: [f32; 4],
    pub normal_col1: [f32; 4],
    pub normal_col2: [f32; 4],
    pub feature_params: [u32; 4],
    pub screen_params: [f32; 4],
}

#[repr(C, align(16))]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct MaterialUniform {
    pub base_color: [f32; 4],
    pub factors: [f32; 4],
    pub emissive_normal: [f32; 4],
    pub params: [f32; 4],
    pub uv_sets: [u32; 4],
    pub extras: [u32; 4],
    pub secondary: [f32; 4],
    pub pattern: [f32; 4],
}

const _: () = assert!(std::mem::size_of::<GlobalUniform>() == 96);
const _: () = assert!(std::mem::size_of::<ObjectUniform>() == 144);
const _: () = assert!(std::mem::size_of::<MaterialUniform>() == 128);

/// Default number of object uniform slots per page (2048 slots = ~512 KB per buffer).
pub const DEFAULT_PAGE_CAPACITY: usize = 2048;

/// A single uniform buffer page with its associated wgpu BindGroup.
pub struct UniformPage {
    pub buffer: wgpu::Buffer,
    pub bind_group: wgpu::BindGroup,
    /// CPU staging scratch buffer for zero-alloc batch packing.
    pub staging: Vec<u8>,
    pub active_count: usize,
}

/// Dynamically expanding paged dynamic uniform manager.
pub struct PagedObjectUniforms {
    pub pages: Vec<UniformPage>,
    pub slot_stride: u64,
    pub page_capacity: usize,
}

impl PagedObjectUniforms {
    pub fn new(
        device: &wgpu::Device,
        layout: &wgpu::BindGroupLayout,
        page_capacity: usize,
    ) -> Self {
        let alignment = device.limits().min_uniform_buffer_offset_alignment as u64;
        let stride = 144u64.div_ceil(alignment) * alignment;
        let capacity = page_capacity.max(256);

        let mut mgr = Self {
            pages: Vec::new(),
            slot_stride: stride,
            page_capacity: capacity,
        };
        // Pre-allocate first page
        mgr.allocate_page(device, layout);
        mgr
    }

    fn allocate_page(&mut self, device: &wgpu::Device, layout: &wgpu::BindGroupLayout) {
        let page_bytes = self.slot_stride * self.page_capacity as u64;
        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(&format!("object uniform page {}", self.pages.len())),
            size: page_bytes,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some(&format!("object uniform page {} group", self.pages.len())),
            layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: &buffer,
                    offset: 0,
                    size: NonZeroU64::new(144),
                }),
            }],
        });

        self.pages.push(UniformPage {
            buffer,
            bind_group,
            staging: vec![0u8; page_bytes as usize],
            active_count: 0,
        });
    }

    /// Ensures that there are enough pages to store `required_slots` objects.
    pub fn ensure_capacity(
        &mut self,
        device: &wgpu::Device,
        layout: &wgpu::BindGroupLayout,
        required_slots: usize,
    ) {
        let pages_needed = required_slots.div_ceil(self.page_capacity).max(1);
        while self.pages.len() < pages_needed {
            self.allocate_page(device, layout);
        }
    }

    /// Resets active counts and clears staging scratch before packing a new frame.
    pub fn begin_frame(&mut self) {
        for page in &mut self.pages {
            page.active_count = 0;
        }
    }

    /// Writes an ObjectUniform into the CPU staging page buffer.
    #[inline]
    pub fn write_slot(&mut self, slot_index: usize, uniform: &ObjectUniform) {
        let page_idx = slot_index / self.page_capacity;
        let page_slot = slot_index % self.page_capacity;
        let offset = page_slot * self.slot_stride as usize;

        let page = &mut self.pages[page_idx];
        let bytes = bytemuck::bytes_of(uniform);
        page.staging[offset..offset + 144].copy_from_slice(bytes);
        if page_slot + 1 > page.active_count {
            page.active_count = page_slot + 1;
        }
    }

    /// Flushes all staging pages with active objects to the GPU in a single `write_buffer` per page.
    pub fn upload_all(&mut self, queue: &wgpu::Queue) {
        for page in &mut self.pages {
            if page.active_count > 0 {
                let bytes_to_upload = page.active_count * self.slot_stride as usize;
                queue.write_buffer(&page.buffer, 0, &page.staging[..bytes_to_upload]);
            }
        }
    }

    /// Returns the (page_index, dynamic_offset) for the given slot.
    #[inline]
    pub fn slot_location(&self, slot_index: usize) -> (usize, u32) {
        let page_idx = slot_index / self.page_capacity;
        let page_slot = slot_index % self.page_capacity;
        let offset = (page_slot as u64 * self.slot_stride) as u32;
        (page_idx, offset)
    }

    /// Total capacity across all currently allocated pages.
    pub fn total_capacity(&self) -> usize {
        self.pages.len() * self.page_capacity
    }

    /// Total GPU buffer memory allocated for all pages in bytes.
    pub fn total_gpu_bytes(&self) -> usize {
        self.pages.len() * (self.slot_stride * self.page_capacity as u64) as usize
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uniform_sizes_and_alignments() {
        assert_eq!(std::mem::size_of::<GlobalUniform>(), 96);
        assert_eq!(std::mem::size_of::<ObjectUniform>(), 144);
        assert_eq!(std::mem::size_of::<MaterialUniform>(), 128);
    }

    #[test]
    fn slot_location_computation() {
        // Mock stride of 256 bytes, capacity of 1024
        let paged = PagedObjectUniforms {
            pages: Vec::new(),
            slot_stride: 256,
            page_capacity: 1024,
        };

        let (p0, off0) = paged.slot_location(0);
        assert_eq!(p0, 0);
        assert_eq!(off0, 0);

        let (p1, off1) = paged.slot_location(1);
        assert_eq!(p1, 0);
        assert_eq!(off1, 256);

        let (p1023, off1023) = paged.slot_location(1023);
        assert_eq!(p1023, 0);
        assert_eq!(off1023, 1023 * 256);

        let (p1024, off1024) = paged.slot_location(1024);
        assert_eq!(p1024, 1);
        assert_eq!(off1024, 0);

        let (p3000, off3000) = paged.slot_location(3000);
        assert_eq!(p3000, 2);
        assert_eq!(off3000, (3000 - 2048) * 256);
    }
}
