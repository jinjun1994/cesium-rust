use cesium_math::{DeveloperError, RuntimeError};

/// Initialization options for an [`ImplicitAvailabilityBitstream`].
#[derive(Debug, Clone, Default)]
pub struct ImplicitAvailabilityBitstreamOptions {
    /// The length of the bitstream in bits.
    pub length_bits: Option<usize>,
    /// A single boolean value indicating the value of all bits if constant.
    pub constant: Option<bool>,
    /// Raw byte buffer storing the bitstream.
    pub bitstream: Option<Vec<u8>>,
    /// Pre-counted number of 1-bits in the bitstream.
    pub available_count: Option<usize>,
    /// If true and available_count is not specified, it is computed from the bitstream.
    pub compute_available_count_enabled: bool,
}

/// Availability bitstream representation (constant or byte buffer) with bit-level addressing and population counting.
#[derive(Debug, Clone, PartialEq)]
pub struct ImplicitAvailabilityBitstream {
    length_bits: usize,
    available_count: Option<usize>,
    constant: Option<bool>,
    bitstream: Option<Vec<u8>>,
}

impl Default for ImplicitAvailabilityBitstream {
    fn default() -> Self {
        Self::constant(false, 0)
    }
}

impl ImplicitAvailabilityBitstream {
    /// Creates a new `ImplicitAvailabilityBitstream` from options.
    pub fn new(options: ImplicitAvailabilityBitstreamOptions) -> Result<Self, RuntimeError> {
        let length_bits = options
            .length_bits
            .ok_or_else(|| RuntimeError::new("options.lengthBits is required"))?;

        let mut available_count = options.available_count;
        let constant = options.constant;
        let bitstream = options.bitstream;

        if let Some(c) = constant {
            available_count = Some(if c { length_bits } else { 0 });
        } else {
            let stream = bitstream
                .as_ref()
                .ok_or_else(|| RuntimeError::new("Either constant or bitstream must be defined"))?;
            let expected_length = length_bits.div_ceil(8);
            if stream.len() != expected_length {
                return Err(RuntimeError::new(format!(
                    "Availability bitstream must be exactly {expected_length} bytes long to store {length_bits} bits. Actual bitstream was {} bytes long.",
                    stream.len()
                )));
            }

            if available_count.is_none() && options.compute_available_count_enabled {
                available_count = Some(count_1_bits(stream, length_bits));
            }
        }

        Ok(Self {
            length_bits,
            available_count,
            constant,
            bitstream,
        })
    }

    /// Creates a constant availability bitstream (e.g. constant true/false for length bits).
    pub fn constant(val: bool, length_bits: usize) -> Self {
        Self {
            length_bits,
            available_count: Some(if val { length_bits } else { 0 }),
            constant: Some(val),
            bitstream: None,
        }
    }

    /// Creates a bitstream from raw bytes with bit length.
    pub fn from_bytes(bytes: Vec<u8>, length_bits: usize) -> Self {
        Self {
            length_bits,
            available_count: None,
            constant: None,
            bitstream: Some(bytes),
        }
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
        Self {
            length_bits: bit_length,
            available_count: None,
            constant: None,
            bitstream: Some(bytes),
        }
    }

    /// The length of the bitstream in bits.
    pub fn length_bits(&self) -> usize {
        self.length_bits
    }

    /// Number of available (1) bits, if precomputed or computed.
    pub fn available_count(&self) -> Option<usize> {
        self.available_count
    }

    /// Gets bit at `index`, throwing `DeveloperError` if out of bounds.
    pub fn get_bit(&self, index: usize) -> Result<bool, DeveloperError> {
        if index >= self.length_bits {
            return Err(DeveloperError::new("Bit index out of bounds."));
        }

        if let Some(c) = self.constant {
            return Ok(c);
        }

        if let Some(stream) = &self.bitstream {
            let byte_idx = index >> 3;
            let bit_idx = index % 8;
            Ok(((stream[byte_idx] >> bit_idx) & 1) == 1)
        } else {
            Ok(false)
        }
    }

    /// Non-throwing availability query (returns false if index out of bounds).
    pub fn is_available(&self, index: usize) -> bool {
        self.get_bit(index).unwrap_or(false)
    }
}

fn count_1_bits(bitstream: &[u8], length_bits: usize) -> usize {
    let mut count = 0;
    for i in 0..length_bits {
        let byte_idx = i >> 3;
        let bit_idx = i % 8;
        if ((bitstream[byte_idx] >> bit_idx) & 1) == 1 {
            count += 1;
        }
    }
    count
}
