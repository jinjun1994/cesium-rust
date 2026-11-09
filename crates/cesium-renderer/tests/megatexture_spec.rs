use cesium_math::Cartesian3;
use cesium_renderer::{Megatexture, MetadataComponentType};

#[test]
fn test_constructs() {
    let dimensions = Cartesian3::new(16.0, 16.0, 16.0);
    let channel_count = 4;
    let component_type = MetadataComponentType::Float32;
    let texture_memory_byte_length = 16 * 16 * 16 * 4 * 4;

    // Constructor should throw if memory is too small
    let err = Megatexture::new(
        dimensions,
        channel_count,
        component_type,
        Some(texture_memory_byte_length / 2),
        None,
    );
    assert!(err.is_err());

    let megatexture = Megatexture::new(
        dimensions,
        channel_count,
        component_type,
        Some(texture_memory_byte_length),
        None,
    )
    .unwrap();

    assert_eq!(megatexture.channel_count, channel_count);
    assert_eq!(megatexture.component_type, component_type);
    assert_eq!(megatexture.voxel_count_per_tile, dimensions);
}

#[test]
fn test_3d_texture_dimensions_are_constrained_by_memory_limit() {
    let dimensions = Cartesian3::new(23.0, 7.0, 13.0);
    let bytes_per_sample = 4;
    let tile_count = (8usize.pow(5) - 1) / 7; // LODs 0 through 4 fully populated
    let available_texture_memory_bytes = 16 * 1024 * 1024;
    let max_3d_texture_size = 256.0;

    let texture_dimensions = Megatexture::get_3d_texture_dimension_with_limit(
        dimensions,
        bytes_per_sample,
        available_texture_memory_bytes,
        Some(tile_count),
        max_3d_texture_size,
    )
    .unwrap();

    let expected_dimensions = Cartesian3::new(69.0, 245.0, 247.0);
    assert_eq!(texture_dimensions, expected_dimensions);

    let actual_memory_usage =
        (texture_dimensions.x * texture_dimensions.y * texture_dimensions.z) as usize * bytes_per_sample;
    assert!(actual_memory_usage < available_texture_memory_bytes);
    assert!(actual_memory_usage as f64 >= available_texture_memory_bytes as f64 * 0.8);
}

#[test]
fn test_3d_texture_is_large_enough_to_fit_number_of_tiles() {
    let dimensions = Cartesian3::new(23.0, 7.0, 13.0);
    let bytes_per_sample = 4;
    let tile_count = (8usize.pow(5) - 1) / 7;
    let available_texture_memory_bytes = 256 * 1024 * 1024;
    let max_3d_texture_size = 256.0;

    let texture_dimensions = Megatexture::get_3d_texture_dimension_with_limit(
        dimensions,
        bytes_per_sample,
        available_texture_memory_bytes,
        Some(tile_count),
        max_3d_texture_size,
    )
    .unwrap();

    let expected_dimensions = Cartesian3::new(161.0, 252.0, 247.0);
    assert_eq!(texture_dimensions, expected_dimensions);

    let maximum_tile_count = ((texture_dimensions.x / dimensions.x)
        * (texture_dimensions.y / dimensions.y)
        * (texture_dimensions.z / dimensions.z)) as usize;
    assert!(maximum_tile_count >= tile_count);
    assert!((maximum_tile_count as f64) <= tile_count as f64 * 1.25);
}

#[test]
fn test_3d_texture_maximizes_usage_of_small_device_limits() {
    let dimensions = Cartesian3::new(7.0, 17.0, 23.0);
    let bytes_per_sample = 1;
    let tile_count = 6001;
    let available_texture_memory_bytes = 100_000_000;
    let max_3d_texture_size = 256.0;

    let texture_dimensions = Megatexture::get_3d_texture_dimension_with_limit(
        dimensions,
        bytes_per_sample,
        available_texture_memory_bytes,
        Some(tile_count),
        max_3d_texture_size,
    )
    .unwrap();

    let expected_dimensions = Cartesian3::new(252.0, 255.0, 253.0);
    assert_eq!(texture_dimensions, expected_dimensions);
}

