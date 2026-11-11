//! ResourceCacheStatisticsSpec matching CesiumJS `Scene/ResourceCacheStatisticsSpec.js` (Apache-2.0).

use cesium_renderer::ResourceCacheStatistics;

#[test]
fn test_constructs() {
    let statistics = ResourceCacheStatistics::new();
    assert_eq!(statistics.geometry_byte_length, 0);
    assert_eq!(statistics.textures_byte_length, 0);
    assert!(statistics.geometry_sizes().is_empty());
    assert!(statistics.texture_sizes().is_empty());
}

#[test]
fn test_clears() {
    let mut statistics = ResourceCacheStatistics::new();

    statistics.add_geometry("vertices", 100).unwrap();
    statistics.add_texture("texture", 300).unwrap();

    assert_ne!(statistics.geometry_byte_length, 0);
    assert_ne!(statistics.textures_byte_length, 0);
    assert!(!statistics.geometry_sizes().is_empty());
    assert!(!statistics.texture_sizes().is_empty());

    statistics.clear();

    assert_eq!(statistics.geometry_byte_length, 0);
    assert_eq!(statistics.textures_byte_length, 0);
    assert!(statistics.geometry_sizes().is_empty());
    assert!(statistics.texture_sizes().is_empty());
}

#[test]
fn test_add_geometry_memory_and_no_double_counting() {
    let mut statistics = ResourceCacheStatistics::new();

    // Counts geometry memory
    statistics.add_geometry("vertices", 100).unwrap();
    assert_eq!(statistics.geometry_byte_length, 100);
    assert_eq!(statistics.textures_byte_length, 0);
    assert_eq!(statistics.geometry_sizes().get("vertices"), Some(&100));
    assert!(statistics.texture_sizes().is_empty());

    // Does not double count same cache key
    statistics.add_geometry("vertices", 100).unwrap();
    assert_eq!(statistics.geometry_byte_length, 100);
    assert_eq!(statistics.textures_byte_length, 0);
}

#[test]
fn test_add_texture_memory_and_no_double_counting() {
    let mut statistics = ResourceCacheStatistics::new();

    // Counts texture memory
    statistics.add_texture("texture", 100).unwrap();
    assert_eq!(statistics.geometry_byte_length, 0);
    assert_eq!(statistics.textures_byte_length, 100);
    assert!(statistics.geometry_sizes().is_empty());
    assert_eq!(statistics.texture_sizes().get("texture"), Some(&100));

    // Does not double count same cache key
    statistics.add_texture("texture", 100).unwrap();
    assert_eq!(statistics.geometry_byte_length, 0);
    assert_eq!(statistics.textures_byte_length, 100);
}

#[test]
fn test_remove_loader_correctly_updates_memory_for_buffers() {
    let mut statistics = ResourceCacheStatistics::new();

    statistics.add_geometry("vertices", 100).unwrap();
    statistics.add_geometry("indices", 200).unwrap();
    statistics.add_texture("texture", 300).unwrap();

    assert_eq!(statistics.geometry_byte_length, 300);
    assert_eq!(statistics.textures_byte_length, 300);

    statistics.remove_by_key("indices").unwrap();

    assert_eq!(statistics.geometry_byte_length, 100);
    assert_eq!(statistics.textures_byte_length, 300);
    assert_eq!(statistics.geometry_sizes().get("vertices"), Some(&100));
    assert_eq!(statistics.geometry_sizes().get("indices"), None);
    assert_eq!(statistics.texture_sizes().get("texture"), Some(&300));
}

#[test]
fn test_remove_loader_correctly_updates_memory_for_textures() {
    let mut statistics = ResourceCacheStatistics::new();

    statistics.add_geometry("vertices", 100).unwrap();
    statistics.add_texture("texture", 300).unwrap();
    statistics.add_texture("texture2", 500).unwrap();

    assert_eq!(statistics.geometry_byte_length, 100);
    assert_eq!(statistics.textures_byte_length, 800);

    statistics.remove_by_key("texture2").unwrap();

    assert_eq!(statistics.geometry_byte_length, 100);
    assert_eq!(statistics.textures_byte_length, 300);
    assert_eq!(statistics.geometry_sizes().get("vertices"), Some(&100));
    assert_eq!(statistics.texture_sizes().get("texture"), Some(&300));
    assert_eq!(statistics.texture_sizes().get("texture2"), None);
}

#[test]
fn test_remove_loader_gracefully_handles_untracked() {
    let mut statistics = ResourceCacheStatistics::new();

    // Removing untracked resource succeeds without error and maintains 0 counts
    assert!(statistics.remove_by_key("untracked").is_ok());
    assert_eq!(statistics.geometry_byte_length, 0);
    assert_eq!(statistics.textures_byte_length, 0);
    assert!(statistics.geometry_sizes().is_empty());
    assert!(statistics.texture_sizes().is_empty());
}

#[test]
fn test_empty_cache_key_errors() {
    let mut statistics = ResourceCacheStatistics::new();
    assert!(statistics.add_geometry("", 100).is_err());
    assert!(statistics.add_texture("", 100).is_err());
    assert!(statistics.remove_by_key("").is_err());
}
