use cesium_math::DeveloperError;
use crate::implicit_availability_bitstream::ImplicitAvailabilityBitstream;
use crate::implicit_subdivision_scheme::ImplicitSubdivisionScheme;

/// Subtree payload holding availability bitstreams and metadata.
#[derive(Debug, Clone, Default)]
pub struct ImplicitSubtree {
    pub subdivision_scheme: ImplicitSubdivisionScheme,
    pub subtree_levels: u32,
    pub tile_availability: ImplicitAvailabilityBitstream,
    pub content_availability: Vec<ImplicitAvailabilityBitstream>,
    pub child_subtree_availability: ImplicitAvailabilityBitstream,
}

impl ImplicitSubtree {
    /// Creates a new `ImplicitSubtree`.
    pub fn new(
        subdivision_scheme: ImplicitSubdivisionScheme,
        subtree_levels: u32,
        tile_availability: ImplicitAvailabilityBitstream,
        content_availability: Vec<ImplicitAvailabilityBitstream>,
        child_subtree_availability: ImplicitAvailabilityBitstream,
    ) -> Self {
        Self {
            subdivision_scheme,
            subtree_levels,
            tile_availability,
            content_availability,
            child_subtree_availability,
        }
    }

    /// Parses an `ImplicitSubtree` from JSON value and optional buffer.
    pub fn from_json_value(
        json: &serde_json::Value,
        _buffer: Option<&[u8]>,
        subdivision_scheme: ImplicitSubdivisionScheme,
        subtree_levels: u32,
    ) -> Result<Self, DeveloperError> {
        let tile_avail = parse_availability_entry(json.get("tileAvailability"))?;
        let child_subtree_avail = parse_availability_entry(json.get("childSubtreeAvailability"))?;

        let mut content_avails = Vec::new();
        if let Some(ca) = json.get("contentAvailability") {
            if let Some(arr) = ca.as_array() {
                for item in arr {
                    content_avails.push(parse_availability_entry(Some(item))?);
                }
            } else {
                content_avails.push(parse_availability_entry(Some(ca))?);
            }
        }

        Ok(Self {
            subdivision_scheme,
            subtree_levels,
            tile_availability: tile_avail,
            content_availability: content_avails,
            child_subtree_availability: child_subtree_avail,
        })
    }

    /// Whether tile at `relative_index` is available in this subtree.
    pub fn is_tile_available(&self, relative_index: usize) -> bool {
        self.tile_availability.is_available(relative_index)
    }

    /// Whether content at `content_index` and `relative_index` is available.
    pub fn is_content_available(&self, content_index: usize, relative_index: usize) -> bool {
        self.content_availability
            .get(content_index)
            .map(|a| a.is_available(relative_index))
            .unwrap_or(false)
    }

    /// Whether child subtree at `relative_child_subtree_index` is available.
    pub fn is_child_subtree_available(&self, relative_child_subtree_index: usize) -> bool {
        self.child_subtree_availability.is_available(relative_child_subtree_index)
    }
}

fn parse_availability_entry(
    entry: Option<&serde_json::Value>,
) -> Result<ImplicitAvailabilityBitstream, DeveloperError> {
    let obj = match entry {
        Some(v) => v,
        None => return Ok(ImplicitAvailabilityBitstream::default()),
    };

    if let Some(constant) = obj.get("constant").and_then(|c| c.as_u64()) {
        let bit_len = obj.get("bitLength").and_then(|b| b.as_u64()).unwrap_or(1024) as usize;
        return Ok(ImplicitAvailabilityBitstream::constant(constant != 0, bit_len));
    }

    if let Some(descriptor) = obj.get("descriptor").and_then(|d| d.as_str()) {
        let bit_len = obj
            .get("bitLength")
            .and_then(|b| b.as_u64())
            .unwrap_or(descriptor.len() as u64) as usize;
        return Ok(ImplicitAvailabilityBitstream::from_descriptor(
            descriptor, bit_len,
        ));
    }

    Ok(ImplicitAvailabilityBitstream::default())
}
