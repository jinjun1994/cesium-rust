use std::sync::OnceLock;

use glam::DVec3;

use crate::cartesian::Cartesian3;
use crate::ellipsoid::Ellipsoid;
use crate::Cartographic;

pub const S2_MAX_LEVEL: i32 = 30;
const S2_LIMIT_IJ: i64 = 1 << S2_MAX_LEVEL;
const S2_MAX_SITI: f64 = (1_u64 << (S2_MAX_LEVEL + 1)) as f64;
const S2_POSITION_BITS: u32 = 2 * (S2_MAX_LEVEL as u32) + 1;
const S2_LOOKUP_BITS: usize = 4;

const S2_SWAP_MASK: usize = 1;
const S2_INVERT_MASK: usize = 2;

const S2_POSITION_TO_IJ: [[usize; 4]; 4] = [
    [0, 1, 3, 2],
    [0, 2, 3, 1],
    [3, 2, 0, 1],
    [3, 1, 0, 2],
];

const S2_POSITION_TO_ORIENTATION_MASK: [usize; 4] = [
    S2_SWAP_MASK,
    0,
    0,
    S2_SWAP_MASK | S2_INVERT_MASK,
];

static S2_LOOKUP_TABLES: OnceLock<(Vec<u16>, Vec<u16>)> = OnceLock::new();

fn init_lookup_tables() -> (Vec<u16>, Vec<u16>) {
    let mut lookup_positions = vec![0_u16; 1024];
    let mut lookup_ij = vec![0_u16; 1024];

    #[allow(clippy::too_many_arguments)]
    fn generate_lookup_cell(
        level: usize,
        mut i: usize,
        mut j: usize,
        original_orientation: usize,
        mut position: usize,
        orientation: usize,
        lookup_positions: &mut [u16],
        lookup_ij: &mut [u16],
    ) {
        if level == S2_LOOKUP_BITS {
            let ij = (i << S2_LOOKUP_BITS) + j;
            lookup_positions[(ij << 2) + original_orientation] =
                ((position << 2) + orientation) as u16;
            lookup_ij[(position << 2) + original_orientation] =
                ((ij << 2) + orientation) as u16;
        } else {
            let next_level = level + 1;
            i <<= 1;
            j <<= 1;
            position <<= 2;
            let r = S2_POSITION_TO_IJ[orientation];
            generate_lookup_cell(
                next_level,
                i + (r[0] >> 1),
                j + (r[0] & 1),
                original_orientation,
                position,
                orientation ^ S2_POSITION_TO_ORIENTATION_MASK[0],
                lookup_positions,
                lookup_ij,
            );
            generate_lookup_cell(
                next_level,
                i + (r[1] >> 1),
                j + (r[1] & 1),
                original_orientation,
                position + 1,
                orientation ^ S2_POSITION_TO_ORIENTATION_MASK[1],
                lookup_positions,
                lookup_ij,
            );
            generate_lookup_cell(
                next_level,
                i + (r[2] >> 1),
                j + (r[2] & 1),
                original_orientation,
                position + 2,
                orientation ^ S2_POSITION_TO_ORIENTATION_MASK[2],
                lookup_positions,
                lookup_ij,
            );
            generate_lookup_cell(
                next_level,
                i + (r[3] >> 1),
                j + (r[3] & 1),
                original_orientation,
                position + 3,
                orientation ^ S2_POSITION_TO_ORIENTATION_MASK[3],
                lookup_positions,
                lookup_ij,
            );
        }
    }

    generate_lookup_cell(
        0,
        0,
        0,
        0,
        0,
        0,
        &mut lookup_positions,
        &mut lookup_ij,
    );
    generate_lookup_cell(
        0,
        0,
        0,
        S2_SWAP_MASK,
        0,
        S2_SWAP_MASK,
        &mut lookup_positions,
        &mut lookup_ij,
    );
    generate_lookup_cell(
        0,
        0,
        0,
        S2_INVERT_MASK,
        0,
        S2_INVERT_MASK,
        &mut lookup_positions,
        &mut lookup_ij,
    );
    generate_lookup_cell(
        0,
        0,
        0,
        S2_SWAP_MASK | S2_INVERT_MASK,
        0,
        S2_SWAP_MASK | S2_INVERT_MASK,
        &mut lookup_positions,
        &mut lookup_ij,
    );

    (lookup_positions, lookup_ij)
}

