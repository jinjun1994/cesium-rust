use std::f64::consts::PI;

use cesium_math::{
    Cartesian3, Cartographic, ComponentDatatype, Ellipsoid, FixedFrameTransforms, GeoRectangle,
    WebMercatorProjection, EPSILON4, EPSILON7,
};
use cesium_renderer::shadow_volume_appearance::{ShadowAppearanceSource, ShadowVolumeAppearance};

const TEST_VS: &str = "in vec3 position3DHigh;\n\
in vec3 position3DLow;\n\
in float batchId;\n\
void main() {\n\
    vec4 position = czm_computePosition();\n\
    gl_Position = czm_depthClamp(czm_modelViewProjectionRelativeToEye * position);\n\
}\n";

#[test]
fn test_spherical_extent_geometry_instance_attributes() {
    let unit_sphere = Ellipsoid::UNIT_SPHERE;
    let projection = WebMercatorProjection::new(unit_sphere).unwrap();
    let large_test_rectangle = GeoRectangle::from_degrees(-45.0, -45.0, 45.0, 45.0);

    let attributes = ShadowVolumeAppearance::get_spherical_extent_geometry_instance_attributes(
        &large_test_rectangle,
        &[0.0, 0.0, 0.0, 1.0, 1.0, 0.0],
        &unit_sphere,
        &projection,
    );

    let spherical_extents = attributes.get("sphericalExtents").unwrap();
    assert_eq!(spherical_extents.component_datatype, ComponentDatatype::Float);
    assert_eq!(spherical_extents.components_per_attribute, 4);
    assert!(!spherical_extents.normalize);

    let val = &spherical_extents.value;
    assert!((val[0] - (-PI / 4.0)).abs() < EPSILON4);
    assert!((val[1] - (-PI / 4.0)).abs() < EPSILON4);
    assert!((val[2] - (1.0 / (PI / 2.0))).abs() < EPSILON4);
    assert!((val[3] - (1.0 / (PI / 2.0))).abs() < EPSILON4);

    let longitude_rotation = attributes.get("longitudeRotation").unwrap();
    assert_eq!(longitude_rotation.component_datatype, ComponentDatatype::Float);
    assert_eq!(longitude_rotation.components_per_attribute, 1);
    assert!(!longitude_rotation.normalize);
    assert!(longitude_rotation.value[0].abs() < EPSILON4);
}

#[test]
fn test_planar_texture_coordinate_attributes_in_3d() {
    let unit_sphere = Ellipsoid::UNIT_SPHERE;
    let projection = WebMercatorProjection::new(unit_sphere).unwrap();
    let small_test_rectangle = GeoRectangle::from_degrees(-0.1, -0.1, 0.1, 0.1);

    let attributes = ShadowVolumeAppearance::get_planar_texture_coordinate_attributes(
        &small_test_rectangle,
        &[0.0, 0.0, 0.0, 1.0, 1.0, 0.0],
        &unit_sphere,
        &projection,
        None,
    )
    .expect("should get planar attributes");

    let sw_high = attributes.get("southWest_HIGH").unwrap();
    let sw_low = attributes.get("southWest_LOW").unwrap();
    let eastward = attributes.get("eastward").unwrap();
    let northward = attributes.get("northward").unwrap();

    assert_eq!(sw_high.components_per_attribute, 3);
    assert_eq!(sw_low.components_per_attribute, 3);
    assert_eq!(eastward.components_per_attribute, 3);
    assert_eq!(northward.components_per_attribute, 3);

    // Using unit sphere, high values should be essentially 0
    assert!(sw_high.value[0].abs() < EPSILON7);
    assert!(sw_high.value[1].abs() < EPSILON7);
    assert!(sw_high.value[2].abs() < EPSILON7);

    // sw_low.x > cos(0.2 deg)
    assert!(sw_low.value[0] > (0.2f64.to_radians()).cos());

    // Verify eastward and northward in ENU frame
    let center = small_test_rectangle.center();
    let center_carto = Cartographic::new(center.longitude, center.latitude, 0.0);
    let p = unit_sphere.cartographic_to_cartesian(center_carto).unwrap();
    let center_cartesian = Cartesian3::new(p.x, p.y, p.z);
    let enu_matrix =
        FixedFrameTransforms::east_north_up_to_fixed_frame(center_cartesian, Some(unit_sphere))
            .unwrap();
    let inverse_enu = enu_matrix.inverse().unwrap();

    let east_vec = Cartesian3::new(eastward.value[0], eastward.value[1], eastward.value[2]);
    let mut east_enu = inverse_enu.multiply_by_point_as_vector(east_vec);
    east_enu = east_enu.normalize();
    assert!((east_enu.x - 1.0).abs() < EPSILON7);
    assert!(east_enu.y.abs() < EPSILON7);
    assert!(east_enu.z.abs() < EPSILON7);

    let north_vec = Cartesian3::new(northward.value[0], northward.value[1], northward.value[2]);
    let mut north_enu = inverse_enu.multiply_by_point_as_vector(north_vec);
    north_enu = north_enu.normalize();
    assert!(north_enu.x.abs() < EPSILON7);
    assert!((north_enu.y - 1.0).abs() < EPSILON7);
    assert!(north_enu.z.abs() < EPSILON7);
}

