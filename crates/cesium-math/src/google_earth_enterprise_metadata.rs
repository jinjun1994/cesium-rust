use std::collections::HashMap;

use crate::DeveloperError;

const COMPRESSED_MAGIC: u32 = 0x7468dead;
const COMPRESSED_MAGIC_SWAP: u32 = 0xadde6874;

const CHILDREN_BITMASKS: [u8; 4] = [0x01, 0x02, 0x04, 0x08];
const ANY_CHILD_BITMASK: u8 = 0x0f;
const CACHE_FLAG_BITMASK: u8 = 0x10;
const IMAGE_BITMASK: u8 = 0x40;
const TERRAIN_BITMASK: u8 = 0x80;

/// Decodes data that is received from a Google Earth Enterprise server.
pub fn decode_google_earth_enterprise_data(key: &[u8], data: &mut [u8]) -> Result<(), DeveloperError> {
    let key_len = key.len();
    if key_len == 0 || !key_len.is_multiple_of(4) {
        return Err(DeveloperError::new(
            "The length of key must be greater than 0 and a multiple of 4.",
        ));
    }

    if data.len() < 4 {
        return Ok(());
    }

    let magic = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);
    if magic == COMPRESSED_MAGIC || magic == COMPRESSED_MAGIC_SWAP {
        // Occasionally packets don't come back encoded, so just return
        return Ok(());
    }

    let mut dp = 0usize;
    let dpend = data.len();
    let dpend64 = dpend - (dpend % 8);
    let kpend = key_len;
    let mut kp = 0usize;
    let mut off = 8usize;

    while dp < dpend64 {
        off = (off + 8) % 24;
        kp = off;

        while dp < dpend64 && kp < kpend {
            let d0 = u32::from_le_bytes(data[dp..dp + 4].try_into().unwrap());
            let k0 = u32::from_le_bytes(key[kp..kp + 4].try_into().unwrap());
            data[dp..dp + 4].copy_from_slice(&(d0 ^ k0).to_le_bytes());

            let d1 = u32::from_le_bytes(data[dp + 4..dp + 8].try_into().unwrap());
            let k1 = u32::from_le_bytes(key[kp + 4..kp + 8].try_into().unwrap());
            data[dp + 4..dp + 8].copy_from_slice(&(d1 ^ k1).to_le_bytes());

            dp += 8;
            kp += 24;
        }
    }

    if dp < dpend {
        if kp >= kpend {
            off = (off + 8) % 24;
            kp = off;
        }

        while dp < dpend && kp < kpend {
            data[dp] ^= key[kp];
            dp += 1;
            kp += 1;
        }
    }

    Ok(())
}

/// Contains information about each tile from a Google Earth Enterprise server.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GoogleEarthEnterpriseTileInformation {
    bits: u8,
    pub cnode_version: u32,
    pub imagery_version: u32,
    pub terrain_version: u32,
    pub imagery_provider: u32,
    pub terrain_provider: u32,
    pub ancestor_has_terrain: bool,
    pub terrain_state: Option<u8>,
}

impl GoogleEarthEnterpriseTileInformation {
    pub fn new(
        bits: u8,
        cnode_version: u32,
        imagery_version: u32,
        terrain_version: u32,
        imagery_provider: u32,
        terrain_provider: u32,
    ) -> Self {
        Self {
            bits,
            cnode_version,
            imagery_version,
            terrain_version,
            imagery_provider,
            terrain_provider,
            ancestor_has_terrain: false,
            terrain_state: None,
        }
    }

    pub fn set_parent(&mut self, parent: &GoogleEarthEnterpriseTileInformation) {
        self.ancestor_has_terrain = parent.ancestor_has_terrain || self.has_terrain();
    }

    pub fn has_subtree(&self) -> bool {
        (self.bits & CACHE_FLAG_BITMASK) != 0
    }

    pub fn has_imagery(&self) -> bool {
        (self.bits & IMAGE_BITMASK) != 0
    }

    pub fn has_terrain(&self) -> bool {
        (self.bits & TERRAIN_BITMASK) != 0
    }

    pub fn has_children(&self) -> bool {
        (self.bits & ANY_CHILD_BITMASK) != 0
    }

    pub fn has_child(&self, index: usize) -> bool {
        if index < 4 {
            (self.bits & CHILDREN_BITMASKS[index]) != 0
        } else {
            false
        }
    }

    pub fn get_child_bitmask(&self) -> u8 {
        self.bits & ANY_CHILD_BITMASK
    }

    pub fn bits(&self) -> u8 {
        self.bits
    }
}

