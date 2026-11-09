use cesium_math::{Cartesian3, Cartographic, Ellipsoid};
use cesium_renderer::heightmap_tessellator::{
    ComputeVerticesOptions, HeightmapNativeRectangle, HeightmapStructure, HeightmapTessellator,
};

#[test]
fn throws_when_required_parameters_are_missing() {
    let dummy_heightmap = [1.0, 2.0, 3.0, 4.0];
    let native_rect = HeightmapNativeRectangle::new(10.0, 20.0, 20.0, 30.0);

    // Missing heightmap
    assert!(HeightmapTessellator::compute_vertices(ComputeVerticesOptions {
        heightmap: None,
        width: Some(2),
        height: Some(2),
        skirt_height: Some(10.0),
        native_rectangle: Some(native_rect),
        rectangle: None,
        is_geographic: None,
        relative_to_center: None,
        ellipsoid: None,
        structure: None,
        exaggeration: None,
        exaggeration_relative_height: None,
        include_web_mercator_t: None,
    })
    .is_err());

    // Missing width
    assert!(HeightmapTessellator::compute_vertices(ComputeVerticesOptions {
        heightmap: Some(&dummy_heightmap),
        width: None,
        height: Some(2),
        skirt_height: Some(10.0),
        native_rectangle: Some(native_rect),
        rectangle: None,
        is_geographic: None,
        relative_to_center: None,
        ellipsoid: None,
        structure: None,
        exaggeration: None,
        exaggeration_relative_height: None,
        include_web_mercator_t: None,
    })
    .is_err());

    // Missing native_rectangle
    assert!(HeightmapTessellator::compute_vertices(ComputeVerticesOptions {
        heightmap: Some(&dummy_heightmap),
        width: Some(2),
        height: Some(2),
        skirt_height: Some(10.0),
        native_rectangle: None,
        rectangle: None,
        is_geographic: None,
        relative_to_center: None,
        ellipsoid: None,
        structure: None,
        exaggeration: None,
        exaggeration_relative_height: None,
        include_web_mercator_t: None,
    })
    .is_err());

    // Missing skirt_height
    assert!(HeightmapTessellator::compute_vertices(ComputeVerticesOptions {
        heightmap: Some(&dummy_heightmap),
        width: Some(2),
        height: Some(2),
        skirt_height: None,
        native_rectangle: Some(native_rect),
        rectangle: None,
        is_geographic: None,
        relative_to_center: None,
        ellipsoid: None,
        structure: None,
        exaggeration: None,
        exaggeration_relative_height: None,
        include_web_mercator_t: None,
    })
    .is_err());
}

fn lerp(p0: f64, p1: f64, t: f64) -> f64 {
    p0 + t * (p1 - p0)
}

#[test]
fn creates_mesh_without_skirt() {
    let width = 3;
    let height = 3;
    let heightmap = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0];
    let native_rect = HeightmapNativeRectangle::new(10.0, 30.0, 20.0, 40.0);

    let results = HeightmapTessellator::compute_vertices(ComputeVerticesOptions {
        heightmap: Some(&heightmap),
        width: Some(width),
        height: Some(height),
        skirt_height: Some(0.0),
        native_rectangle: Some(native_rect),
        rectangle: None,
        is_geographic: Some(true),
        relative_to_center: None,
        ellipsoid: Some(Ellipsoid::WGS84),
        structure: None,
        exaggeration: None,
        exaggeration_relative_height: None,
        include_web_mercator_t: None,
    })
    .unwrap();

    let vertices = results.vertices;
    assert_eq!(vertices.len(), width * height * 6);

    let ellipsoid = Ellipsoid::WGS84;

    for j in 0..height {
        let lat_deg = lerp(native_rect.north, native_rect.south, j as f64 / (height - 1) as f64);
        let latitude = lat_deg.to_radians();

        for i in 0..width {
            let lon_deg = lerp(native_rect.west, native_rect.east, i as f64 / (width - 1) as f64);
            let longitude = lon_deg.to_radians();

            let height_sample = heightmap[j * width + i];

            let mut carto = Cartographic::from_radians(longitude, latitude);
            carto.height = height_sample;
            let expected_pos = ellipsoid.cartographic_to_cartesian(carto).unwrap();

            let idx = (j * width + i) * 6;
            let actual_pos = Cartesian3::new(
                vertices[idx] as f64,
                vertices[idx + 1] as f64,
                vertices[idx + 2] as f64,
            );

            assert!((actual_pos.x - expected_pos.x).abs() < 1.0);
            assert!((actual_pos.y - expected_pos.y).abs() < 1.0);
            assert!((actual_pos.z - expected_pos.z).abs() < 1.0);
            assert!((vertices[idx + 3] as f64 - height_sample).abs() < 1e-4);
        }
    }
}

