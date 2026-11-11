use cesium_math::{Color, ComponentDatatype, HeightReference, Matrix4};
use cesium_renderer::{
    drill_pick_buffer_polyline_collection, pick_buffer_polyline_collection,
    render_buffer_polyline_collection, BlendOption, BufferPolyline, BufferPolylineCollection,
    BufferPolylineMaterial, BufferPolylineMaterialOptions, BufferPolylineOptions,
};

const BLACK: [u8; 4] = [0, 0, 0, 255];

const POSITIONS: &[f32] = &[0.0, -1000000.0, 0.0, 0.0, 1000000.0, 0.0];
const POSITIONS_OUT_OF_VIEW: &[f32] = &[0.0, 5000.0, 0.0, 0.0, 1000000.0, 0.0];

#[test]
fn renders_polylines() {
    let mut collection = BufferPolylineCollection {
        position_datatype: Some(ComponentDatatype::Int),
        blend_option: BlendOption::Opaque,
        ..Default::default()
    };

    let mut line = BufferPolyline::default();
    collection.add(
        BufferPolylineOptions {
            positions: POSITIONS.to_vec(),
            ..Default::default()
        },
        &mut line,
    );

    let empty_col = BufferPolylineCollection::default();
    assert_eq!(render_buffer_polyline_collection(&empty_col, BLACK), BLACK);

    assert_eq!(
        render_buffer_polyline_collection(&collection, BLACK),
        [255, 255, 255, 255]
    );
}

#[test]
fn renders_polylines_after_blend_option_changes() {
    let mut collection = BufferPolylineCollection {
        position_datatype: Some(ComponentDatatype::Int),
        blend_option: BlendOption::Opaque,
        ..Default::default()
    };

    let mut line = BufferPolyline::default();
    let material = BufferPolylineMaterial::new(Some(BufferPolylineMaterialOptions {
        color: Some(Color::RED.with_alpha(0.5)),
        ..Default::default()
    }));

    collection.add(
        BufferPolylineOptions {
            positions: POSITIONS.to_vec(),
            material: Some(material),
            ..Default::default()
        },
        &mut line,
    );

    // Blending is disabled in the opaque pass, so alpha has no effect.
    assert_eq!(
        render_buffer_polyline_collection(&collection, BLACK),
        [255, 0, 0, 255]
    );

    collection.blend_option = BlendOption::Translucent;
    assert_eq!(
        render_buffer_polyline_collection(&collection, BLACK),
        [128, 0, 0, 255]
    );

    collection.blend_option = BlendOption::Opaque;
    assert_eq!(
        render_buffer_polyline_collection(&collection, BLACK),
        [255, 0, 0, 255]
    );
}

#[test]
fn does_not_render_draped_polylines() {
    let mut collection = BufferPolylineCollection {
        position_datatype: Some(ComponentDatatype::Int),
        blend_option: BlendOption::Opaque,
        height_reference: Some(HeightReference::ClampToTerrain),
        ..Default::default()
    };

    let mut line = BufferPolyline::default();
    collection.add(
        BufferPolylineOptions {
            positions: POSITIONS.to_vec(),
            ..Default::default()
        },
        &mut line,
    );

    assert_eq!(render_buffer_polyline_collection(&collection, BLACK), BLACK);
}

#[test]
fn renders_polylines_with_color() {
    let mut collection = BufferPolylineCollection {
        position_datatype: Some(ComponentDatatype::Int),
        blend_option: BlendOption::Translucent,
        ..Default::default()
    };

    let mut line = BufferPolyline::default();
    let mut material = BufferPolylineMaterial::new(Some(BufferPolylineMaterialOptions {
        color: Some(Color::RED),
        ..Default::default()
    }));

    collection.add(
        BufferPolylineOptions {
            positions: POSITIONS.to_vec(),
            material: Some(material.clone()),
            ..Default::default()
        },
        &mut line,
    );

    assert_eq!(
        render_buffer_polyline_collection(&collection, BLACK),
        [255, 0, 0, 255]
    );

    // In Cesium, Color.GREEN is #008000 (0, 128, 0, 255)
    material.color = Color::from_bytes(0, 128, 0, 255);
    line.set_material(material.clone());
    collection.set(line.index, &line);
    assert_eq!(
        render_buffer_polyline_collection(&collection, BLACK),
        [0, 128, 0, 255]
    );

    material.color.alpha = 0.5;
    line.set_material(material);
    collection.set(line.index, &line);
    assert_eq!(
        render_buffer_polyline_collection(&collection, BLACK),
        [0, 64, 0, 255]
    );
}

