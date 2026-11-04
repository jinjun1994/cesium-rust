use cesium_math::{
    AttributeCompression, AxisAlignedBoundingBox, Cartesian2, Cartesian3, Ellipsoid,
    FixedFrameTransforms, Matrix4, TerrainEncoding, TerrainQuantization, VerticalExaggeration,
    EPSILON5,
};

fn center_at_degrees(lon: f64, lat: f64) -> Cartesian3 {
    Cartesian3::from_degrees(lon, lat, None, None).expect("valid degrees")
}

fn oct_encode(v: Cartesian3) -> Cartesian2 {
    AttributeCompression::oct_encode(v).expect("oct encode")
}

fn equals_epsilon(a: &Cartesian3, b: &Cartesian3, eps: f64) -> bool {
    Cartesian3::equals_epsilon(a, b, Some(eps), None)
}

#[test]
fn test_default_constructs() {
    let encoding = TerrainEncoding::default();
    assert_eq!(encoding.quantization, TerrainQuantization::None);
    assert!(encoding.minimum_height.is_none());
    assert!(encoding.maximum_height.is_none());
    assert!(encoding.center.is_none());
    assert!(encoding.to_scaled_enu.is_none());
    assert!(encoding.from_scaled_enu.is_none());
    assert!(encoding.matrix.is_none());
    assert!(!encoding.has_vertex_normals);
    assert!(!encoding.has_web_mercator_t);
    assert!(!encoding.has_geodetic_surface_normals);
    assert_eq!(encoding.exaggeration, 1.0);
    assert_eq!(encoding.exaggeration_relative_height, 0.0);
    assert_eq!(encoding.stride, 6);
}

#[test]
fn test_constructs_without_quantization() {
    let center = center_at_degrees(0.0, 0.0);
    let maximum = Cartesian3::new(1.0e6, 1.0e6, 1.0e6);
    let minimum = -maximum;
    let aabox = AxisAlignedBoundingBox::new(minimum.into(), maximum.into(), Some(center.into()));
    let maximum_height = 1.0e6;
    let minimum_height = maximum_height;
    let has_vertex_normals = false;
    let from_enu = FixedFrameTransforms::east_north_up_to_fixed_frame(center, None)
        .expect("valid fixed frame");

    let encoding = TerrainEncoding::new(
        center,
        Some(&aabox),
        Some(minimum_height),
        Some(maximum_height),
        Some(from_enu),
        has_vertex_normals,
        None,
        None,
        None,
        None,
    );

    assert_eq!(encoding.quantization, TerrainQuantization::None);
    assert_eq!(encoding.minimum_height, Some(minimum_height));
    assert_eq!(encoding.maximum_height, Some(maximum_height));
    assert_eq!(encoding.center, Some(center));
    assert!(encoding.to_scaled_enu.is_some());
    assert!(encoding.from_scaled_enu.is_some());
    assert!(encoding.matrix.is_some());
    assert_eq!(encoding.has_vertex_normals, has_vertex_normals);
}

#[test]
fn test_constructs_with_quantization() {
    let center = center_at_degrees(0.0, 0.0);
    let maximum = Cartesian3::new(100.0, 100.0, 100.0);
    let minimum = -maximum;
    let aabox = AxisAlignedBoundingBox::new(minimum.into(), maximum.into(), Some(center.into()));
    let minimum_height = -100.0;
    let maximum_height = 100.0;
    let has_vertex_normals = false;
    let from_enu = FixedFrameTransforms::east_north_up_to_fixed_frame(center, None)
        .expect("valid fixed frame");

    let encoding = TerrainEncoding::new(
        center,
        Some(&aabox),
        Some(minimum_height),
        Some(maximum_height),
        Some(from_enu),
        has_vertex_normals,
        None,
        None,
        None,
        None,
    );

    assert_eq!(encoding.quantization, TerrainQuantization::Bits12);
    assert_eq!(encoding.minimum_height, Some(minimum_height));
    assert_eq!(encoding.maximum_height, Some(maximum_height));
    assert_eq!(encoding.center, Some(center));
    assert!(encoding.to_scaled_enu.is_some());
    assert!(encoding.from_scaled_enu.is_some());
    assert!(encoding.matrix.is_some());
    assert_eq!(encoding.has_vertex_normals, has_vertex_normals);
}

