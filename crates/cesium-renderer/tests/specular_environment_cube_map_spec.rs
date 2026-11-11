use cesium_math::Cartesian3;
use cesium_renderer::specular_environment_cube_map::{
    CubeMapDirectionalColors, SpecularEnvironmentCubeMap, KIARA_DIRECTIONAL_COLORS,
};
use cesium_renderer::TextureCache;

#[test]
fn test_is_supported() {
    assert!(SpecularEnvironmentCubeMap::is_supported());
}

#[test]
fn test_correctly_samples_the_given_cube_map_at_all_mip_levels() {
    let url = "./Data/EnvironmentMap/kiara_6_afternoon_2k_ibl.ktx2";
    let mut cube_map = SpecularEnvironmentCubeMap::new(url);

    assert!(!cube_map.ready);
    cube_map.update(None);
    assert!(cube_map.ready);
    assert_eq!(cube_map.maximum_mipmap_level, 7.0);

    let directions = [
        ("positiveX", Cartesian3::new(1.0, 0.0, 0.0)),
        ("negativeX", Cartesian3::new(-1.0, 0.0, 0.0)),
        ("positiveY", Cartesian3::new(0.0, 1.0, 0.0)),
        ("negativeY", Cartesian3::new(0.0, -1.0, 0.0)),
        ("positiveZ", Cartesian3::new(0.0, 0.0, 1.0)),
        ("negativeZ", Cartesian3::new(0.0, 0.0, -1.0)),
    ];

    for (mip_level, expected_colors) in KIARA_DIRECTIONAL_COLORS.iter().enumerate() {
        for (name, dir) in &directions {
            let sampled = cube_map.sample(*dir, mip_level as f64);
            let expected = match *name {
                "positiveX" => expected_colors.positive_x,
                "negativeX" => expected_colors.negative_x,
                "positiveY" => expected_colors.positive_y,
                "negativeY" => expected_colors.negative_y,
                "positiveZ" => expected_colors.positive_z,
                "negativeZ" => expected_colors.negative_z,
                _ => unreachable!(),
            };

            for c in 0..4 {
                assert!(
                    (sampled[c] as i32 - expected[c] as i32).abs() <= 1,
                    "Mismatch on mip {mip_level} face {name} component {c}: sampled {:?}, expected {:?}",
                    sampled,
                    expected
                );
            }
        }
    }
}

#[test]
fn test_caches_cubemap_textures() {
    let texture_cache = TextureCache::new();
    let url = "./Data/EnvironmentMap/kiara_6_afternoon_2k_ibl.ktx2";

    let mut cube_map1 = SpecularEnvironmentCubeMap::new(url);
    cube_map1.update(Some(&texture_cache));
    assert!(cube_map1.ready);
    assert!(cube_map1.texture_id.is_some());

    let mut cube_map2 = SpecularEnvironmentCubeMap::new(url);
    cube_map2.update(Some(&texture_cache));
    assert!(cube_map2.ready);
    assert_eq!(cube_map1.texture_id, cube_map2.texture_id);
    assert_eq!(
        cube_map1.maximum_mipmap_level,
        cube_map2.maximum_mipmap_level
    );

    cube_map2.destroy();
    assert!(cube_map2.is_destroyed);

    cube_map1.destroy();
    assert!(cube_map1.is_destroyed);
}

#[test]
fn test_raises_error_when_environment_map_fails_to_load() {
    let mut cube_map = SpecularEnvironmentCubeMap::new("http://invalid.url");
    cube_map.update(None);

    assert!(!cube_map.ready);
    assert!(cube_map.error_message.is_some());
    assert!(cube_map
        .error_message
        .as_ref()
        .unwrap()
        .contains("Failed to load"));
}

#[test]
fn test_directional_colors_structure_clone_eq() {
    let colors = CubeMapDirectionalColors {
        positive_x: [1, 2, 3, 255],
        negative_x: [4, 5, 6, 255],
        positive_y: [7, 8, 9, 255],
        negative_y: [10, 11, 12, 255],
        positive_z: [13, 14, 15, 255],
        negative_z: [16, 17, 18, 255],
    };

    let cloned = colors.clone();
    assert_eq!(colors, cloned);
}