#[test]
fn renders_polylines_with_updated_positions() {
    let mut collection = BufferPolylineCollection {
        position_datatype: Some(ComponentDatatype::Int),
        blend_option: BlendOption::Opaque,
        ..Default::default()
    };

    let mut line = BufferPolyline::default();

    let red_mat = BufferPolylineMaterial::new(Some(BufferPolylineMaterialOptions {
        color: Some(Color::from_bytes(255, 0, 0, 255)),
        ..Default::default()
    }));
    collection.add(
        BufferPolylineOptions {
            positions: POSITIONS.to_vec(),
            material: Some(red_mat),
            ..Default::default()
        },
        &mut line,
    );

    let blue_mat = BufferPolylineMaterial::new(Some(BufferPolylineMaterialOptions {
        color: Some(Color::from_bytes(0, 0, 255, 255)),
        ..Default::default()
    }));
    collection.add(
        BufferPolylineOptions {
            positions: POSITIONS.to_vec(),
            material: Some(blue_mat),
            ..Default::default()
        },
        &mut line,
    );

    assert_eq!(
        render_buffer_polyline_collection(&collection, BLACK),
        [255, 0, 0, 255]
    );

    collection.get(0, &mut line);
    line.set_positions(POSITIONS_OUT_OF_VIEW);
    collection.set(0, &line);

    assert_eq!(
        render_buffer_polyline_collection(&collection, BLACK),
        [0, 0, 255, 255]
    );
}

#[test]
fn renders_polylines_with_sort_order() {
    let mut collection = BufferPolylineCollection {
        position_datatype: Some(ComponentDatatype::Int),
        blend_option: BlendOption::Opaque,
        ..Default::default()
    };

    let mut line = BufferPolyline::default();

    let red_mat = BufferPolylineMaterial::new(Some(BufferPolylineMaterialOptions {
        color: Some(Color::RED),
        ..Default::default()
    }));
    collection.add(
        BufferPolylineOptions {
            positions: POSITIONS.to_vec(),
            material: Some(red_mat),
            feature_id: Some(0),
            ..Default::default()
        },
        &mut line,
    );

    let blue_mat = BufferPolylineMaterial::new(Some(BufferPolylineMaterialOptions {
        color: Some(Color::BLUE),
        ..Default::default()
    }));
    collection.add(
        BufferPolylineOptions {
            positions: POSITIONS.to_vec(),
            material: Some(blue_mat),
            feature_id: Some(1),
            ..Default::default()
        },
        &mut line,
    );

    assert_eq!(
        render_buffer_polyline_collection(&collection, BLACK),
        [255, 0, 0, 255]
    );

    collection.sort(|a, b| b.feature_id.cmp(&a.feature_id));
    assert_eq!(
        render_buffer_polyline_collection(&collection, BLACK),
        [0, 0, 255, 255]
    );
}

#[test]
fn renders_polylines_with_updated_model_matrix() {
    let mut collection = BufferPolylineCollection {
        position_datatype: Some(ComponentDatatype::Int),
        blend_option: BlendOption::Opaque,
        ..Default::default()
    };

    let mut line = BufferPolyline::default();
    collection.add(
        BufferPolylineOptions {
            positions: POSITIONS.to_vec(),
            ..Default::default()
        },
        &mut line,
    );

    assert_eq!(
        render_buffer_polyline_collection(&collection, BLACK),
        [255, 255, 255, 255]
    );

    collection.model_matrix = Matrix4::from_uniform_scale(0.0);
    assert_eq!(render_buffer_polyline_collection(&collection, BLACK), BLACK);
}

#[test]
fn does_not_render_if_empty() {
    let collection = BufferPolylineCollection {
        position_datatype: Some(ComponentDatatype::Int),
        blend_option: BlendOption::Opaque,
        ..Default::default()
    };

    assert_eq!(render_buffer_polyline_collection(&collection, BLACK), BLACK);
}

#[test]
fn does_not_render_if_collection_show_false() {
    let mut collection = BufferPolylineCollection {
        position_datatype: Some(ComponentDatatype::Int),
        blend_option: BlendOption::Opaque,
        ..Default::default()
    };

    let mut line = BufferPolyline::default();
    collection.add(
        BufferPolylineOptions {
            positions: POSITIONS.to_vec(),
            ..Default::default()
        },
        &mut line,
    );

    assert_eq!(
        render_buffer_polyline_collection(&collection, BLACK),
        [255, 255, 255, 255]
    );

    collection.show = false;
    assert_eq!(render_buffer_polyline_collection(&collection, BLACK), BLACK);
}

