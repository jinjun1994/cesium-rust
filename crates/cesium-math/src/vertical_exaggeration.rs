use crate::{Cartesian3, Ellipsoid};

/// Functions for computing vertically-exaggerated heights and positions.
pub struct VerticalExaggeration;

impl VerticalExaggeration {
    /// Scales a height relative to an offset.
    ///
    /// * `height` - The input height.
    /// * `scale` - A scalar used to exaggerate the terrain. If 1.0, there is no effect.
    /// * `relative_height` - The height relative to which terrain is exaggerated.
    #[inline]
    pub fn get_height(height: f64, scale: f64, relative_height: f64) -> f64 {
        (height - relative_height) * scale + relative_height
    }

    /// Scales a Cartesian3 position by vertical exaggeration.
    ///
    /// * `position` - The original Cartesian3 position.
    /// * `ellipsoid` - The reference ellipsoid.
    /// * `vertical_exaggeration` - Scale factor. If 1.0, there is no effect.
    /// * `relative_height` - Reference height for exaggeration.
    pub fn get_position(
        position: &Cartesian3,
        ellipsoid: &Ellipsoid,
        vertical_exaggeration: f64,
        relative_height: f64,
    ) -> Cartesian3 {
        let carto = ellipsoid.cartesian_to_cartographic((*position).into());
        if let Some(carto) = carto {
            let new_height = Self::get_height(carto.height, vertical_exaggeration, relative_height);
            Cartesian3::from_radians(
                carto.longitude,
                carto.latitude,
                Some(new_height),
                Some(ellipsoid),
            )
            .unwrap_or(*position)
        } else {
            *position
        }
    }
}
