use std::f64::consts::PI;

use cesium_math::{
    Cartesian3, Cartographic, Ellipsoid, GeoRectangle, GeographicTilingScheme,
    WebMercatorTilingScheme,
};
use cesium_renderer::quadtree_tile::{
    AnyTilingScheme, CustomDataEntry, QuadtreeTile, QuadtreeTileOptions,
};

#[test]
fn test_creates_rectangle_on_construction() {
    let wm = WebMercatorTilingScheme::new(Ellipsoid::WGS84, None, 1, 1).unwrap();
    let scheme = AnyTilingScheme::WebMercator(wm);

    let tile = QuadtreeTile::new(QuadtreeTileOptions {
        x: 0,
        y: 0,
        level: 0,
        tiling_scheme: scheme.clone(),
        parent: None,
    })
    .unwrap();

    let expected_rect = scheme.tile_xy_to_rectangle(0, 0, 0).unwrap();
    assert_eq!(tile.borrow().rectangle, expected_rect);
}

#[test]
fn test_can_get_tiles_around_a_root_tile() {
    let geo = GeographicTilingScheme::new(Ellipsoid::WGS84, GeoRectangle::MAXIMUM, 3, 3).unwrap();
    let scheme = AnyTilingScheme::Geographic(geo);
    let level_zero_tiles = QuadtreeTile::create_level_zero_tiles(scheme).unwrap();

    let get_tile = |x: u32, y: u32| {
        level_zero_tiles
            .iter()
            .find(|t| t.borrow().x == x && t.borrow().y == y)
            .unwrap()
            .clone()
    };

    let l0x0y0 = get_tile(0, 0);
    let l0x1y0 = get_tile(1, 0);
    let l0x2y0 = get_tile(2, 0);
    let l0x0y1 = get_tile(0, 1);
    let l0x1y1 = get_tile(1, 1);
    let l0x2y1 = get_tile(2, 1);
    let _l0x0y2 = get_tile(0, 2);
    let l0x1y2 = get_tile(1, 2);
    let _l0x2y2 = get_tile(2, 2);

    assert_eq!(
        QuadtreeTile::find_tile_to_west(&l0x0y0, &level_zero_tiles).unwrap().borrow().x,
        l0x2y0.borrow().x
    );
    assert_eq!(
        QuadtreeTile::find_tile_to_east(&l0x0y0, &level_zero_tiles).unwrap().borrow().x,
        l0x1y0.borrow().x
    );
    assert!(QuadtreeTile::find_tile_to_north(&l0x0y0, &level_zero_tiles).is_none());
    assert_eq!(
        QuadtreeTile::find_tile_to_south(&l0x0y0, &level_zero_tiles).unwrap().borrow().y,
        l0x0y1.borrow().y
    );

    assert_eq!(
        QuadtreeTile::find_tile_to_west(&l0x1y1, &level_zero_tiles).unwrap().borrow().x,
        l0x0y1.borrow().x
    );
    assert_eq!(
        QuadtreeTile::find_tile_to_east(&l0x1y1, &level_zero_tiles).unwrap().borrow().x,
        l0x2y1.borrow().x
    );
    assert_eq!(
        QuadtreeTile::find_tile_to_north(&l0x1y1, &level_zero_tiles).unwrap().borrow().y,
        l0x1y0.borrow().y
    );
    assert_eq!(
        QuadtreeTile::find_tile_to_south(&l0x1y1, &level_zero_tiles).unwrap().borrow().y,
        l0x1y2.borrow().y
    );

    assert!(QuadtreeTile::find_tile_to_south(&l0x1y2, &level_zero_tiles).is_none());
}