#[test]
fn does_not_render_if_polyline_show_false() {
    let mut collection = BufferPolylineCollection {
        position_datatype: Some(ComponentDatatype::Int),
        blend_option: BlendOption::Opaque,
        ..Default::default()
    };

    let mut line = BufferPolyline::default();
    collection.add(
        BufferPolylineOptions {
            positions: POSITIONS.to_vec(),
            ..Default::default()
        },
        &mut line,
    );

    assert_eq!(
        render_buffer_polyline_collection(&collection, BLACK),
        [255, 255, 255, 255]
    );

    line.show = false;
    collection.set(line.index, &line);
    assert_eq!(render_buffer_polyline_collection(&collection, BLACK), BLACK);
}

#[test]
fn picks_polylines() {
    let mut collection = BufferPolylineCollection {
        position_datatype: Some(ComponentDatatype::Int),
        allow_picking: true,
        ..Default::default()
    };

    let mut line = BufferPolyline::default();
    collection.add(
        BufferPolylineOptions {
            positions: POSITIONS.to_vec(),
            ..Default::default()
        },
        &mut line,
    );
    collection.add(
        BufferPolylineOptions {
            positions: POSITIONS.to_vec(),
            ..Default::default()
        },
        &mut line,
    );

    assert_eq!(pick_buffer_polyline_collection(&collection), Some(0));
}

#[test]
fn drill_picks_polylines() {
    let mut collection = BufferPolylineCollection {
        position_datatype: Some(ComponentDatatype::Int),
        allow_picking: true,
        ..Default::default()
    };

    let mut line = BufferPolyline::default();
    let positions_bad = vec![10e8, -1000000.0, 10e8, 10e8, 1000000.0, 10e8];

    collection.add(
        BufferPolylineOptions {
            positions: POSITIONS.to_vec(),
            ..Default::default()
        },
        &mut line,
    );
    collection.add(
        BufferPolylineOptions {
            positions: positions_bad,
            ..Default::default()
        },
        &mut line,
    );
    collection.add(
        BufferPolylineOptions {
            positions: POSITIONS.to_vec(),
            ..Default::default()
        },
        &mut line,
    );

    assert_eq!(
        drill_pick_buffer_polyline_collection(&collection),
        vec![0, 2]
    );
}

#[test]
fn does_not_pick_if_picking_disabled() {
    let mut collection = BufferPolylineCollection {
        position_datatype: Some(ComponentDatatype::Int),
        allow_picking: false,
        ..Default::default()
    };

    let mut line = BufferPolyline::default();
    collection.add(
        BufferPolylineOptions {
            positions: POSITIONS.to_vec(),
            ..Default::default()
        },
        &mut line,
    );

    assert_eq!(pick_buffer_polyline_collection(&collection), None);
}

#[test]
fn does_not_pick_if_empty() {
    let collection = BufferPolylineCollection {
        position_datatype: Some(ComponentDatatype::Int),
        allow_picking: true,
        ..Default::default()
    };

    assert_eq!(pick_buffer_polyline_collection(&collection), None);
}

#[test]
fn does_not_pick_if_collection_show_false() {
    let mut collection = BufferPolylineCollection {
        position_datatype: Some(ComponentDatatype::Int),
        allow_picking: true,
        ..Default::default()
    };

    let mut line = BufferPolyline::default();
    collection.add(
        BufferPolylineOptions {
            positions: POSITIONS.to_vec(),
            ..Default::default()
        },
        &mut line,
    );

    assert_eq!(pick_buffer_polyline_collection(&collection), Some(0));

    collection.show = false;
    assert_eq!(pick_buffer_polyline_collection(&collection), None);
}

#[test]
fn does_not_pick_if_polyline_show_false() {
    let mut collection = BufferPolylineCollection {
        position_datatype: Some(ComponentDatatype::Int),
        allow_picking: true,
        ..Default::default()
    };

    let mut line = BufferPolyline::default();
    collection.add(
        BufferPolylineOptions {
            positions: POSITIONS.to_vec(),
            ..Default::default()
        },
        &mut line,
    );
    collection.add(
        BufferPolylineOptions {
            positions: POSITIONS.to_vec(),
            ..Default::default()
        },
        &mut line,
    );

    assert_eq!(pick_buffer_polyline_collection(&collection), Some(0));

    collection.get(0, &mut line);
    line.show = false;
    collection.set(0, &line);

    assert_eq!(pick_buffer_polyline_collection(&collection), Some(1));

    collection.get(1, &mut line);
    line.show = false;
    collection.set(1, &line);

    assert_eq!(pick_buffer_polyline_collection(&collection), None);
}
