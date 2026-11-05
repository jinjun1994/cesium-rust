use cesium_math::{
    Credit, VRTheWorldTerrainProvider, VRTheWorldTerrainProviderOptions,
};

const VALID_XML: &str = r#"
<TileMap version="1.0.0" tilemapservice="http://www.vr-theworld.com/vr-theworld/tiles/1.0.0/">
    <Title>Hawaii World elev</Title>
    <Abstract>layer to make cesium work right</Abstract>
    <SRS>EPSG:4326</SRS>
    <BoundingBox minx="-180.000000" miny="-90.000000" maxx="180.000000" maxy="90.000000"/>
    <Origin x="-180.000000" y="-90.000000"/>
    <TileFormat width="32" height="32" mime-type="image/tif" extension="tif"/>
    <TileSets>
        <TileSet href="http://www.vr-theworld.com/vr-theworld/tiles/1.0.0/73/0" units-per-pixel="5.62500000000000000000" order="0"/>
        <TileSet href="http://www.vr-theworld.com/vr-theworld/tiles/1.0.0/73/1" units-per-pixel="2.81250000000000000000" order="1"/>
        <TileSet href="http://www.vr-theworld.com/vr-theworld/tiles/1.0.0/73/2" units-per-pixel="1.40625000000000000000" order="2"/>
        <TileSet href="http://www.vr-theworld.com/vr-theworld/tiles/1.0.0/73/3" units-per-pixel="0.70312500000000000000" order="3"/>
    </TileSets>
    <DataExtents>
        <DataExtent minx="-180.000000" miny="-90.000000" maxx="180.000000" maxy="90.000000" minlevel="0" maxlevel="9"/>
        <DataExtent minx="24.999584" miny="-0.000417" maxx="30.000417" maxy="5.000417" minlevel="0" maxlevel="13"/>
    </DataExtents>
</TileMap>
"#;

#[test]
fn test_from_url_rejects_without_url() {
    assert!(VRTheWorldTerrainProvider::from_url("", None).is_err());
}

#[test]
fn test_from_xml_resolves_to_new_provider() {
    let provider = VRTheWorldTerrainProvider::from_xml("made/up/url", VALID_XML, None).unwrap();
    assert_eq!(provider.heightmap_width(), 32);
    assert_eq!(provider.heightmap_height(), 32);
    assert_eq!(provider.tiling_scheme().number_of_x_tiles_at_level(0), 2);
    assert!(!provider.has_water_mask());
    assert!(!provider.has_vertex_normals());
    assert!(provider.credit().is_none());
}

#[test]
fn test_returns_reasonable_geometric_error_for_various_levels() {
    let provider = VRTheWorldTerrainProvider::from_xml("made/up/url", VALID_XML, None).unwrap();

    let err0 = provider.get_level_maximum_geometric_error(0);
    let err1 = provider.get_level_maximum_geometric_error(1);
    let err2 = provider.get_level_maximum_geometric_error(2);

    assert!(err0 > 0.0);
    assert!((err0 - err1 * 2.0).abs() < 1e-10);
    assert!((err1 - err2 * 2.0).abs() < 1e-10);
}

#[test]
fn test_credit_handling() {
    let provider = VRTheWorldTerrainProvider::from_xml("made/up/url", VALID_XML, None).unwrap();
    assert!(provider.credit().is_none());

    let credit = Credit::new("thanks to our awesome made up contributors!", Some(false));
    let provider_with_credit = VRTheWorldTerrainProvider::from_xml(
        "made/up/url",
        VALID_XML,
        Some(VRTheWorldTerrainProviderOptions {
            credit: Some(credit),
        }),
    )
    .unwrap();
    assert!(provider_with_credit.credit().is_some());
    assert_eq!(
        provider_with_credit.credit().unwrap().html(),
        "thanks to our awesome made up contributors!"
    );
}

#[test]
fn test_throws_if_srs_is_not_supported() {
    let invalid_srs_xml = VALID_XML.replace("<SRS>EPSG:4326</SRS>", "<SRS>EPSG:foo</SRS>");
    let res = VRTheWorldTerrainProvider::from_xml("made/up/url", &invalid_srs_xml, None);
    assert!(res.is_err());
    let err = res.unwrap_err();
    assert!(err.message.contains("SRS EPSG:foo is not supported"));
}

#[test]
fn test_request_tile_geometry_and_tile_url() {
    let provider = VRTheWorldTerrainProvider::from_xml("made/up/url", VALID_XML, None).unwrap();

    let tile_url = provider.build_tile_url(0, 0, 0);
    assert!(tile_url.contains(".tif?cesium=true"));

    let data = provider.request_tile_geometry(0, 0, 0).unwrap();
    assert_eq!(data.width(), 32);
    assert_eq!(data.height(), 32);
    assert_eq!(data.child_tile_mask(), 15);
}
