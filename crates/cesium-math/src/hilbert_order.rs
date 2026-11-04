use crate::DeveloperError;

/// Hilbert Order helper functions.
pub struct HilbertOrder;

fn rotate(n: u64, p: &mut (u64, u64), rx: u64, ry: u64) {
    if ry != 0 {
        return;
    }
    if rx == 1 {
        p.0 = n - 1 - p.0;
        p.1 = n - 1 - p.1;
    }
    std::mem::swap(&mut p.0, &mut p.1);
}

impl HilbertOrder {
    /// Computes the Hilbert index at the given level from 2D coordinates.
    pub fn encode_2d(level: u32, x: u64, y: u64) -> Result<u128, DeveloperError> {
        if level < 1 {
            return Err(DeveloperError::new("Hilbert level cannot be less than 1."));
        }
        if level > 31 {
            return Err(DeveloperError::new("Hilbert level too large."));
        }
        let n = 1u64 << level;
        if x >= n || y >= n {
            return Err(DeveloperError::new("Invalid coordinates for given level."));
        }

        let mut p = (x, y);
        let mut index: u128 = 0;
        let mut s = n / 2;

        while s > 0 {
            let rx = if (p.0 & s) > 0 { 1 } else { 0 };
            let ry = if (p.1 & s) > 0 { 1 } else { 0 };
            index += (((3 * rx) ^ ry) as u128) * (s as u128) * (s as u128);
            rotate(n, &mut p, rx, ry);
            s /= 2;
        }

        Ok(index)
    }

    /// Computes the 2D coordinates from the Hilbert index at the given level.
    pub fn decode_2d(level: u32, index: u128) -> Result<(u64, u64), DeveloperError> {
        if level < 1 {
            return Err(DeveloperError::new("Hilbert level cannot be less than 1."));
        }
        if level > 31 {
            return Err(DeveloperError::new("Hilbert level too large."));
        }
        let max_index = 1u128 << (level * 2);
        if index >= max_index {
            return Err(DeveloperError::new(
                "Hilbert index exceeds valid maximum for given level.",
            ));
        }

        let n = 1u64 << level;
        let mut p = (0u64, 0u64);
        let mut s = 1u64;
        let mut t = index;

        while s < n {
            let rx = (t / 2) as u64 & 1;
            let ry = (t ^ (rx as u128)) as u64 & 1;
            rotate(s, &mut p, rx, ry);
            p.0 += s * rx;
            p.1 += s * ry;
            t /= 4;
            s *= 2;
        }

        Ok(p)
    }
}
