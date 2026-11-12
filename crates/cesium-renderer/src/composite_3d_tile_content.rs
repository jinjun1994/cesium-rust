//! Composite3DTileContent matching CesiumJS `Scene/Composite3DTileContent.js`.

use cesium_math::{Color, RuntimeError};

use crate::cesium_3d_tile_content_type::Cesium3DTileContentType;
use crate::cesium_3d_tile_style::Cesium3DTileStyleDocument;
use crate::content_metadata::ContentMetadata;
use crate::multiple_3d_tile_content::Cesium3DContentGroup;

/// Supported inner content types inside a Composite (`cmpt`) tile.
#[derive(Clone, Debug)]
pub enum CompositeInnerContent {
    /// Nested composite tile (`cmpt`).
    Composite(Composite3DTileContent),
    /// Leaf tile content (`b3dm`, `i3dm`, `pnts`, `vctr`, `geom`, etc.).
    Leaf(CompositeLeafContent),
}

/// Leaf inner content item inside a Composite (`cmpt`) tile.
#[derive(Clone, Debug)]
pub struct CompositeLeafContent {
    pub magic: String,
    pub content_type: Cesium3DTileContentType,
    pub resource_url: String,
    pub payload: Vec<u8>,
    pub metadata: Option<ContentMetadata>,
    pub group: Option<Cesium3DContentGroup>,
    pub feature_properties_dirty: bool,
    pub debug_enabled: bool,
    pub debug_color: Option<Color>,
    pub style_applied: bool,
    pub ready: bool,
    is_destroyed: bool,
}

impl CompositeLeafContent {
    /// Creates a new leaf inner content item.
    pub fn new(
        magic: String,
        content_type: Cesium3DTileContentType,
        resource_url: String,
        payload: Vec<u8>,
    ) -> Self {
        Self {
            magic,
            content_type,
            resource_url,
            payload,
            metadata: None,
            group: None,
            feature_properties_dirty: false,
            debug_enabled: false,
            debug_color: None,
            style_applied: false,
            ready: true,
            is_destroyed: false,
        }
    }

    pub fn is_destroyed(&self) -> bool {
        self.is_destroyed
    }

    pub fn destroy(&mut self) {
        self.payload.clear();
        self.is_destroyed = true;
    }
}

impl CompositeInnerContent {
    pub fn resource_url(&self) -> &str {
        match self {
            CompositeInnerContent::Composite(c) => &c.resource_url,
            CompositeInnerContent::Leaf(l) => &l.resource_url,
        }
    }

    pub fn metadata(&self) -> Option<&ContentMetadata> {
        match self {
            CompositeInnerContent::Composite(c) => c.metadata(),
            CompositeInnerContent::Leaf(l) => l.metadata.as_ref(),
        }
    }

    pub fn set_metadata(&mut self, metadata: Option<ContentMetadata>) {
        match self {
            CompositeInnerContent::Composite(c) => c.set_metadata(metadata),
            CompositeInnerContent::Leaf(l) => l.metadata = metadata,
        }
    }

    pub fn group(&self) -> Option<&Cesium3DContentGroup> {
        match self {
            CompositeInnerContent::Composite(c) => c.group(),
            CompositeInnerContent::Leaf(l) => l.group.as_ref(),
        }
    }

    pub fn set_group(&mut self, group: Option<Cesium3DContentGroup>) {
        match self {
            CompositeInnerContent::Composite(c) => c.set_group(group),
            CompositeInnerContent::Leaf(l) => l.group = group,
        }
    }

    pub fn feature_properties_dirty(&self) -> bool {
        match self {
            CompositeInnerContent::Composite(c) => c.feature_properties_dirty(),
            CompositeInnerContent::Leaf(l) => l.feature_properties_dirty,
        }
    }

    pub fn set_feature_properties_dirty(&mut self, dirty: bool) {
        match self {
            CompositeInnerContent::Composite(c) => c.set_feature_properties_dirty(dirty),
            CompositeInnerContent::Leaf(l) => l.feature_properties_dirty = dirty,
        }
    }

    pub fn apply_debug_settings(&mut self, enabled: bool, color: Option<Color>) {
        match self {
            CompositeInnerContent::Composite(c) => c.apply_debug_settings(enabled, color),
            CompositeInnerContent::Leaf(l) => {
                l.debug_enabled = enabled;
                l.debug_color = color;
            }
        }
    }