#[test]
fn test_can_get_tiles_around_a_tile_when_they_share_a_common_parent() {
    let geo = GeographicTilingScheme::new(Ellipsoid::WGS84, GeoRectangle::MAXIMUM, 2, 1).unwrap();
    let scheme = AnyTilingScheme::Geographic(geo);
    let level_zero_tiles = QuadtreeTile::create_level_zero_tiles(scheme).unwrap();

    let parent = &level_zero_tiles[1];
    let sw = QuadtreeTile::southwest_child(parent);
    let se = QuadtreeTile::southeast_child(parent);
    let nw = QuadtreeTile::northwest_child(parent);
    let ne = QuadtreeTile::northeast_child(parent);

    assert!(std::rc::Rc::ptr_eq(
        &QuadtreeTile::find_tile_to_east(&sw, &level_zero_tiles).unwrap(),
        &se
    ));
    assert!(std::rc::Rc::ptr_eq(
        &QuadtreeTile::find_tile_to_north(&sw, &level_zero_tiles).unwrap(),
        &nw
    ));
    assert!(std::rc::Rc::ptr_eq(
        &QuadtreeTile::find_tile_to_west(&se, &level_zero_tiles).unwrap(),
        &sw
    ));
    assert!(std::rc::Rc::ptr_eq(
        &QuadtreeTile::find_tile_to_north(&se, &level_zero_tiles).unwrap(),
        &ne
    ));
    assert!(std::rc::Rc::ptr_eq(
        &QuadtreeTile::find_tile_to_east(&nw, &level_zero_tiles).unwrap(),
        &ne
    ));
    assert!(std::rc::Rc::ptr_eq(
        &QuadtreeTile::find_tile_to_south(&nw, &level_zero_tiles).unwrap(),
        &sw
    ));
    assert!(std::rc::Rc::ptr_eq(
        &QuadtreeTile::find_tile_to_west(&ne, &level_zero_tiles).unwrap(),
        &nw
    ));
    assert!(std::rc::Rc::ptr_eq(
        &QuadtreeTile::find_tile_to_south(&ne, &level_zero_tiles).unwrap(),
        &se
    ));
}

#[test]
fn test_can_get_tiles_around_a_tile_when_they_do_not_share_a_common_parent() {
    let geo = GeographicTilingScheme::new(Ellipsoid::WGS84, GeoRectangle::MAXIMUM, 2, 2).unwrap();
    let scheme = AnyTilingScheme::Geographic(geo);
    let level_zero_tiles = QuadtreeTile::create_level_zero_tiles(scheme).unwrap();

    let nw = &level_zero_tiles[0];
    let nwse = QuadtreeTile::southeast_child(nw);
    let nwne = QuadtreeTile::northeast_child(nw);
    let nwsw = QuadtreeTile::southwest_child(nw);

    let ne = &level_zero_tiles[1];
    let nesw = QuadtreeTile::southwest_child(ne);
    let nenw = QuadtreeTile::northwest_child(ne);
    let nese = QuadtreeTile::southeast_child(ne);

    let sw = &level_zero_tiles[2];
    let swne = QuadtreeTile::northeast_child(sw);
    let swnw = QuadtreeTile::northwest_child(sw);
    let swse = QuadtreeTile::southeast_child(sw);

    let se = &level_zero_tiles[3];
    let senw = QuadtreeTile::northwest_child(se);
    let sene = QuadtreeTile::northeast_child(se);
    let sesw = QuadtreeTile::southwest_child(se);

    assert!(std::rc::Rc::ptr_eq(
        &QuadtreeTile::find_tile_to_east(&nwse, &level_zero_tiles).unwrap(),
        &nesw
    ));
    assert!(std::rc::Rc::ptr_eq(
        &QuadtreeTile::find_tile_to_south(&nwse, &level_zero_tiles).unwrap(),
        &swne
    ));
    assert!(std::rc::Rc::ptr_eq(
        &QuadtreeTile::find_tile_to_east(&nwne, &level_zero_tiles).unwrap(),
        &nenw
    ));
    assert!(std::rc::Rc::ptr_eq(
        &QuadtreeTile::find_tile_to_south(&nwsw, &level_zero_tiles).unwrap(),
        &swnw
    ));

    assert!(std::rc::Rc::ptr_eq(
        &QuadtreeTile::find_tile_to_west(&nesw, &level_zero_tiles).unwrap(),
        &nwse
    ));
    assert!(std::rc::Rc::ptr_eq(
        &QuadtreeTile::find_tile_to_south(&nesw, &level_zero_tiles).unwrap(),
        &senw
    ));
    assert!(std::rc::Rc::ptr_eq(
        &QuadtreeTile::find_tile_to_west(&nenw, &level_zero_tiles).unwrap(),
        &nwne
    ));
    assert!(std::rc::Rc::ptr_eq(
        &QuadtreeTile::find_tile_to_south(&nese, &level_zero_tiles).unwrap(),
        &sene
    ));

    assert!(std::rc::Rc::ptr_eq(
        &QuadtreeTile::find_tile_to_east(&swne, &level_zero_tiles).unwrap(),
        &senw
    ));
    assert!(std::rc::Rc::ptr_eq(
        &QuadtreeTile::find_tile_to_north(&swne, &level_zero_tiles).unwrap(),
        &nwse
    ));
    assert!(std::rc::Rc::ptr_eq(
        &QuadtreeTile::find_tile_to_north(&swnw, &level_zero_tiles).unwrap(),
        &nwsw
    ));
    assert!(std::rc::Rc::ptr_eq(
        &QuadtreeTile::find_tile_to_east(&swse, &level_zero_tiles).unwrap(),
        &sesw
    ));

    assert!(std::rc::Rc::ptr_eq(
        &QuadtreeTile::find_tile_to_west(&senw, &level_zero_tiles).unwrap(),
        &swne
    ));
    assert!(std::rc::Rc::ptr_eq(
        &QuadtreeTile::find_tile_to_north(&senw, &level_zero_tiles).unwrap(),
        &nesw
    ));
    assert!(std::rc::Rc::ptr_eq(
        &QuadtreeTile::find_tile_to_north(&sene, &level_zero_tiles).unwrap(),
        &nese
    ));
    assert!(std::rc::Rc::ptr_eq(
        &QuadtreeTile::find_tile_to_west(&sesw, &level_zero_tiles).unwrap(),
        &swse
    ));
}

