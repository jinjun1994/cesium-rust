use cesium_math::{Color, ComponentDatatype, HeightReference, Matrix4};
use cesium_renderer::{
    drill_pick_buffer_polygon_collection, pick_buffer_polygon_collection,
    render_buffer_polygon_collection, BlendOption, BufferPolygon, BufferPolygonCollection,
    BufferPolygonMaterial, BufferPolygonMaterialOptions, BufferPolygonOptions,
};

const BLACK: [u8; 4] = [0, 0, 0, 255];

const POSITIONS: &[f32] = &[
    -1000.0, -1000.0, -1000.0,
    -1000.0, -1000.0, 2000.0,
    -1000.0, 2000.0, -1000.0,
];

const POSITIONS_OUT_OF_VIEW: &[f32] = &[
    -1000.0, 1000.0, -1000.0,
    -1000.0, 1000.0, 2000.0,
    -1000.0, 3000.0, -1000.0,
];

const TRIANGLES: &[u32] = &[0, 1, 2];

#[test]
fn renders_polygons() {
    let mut collection = BufferPolygonCollection {
        position_datatype: Some(ComponentDatatype::Int),
        blend_option: BlendOption::Opaque,
        ..Default::default()
    };

    let mut polygon = BufferPolygon::default();
    collection.add(
        BufferPolygonOptions {
            positions: POSITIONS.to_vec(),
            triangles: TRIANGLES.to_vec(),
            ..Default::default()
        },
        &mut polygon,
    );

    let empty_col = BufferPolygonCollection::default();
    assert_eq!(render_buffer_polygon_collection(&empty_col, BLACK), BLACK);

    assert_eq!(
        render_buffer_polygon_collection(&collection, BLACK),
        [255, 255, 255, 255]
    );
}

#[test]
fn renders_polygons_after_blend_option_changes() {
    let mut collection = BufferPolygonCollection {
        position_datatype: Some(ComponentDatatype::Int),
        blend_option: BlendOption::Opaque,
        ..Default::default()
    };

    let mut polygon = BufferPolygon::default();
    let material = BufferPolygonMaterial::new(Some(BufferPolygonMaterialOptions {
        color: Some(Color::RED.with_alpha(0.5)),
        ..Default::default()
    }));

    collection.add(
        BufferPolygonOptions {
            positions: POSITIONS.to_vec(),
            triangles: TRIANGLES.to_vec(),
            material: Some(material),
            ..Default::default()
        },
        &mut polygon,
    );

    // Blending is disabled in the opaque pass, so alpha has no effect.
    assert_eq!(
        render_buffer_polygon_collection(&collection, BLACK),
        [255, 0, 0, 255]
    );

    collection.blend_option = BlendOption::Translucent;
    assert_eq!(
        render_buffer_polygon_collection(&collection, BLACK),
        [128, 0, 0, 255]
    );

    collection.blend_option = BlendOption::Opaque;
    assert_eq!(
        render_buffer_polygon_collection(&collection, BLACK),
        [255, 0, 0, 255]
    );
}

#[test]
fn does_not_render_draped_polygons() {
    let mut collection = BufferPolygonCollection {
        position_datatype: Some(ComponentDatatype::Int),
        blend_option: BlendOption::Opaque,
        height_reference: Some(HeightReference::ClampToTerrain),
        ..Default::default()
    };

    let mut polygon = BufferPolygon::default();
    collection.add(
        BufferPolygonOptions {
            positions: POSITIONS.to_vec(),
            triangles: TRIANGLES.to_vec(),
            ..Default::default()
        },
        &mut polygon,
    );

    assert_eq!(render_buffer_polygon_collection(&collection, BLACK), BLACK);
}

#[test]
fn renders_polygons_with_color() {
    let mut collection = BufferPolygonCollection {
        position_datatype: Some(ComponentDatatype::Int),
        blend_option: BlendOption::Translucent,
        ..Default::default()
    };

    let mut polygon = BufferPolygon::default();
    let mut material = BufferPolygonMaterial::new(Some(BufferPolygonMaterialOptions {
        color: Some(Color::RED),
        ..Default::default()
    }));

    collection.add(
        BufferPolygonOptions {
            positions: POSITIONS.to_vec(),
            triangles: TRIANGLES.to_vec(),
            material: Some(material.clone()),
            ..Default::default()
        },
        &mut polygon,
    );

    assert_eq!(
        render_buffer_polygon_collection(&collection, BLACK),
        [255, 0, 0, 255]
    );

    // In Cesium, Color.GREEN is #008000 (0, 128, 0, 255)
    material.color = Color::from_bytes(0, 128, 0, 255);
    polygon.set_material(material.clone());
    collection.set(polygon.index, &polygon);
    assert_eq!(
        render_buffer_polygon_collection(&collection, BLACK),
        [0, 128, 0, 255]
    );

    material.color.alpha = 0.5;
    polygon.set_material(material);
    collection.set(polygon.index, &polygon);
    assert_eq!(
        render_buffer_polygon_collection(&collection, BLACK),
        [0, 64, 0, 255]
    );
}