#[test]
fn test_adds_data_to_existing_megatexture() {
    let dimension = 16.0;
    let dimensions = Cartesian3::new(dimension, dimension, dimension);
    let channel_count = 4;
    let component_type = MetadataComponentType::Uint16;
    let tile_size = 16 * 16 * 16 * 4 * MetadataComponentType::get_size_in_bytes(component_type);

    let mut megatexture =
        Megatexture::new(dimensions, channel_count, component_type, None, None).unwrap();

    let data = vec![0u8; tile_size];
    let index = megatexture.add(&data).unwrap();
    assert_eq!(index, 0);
}

#[test]
fn test_throws_if_trying_to_add_to_full_megatexture() {
    let dimension = 16.0;
    let dimensions = Cartesian3::new(dimension, dimension, dimension);
    let channel_count = 1;
    let component_type = MetadataComponentType::Float32;
    let tile_size = 16 * 16 * 16 * MetadataComponentType::get_size_in_bytes(component_type);

    let tile_count = 1;
    let texture_memory_byte_length = tile_count * tile_size;
    let mut megatexture = Megatexture::new(
        dimensions,
        channel_count,
        component_type,
        Some(texture_memory_byte_length),
        None,
    )
    .unwrap();

    let data = vec![0u8; tile_size];
    assert!(megatexture.add(&data).is_ok());

    // megatexture should be full now
    assert!(megatexture.is_full());
    let err = megatexture.add(&data);
    assert!(err.is_err());
    assert_eq!(
        err.unwrap_err().message,
        "Trying to add when there are no empty spots"
    );
}

#[test]
fn test_removes_tile_from_a_texture() {
    let dimension = 16.0;
    let dimensions = Cartesian3::new(dimension, dimension, dimension);
    let channel_count = 1;
    let component_type = MetadataComponentType::Float32;
    let tile_size = 16 * 16 * 16 * MetadataComponentType::get_size_in_bytes(component_type);

    let tile_count = 4;
    let texture_memory_byte_length = tile_count * tile_size;
    let mut megatexture = Megatexture::new(
        dimensions,
        channel_count,
        component_type,
        Some(texture_memory_byte_length),
        None,
    )
    .unwrap();

    assert_eq!(megatexture.occupied_count, 0);

    let data = vec![0u8; tile_size];
    let _i0 = megatexture.add(&data).unwrap();
    let _i1 = megatexture.add(&data).unwrap();
    let index2 = megatexture.add(&data).unwrap();
    let _i3 = megatexture.add(&data).unwrap();

    assert_eq!(megatexture.occupied_count, 4);
    assert!(megatexture.add(&data).is_err());

    // Remove one tile
    megatexture.remove(index2 as i64).unwrap();
    assert_eq!(megatexture.occupied_count, 3);

    // Can add again
    assert!(megatexture.add(&data).is_ok());
    assert_eq!(megatexture.occupied_count, 4);

    // Out of bounds index
    assert!(megatexture.remove(-1).is_err());
    assert!(megatexture.remove(4).is_err());
}

#[test]
fn test_destroys() {
    let tile_count = 4;
    let dimension = 16.0;
    let dimensions = Cartesian3::new(dimension, dimension, dimension);
    let channel_count = 4u32;
    let component_type = MetadataComponentType::Float32;
    let texture_memory_byte_length =
        tile_count * 16 * 16 * 16 * (channel_count as usize) * MetadataComponentType::get_size_in_bytes(component_type);

    let mut megatexture = Megatexture::new(
        dimensions,
        channel_count,
        component_type,
        Some(texture_memory_byte_length),
        None,
    )
    .unwrap();

    assert_eq!(megatexture.maximum_tile_count, 4);
    assert!(megatexture.remove(0).is_ok());
    assert!(!megatexture.is_destroyed());

    megatexture.destroy();

    assert!(megatexture.is_destroyed());
    assert!(megatexture.remove(0).is_err());
}