    pub fn apply_style(&mut self, style: Option<&Cesium3DTileStyleDocument>) {
        match self {
            CompositeInnerContent::Composite(c) => c.apply_style(style),
            CompositeInnerContent::Leaf(l) => {
                l.style_applied = style.is_some();
            }
        }
    }

    pub fn update(&mut self) -> bool {
        match self {
            CompositeInnerContent::Composite(c) => c.update(),
            CompositeInnerContent::Leaf(l) => l.ready,
        }
    }

    pub fn is_destroyed(&self) -> bool {
        match self {
            CompositeInnerContent::Composite(c) => c.is_destroyed(),
            CompositeInnerContent::Leaf(l) => l.is_destroyed(),
        }
    }

    pub fn destroy(&mut self) {
        match self {
            CompositeInnerContent::Composite(c) => c.destroy(),
            CompositeInnerContent::Leaf(l) => l.destroy(),
        }
    }
}

/// Represents the contents of a Composite (`cmpt`) tile in a 3D Tiles tileset.
#[derive(Clone, Debug)]
pub struct Composite3DTileContent {
    pub tileset_id: Option<String>,
    pub tile_id: Option<String>,
    pub resource_url: String,
    contents: Vec<CompositeInnerContent>,
    metadata: Option<ContentMetadata>,
    group: Option<Cesium3DContentGroup>,
    ready: bool,
    is_destroyed: bool,
}

impl Composite3DTileContent {
    /// Creates a new `Composite3DTileContent` from pre-parsed inner contents.
    pub fn new(
        tileset_id: Option<String>,
        tile_id: Option<String>,
        resource_url: impl Into<String>,
        contents: Vec<CompositeInnerContent>,
    ) -> Self {
        let ready = contents.iter().all(|c| match c {
            CompositeInnerContent::Composite(comp) => comp.ready(),
            CompositeInnerContent::Leaf(leaf) => leaf.ready,
        });
        Self {
            tileset_id,
            tile_id,
            resource_url: resource_url.into(),
            contents,
            metadata: None,
            group: None,
            ready,
            is_destroyed: false,
        }
    }

    /// Parses a Composite (`cmpt`) tile binary buffer starting at `byte_offset`.
    pub fn from_tile_type(
        tileset_id: Option<String>,
        tile_id: Option<String>,
        resource_url: impl Into<String>,
        bytes: &[u8],
        byte_offset: usize,
    ) -> Result<Self, RuntimeError> {
        let resource_url = resource_url.into();
        let existing_prefix = extract_composite_index(&resource_url);
        Self::parse_at(
            tileset_id,
            tile_id,
            &resource_url,
            bytes,
            byte_offset,
            existing_prefix.as_deref(),
        )
    }

