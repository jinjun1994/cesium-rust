/// Availability bitstream representation (constant 0/1 or raw bit buffer).
#[derive(Debug, Clone, PartialEq)]
pub enum ImplicitAvailabilityBitstream {
    Constant(u8),
    Bitstream {
        bytes: Vec<u8>,
        length_bits: usize,
    },
}

impl Default for ImplicitAvailabilityBitstream {
    fn default() -> Self {
        Self::Constant(0)
    }
}

impl ImplicitAvailabilityBitstream {
    /// Creates a constant availability bitstream (e.g. 0 or 1).
    pub fn constant(val: u8) -> Self {
        Self::Constant(val)
    }

    /// Creates a bitstream from raw bytes with bit length.
    pub fn from_bytes(bytes: Vec<u8>, length_bits: usize) -> Self {
        Self::Bitstream { bytes, length_bits }
    }

    /// Creates a bitstream from a binary string descriptor (e.g. "11010").
    /// In 3D Tiles 1.1 bitstream semantics, the first character is bit 0 (stored in LSB).
    pub fn from_descriptor(descriptor: &str, bit_length: usize) -> Self {
        let mut bytes = vec![0u8; bit_length.div_ceil(8)];
        for (i, ch) in descriptor.chars().enumerate().take(bit_length) {
            if ch == '1' {
                let byte_idx = i / 8;
                let bit_idx = i % 8;
                if byte_idx < bytes.len() {
                    bytes[byte_idx] |= 1 << bit_idx;
                }
            }
        }
        Self::Bitstream { bytes, length_bits: bit_length }
    }

    /// Queries availability of the bit at `index`.
    pub fn is_available(&self, index: usize) -> bool {
        match self {
            Self::Constant(v) => *v != 0,
            Self::Bitstream { bytes, length_bits } => {
                if index >= *length_bits {
                    return false;
                }
                let byte_idx = index / 8;
                let bit_idx = index % 8;
                if byte_idx < bytes.len() {
                    (bytes[byte_idx] & (1 << bit_idx)) != 0
                } else {
                    false
                }
            }
        }
    }
}