#[test]
fn test_can_get_adjacent_tiles_wrapping_around_anti_meridian() {
    let geo = GeographicTilingScheme::new(Ellipsoid::WGS84, GeoRectangle::MAXIMUM, 2, 1).unwrap();
    let scheme = AnyTilingScheme::Geographic(geo);
    let level_zero_tiles = QuadtreeTile::create_level_zero_tiles(scheme).unwrap();

    let west = &level_zero_tiles[0];
    let wsw = QuadtreeTile::southwest_child(west);
    let wnw = QuadtreeTile::northwest_child(west);

    let east = &level_zero_tiles[1];
    let ene = QuadtreeTile::northeast_child(east);
    let ese = QuadtreeTile::southeast_child(east);

    assert!(std::rc::Rc::ptr_eq(
        &QuadtreeTile::find_tile_to_west(&wsw, &level_zero_tiles).unwrap(),
        &ese
    ));
    assert!(std::rc::Rc::ptr_eq(
        &QuadtreeTile::find_tile_to_west(&wnw, &level_zero_tiles).unwrap(),
        &ene
    ));
    assert!(std::rc::Rc::ptr_eq(
        &QuadtreeTile::find_tile_to_east(&ene, &level_zero_tiles).unwrap(),
        &wnw
    ));
    assert!(std::rc::Rc::ptr_eq(
        &QuadtreeTile::find_tile_to_east(&ese, &level_zero_tiles).unwrap(),
        &wsw
    ));
}