#[test]
fn test_encodes_without_quantization_or_normals() {
    let center = center_at_degrees(0.0, 0.0);
    let maximum = Cartesian3::new(6.0e3, 6.0e3, 6.0e3);
    let minimum = -maximum;
    let aabox = AxisAlignedBoundingBox::new(minimum.into(), maximum.into(), Some(center.into()));
    let maximum_height = 6.0e3;
    let minimum_height = maximum_height;
    let has_vertex_normals = false;
    let from_enu = FixedFrameTransforms::east_north_up_to_fixed_frame(center, None)
        .expect("valid fixed frame");

    let encoding = TerrainEncoding::new(
        center,
        Some(&aabox),
        Some(minimum_height),
        Some(maximum_height),
        Some(from_enu),
        has_vertex_normals,
        None,
        None,
        None,
        None,
    );

    let position = Cartesian3::new(1.0e3, 1.0e3, 1.0e3);
    let world_pos = from_enu.multiply_by_point(position);

    let mut buffer = Vec::new();
    encoding.encode(
        &mut buffer,
        0,
        world_pos,
        Cartesian2::ZERO,
        100.0,
        None,
        None,
        None,
    );

    assert_eq!(encoding.stride, 6);
    assert_eq!(buffer.len(), encoding.stride);

    let decoded = encoding.decode_position(&buffer, 0);
    assert!(equals_epsilon(&decoded, &world_pos, 1e-4));
}

#[test]
fn test_encodes_without_quantization_and_with_normals() {
    let center = center_at_degrees(0.0, 0.0);
    let maximum = Cartesian3::new(6.0e3, 6.0e3, 6.0e3);
    let minimum = -maximum;
    let aabox = AxisAlignedBoundingBox::new(minimum.into(), maximum.into(), Some(center.into()));
    let maximum_height = 6.0e3;
    let minimum_height = maximum_height;
    let has_vertex_normals = true;
    let from_enu = FixedFrameTransforms::east_north_up_to_fixed_frame(center, None)
        .expect("valid fixed frame");

    let encoding = TerrainEncoding::new(
        center,
        Some(&aabox),
        Some(minimum_height),
        Some(maximum_height),
        Some(from_enu),
        has_vertex_normals,
        None,
        None,
        None,
        None,
    );

    let position = Cartesian3::new(1.0e3, 1.0e3, 1.0e3);
    let world_pos = from_enu.multiply_by_point(position);
    let normal = Cartesian3::normalize(&world_pos);
    let oct_normal = oct_encode(normal);

    let mut buffer = Vec::new();
    encoding.encode(
        &mut buffer,
        0,
        world_pos,
        Cartesian2::ZERO,
        100.0,
        Some(oct_normal),
        None,
        None,
    );

    assert_eq!(encoding.stride, 7);
    assert_eq!(buffer.len(), encoding.stride);

    let decoded = encoding.decode_position(&buffer, 0);
    assert!(equals_epsilon(&decoded, &world_pos, 1e-4));
}

