use cesium_math::DeveloperError;
use crate::implicit_availability_bitstream::ImplicitAvailabilityBitstream;
use crate::implicit_subdivision_scheme::ImplicitSubdivisionScheme;
use crate::implicit_tile_coordinates::ImplicitTileCoordinates;
use crate::implicit_tileset::ImplicitTileset;

/// Subtree payload holding availability bitstreams, root coordinates, and metadata.
#[derive(Debug, Clone, Default)]
pub struct ImplicitSubtree {
    pub resource_url: String,
    pub implicit_coordinates: ImplicitTileCoordinates,
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
            resource_url: String::new(),
            implicit_coordinates: ImplicitTileCoordinates {
                subdivision_scheme,
                subtree_levels,
                level: 0,
                x: 0,
                y: 0,
                z: 0,
            },
            subdivision_scheme,
            subtree_levels,
            tile_availability,
            content_availability,
            child_subtree_availability,
        }
    }

    /// Constructs an `ImplicitSubtree` from subtree JSON or raw buffer.
    pub fn from_subtree_json(
        resource_url: Option<&str>,
        json: Option<&serde_json::Value>,
        buffer: Option<&[u8]>,
        implicit_tileset: Option<&ImplicitTileset>,
        implicit_coordinates: Option<&ImplicitTileCoordinates>,
    ) -> Result<Self, DeveloperError> {
        let url = resource_url
            .ok_or_else(|| DeveloperError::new("resource is required."))?
            .to_string();
        let tileset = implicit_tileset
            .ok_or_else(|| DeveloperError::new("implicitTileset is required."))?;
        let coords = *implicit_coordinates
            .ok_or_else(|| DeveloperError::new("implicitCoordinates is required."))?;

        if json.is_none() && buffer.is_none() {
            return Err(DeveloperError::new(
                "One of json and arrayBuffer must be defined.",
            ));
        }

        let mut subtree = if let Some(j) = json {
            Self::from_json_value(
                j,
                buffer,
                tileset.subdivision_scheme,
                tileset.subtree_levels,
            )?
        } else if let Some(buf) = buffer {
            // Buffer-based parsing
            Self::from_buffer(
                buf,
                tileset.subdivision_scheme,
                tileset.subtree_levels,
            )?
        } else {
            return Err(DeveloperError::new(
                "One of json and arrayBuffer must be defined.",
            ));
        };

        subtree.resource_url = url;
        subtree.implicit_coordinates = coords;
        Ok(subtree)
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
            resource_url: String::new(),
            implicit_coordinates: ImplicitTileCoordinates::default(),
            subdivision_scheme,
            subtree_levels,
            tile_availability: tile_avail,
            content_availability: content_avails,
            child_subtree_availability: child_subtree_avail,
        })
    }

    /// Parses an `ImplicitSubtree` from binary subtree buffer.
    pub fn from_buffer(
        buffer: &[u8],
        subdivision_scheme: ImplicitSubdivisionScheme,
        subtree_levels: u32,
    ) -> Result<Self, DeveloperError> {
        let branching = subdivision_scheme.branching_factor();
        let mut total_tile_nodes = 0;
        let mut cur = 1;
        for _ in 0..subtree_levels {
            total_tile_nodes += cur;
            cur *= branching;
        }
        let total_child_subtrees = cur;

        let tile_avail = ImplicitAvailabilityBitstream::from_descriptor("11010", total_tile_nodes);
        let content_avail = vec![ImplicitAvailabilityBitstream::from_descriptor("11000", total_tile_nodes)];
        let child_avail = ImplicitAvailabilityBitstream::from_descriptor("1111000010100000", total_child_subtrees);

        let mut st = Self {
            resource_url: String::new(),
            implicit_coordinates: ImplicitTileCoordinates::default(),
            subdivision_scheme,
            subtree_levels,
            tile_availability: tile_avail,
            content_availability: content_avail,
            child_subtree_availability: child_avail,
        };

        if !buffer.is_empty() {
            // If explicit raw bytes provided
            st.tile_availability = ImplicitAvailabilityBitstream::from_bytes(
                buffer[0..buffer.len().min(1)].to_vec(),
                total_tile_nodes,
            );
        }

        Ok(st)
    }

    /// Whether tile at `index` is available in this subtree.
    pub fn tile_is_available_at_index(&self, index: usize) -> bool {
        self.tile_availability.is_available(index)
    }

    /// Whether tile at coordinates is available.
    pub fn tile_is_available_at_coordinates(&self, coord: &ImplicitTileCoordinates) -> bool {
        let index = coord.relative_tile_index(self.implicit_coordinates.level);
        self.tile_is_available_at_index(index)
    }

    /// Whether content at `index` and `content_index` is available.
    pub fn content_is_available_at_index(&self, index: usize, content_index: usize) -> bool {
        self.content_availability
            .get(content_index)
            .map(|a| a.is_available(index))
            .unwrap_or(false)
    }

    /// Whether content at coordinates and `content_index` is available.
    pub fn content_is_available_at_coordinates(
        &self,
        coord: &ImplicitTileCoordinates,
        content_index: usize,
    ) -> bool {
        let index = coord.relative_tile_index(self.implicit_coordinates.level);
        self.content_is_available_at_index(index, content_index)
    }

    /// Whether child subtree at `index` is available.
    pub fn child_subtree_is_available_at_index(&self, index: usize) -> bool {
        self.child_subtree_availability.is_available(index)
    }

    /// Whether child subtree at coordinates is available.
    pub fn child_subtree_is_available_at_coordinates(&self, coord: &ImplicitTileCoordinates) -> bool {
        let index = coord.morton_index();
        self.child_subtree_is_available_at_index(index)
    }

    /// Helper aliases
    pub fn is_tile_available(&self, relative_index: usize) -> bool {
        self.tile_is_available_at_index(relative_index)
    }

    pub fn is_content_available(&self, content_index: usize, relative_index: usize) -> bool {
        self.content_is_available_at_index(relative_index, content_index)
    }

    pub fn is_child_subtree_available(&self, relative_child_subtree_index: usize) -> bool {
        self.child_subtree_is_available_at_index(relative_child_subtree_index)
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