#[test]
fn test_create_level_zero_tiles_counts_and_ordering() {
    let geo1x1 = GeographicTilingScheme::new(Ellipsoid::WGS84, GeoRectangle::MAXIMUM, 1, 1).unwrap();
    let geo2x2 = GeographicTilingScheme::new(Ellipsoid::WGS84, GeoRectangle::MAXIMUM, 2, 2).unwrap();
    let geo2x1 = GeographicTilingScheme::new(Ellipsoid::WGS84, GeoRectangle::MAXIMUM, 2, 1).unwrap();
    let geo1x2 = GeographicTilingScheme::new(Ellipsoid::WGS84, GeoRectangle::MAXIMUM, 1, 2).unwrap();

    assert_eq!(QuadtreeTile::create_level_zero_tiles(AnyTilingScheme::Geographic(geo1x1)).unwrap().len(), 1);
    let tiles2x2 = QuadtreeTile::create_level_zero_tiles(AnyTilingScheme::Geographic(geo2x2)).unwrap();
    assert_eq!(tiles2x2.len(), 4);
    assert_eq!(QuadtreeTile::create_level_zero_tiles(AnyTilingScheme::Geographic(geo2x1)).unwrap().len(), 2);
    assert_eq!(QuadtreeTile::create_level_zero_tiles(AnyTilingScheme::Geographic(geo1x2)).unwrap().len(), 2);

    let nw = &tiles2x2[0];
    let ne = &tiles2x2[1];
    let sw = &tiles2x2[2];
    let se = &tiles2x2[3];

    assert!(ne.borrow().rectangle.west > nw.borrow().rectangle.west);
    assert!(se.borrow().rectangle.west > sw.borrow().rectangle.west);
    assert!(ne.borrow().rectangle.south > se.borrow().rectangle.south);
    assert!(nw.borrow().rectangle.south > sw.borrow().rectangle.south);
}

#[test]
fn test_position_cache() {
    let geo = GeographicTilingScheme::new(Ellipsoid::WGS84, GeoRectangle::MAXIMUM, 2, 2).unwrap();
    let scheme = AnyTilingScheme::Geographic(geo);

    let tile = QuadtreeTile::new(QuadtreeTileOptions {
        x: 0,
        y: 0,
        level: 3,
        tiling_scheme: scheme,
        parent: None,
    })
    .unwrap();

    let lon = 45.0 * PI / 180.0;
    let lat = 30.0 * PI / 180.0;
    let carto1 = Cartographic::new(lon, lat, 0.0);
    let carto2 = Cartographic::new(lon, lat, 0.0);

    let dummy_pos = Cartesian3::new(1000.0, 2000.0, 3000.0);

    tile.borrow().set_position_cache_entry(carto1, 2.0, dummy_pos);
    assert_eq!(tile.borrow().position_cache.borrow().size(), 1);

    let cached = tile.borrow().get_position_cache_entry(carto2, 2.0);
    assert_eq!(cached, Some(dummy_pos));

    tile.borrow().clear_position_cache();
    assert_eq!(tile.borrow().position_cache.borrow().size(), 0);
}

#[test]
fn test_update_custom_data() {
    let geo = GeographicTilingScheme::new(Ellipsoid::WGS84, GeoRectangle::MAXIMUM, 1, 1).unwrap();
    let scheme = AnyTilingScheme::Geographic(geo);

    let tile = QuadtreeTile::new(QuadtreeTileOptions {
        x: 0,
        y: 0,
        level: 0,
        tiling_scheme: scheme,
        parent: None,
    })
    .unwrap();

    let nw_child = QuadtreeTile::northwest_child(&tile);
    let nw_rect = nw_child.borrow().rectangle;
    let center = Cartographic::new(
        (nw_rect.west + nw_rect.east) * 0.5,
        (nw_rect.south + nw_rect.north) * 0.5,
        0.0,
    );

    let data = CustomDataEntry {
        id: 42,
        position_cartographic: center,
    };

    tile.borrow().added_custom_data.borrow_mut().push(data.clone());
    QuadtreeTile::update_custom_data(&tile);

    assert!(tile.borrow().custom_data.borrow().contains(&data));
    assert_eq!(tile.borrow().added_custom_data.borrow().len(), 0);
    assert_eq!(nw_child.borrow().added_custom_data.borrow().len(), 1);

    QuadtreeTile::update_custom_data(&nw_child);
    assert!(nw_child.borrow().custom_data.borrow().contains(&data));

    // Remove
    tile.borrow().removed_custom_data.borrow_mut().push(data.clone());
    QuadtreeTile::update_custom_data(&tile);

    assert!(!tile.borrow().custom_data.borrow().contains(&data));
    assert_eq!(nw_child.borrow().removed_custom_data.borrow().len(), 1);

    QuadtreeTile::update_custom_data(&nw_child);
    assert!(!nw_child.borrow().custom_data.borrow().contains(&data));
}