#[test]
fn test_encodes_position_with_quantization_and_without_normals() {
    let center = center_at_degrees(0.0, 0.0);
    let maximum = Cartesian3::new(6.0e2, 6.0e2, 6.0e2);
    let minimum = -maximum;
    let aabox = AxisAlignedBoundingBox::new(minimum.into(), maximum.into(), Some(center.into()));
    let maximum_height = 6.0e2;
    let minimum_height = maximum_height;
    let from_enu = FixedFrameTransforms::east_north_up_to_fixed_frame(center, None)
        .expect("valid fixed frame");
    let has_vertex_normals = false;

    let encoding = TerrainEncoding::new(
        center,
        Some(&aabox),
        Some(minimum_height),
        Some(maximum_height),
        Some(from_enu),
        has_vertex_normals,
        None,
        None,
        None,
        None,
    );

    let position = Cartesian3::new(1.0e2, 1.0e2, 1.0e2);
    let world_pos = from_enu.multiply_by_point(position);

    let mut buffer = Vec::new();
    encoding.encode(
        &mut buffer,
        0,
        world_pos,
        Cartesian2::ZERO,
        100.0,
        None,
        None,
        None,
    );

    assert_eq!(encoding.stride, 3);
    assert_eq!(buffer.len(), encoding.stride);

    let decoded = encoding.decode_position(&buffer, 0);
    assert!(equals_epsilon(&decoded, &world_pos, 1.0));
}

#[test]
fn test_encodes_position_with_quantization_and_normals() {
    let center = center_at_degrees(0.0, 0.0);
    let maximum = Cartesian3::new(6.0e2, 6.0e2, 6.0e2);
    let minimum = -maximum;
    let aabox = AxisAlignedBoundingBox::new(minimum.into(), maximum.into(), Some(center.into()));
    let maximum_height = 6.0e2;
    let minimum_height = maximum_height;
    let from_enu = FixedFrameTransforms::east_north_up_to_fixed_frame(center, None)
        .expect("valid fixed frame");
    let has_vertex_normals = true;

    let encoding = TerrainEncoding::new(
        center,
        Some(&aabox),
        Some(minimum_height),
        Some(maximum_height),
        Some(from_enu),
        has_vertex_normals,
        None,
        None,
        None,
        None,
    );

    let position = Cartesian3::new(1.0e2, 1.0e2, 1.0e2);
    let world_pos = from_enu.multiply_by_point(position);
    let normal = Cartesian3::normalize(&world_pos);
    let oct_normal = oct_encode(normal);

    let mut buffer = Vec::new();
    encoding.encode(
        &mut buffer,
        0,
        world_pos,
        Cartesian2::ZERO,
        100.0,
        Some(oct_normal),
        None,
        None,
    );

    assert_eq!(encoding.stride, 4);
    assert_eq!(buffer.len(), encoding.stride);

    let decoded = encoding.decode_position(&buffer, 0);
    assert!(equals_epsilon(&decoded, &world_pos, 1.0));
}

#[test]
fn test_encodes_position_without_quantization_and_with_exaggeration() {
    let has_vertex_normals = false;
    let has_web_mercator_t = false;
    let has_geodetic_surface_normals = true;

    let height = 1_000_000.0;
    let position = Cartesian3::new(height, 0.0, 0.0);
    let geodetic_surface_normal = Cartesian3::new(1.0, 0.0, 0.0);

    let exaggeration = 2.0;
    let exaggeration_relative_height = 10.0;
    let exaggerated_height =
        VerticalExaggeration::get_height(height, exaggeration, exaggeration_relative_height);
    let exaggerated_position = Cartesian3::new(exaggerated_height, 0.0, 0.0);

    let maximum_height = height;
    let minimum_height = -height;
    let maximum = Cartesian3::new(height, height, height);
    let minimum = -maximum;
    let aabox =
        AxisAlignedBoundingBox::new(minimum.into(), maximum.into(), Some(Cartesian3::ZERO.into()));

    let encoding = TerrainEncoding::new(
        Cartesian3::ZERO,
        Some(&aabox),
        Some(minimum_height),
        Some(maximum_height),
        Some(Matrix4::IDENTITY),
        has_vertex_normals,
        Some(has_web_mercator_t),
        Some(has_geodetic_surface_normals),
        Some(exaggeration),
        Some(exaggeration_relative_height),
    );

    let mut buffer = Vec::new();
    encoding.encode(
        &mut buffer,
        0,
        position,
        Cartesian2::ZERO,
        height,
        None,
        None,
        Some(geodetic_surface_normal),
    );

    assert_eq!(encoding.stride, 9);
    assert_eq!(buffer.len(), encoding.stride);

    let dec_pos = encoding.get_exaggerated_position(&buffer, 0);
    assert!(equals_epsilon(&dec_pos, &exaggerated_position, EPSILON5));

    let dec_norm = encoding.decode_geodetic_surface_normal(&buffer, 0);
    assert!(equals_epsilon(&dec_norm, &geodetic_surface_normal, EPSILON5));
}