/// Provides metadata using the Google Earth Enterprise REST API.
#[derive(Debug, Clone)]
pub struct GoogleEarthEnterpriseMetadata {
    pub url: String,
    pub imagery_present: bool,
    pub proto_imagery: Option<bool>,
    pub terrain_present: bool,
    pub negative_altitude_exponent_bias: i32,
    pub negative_altitude_threshold: f64,
    pub providers: HashMap<u32, String>,
    tile_info: HashMap<String, GoogleEarthEnterpriseTileInformation>,
}

impl GoogleEarthEnterpriseMetadata {
    pub fn new(url: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            imagery_present: true,
            proto_imagery: None,
            terrain_present: true,
            negative_altitude_exponent_bias: 32,
            negative_altitude_threshold: 1e-12, // CesiumMath.EPSILON12
            providers: HashMap::new(),
            tile_info: HashMap::new(),
        }
    }

    pub fn from_url(url: impl Into<String>) -> Result<Self, DeveloperError> {
        let url = url.into();
        if url.trim().is_empty() {
            return Err(DeveloperError::new("url is required."));
        }
        Ok(Self::new(url))
    }

    pub fn tile_xy_to_quad_key(x: u32, y: u32, level: u32) -> String {
        let mut quadkey = String::with_capacity((level + 1) as usize);
        for i in (0..=level).rev() {
            let bitmask = 1 << i;
            let mut digit = 0u8;

            if (y & bitmask) == 0 {
                // Top Row
                digit |= 2;
                if (x & bitmask) == 0 {
                    digit |= 1;
                }
            } else if (x & bitmask) != 0 {
                digit |= 1;
            }

            quadkey.push((b'0' + digit) as char);
        }
        quadkey
    }

    pub fn quad_key_to_tile_xy(quadkey: &str) -> Result<(u32, u32, u32), DeveloperError> {
        if quadkey.is_empty() {
            return Err(DeveloperError::new("quadkey must not be empty."));
        }
        let level = (quadkey.len() - 1) as u32;
        let mut x = 0u32;
        let mut y = 0u32;
        let bytes = quadkey.as_bytes();
        for i in (0..=level).rev() {
            let bitmask = 1 << i;
            let char_byte = bytes[(level - i) as usize];
            let digit = match char_byte {
                b'0'..=b'3' => char_byte - b'0',
                _ => return Err(DeveloperError::new("invalid digit in quadkey")),
            };

            if (digit & 2) != 0 {
                // Top Row
                if (digit & 1) == 0 {
                    x |= bitmask;
                }
            } else {
                y |= bitmask;
                if (digit & 1) != 0 {
                    x |= bitmask;
                }
            }
        }
        Ok((x, y, level))
    }

    pub fn get_tile_information(
        &self,
        x: u32,
        y: u32,
        level: u32,
    ) -> Option<&GoogleEarthEnterpriseTileInformation> {
        let quadkey = Self::tile_xy_to_quad_key(x, y, level);
        self.tile_info.get(&quadkey)
    }

    pub fn get_tile_information_from_quad_key(
        &self,
        quadkey: &str,
    ) -> Option<&GoogleEarthEnterpriseTileInformation> {
        self.tile_info.get(quadkey)
    }

    pub fn set_tile_information(
        &mut self,
        quadkey: impl Into<String>,
        info: GoogleEarthEnterpriseTileInformation,
    ) {
        self.tile_info.insert(quadkey.into(), info);
    }

    pub fn is_valid(&self, quad_key: &str) -> bool {
        if self.tile_info.contains_key(quad_key) {
            return true;
        }

        let mut q = quad_key;
        while q.len() > 1 {
            let parent_key = &q[..q.len() - 1];
            if let Some(parent) = self.tile_info.get(parent_key) {
                let last_digit = (q.as_bytes()[q.len() - 1] - b'0') as usize;
                return parent.has_child(last_digit);
            }
            q = parent_key;
        }

        // If root not even known, default to true or check root "0"
        true
    }

    pub fn tile_info_map(&self) -> &HashMap<String, GoogleEarthEnterpriseTileInformation> {
        &self.tile_info
    }

    pub fn populate_subtree(&mut self, x: u32, y: u32, level: u32) {
        let quadkey = Self::tile_xy_to_quad_key(x, y, level);
        for len in 1..=quadkey.len() {
            let sub = &quadkey[..len];
            self.tile_info.entry(sub.to_string()).or_insert_with(|| {
                GoogleEarthEnterpriseTileInformation::new(0xff, 1, 1, 1, 0, 0)
            });
        }
    }
}
