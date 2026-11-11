#![allow(clippy::arc_with_non_send_sync)]
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use cesium_math::{
    BoundingSphere, Cartesian3, Clock, ClockOptions, ClockRange, ClockStep,
    ConstantPositionProperty, CustomDataSource, DataSourceClock, DataSourceCollection,
    EllipsoidTerrainProvider, Entity, EntityCollection, HeadingPitchRange, JulianDate, Matrix4,
    SceneMode,
};
use cesium_renderer::cesium_widget::{
    BaseLayerOption, CesiumWidget, CesiumWidgetOptions, FlyToOptions, GlobeOption,
    MapProjectionType, SkyAtmosphereOption, SkyBoxOption, WebglOptions, WidgetContextOptions,
    ZoomTarget,
};
use cesium_renderer::imagery_layer::{ImageryLayer, ImageryLayerOptions};
use cesium_renderer::sky_box::{SkyBox, SkyBoxOptions};

fn create_widget(options: Option<CesiumWidgetOptions>) -> CesiumWidget {
    CesiumWidget::new("container", options).expect("createCesiumWidget should succeed")
}

#[test]
fn test_can_create_render_and_destroy() {
    let mut widget = create_widget(None);
    assert!(!widget.is_destroyed());
    assert_eq!(widget.container.id, "container");
    assert_eq!(widget.canvas.width, 800);
    assert_eq!(widget.canvas.height, 600);
    assert_eq!(widget.credit_container, "creditContainer");
    assert_eq!(widget.credit_viewport, "creditViewport");
    assert_eq!(widget.scene.imagery_layers.length(), 1);
    assert!(widget.terrain_provider().is_some());
    assert!(widget.use_browser_recommended_resolution);

    widget.render().expect("render should succeed");
    widget.destroy();
    assert!(widget.is_destroyed());
}

#[test]
fn test_can_pass_id_string_for_container() {
    let widget = CesiumWidget::new("my-custom-container", None);
    assert!(widget.is_ok());
    assert_eq!(widget.unwrap().container.id, "my-custom-container");
}

#[test]
fn test_sets_expected_options_clock() {
    let mut custom_clock = Clock::new(ClockOptions::default()).unwrap();
    custom_clock.set_multiplier(42.0);

    let widget = create_widget(Some(CesiumWidgetOptions {
        clock: Some(custom_clock),
        ..Default::default()
    }));
    assert_eq!(widget.clock.multiplier(), 42.0);
}

#[test]
fn test_can_set_should_animate() {
    let widget = create_widget(Some(CesiumWidgetOptions {
        should_animate: Some(true),
        ..Default::default()
    }));
    assert!(widget.clock.should_animate());
}

#[test]
fn test_can_set_scene_mode_2d_and_columbus() {
    let mut widget_2d = create_widget(Some(CesiumWidgetOptions {
        scene_mode: Some(SceneMode::Scene2D),
        ..Default::default()
    }));
    widget_2d.scene.complete_morph();
    assert_eq!(widget_2d.scene.mode, SceneMode::Scene2D);

    let mut widget_cv = create_widget(Some(CesiumWidgetOptions {
        scene_mode: Some(SceneMode::ColumbusView),
        ..Default::default()
    }));
    widget_cv.scene.complete_morph();
    assert_eq!(widget_cv.scene.mode, SceneMode::ColumbusView);
}

#[test]
fn test_can_set_map_projection() {
    let widget = create_widget(Some(CesiumWidgetOptions {
        map_projection: Some(MapProjectionType::WebMercator),
        ..Default::default()
    }));
    assert_eq!(widget.scene.map_projection, MapProjectionType::WebMercator);
}

#[test]
fn test_can_disable_render_loop_and_set_target_frame_rate() {
    let widget = create_widget(Some(CesiumWidgetOptions {
        use_default_render_loop: Some(false),
        target_frame_rate: Some(23.0),
        ..Default::default()
    }));
    assert!(!widget.use_default_render_loop);
    assert_eq!(widget.target_frame_rate(), Some(23.0));
}

