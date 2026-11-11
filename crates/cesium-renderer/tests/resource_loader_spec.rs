//! ResourceLoaderSpec matching CesiumJS `Scene/ResourceLoaderSpec.js` (Apache-2.0).

use cesium_renderer::BaseResourceLoader;

#[test]
fn test_throws_when_using_resource_loader_directly() {
    let loader = BaseResourceLoader::new();
    assert!(loader.cache_key().is_err());
    assert!(loader.load().is_err());
}

#[test]
fn test_get_error_from_error_message() {
    let loader = BaseResourceLoader::new();
    let error_message = "Resource failed";
    let runtime_error = loader.get_error(Some(error_message), None).unwrap();
    assert_eq!(runtime_error.message, error_message);
}

#[test]
fn test_get_error_from_error_message_and_error() {
    let loader = BaseResourceLoader::new();
    let error_message = "Resource failed";
    let sub_error = "404 Not Found";
    let runtime_error = loader
        .get_error(Some(error_message), Some(sub_error))
        .unwrap();
    assert_eq!(runtime_error.message, "Resource failed\n404 Not Found");
}

#[test]
fn test_get_error_throws_if_error_message_is_undefined() {
    let loader = BaseResourceLoader::new();
    assert!(loader.get_error(None, None).is_err());
}

#[test]
fn test_destroys() {
    let loader = BaseResourceLoader::new();
    assert!(!loader.is_destroyed());
    loader.destroy();
    assert!(loader.is_destroyed());
}