    fn parse_at(
        tileset_id: Option<String>,
        tile_id: Option<String>,
        resource_url: &str,
        bytes: &[u8],
        byte_offset: usize,
        composite_prefix: Option<&str>,
    ) -> Result<Self, RuntimeError> {
        if byte_offset.saturating_add(16) > bytes.len() {
            return Err(RuntimeError::new(
                "Composite tile buffer is too small for 16-byte header.",
            ));
        }

        let magic = &bytes[byte_offset..byte_offset + 4];
        if magic != b"cmpt" {
            let magic_str = String::from_utf8_lossy(magic);
            return Err(RuntimeError::new(format!(
                "Invalid Composite tile magic: {magic_str}"
            )));
        }

        let version = read_u32_le(bytes, byte_offset + 4)?;
        if version != 1 {
            return Err(RuntimeError::new(format!(
                "Only Composite Tile version 1 is supported. Version {version} is not."
            )));
        }

        let _byte_length = read_u32_le(bytes, byte_offset + 8)? as usize;
        let tiles_length = read_u32_le(bytes, byte_offset + 12)? as usize;

        let prefix = match composite_prefix {
            Some(p) if !p.is_empty() => format!("{p}_"),
            _ => String::new(),
        };

        let mut cursor = byte_offset + 16;
        let mut contents = Vec::with_capacity(tiles_length);

        for i in 0..tiles_length {
            if cursor.saturating_add(12) > bytes.len() {
                return Err(RuntimeError::new(
                    "Composite inner tile header extends past end of buffer.",
                ));
            }
            let tile_type_bytes = &bytes[cursor..cursor + 4];
            let tile_type = String::from_utf8_lossy(tile_type_bytes).into_owned();
            let tile_byte_length = read_u32_le(bytes, cursor + 8)? as usize;
            if tile_byte_length < 12 || cursor.saturating_add(tile_byte_length) > bytes.len() {
                return Err(RuntimeError::new(format!(
                    "Composite inner tile {i} has invalid byteLength {tile_byte_length}."
                )));
            }

            let composite_index = format!("{prefix}{i}");
            let child_url = with_composite_index(resource_url, &composite_index);

            let inner = match tile_type.as_str() {
                "cmpt" => {
                    let nested = Self::parse_at(
                        tileset_id.clone(),
                        tile_id.clone(),
                        &child_url,
                        bytes,
                        cursor,
                        Some(&composite_index),
                    )?;
                    CompositeInnerContent::Composite(nested)
                }
                "b3dm" => CompositeInnerContent::Leaf(CompositeLeafContent::new(
                    tile_type,
                    Cesium3DTileContentType::Batched3DModel,
                    child_url,
                    bytes[cursor..cursor + tile_byte_length].to_vec(),
                )),
                "i3dm" => {
                    validate_i3dm_slice(&bytes[cursor..cursor + tile_byte_length])?;
                    CompositeInnerContent::Leaf(CompositeLeafContent::new(
                        tile_type,
                        Cesium3DTileContentType::Instanced3DModel,
                        child_url,
                        bytes[cursor..cursor + tile_byte_length].to_vec(),
                    ))
                }
                "pnts" => CompositeInnerContent::Leaf(CompositeLeafContent::new(
                    tile_type,
                    Cesium3DTileContentType::PointCloud,
                    child_url,
                    bytes[cursor..cursor + tile_byte_length].to_vec(),
                )),
                "vctr" => CompositeInnerContent::Leaf(CompositeLeafContent::new(
                    tile_type,
                    Cesium3DTileContentType::Vector,
                    child_url,
                    bytes[cursor..cursor + tile_byte_length].to_vec(),
                )),
                "geom" => CompositeInnerContent::Leaf(CompositeLeafContent::new(
                    tile_type,
                    Cesium3DTileContentType::Geometry,
                    child_url,
                    bytes[cursor..cursor + tile_byte_length].to_vec(),
                )),
                other => {
                    return Err(RuntimeError::new(format!(
                        "Unknown tile content type, {other}, inside Composite tile"
                    )));
                }
            };

            contents.push(inner);
            cursor += tile_byte_length;
        }

        Ok(Self::new(tileset_id, tile_id, resource_url, contents))
    }

    pub fn feature_properties_dirty(&self) -> bool {
        self.contents
            .iter()
            .any(CompositeInnerContent::feature_properties_dirty)
    }

    pub fn set_feature_properties_dirty(&mut self, value: bool) {
        for c in &mut self.contents {
            c.set_feature_properties_dirty(value);
        }
    }

    /// Always 0 for `Composite3DTileContent`.
    pub fn features_length(&self) -> usize {
        0
    }

    /// Always 0 for `Composite3DTileContent`.
    pub fn points_length(&self) -> usize {
        0
    }

    /// Always 0 for `Composite3DTileContent`.
    pub fn triangles_length(&self) -> usize {
        0
    }

    /// Always 0 for `Composite3DTileContent`.
    pub fn geometry_byte_length(&self) -> usize {
        0
    }

    /// Always 0 for `Composite3DTileContent`.
    pub fn textures_byte_length(&self) -> usize {
        0
    }

    /// Always 0 for `Composite3DTileContent`.
    pub fn batch_table_byte_length(&self) -> usize {
        0
    }

    pub fn inner_contents(&self) -> &[CompositeInnerContent] {
        &self.contents
    }

    pub fn inner_contents_mut(&mut self) -> &mut [CompositeInnerContent] {
        &mut self.contents
    }

    pub fn ready(&self) -> bool {
        self.ready
    }

    pub fn url(&self) -> &str {
        &self.resource_url
    }

    pub fn metadata(&self) -> Option<&ContentMetadata> {
        self.metadata.as_ref()
    }

