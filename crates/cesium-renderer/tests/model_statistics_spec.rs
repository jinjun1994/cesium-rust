use std::sync::Arc;

use cesium_renderer::model_statistics::{
    BufferStatisticsItem, DynamicBatchTextureStatistics, ModelStatistics, TextureStatisticsItem,
};

#[test]
fn test_constructs() {
    let statistics = ModelStatistics::new();

    assert_eq!(statistics.points_length, 0);
    assert_eq!(statistics.triangles_length, 0);
    assert_eq!(statistics.geometry_byte_length, 0);
    assert_eq!(statistics.textures_byte_length, 0);
    assert_eq!(statistics.property_tables_byte_length, 0);
    assert_eq!(statistics.batch_textures_byte_length(), 0);
    assert!(statistics.buffer_id_set.is_empty());
    assert!(statistics.texture_id_byte_lengths.is_empty());
    assert!(statistics.batch_texture_id_map.is_empty());
}

#[test]
fn test_clears() {
    let mut statistics = ModelStatistics::new();
    statistics.points_length = 10;
    statistics.triangles_length = 10;
    statistics.geometry_byte_length = 10;
    statistics.textures_byte_length = 10;
    statistics.property_tables_byte_length = 10;
    statistics.buffer_id_set.insert("uuid1".to_string());
    statistics.buffer_id_set.insert("uuid2".to_string());
    statistics
        .texture_id_byte_lengths
        .insert("uuid3".to_string(), 1234);

    let batch_texture = Arc::new(DynamicBatchTextureStatistics::new("uuid"));
    statistics
        .batch_texture_id_map
        .insert("uuid".to_string(), batch_texture);

    statistics.clear();

    assert_eq!(statistics.points_length, 0);
    assert_eq!(statistics.triangles_length, 0);
    assert_eq!(statistics.geometry_byte_length, 0);
    assert_eq!(statistics.textures_byte_length, 0);
    assert_eq!(statistics.property_tables_byte_length, 0);
    assert_eq!(statistics.batch_textures_byte_length(), 0);
    assert!(statistics.buffer_id_set.is_empty());
    assert!(statistics.texture_id_byte_lengths.is_empty());
    assert!(statistics.batch_texture_id_map.is_empty());
}

#[test]
fn test_add_buffer_counts_gpu_buffers() {
    let mut statistics = ModelStatistics::new();

    let buffer = BufferStatisticsItem {
        id: "uuid".to_string(),
        size_in_bytes: 10,
    };

    statistics.add_buffer(&buffer, false);

    assert_eq!(statistics.points_length, 0);
    assert_eq!(statistics.triangles_length, 0);
    assert_eq!(statistics.geometry_byte_length, 10);
    assert_eq!(statistics.textures_byte_length, 0);
    assert_eq!(statistics.property_tables_byte_length, 0);
    assert_eq!(statistics.batch_textures_byte_length(), 0);
    assert_eq!(statistics.buffer_id_set.len(), 1);
    assert!(statistics.buffer_id_set.contains("uuid"));
    assert!(statistics.texture_id_byte_lengths.is_empty());
    assert!(statistics.batch_texture_id_map.is_empty());
}

#[test]
fn test_add_buffer_counts_gpu_buffers_with_cpu_copy() {
    let mut statistics = ModelStatistics::new();

    let buffer = BufferStatisticsItem {
        id: "uuid".to_string(),
        size_in_bytes: 10,
    };

    statistics.add_buffer(&buffer, true);

    assert_eq!(statistics.points_length, 0);
    assert_eq!(statistics.triangles_length, 0);
    assert_eq!(statistics.geometry_byte_length, 20);
    assert_eq!(statistics.textures_byte_length, 0);
    assert_eq!(statistics.property_tables_byte_length, 0);
    assert_eq!(statistics.batch_textures_byte_length(), 0);
    assert_eq!(statistics.buffer_id_set.len(), 1);
    assert!(statistics.buffer_id_set.contains("uuid"));
    assert!(statistics.texture_id_byte_lengths.is_empty());
    assert!(statistics.batch_texture_id_map.is_empty());
}

#[test]
fn test_add_buffer_deduplicates_buffers() {
    let mut statistics = ModelStatistics::new();

    let buffer1 = BufferStatisticsItem {
        id: "uuid1".to_string(),
        size_in_bytes: 2,
    };
    let buffer2 = BufferStatisticsItem {
        id: "uuid2".to_string(),
        size_in_bytes: 3,
    };

    statistics.add_buffer(&buffer1, false);
    statistics.add_buffer(&buffer1, false);
    statistics.add_buffer(&buffer2, false);

    assert_eq!(statistics.points_length, 0);
    assert_eq!(statistics.triangles_length, 0);
    // 2 + 3 = 5. De-duplication avoids 2 * 2 + 3 = 7.
    assert_eq!(statistics.geometry_byte_length, 5);
    assert_eq!(statistics.textures_byte_length, 0);
    assert_eq!(statistics.property_tables_byte_length, 0);
    assert_eq!(statistics.batch_textures_byte_length(), 0);
    assert_eq!(statistics.buffer_id_set.len(), 2);
    assert!(statistics.buffer_id_set.contains("uuid1"));
    assert!(statistics.buffer_id_set.contains("uuid2"));
    assert!(statistics.texture_id_byte_lengths.is_empty());
    assert!(statistics.batch_texture_id_map.is_empty());
}