#[test]
fn renders_polygons_with_updated_positions() {
    let mut collection = BufferPolygonCollection {
        position_datatype: Some(ComponentDatatype::Int),
        blend_option: BlendOption::Opaque,
        ..Default::default()
    };

    let mut polygon = BufferPolygon::default();

    let red_mat = BufferPolygonMaterial::new(Some(BufferPolygonMaterialOptions {
        color: Some(Color::from_bytes(255, 0, 0, 255)),
        ..Default::default()
    }));
    collection.add(
        BufferPolygonOptions {
            positions: POSITIONS.to_vec(),
            triangles: TRIANGLES.to_vec(),
            material: Some(red_mat),
            ..Default::default()
        },
        &mut polygon,
    );

    let blue_mat = BufferPolygonMaterial::new(Some(BufferPolygonMaterialOptions {
        color: Some(Color::from_bytes(0, 0, 255, 255)),
        ..Default::default()
    }));
    collection.add(
        BufferPolygonOptions {
            positions: POSITIONS.to_vec(),
            triangles: TRIANGLES.to_vec(),
            material: Some(blue_mat),
            ..Default::default()
        },
        &mut polygon,
    );

    assert_eq!(
        render_buffer_polygon_collection(&collection, BLACK),
        [255, 0, 0, 255]
    );

    collection.get(0, &mut polygon);
    polygon.set_positions(POSITIONS_OUT_OF_VIEW);
    collection.set(0, &polygon);

    assert_eq!(
        render_buffer_polygon_collection(&collection, BLACK),
        [0, 0, 255, 255]
    );
}

#[test]
fn renders_polygons_with_sort_order() {
    let mut collection = BufferPolygonCollection {
        position_datatype: Some(ComponentDatatype::Int),
        blend_option: BlendOption::Opaque,
        ..Default::default()
    };

    let mut polygon = BufferPolygon::default();

    let red_mat = BufferPolygonMaterial::new(Some(BufferPolygonMaterialOptions {
        color: Some(Color::RED),
        ..Default::default()
    }));
    collection.add(
        BufferPolygonOptions {
            positions: POSITIONS.to_vec(),
            triangles: TRIANGLES.to_vec(),
            material: Some(red_mat),
            feature_id: Some(0),
            ..Default::default()
        },
        &mut polygon,
    );

    let blue_mat = BufferPolygonMaterial::new(Some(BufferPolygonMaterialOptions {
        color: Some(Color::BLUE),
        ..Default::default()
    }));
    collection.add(
        BufferPolygonOptions {
            positions: POSITIONS.to_vec(),
            triangles: TRIANGLES.to_vec(),
            material: Some(blue_mat),
            feature_id: Some(1),
            ..Default::default()
        },
        &mut polygon,
    );

    assert_eq!(
        render_buffer_polygon_collection(&collection, BLACK),
        [255, 0, 0, 255]
    );

    collection.sort(|a, b| b.feature_id.cmp(&a.feature_id));
    assert_eq!(
        render_buffer_polygon_collection(&collection, BLACK),
        [0, 0, 255, 255]
    );
}

#[test]
fn renders_polygons_with_updated_model_matrix() {
    let mut collection = BufferPolygonCollection {
        position_datatype: Some(ComponentDatatype::Int),
        blend_option: BlendOption::Opaque,
        ..Default::default()
    };

    let mut polygon = BufferPolygon::default();
    collection.add(
        BufferPolygonOptions {
            positions: POSITIONS.to_vec(),
            triangles: TRIANGLES.to_vec(),
            ..Default::default()
        },
        &mut polygon,
    );

    assert_eq!(
        render_buffer_polygon_collection(&collection, BLACK),
        [255, 255, 255, 255]
    );

    collection.model_matrix = Matrix4::from_uniform_scale(0.0);
    assert_eq!(render_buffer_polygon_collection(&collection, BLACK), BLACK);
}

#[test]
fn does_not_render_if_empty() {
    let collection = BufferPolygonCollection {
        position_datatype: Some(ComponentDatatype::Int),
        blend_option: BlendOption::Opaque,
        ..Default::default()
    };

    assert_eq!(render_buffer_polygon_collection(&collection, BLACK), BLACK);
}

#[test]
fn does_not_render_if_collection_show_false() {
    let mut collection = BufferPolygonCollection {
        position_datatype: Some(ComponentDatatype::Int),
        blend_option: BlendOption::Opaque,
        ..Default::default()
    };

    let mut polygon = BufferPolygon::default();
    collection.add(
        BufferPolygonOptions {
            positions: POSITIONS.to_vec(),
            triangles: TRIANGLES.to_vec(),
            ..Default::default()
        },
        &mut polygon,
    );

    assert_eq!(
        render_buffer_polygon_collection(&collection, BLACK),
        [255, 255, 255, 255]
    );

    collection.show = false;
    assert_eq!(render_buffer_polygon_collection(&collection, BLACK), BLACK);
}

