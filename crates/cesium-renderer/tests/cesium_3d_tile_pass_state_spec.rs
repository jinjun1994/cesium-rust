use cesium_renderer::{
    Cesium3DTilePass, Cesium3DTilePassState, Cesium3DTilePassStateOptions,
};

#[test]
fn test_sets_default_values() {
    let pass_state: Cesium3DTilePassState = Cesium3DTilePassState::new(Some(
        Cesium3DTilePassStateOptions {
            pass: Some(Cesium3DTilePass::Render),
            command_list: None,
            camera: None,
            culling_volume: None,
        },
    ))
    .unwrap();

    assert_eq!(pass_state.pass, Cesium3DTilePass::Render);
    assert_eq!(pass_state.command_list, None);
    assert_eq!(pass_state.camera, None);
    assert_eq!(pass_state.culling_volume, None);
    assert!(!pass_state.ready);
}

#[test]
fn test_constructed_with_options() {
    let mock_command_list = vec!["cmd1", "cmd2"];
    let mock_camera = "camera";
    let mock_culling_volume = "culling_volume";

    let pass_state = Cesium3DTilePassState::new(Some(Cesium3DTilePassStateOptions {
        pass: Some(Cesium3DTilePass::Render),
        command_list: Some(mock_command_list.clone()),
        camera: Some(mock_camera),
        culling_volume: Some(mock_culling_volume),
    }))
    .unwrap();

    assert_eq!(pass_state.pass, Cesium3DTilePass::Render);
    assert_eq!(pass_state.command_list, Some(mock_command_list));
    assert_eq!(pass_state.camera, Some(mock_camera));
    assert_eq!(pass_state.culling_volume, Some(mock_culling_volume));
    assert!(!pass_state.ready);
}

#[test]
fn test_throws_if_options_undefined() {
    let res: Result<Cesium3DTilePassState, _> = Cesium3DTilePassState::new(None);
    assert!(res.is_err());
}

#[test]
fn test_throws_if_options_pass_undefined() {
    let res: Result<Cesium3DTilePassState, _> =
        Cesium3DTilePassState::new(Some(Cesium3DTilePassStateOptions::default()));
    assert!(res.is_err());
}

#[test]
fn test_pass_attributes() {
    assert!(Cesium3DTilePass::Render.is_render());
    assert!(!Cesium3DTilePass::Pick.is_render());

    assert!(Cesium3DTilePass::Render.request_tiles());
    assert!(!Cesium3DTilePass::Pick.request_tiles());
    assert!(!Cesium3DTilePass::MostDetailedPick.request_tiles());

    assert!(!Cesium3DTilePass::Render.ignore_commands());
    assert!(Cesium3DTilePass::Preload.ignore_commands());
    assert!(Cesium3DTilePass::PreloadFlight.ignore_commands());
    assert!(Cesium3DTilePass::RequestRenderModeDeferCheck.ignore_commands());
    assert!(Cesium3DTilePass::MostDetailedPreload.ignore_commands());
}