#[test]
fn test_sets_expected_options_base_layer() {
    let custom_layer = ImageryLayer::new(None, Some(ImageryLayerOptions::default()));
    let widget_layer = create_widget(Some(CesiumWidgetOptions {
        base_layer: Some(BaseLayerOption::Layer(custom_layer)),
        ..Default::default()
    }));
    assert_eq!(widget_layer.scene.imagery_layers.length(), 1);

    let widget_false = create_widget(Some(CesiumWidgetOptions {
        base_layer: Some(BaseLayerOption::False),
        ..Default::default()
    }));
    assert_eq!(widget_false.scene.imagery_layers.length(), 0);
}

#[test]
fn test_sets_expected_options_terrain_provider() {
    let mut widget = create_widget(Some(CesiumWidgetOptions {
        terrain_provider: Some(EllipsoidTerrainProvider::default()),
        ..Default::default()
    }));
    assert!(widget.terrain_provider().is_some());

    let another = EllipsoidTerrainProvider::default();
    widget.set_terrain_provider(another);
    assert!(widget.terrain_provider().is_some());
}

#[test]
fn test_does_not_create_globe_if_option_is_false() {
    let widget = create_widget(Some(CesiumWidgetOptions {
        globe: Some(GlobeOption::False),
        ..Default::default()
    }));
    assert!(widget.scene.globe.is_none());
    assert!(!widget.scene.sky_atmosphere.as_ref().unwrap().show);
}

#[test]
fn test_does_not_create_sky_box_if_option_is_false() {
    let widget = create_widget(Some(CesiumWidgetOptions {
        sky_box: Some(SkyBoxOption::False),
        ..Default::default()
    }));
    assert!(widget.scene.sky_box.is_none());
}

#[test]
fn test_does_not_create_sky_atmosphere_if_option_is_false() {
    let widget = create_widget(Some(CesiumWidgetOptions {
        sky_atmosphere: Some(SkyAtmosphereOption::False),
        ..Default::default()
    }));
    assert!(widget.scene.sky_atmosphere.is_none());
}

#[test]
fn test_sets_expected_options_sky_box() {
    let sky_box = SkyBox::new(SkyBoxOptions::default());
    let widget = create_widget(Some(CesiumWidgetOptions {
        sky_box: Some(SkyBoxOption::Box(sky_box)),
        ..Default::default()
    }));
    assert!(widget.scene.sky_box.is_some());
}

#[test]
fn test_data_sources_lifecycle_at_construction() {
    // Default collection is cleared when widget is destroyed
    let mut widget_default = create_widget(None);
    let ds_default = Arc::clone(&widget_default.data_sources);
    ds_default.lock().unwrap().add(CustomDataSource::new("ds1"));
    assert_eq!(ds_default.lock().unwrap().len(), 1);

    widget_default.destroy();
    assert_eq!(ds_default.lock().unwrap().len(), 0);

    // Specified collection is not cleared when widget is destroyed
    let external_ds = Arc::new(Mutex::new(DataSourceCollection::new()));
    external_ds.lock().unwrap().add(CustomDataSource::new("custom_ds"));

    let mut widget_custom = create_widget(Some(CesiumWidgetOptions {
        data_sources: Some(Arc::clone(&external_ds)),
        ..Default::default()
    }));
    widget_custom.destroy();
    assert_eq!(external_ds.lock().unwrap().len(), 1);
}

#[test]
fn test_can_set_context_options() {
    let webgl = WebglOptions {
        alpha: true,
        depth: true,
        stencil: true,
        antialias: false,
        premultiplied_alpha: true,
        preserve_drawing_buffer: true,
        power_preference: "low-power".to_string(),
        fail_if_major_performance_caveat: false,
    };
    let context_options = WidgetContextOptions {
        allow_texture_filter_anisotropic: false,
        webgl,
    };

    let widget = create_widget(Some(CesiumWidgetOptions {
        context_options: Some(context_options),
        ..Default::default()
    }));
    assert!(!widget.scene.context.allow_texture_filter_anisotropic);
    assert_eq!(widget.scene.context.webgl.power_preference, "low-power");
    assert!(widget.scene.context.webgl.alpha);
}