#[test]
fn does_not_render_if_polygon_show_false() {
    let mut collection = BufferPolygonCollection {
        position_datatype: Some(ComponentDatatype::Int),
        blend_option: BlendOption::Opaque,
        ..Default::default()
    };

    let mut polygon = BufferPolygon::default();
    collection.add(
        BufferPolygonOptions {
            positions: POSITIONS.to_vec(),
            triangles: TRIANGLES.to_vec(),
            ..Default::default()
        },
        &mut polygon,
    );

    assert_eq!(
        render_buffer_polygon_collection(&collection, BLACK),
        [255, 255, 255, 255]
    );

    polygon.show = false;
    collection.set(polygon.index, &polygon);
    assert_eq!(render_buffer_polygon_collection(&collection, BLACK), BLACK);
}

#[test]
fn picks_polygons() {
    let mut collection = BufferPolygonCollection {
        position_datatype: Some(ComponentDatatype::Int),
        allow_picking: true,
        ..Default::default()
    };

    let mut polygon = BufferPolygon::default();
    collection.add(
        BufferPolygonOptions {
            positions: POSITIONS.to_vec(),
            triangles: TRIANGLES.to_vec(),
            ..Default::default()
        },
        &mut polygon,
    );
    collection.add(
        BufferPolygonOptions {
            positions: POSITIONS.to_vec(),
            triangles: TRIANGLES.to_vec(),
            ..Default::default()
        },
        &mut polygon,
    );

    assert_eq!(pick_buffer_polygon_collection(&collection), Some(0));
}

#[test]
fn drill_picks_polygons() {
    let mut collection = BufferPolygonCollection {
        position_datatype: Some(ComponentDatatype::Int),
        allow_picking: true,
        ..Default::default()
    };

    let mut polygon = BufferPolygon::default();
    let positions_bad = vec![0.0, 0.0, 0.0, 0.0, 0.0, 0.0];

    collection.add(
        BufferPolygonOptions {
            positions: POSITIONS.to_vec(),
            triangles: TRIANGLES.to_vec(),
            ..Default::default()
        },
        &mut polygon,
    );
    collection.add(
        BufferPolygonOptions {
            positions: positions_bad,
            triangles: vec![],
            ..Default::default()
        },
        &mut polygon,
    );
    collection.add(
        BufferPolygonOptions {
            positions: POSITIONS.to_vec(),
            triangles: TRIANGLES.to_vec(),
            ..Default::default()
        },
        &mut polygon,
    );

    assert_eq!(drill_pick_buffer_polygon_collection(&collection), vec![0, 2]);
}

#[test]
fn does_not_pick_if_picking_disabled() {
    let mut collection = BufferPolygonCollection {
        position_datatype: Some(ComponentDatatype::Int),
        allow_picking: false,
        ..Default::default()
    };

    let mut polygon = BufferPolygon::default();
    collection.add(
        BufferPolygonOptions {
            positions: POSITIONS.to_vec(),
            triangles: TRIANGLES.to_vec(),
            ..Default::default()
        },
        &mut polygon,
    );

    assert_eq!(pick_buffer_polygon_collection(&collection), None);
}

#[test]
fn does_not_pick_if_empty() {
    let collection = BufferPolygonCollection {
        position_datatype: Some(ComponentDatatype::Int),
        allow_picking: true,
        ..Default::default()
    };

    assert_eq!(pick_buffer_polygon_collection(&collection), None);
}

#[test]
fn does_not_pick_if_collection_show_false() {
    let mut collection = BufferPolygonCollection {
        position_datatype: Some(ComponentDatatype::Int),
        allow_picking: true,
        ..Default::default()
    };

    let mut polygon = BufferPolygon::default();
    collection.add(
        BufferPolygonOptions {
            positions: POSITIONS.to_vec(),
            triangles: TRIANGLES.to_vec(),
            ..Default::default()
        },
        &mut polygon,
    );

    assert_eq!(pick_buffer_polygon_collection(&collection), Some(0));

    collection.show = false;
    assert_eq!(pick_buffer_polygon_collection(&collection), None);
}

#[test]
fn does_not_pick_if_polygon_show_false() {
    let mut collection = BufferPolygonCollection {
        position_datatype: Some(ComponentDatatype::Int),
        allow_picking: true,
        ..Default::default()
    };

    let mut polygon = BufferPolygon::default();
    collection.add(
        BufferPolygonOptions {
            positions: POSITIONS.to_vec(),
            triangles: TRIANGLES.to_vec(),
            ..Default::default()
        },
        &mut polygon,
    );
    collection.add(
        BufferPolygonOptions {
            positions: POSITIONS.to_vec(),
            triangles: TRIANGLES.to_vec(),
            ..Default::default()
        },
        &mut polygon,
    );

    assert_eq!(pick_buffer_polygon_collection(&collection), Some(0));

    collection.get(0, &mut polygon);
    polygon.show = false;
    collection.set(0, &polygon);

    assert_eq!(pick_buffer_polygon_collection(&collection), Some(1));

    collection.get(1, &mut polygon);
    polygon.show = false;
    collection.set(1, &polygon);

    assert_eq!(pick_buffer_polygon_collection(&collection), None);
}