#[test]
fn test_add_texture_counts_textures() {
    let mut statistics = ModelStatistics::new();

    let texture = TextureStatisticsItem {
        id: "uuid".to_string(),
        size_in_bytes: 10,
    };

    statistics.add_texture(&texture);

    assert_eq!(statistics.points_length, 0);
    assert_eq!(statistics.triangles_length, 0);
    assert_eq!(statistics.geometry_byte_length, 0);
    assert_eq!(statistics.textures_byte_length, 10);
    assert_eq!(statistics.property_tables_byte_length, 0);
    assert_eq!(statistics.batch_textures_byte_length(), 0);
    assert!(statistics.buffer_id_set.is_empty());
    assert_eq!(
        statistics.texture_id_byte_lengths.get("uuid"),
        Some(&10)
    );
    assert!(statistics.batch_texture_id_map.is_empty());

    assert_eq!(statistics.get_texture_ids(), vec!["uuid".to_string()]);
    assert_eq!(statistics.get_texture_byte_length_by_id("uuid"), Some(10));
    assert_eq!(statistics.get_texture_byte_length_by_id("not_found"), None);
}

#[test]
fn test_add_texture_deduplicates_textures() {
    let mut statistics = ModelStatistics::new();

    let texture1 = TextureStatisticsItem {
        id: "uuid1".to_string(),
        size_in_bytes: 2,
    };
    let texture2 = TextureStatisticsItem {
        id: "uuid2".to_string(),
        size_in_bytes: 3,
    };

    statistics.add_texture(&texture1);
    statistics.add_texture(&texture1);
    statistics.add_texture(&texture2);

    assert_eq!(statistics.points_length, 0);
    assert_eq!(statistics.triangles_length, 0);
    assert_eq!(statistics.geometry_byte_length, 0);
    // 2 + 3 = 5. De-duplication avoids 2 * 2 + 3 = 7.
    assert_eq!(statistics.textures_byte_length, 5);
    assert_eq!(statistics.property_tables_byte_length, 0);
    assert_eq!(statistics.batch_textures_byte_length(), 0);
    assert!(statistics.buffer_id_set.is_empty());
    assert_eq!(
        statistics.texture_id_byte_lengths.get("uuid1"),
        Some(&2)
    );
    assert_eq!(
        statistics.texture_id_byte_lengths.get("uuid2"),
        Some(&3)
    );
    assert!(statistics.batch_texture_id_map.is_empty());
}

#[test]
fn test_add_batch_texture_counts_batch_textures_as_they_load() {
    let mut statistics = ModelStatistics::new();

    let batch_texture = Arc::new(DynamicBatchTextureStatistics::new("uuid"));
    statistics.add_batch_texture(batch_texture.clone());

    assert_eq!(statistics.points_length, 0);
    assert_eq!(statistics.triangles_length, 0);
    assert_eq!(statistics.geometry_byte_length, 0);
    assert_eq!(statistics.textures_byte_length, 0);
    assert_eq!(statistics.property_tables_byte_length, 0);
    assert_eq!(statistics.batch_textures_byte_length(), 0);
    assert!(statistics.buffer_id_set.is_empty());
    assert!(statistics.texture_id_byte_lengths.is_empty());
    assert_eq!(statistics.batch_texture_id_map.len(), 1);

    // Simulate creating a batch texture
    batch_texture.set_batch_texture_size(10);
    assert_eq!(statistics.batch_textures_byte_length(), 10);

    // Simulate creating a pick texture
    batch_texture.set_pick_texture_size(15);
    assert_eq!(statistics.batch_textures_byte_length(), 25);
}

#[test]
fn test_add_batch_texture_deduplicates_batch_textures() {
    let mut statistics = ModelStatistics::new();

    let batch_texture1 = Arc::new(DynamicBatchTextureStatistics::new("uuid1"));
    let batch_texture2 = Arc::new(DynamicBatchTextureStatistics::new("uuid2"));

    statistics.add_batch_texture(batch_texture1.clone());
    statistics.add_batch_texture(batch_texture2.clone());
    statistics.add_batch_texture(batch_texture1.clone());

    assert_eq!(statistics.points_length, 0);
    assert_eq!(statistics.triangles_length, 0);
    assert_eq!(statistics.geometry_byte_length, 0);
    assert_eq!(statistics.textures_byte_length, 0);
    assert_eq!(statistics.property_tables_byte_length, 0);
    assert_eq!(statistics.batch_textures_byte_length(), 0);
    assert!(statistics.buffer_id_set.is_empty());
    assert!(statistics.texture_id_byte_lengths.is_empty());
    assert_eq!(statistics.batch_texture_id_map.len(), 2);

    // Simulate creating first batch and pick texture
    batch_texture1.set_batch_texture_size(10);
    batch_texture1.set_pick_texture_size(20);
    assert_eq!(statistics.batch_textures_byte_length(), 30);

    // Simulate creating second batch and pick texture
    batch_texture2.set_batch_texture_size(25);
    batch_texture2.set_pick_texture_size(45);
    assert_eq!(statistics.batch_textures_byte_length(), 100);
}
