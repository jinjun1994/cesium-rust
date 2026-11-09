use cesium_renderer::{BufferLoader, BufferLoaderOptions};

#[test]
fn test_throws_if_neither_defined() {
    let res = BufferLoader::new(BufferLoaderOptions::default());
    assert!(res.is_err());
}

#[test]
fn test_throws_if_both_defined() {
    let res = BufferLoader::new(BufferLoaderOptions {
        typed_array: Some(vec![1, 2, 3]),
        resource: Some("https://example.com/external.bin".to_string()),
        cache_key: None,
    });
    assert!(res.is_err());
}

#[test]
fn test_load_throws_if_buffer_cannot_be_fetched() {
    pollster::block_on(async {
        let buffer_loader = BufferLoader::new(BufferLoaderOptions {
            typed_array: None,
            resource: Some("https://example.com/external.bin".to_string()),
            cache_key: None,
        })
        .unwrap();

        let fetch_err = |_url| async { Err("404 Not Found".to_string()) };

        let res = buffer_loader.load(Some(fetch_err)).await;
        assert!(res.is_err());
        let err_msg = res.unwrap_err().message;
        assert!(err_msg.contains("Failed to load external buffer: https://example.com/external.bin"));
        assert!(err_msg.contains("404 Not Found"));
    });
}

#[test]
fn test_loads_buffer_from_typed_array() {
    pollster::block_on(async {
        let typed_array = vec![1, 3, 7, 15, 31, 63, 127, 255];
        let buffer_loader = BufferLoader::new(BufferLoaderOptions {
            typed_array: Some(typed_array.clone()),
            resource: None,
            cache_key: None,
        })
        .unwrap();

        buffer_loader
            .load::<fn(String) -> std::future::Ready<Result<Vec<u8>, String>>, _>(None)
            .await
            .unwrap();

        assert_eq!(buffer_loader.typed_array(), Some(typed_array));
    });
}

#[test]
fn test_loads_external_buffer() {
    pollster::block_on(async {
        let array_buffer = vec![1, 3, 7, 15, 31, 63, 127, 255];
        let buffer_loader = BufferLoader::new(BufferLoaderOptions {
            typed_array: None,
            resource: Some("https://example.com/external.bin".to_string()),
            cache_key: None,
        })
        .unwrap();

        let buffer_to_return = array_buffer.clone();
        let fetch_ok = |_url| async move { Ok(buffer_to_return) };

        buffer_loader.load(Some(fetch_ok)).await.unwrap();

        assert_eq!(buffer_loader.typed_array(), Some(array_buffer));
    });
}

#[test]
fn test_destroys_buffer() {
    pollster::block_on(async {
        let array_buffer = vec![1, 3, 7, 15, 31, 63, 127, 255];
        let buffer_loader = BufferLoader::new(BufferLoaderOptions {
            typed_array: None,
            resource: Some("https://example.com/external.bin".to_string()),
            cache_key: None,
        })
        .unwrap();

        assert_eq!(buffer_loader.typed_array(), None);

        let buffer_to_return = array_buffer.clone();
        let fetch_ok = |_url| async move { Ok(buffer_to_return) };
        buffer_loader.load(Some(fetch_ok)).await.unwrap();

        assert_eq!(buffer_loader.typed_array(), Some(array_buffer));
        assert!(!buffer_loader.is_destroyed());

        buffer_loader.destroy();
        assert_eq!(buffer_loader.typed_array(), None);
        assert!(buffer_loader.is_destroyed());
    });
}

#[test]
fn test_handles_async_load_after_destroy() {
    pollster::block_on(async {
        let buffer_loader = BufferLoader::new(BufferLoaderOptions {
            typed_array: None,
            resource: Some("https://example.com/external.bin".to_string()),
            cache_key: None,
        })
        .unwrap();

        assert_eq!(buffer_loader.typed_array(), None);

        // Destroy before/during load
        buffer_loader.destroy();

        let fetch_ok = |_url| async { Ok(vec![1, 2, 3]) };
        let res = buffer_loader.load(Some(fetch_ok)).await;
        assert!(res.is_ok());

        assert_eq!(buffer_loader.typed_array(), None);
        assert!(buffer_loader.is_destroyed());
    });
}

#[test]
fn test_handles_async_error_after_destroy() {
    pollster::block_on(async {
        let buffer_loader = BufferLoader::new(BufferLoaderOptions {
            typed_array: None,
            resource: Some("https://example.com/external.bin".to_string()),
            cache_key: None,
        })
        .unwrap();

        assert_eq!(buffer_loader.typed_array(), None);

        // Destroy before/during load
        buffer_loader.destroy();

        let fetch_err = |_url| async { Err("Some error".to_string()) };
        let res = buffer_loader.load(Some(fetch_err)).await;
        assert!(res.is_ok());

        assert_eq!(buffer_loader.typed_array(), None);
        assert!(buffer_loader.is_destroyed());
    });
}
