use cesium_viewer::widgets::NavigationHelpButtonViewModel;

#[test]
fn can_construct() {
    let view_model = NavigationHelpButtonViewModel::new();
    assert!(!view_model.show_instructions());
}

#[test]
fn invoking_command_toggles_showing() {
    let view_model = NavigationHelpButtonViewModel::new();
    assert!(!view_model.show_instructions());

    assert!(view_model.command().execute(()).is_ok());
    assert!(view_model.show_instructions());

    assert!(view_model.command().execute(()).is_ok());
    assert!(!view_model.show_instructions());
}