#[test]
fn test_encodes_texture_coordinates_with_quantization_and_without_normals() {
    let center = center_at_degrees(0.0, 0.0);
    let maximum = Cartesian3::new(6.0e2, 6.0e2, 6.0e2);
    let minimum = -maximum;
    let aabox = AxisAlignedBoundingBox::new(minimum.into(), maximum.into(), Some(center.into()));
    let maximum_height = 6.0e2;
    let minimum_height = maximum_height;
    let from_enu = FixedFrameTransforms::east_north_up_to_fixed_frame(center, None)
        .expect("valid fixed frame");
    let has_vertex_normals = false;

    let encoding = TerrainEncoding::new(
        center,
        Some(&aabox),
        Some(minimum_height),
        Some(maximum_height),
        Some(from_enu),
        has_vertex_normals,
        None,
        None,
        None,
        None,
    );

    let tex_coords = Cartesian2::new(0.25, 0.75);
    let mut buffer = Vec::new();
    encoding.encode(
        &mut buffer,
        0,
        Cartesian3::ZERO,
        tex_coords,
        100.0,
        None,
        None,
        None,
    );

    assert_eq!(encoding.stride, 3);
    assert_eq!(buffer.len(), encoding.stride);

    let dec_tc = encoding.decode_texture_coordinates(&buffer, 0);
    assert!((dec_tc.x - tex_coords.x).abs() <= 1.0 / 4095.0);
    assert!((dec_tc.y - tex_coords.y).abs() <= 1.0 / 4095.0);
}

#[test]
fn test_encodes_texture_coordinates_with_quantization_and_normals() {
    let center = center_at_degrees(0.0, 0.0);
    let maximum = Cartesian3::new(6.0e2, 6.0e2, 6.0e2);
    let minimum = -maximum;
    let aabox = AxisAlignedBoundingBox::new(minimum.into(), maximum.into(), Some(center.into()));
    let maximum_height = 6.0e2;
    let minimum_height = maximum_height;
    let from_enu = FixedFrameTransforms::east_north_up_to_fixed_frame(center, None)
        .expect("valid fixed frame");
    let has_vertex_normals = true;

    let encoding = TerrainEncoding::new(
        center,
        Some(&aabox),
        Some(minimum_height),
        Some(maximum_height),
        Some(from_enu),
        has_vertex_normals,
        None,
        None,
        None,
        None,
    );

    let tex_coords = Cartesian2::new(0.75, 0.25);
    let mut buffer = Vec::new();
    let oct_x = oct_encode(Cartesian3::UNIT_X);
    encoding.encode(
        &mut buffer,
        0,
        Cartesian3::ZERO,
        tex_coords,
        100.0,
        Some(oct_x),
        None,
        None,
    );

    assert_eq!(encoding.stride, 4);
    assert_eq!(buffer.len(), encoding.stride);

    let dec_tc = encoding.decode_texture_coordinates(&buffer, 0);
    assert!((dec_tc.x - tex_coords.x).abs() <= 1.0 / 4095.0);
    assert!((dec_tc.y - tex_coords.y).abs() <= 1.0 / 4095.0);
}