#[inline]
fn get_lookup_ij() -> &'static [u16] {
    let tables = S2_LOOKUP_TABLES.get_or_init(init_lookup_tables);
    &tables.1
}

#[inline]
fn lsb(cell_id: u64) -> u64 {
    cell_id & (!cell_id).wrapping_add(1)
}

#[inline]
fn lsb_for_level(level: i32) -> u64 {
    1_u64 << (2 * (S2_MAX_LEVEL - level))
}

/// Represents a cell in the S2 geometry hierarchy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct S2Cell {
    cell_id: u64,
    level: i32,
}

impl S2Cell {
    /// Constructs an S2Cell from a 64-bit cell ID.
    pub fn from_id(cell_id: u64) -> Result<Self, &'static str> {
        if !Self::is_valid_id(cell_id) {
            return Err("cell ID is invalid");
        }
        let level = Self::get_level(cell_id)?;
        Ok(Self { cell_id, level })
    }

    /// Constructs an S2Cell from a hexadecimal token string.
    pub fn from_token(token: &str) -> Result<Self, &'static str> {
        if !Self::is_valid_token(token) {
            return Err("token is invalid");
        }
        let cell_id = Self::get_id_from_token(token)?;
        Self::from_id(cell_id)
    }

    /// Validates an S2 cell ID.
    pub fn is_valid_id(cell_id: u64) -> bool {
        if cell_id == 0 {
            return false;
        }
        if (cell_id >> S2_POSITION_BITS) > 5 {
            return false;
        }
        let lowest_set_bit = lsb(cell_id);
        (lowest_set_bit & 0x1555_5555_5555_5555) != 0
    }

    /// Validates an S2 cell token.
    pub fn is_valid_token(token: &str) -> bool {
        if token.is_empty() || token.len() > 16 {
            return false;
        }
        if !token.chars().all(|c| c.is_ascii_hexdigit()) {
            return false;
        }
        if let Ok(id) = Self::get_id_from_token(token) {
            Self::is_valid_id(id)
        } else {
            false
        }
    }

    /// Converts a token into its corresponding 64-bit cell ID.
    pub fn get_id_from_token(token: &str) -> Result<u64, &'static str> {
        if token.is_empty() || token.len() > 16 {
            return Err("token length must be between 1 and 16");
        }
        if !token.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err("token must be valid hexadecimal");
        }
        let mut padded = token.to_string();
        while padded.len() < 16 {
            padded.push('0');
        }
        u64::from_str_radix(&padded, 16).map_err(|_| "failed to parse token")
    }

    /// Converts a 64-bit cell ID to a hexadecimal token.
    pub fn get_token_from_id(cell_id: u64) -> String {
        let trailing_zero_bits = cell_id.trailing_zeros();
        let trailing_zero_hex_chars = (trailing_zero_bits / 4) as usize;
        let hex_str = format!("{:x}", cell_id);
        let trimmed_hex = hex_str.trim_end_matches('0');
        let total_chars = trimmed_hex.len();
        let prefix_zeros = 16_usize.saturating_sub(trailing_zero_hex_chars + total_chars);
        format!("{}{}", "0".repeat(prefix_zeros), trimmed_hex)
    }

    /// Returns the level of the cell ID [0-30].
    pub fn get_level(cell_id: u64) -> Result<i32, &'static str> {
        if !Self::is_valid_id(cell_id) {
            return Err("cell ID is invalid");
        }
        let lsb_position = cell_id.trailing_zeros() as i32;
        Ok(S2_MAX_LEVEL - (lsb_position >> 1))
    }

    /// Creates an S2Cell from its face, Hilbert position, and level.
    pub fn from_face_position_level(
        face: i32,
        position: u64,
        level: i32,
    ) -> Result<Self, &'static str> {
        if !(0..=5).contains(&face) {
            return Err("face must be in the range [0-5]");
        }
        if !(0..=S2_MAX_LEVEL).contains(&level) {
            return Err("level must be in the range [0-30]");
        }
        if level < 30 {
            let max_position = 1_u64 << (2 * level);
            if position >= max_position {
                return Err("position exceeds maximum allowed for level");
            }
        }

        let face_bits = (face as u64) << S2_POSITION_BITS;
        let shift = 2 * (S2_MAX_LEVEL - level);
        let position_bits = position << (shift + 1);
        let sentinel_bit = 1_u64 << shift;
        let cell_id = face_bits | position_bits | sentinel_bit;
        Self::from_id(cell_id)
    }

    #[inline]
    pub fn cell_id(&self) -> u64 {
        self.cell_id
    }

    #[inline]
    pub fn level(&self) -> i32 {
        self.level
    }

    /// Gets the parent cell.
    pub fn get_parent(&self) -> Result<Self, &'static str> {
        if self.level == 0 {
            return Err("cannot get parent of root cell");
        }
        let new_lsb = lsb(self.cell_id) << 2;
        let parent_id = (self.cell_id & (!new_lsb).wrapping_add(1)) | new_lsb;
        Self::from_id(parent_id)
    }

    /// Gets the parent cell at a given level.
    pub fn get_parent_at_level(&self, level: i32) -> Result<Self, &'static str> {
        if self.level == 0 || level < 0 || self.level < level {
            return Err("cannot get parent at invalid level");
        }
        let new_lsb = lsb_for_level(level);
        let parent_id = (self.cell_id & (!new_lsb).wrapping_add(1)) | new_lsb;
        Self::from_id(parent_id)
    }

    /// Gets the child cell at index [0-3].
    pub fn get_child(&self, index: usize) -> Result<Self, &'static str> {
        if index > 3 {
            return Err("child index must be in the range [0-3]");
        }
        if self.level == 30 {
            return Err("cannot get child of leaf cell");
        }
        let new_lsb = lsb(self.cell_id) >> 2;
        let offset = 2 * (index as i64) + 1 - 4;
        let child_id = ((self.cell_id as i128) + (offset as i128) * (new_lsb as i128)) as u64;
        Self::from_id(child_id)
    }

    /// Returns the geodetic center of the cell on the specified ellipsoid (default WGS84).
    pub fn get_center(&self, ellipsoid: Option<&Ellipsoid>) -> Cartesian3 {
        let el = ellipsoid.copied().unwrap_or(Ellipsoid::WGS84);
        let (face, si, ti) = convert_cell_id_to_face_si_ti(self.cell_id, self.level);
        let xyz = convert_face_si_ti_to_xyz(face, si, ti).normalize();
        let carto = Cartographic::from_radians(
            xyz.y.atan2(xyz.x),
            xyz.z.clamp(-1.0, 1.0).asin(),
        );
        let pos = el.cartographic_to_cartesian(carto).unwrap_or(DVec3::ZERO);
        Cartesian3::new(pos.x, pos.y, pos.z)
    }

    /// Returns a vertex of the cell (indices 0..=3 in CCW order) on the specified ellipsoid (default WGS84).
    pub fn get_vertex(&self, index: usize, ellipsoid: Option<&Ellipsoid>) -> Result<Cartesian3, &'static str> {
        if index > 3 {
            return Err("vertex index must be in the range [0-3]");
        }
        let el = ellipsoid.copied().unwrap_or(Ellipsoid::WGS84);
        let (face, i, j) = convert_cell_id_to_face_ij(self.cell_id);
        let uv = convert_ij_level_to_bound_uv([i, j], self.level);
        let y = (index >> 1) & 1;
        let x = y ^ (index & 1);
        let xyz = convert_face_uv_to_xyz(face, uv[0][x], uv[1][y]).normalize();
        let carto = Cartographic::from_radians(
            xyz.y.atan2(xyz.x),
            xyz.z.clamp(-1.0, 1.0).asin(),
        );
        let pos = el.cartographic_to_cartesian(carto).unwrap_or(DVec3::ZERO);
        Ok(Cartesian3::new(pos.x, pos.y, pos.z))
    }
}

