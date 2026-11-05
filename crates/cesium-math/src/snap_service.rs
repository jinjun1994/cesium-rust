use crate::DeveloperError;

/// Base trait / contract for snapping services.
pub trait SnapService {
    const DEFAULT_SNAP_APERTURE: Option<f64> = None;

    fn snap(&self) -> Result<(), DeveloperError> {
        Err(DeveloperError::new(
            "SnapService.snap is not implemented directly on the base trait.",
        ))
    }
}
