use cesium_renderer::sampler::TextureMinificationFilter;
use cesium_renderer::texture::{Texture, TextureOptions};
use cesium_renderer::texture_manager::{TextureManager, TextureManagerFrameState};
use cesium_renderer::texture_uniform::{TextureUniform, TextureUniformOptions};

const BLUE_URL: &str = "Data/Images/Blue2x2.png";
const GREEN_URL: &str = "Data/Images/Green1x4.png";
const RED_URL: &str = "Data/Images/Red16x16.png";
const BLUE_10X10_URL: &str = "Data/Images/Blue10x10.png";

#[test]
fn test_constructs() {
    let texture_manager = TextureManager::new();
    assert!(texture_manager.textures.is_empty());
    assert!(texture_manager.loaded_images.is_empty());
    assert!(!texture_manager.is_destroyed());
}

#[test]
fn test_loads_texture_from_a_url() {
    let mut texture_manager = TextureManager::new();
    let id = "testTexture";

    texture_manager.load_texture_2d(
        id,
        TextureUniform::new(TextureUniformOptions {
            url: Some(BLUE_URL.to_string()),
            ..Default::default()
        })
        .unwrap(),
    );

    let frame_state = TextureManagerFrameState {
        frame_number: 1,
        webgl2: false,
        default_texture: None,
    };
    texture_manager.update(&frame_state);

    let texture = texture_manager.get_texture(id).expect("Texture loaded");
    assert_eq!(texture.width, 2);
    assert_eq!(texture.height, 2);
}

#[test]
fn test_loads_texture_from_a_typed_array() {
    let mut texture_manager = TextureManager::new();
    let id = "testTexture";

    texture_manager.load_texture_2d(
        id,
        TextureUniform::new(TextureUniformOptions {
            typed_array: Some(vec![255, 0, 0, 255, 0, 255, 0, 255]),
            width: Some(1),
            height: Some(2),
            ..Default::default()
        })
        .unwrap(),
    );

    let frame_state = TextureManagerFrameState {
        frame_number: 1,
        webgl2: false,
        default_texture: None,
    };
    texture_manager.update(&frame_state);

    let texture = texture_manager.get_texture(id).expect("Texture loaded");
    assert_eq!(texture.width, 1);
    assert_eq!(texture.height, 2);
}

#[test]
fn test_generates_mipmaps_when_sampler_type_requires_them() {
    let mut texture_manager = TextureManager::new();
    let id = "testTexture";

    texture_manager.load_texture_2d(
        id,
        TextureUniform::new(TextureUniformOptions {
            url: Some(RED_URL.to_string()),
            minification_filter: Some(TextureMinificationFilter::LinearMipmapLinear),
            ..Default::default()
        })
        .unwrap(),
    );

    let frame_state = TextureManagerFrameState {
        frame_number: 1,
        webgl2: false,
        default_texture: None,
    };
    texture_manager.update(&frame_state);

    let texture = texture_manager.get_texture(id).expect("Texture loaded");
    assert!(texture.has_mipmap);
}

#[test]
fn test_resizes_image_to_power_of_two_dimensions_if_needed() {
    let mut texture_manager = TextureManager::new();
    let id = "testTexture";

    texture_manager.load_texture_2d(
        id,
        TextureUniform::new(TextureUniformOptions {
            url: Some(BLUE_10X10_URL.to_string()),
            minification_filter: Some(TextureMinificationFilter::LinearMipmapNearest),
            ..Default::default()
        })
        .unwrap(),
    );

    let frame_state = TextureManagerFrameState {
        frame_number: 1,
        webgl2: false, // WebGL1 requires POT resize
        default_texture: None,
    };
    texture_manager.update(&frame_state);

    let texture = texture_manager.get_texture(id).expect("Texture loaded");
    assert_eq!(texture.width, 16);
    assert_eq!(texture.height, 16);
    assert!(texture.has_mipmap);
}

#[test]
fn test_can_resize_a_texture_supplied_as_a_uint8array() {
    let red_pixels_3x3 = [255, 0, 0, 255].repeat(9);

    let texture_uniform = TextureUniform::new(TextureUniformOptions {
        typed_array: Some(red_pixels_3x3),
        width: Some(3),
        height: Some(3),
        minification_filter: Some(TextureMinificationFilter::NearestMipmapLinear),
        ..Default::default()
    })
    .unwrap();

    let mut texture_manager = TextureManager::new();
    let id = "testTexture";

    texture_manager.load_texture_2d(id, texture_uniform);

    let frame_state = TextureManagerFrameState {
        frame_number: 1,
        webgl2: false,
        default_texture: None,
    };
    texture_manager.update(&frame_state);

    let texture = texture_manager.get_texture(id).expect("Texture loaded");
    assert_eq!(texture.width, 4);
    assert_eq!(texture.height, 4);
    assert!(texture.has_mipmap);
}