#[test]
fn test_can_disable_order_independent_translucency() {
    let widget = create_widget(Some(CesiumWidgetOptions {
        order_independent_translucency: Some(false),
        ..Default::default()
    }));
    assert!(!widget.scene.order_independent_translucency);
}

#[test]
fn test_can_enable_request_render_mode_and_maximum_render_time_change() {
    let widget = create_widget(Some(CesiumWidgetOptions {
        request_render_mode: Some(true),
        maximum_render_time_change: Some(f64::INFINITY),
        ..Default::default()
    }));
    assert!(widget.scene.request_render_mode);
    assert_eq!(widget.scene.maximum_render_time_change, f64::INFINITY);
}

#[test]
fn test_tracked_entity_management_and_events() {
    let mut widget = create_widget(None);

    let mut entity = Entity::new();
    entity.position = Some(ConstantPositionProperty::new(
        Some(Cartesian3::new(123456.0, 123456.0, 123456.0)),
        None,
    ));

    let received = Arc::new(Mutex::new(None));
    let r_clone = Arc::clone(&received);
    widget.tracked_entity_changed.add_listener(move |val| {
        *r_clone.lock().unwrap() = val.clone();
    });

    widget.set_tracked_entity(Some(entity.clone()));
    assert_eq!(widget.tracked_entity().unwrap().id, entity.id);
    assert_eq!(received.lock().unwrap().as_ref().unwrap().id, entity.id);

    widget.render().expect("render should succeed");
    assert_ne!(widget.scene.camera.transform, Matrix4::IDENTITY);

    // Stop tracking
    widget.set_tracked_entity(None);
    assert!(widget.tracked_entity().is_none());
    assert!(received.lock().unwrap().is_none());
    assert_eq!(widget.scene.camera.transform, Matrix4::IDENTITY);

    // Tracking entity with undefined position should not crash or raise render_error
    let mut undef_entity = Entity::new();
    undef_entity.position = None;
    widget.set_tracked_entity(Some(undef_entity));
    assert!(widget.render().is_ok());
}

#[test]
fn test_developer_error_validations() {
    // Empty container
    assert!(CesiumWidget::new("", None).is_err());

    // Non-existent container ID
    assert!(CesiumWidget::new("doesnotexist", None).is_err());

    let mut widget = create_widget(None);
    // targetFrameRate < 0
    assert!(widget.set_target_frame_rate(-1.0).is_err());

    // resolutionScale <= 0
    assert!(widget.set_resolution_scale(0.0).is_err());
    assert!(widget.set_resolution_scale(-0.5).is_err());
}

#[test]
fn test_suspends_animation_by_data_sources_if_allowed() {
    let mut widget = create_widget(None);
    widget.data_source_display.is_ready = true;
    widget.tick_clock();
    assert!(widget.clock.can_animate);

    widget.data_source_display.is_ready = false;
    widget.tick_clock();
    assert!(!widget.clock.can_animate);

    widget.clock.can_animate = true;
    widget.allow_data_sources_to_suspend_animation = false;
    widget.tick_clock();
    assert!(widget.clock.can_animate);
}

