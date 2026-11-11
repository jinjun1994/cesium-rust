use cesium_math::Matrix3;
use cesium_renderer::gltf_loader_util::{
    CreateModelTextureReaderOptions, CreateSamplerOptions, GetImageIdFromTextureOptions,
    GltfLoaderUtil,
};
use cesium_renderer::gltf_texture_loader::SupportedImageFormats;
use cesium_renderer::sampler::{
    TextureMagnificationFilter, TextureMinificationFilter, TextureWrap,
};
use cesium_renderer::texture::{Texture, TextureOptions};
use serde_json::json;

fn make_test_gltf() -> serde_json::Value {
    json!({
        "images": [
            { "uri": "image.png" },
            { "mimeType": "image/jpeg", "bufferView": 0 },
            { "uri": "image.webp" },
            { "uri": "image.ktx2" }
        ],
        "textures": [
            { "source": 0 },
            {
                "source": 0,
                "extensions": {
                    "EXT_texture_webp": { "source": 2 }
                }
            },
            {
                "source": 0,
                "extensions": {
                    "KHR_texture_basisu": { "source": 3 }
                }
            }
        ],
        "samplers": [
            {
                "magFilter": 9729,
                "minFilter": 9986, // NEAREST_MIPMAP_LINEAR
                "wrapS": 10497,
                "wrapT": 10497
            },
            {
                "magFilter": 9729,
                "minFilter": 9987, // LINEAR_MIPMAP_LINEAR
                "wrapS": 10497,
                "wrapT": 10497
            }
        ]
    })
}

#[test]
fn get_image_id_from_texture_throws_if_options_missing() {
    let gltf = make_test_gltf();

    assert!(GltfLoaderUtil::get_image_id_from_texture(GetImageIdFromTextureOptions {
        gltf: None,
        texture_id: Some(0),
        supported_image_formats: Some(SupportedImageFormats::default()),
    })
    .is_err());

    assert!(GltfLoaderUtil::get_image_id_from_texture(GetImageIdFromTextureOptions {
        gltf: Some(&gltf),
        texture_id: None,
        supported_image_formats: Some(SupportedImageFormats::default()),
    })
    .is_err());

    assert!(GltfLoaderUtil::get_image_id_from_texture(GetImageIdFromTextureOptions {
        gltf: Some(&gltf),
        texture_id: Some(0),
        supported_image_formats: None,
    })
    .is_err());
}

#[test]
fn get_image_id_from_texture_extension_negotiation() {
    let gltf = make_test_gltf();

    // Default image id
    let id0 = GltfLoaderUtil::get_image_id_from_texture(GetImageIdFromTextureOptions {
        gltf: Some(&gltf),
        texture_id: Some(0),
        supported_image_formats: Some(SupportedImageFormats::default()),
    })
    .unwrap();
    assert_eq!(id0, 0);

    // WebP supported vs unsupported
    let id_webp_ok = GltfLoaderUtil::get_image_id_from_texture(GetImageIdFromTextureOptions {
        gltf: Some(&gltf),
        texture_id: Some(1),
        supported_image_formats: Some(SupportedImageFormats::with_flags(false, true)),
    })
    .unwrap();
    assert_eq!(id_webp_ok, 2);

    let id_webp_no = GltfLoaderUtil::get_image_id_from_texture(GetImageIdFromTextureOptions {
        gltf: Some(&gltf),
        texture_id: Some(1),
        supported_image_formats: Some(SupportedImageFormats::with_flags(false, false)),
    })
    .unwrap();
    assert_eq!(id_webp_no, 0);

    // Basis supported vs unsupported
    let id_basis_ok = GltfLoaderUtil::get_image_id_from_texture(GetImageIdFromTextureOptions {
        gltf: Some(&gltf),
        texture_id: Some(2),
        supported_image_formats: Some(SupportedImageFormats::with_flags(true, false)),
    })
    .unwrap();
    assert_eq!(id_basis_ok, 3);

    let id_basis_no = GltfLoaderUtil::get_image_id_from_texture(GetImageIdFromTextureOptions {
        gltf: Some(&gltf),
        texture_id: Some(2),
        supported_image_formats: Some(SupportedImageFormats::with_flags(false, false)),
    })
    .unwrap();
    assert_eq!(id_basis_no, 0);
}

#[test]
fn create_sampler_validation_and_defaults() {
    let gltf = make_test_gltf();
    let tex_info = json!({ "index": 0 });

    assert!(GltfLoaderUtil::create_sampler(CreateSamplerOptions {
        gltf: None,
        texture_info: Some(&tex_info),
        compressed_texture_no_mipmap: None,
    })
    .is_err());

    assert!(GltfLoaderUtil::create_sampler(CreateSamplerOptions {
        gltf: Some(&gltf),
        texture_info: None,
        compressed_texture_no_mipmap: None,
    })
    .is_err());

    // Texture 0 has no sampler, returns default
    let default_sampler = GltfLoaderUtil::create_sampler(CreateSamplerOptions {
        gltf: Some(&gltf),
        texture_info: Some(&tex_info),
        compressed_texture_no_mipmap: None,
    })
    .unwrap();
    assert_eq!(default_sampler.wrap_s, TextureWrap::Repeat);
    assert_eq!(default_sampler.wrap_t, TextureWrap::Repeat);
    assert_eq!(
        default_sampler.minification_filter,
        TextureMinificationFilter::Linear
    );
    assert_eq!(
        default_sampler.magnification_filter,
        TextureMagnificationFilter::Linear
    );
}

