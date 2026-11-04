// Morton Order (aka Z-Order Curve) helper functions.
// Interleaves bits of coordinates for spatial indexing.

fn insert_one_spacing(mut v: u32) -> u32 {
    v = (v ^ (v << 8)) & 0x00ff00ff;
    v = (v ^ (v << 4)) & 0x0f0f0f0f;
    v = (v ^ (v << 2)) & 0x33333333;
    v = (v ^ (v << 1)) & 0x55555555;
    v
}

fn insert_two_spacing(mut v: u32) -> u32 {
    v = (v ^ (v << 16)) & 0x030000ff;
    v = (v ^ (v << 8)) & 0x0300f00f;
    v = (v ^ (v << 4)) & 0x030c30c3;
    v = (v ^ (v << 2)) & 0x09249249;
    v
}

fn remove_one_spacing(mut v: u32) -> u32 {
    v &= 0x55555555;
    v = (v ^ (v >> 1)) & 0x33333333;
    v = (v ^ (v >> 2)) & 0x0f0f0f0f;
    v = (v ^ (v >> 4)) & 0x00ff00ff;
    v = (v ^ (v >> 8)) & 0x0000ffff;
    v
}

fn remove_two_spacing(mut v: u32) -> u32 {
    v &= 0x09249249;
    v = (v ^ (v >> 2)) & 0x030c30c3;
    v = (v ^ (v >> 4)) & 0x0300f00f;
    v = (v ^ (v >> 8)) & 0xff0000ff;
    v = (v ^ (v >> 16)) & 0x000003ff;
    v
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MortonOrder;

impl MortonOrder {
    /// Computes the Morton index from 2D coordinates.
    /// Inputs must be 16-bit unsigned integers [0, 65535].
    pub fn encode_2d(x: u32, y: u32) -> Result<u32, &'static str> {
        if x > 65535 || y > 65535 {
            return Err("inputs must be 16-bit unsigned integers");
        }
        Ok(insert_one_spacing(x) | (insert_one_spacing(y) << 1))
    }

    /// Computes 2D coordinates from a 32-bit Morton index.
    pub fn decode_2d(morton_index: u32) -> [u32; 2] {
        [
            remove_one_spacing(morton_index),
            remove_one_spacing(morton_index >> 1),
        ]
    }

    /// Computes the Morton index from 3D coordinates.
    /// Inputs must be 10-bit unsigned integers [0, 1023].
    pub fn encode_3d(x: u32, y: u32, z: u32) -> Result<u32, &'static str> {
        if x > 1023 || y > 1023 || z > 1023 {
            return Err("inputs must be 10-bit unsigned integers");
        }
        Ok(insert_two_spacing(x)
            | (insert_two_spacing(y) << 1)
            | (insert_two_spacing(z) << 2))
    }

    /// Computes 3D coordinates from a 30-bit Morton index [0, 1073741823].
    pub fn decode_3d(morton_index: u32) -> Result<[u32; 3], &'static str> {
        if morton_index > 1073741823 {
            return Err("input must be a 30-bit unsigned integer");
        }
        Ok([
            remove_two_spacing(morton_index),
            remove_two_spacing(morton_index >> 1),
            remove_two_spacing(morton_index >> 2),
        ])
    }
}