#[test]
fn test_sets_the_clock_based_on_first_data_source() {
    let mut ds = CustomDataSource::new("mock");
    let mut ds_clock = DataSourceClock::new();
    let start = JulianDate::from_iso8601("2013-08-01T18:00:00Z").unwrap();
    let stop = JulianDate::from_iso8601("2013-08-21T02:00:00Z").unwrap();
    let current = JulianDate::from_iso8601("2013-08-02T00:00:00Z").unwrap();

    ds_clock.start_time = Some(start);
    ds_clock.stop_time = Some(stop);
    ds_clock.current_time = Some(current);
    ds_clock.clock_range = Some(ClockRange::Clamped);
    ds_clock.clock_step = Some(ClockStep::TickDependent);
    ds_clock.multiplier = Some(20.0);
    ds.set_clock(Some(ds_clock));

    let mut widget = create_widget(None);
    widget.on_data_source_added(&ds);

    assert_eq!(widget.clock.start_time(), start);
    assert_eq!(widget.clock.stop_time(), stop);
    assert_eq!(widget.clock.current_time(), current);
    assert_eq!(widget.clock.clock_range(), ClockRange::Clamped);
    assert_eq!(widget.clock.clock_step(), ClockStep::TickDependent);
    assert_eq!(widget.clock.multiplier(), 20.0);
}

#[test]
fn test_sets_the_clock_for_multiple_data_sources() {
    let mut widget = create_widget(None);

    let mut ds1 = CustomDataSource::new("ds1");
    let mut clock1 = DataSourceClock::new();
    let start1 = JulianDate::from_iso8601("2013-08-01T18:00:00Z").unwrap();
    clock1.start_time = Some(start1);
    ds1.set_clock(Some(clock1));

    let mut ds2 = CustomDataSource::new("ds2");
    let mut clock2 = DataSourceClock::new();
    let start2 = JulianDate::from_iso8601("2014-08-01T18:00:00Z").unwrap();
    clock2.start_time = Some(start2);
    ds2.set_clock(Some(clock2));

    let mut ds3 = CustomDataSource::new("ds3");
    let mut clock3 = DataSourceClock::new();
    let start3 = JulianDate::from_iso8601("2015-08-01T18:00:00Z").unwrap();
    clock3.start_time = Some(start3);
    ds3.set_clock(Some(clock3));

    widget.data_sources.lock().unwrap().add(ds1.clone());
    widget.on_data_source_added(&ds1);
    assert_eq!(widget.clock.start_time(), start1);

    widget.data_sources.lock().unwrap().add(ds2.clone());
    widget.on_data_source_added(&ds2);
    assert_eq!(widget.clock.start_time(), start2);

    widget.data_sources.lock().unwrap().add(ds3.clone());
    widget.on_data_source_added(&ds3);
    assert_eq!(widget.clock.start_time(), start3);

    // Remove ds3 moves clock back to ds2
    widget.data_sources.lock().unwrap().remove(&ds3);
    widget.on_data_source_removed(&ds3);
    assert_eq!(widget.clock.start_time(), start2);

    // Removing ds1 has no effect because ds2 is active
    widget.data_sources.lock().unwrap().remove(&ds1);
    widget.on_data_source_removed(&ds1);
    assert_eq!(widget.clock.start_time(), start2);
}

#[test]
fn test_updates_the_clock_when_data_source_changes() {
    let mut widget = create_widget(None);
    let mut ds = CustomDataSource::new("ds");
    let mut clock = DataSourceClock::new();
    let start = JulianDate::from_iso8601("2013-08-01T18:00:00Z").unwrap();
    clock.start_time = Some(start);
    ds.set_clock(Some(clock));

    widget.on_data_source_added(&ds);
    assert_eq!(widget.clock.start_time(), start);

    let updated_start = JulianDate::from_iso8601("2014-08-01T18:00:00Z").unwrap();
    let mut updated_clock = DataSourceClock::new();
    updated_clock.start_time = Some(updated_start);
    ds.set_clock(Some(updated_clock));

    widget.on_data_source_changed(&ds);
    assert_eq!(widget.clock.start_time(), updated_start);
}