#[test]
fn test_encodes_height_with_quantization_and_without_normals() {
    let center = center_at_degrees(0.0, 0.0);
    let maximum = Cartesian3::new(6.0e2, 6.0e2, 6.0e2);
    let minimum = -maximum;
    let aabox = AxisAlignedBoundingBox::new(minimum.into(), maximum.into(), Some(center.into()));
    let minimum_height = 0.0;
    let maximum_height = 200.0;
    let from_enu = FixedFrameTransforms::east_north_up_to_fixed_frame(center, None)
        .expect("valid fixed frame");
    let has_vertex_normals = false;

    let encoding = TerrainEncoding::new(
        center,
        Some(&aabox),
        Some(minimum_height),
        Some(maximum_height),
        Some(from_enu),
        has_vertex_normals,
        None,
        None,
        None,
        None,
    );

    let height = (maximum_height + minimum_height) * 0.5;
    let mut buffer = Vec::new();
    encoding.encode(
        &mut buffer,
        0,
        center,
        Cartesian2::ZERO,
        height,
        None,
        None,
        None,
    );

    assert_eq!(encoding.stride, 3);
    assert_eq!(buffer.len(), encoding.stride);

    let dec_h = encoding.decode_height(&buffer, 0);
    assert!((dec_h - height).abs() <= 200.0 / 4095.0);
}

#[test]
fn test_encodes_height_with_quantization_and_normals() {
    let center = center_at_degrees(0.0, 0.0);
    let maximum = Cartesian3::new(6.0e2, 6.0e2, 6.0e2);
    let minimum = -maximum;
    let aabox = AxisAlignedBoundingBox::new(minimum.into(), maximum.into(), Some(center.into()));
    let minimum_height = 0.0;
    let maximum_height = 200.0;
    let from_enu = FixedFrameTransforms::east_north_up_to_fixed_frame(center, None)
        .expect("valid fixed frame");
    let has_vertex_normals = true;

    let encoding = TerrainEncoding::new(
        center,
        Some(&aabox),
        Some(minimum_height),
        Some(maximum_height),
        Some(from_enu),
        has_vertex_normals,
        None,
        None,
        None,
        None,
    );

    let height = (maximum_height + minimum_height) * 0.5;
    let mut buffer = Vec::new();
    let oct_x = oct_encode(Cartesian3::UNIT_X);
    encoding.encode(
        &mut buffer,
        0,
        center,
        Cartesian2::ZERO,
        height,
        Some(oct_x),
        None,
        None,
    );

    assert_eq!(encoding.stride, 4);
    assert_eq!(buffer.len(), encoding.stride);

    let dec_h = encoding.decode_height(&buffer, 0);
    assert!((dec_h - height).abs() <= 200.0 / 4095.0);
}

#[test]
fn test_gets_oct_encoded_normal() {
    let center = center_at_degrees(0.0, 0.0);
    let maximum = Cartesian3::new(6.0e2, 6.0e2, 6.0e2);
    let minimum = -maximum;
    let aabox = AxisAlignedBoundingBox::new(minimum.into(), maximum.into(), Some(center.into()));
    let minimum_height = 6.0e2;
    let maximum_height = 6.0e2;
    let from_enu = FixedFrameTransforms::east_north_up_to_fixed_frame(center, None)
        .expect("valid fixed frame");
    let has_vertex_normals = true;

    let encoding = TerrainEncoding::new(
        center,
        Some(&aabox),
        Some(minimum_height),
        Some(maximum_height),
        Some(from_enu),
        has_vertex_normals,
        None,
        None,
        None,
        None,
    );

    let normal = Cartesian3::normalize(&Cartesian3::new(1.0, 1.0, 1.0));
    let oct_normal = oct_encode(normal);

    let mut buffer = Vec::new();
    encoding.encode(
        &mut buffer,
        0,
        center,
        Cartesian2::ZERO,
        minimum_height,
        Some(oct_normal),
        None,
        None,
    );

    assert_eq!(encoding.stride, 4);
    assert_eq!(buffer.len(), encoding.stride);

    let dec_norm = encoding.get_oct_encoded_normal(&buffer, 0);
    assert_eq!(dec_norm, oct_normal);
}