#[test]
fn test_planes_2d_and_columbus_view_attributes() {
    let unit_sphere = Ellipsoid::UNIT_SPHERE;
    let projection = WebMercatorProjection::new(unit_sphere).unwrap();
    let large_test_rectangle = GeoRectangle::from_degrees(-45.0, -45.0, 45.0, 45.0);

    let attributes = ShadowVolumeAppearance::get_spherical_extent_geometry_instance_attributes(
        &large_test_rectangle,
        &[0.0, 0.0, 0.0, 1.0, 1.0, 0.0],
        &unit_sphere,
        &projection,
    );

    let planes2d_high = attributes.get("planes2D_HIGH").unwrap();
    let planes2d_low = attributes.get("planes2D_LOW").unwrap();

    for v in &planes2d_high.value {
        assert!(v.abs() < EPSILON7);
    }

    let sw_carto = Cartographic::new(-45.0f64.to_radians(), -45.0f64.to_radians(), 0.0);
    let sw_cartesian = projection.project(sw_carto).unwrap();
    let low_val = &planes2d_low.value;
    assert!((low_val[0] - sw_cartesian.x).abs() < EPSILON7);
    assert!((low_val[1] - sw_cartesian.y).abs() < EPSILON7);
    assert!((low_val[2] - (-sw_cartesian.y)).abs() < EPSILON7);
    assert!((low_val[3] - (-sw_cartesian.x)).abs() < EPSILON7);
}

#[test]
fn test_texture_coordinate_rotation_attributes() {
    let unit_sphere = Ellipsoid::UNIT_SPHERE;
    let projection = WebMercatorProjection::new(unit_sphere).unwrap();
    let small_test_rectangle = GeoRectangle::from_degrees(-0.1, -0.1, 0.1, 0.1);

    let attributes = ShadowVolumeAppearance::get_planar_texture_coordinate_attributes(
        &small_test_rectangle,
        &[1.0, 0.0, 0.0, 0.0, 1.0, 1.0],
        &unit_sphere,
        &projection,
        Some(0.0),
    )
    .unwrap();

    let u_max_v_max = attributes.get("uMaxVmax").unwrap();
    assert_eq!(u_max_v_max.components_per_attribute, 4);
    assert_eq!(u_max_v_max.value, vec![0.0, 0.0, 1.0, 1.0]);

    let uv_min = attributes.get("uvMinAndExtents").unwrap();
    assert_eq!(uv_min.components_per_attribute, 4);
    assert_eq!(uv_min.value, vec![1.0, 0.0, 1.0, 1.0]);
}

#[test]
fn test_attribute_presence_checks_and_should_use_spherical() {
    let unit_sphere = Ellipsoid::UNIT_SPHERE;
    let projection = WebMercatorProjection::new(unit_sphere).unwrap();
    let small_test_rectangle = GeoRectangle::from_degrees(-0.1, -0.1, 0.1, 0.1);
    let large_test_rectangle = GeoRectangle::from_degrees(-45.0, -45.0, 45.0, 45.0);

    let small_attrs = ShadowVolumeAppearance::get_planar_texture_coordinate_attributes(
        &small_test_rectangle,
        &[0.0, 0.0, 0.0, 1.0, 1.0, 0.0],
        &unit_sphere,
        &projection,
        None,
    )
    .unwrap();
    let large_attrs = ShadowVolumeAppearance::get_spherical_extent_geometry_instance_attributes(
        &large_test_rectangle,
        &[0.0, 0.0, 0.0, 1.0, 1.0, 0.0],
        &unit_sphere,
        &projection,
    );

    assert!(!ShadowVolumeAppearance::has_attributes_for_spherical_extents(&small_attrs));
    assert!(ShadowVolumeAppearance::has_attributes_for_spherical_extents(&large_attrs));

    assert!(ShadowVolumeAppearance::has_attributes_for_texture_coordinate_planes(&small_attrs));
    assert!(!ShadowVolumeAppearance::has_attributes_for_texture_coordinate_planes(&large_attrs));

    assert!(!ShadowVolumeAppearance::should_use_spherical_coordinates(&small_test_rectangle));
    assert!(ShadowVolumeAppearance::should_use_spherical_coordinates(&large_test_rectangle));
}