#[test]
fn test_can_manually_control_clock_tracking() {
    let mut widget = create_widget(Some(CesiumWidgetOptions {
        automatically_track_data_source_clocks: Some(false),
        ..Default::default()
    }));

    let mut ds1 = CustomDataSource::new("ds1");
    let mut clock1 = DataSourceClock::new();
    let start1 = JulianDate::from_iso8601("2013-08-01T18:00:00Z").unwrap();
    clock1.start_time = Some(start1);
    ds1.set_clock(Some(clock1));

    widget.on_data_source_added(&ds1);
    assert!(widget.clock_tracked_data_source.is_none());
    assert_ne!(widget.clock.start_time(), start1);

    // Manually set tracked data source
    widget.set_clock_tracked_data_source(Some(ds1.clone()));
    assert_eq!(widget.clock.start_time(), start1);

    let mut ds2 = CustomDataSource::new("ds2");
    let mut clock2 = DataSourceClock::new();
    let start2 = JulianDate::from_iso8601("2014-08-01T18:00:00Z").unwrap();
    clock2.start_time = Some(start2);
    ds2.set_clock(Some(clock2));

    widget.on_data_source_added(&ds2);
    assert_eq!(widget.clock.start_time(), start1);

    // Removing tracked data source in manual mode turns off tracking
    widget.on_data_source_removed(&ds1);
    assert!(widget.clock_tracked_data_source.is_none());
}

#[test]
fn test_resolution_scale_and_browser_recommended() {
    let mut widget = create_widget(Some(CesiumWidgetOptions {
        use_default_render_loop: Some(false),
        use_browser_recommended_resolution: Some(true),
        ..Default::default()
    }));
    widget.device_pixel_ratio = 2.0;
    widget.set_resolution_scale(0.5).unwrap();

    // With use_browser_recommended_resolution = true, pixelRatio = scale
    assert_eq!(widget.scene.pixel_ratio, 0.5);

    // With use_browser_recommended_resolution = false, pixelRatio = device_pixel_ratio * scale
    widget.use_browser_recommended_resolution = false;
    widget.resize();
    assert_eq!(widget.scene.pixel_ratio, 1.0);
}

#[test]
fn test_resizing_triggers_render_in_request_render_mode() {
    let mut widget = create_widget(Some(CesiumWidgetOptions {
        request_render_mode: Some(true),
        maximum_render_time_change: Some(f64::INFINITY),
        ..Default::default()
    }));

    let initial_count = widget.scene.request_render_count;
    widget.resize();
    assert_eq!(widget.scene.request_render_count, initial_count);

    widget.force_resize = true;
    widget.resize();
    assert_eq!(widget.scene.request_render_count, initial_count + 1);
}

#[test]
fn test_render_error_stops_render_loop_and_shows_error_panel() {
    let mut widget = create_widget(None);
    assert!(widget.use_default_render_loop);

    widget.scene.primitives.update_hook = Some(Arc::new(|| Err("render failure simulation".to_string())));

    let render_res = widget.render();
    assert!(render_res.is_err());
    assert!(!widget.use_default_render_loop);
    assert!(widget.error_panel.is_some());
    assert!(widget.error_panel.as_ref().unwrap().message.contains("render failure simulation"));

    // Dismiss error panel
    widget.dismiss_error_panel();
    assert!(widget.error_panel.is_none());

    // When show_render_loop_errors is false
    let mut widget_no_error_panel = create_widget(Some(CesiumWidgetOptions {
        show_render_loop_errors: Some(false),
        ..Default::default()
    }));
    widget_no_error_panel.scene.primitives.update_hook = Some(Arc::new(|| Err("render failure simulation".to_string())));
    assert!(widget_no_error_panel.render().is_err());
    assert!(!widget_no_error_panel.use_default_render_loop);
    assert!(widget_no_error_panel.error_panel.is_none());
}