#[test]
fn test_adds_geodetic_surface_normals() {
    let center = center_at_degrees(0.0, 0.0);
    let maximum = Cartesian3::new(6.0e2, 6.0e2, 6.0e2);
    let minimum = -maximum;
    let aabox = AxisAlignedBoundingBox::new(minimum.into(), maximum.into(), Some(center.into()));
    let minimum_height = 6.0e2;
    let maximum_height = 6.0e2;
    let from_enu = FixedFrameTransforms::east_north_up_to_fixed_frame(center, None)
        .expect("valid fixed frame");
    let has_vertex_normals = false;

    let mut encoding = TerrainEncoding::new(
        center,
        Some(&aabox),
        Some(minimum_height),
        Some(maximum_height),
        Some(from_enu),
        has_vertex_normals,
        None,
        None,
        None,
        None,
    );

    let mut old_buffer = Vec::new();
    encoding.encode(
        &mut old_buffer,
        0,
        center,
        Cartesian2::ZERO,
        minimum_height,
        None,
        None,
        None,
    );
    let old_stride = encoding.stride;

    let mut new_buffer = Vec::new();
    let ellipsoid = Ellipsoid::UNIT_SPHERE;
    encoding.add_geodetic_surface_normals(&old_buffer, &mut new_buffer, &ellipsoid);
    let new_stride = encoding.stride;
    let stride_difference = (new_stride as isize) - (old_stride as isize);

    assert_eq!(stride_difference, 3);
    assert_eq!(old_buffer.len(), old_stride);
    assert_eq!(new_buffer.len(), new_stride);
}

#[test]
fn test_removes_geodetic_surface_normals() {
    let center = center_at_degrees(0.0, 0.0);
    let maximum = Cartesian3::new(6.0e2, 6.0e2, 6.0e2);
    let minimum = -maximum;
    let aabox = AxisAlignedBoundingBox::new(minimum.into(), maximum.into(), Some(center.into()));
    let minimum_height = 6.0e2;
    let maximum_height = 6.0e2;
    let from_enu = FixedFrameTransforms::east_north_up_to_fixed_frame(center, None)
        .expect("valid fixed frame");
    let has_vertex_normals = false;
    let has_web_mercator_t = false;
    let has_geodetic_surface_normals = true;

    let mut encoding = TerrainEncoding::new(
        center,
        Some(&aabox),
        Some(minimum_height),
        Some(maximum_height),
        Some(from_enu),
        has_vertex_normals,
        Some(has_web_mercator_t),
        Some(has_geodetic_surface_normals),
        None,
        None,
    );

    let geodetic_surface_normal = Cartesian3::new(1.0, 0.0, 0.0);
    let mut old_buffer = Vec::new();
    encoding.encode(
        &mut old_buffer,
        0,
        center,
        Cartesian2::ZERO,
        minimum_height,
        None,
        None,
        Some(geodetic_surface_normal),
    );
    let old_stride = encoding.stride;

    let mut new_buffer = Vec::new();
    encoding.remove_geodetic_surface_normals(&old_buffer, &mut new_buffer);
    let new_stride = encoding.stride;
    let stride_difference = (new_stride as isize) - (old_stride as isize);

    assert_eq!(stride_difference, -3);
    assert_eq!(old_buffer.len(), old_stride);
    assert_eq!(new_buffer.len(), new_stride);
}

#[test]
fn test_gets_attributes() {
    let center = center_at_degrees(0.0, 0.0);
    let maximum = Cartesian3::new(1.0e6, 1.0e6, 1.0e6);
    let minimum = -maximum;
    let aabox = AxisAlignedBoundingBox::new(minimum.into(), maximum.into(), Some(center.into()));
    let maximum_height = 1.0e6;
    let minimum_height = maximum_height;
    let from_enu = FixedFrameTransforms::east_north_up_to_fixed_frame(center, None)
        .expect("valid fixed frame");
    let has_vertex_normals = false;

    let encoding = TerrainEncoding::new(
        center,
        Some(&aabox),
        Some(minimum_height),
        Some(maximum_height),
        Some(from_enu),
        has_vertex_normals,
        None,
        None,
        None,
        None,
    );

    let buffer = Vec::new();
    let attributes = encoding.get_attributes(&buffer);
    assert_eq!(attributes.len(), 2);
}

