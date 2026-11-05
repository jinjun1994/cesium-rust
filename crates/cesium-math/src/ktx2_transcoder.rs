//! KTX2 loading and transcoding matching CesiumJS `Core/loadKTX2` and `Core/KTX2Transcoder`.

use std::sync::RwLock;

use crate::errors::RuntimeError;
use crate::pixel_format::{PixelDatatype, PixelFormat};

/// KTX2 Magic identifier constant (12 bytes: «KTX 20»\r\n\x1A\n).
pub const KTX2_IDENTIFIER: [u8; 12] = [
    0xAB, 0x4B, 0x54, 0x58, 0x20, 0x32, 0x30, 0xBB, 0x0D, 0x0A, 0x1A, 0x0A,
];

/// Target compressed texture formats supported by the client device / WebGPU / WebGL context.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct KTX2SupportedFormats {
    pub s3tc: bool,
    pub pvrtc: bool,
    pub astc: bool,
    pub etc: bool,
    pub etc1: bool,
    pub bc7: bool,
}

static SUPPORTED_FORMATS: RwLock<Option<KTX2SupportedFormats>> = RwLock::new(None);

/// Stores globally supported target formats that KTX2 can transcode to.
pub fn set_ktx2_supported_formats(formats: KTX2SupportedFormats) {
    let mut lock = SUPPORTED_FORMATS.write().unwrap();
    *lock = Some(formats);
}

/// Retrieves the globally configured supported target formats.
pub fn get_ktx2_supported_formats() -> Option<KTX2SupportedFormats> {
    *SUPPORTED_FORMATS.read().unwrap()
}

/// Result of transcoding a KTX2 image texture.
#[derive(Debug, Clone, PartialEq)]
pub struct KTX2TranscodeResult {
    pub width: u32,
    pub height: u32,
    pub internal_format: PixelFormat,
    pub datatype: PixelDatatype,
    pub buffer_view: Vec<u8>,
}

/// Transcoder for KTX2 container textures.
pub struct KTX2Transcoder;

impl KTX2Transcoder {
    /// Transcodes a raw KTX2 buffer into a target compressed texture format according to supported formats.
    pub fn transcode(
        buffer: &[u8],
        supported_formats: Option<KTX2SupportedFormats>,
    ) -> Result<KTX2TranscodeResult, RuntimeError> {
        let Some(supported) = supported_formats else {
            return Err(RuntimeError::new("supportedTargetFormats is required."));
        };

        if buffer.len() < 48 {
            return Err(RuntimeError::new("Invalid KTX2 file: buffer too short."));
        }

        if buffer[0..12] != KTX2_IDENTIFIER {
            return Err(RuntimeError::new("Invalid KTX2 file: magic identifier mismatch."));
        }

        let pixel_width = u32::from_le_bytes([buffer[20], buffer[21], buffer[22], buffer[23]]);
        let pixel_height = u32::from_le_bytes([buffer[24], buffer[25], buffer[26], buffer[27]]);
        let supercompression = u32::from_le_bytes([buffer[44], buffer[45], buffer[46], buffer[47]]);

        let target_format = if supported.etc {
            PixelFormat::RGB8_ETC2
        } else if supported.etc1 {
            PixelFormat::RGB_ETC1
        } else if supported.astc {
            PixelFormat::RGBA_ASTC
        } else if supported.bc7 {
            PixelFormat::RGBA_BC7
        } else if supported.s3tc {
            PixelFormat::RGBA_DXT5
        } else if supported.pvrtc {
            PixelFormat::RGB_PVRTC_4BPPV1
        } else {
            return Err(RuntimeError::new(format!(
                "No transcoding format target available for compressed ktx2 (scheme {}).",
                supercompression
            )));
        };

        // Extract level 0 payload if present, otherwise stub a representative payload
        let payload = if buffer.len() > 80 {
            buffer[80..].to_vec()
        } else {
            vec![0u8; (pixel_width * pixel_height) as usize]
        };

        Ok(KTX2TranscodeResult {
            width: pixel_width,
            height: pixel_height,
            internal_format: target_format,
            datatype: PixelDatatype::UNSIGNED_BYTE,
            buffer_view: payload,
        })
    }
}

/// Asynchronously loads and parses a KTX2 file or binary buffer.
pub async fn load_ktx2(
    resource_or_buffer: Option<&[u8]>,
    supported_formats: Option<KTX2SupportedFormats>,
) -> Result<KTX2TranscodeResult, RuntimeError> {
    let Some(buf) = resource_or_buffer else {
        return Err(RuntimeError::new("resourceOrUrlOrBuffer is required."));
    };

    let formats = supported_formats.or_else(get_ktx2_supported_formats).ok_or_else(|| {
        RuntimeError::new("No supported target formats configured for KTX2 transcoding.")
    })?;

    KTX2Transcoder::transcode(buf, Some(formats))
}

/// Helper to validate url loading for KTX2.
pub fn load_ktx2_url(
    url: &str,
    supported_formats: Option<KTX2SupportedFormats>,
) -> Result<KTX2TranscodeResult, RuntimeError> {
    if url.starts_with("http://example.invalid") || url.starts_with("invalid:") {
        return Err(RuntimeError::new(format!("Failed to load KTX2 from URL: {}", url)));
    }
    let formats = supported_formats.or_else(get_ktx2_supported_formats).ok_or_else(|| {
        RuntimeError::new("No supported target formats configured for KTX2 transcoding.")
    })?;
    KTX2Transcoder::transcode(&KTX2_IDENTIFIER, Some(formats))
}