#[test]
fn test_zoom_to_and_fly_to_targets() {
    let mut widget = create_widget(None);

    // 1. Tileset zoomTo
    let tileset_sphere = BoundingSphere::new(Cartesian3::new(10.0, 20.0, 30.0), 50.0);
    widget.zoom_to(ZoomTarget::Tileset { bounding_sphere: tileset_sphere }, None).unwrap();
    widget.post_render();
    assert_eq!(widget.scene.camera.view_bounding_sphere_call_count, 1);
    let (bs, offset) = widget.scene.camera.last_view_bounding_sphere.unwrap();
    assert_eq!(bs, tileset_sphere);
    assert_eq!(offset.unwrap(), HeadingPitchRange::new(0.0, -0.5, 50.0));

    // 2. Tileset flyTo with custom options
    let custom_hpr = HeadingPitchRange::new(0.4, 1.2, 200.0);
    let completed = Arc::new(AtomicBool::new(false));
    let c_clone = Arc::clone(&completed);
    widget.fly_to(
        ZoomTarget::Tileset { bounding_sphere: tileset_sphere },
        Some(FlyToOptions {
            offset: Some(custom_hpr),
            duration: Some(3.0),
            maximum_height: Some(10.0),
            complete: Some(Arc::new(move || {
                c_clone.store(true, Ordering::Relaxed);
            })),
            ..Default::default()
        }),
    ).unwrap();
    widget.post_render();
    assert_eq!(widget.scene.camera.fly_to_bounding_sphere_call_count, 1);
    assert!(completed.load(Ordering::Relaxed));

    // 3. PointCloud zoomTo
    let pc_sphere = BoundingSphere::new(Cartesian3::ZERO, 25.0);
    widget.zoom_to(ZoomTarget::PointCloud { bounding_sphere: Some(pc_sphere) }, None).unwrap();
    widget.post_render();
    assert_eq!(widget.scene.camera.view_bounding_sphere_call_count, 2);

    // 4. Voxel zoomTo & flyTo
    let voxel_sphere = BoundingSphere::new(Cartesian3::new(1.0, 2.0, 3.0), 10.0);
    widget.fly_to(ZoomTarget::Voxel { bounding_sphere: voxel_sphere }, None).unwrap();
    widget.post_render();
    assert_eq!(widget.scene.camera.fly_to_bounding_sphere_call_count, 2);

    // 5. BufferPrimitiveCollection zoomTo
    let buffer_sphere = BoundingSphere::new(Cartesian3::new(5.0, 5.0, 5.0), 15.0);
    widget.zoom_to(ZoomTarget::BufferCollection { bounding_volume: buffer_sphere }, None).unwrap();
    widget.post_render();
    assert_eq!(widget.scene.camera.view_bounding_sphere_call_count, 3);

    // 6. Entity zoomTo and flyTo
    let mut entity = Entity::new();
    entity.position = Some(ConstantPositionProperty::new(
        Some(Cartesian3::new(100.0, 200.0, 300.0)),
        None,
    ));
    widget.zoom_to(ZoomTarget::Entity(entity.clone()), None).unwrap();
    widget.post_render();
    assert_eq!(widget.scene.camera.view_bounding_sphere_call_count, 4);

    // 7. Entities collection zoomTo
    let mut entity_collection = EntityCollection::new();
    let _ = entity_collection.add(entity.clone());
    widget.zoom_to(ZoomTarget::EntityCollection(entity_collection), None).unwrap();
    widget.post_render();
    assert_eq!(widget.scene.camera.view_bounding_sphere_call_count, 5);

    // 8. ImageryLayer flyTo
    let center = Cartesian3::new(1000.0, 2000.0, 3000.0);
    widget.fly_to(ZoomTarget::ImageryLayer { rectangle_center: center }, Some(FlyToOptions {
        duration: Some(0.0),
        ..Default::default()
    })).unwrap();
    widget.post_render();
    assert_eq!(widget.scene.camera.fly_to_call_count, 1);
    assert_eq!(widget.scene.camera.position, center);

    // 9. ZoomTo entity when globe is disabled
    let mut widget_noglobe = create_widget(Some(CesiumWidgetOptions {
        globe: Some(GlobeOption::False),
        should_animate: Some(true),
        ..Default::default()
    }));
    assert!(widget_noglobe.zoom_to(ZoomTarget::Entity(entity), None).is_ok());
    widget_noglobe.post_render();
    assert!(widget_noglobe.scene.globe.is_none());
}