#[test]
fn creates_mesh_with_skirt() {
    let width = 3;
    let height = 3;
    let heightmap = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0];
    let native_rect = HeightmapNativeRectangle::new(10.0, 30.0, 20.0, 40.0);
    let skirt_height = 10.0;

    let results = HeightmapTessellator::compute_vertices(ComputeVerticesOptions {
        heightmap: Some(&heightmap),
        width: Some(width),
        height: Some(height),
        skirt_height: Some(skirt_height),
        native_rectangle: Some(native_rect),
        rectangle: None,
        is_geographic: Some(true),
        relative_to_center: None,
        ellipsoid: Some(Ellipsoid::WGS84),
        structure: None,
        exaggeration: None,
        exaggeration_relative_height: None,
        include_web_mercator_t: None,
    })
    .unwrap();

    let grid_vertex_count = width * height;
    let edge_vertex_count = width * 2 + height * 2;
    let total_vertex_count = grid_vertex_count + edge_vertex_count;

    assert_eq!(results.vertices.len(), total_vertex_count * 6);
}

#[test]
fn supports_multi_element_little_endian_heights() {
    let width = 3;
    let height = 3;
    let heightmap = [
        1.0, 2.0, 100.0, 3.0, 4.0, 100.0, 5.0, 6.0, 100.0, 7.0, 8.0, 100.0, 9.0, 10.0, 100.0,
        11.0, 12.0, 100.0, 13.0, 14.0, 100.0, 15.0, 16.0, 100.0, 17.0, 18.0, 100.0,
    ];
    let native_rect = HeightmapNativeRectangle::new(10.0, 30.0, 20.0, 40.0);

    let structure = HeightmapStructure {
        stride: 3,
        elements_per_height: 2,
        element_multiplier: 10.0,
        is_big_endian: false,
        ..Default::default()
    };

    let results = HeightmapTessellator::compute_vertices(ComputeVerticesOptions {
        heightmap: Some(&heightmap),
        width: Some(width),
        height: Some(height),
        skirt_height: Some(0.0),
        native_rectangle: Some(native_rect),
        rectangle: None,
        is_geographic: Some(true),
        relative_to_center: None,
        ellipsoid: Some(Ellipsoid::WGS84),
        structure: Some(structure),
        exaggeration: None,
        exaggeration_relative_height: None,
        include_web_mercator_t: None,
    })
    .unwrap();

    let vertices = results.vertices;
    for j in 0..height {
        for i in 0..width {
            let sample_idx = (j * width + i) * 3;
            let expected_h = heightmap[sample_idx] + heightmap[sample_idx + 1] * 10.0;
            let vertex_idx = (j * width + i) * 6;
            assert!((vertices[vertex_idx + 3] as f64 - expected_h).abs() < 1e-4);
        }
    }
}

#[test]
fn supports_multi_element_big_endian_heights() {
    let width = 3;
    let height = 3;
    let heightmap = [
        1.0, 2.0, 100.0, 3.0, 4.0, 100.0, 5.0, 6.0, 100.0, 7.0, 8.0, 100.0, 9.0, 10.0, 100.0,
        11.0, 12.0, 100.0, 13.0, 14.0, 100.0, 15.0, 16.0, 100.0, 17.0, 18.0, 100.0,
    ];
    let native_rect = HeightmapNativeRectangle::new(10.0, 30.0, 20.0, 40.0);

    let structure = HeightmapStructure {
        stride: 3,
        elements_per_height: 2,
        element_multiplier: 10.0,
        is_big_endian: true,
        ..Default::default()
    };

    let results = HeightmapTessellator::compute_vertices(ComputeVerticesOptions {
        heightmap: Some(&heightmap),
        width: Some(width),
        height: Some(height),
        skirt_height: Some(0.0),
        native_rectangle: Some(native_rect),
        rectangle: None,
        is_geographic: Some(true),
        relative_to_center: None,
        ellipsoid: Some(Ellipsoid::WGS84),
        structure: Some(structure),
        exaggeration: None,
        exaggeration_relative_height: None,
        include_web_mercator_t: None,
    })
    .unwrap();

    let vertices = results.vertices;
    for j in 0..height {
        for i in 0..width {
            let sample_idx = (j * width + i) * 3;
            let expected_h = heightmap[sample_idx] * 10.0 + heightmap[sample_idx + 1];
            let vertex_idx = (j * width + i) * 6;
            assert!((vertices[vertex_idx + 3] as f64 - expected_h).abs() < 1e-4);
        }
    }
}

#[test]
fn creates_mesh_with_web_mercator_projection() {
    let width = 3;
    let height = 3;
    let heightmap = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0];
    let native_rect = HeightmapNativeRectangle::new(100000.0, 200000.0, 300000.0, 400000.0);

    let results = HeightmapTessellator::compute_vertices(ComputeVerticesOptions {
        heightmap: Some(&heightmap),
        width: Some(width),
        height: Some(height),
        skirt_height: Some(0.0),
        native_rectangle: Some(native_rect),
        rectangle: None,
        is_geographic: Some(false),
        relative_to_center: None,
        ellipsoid: Some(Ellipsoid::WGS84),
        structure: None,
        exaggeration: None,
        exaggeration_relative_height: None,
        include_web_mercator_t: Some(true),
    })
    .unwrap();

    assert_eq!(results.vertices.len(), width * height * 7); // stride 7 when web mercator T included
    assert!(results.bounding_sphere_3d.radius > 0.0);
}