fn convert_cell_id_to_face_si_ti(cell_id: u64, level: i32) -> (i32, i64, i64) {
    let (face, i, j) = convert_cell_id_to_face_ij(cell_id);
    let is_leaf = level == 30;
    let should_correct = !is_leaf && (((i as u64) ^ (cell_id >> 2)) & 1) != 0;
    let correction = if is_leaf {
        1
    } else if should_correct {
        2
    } else {
        0
    };
    let si = (i << 1) + correction;
    let ti = (j << 1) + correction;
    (face, si, ti)
}

fn convert_cell_id_to_face_ij(cell_id: u64) -> (i32, i64, i64) {
    let face = (cell_id >> S2_POSITION_BITS) as i32;
    let mut bits = (face as usize) & S2_SWAP_MASK;
    let lookup_mask = (1_i64 << S2_LOOKUP_BITS) - 1;
    let lookup_ij = get_lookup_ij();

    let mut i = 0_i64;
    let mut j = 0_i64;

    for k in (0..=7).rev() {
        let number_of_bits = if k == 7 {
            S2_MAX_LEVEL as usize - 7 * S2_LOOKUP_BITS
        } else {
            S2_LOOKUP_BITS
        };
        let extract_mask = (1_u64 << (2 * number_of_bits)) - 1;
        let shift = k * 2 * S2_LOOKUP_BITS + 1;
        bits += (((cell_id >> shift) & extract_mask) as usize) << 2;

        bits = lookup_ij[bits] as usize;

        let offset = k * S2_LOOKUP_BITS;
        i += ((bits >> (S2_LOOKUP_BITS + 2)) as i64) << offset;
        j += (((bits >> 2) as i64) & lookup_mask) << offset;

        bits &= S2_SWAP_MASK | S2_INVERT_MASK;
    }

    (face, i, j)
}

