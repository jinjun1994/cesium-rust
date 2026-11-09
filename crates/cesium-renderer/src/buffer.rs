//! GPU Buffer implementations matching CesiumJS `Renderer/Buffer.js` (Apache-2.0).

use cesium_math::{DeveloperError, IndexDatatype};
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_BUFFER_ID: AtomicU64 = AtomicU64::new(1);

/// Buffer usage hints.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub enum BufferUsage {
    #[default]
    StaticDraw,
    StreamDraw,
    DynamicDraw,
}

/// Buffer target type.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BufferTarget {
    VertexBuffer,
    IndexBuffer,
    PixelBuffer,
}

/// Creation options for vertex or pixel buffers.
#[derive(Clone, Debug, Default)]
pub struct BufferOptions {
    pub size_in_bytes: usize,
    pub usage: Option<BufferUsage>,
    pub typed_array: Option<Vec<u8>>,
}

/// Creation options for index buffers.
#[derive(Clone, Debug)]
pub struct IndexBufferOptions {
    pub size_in_bytes: usize,
    pub usage: Option<BufferUsage>,
    pub typed_array: Option<Vec<u8>>,
    pub index_datatype: IndexDatatype,
}

/// Buffer representation storing data, target type, usage, and lifetime.
#[derive(Clone, Debug)]
pub struct Buffer {
    pub id: u64,
    pub target: BufferTarget,
    pub size_in_bytes: usize,
    pub usage: BufferUsage,
    pub data: Vec<u8>,
    pub index_datatype: Option<IndexDatatype>,
    pub is_destroyed: bool,
}

impl PartialEq for Buffer {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Buffer {
    /// Creates a vertex buffer.
    pub fn create_vertex_buffer(options: BufferOptions) -> Result<Self, DeveloperError> {
        Self::create_internal(
            BufferTarget::VertexBuffer,
            options.size_in_bytes,
            options.usage,
            options.typed_array,
            None,
        )
    }

    /// Creates an index buffer.
    pub fn create_index_buffer(options: IndexBufferOptions) -> Result<Self, DeveloperError> {
        Self::create_internal(
            BufferTarget::IndexBuffer,
            options.size_in_bytes,
            options.usage,
            options.typed_array,
            Some(options.index_datatype),
        )
    }

    /// Creates a pixel buffer.
    pub fn create_pixel_buffer(options: BufferOptions) -> Result<Self, DeveloperError> {
        Self::create_internal(
            BufferTarget::PixelBuffer,
            options.size_in_bytes,
            options.usage,
            options.typed_array,
            None,
        )
    }

    fn create_internal(
        target: BufferTarget,
        size_in_bytes: usize,
        usage: Option<BufferUsage>,
        typed_array: Option<Vec<u8>>,
        index_datatype: Option<IndexDatatype>,
    ) -> Result<Self, DeveloperError> {
        let usage = usage.unwrap_or(BufferUsage::StaticDraw);

        // Either typed_array or size_in_bytes must be specified, but not both or neither
        let has_typed_array = typed_array.is_some();
        let has_size = size_in_bytes > 0;

        if !has_typed_array && !has_size {
            return Err(DeveloperError::new(
                "Either options.sizeInBytes or options.typedArray is required, and size must be > 0.",
            ));
        }

        if has_typed_array && has_size {
            return Err(DeveloperError::new(
                "Cannot have both options.sizeInBytes and options.typedArray.",
            ));
        }

        let (final_size, data) = if let Some(arr) = typed_array {
            if arr.is_empty() {
                return Err(DeveloperError::new("typedArray must not be empty."));
            }
            let len = arr.len();
            (len, arr)
        } else {
            (size_in_bytes, vec![0u8; size_in_bytes])
        };

        if target == BufferTarget::IndexBuffer && index_datatype.is_none() {
            return Err(DeveloperError::new(
                "IndexDatatype is required for an index buffer.",
            ));
        }

        let id = NEXT_BUFFER_ID.fetch_add(1, Ordering::Relaxed);

        Ok(Self {
            id,
            target,
            size_in_bytes: final_size,
            usage,
            data,
            index_datatype,
            is_destroyed: false,
        })
    }

    /// Bytes per index element if this is an index buffer.
    pub fn bytes_per_index(&self) -> Option<usize> {
        self.index_datatype.map(|dt| dt.get_size_in_bytes())
    }

    /// Total number of indices if this is an index buffer.
    pub fn number_of_indices(&self) -> Option<usize> {
        self.bytes_per_index().map(|bpi| self.size_in_bytes / bpi)
    }

    /// Copies data from a byte slice into this buffer.
    pub fn copy_from_array_view(
        &mut self,
        array_view: &[u8],
        offset_in_bytes: Option<usize>,
    ) -> Result<(), DeveloperError> {
        if self.is_destroyed {
            return Err(DeveloperError::new(
                "This object was destroyed, i.e., destroy() was called.",
            ));
        }
        if array_view.is_empty() {
            return Err(DeveloperError::new("array_view must not be empty."));
        }
        let offset = offset_in_bytes.unwrap_or(0);
        if offset + array_view.len() > self.size_in_bytes {
            return Err(DeveloperError::new(
                "array_view size with offset exceeds buffer sizeInBytes.",
            ));
        }

        self.data[offset..offset + array_view.len()].copy_from_slice(array_view);
        Ok(())
    }

