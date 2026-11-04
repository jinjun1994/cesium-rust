use std::sync::atomic::{AtomicU64, Ordering};

/// Creates a Globally Unique Identifier (GUID / UUID v4) string conforming to RFC 4122.
pub fn create_guid() -> String {
    static COUNTER: AtomicU64 = AtomicU64::new(0x1234_5678_9ABC_DEF0);
    let count = COUNTER.fetch_add(0x9E37_79B9_7F4A_7C15, Ordering::Relaxed);
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);

    let mut bytes = [0u8; 16];
    bytes[0..8].copy_from_slice(&(now as u64 ^ count).to_be_bytes());
    bytes[8..16].copy_from_slice(&(count.rotate_left(31) ^ (now >> 64) as u64).to_be_bytes());
    // UUID v4: bits 4-7 of time_hi_and_version set to 0100
    bytes[6] = (bytes[6] & 0x0f) | 0x40;
    // Variant: bits 6-7 of clock_seq_hi_and_reserved set to 10
    bytes[8] = (bytes[8] & 0x3f) | 0x80;

    format!(
        "{:02x}{:02x}{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
        bytes[0], bytes[1], bytes[2], bytes[3],
        bytes[4], bytes[5],
        bytes[6], bytes[7],
        bytes[8], bytes[9],
        bytes[10], bytes[11], bytes[12], bytes[13], bytes[14], bytes[15]
    )
}