fn convert_face_si_ti_to_xyz(face: i32, si: i64, ti: i64) -> DVec3 {
    let s = (si as f64) / S2_MAX_SITI;
    let t = (ti as f64) / S2_MAX_SITI;
    let u = convert_st_to_uv(s);
    let v = convert_st_to_uv(t);
    convert_face_uv_to_xyz(face, u, v)
}

fn convert_face_uv_to_xyz(face: i32, u: f64, v: f64) -> DVec3 {
    match face {
        0 => DVec3::new(1.0, u, v),
        1 => DVec3::new(-u, 1.0, v),
        2 => DVec3::new(-u, -v, 1.0),
        3 => DVec3::new(-1.0, -v, -u),
        4 => DVec3::new(v, -1.0, -u),
        _ => DVec3::new(v, u, -1.0),
    }
}

fn convert_st_to_uv(s: f64) -> f64 {
    if s >= 0.5 {
        (1.0 / 3.0) * (4.0 * s * s - 1.0)
    } else {
        (1.0 / 3.0) * (1.0 - 4.0 * (1.0 - s) * (1.0 - s))
    }
}

fn convert_ij_level_to_bound_uv(ij: [i64; 2], level: i32) -> [[f64; 2]; 2] {
    let mut result = [[0.0; 2]; 2];
    let cell_size = 1_i64 << (S2_MAX_LEVEL - level);
    for d in 0..2 {
        let ij_low = ij[d] & -cell_size;
        let ij_high = ij_low + cell_size;
        result[d][0] = convert_st_to_uv((ij_low as f64) / (S2_LIMIT_IJ as f64));
        result[d][1] = convert_st_to_uv((ij_high as f64) / (S2_LIMIT_IJ as f64));
    }
    result
}
