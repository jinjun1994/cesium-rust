use cesium_formats::{parse_pnts, DecodeLimits, Pnts};
use cesium_math::{DeveloperError, RuntimeError};

/// Parser for 3D Tiles Point Cloud (.pnts) tiles matching CesiumJS `Scene/PntsParser.js`.
#[derive(Debug)]
pub struct PntsParser;

impl PntsParser {
    /// Parses a Point Cloud (.pnts) array buffer.
    ///
    /// # Errors
    /// Returns `DeveloperError` if `buffer` is `None`.
    /// Returns `RuntimeError` if header, feature table, or required semantics are invalid.
    pub fn parse<'a>(
        buffer: Option<&'a [u8]>,
    ) -> Result<Pnts<'a>, Box<dyn std::error::Error + Send + Sync>> {
        let buf = buffer.ok_or_else(|| DeveloperError::new("arrayBuffer is required."))?;
        let parsed = parse_pnts(buf, &DecodeLimits::default())
            .map_err(|e| RuntimeError::new(format!("RuntimeError: failed to parse pnts: {e}")))?;
        Ok(parsed)
    }
}
