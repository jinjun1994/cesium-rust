use cesium_renderer::Cesium3DTileContentType;

#[test]
fn test_is_binary_format_correctly_identifies_binary_contents() {
    let types = [
        "b3dm",
        "i3dm",
        "glb",
        "vctr",
        "geom",
        "subt",
        "cmpt",
        "pnts",
        "subtreeGlb",
    ];

    for t in types {
        assert!(
            Cesium3DTileContentType::is_binary_format(t),
            "Expected {t} to be binary format"
        );
    }
}

#[test]
fn test_is_binary_format_returns_false_for_other_content_types() {
    let types = [
        "gltf",
        "subtreeJson",
        "subtreeGltf",
        "externalTileset",
        "multipleContent",
        "geoJson",
        "notAMagic",
    ];

    for t in types {
        assert!(
            !Cesium3DTileContentType::is_binary_format(t),
            "Expected {t} NOT to be binary format"
        );
    }
}

#[test]
fn test_is_subtree_correctly_identifies_subtree_contents() {
    let types = ["subt", "subtreeJson", "subtreeGltf", "subtreeGlb"];

    for t in types {
        assert!(
            Cesium3DTileContentType::is_subtree(t),
            "Expected {t} to be subtree"
        );
    }
}

#[test]
fn test_is_subtree_returns_false_for_other_content_types() {
    let types = [
        "b3dm",
        "glb",
        "gltf",
        "externalTileset",
        "multipleContent",
        "geoJson",
        "notAMagic",
    ];

    for t in types {
        assert!(
            !Cesium3DTileContentType::is_subtree(t),
            "Expected {t} NOT to be subtree"
        );
    }
}