#[test]
fn test_gets_attribute_locations() {
    let center = center_at_degrees(0.0, 0.0);
    let maximum = Cartesian3::new(1.0e6, 1.0e6, 1.0e6);
    let minimum = -maximum;
    let aabox = AxisAlignedBoundingBox::new(minimum.into(), maximum.into(), Some(center.into()));
    let maximum_height = 1.0e6;
    let minimum_height = maximum_height;
    let from_enu = FixedFrameTransforms::east_north_up_to_fixed_frame(center, None)
        .expect("valid fixed frame");
    let has_vertex_normals = false;

    let encoding = TerrainEncoding::new(
        center,
        Some(&aabox),
        Some(minimum_height),
        Some(maximum_height),
        Some(from_enu),
        has_vertex_normals,
        None,
        None,
        None,
        None,
    );

    let locations = encoding.get_attribute_locations();
    assert_eq!(locations.position_3d_and_height, 0);
    assert_eq!(locations.texture_coord_and_encoded_normals, 1);
    assert_eq!(locations.geodetic_surface_normal, 2);
}

#[test]
fn test_clones() {
    let center = center_at_degrees(0.0, 0.0);
    let maximum = Cartesian3::new(1.0e6, 1.0e6, 1.0e6);
    let minimum = -maximum;
    let aabox = AxisAlignedBoundingBox::new(minimum.into(), maximum.into(), Some(center.into()));
    let maximum_height = 1.0e6;
    let minimum_height = maximum_height;
    let from_enu = FixedFrameTransforms::east_north_up_to_fixed_frame(center, None)
        .expect("valid fixed frame");
    let has_vertex_normals = false;

    let encoding = TerrainEncoding::new(
        center,
        Some(&aabox),
        Some(minimum_height),
        Some(maximum_height),
        Some(from_enu),
        has_vertex_normals,
        None,
        None,
        None,
        None,
    );

    let cloned = encoding.clone();
    assert_eq!(cloned.quantization, encoding.quantization);
    assert_eq!(cloned.minimum_height, encoding.minimum_height);
    assert_eq!(cloned.maximum_height, encoding.maximum_height);
    assert_eq!(cloned.center, encoding.center);
    assert_eq!(cloned.to_scaled_enu, encoding.to_scaled_enu);
    assert_eq!(cloned.from_scaled_enu, encoding.from_scaled_enu);
    assert_eq!(cloned.matrix, encoding.matrix);
    assert_eq!(cloned.has_vertex_normals, encoding.has_vertex_normals);
}

#[test]
fn test_clones_with_result() {
    let center = center_at_degrees(0.0, 0.0);
    let maximum = Cartesian3::new(1.0e6, 1.0e6, 1.0e6);
    let minimum = -maximum;
    let aabox = AxisAlignedBoundingBox::new(minimum.into(), maximum.into(), Some(center.into()));
    let maximum_height = 1.0e6;
    let minimum_height = maximum_height;
    let from_enu = FixedFrameTransforms::east_north_up_to_fixed_frame(center, None)
        .expect("valid fixed frame");
    let has_vertex_normals = false;

    let encoding = TerrainEncoding::new(
        center,
        Some(&aabox),
        Some(minimum_height),
        Some(maximum_height),
        Some(from_enu),
        has_vertex_normals,
        None,
        None,
        None,
        None,
    );

    let mut result = TerrainEncoding::default();
    encoding.clone_into(&mut result);

    assert_eq!(result.quantization, encoding.quantization);
    assert_eq!(result.minimum_height, encoding.minimum_height);
    assert_eq!(result.maximum_height, encoding.maximum_height);
    assert_eq!(result.center, encoding.center);
    assert_eq!(result.to_scaled_enu, encoding.to_scaled_enu);
    assert_eq!(result.from_scaled_enu, encoding.from_scaled_enu);
    assert_eq!(result.matrix, encoding.matrix);
    assert_eq!(result.has_vertex_normals, encoding.has_vertex_normals);
}
