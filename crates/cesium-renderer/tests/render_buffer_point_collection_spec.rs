use cesium_math::{Cartesian3, Color, Matrix4};
use cesium_renderer::{
    drill_pick_buffer_point_collection, pick_buffer_point_collection,
    render_buffer_point_collection, BlendOption, BufferPoint, BufferPointCollection,
    BufferPointMaterial, BufferPointMaterialOptions, BufferPointOptions,
};

const BLACK: [u8; 4] = [0, 0, 0, 255];

#[test]
fn renders_points() {
    let mut collection = BufferPointCollection {
        blend_option: BlendOption::Opaque,
        ..Default::default()
    };

    let mut point = BufferPoint::default();
    collection.add(
        BufferPointOptions {
            position: Some(Cartesian3::new(0.0, -1000.0, 0.0)),
            ..Default::default()
        },
        &mut point,
    );

    let empty_col = BufferPointCollection::default();
    assert_eq!(render_buffer_point_collection(&empty_col, BLACK), BLACK);

    assert_eq!(
        render_buffer_point_collection(&collection, BLACK),
        [255, 255, 255, 255]
    );
}

#[test]
fn renders_points_after_blend_option_changes() {
    let mut collection = BufferPointCollection {
        blend_option: BlendOption::Opaque,
        ..Default::default()
    };

    let mut point = BufferPoint::default();
    let material = BufferPointMaterial::new(Some(BufferPointMaterialOptions {
        color: Some(Color::RED.with_alpha(0.5)),
        size: Some(8),
        ..Default::default()
    }));

    collection.add(
        BufferPointOptions {
            position: Some(Cartesian3::new(0.0, -1000.0, 0.0)),
            material: Some(material),
            ..Default::default()
        },
        &mut point,
    );

    // Blending is disabled in the opaque pass, so alpha has no effect.
    assert_eq!(
        render_buffer_point_collection(&collection, BLACK),
        [255, 0, 0, 255]
    );

    collection.blend_option = BlendOption::Translucent;
    assert_eq!(
        render_buffer_point_collection(&collection, BLACK),
        [128, 0, 0, 255]
    );

    collection.blend_option = BlendOption::Opaque;
    assert_eq!(
        render_buffer_point_collection(&collection, BLACK),
        [255, 0, 0, 255]
    );
}

#[test]
fn renders_points_with_color() {
    let mut collection = BufferPointCollection {
        blend_option: BlendOption::Translucent,
        ..Default::default()
    };

    let mut point = BufferPoint::default();
    let mut material = BufferPointMaterial::new(Some(BufferPointMaterialOptions {
        color: Some(Color::RED),
        size: Some(8),
        ..Default::default()
    }));

    collection.add(
        BufferPointOptions {
            position: Some(Cartesian3::new(0.0, -1000.0, 0.0)),
            material: Some(material.clone()),
            ..Default::default()
        },
        &mut point,
    );

    assert_eq!(
        render_buffer_point_collection(&collection, BLACK),
        [255, 0, 0, 255]
    );

    // In Cesium, Color.GREEN is #008000 (0, 128, 0, 255)
    material.color = Color::from_bytes(0, 128, 0, 255);
    point.set_material(material.clone());
    collection.set(point.index, &point);
    assert_eq!(
        render_buffer_point_collection(&collection, BLACK),
        [0, 128, 0, 255]
    );

    material.color.alpha = 0.5;
    point.set_material(material);
    collection.set(point.index, &point);
    assert_eq!(
        render_buffer_point_collection(&collection, BLACK),
        [0, 64, 0, 255]
    );
}

