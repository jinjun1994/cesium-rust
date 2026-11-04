//! Index datatype representations matching CesiumJS `Core/IndexDatatype`.
//! Adapted from CesiumJS (Apache-2.0).

use crate::core_math::SIXTY_FOUR_KILOBYTES;
use crate::errors::DeveloperError;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum IndexDatatype {
    UnsignedByte = 5121,
    UnsignedShort = 5123,
    UnsignedInt = 5125,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum IndexBuffer {
    U16(Vec<u16>),
    U32(Vec<u32>),
}

impl IndexBuffer {
    pub fn bytes_per_element(&self) -> usize {
        match self {
            Self::U16(_) => 2,
            Self::U32(_) => 4,
        }
    }

    pub fn len(&self) -> usize {
        match self {
            Self::U16(v) => v.len(),
            Self::U32(v) => v.len(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn get(&self, index: usize) -> Option<u32> {
        match self {
            Self::U16(v) => v.get(index).map(|&x| x as u32),
            Self::U32(v) => v.get(index).copied(),
        }
    }
}

impl IndexDatatype {
    pub const UNSIGNED_BYTE: Self = Self::UnsignedByte;
    pub const UNSIGNED_SHORT: Self = Self::UnsignedShort;
    pub const UNSIGNED_INT: Self = Self::UnsignedInt;

    pub const fn as_gl_enum(&self) -> u32 {
        *self as u32
    }

    pub const fn get_size_in_bytes(&self) -> usize {
        match self {
            Self::UnsignedByte => 1,
            Self::UnsignedShort => 2,
            Self::UnsignedInt => 4,
        }
    }

    pub fn from_size_in_bytes(size: usize) -> Result<Self, DeveloperError> {
        match size {
            1 => Ok(Self::UnsignedByte),
            2 => Ok(Self::UnsignedShort),
            4 => Ok(Self::UnsignedInt),
            _ => Err(DeveloperError::new(
                "Size in bytes cannot be mapped to an IndexDatatype",
            )),
        }
    }

    pub fn from_gl_enum(code: u32) -> Option<Self> {
        match code {
            5121 => Some(Self::UnsignedByte),
            5123 => Some(Self::UnsignedShort),
            5125 => Some(Self::UnsignedInt),
            _ => None,
        }
    }

    pub fn validate(code: u32) -> bool {
        Self::from_gl_enum(code).is_some()
    }

    pub fn create_typed_array(number_of_vertices: usize, length: usize) -> IndexBuffer {
        if number_of_vertices >= SIXTY_FOUR_KILOBYTES {
            IndexBuffer::U32(vec![0u32; length])
        } else {
            IndexBuffer::U16(vec![0u16; length])
        }
    }

    pub fn create_typed_array_from_buffer(
        number_of_vertices: usize,
        buffer: &[u8],
        byte_offset: usize,
        length: usize,
    ) -> Result<IndexBuffer, DeveloperError> {
        if number_of_vertices >= SIXTY_FOUR_KILOBYTES {
            let elem_size = 4;
            let required_bytes = byte_offset.saturating_add(length.saturating_mul(elem_size));
            if buffer.len() < required_bytes {
                return Err(DeveloperError::new("Buffer too short for requested slice."));
            }
            let mut out = Vec::with_capacity(length);
            for i in 0..length {
                let start = byte_offset + i * elem_size;
                let slice: [u8; 4] = buffer[start..start + 4]
                    .try_into()
                    .map_err(|_| DeveloperError::new("Invalid byte slice for u32"))?;
                out.push(u32::from_ne_bytes(slice));
            }
            Ok(IndexBuffer::U32(out))
        } else {
            let elem_size = 2;
            let required_bytes = byte_offset.saturating_add(length.saturating_mul(elem_size));
            if buffer.len() < required_bytes {
                return Err(DeveloperError::new("Buffer too short for requested slice."));
            }
            let mut out = Vec::with_capacity(length);
            for i in 0..length {
                let start = byte_offset + i * elem_size;
                let slice: [u8; 2] = buffer[start..start + 2]
                    .try_into()
                    .map_err(|_| DeveloperError::new("Invalid byte slice for u16"))?;
                out.push(u16::from_ne_bytes(slice));
            }
            Ok(IndexBuffer::U16(out))
        }
    }
}
