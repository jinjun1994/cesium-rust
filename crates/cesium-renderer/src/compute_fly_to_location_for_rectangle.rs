//! computeFlyToLocationForRectangle matching CesiumJS `Scene/computeFlyToLocationForRectangle.js` (Apache-2.0).

use cesium_math::{Cartesian3, Cartographic, Ellipsoid, GeoRectangle, SceneMode};
use glam::DVec3;

/// Extracts the 5 sample points (center, southeast, southwest, northeast, northwest) for a rectangle.
pub fn sample_cartographics_for_rectangle(rect: &GeoRectangle) -> [Cartographic; 5] {
    let center = rect.center();
    let southeast = Cartographic::new(rect.east, rect.south, 0.0);
    let southwest = Cartographic::new(rect.west, rect.south, 0.0);
    let northeast = Cartographic::new(rect.east, rect.north, 0.0);
    let northwest = Cartographic::new(rect.west, rect.north, 0.0);

    [center, southeast, southwest, northeast, northwest]
}

/// Computes the final camera location to view a rectangle adjusted for terrain height.
pub fn compute_fly_to_location_for_rectangle(
    rectangle: GeoRectangle,
    scene_mode: SceneMode,
    camera_coords: Cartesian3,
    ellipsoid: Ellipsoid,
    sampled_terrain_heights: Option<&[Option<f64>]>,
) -> Cartographic {
    let mut position_without_terrain = match scene_mode {
        SceneMode::Scene3D | SceneMode::Morphing => {
            ellipsoid
                .cartesian_to_cartographic(DVec3::new(
                    camera_coords.x,
                    camera_coords.y,
                    camera_coords.z,
                ))
                .unwrap_or_else(|| {
                    let center = rectangle.center();
                    Cartographic::new(center.longitude, center.latitude, camera_coords.z)
                })
        }
        SceneMode::ColumbusView | SceneMode::Scene2D => {
            // For 2D / Columbus View map projection unproject:
            // x -> lon * radius, y -> lat * radius, z -> height
            let radius = ellipsoid.maximum_radius();
            let lon = if radius > 0.0 { camera_coords.x / radius } else { 0.0 };
            let lat = if radius > 0.0 { camera_coords.y / radius } else { 0.0 };
            Cartographic::new(lon, lat, camera_coords.z)
        }
    };

    if scene_mode == SceneMode::Scene2D {
        return position_without_terrain;
    }

    if let Some(heights) = sampled_terrain_heights {
        let mut max_height = -f64::MAX;
        let mut height_found = false;

        for h in heights.iter().flatten() {
            height_found = true;
            if *h > max_height {
                max_height = *h;
            }
        }

        if height_found {
            position_without_terrain.height += max_height;
        }
    }

    position_without_terrain
}
