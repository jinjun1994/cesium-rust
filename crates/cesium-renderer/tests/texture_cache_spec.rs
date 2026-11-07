use std::cell::RefCell;
use std::rc::Rc;
use cesium_renderer::{Texture, TextureCache, TextureOptions};

#[test]
fn test_adds_and_removes() {
    let cache = TextureCache::new();

    let keyword = "texture";
    let texture = Rc::new(RefCell::new(Texture::new(TextureOptions {
        width: 1,
        height: 1,
        ..Default::default()
    })));

    cache.add_texture(keyword, texture.clone());

    assert_eq!(cache.number_of_textures(), 1);

    texture.borrow_mut().destroy();
    assert!(!texture.borrow().is_destroyed());
    assert_eq!(cache.number_of_textures(), 1);

    cache.destroy_released_textures();
    assert!(texture.borrow().is_destroyed());
    assert_eq!(cache.number_of_textures(), 0);

    cache.destroy();
}

#[test]
fn test_has_a_cache_hit() {
    let cache = TextureCache::new();

    let keyword = "texture";
    let texture = Rc::new(RefCell::new(Texture::new(TextureOptions {
        width: 1,
        height: 1,
        ..Default::default()
    })));

    cache.add_texture(keyword, texture.clone());

    let texture2 = cache.get_texture(keyword).unwrap();
    assert!(Rc::ptr_eq(&texture, &texture2));
    assert_eq!(cache.number_of_textures(), 1);

    texture.borrow_mut().destroy();
    texture2.borrow_mut().destroy();
    cache.destroy_released_textures();

    assert!(texture.borrow().is_destroyed());
    assert_eq!(cache.number_of_textures(), 0);

    cache.destroy();
}

#[test]
fn test_avoids_thrashing() {
    let cache = TextureCache::new();

    let keyword = "texture";
    let texture = Rc::new(RefCell::new(Texture::new(TextureOptions {
        width: 1,
        height: 1,
        ..Default::default()
    })));

    cache.add_texture(keyword, texture.clone());
    texture.borrow_mut().destroy();

    let texture2 = cache.get_texture(keyword).unwrap(); // still cache hit

    cache.destroy_released_textures(); // does not destroy
    assert!(!texture.borrow().is_destroyed());
    assert!(!texture2.borrow().is_destroyed());

    texture2.borrow_mut().destroy();
    cache.destroy_released_textures(); // now destroys

    assert!(texture.borrow().is_destroyed());
    assert!(texture2.borrow().is_destroyed());

    cache.destroy();
}

#[test]
fn test_is_destroyed_and_not_destroyed() {
    let cache = TextureCache::new();
    assert!(!cache.is_destroyed());

    let keyword = "texture";
    let texture = Rc::new(RefCell::new(Texture::new(TextureOptions {
        width: 1,
        height: 1,
        ..Default::default()
    })));

    cache.add_texture(keyword, texture.clone());
    cache.destroy();

    assert!(texture.borrow().is_destroyed());
    assert!(cache.is_destroyed());
}