    /// Reads data from this buffer into `destination`.
    pub fn get_buffer_data(
        &self,
        destination: &mut [u8],
        source_offset: Option<usize>,
        destination_offset: Option<usize>,
        length: Option<usize>,
    ) -> Result<(), DeveloperError> {
        if self.is_destroyed {
            return Err(DeveloperError::new(
                "This object was destroyed, i.e., destroy() was called.",
            ));
        }
        let src_off = source_offset.unwrap_or(0);
        let dst_off = destination_offset.unwrap_or(0);

        if src_off >= self.size_in_bytes {
            return Err(DeveloperError::new(
                "source_offset must be less than buffer size.",
            ));
        }
        if dst_off >= destination.len() {
            return Err(DeveloperError::new(
                "destination_offset must be less than destination length.",
            ));
        }

        let len = length.unwrap_or_else(|| {
            let max_src = self.size_in_bytes - src_off;
            let max_dst = destination.len() - dst_off;
            max_src.min(max_dst)
        });

        if src_off + len > self.size_in_bytes || dst_off + len > destination.len() {
            return Err(DeveloperError::new(
                "Requested copy length exceeds buffer or destination bounds.",
            ));
        }

        destination[dst_off..dst_off + len].copy_from_slice(&self.data[src_off..src_off + len]);
        Ok(())
    }

    /// Reads and returns a copy of all bytes in this buffer.
    pub fn copy_data(&self) -> Vec<u8> {
        self.data.clone()
    }

    /// Copies bytes between buffers.
    pub fn copy_from_buffer(
        &mut self,
        read_buffer: &Buffer,
        read_offset: usize,
        write_offset: usize,
        size_in_bytes: usize,
    ) -> Result<(), DeveloperError> {
        if self.is_destroyed || read_buffer.is_destroyed {
            return Err(DeveloperError::new(
                "This object was destroyed, i.e., destroy() was called.",
            ));
        }

        if (self.target == BufferTarget::IndexBuffer)
            != (read_buffer.target == BufferTarget::IndexBuffer)
        {
            return Err(DeveloperError::new(
                "Cannot copy between an index buffer and a non-index buffer.",
            ));
        }

        if size_in_bytes == 0 {
            return Err(DeveloperError::new("sizeInBytes must be greater than zero."));
        }

        if read_offset + size_in_bytes > read_buffer.size_in_bytes {
            return Err(DeveloperError::new(
                "read_offset + size_in_bytes exceeds read_buffer size.",
            ));
        }

        if write_offset + size_in_bytes > self.size_in_bytes {
            return Err(DeveloperError::new(
                "write_offset + size_in_bytes exceeds write buffer size.",
            ));
        }

        // Check overlapping if source and destination are the exact same buffer
        if self.id == read_buffer.id {
            let read_end = read_offset + size_in_bytes;
            let write_end = write_offset + size_in_bytes;
            let overlaps = !(read_end <= write_offset || write_end <= read_offset);
            if overlaps {
                return Err(DeveloperError::new(
                    "readBuffer and writeBuffer ranges cannot overlap when they are the same buffer.",
                ));
            }
        }

        let slice = read_buffer.data[read_offset..read_offset + size_in_bytes].to_vec();
        self.data[write_offset..write_offset + size_in_bytes].copy_from_slice(&slice);
        Ok(())
    }

    /// Copies bytes within the same buffer, forbidding overlapping source and destination regions.
    pub fn copy_sub_data(
        &mut self,
        read_offset: usize,
        write_offset: usize,
        size_in_bytes: usize,
    ) -> Result<(), DeveloperError> {
        if self.is_destroyed {
            return Err(DeveloperError::new(
                "This object was destroyed, i.e., destroy() was called.",
            ));
        }

        if size_in_bytes == 0 {
            return Err(DeveloperError::new("sizeInBytes must be greater than zero."));
        }

        if read_offset + size_in_bytes > self.size_in_bytes {
            return Err(DeveloperError::new(
                "read_offset + size_in_bytes exceeds buffer size.",
            ));
        }

        if write_offset + size_in_bytes > self.size_in_bytes {
            return Err(DeveloperError::new(
                "write_offset + size_in_bytes exceeds buffer size.",
            ));
        }

        let read_end = read_offset + size_in_bytes;
        let write_end = write_offset + size_in_bytes;
        let overlaps = !(read_end <= write_offset || write_end <= read_offset);
        if overlaps {
            return Err(DeveloperError::new(
                "readBuffer and writeBuffer ranges cannot overlap when they are the same buffer.",
            ));
        }

        let slice = self.data[read_offset..read_offset + size_in_bytes].to_vec();
        self.data[write_offset..write_offset + size_in_bytes].copy_from_slice(&slice);
        Ok(())
    }

    /// Returns whether this buffer is destroyed.
    pub fn is_destroyed(&self) -> bool {
        self.is_destroyed
    }

    /// Destroys this buffer. Throws error if already destroyed.
    pub fn destroy(&mut self) -> Result<(), DeveloperError> {
        if self.is_destroyed {
            return Err(DeveloperError::new(
                "This object was destroyed, i.e., destroy() was called.",
            ));
        }
        self.is_destroyed = true;
        self.data.clear();
        self.data.shrink_to_fit();
        Ok(())
    }
}