#[test]
fn test_create_vertex_shaders_color_and_pick() {
    let unit_sphere = Ellipsoid::UNIT_SPHERE;
    let projection = WebMercatorProjection::new(unit_sphere).unwrap();

    let textured_app = ShadowVolumeAppearance::new(
        true,
        false,
        ShadowAppearanceSource::Material {
            material_shader_source: "czm_materialInput materialInput; materialInput.st = vec2(0.0);\n"
                .to_string(),
            fragment_shader_source: "".to_string(),
            flat: false,
        },
    );

    // 3D Color
    let vs_color = textured_app.create_vertex_shader(&[], TEST_VS, false, &projection);
    assert!(vs_color.defines.contains(&"TEXTURE_COORDINATES".to_string()));
    assert!(vs_color.defines.contains(&"SPHERICAL".to_string()));
    assert_eq!(vs_color.defines.len(), 2);

    // 2D Color
    let vs_color_2d = textured_app.create_vertex_shader(&[], TEST_VS, true, &projection);
    assert!(vs_color_2d.defines.contains(&"TEXTURE_COORDINATES".to_string()));
    assert!(vs_color_2d.defines.contains(&"COLUMBUS_VIEW_2D".to_string()));
    assert!(vs_color_2d.defines.iter().any(|d| d.starts_with("EAST_MOST_X_HIGH")));
    assert_eq!(vs_color_2d.defines.len(), 6);

    // Unculled color appearance
    let unculled_color = ShadowVolumeAppearance::new(
        false,
        false,
        ShadowAppearanceSource::PerInstanceColor { flat: false },
    );
    let vs_unculled = unculled_color.create_vertex_shader(&[], TEST_VS, false, &projection);
    assert_eq!(vs_unculled.defines, vec!["PER_INSTANCE_COLOR".to_string()]);

    // Pick 3D
    let vs_pick = textured_app.create_pick_vertex_shader(&[], TEST_VS, false, &projection);
    assert!(vs_pick.defines.contains(&"TEXTURE_COORDINATES".to_string()));
    assert!(vs_pick.defines.contains(&"SPHERICAL".to_string()));
    assert_eq!(vs_pick.defines.len(), 2);

    // Pick 3D unculled
    let vs_pick_unculled = unculled_color.create_pick_vertex_shader(&[], TEST_VS, false, &projection);
    assert!(vs_pick_unculled.defines.is_empty());
}

#[test]
fn test_create_fragment_shaders_color_and_pick() {
    let textured_app = ShadowVolumeAppearance::new(
        true,
        false,
        ShadowAppearanceSource::Material {
            material_shader_source: "\
czm_materialInput materialInput;\n\
materialInput.normalEC = vec3(0.0);\n\
materialInput.positionToEyeEC = vec3(0.0);\n\
materialInput.tangentToEyeMatrix = mat3(1.0);\n\
materialInput.st = vec2(0.0);\n"
                .to_string(),
            fragment_shader_source: "".to_string(),
            flat: false,
        },
    );

    let fs = textured_app.create_fragment_shader(false);
    assert!(fs.defines.contains(&"SPHERICAL".to_string()));
    assert!(fs.defines.contains(&"REQUIRES_EC".to_string()));
    assert!(fs.defines.contains(&"REQUIRES_WC".to_string()));
    assert!(fs.defines.contains(&"TEXTURE_COORDINATES".to_string()));
    assert!(fs.defines.contains(&"CULL_FRAGMENTS".to_string()));
    assert!(fs.defines.contains(&"NORMAL_EC".to_string()));
    assert!(fs.defines.contains(&"USES_NORMAL_EC".to_string()));
    assert!(fs.defines.contains(&"USES_POSITION_TO_EYE_EC".to_string()));
    assert!(fs.defines.contains(&"USES_TANGENT_TO_EYE".to_string()));
    assert!(fs.defines.contains(&"USES_ST".to_string()));
    assert_eq!(fs.defines.len(), 10);

    // 2D variant
    let fs_2d = textured_app.create_fragment_shader(true);
    assert!(!fs_2d.defines.contains(&"SPHERICAL".to_string()));
    assert_eq!(fs_2d.defines.len(), 9);

    // Pick fragment shader
    let planar_color = ShadowVolumeAppearance::new(
        true,
        true,
        ShadowAppearanceSource::PerInstanceColor { flat: false },
    );
    let fs_pick = planar_color.create_pick_fragment_shader(true);
    assert!(fs_pick.defines.contains(&"PICK".to_string()));
    assert!(fs_pick.defines.contains(&"REQUIRES_EC".to_string()));
    assert!(fs_pick.defines.contains(&"TEXTURE_COORDINATES".to_string()));
    assert!(fs_pick.defines.contains(&"CULL_FRAGMENTS".to_string()));
    assert_eq!(fs_pick.defines.len(), 5);
}