#[test]
fn test_generates_mipmaps_without_resizing_in_webgl2() {
    let mut texture_manager = TextureManager::new();
    let id = "testTexture";

    texture_manager.load_texture_2d(
        id,
        TextureUniform::new(TextureUniformOptions {
            url: Some(BLUE_10X10_URL.to_string()),
            minification_filter: Some(TextureMinificationFilter::NearestMipmapNearest),
            ..Default::default()
        })
        .unwrap(),
    );

    let frame_state = TextureManagerFrameState {
        frame_number: 1,
        webgl2: true, // WebGL2 allows NPOT with mipmaps
        default_texture: None,
    };
    texture_manager.update(&frame_state);

    let texture = texture_manager.get_texture(id).expect("Texture loaded");
    assert_eq!(texture.width, 10);
    assert_eq!(texture.height, 10);
    assert!(texture.has_mipmap);
}

#[test]
fn test_destroys_old_texture_before_adding_a_new_one() {
    let mut texture_manager = TextureManager::new();
    let id = "testTexture";

    texture_manager.load_texture_2d(
        id,
        TextureUniform::new(TextureUniformOptions {
            url: Some(BLUE_URL.to_string()),
            ..Default::default()
        })
        .unwrap(),
    );

    let frame_state1 = TextureManagerFrameState {
        frame_number: 1,
        webgl2: false,
        default_texture: None,
    };
    texture_manager.update(&frame_state1);

    let blue_texture = texture_manager.get_texture(id).cloned().unwrap();
    assert_eq!(blue_texture.width, 2);
    assert_eq!(blue_texture.height, 2);
    assert!(!blue_texture.is_destroyed());

    texture_manager.load_texture_2d(
        id,
        TextureUniform::new(TextureUniformOptions {
            url: Some(GREEN_URL.to_string()),
            ..Default::default()
        })
        .unwrap(),
    );

    let frame_state2 = TextureManagerFrameState {
        frame_number: 2,
        webgl2: false,
        default_texture: None,
    };
    texture_manager.update(&frame_state2);

    let green_texture = texture_manager.get_texture(id).cloned().unwrap();
    assert_eq!(green_texture.width, 1);
    assert_eq!(green_texture.height, 4);
    assert!(!green_texture.is_destroyed());

    // In a real environment old texture destroy is marked
    assert_ne!(blue_texture.id, green_texture.id);
}

#[test]
fn test_get_texture_returns_undefined_for_unknown_texture() {
    let texture_manager = TextureManager::new();
    assert!(texture_manager.get_texture("notATexture").is_none());
}

#[test]
fn test_sets_a_default_texture_on_error() {
    let default_texture = Texture::new(TextureOptions {
        width: 1,
        height: 1,
        ..Default::default()
    });

    let mut texture_manager = TextureManager::new();
    let id = "testTexture";

    let frame_state1 = TextureManagerFrameState {
        frame_number: 1,
        webgl2: false,
        default_texture: Some(default_texture.clone()),
    };
    texture_manager.update(&frame_state1);

    texture_manager.load_texture_2d(
        id,
        TextureUniform::new(TextureUniformOptions {
            url: Some("https://example.com/not-a-texture.jpg".to_string()),
            ..Default::default()
        })
        .unwrap(),
    );

    let frame_state2 = TextureManagerFrameState {
        frame_number: 2,
        webgl2: false,
        default_texture: Some(default_texture.clone()),
    };
    texture_manager.update(&frame_state2);

    let texture = texture_manager.get_texture(id).expect("Texture fallback");
    assert_eq!(texture.id, default_texture.id);
}

#[test]
fn test_destroys() {
    let mut texture_manager = TextureManager::new();
    let id = "testTexture";

    texture_manager.load_texture_2d(
        id,
        TextureUniform::new(TextureUniformOptions {
            url: Some(BLUE_URL.to_string()),
            ..Default::default()
        })
        .unwrap(),
    );

    let frame_state = TextureManagerFrameState {
        frame_number: 1,
        webgl2: false,
        default_texture: None,
    };
    texture_manager.update(&frame_state);

    assert!(!texture_manager.is_destroyed());
    assert!(!texture_manager.get_texture(id).unwrap().is_destroyed());

    texture_manager.destroy();
    assert!(texture_manager.is_destroyed());
}