#[test]
fn renders_points_with_updated_positions() {
    let mut collection = BufferPointCollection {
        blend_option: BlendOption::Opaque,
        ..Default::default()
    };

    let mut point = BufferPoint::default();
    let pos = Cartesian3::new(0.0, -1000.0, 0.0);

    let red_mat = BufferPointMaterial::new(Some(BufferPointMaterialOptions {
        color: Some(Color::from_bytes(255, 0, 0, 255)),
        size: Some(8),
        ..Default::default()
    }));
    collection.add(
        BufferPointOptions {
            position: Some(pos),
            material: Some(red_mat),
            ..Default::default()
        },
        &mut point,
    );

    let blue_mat = BufferPointMaterial::new(Some(BufferPointMaterialOptions {
        color: Some(Color::from_bytes(0, 0, 255, 255)),
        size: Some(8),
        ..Default::default()
    }));
    collection.add(
        BufferPointOptions {
            position: Some(pos),
            material: Some(blue_mat),
            ..Default::default()
        },
        &mut point,
    );

    assert_eq!(
        render_buffer_point_collection(&collection, BLACK),
        [255, 0, 0, 255]
    );

    collection.get(0, &mut point);
    point.set_position(Cartesian3::new(1e6, 1e6, 1e6));
    collection.set(0, &point);

    assert_eq!(
        render_buffer_point_collection(&collection, BLACK),
        [0, 0, 255, 255]
    );
}

#[test]
fn renders_points_with_sort_order() {
    let mut collection = BufferPointCollection {
        blend_option: BlendOption::Opaque,
        ..Default::default()
    };

    let mut point = BufferPoint::default();

    let red_mat = BufferPointMaterial::new(Some(BufferPointMaterialOptions {
        color: Some(Color::RED),
        size: Some(8),
        ..Default::default()
    }));
    collection.add(
        BufferPointOptions {
            position: Some(Cartesian3::new(0.0, -1000.0, 0.0)),
            material: Some(red_mat),
            feature_id: Some(0),
            ..Default::default()
        },
        &mut point,
    );

    let blue_mat = BufferPointMaterial::new(Some(BufferPointMaterialOptions {
        color: Some(Color::BLUE),
        size: Some(8),
        ..Default::default()
    }));
    collection.add(
        BufferPointOptions {
            position: Some(Cartesian3::new(0.0, -1000.0, 0.0)),
            material: Some(blue_mat),
            feature_id: Some(1),
            ..Default::default()
        },
        &mut point,
    );

    assert_eq!(
        render_buffer_point_collection(&collection, BLACK),
        [255, 0, 0, 255]
    );

    collection.sort(|a, b| b.feature_id.cmp(&a.feature_id));
    assert_eq!(
        render_buffer_point_collection(&collection, BLACK),
        [0, 0, 255, 255]
    );
}

#[test]
fn renders_points_with_updated_model_matrix() {
    let mut collection = BufferPointCollection {
        blend_option: BlendOption::Opaque,
        ..Default::default()
    };

    let mut point = BufferPoint::default();
    collection.add(
        BufferPointOptions {
            position: Some(Cartesian3::new(0.0, -1000.0, 0.0)),
            ..Default::default()
        },
        &mut point,
    );

    assert_eq!(
        render_buffer_point_collection(&collection, BLACK),
        [255, 255, 255, 255]
    );

    collection.model_matrix = Matrix4::from_uniform_scale(0.0);
    assert_eq!(render_buffer_point_collection(&collection, BLACK), BLACK);
}

#[test]
fn does_not_render_if_empty() {
    let collection = BufferPointCollection {
        blend_option: BlendOption::Opaque,
        ..Default::default()
    };

    assert_eq!(render_buffer_point_collection(&collection, BLACK), BLACK);
}

#[test]
fn does_not_render_if_collection_show_false() {
    let mut collection = BufferPointCollection {
        blend_option: BlendOption::Opaque,
        ..Default::default()
    };

    let mut point = BufferPoint::default();
    collection.add(
        BufferPointOptions {
            position: Some(Cartesian3::new(0.0, -1000.0, 0.0)),
            ..Default::default()
        },
        &mut point,
    );

    assert_eq!(
        render_buffer_point_collection(&collection, BLACK),
        [255, 255, 255, 255]
    );

    collection.show = false;
    assert_eq!(render_buffer_point_collection(&collection, BLACK), BLACK);
}