#[test]
fn create_sampler_compressed_texture_mipmap_downgrade() {
    let gltf = json!({
        "textures": [
            { "source": 0, "sampler": 0 },
            { "source": 0, "sampler": 1 }
        ],
        "samplers": [
            {
                "magFilter": 9729,
                "minFilter": 9986, // NEAREST_MIPMAP_LINEAR -> NEAREST
                "wrapS": 10497,
                "wrapT": 10497
            },
            {
                "magFilter": 9729,
                "minFilter": 9987, // LINEAR_MIPMAP_LINEAR -> LINEAR
                "wrapS": 10497,
                "wrapT": 10497
            }
        ]
    });

    let s0 = GltfLoaderUtil::create_sampler(CreateSamplerOptions {
        gltf: Some(&gltf),
        texture_info: Some(&json!({ "index": 0 })),
        compressed_texture_no_mipmap: Some(true),
    })
    .unwrap();
    assert_eq!(s0.minification_filter, TextureMinificationFilter::Nearest);
    assert_eq!(s0.magnification_filter, TextureMagnificationFilter::Linear);

    let s1 = GltfLoaderUtil::create_sampler(CreateSamplerOptions {
        gltf: Some(&gltf),
        texture_info: Some(&json!({ "index": 1 })),
        compressed_texture_no_mipmap: Some(true),
    })
    .unwrap();
    assert_eq!(s1.minification_filter, TextureMinificationFilter::Linear);
    assert_eq!(s1.magnification_filter, TextureMagnificationFilter::Linear);
}

#[test]
fn create_sampler_falls_back_for_invalid_parameters() {
    let gltf = json!({
        "textures": [{ "source": 0, "sampler": 0 }],
        "samplers": [
            {
                "magFilter": -1,
                "minFilter": -1,
                "wrapS": -1,
                "wrapT": -1
            }
        ]
    });

    let sampler = GltfLoaderUtil::create_sampler(CreateSamplerOptions {
        gltf: Some(&gltf),
        texture_info: Some(&json!({ "index": 0 })),
        compressed_texture_no_mipmap: None,
    })
    .unwrap();

    assert_eq!(sampler.wrap_s, TextureWrap::Repeat);
    assert_eq!(sampler.wrap_t, TextureWrap::Repeat);
    assert_eq!(
        sampler.minification_filter,
        TextureMinificationFilter::Linear
    );
    assert_eq!(
        sampler.magnification_filter,
        TextureMagnificationFilter::Linear
    );
}

#[test]
fn create_model_texture_reader_basic_and_transform() {
    // 1. Throws if texture_info missing
    assert!(GltfLoaderUtil::create_model_texture_reader(CreateModelTextureReaderOptions {
        texture_info: None,
        channels: None,
        texture: None,
    })
    .is_err());

    // 2. Default reader
    let tex_info_default = json!({ "index": 0 });
    let reader_default = GltfLoaderUtil::create_model_texture_reader(
        CreateModelTextureReaderOptions {
            texture_info: Some(&tex_info_default),
            channels: None,
            texture: None,
        },
    )
    .unwrap();
    assert_eq!(reader_default.index, 0);
    assert_eq!(reader_default.tex_coord, 0);
    assert!(reader_default.texture.is_none());
    assert!(reader_default.transform.is_none());
    assert!(reader_default.channels.is_none());

    // 3. KHR_texture_transform (offset and scale)
    let tex_info_transform = json!({
        "index": 0,
        "texCoord": 0,
        "extensions": {
            "KHR_texture_transform": {
                "offset": [0.5, 0.5],
                "scale": [0.1, 0.2],
                "texCoord": 1
            }
        }
    });

    let expected_transform = Matrix3::new(
        0.1, 0.0, 0.5,
        0.0, 0.2, 0.5,
        0.0, 0.0, 1.0,
    );

    let reader_trans = GltfLoaderUtil::create_model_texture_reader(
        CreateModelTextureReaderOptions {
            texture_info: Some(&tex_info_transform),
            channels: None,
            texture: None,
        },
    )
    .unwrap();
    assert_eq!(reader_trans.tex_coord, 1);
    assert!(reader_trans
        .transform
        .unwrap()
        .equals_epsilon(&expected_transform, 1e-6));

    // 4. KHR_texture_transform rotation
    let angle = std::f64::consts::FRAC_PI_2;
    let tex_info_rotation = json!({
        "index": 0,
        "texCoord": 0,
        "extensions": {
            "KHR_texture_transform": {
                "rotation": angle,
                "texCoord": 1
            }
        }
    });

    let expected_rot_transform = Matrix3::new(
        (-angle).cos(), -(-angle).sin(), 0.0,
        (-angle).sin(),  (-angle).cos(), 0.0,
        0.0,            0.0,            1.0,
    );

    let reader_rot = GltfLoaderUtil::create_model_texture_reader(
        CreateModelTextureReaderOptions {
            texture_info: Some(&tex_info_rotation),
            channels: None,
            texture: None,
        },
    )
    .unwrap();
    assert_eq!(reader_rot.tex_coord, 1);
    assert!(reader_rot
        .transform
        .unwrap()
        .equals_epsilon(&expected_rot_transform, 1e-6));

    // 5. With texture and channels
    let mut tex = Texture::new(TextureOptions {
        width: 1,
        height: 1,
        ..Default::default()
    });
    let tex_info_with_tex = json!({
        "index": 0,
        "texCoord": 1
    });

    let reader_full = GltfLoaderUtil::create_model_texture_reader(
        CreateModelTextureReaderOptions {
            texture_info: Some(&tex_info_with_tex),
            channels: Some("r".to_string()),
            texture: Some(tex.clone()),
        },
    )
    .unwrap();
    assert_eq!(reader_full.tex_coord, 1);
    assert_eq!(reader_full.channels.as_deref(), Some("r"));
    assert_eq!(reader_full.texture.as_ref().unwrap().id, tex.id);

    tex.destroy();
}
