use cesium_renderer::imagery_layer::ImageryProvider;
use cesium_renderer::quadtree_tile::AnyTilingScheme;
use cesium_renderer::TileMapServiceImageryProvider;

const VALID_SAMPLE_XML: &str = r#"<TileMap version="1.0.0" tilemapservice="http://tms.osgeo.org/1.0.0">
    <Title>NE2_HR_LC_SR_W_DR_recolored.tif</Title>
    <Abstract></Abstract>
    <SRS>EPSG:4326</SRS>
    <BoundingBox miny="-90.00000000000000" minx="-180.00000000000000" maxy="90.00000000000000" maxx="180.00000000000000"/>
    <Origin y="-90.00000000000000" x="-180.00000000000000"/>
    <TileFormat width="256" height="256" mime-type="image/jpg" extension="jpg"/>
    <TileSets profile="geodetic">
        <TileSet href="0" units-per-pixel="0.70312500000000" order="0"/>
        <TileSet href="1" units-per-pixel="0.35156250000000" order="1"/>
        <TileSet href="2" units-per-pixel="0.17578125000000" order="2"/>
    </TileSets>
</TileMap>"#;

#[test]
fn test_from_url_throws_without_url() {
    let result = TileMapServiceImageryProvider::from_url("", None);
    assert!(result.is_err());
}

#[test]
fn test_from_url_default_fallback() {
    let provider = TileMapServiceImageryProvider::from_url("made/up/tms/server/", None).unwrap();
    assert_eq!(provider.file_extension(), "png");
    assert_eq!(provider.tile_width(), 256);
    assert_eq!(provider.tile_height(), 256);
    assert!(matches!(provider.tiling_scheme(), AnyTilingScheme::WebMercator(_)));
    assert_eq!(
        provider.url_template(),
        "made/up/tms/server/{z}/{x}/{reverseY}.png"
    );
}

#[test]
fn test_from_xml_valid_sample() {
    let provider = TileMapServiceImageryProvider::from_xml(
        VALID_SAMPLE_XML,
        "made/up/tms/server/",
        None,
    )
    .unwrap();

    assert_eq!(provider.file_extension(), "jpg");
    assert_eq!(provider.tile_width(), 256);
    assert_eq!(provider.tile_height(), 256);
    assert_eq!(provider.minimum_level(), 0);
    assert_eq!(provider.maximum_level(), Some(2));
    assert!(matches!(provider.tiling_scheme(), AnyTilingScheme::Geographic(_)));

    let tile_url = provider.tile_url(0, 0, 0);
    assert!(tile_url.starts_with("made/up/tms/server/0/0/"));
    assert!(tile_url.ends_with(".jpg"));
}

#[test]
fn test_from_xml_throws_on_unsupported_profile() {
    let xml = r#"<TileMap version="1.0.0" tilemapservice="http://tms.osgeo.org/1.0.0">
        <Title/>
        <Abstract/>
        <SRS>EPSG:4326</SRS>
        <BoundingBox minx="-10.0" miny="-123.0" maxx="11.0" maxy="-110.0"/>
        <Origin x="-90.0" y="-180.0"/>
        <TileFormat width="256" height="256" mime-type="image/png" extension="png"/>
        <TileSets profile="foobar">
            <TileSet href="2" units-per-pixel="39135.75848201024200" order="2"/>
            <TileSet href="3" units-per-pixel="19567.87924100512100" order="3"/>
        </TileSets>
    </TileMap>"#;

    let result = TileMapServiceImageryProvider::from_xml(xml, "made/up/tms/server", None);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err
        .message
        .contains("specifies an unsupported profile attribute, foobar"));
}

#[test]
fn test_from_xml_throws_on_missing_tilesets_or_bbox() {
    let xml = r#"<TileMap version="1.0.0">
        <Title/>
    </TileMap>"#;

    let result = TileMapServiceImageryProvider::from_xml(xml, "made/up/tms/server", None);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err
        .message
        .contains("Unable to find expected tilesets or bbox attributes"));
}

#[test]
fn test_conforms_to_imagery_provider_interface() {
    let provider = TileMapServiceImageryProvider::from_url("my/tms", None).unwrap();
    let trait_obj: &dyn ImageryProvider = &provider;
    assert_eq!(trait_obj.tile_width(), 256);
    assert_eq!(trait_obj.tile_height(), 256);
    assert!(trait_obj.has_alpha_channel());
}