    /// Stores metadata and propagates it to all inner contents.
    pub fn set_metadata(&mut self, metadata: Option<ContentMetadata>) {
        self.metadata = metadata.clone();
        for c in &mut self.contents {
            c.set_metadata(metadata.clone());
        }
    }

    pub fn batch_table(&self) -> Option<()> {
        None
    }

    pub fn group(&self) -> Option<&Cesium3DContentGroup> {
        self.group.as_ref()
    }

    /// Stores group metadata and propagates it to all inner contents.
    pub fn set_group(&mut self, group: Option<Cesium3DContentGroup>) {
        self.group = group.clone();
        for c in &mut self.contents {
            c.set_group(group.clone());
        }
    }

    pub fn has_property(&self, _batch_id: usize, _name: &str) -> bool {
        false
    }

    pub fn get_feature(&self, _batch_id: usize) -> Option<()> {
        None
    }

    pub fn apply_debug_settings(&mut self, enabled: bool, color: Option<Color>) {
        for c in &mut self.contents {
            c.apply_debug_settings(enabled, color);
        }
    }

    pub fn apply_style(&mut self, style: Option<&Cesium3DTileStyleDocument>) {
        for c in &mut self.contents {
            c.apply_style(style);
        }
    }

    pub fn update(&mut self) -> bool {
        let mut all_ready = true;
        for c in &mut self.contents {
            if !c.update() {
                all_ready = false;
            }
        }
        self.ready = all_ready;
        all_ready
    }

    pub fn is_destroyed(&self) -> bool {
        self.is_destroyed
    }

    pub fn destroy(&mut self) {
        for c in &mut self.contents {
            c.destroy();
        }
        self.contents.clear();
        self.is_destroyed = true;
    }
}

fn read_u32_le(bytes: &[u8], offset: usize) -> Result<u32, RuntimeError> {
    let slice: [u8; 4] = bytes
        .get(offset..offset + 4)
        .and_then(|s| s.try_into().ok())
        .ok_or_else(|| RuntimeError::new("Unexpected end of Composite tile buffer."))?;
    Ok(u32::from_le_bytes(slice))
}

fn validate_i3dm_slice(bytes: &[u8]) -> Result<(), RuntimeError> {
    // i3dm header is 32 bytes: magic(4), version(4), byteLength(4), ftJson(4), ftBin(4), btJson(4), btBin(4), gltfFormat(4)
    if bytes.len() < 32 {
        return Err(RuntimeError::new(
            "Invalid i3dm inner tile: header smaller than 32 bytes.",
        ));
    }
    let version = read_u32_le(bytes, 4)?;
    if version != 1 {
        return Err(RuntimeError::new(format!(
            "Invalid i3dm version {version}."
        )));
    }
    let gltf_format = read_u32_le(bytes, 28)?;
    if gltf_format > 1 {
        return Err(RuntimeError::new(format!(
            "Invalid i3dm gltfFormat {gltf_format}."
        )));
    }
    if gltf_format == 0 {
        let ft_json_len = read_u32_le(bytes, 12)? as usize;
        let ft_bin_len = read_u32_le(bytes, 16)? as usize;
        let bt_json_len = read_u32_le(bytes, 20)? as usize;
        let bt_bin_len = read_u32_le(bytes, 24)? as usize;
        let uri_offset = 32usize
            .saturating_add(ft_json_len)
            .saturating_add(ft_bin_len)
            .saturating_add(bt_json_len)
            .saturating_add(bt_bin_len);
        let uri_bytes = bytes.get(uri_offset..).unwrap_or(&[]);
        let uri = String::from_utf8_lossy(uri_bytes).trim().to_string();
        if uri.is_empty() || uri == "invalid" {
            return Err(RuntimeError::new(format!(
                "Invalid external glTF URI in i3dm tile: {uri}"
            )));
        }
    }
    Ok(())
}

fn extract_composite_index(url: &str) -> Option<String> {
    let (_, query) = url.split_once('?')?;
    for pair in query.split('&') {
        if let Some((k, v)) = pair.split_once('=') {
            if k == "compositeIndex" {
                return Some(v.to_string());
            }
        }
    }
    None
}

fn with_composite_index(url: &str, composite_index: &str) -> String {
    let base = url.split('?').next().unwrap_or(url);
    format!("{base}?compositeIndex={composite_index}")
}