#[test]
fn does_not_render_if_point_show_false() {
    let mut collection = BufferPointCollection {
        blend_option: BlendOption::Opaque,
        ..Default::default()
    };

    let mut point = BufferPoint::default();
    collection.add(
        BufferPointOptions {
            position: Some(Cartesian3::new(0.0, -1000.0, 0.0)),
            ..Default::default()
        },
        &mut point,
    );

    assert_eq!(
        render_buffer_point_collection(&collection, BLACK),
        [255, 255, 255, 255]
    );

    point.show = false;
    collection.set(point.index, &point);
    assert_eq!(render_buffer_point_collection(&collection, BLACK), BLACK);
}

#[test]
fn picks_points() {
    let mut collection = BufferPointCollection {
        allow_picking: true,
        ..Default::default()
    };

    let mut point = BufferPoint::default();
    let pos = Cartesian3::new(0.0, -1000.0, 0.0);
    collection.add(
        BufferPointOptions {
            position: Some(pos),
            ..Default::default()
        },
        &mut point,
    );
    collection.add(
        BufferPointOptions {
            position: Some(pos),
            ..Default::default()
        },
        &mut point,
    );

    assert_eq!(pick_buffer_point_collection(&collection), Some(0));
}

#[test]
fn drill_picks_points() {
    let mut collection = BufferPointCollection {
        allow_picking: true,
        ..Default::default()
    };

    let mut point = BufferPoint::default();
    let pos = Cartesian3::new(0.0, -1000.0, 0.0);
    let pos_bad = Cartesian3::new(-10e8, 0.0, 0.0);

    collection.add(
        BufferPointOptions {
            position: Some(pos),
            ..Default::default()
        },
        &mut point,
    );
    collection.add(
        BufferPointOptions {
            position: Some(pos_bad),
            ..Default::default()
        },
        &mut point,
    );
    collection.add(
        BufferPointOptions {
            position: Some(pos),
            ..Default::default()
        },
        &mut point,
    );

    assert_eq!(drill_pick_buffer_point_collection(&collection), vec![0, 2]);
}

#[test]
fn does_not_pick_if_picking_disabled() {
    let mut collection = BufferPointCollection {
        allow_picking: false,
        ..Default::default()
    };

    let mut point = BufferPoint::default();
    collection.add(
        BufferPointOptions {
            position: Some(Cartesian3::new(0.0, -1000.0, 0.0)),
            ..Default::default()
        },
        &mut point,
    );

    assert_eq!(pick_buffer_point_collection(&collection), None);
}

#[test]
fn does_not_pick_if_empty() {
    let collection = BufferPointCollection {
        allow_picking: true,
        ..Default::default()
    };

    assert_eq!(pick_buffer_point_collection(&collection), None);
}

#[test]
fn does_not_pick_if_collection_show_false() {
    let mut collection = BufferPointCollection {
        allow_picking: true,
        ..Default::default()
    };

    let mut point = BufferPoint::default();
    collection.add(
        BufferPointOptions {
            position: Some(Cartesian3::new(0.0, -1000.0, 0.0)),
            ..Default::default()
        },
        &mut point,
    );

    assert_eq!(pick_buffer_point_collection(&collection), Some(0));

    collection.show = false;
    assert_eq!(pick_buffer_point_collection(&collection), None);
}

#[test]
fn does_not_pick_if_point_show_false() {
    let mut collection = BufferPointCollection {
        allow_picking: true,
        ..Default::default()
    };

    let mut point = BufferPoint::default();
    let pos = Cartesian3::new(0.0, -1000.0, 0.0);
    collection.add(
        BufferPointOptions {
            position: Some(pos),
            ..Default::default()
        },
        &mut point,
    );
    collection.add(
        BufferPointOptions {
            position: Some(pos),
            ..Default::default()
        },
        &mut point,
    );

    assert_eq!(pick_buffer_point_collection(&collection), Some(0));

    collection.get(0, &mut point);
    point.show = false;
    collection.set(0, &point);

    assert_eq!(pick_buffer_point_collection(&collection), Some(1));

    collection.get(1, &mut point);
    point.show = false;
    collection.set(1, &point);

    assert_eq!(pick_buffer_point_collection(&collection), None);
}
